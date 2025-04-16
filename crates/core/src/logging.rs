//! Logging utilities for the P2P network

use tracing::{Level, Subscriber};
use tracing_subscriber::{fmt, EnvFilter};

/// Initialize logging with the specified log level
pub fn init_logging(level: Level) -> Result<(), String> {
    let filter = EnvFilter::from_default_env().add_directive(
        format!("core={}", level)
            .parse()
            .map_err(|e: tracing_subscriber::filter::ParseError| e.to_string())?,
    );

    let subscriber = fmt::Subscriber::builder()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .finish();

    init_subscriber(subscriber)
}

/// Initialize with custom tracing subscriber
pub fn init_subscriber<S>(subscriber: S) -> Result<(), String>
where
    S: Subscriber + Send + Sync + 'static,
{
    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| format!("Failed to set global default subscriber: {}", e))
}

/// Initialize logging with default settings for development
pub fn init_default_logging() -> Result<(), String> {
    init_logging(Level::DEBUG)
}
