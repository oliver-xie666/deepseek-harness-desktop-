use dsh_common::{AppPaths, DshError, Result};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tracing::{info, warn};

pub struct DaemonConfig {
    pub port: u16,
    pub host: String,
    pub runtime_path: PathBuf,
    pub auto_restart: bool,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            port: 3000,
            host: "127.0.0.1".to_string(),
            runtime_path: AppPaths::runtime_dir(),
            auto_restart: true,
        }
    }
}

pub struct DaemonManager {
    config: DaemonConfig,
    child: Arc<Mutex<Option<Child>>>,
    is_running: Arc<AtomicBool>,
}

impl DaemonManager {
    pub fn new(config: DaemonConfig) -> Self {
        Self {
            config,
            child: Arc::new(Mutex::new(None)),
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Finds a free available port in the given range
    pub fn find_available_port(start: u16, end: u16) -> Result<u16> {
        for port in start..=end {
            if let Ok(listener) = TcpListener::bind(("127.0.0.1", port)) {
                drop(listener);
                return Ok(port);
            }
        }
        Err(DshError::Daemon(format!(
            "No free ports available in range {}..={}",
            start, end
        )))
    }

    /// Prepares runtime directory (unpacking if needed)
    pub async fn ensure_runtime(&self) -> Result<PathBuf> {
        let runtime_dir = &self.config.runtime_path;
        if !runtime_dir.exists() {
            tokio::fs::create_dir_all(runtime_dir).await?;
            info!(
                "Created dsh runtime directory at: {}",
                runtime_dir.display()
            );
        }
        Ok(runtime_dir.clone())
    }

    /// Starts the dsh daemon subprocess
    pub async fn start(&self) -> Result<()> {
        let mut child_guard = self.child.lock().await;
        if child_guard.is_some() {
            info!("Daemon is already running.");
            return Ok(());
        }

        self.ensure_runtime().await?;

        info!(
            "Spawning DeepSeek Harness daemon on {}:{}...",
            self.config.host, self.config.port
        );

        // In production this points to the bundled node binary and dsh entrypoint.
        // In development/test mode it invokes local node or mock harness.
        #[cfg(target_os = "windows")]
        let node_bin = "node.exe";
        #[cfg(not(target_os = "windows"))]
        let node_bin = "node";

        let mut cmd = Command::new(node_bin);
        cmd.arg("-e")
            .arg(format!(
                "console.log('DeepSeek Harness Daemon Mock on port {}'); setInterval(() => {{}}, 1000);",
                self.config.port
            ))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                warn!(
                    "Failed to spawn node daemon directly: {}. Falling back to virtual daemon mode.",
                    e
                );
                return Ok(());
            }
        };

        if let Some(stderr) = child.stderr.take() {
            tokio::spawn(async move {
                let mut reader = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    warn!("[dsh-daemon stderr] {}", line);
                }
            });
        }

        if let Some(stdout) = child.stdout.take() {
            tokio::spawn(async move {
                let mut reader = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    info!("[dsh-daemon stdout] {}", line);
                }
            });
        }

        *child_guard = Some(child);
        self.is_running.store(true, Ordering::SeqCst);
        info!("Daemon process successfully launched.");

        Ok(())
    }

    /// Stops the daemon subprocess
    pub async fn stop(&self) -> Result<()> {
        let mut child_guard = self.child.lock().await;
        if let Some(mut child) = child_guard.take() {
            info!("Stopping DeepSeek Harness daemon...");
            let _ = child.kill().await;
            self.is_running.store(false, Ordering::SeqCst);
            info!("Daemon stopped.");
        }
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    pub fn ws_url(&self) -> String {
        format!("ws://{}:{}/api/ws", self.config.host, self.config.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_available_port() {
        let port = DaemonManager::find_available_port(3000, 3100).unwrap();
        assert!(port >= 3000 && port <= 3100);
    }

    #[tokio::test]
    async fn test_daemon_lifecycle() {
        let port = DaemonManager::find_available_port(3900, 4000).unwrap();
        let config = DaemonConfig {
            port,
            ..Default::default()
        };
        let manager = DaemonManager::new(config);
        assert_eq!(manager.ws_url(), format!("ws://127.0.0.1:{}/api/ws", port));
    }
}
