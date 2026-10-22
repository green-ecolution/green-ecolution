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
        /// Filter expression, e.g. 'level>=warn target:sqlx "jwks"'. Without a running supervisor, -n counts lines before filtering.
        #[arg(short = 'F', long, default_value = "")]
        filter: String,
        #[arg(short, long)]
        follow: bool,
        #[arg(long)]
        json: bool,
        #[arg(short = 'n', long, default_value_t = 200)]
        tail: usize,
    },
    /// Show every log record of one request, indented by span
    Trace {
        /// Full request id or a unique prefix
        request_id: String,
        #[arg(long)]
        json: bool,
    },
    /// Diagnose known pitfalls of the local setup; exits with 1 while an error is open
    Doctor {
        /// Run the fixes; destructive ones ask first
        #[arg(long)]
        fix: bool,
        /// Also run the slow checks (cargo sqlx prepare --check)
        #[arg(long)]
        deep: bool,
        #[arg(long, conflicts_with = "fix")]
        json: bool,
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

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    #[test]
    fn logs_take_units_and_a_filter() {
        let cli =
            Cli::try_parse_from(["geco", "logs", "backend", "-F", "level>=warn", "-f"]).unwrap();
        let Some(Command::Logs {
            units,
            filter,
            follow,
            ..
        }) = cli.command
        else {
            panic!("expected logs");
        };
        assert_eq!(units, ["backend"]);
        assert_eq!(filter, "level>=warn");
        assert!(follow);
    }

    #[test]
    fn trace_takes_a_request_id() {
        let cli = Cli::try_parse_from(["geco", "trace", "1c40b9b5", "--json"]).unwrap();
        assert!(matches!(
            cli.command,
            Some(Command::Trace { request_id, json: true }) if request_id == "1c40b9b5"
        ));
    }

    #[test]
    fn doctor_takes_fix_and_deep_but_not_json_with_fix() {
        let cli = Cli::try_parse_from(["geco", "doctor", "--fix", "--deep"]).unwrap();
        assert!(matches!(
            cli.command,
            Some(Command::Doctor {
                fix: true,
                deep: true,
                json: false
            })
        ));
        assert!(Cli::try_parse_from(["geco", "doctor", "--fix", "--json"]).is_err());
    }
}
