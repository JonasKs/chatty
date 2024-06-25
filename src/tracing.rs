use std::{fs, path::PathBuf};

use tracing::Level;
use tracing_appender::{non_blocking, non_blocking::WorkerGuard};

pub fn init_tracing() -> WorkerGuard {
    let log_file_path = get_log_file_path();
    let log_file = fs::File::create(log_file_path).unwrap();
    let (non_blocking, guard) = non_blocking(log_file);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_max_level(Level::DEBUG)
        .init();
    guard
}

fn get_log_file_path() -> PathBuf {
    let log_file_name = "terminal-ai-ops.log".to_string();
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        PathBuf::from("/tmp/").join(log_file_name)
    }
    #[cfg(target_os = "windows")]
    {
        let temp_dir = env::var("TEMP").unwrap_or_else(|_| "C:\\Windows\\Temp".to_string());
        PathBuf::from(temp_dir).join(log_file_name)
    }
}
