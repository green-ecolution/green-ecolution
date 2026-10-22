use std::process::ExitCode;

use anyhow::bail;

use super::output;
use crate::{
    config::GecoConfig,
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
            Err(IpcError::NotRunning) => ProfileList {
                active: String::new(),
                available: GecoConfig::load(&paths.config())?
                    .profile
                    .into_keys()
                    .collect(),
            },
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
