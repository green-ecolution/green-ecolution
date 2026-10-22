use std::process::ExitCode;

use clap::Parser;
use devcli::{cli::Cli, commands};

#[tokio::main]
async fn main() -> ExitCode {
    match commands::run(Cli::parse()).await {
        Ok(code) => code,
        Err(err) => {
            anstream::eprintln!("error: {err:#}");
            ExitCode::FAILURE
        }
    }
}
