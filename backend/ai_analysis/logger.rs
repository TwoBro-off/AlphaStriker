use std::path::Path;
use tracing_subscriber::{prelude::*, EnvFilter, fmt};
use tracing_appender::non_blocking::WorkerGuard;

pub fn init_tracing(log_dir: &str) -> Result<WorkerGuard, Box<dyn std::error::Error + Send + Sync>> {
    if !Path::new(log_dir).exists() {
        std::fs::create_dir_all(log_dir)?;
    }

    let console_layer = fmt::layer().with_writer(std::io::stderr);

    let file_appender = tracing_appender::rolling::daily(log_dir, "app.log");
    let (non_blocking_writer, guard) = tracing_appender::non_blocking(file_appender);
    let file_layer = fmt::layer()
        .with_writer(non_blocking_writer)
        .json()
        .with_span_events(fmt::format::FmtSpan::CLOSE);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .init();

    Ok(guard)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_init_tracing_creates_dir() {
        let log_dir = format!("./tmp_log_dir_{}", uuid::Uuid::new_v4());
        assert!(!Path::new(&log_dir).exists());
        let _guard = init_tracing(&log_dir).expect("init_tracing should succeed");
        assert!(Path::new(&log_dir).exists());
        std::fs::remove_dir_all(&log_dir).unwrap();
    }
}