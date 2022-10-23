const DEFAULT_LOG_LEVEL: &str = "INFO";

pub fn init() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(DEFAULT_LOG_LEVEL)).init();
}
