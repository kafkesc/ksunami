use clap::error::ErrorKind;
pub use clap::{CommandFactory, Parser};

use crate::transition::Transition;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Minimum amount of records per second
    #[arg(long = "min", value_name = "RECORDS_PER_SECOND")]
    pub min: u32,

    /// How long to stay at minimum records/sec before ramp-up
    #[arg(long = "min-sec", default_value_t = 60, value_name = "SECONDS")]
    pub min_sec: u32,

    /// Maximum amount of records per second
    #[arg(long = "max", value_name = "RECORDS_PER_SECOND")]
    pub max: u32,

    /// How long to stay at maximum records/sec, before ramp-down
    #[arg(long = "max-sec", default_value_t = 60, value_name = "SECONDS")]
    pub max_sec: u32,

    /// Ramp-up transition from minimum to maximum records/sec
    #[arg(long = "up", value_enum, default_value_t = Transition::Linear, value_name = "TRANSITION_TYPE")]
    pub up: Transition,

    /// Ramp-up transition duration
    #[arg(long = "up-sec", value_name = "SECONDS")]
    pub up_sec: u32,

    /// Ramp-down transition from maximum to minimum records/sec
    #[arg(long = "down", value_enum, default_value_t = Transition::None, value_name = "TRANSITION_TYPE")]
    pub down: Transition,

    /// Ramp-down transition duration
    #[arg(long = "down-sec", value_name = "SECONDS")]
    pub down_sec: u32,

    /// Verbosity level
    #[arg(short,long, action = clap::ArgAction::Count)]
    verbose: u8,
}

impl Cli {
    pub fn parse_and_validate() -> Self {
        let cli = Self::parse();

        // Validate values provided for `min` and `max`
        if cli.min >= cli.max {
            let mut cmd = Cli::command();
            cmd.error(ErrorKind::InvalidValue, "Argument 'min' must be less than 'max'").exit();
        }

        // Validate `(up|down)` transition in respect to their `(up|down)_sec` value
        if cli.up != Transition::None && cli.up_sec == 0 {
            let mut cmd = Cli::command();
            cmd.error(
                ErrorKind::InvalidValue,
                "Argument 'up-sec' must be greater than 0 when 'up' transition is not 'none'",
            )
            .exit();
        }
        if cli.down != Transition::None && cli.down_sec == 0 {
            let mut cmd = Cli::command();
            cmd.error(
                ErrorKind::InvalidValue,
                "Argument 'down-sec' must be greater than 0 when 'down' transition is not 'none'",
            )
            .exit();
        }

        // Validate that non-zero values are assigned to `min_sec` and `max_sec`
        if cli.min_sec == 0 || cli.max_sec == 0 {
            let mut cmd = Cli::command();
            cmd.error(ErrorKind::InvalidValue, "Arguments 'min/max' must be greater than 0").exit();
        }

        cli
    }
}
