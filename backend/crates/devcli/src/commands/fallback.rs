use std::{io::Write, process::ExitCode};

use anyhow::Context;
use bollard::{Docker, query_parameters::LogsOptionsBuilder};
use chrono::Utc;
use futures::StreamExt;
use tokio::sync::mpsc;

use super::output;
use crate::{
    config::{GecoConfig, LogFormat},
    logs::parse_line,
    paths::Paths,
    units::compose::{ComposeCli, find_container, service_overview},
};

async fn docker_and_project(paths: &Paths) -> anyhow::Result<(Docker, String)> {
    let docker = Docker::connect_with_local_defaults().context("cannot connect to Docker")?;
    docker.ping().await.context("Docker is not reachable")?;
    let project = ComposeCli::new(paths.root.clone()).project().await?;
    Ok((docker, project.name))
}

pub async fn status(paths: &Paths, json: bool) -> anyhow::Result<ExitCode> {
    let (docker, project) = docker_and_project(paths).await?;
    let services = service_overview(&docker, &project).await?;
    let mut out = anstream::stdout();
    if json {
        writeln!(out, "{}", serde_json::to_string_pretty(&services)?)?;
    } else {
        anstream::eprintln!("No geco supervisor is running; showing compose containers only.");
        for (service, status) in &services {
            writeln!(out, "  {service:<20} {status}")?;
        }
    }
    Ok(ExitCode::FAILURE)
}

pub async fn logs(
    paths: &Paths,
    units: &[String],
    tail: usize,
    follow: bool,
    json: bool,
) -> anyhow::Result<ExitCode> {
    let config = GecoConfig::load(&paths.config())?;
    let (docker, project) = docker_and_project(paths).await?;
    let services: Vec<String> = if units.is_empty() {
        service_overview(&docker, &project)
            .await?
            .into_keys()
            .collect()
    } else {
        units.to_vec()
    };

    let (tx, mut rx) = mpsc::unbounded_channel();
    for service in services {
        if config
            .unit
            .get(&service)
            .is_some_and(|unit| unit.kind.is_some())
        {
            anstream::eprintln!(
                "`{service}` is a local process; its logs only exist while a geco supervisor runs"
            );
            continue;
        }
        let id = find_container(&docker, &project, &service).await?;
        let options = LogsOptionsBuilder::new()
            .follow(follow)
            .stdout(true)
            .stderr(true)
            .tail(&tail.to_string())
            .build();
        let docker = docker.clone();
        let tx = tx.clone();
        tokio::spawn(async move {
            let mut stream = docker.logs(&id, Some(options));
            while let Some(chunk) = stream.next().await {
                if tx.send((service.clone(), chunk)).is_err() {
                    break;
                }
            }
        });
    }
    drop(tx);

    let mut out = anstream::stdout();
    while let Some((service, chunk)) = rx.recv().await {
        let bytes = chunk?.into_bytes();
        for line in String::from_utf8_lossy(&bytes).lines() {
            let record = parse_line(&service, LogFormat::Plain, line, Utc::now());
            if json {
                writeln!(out, "{}", serde_json::to_string(&record)?)?;
            } else {
                output::write_log(&mut out, &record)?;
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}
