pub mod llm_example_network;

/// Initialize env_logger for tests. Safe to call multiple times.
pub fn init_logger() {
    let _ = env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Trace)
        .is_test(true)
        .try_init();
}
