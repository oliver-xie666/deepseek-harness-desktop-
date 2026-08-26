use directories::ProjectDirs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DshError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Daemon process error: {0}")]
    Daemon(String),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("WebSocket communication error: {0}")]
    WebSocket(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Unknown error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, DshError>;

pub struct AppPaths;

impl AppPaths {
    pub fn get_project_dirs() -> Option<ProjectDirs> {
        ProjectDirs::from("com", "deepseek", "dsh-desktop")
    }

    pub fn data_dir() -> PathBuf {
        if let Ok(dir) = std::env::var("DSH_DATA_DIR") {
            if !dir.trim().is_empty() {
                return PathBuf::from(dir.trim());
            }
        }
        Self::get_project_dirs()
            .map(|dirs| dirs.data_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("./.dsh_data"))
    }

    pub fn config_dir() -> PathBuf {
        Self::get_project_dirs()
            .map(|dirs| dirs.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("./.dsh_config"))
    }

    pub fn runtime_dir() -> PathBuf {
        Self::data_dir().join("runtime")
    }

    pub fn logs_dir() -> PathBuf {
        Self::data_dir().join("logs")
    }
}

pub fn init_logging() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,dsh=debug")),
        )
        .try_init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paths() {
        let data = AppPaths::data_dir();
        let runtime = AppPaths::runtime_dir();
        assert!(runtime.starts_with(&data));
    }
}
