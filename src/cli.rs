use clap::error::ErrorKind;
pub use clap::{value_parser, CommandFactory, Parser, ArgGroup};

use crate::generator::ValueGenerator;
use crate::rdkafka::PartitionerConfig;
use crate::transition::Transition;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(group(
    ArgGroup::new("logging_flags")
        .required(false)
        .multiple(false)
        .args(["verbose", "quiet"]),
))]
pub struct Cli {
    // ---------------------------------------------------------------------- Producer configuration
    /// Initial Kafka Brokers to connect to (format: 'HOST:PORT,...').
    ///
    /// Equivalent to '--config=bootstrap.brokers:host:port,...'.
    #[arg(short, long = "brokers", value_name = "BOOTSTRAP_BROKERS")]
    pub bootstrap_brokers: String,

    /// Client identifier used by the internal Kafka Producer.
    ///
    /// Equivalent to '--config=client.id:my-client-id'.
    #[arg(long = "client-id", value_name = "CLIENT_ID", default_value = env!("CARGO_PKG_NAME"))]
    pub client_id: Option<String>,

    /// Partitioner used by the internal Kafka Producer.
    ///
    /// Equivalent to '--config=partitioner:random'.
    #[arg(long, value_name = "PARTITIONER", value_enum, default_value_t = PartitionerConfig::ConsistentRandom)]
    pub partitioner: PartitionerConfig,

    /// Additional configuration used by the internal Kafka Producer (format: 'CONF_KEY:CONF_VAL').
    ///
    /// To set multiple configurations keys, use this argument multiple times.
    /// See: https://github.com/edenhill/librdkafka/blob/master/CONFIGURATION.md.
    #[arg(short, long, value_name = "CONF_KEY:CONF_VAL", value_parser = kv_clap_value_parser)]
    pub config: Vec<KVPair>,

    // ------------------------------------------------------------- Record generation configuration
    /// Destination Topic.
    ///
    /// Topic must already exist.
    #[arg(short = 't', long = "topic", value_name = "TOPIC")]
    pub topic: String,

    /// Records Key (format: 'KEY_TYPE:INPUT').
    ///
    /// The supported key types are:
    ///
    /// * 'string:STR': STR is a plain string
    /// * 'file:PATH': PATH is a path to an existing file
    /// * 'alpha:LENGTH': LENGTH is the length of a random alphanumeric string
    /// * 'bytes:LENGTH': LENGTH is the length of a random bytes array
    /// * 'int:MIN-MAX': MIN and MAX are limits of an inclusive range from which an integer number is picked
    /// * 'float:MIN-MAX': MIN and MAX are limits of an inclusive range from which an float number is picked
    #[arg(short, long, value_name = "KEY_TYPE:INPUT", value_parser = ValueGenerator::clap_value_parser, verbatim_doc_comment)]
    pub key: Option<ValueGenerator>,

    /// Records Payload (format: 'PAYLOAD_TYPE:INPUT').
    ///
    /// The supported payload types are:
    ///
    /// * 'string:STR': STR is a plain string
    /// * 'file:PATH': PATH is a path to an existing file
    /// * 'alpha:LENGTH': LENGTH is the length of a random alphanumeric string
    /// * 'bytes:LENGTH': LENGTH is the length of a random bytes array
    /// * 'int:MIN-MAX': MIN and MAX are limits of an inclusive range from which an integer number is picked
    /// * 'float:MIN-MAX': MIN and MAX are limits of an inclusive range from which an float number is picked
    #[arg(short, long, value_name = "PAYLOAD_TYPE:INPUT", value_parser = ValueGenerator::clap_value_parser, verbatim_doc_comment)]
    pub payload: Option<ValueGenerator>,

    /// Destination Topic Partition.
    ///
    /// If not specified (or '-1'), Producer will rely on the Partitioner.
    /// See the '--partitioner' argument.
    #[arg(long, value_name = "PARTITION", value_parser = value_parser!(i32).range(-1..))]
    pub partition: Option<i32>,

    /// Records Header(s) (format: 'HEAD_KEY:HEAD_VAL').
    ///
    /// To set multiple headers, use this argument multiple times.
    #[arg(long = "head", value_name = "HEAD_KEY:HEAD_VAL", value_parser = kv_clap_value_parser)]
    pub headers: Vec<KVPair>,

    // ---------------------------------------------------------------------- Workload configuration
    /// Minimum amount of records/sec.
    #[arg(long = "min", value_name = "REC/SEC")]
    pub min: u32,

    /// How long to produce at minimum records/sec, before ramp-up.
    #[arg(long = "min-sec", default_value_t = 60, value_name = "SEC")]
    pub min_sec: u32,

    /// Maximum amount of records/sec.
    #[arg(long = "max", value_name = "REC/SEC")]
    pub max: u32,

    /// How long to produce at maximum records/sec, before ramp-down.
    #[arg(long = "max-sec", default_value_t = 60, value_name = "SEC")]
    pub max_sec: u32,

    /// Ramp-up transition from minimum to maximum records/sec.
    #[arg(long = "up", value_enum, default_value_t = Transition::Linear, value_name = "TRANSITION_TYPE")]
    pub up: Transition,

    /// How long the ramp-up transition should last.
    #[arg(long = "up-sec", default_value_t = 10, value_name = "SEC")]
    pub up_sec: u32,

    /// Ramp-down transition from maximum to minimum records/sec.
    #[arg(long = "down", value_enum, default_value_t = Transition::None, value_name = "TRANSITION_TYPE")]
    pub down: Transition,

    /// How long the ramp-down transition should last.
    #[arg(long = "down-sec", default_value_t = 10, value_name = "SEC")]
    pub down_sec: u32,

    /// Verbose logging.
    ///
    /// * none    = 'WARN'
    /// * '-v'    = 'INFO'
    /// * '-vv'   = 'DEBUG'
    /// * '-vvv'  = 'TRACE'
    ///
    /// Alternatively, set environment variable 'KSUNAMI_LOG=(ERROR|WARN|INFO|DEBUG|TRACE|OFF)'.
    #[arg(short,long, action = clap::ArgAction::Count, verbatim_doc_comment)]
    pub verbose: u8,

    /// Quiet logging.
    ///
    /// * none    = 'WARN'
    /// * '-q'    = 'ERROR'
    /// * '-qq'   = 'OFF'
    ///
    /// Alternatively, set environment variable 'KSUNAMI_LOG=(ERROR|WARN|INFO|DEBUG|TRACE|OFF)'.
    #[arg(short,long, action = clap::ArgAction::Count, verbatim_doc_comment)]
    pub quiet: u8,
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

    pub fn verbosity_level(&self) -> i8 {
        self.verbose as i8 - self.quiet as i8
    }
}

pub type KVPair = (String, String);

fn kv_clap_value_parser(kv: &str) -> Result<KVPair, String> {
    let (k, v) = match kv.split_once(':') {
        None => {
            return Err("Should have 'K:V' format".to_string());
        }
        Some((k, v)) => (k, v),
    };

    Ok((k.to_string(), v.to_string()))
}
