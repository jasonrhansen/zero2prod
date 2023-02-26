use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::EnvFilter;

pub fn initialize() {
    let format = tracing_subscriber::fmt::format()
        .without_time()
        .with_target(false)
        .with_level(false)
        .compact();

    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    tracing_subscriber::fmt::fmt()
        .with_env_filter(filter)
        .event_format(format)
        .init();
}
