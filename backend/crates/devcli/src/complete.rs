//! Candidates for the dynamic shell completion (`COMPLETE=zsh geco`); they run on every Tab
//! press, so a missing repository or supervisor just yields fewer candidates.

use std::{
    collections::BTreeSet,
    io::{self, BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    path::Path,
    time::Duration,
};

use clap_complete::{CompletionCandidate, env::Shells};

use crate::{
    config::GecoConfig,
    ipc::{Call, PROTOCOL_VERSION, Reply, Request},
    paths::Paths,
    supervisor::UnitStatus,
    tasks::{Catalog, JustRecipes, RecipeSource},
};

type Named = Vec<(String, Option<String>)>;

pub fn units() -> Vec<CompletionCandidate> {
    from_cwd(unit_names)
}

pub fn profiles() -> Vec<CompletionCandidate> {
    from_cwd(profile_names)
}

pub fn recipes() -> Vec<CompletionCandidate> {
    from_cwd(|paths| {
        let tasks = GecoConfig::load(&paths.config())
            .map(|config| config.tasks)
            .unwrap_or_default();
        JustRecipes::new(paths.root.clone())
            .dump()
            .ok()
            .and_then(|dump| Catalog::parse(&dump, &tasks).ok())
            .map(|catalog| recipe_names(&catalog))
            .unwrap_or_default()
    })
}

fn from_cwd(names: impl Fn(&Paths) -> Named) -> Vec<CompletionCandidate> {
    let Some(paths) = std::env::current_dir()
        .ok()
        .and_then(|cwd| Paths::discover(&cwd))
    else {
        return Vec::new();
    };
    names(&paths)
        .into_iter()
        .map(|(name, help)| CompletionCandidate::new(name).help(help.map(Into::into)))
        .collect()
}

/// The running supervisor knows the units exactly, tasks included; otherwise every unit
/// `geco.toml` names, which leaves out compose services no profile mentions.
pub fn unit_names(paths: &Paths) -> Named {
    if let Some(units) = running_units(&paths.socket()) {
        return units
            .into_iter()
            .map(|unit| (unit.name, Some(unit.state.short().to_owned())))
            .collect();
    }
    let Ok(config) = GecoConfig::load(&paths.config()) else {
        return Vec::new();
    };
    let names: BTreeSet<String> = config
        .unit
        .iter()
        .filter(|(_, unit)| unit.kind.is_some())
        .map(|(name, _)| name.clone())
        .chain(
            config
                .profile
                .values()
                .flat_map(|profile| profile.units.iter().cloned()),
        )
        .collect();
    names.into_iter().map(|name| (name, None)).collect()
}

fn running_units(socket: &Path) -> Option<Vec<UnitStatus>> {
    let mut stream = UnixStream::connect(socket).ok()?;
    stream
        .set_read_timeout(Some(Duration::from_millis(300)))
        .ok()?;
    let mut line = serde_json::to_vec(&Request {
        v: PROTOCOL_VERSION,
        call: Call::Status,
    })
    .ok()?;
    line.push(b'\n');
    stream.write_all(&line).ok()?;
    let mut reply = String::new();
    BufReader::new(stream).read_line(&mut reply).ok()?;
    match serde_json::from_str(&reply).ok()? {
        Reply::Status(units) => Some(units),
        _ => None,
    }
}

pub fn recipe_names(catalog: &Catalog) -> Named {
    catalog
        .recipes()
        .filter(|recipe| !recipe.private)
        .map(|recipe| (recipe.name.clone(), recipe.doc.clone()))
        .chain(
            catalog
                .aliases()
                .map(|(alias, target)| (alias.to_owned(), Some(format!("alias for {target}")))),
        )
        .collect()
}

pub fn profile_names(paths: &Paths) -> Named {
    GecoConfig::load(&paths.config())
        .map(|config| {
            config
                .profile
                .into_iter()
                .map(|(name, profile)| {
                    (
                        name,
                        profile.extends.map(|parent| format!("extends {parent}")),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The script `COMPLETE=<shell> geco` prints; it calls `geco` from the PATH on every Tab.
pub fn write_registration(shell: &str, out: &mut dyn Write) -> io::Result<()> {
    let shells = Shells::builtins();
    let completer = shells
        .completer(shell)
        .ok_or_else(|| io::Error::other(format!("unknown shell `{shell}`")))?;
    completer.write_registration("COMPLETE", "geco", "geco", "geco", out)
}

#[cfg(test)]
mod tests {
    use std::{
        io::{BufRead, BufReader, Write},
        os::unix::net::UnixListener,
    };

    use chrono::Utc;

    use super::*;
    use crate::{
        ipc::Reply,
        supervisor::{UnitState, UnitStatus},
        tasks::catalog::tests::catalog,
        units::{Origin, UnitKind},
    };

    const CONFIG: &str = "[unit.backend]\nkind = \"process\"\nrun = \"bacon\"\n\
        [unit.reverse-proxy]\npre_start = [\"just _acme-init\"]\n\
        [profile.minimal]\nunits = [\"db\", \"backend\"]\n\
        [profile.full]\nextends = \"minimal\"\nunits = [\"valhalla\"]\n";

    fn repo() -> (tempfile::TempDir, Paths) {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("geco.toml"), CONFIG).unwrap();
        let paths = Paths::new(dir.path().to_path_buf());
        (dir, paths)
    }

    fn names(candidates: Vec<(String, Option<String>)>) -> Vec<String> {
        candidates.into_iter().map(|(name, _)| name).collect()
    }

    #[test]
    fn without_a_supervisor_units_come_from_geco_toml() {
        let (_dir, paths) = repo();
        assert_eq!(names(unit_names(&paths)), ["backend", "db", "valhalla"]);
    }

    #[test]
    fn a_running_supervisor_names_its_units() {
        let (_dir, paths) = repo();
        std::fs::create_dir_all(paths.state_dir()).unwrap();
        let listener = UnixListener::bind(paths.socket()).unwrap();
        let server = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut request = String::new();
            BufReader::new(&stream).read_line(&mut request).unwrap();
            assert!(request.contains("\"status\""), "{request}");
            let unit = UnitStatus {
                name: "migrate-up".into(),
                kind: UnitKind::Oneshot,
                origin: Origin::Task,
                state: UnitState::Starting,
                since: Utc::now(),
            };
            let mut reply = serde_json::to_vec(&Reply::Status(vec![unit])).unwrap();
            reply.push(b'\n');
            (&stream).write_all(&reply).unwrap();
        });

        let units = unit_names(&paths);
        server.join().unwrap();
        assert_eq!(
            units,
            [("migrate-up".to_owned(), Some("starting".to_owned()))]
        );
    }

    #[test]
    fn recipes_leave_out_private_ones_and_include_aliases() {
        let recipes = recipe_names(&catalog());
        let names: Vec<&str> = recipes.iter().map(|(name, _)| name.as_str()).collect();
        assert!(names.contains(&"migrate-up"));
        assert!(names.contains(&"fe-dev"));
        assert!(!names.contains(&"_compose"));
        let up = recipes
            .iter()
            .find(|(name, _)| name == "migrate-up")
            .unwrap();
        assert_eq!(up.1.as_deref(), Some("Apply all pending migrations"));
    }

    #[test]
    fn profiles_come_from_geco_toml() {
        let (_dir, paths) = repo();
        assert_eq!(names(profile_names(&paths)), ["full", "minimal"]);
    }

    #[test]
    fn registration_scripts_call_geco_back() {
        let mut out = Vec::new();
        write_registration("zsh", &mut out).unwrap();
        let script = String::from_utf8(out).unwrap();
        assert!(script.contains("geco"), "{script}");
        assert!(write_registration("cmd.exe", &mut Vec::new()).is_err());
    }
}
