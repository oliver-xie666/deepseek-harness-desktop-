use dsh_common::{AppPaths, DshError, Result};
use dsh_protocol::{AgentState, HarnessClientMessage, HarnessServerEvent, ToolStatus};
use futures::{SinkExt, StreamExt};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpListener as TokioTcpListener;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tracing::{info, warn};

pub struct DaemonConfig {
    pub port: u16,
    pub host: String,
    pub runtime_path: PathBuf,
    pub auto_restart: bool,
    pub use_embedded_mock: bool,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            port: 3000,
            host: "127.0.0.1".to_string(),
            runtime_path: AppPaths::runtime_dir(),
            auto_restart: true,
            use_embedded_mock: true,
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

    /// Starts either the embedded mock server or the Node.js dsh subprocess
    pub async fn start(&self) -> Result<()> {
        if self.is_running() {
            info!("Daemon is already running.");
            return Ok(());
        }

        self.ensure_runtime().await?;

        if self.config.use_embedded_mock {
            self.start_mock_server().await?;
        } else {
            self.start_node_subprocess().await?;
        }

        self.is_running.store(true, Ordering::SeqCst);
        Ok(())
    }

    async fn start_mock_server(&self) -> Result<()> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let listener = TokioTcpListener::bind(&addr).await.map_err(|e| {
            DshError::Daemon(format!("Failed to bind mock server on {}: {}", addr, e))
        })?;

        info!("Embedded DeepSeek Harness Mock Server listening on ws://{}", addr);

        tokio::spawn(async move {
            while let Ok((stream, peer_addr)) = listener.accept().await {
                info!("Accepted WebSocket connection from {}", peer_addr);

                tokio::spawn(async move {
                    if let Ok(mut ws_stream) = accept_async(stream).await {
                        while let Some(msg_res) = ws_stream.next().await {
                            if let Ok(WsMessage::Text(text)) = msg_res {
                                if let Ok(client_msg) =
                                    serde_json::from_str::<HarnessClientMessage>(&text)
                                {
                                    match client_msg {
                                        HarnessClientMessage::SendPrompt {
                                            session_id,
                                            text: prompt,
                                            ..
                                        } => {
                                            // 1. Send Thinking state
                                            let evt = HarnessServerEvent::AgentStateChange {
                                                session_id: session_id.clone(),
                                                state: AgentState::Thinking,
                                            };
                                            let _ = ws_stream
                                                .send(WsMessage::Text(
                                                    serde_json::to_string(&evt).unwrap().into(),
                                                ))
                                                .await;

                                            // 2. Stream tokens
                                            let response = format!(
                                                "收到您的请求：\"{}\"。正在使用 DeepSeek-V3 为您分析并生成 Rust 代码：\n\n```rust\npub fn greet() {{\n    println!(\"Hello from DeepSeek Harness Native!\");\n}}\n```\n\n> [!TIP]\n> 这是一个由内置 WebSocket 实时推送的高性能流式响应！",
                                                prompt
                                            );

                                            let message_id = uuid::Uuid::new_v4().to_string();
                                            for chunk in response.split_inclusive(' ') {
                                                tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
                                                let evt = HarnessServerEvent::TokenChunk {
                                                    session_id: session_id.clone(),
                                                    message_id: message_id.clone(),
                                                    text: chunk.to_string(),
                                                };
                                                let _ = ws_stream
                                                    .send(WsMessage::Text(
                                                        serde_json::to_string(&evt).unwrap().into(),
                                                    ))
                                                    .await;
                                            }

                                            // 3. Send Tool call
                                            let call_id = uuid::Uuid::new_v4().to_string();
                                            let tool_start = HarnessServerEvent::ToolCallStart {
                                                session_id: session_id.clone(),
                                                call_id: call_id.clone(),
                                                tool_name: "grep_search".to_string(),
                                                input: serde_json::json!({"query": "greet"}),
                                            };
                                            let _ = ws_stream
                                                .send(WsMessage::Text(
                                                    serde_json::to_string(&tool_start).unwrap().into(),
                                                ))
                                                .await;

                                            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

                                            let tool_end = HarnessServerEvent::ToolCallEnd {
                                                session_id: session_id.clone(),
                                                call_id: call_id.clone(),
                                                output: serde_json::json!({"matches": 1}),
                                                status: ToolStatus::Success,
                                                duration_ms: 100,
                                            };
                                            let _ = ws_stream
                                                .send(WsMessage::Text(
                                                    serde_json::to_string(&tool_end).unwrap().into(),
                                                ))
                                                .await;

                                            // 4. Send State completed
                                            let completed = HarnessServerEvent::AgentStateChange {
                                                session_id: session_id.clone(),
                                                state: AgentState::Completed,
                                            };
                                            let _ = ws_stream
                                                .send(WsMessage::Text(
                                                    serde_json::to_string(&completed).unwrap().into(),
                                                ))
                                                .await;
                                        }
                                        HarnessClientMessage::Ping => {
                                            let pong = HarnessServerEvent::Pong;
                                            let _ = ws_stream
                                                .send(WsMessage::Text(
                                                    serde_json::to_string(&pong).unwrap().into(),
                                                ))
                                                .await;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                });
            }
        });

        Ok(())
    }

    async fn start_node_subprocess(&self) -> Result<()> {
        let mut child_guard = self.child.lock().await;

        #[cfg(target_os = "windows")]
        let node_bin = "node.exe";
        #[cfg(not(target_os = "windows"))]
        let node_bin = "node";

        let mut cmd = Command::new(node_bin);
        cmd.arg("-e")
            .arg(format!(
                "console.log('DeepSeek Harness Node Daemon on port {}'); setInterval(() => {{}}, 1000);",
                self.config.port
            ))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to spawn node: {}. Falling back to mock.", e);
                return self.start_mock_server().await;
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
        Ok(())
    }

    /// Stops the daemon
    pub async fn stop(&self) -> Result<()> {
        let mut child_guard = self.child.lock().await;
        if let Some(mut child) = child_guard.take() {
            let _ = child.kill().await;
        }
        self.is_running.store(false, Ordering::SeqCst);
        info!("Daemon stopped.");
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    pub fn ws_url(&self) -> String {
        format!("ws://{}:{}", self.config.host, self.config.port)
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
    async fn test_mock_server_start() {
        let port = DaemonManager::find_available_port(3800, 3900).unwrap();
        let config = DaemonConfig {
            port,
            use_embedded_mock: true,
            ..Default::default()
        };
        let manager = DaemonManager::new(config);
        manager.start().await.unwrap();
        assert!(manager.is_running());
    }
}
