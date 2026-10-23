use std::{
    fs,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpStream},
    process::Command,
    time::Duration,
};

use async_trait::async_trait;

use crate::doctor::{Check, Env, Finding, Severity, Trigger};

/// The fixed host ports of the local stack (Postgres, Traefik, backend, Vite), each with the
/// unit that binds it.
const PORTS: [(u16, &str); 4] = [
    (5432, "db"),
    (3000, "reverse-proxy"),
    (3030, "backend"),
    (5173, "frontend"),
];

pub fn is_taken(port: u16) -> bool {
    [
        IpAddr::V4(Ipv4Addr::LOCALHOST),
        IpAddr::V6(Ipv6Addr::LOCALHOST),
    ]
    .into_iter()
    .any(|ip| {
        TcpStream::connect_timeout(&SocketAddr::new(ip, port), Duration::from_millis(300)).is_ok()
    })
}

/// Socket inodes listening on `port` in a `/proc/net/tcp` or `/proc/net/tcp6` table.
pub fn listening_inodes(table: &str, port: u16) -> Vec<u64> {
    table
        .lines()
        .skip(1)
        .filter_map(|line| {
            let columns: Vec<&str> = line.split_whitespace().collect();
            let (_, hex_port) = columns.get(1)?.rsplit_once(':')?;
            let listening = *columns.get(3)? == "0A";
            let matches = u16::from_str_radix(hex_port, 16).ok()? == port;
            (listening && matches)
                .then(|| columns.get(9)?.parse().ok())
                .flatten()
        })
        .collect()
}

/// `lsof -F pc` prints a `p<pid>` line followed by a `c<command>` line.
pub fn parse_lsof(output: &str) -> Option<(u32, String)> {
    let mut pid = None;
    for line in output.lines() {
        if let Some(value) = line.strip_prefix('p') {
            pid = value.parse().ok();
        } else if let (Some(name), Some(pid)) = (line.strip_prefix('c'), pid) {
            return Some((pid, name.to_owned()));
        }
    }
    None
}

/// Sockets of other users' processes (docker-proxy runs as root) are not readable; the
/// owner then stays unknown.
fn owner_from_proc(port: u16) -> Option<(u32, String)> {
    let inodes: Vec<String> = ["/proc/net/tcp", "/proc/net/tcp6"]
        .iter()
        .filter_map(|table| fs::read_to_string(table).ok())
        .flat_map(|table| listening_inodes(&table, port))
        .map(|inode| format!("socket:[{inode}]"))
        .collect();
    if inodes.is_empty() {
        return None;
    }
    for entry in fs::read_dir("/proc").ok()?.flatten() {
        let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() else {
            continue;
        };
        let Ok(fds) = fs::read_dir(entry.path().join("fd")) else {
            continue;
        };
        let holds = fds.flatten().any(|fd| {
            fs::read_link(fd.path()).is_ok_and(|target| {
                inodes
                    .iter()
                    .any(|inode| target.as_os_str() == inode.as_str())
            })
        });
        if holds {
            let name = fs::read_to_string(entry.path().join("comm")).unwrap_or_default();
            return Some((pid, name.trim().to_owned()));
        }
    }
    None
}

fn owner_from_lsof(port: u16) -> Option<(u32, String)> {
    let output = Command::new("lsof")
        .args(["-nP", &format!("-iTCP:{port}"), "-sTCP:LISTEN", "-Fpc"])
        .output()
        .ok()?;
    parse_lsof(&String::from_utf8_lossy(&output.stdout))
}

fn owner(port: u16) -> Option<(u32, String)> {
    if cfg!(target_os = "linux") {
        owner_from_proc(port)
    } else {
        owner_from_lsof(port)
    }
}

pub fn port_finding(port: u16, unit: &str, owner: Option<(u32, String)>) -> Finding {
    let holder = match owner {
        Some((pid, name)) => format!("{name} (pid {pid})"),
        None => "another process".to_owned(),
    };
    Finding::new(
        "ports",
        Severity::Error,
        format!("port {port} is taken by {holder}"),
    )
    .subject(&port.to_string())
    .unit(unit)
    .detail(format!(
        "geco cannot start `{unit}` while port {port} is in use. Stop that process first; \
             there is no automatic fix."
    ))
}

pub struct Ports;

#[async_trait]
impl Check for Ports {
    fn id(&self) -> &'static str {
        "ports"
    }

    async fn diagnose(&self, env: &Env) -> Result<Vec<Finding>, String> {
        let idle: Vec<(u16, &str)> = PORTS
            .into_iter()
            .filter(|(_, unit)| !env.runs(unit))
            .collect();
        if idle.is_empty() {
            return Ok(Vec::new());
        }
        // A container of our own project that is already up (e.g. after `just infra-up`)
        // holds its port legitimately.
        let running = env
            .compose
            .run(&["ps", "--status", "running", "--services"])
            .await
            .unwrap_or_default();
        let candidates: Vec<(u16, String)> = idle
            .into_iter()
            .filter(|(_, unit)| !running.lines().any(|service| service.trim() == *unit))
            .map(|(port, unit)| (port, unit.to_owned()))
            .collect();
        tokio::task::spawn_blocking(move || {
            candidates
                .into_iter()
                .filter(|(port, _)| is_taken(*port))
                .map(|(port, unit)| port_finding(port, &unit, owner(port)))
                .collect()
        })
        .await
        .map_err(|err| err.to_string())
    }

    fn reacts_to(&self, trigger: &Trigger<'_>) -> bool {
        matches!(trigger, Trigger::Failed { unit } if PORTS.iter().any(|(_, owner)| owner == unit))
    }
}

#[cfg(test)]
mod tests {
    use std::net::TcpListener;

    use super::*;

    const TABLE: &str = "  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode
   0: 0100007F:1538 00000000:0000 0A 00000000:00000000 00:00000000 00000000  1000        0 4711 1 0000000000000000 100 0 0 10 0
   1: 0100007F:1538 0100007F:A2C4 01 00000000:00000000 00:00000000 00000000  1000        0 4712 1 0000000000000000 20 4 30 10 -1
   2: 00000000:0BB8 00000000:0000 0A 00000000:00000000 00:00000000 00000000     0        0 815 1 0000000000000000 100 0 0 10 0";

    #[test]
    fn reads_listening_sockets_from_proc_net_tcp() {
        assert_eq!(listening_inodes(TABLE, 5432), [4711]);
        assert_eq!(listening_inodes(TABLE, 3000), [815]);
        assert!(listening_inodes(TABLE, 3030).is_empty());
    }

    #[test]
    fn reads_the_owner_from_lsof_field_output() {
        assert_eq!(
            parse_lsof("p4242\ncpostgres\nf7\n"),
            Some((4242, "postgres".to_owned()))
        );
        assert_eq!(parse_lsof(""), None);
    }

    #[test]
    fn detects_a_taken_port() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        assert!(is_taken(port));
        drop(listener);
    }

    #[test]
    fn findings_name_the_holder_and_the_unit() {
        let known = port_finding(5432, "db", Some((4242, "postgres".into())));
        assert_eq!(known.id, "ports:5432");
        assert_eq!(known.unit.as_deref(), Some("db"));
        assert_eq!(known.title, "port 5432 is taken by postgres (pid 4242)");
        assert!(known.fix.is_none());

        let unknown = port_finding(3030, "backend", None);
        assert_eq!(unknown.title, "port 3030 is taken by another process");
    }

    #[test]
    fn reacts_to_failures_of_units_that_own_a_port() {
        assert!(Ports.reacts_to(&Trigger::Failed { unit: "backend" }));
        assert!(!Ports.reacts_to(&Trigger::Failed { unit: "migrations" }));
    }
}
