use std::{path::Path, sync::OnceLock};
use tracing_subscriber::EnvFilter;

static LOG_GUARD: OnceLock<tracing_appender::non_blocking::WorkerGuard> = OnceLock::new();

pub fn init(verbose: bool, tui: bool, path: Option<&Path>) -> super::error::Result<()> {
    let filter = EnvFilter::new(if verbose {
        "nayuta_cli=debug"
    } else {
        "nayuta_cli=warn"
    });
    if tui || path.is_some() {
        let log = path.map(std::path::PathBuf::from).unwrap_or_else(|| {
            directories::ProjectDirs::from("dev", "nayuta", "nayuta")
                .map(|d| d.cache_dir().join("logs/nayuta.log"))
                .unwrap_or_else(|| std::env::temp_dir().join("nayuta.log"))
        });
        if let Some(parent) = log.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let (writer, guard) = tracing_appender::non_blocking(
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log)?,
        );
        let _ = LOG_GUARD.set(guard);
        let _ = tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_ansi(false)
            .with_writer(writer)
            .try_init();
    } else {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_writer(std::io::stderr)
            .try_init();
    }
    Ok(())
}
