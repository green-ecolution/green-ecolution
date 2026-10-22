use clap::{Parser, Subcommand};
use clap_complete::ArgValueCandidates;

use crate::complete;

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
    #[arg(long, short, global = true, add = ArgValueCandidates::new(complete::profiles))]
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
        #[arg(add = ArgValueCandidates::new(complete::units))]
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
        #[arg(add = ArgValueCandidates::new(complete::units))]
        unit: String,
        /// Also restart every unit that depends on it
        #[arg(long)]
        cascade: bool,
    },
    /// List the profiles, or switch the running supervisor to NAME
    Profile {
        #[arg(add = ArgValueCandidates::new(complete::profiles))]
        name: Option<String>,
    },
    /// Run a just recipe as a task of the supervisor (or in the foreground without one)
    Run {
        #[arg(add = ArgValueCandidates::new(complete::recipes))]
        recipe: String,
        /// Arguments for the recipe, passed on unchanged
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
        /// Run a destructive recipe without asking
        #[arg(long)]
        yes: bool,
    },
    /// Start a stopped unit
    Start {
        #[arg(add = ArgValueCandidates::new(complete::units))]
        unit: String,
    },
    /// Stop a unit
    Stop {
        #[arg(add = ArgValueCandidates::new(complete::units))]
        unit: String,
    },
    /// Print the shell completion script (source it, e.g. `source <(geco completions zsh)`)
    Completions {
        #[arg(value_parser = ["bash", "elvish", "fish", "powershell", "zsh"])]
        shell: String,
    },
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
    fn completions_take_a_known_shell() {
        let cli = Cli::try_parse_from(["geco", "completions", "zsh"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Completions { shell }) if shell == "zsh"));
        assert!(Cli::try_parse_from(["geco", "completions", "cmd"]).is_err());
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
    fn profile_takes_an_optional_name() {
        let cli = Cli::try_parse_from(["geco", "profile"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Profile { name: None })));
        let cli = Cli::try_parse_from(["geco", "profile", "full"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Profile { name: Some(n) }) if n == "full"));
    }

    #[test]
    fn run_passes_everything_after_the_recipe_through() {
        let cli =
            Cli::try_parse_from(["geco", "run", "import-kataster-fl", "--dry-run", "x"]).unwrap();
        let Some(Command::Run { recipe, args, yes }) = cli.command else {
            panic!("expected run");
        };
        assert_eq!(recipe, "import-kataster-fl");
        assert_eq!(args, ["--dry-run", "x"]);
        assert!(!yes);

        let cli = Cli::try_parse_from(["geco", "run", "--yes", "migrate-reset"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Run { yes: true, .. })));
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
