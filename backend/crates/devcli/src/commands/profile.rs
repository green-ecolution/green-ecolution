use std::process::ExitCode;

use anyhow::bail;

use super::output;
use crate::{
    config::{ConfigError, GecoConfig},
    ipc::{Call, Client, IpcError, Reply},
    paths::Paths,
    profiles::ProfileList,
};

pub async fn profile(
    paths: &Paths,
    client: &Client,
    name: Option<String>,
) -> anyhow::Result<ExitCode> {
    let mut out = anstream::stdout();
    let Some(name) = name else {
        let list = match client.call(Call::Profiles).await {
            Ok(Reply::Profiles(list)) => list,
            Ok(other) => bail!("unexpected reply from the supervisor: {other:?}"),
            Err(IpcError::NotRunning) => configured(paths)?,
            Err(err) => return Err(err.into()),
        };
        output::write_profile_list(&mut out, &list)?;
        return Ok(ExitCode::SUCCESS);
    };
    match client
        .call(Call::SwitchProfile {
            profile: name.clone(),
        })
        .await
    {
        Ok(Reply::Switched(change)) => output::write_profile_change(&mut out, &change)?,
        Ok(other) => bail!("unexpected reply from the supervisor: {other:?}"),
        Err(IpcError::NotRunning) => {
            bail!("no geco supervisor is running; start one with `geco up -p {name}`")
        }
        Err(err) => return Err(err.into()),
    }
    Ok(ExitCode::SUCCESS)
}

/// Without a supervisor, the profile `geco up` would start counts as active.
fn configured(paths: &Paths) -> Result<ProfileList, ConfigError> {
    let config = GecoConfig::load(&paths.config())?;
    Ok(ProfileList {
        active: config.default_profile,
        available: config.profile.into_keys().collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn without_a_supervisor_the_default_profile_is_active() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("geco.toml"),
            "default_profile = \"full\"\n[profile.full]\nunits = []\n[profile.minimal]\nunits = []\n",
        )
        .unwrap();
        let list = configured(&Paths::new(dir.path().to_path_buf())).unwrap();
        assert_eq!(list.active, "full");
        assert_eq!(list.available, ["full", "minimal"]);
    }
}
