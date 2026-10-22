use std::process::ExitCode;

use clap::{CommandFactory, Parser};
use clap_complete::CompleteEnv;
use devcli::{cli::Cli, commands};

fn main() -> ExitCode {
    // Before the runtime exists, so a Tab press never builds one.
    CompleteEnv::with_factory(Cli::command).complete();
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(err) => {
            anstream::eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };
    match runtime.block_on(commands::run(Cli::parse())) {
        Ok(code) => code,
        Err(err) => {
            anstream::eprintln!("error: {err:#}");
            ExitCode::FAILURE
        }
    }
}
