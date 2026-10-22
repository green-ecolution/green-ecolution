use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "geco",
    version,
    about = "Developer console for the Green Ecolution stack"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Profile to start when no supervisor is running (default from geco.toml)
    #[arg(long, short, global = true)]
    pub profile: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Start the profile's units; runs in the foreground unless --detach is given
    Up {
        #[arg(long)]
        detach: bool,
    },
    /// Stop the supervisor and all of its units
    Down,
    /// Show unit states; exits with 1 when a unit is not ready
    Status {
        #[arg(long)]
        json: bool,
    },
    /// Print logs of the given units (all units when none are given)
    Logs {
        units: Vec<String>,
        #[arg(short, long)]
        follow: bool,
        #[arg(long)]
        json: bool,
        #[arg(short = 'n', long, default_value_t = 200)]
        tail: usize,
    },
    /// Restart a unit
    Restart {
        unit: String,
        /// Also restart every unit that depends on it
        #[arg(long)]
        cascade: bool,
    },
    /// Start a stopped unit
    Start { unit: String },
    /// Stop a unit
    Stop { unit: String },
    #[command(name = "__supervise", hide = true)]
    Supervise,
}
