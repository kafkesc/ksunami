pub const LOG_FILTER_ENV_VAR: &str = "KSUNAMI_LOG";

/// Log level will be configured based on the given `verbosity_level`.
///
/// If the env var `KSUNAMI_LOG` is set, that will take precedence and configuration
/// will be based on the rules described [here](https://docs.rs/env_logger/latest/env_logger/#enabling-logging).
pub fn init(verbosity_level: u8) {
    let default_log_level = match verbosity_level {
        0 => log::Level::Info,
        1 => log::Level::Debug,
        _ => log::Level::Trace,
    };

    let logger_env = env_logger::Env::default().filter_or(LOG_FILTER_ENV_VAR, default_log_level.as_str());
    let mut logger_builder = env_logger::Builder::from_env(logger_env);
    logger_builder.init();

    info!("Configured log level: {}", log::max_level().as_str());
}
