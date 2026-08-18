use crate::AppState;
use dsh_protocol::{HarnessClientMessage, HarnessServerEvent};
use futures::{SinkExt, StreamExt};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tracing::{info, warn};

pub struct HarnessWsClient {
    ws_url: String,
    state: Arc<AppState>,
    is_connected: Arc<AtomicBool>,
}

impl HarnessWsClient {
    pub fn new(ws_url: &str, state: Arc<AppState>) -> Self {
        Self {
            ws_url: ws_url.to_string(),
            state,
            is_connected: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_connected(&self) -> bool {
        self.is_connected.load(Ordering::SeqCst)
    }

    /// Starts the background client runner loop
    pub fn start(self: Arc<Self>, mut outbox_rx: mpsc::Receiver<HarnessClientMessage>) {
        tokio::spawn(async move {
            let mut backoff_ms = 500u64;

            loop {
                info!(
                    "Connecting to DeepSeek Harness WebSocket at {}...",
                    self.ws_url
                );

                match connect_async(&self.ws_url).await {
                    Ok((ws_stream, _)) => {
                        info!("Successfully connected to DeepSeek Harness WebSocket!");
                        self.is_connected.store(true, Ordering::SeqCst);
                        backoff_ms = 500;

                        let (mut write, mut read) = ws_stream.split();

                        // Loop handling bidirectional traffic
                        loop {
                            tokio::select! {
                                // Outbound message from AppState to WebSocket
                                Some(out_msg) = outbox_rx.recv() => {
                                    if let Ok(json) = serde_json::to_string(&out_msg) {
                                        if let Err(e) = write.send(WsMessage::Text(json.into())).await {
                                            warn!("Failed to send message over WebSocket: {}", e);
                                            break;
                                        }
                                    }
                                }
                                // Inbound message from WebSocket to AppState
                                Some(msg_res) = read.next() => {
                                    match msg_res {
                                        Ok(WsMessage::Text(text)) => {
                                            if let Ok(event) = serde_json::from_str::<HarnessServerEvent>(&text) {
                                                self.state.handle_server_event(event).await;
                                            }
                                        }
                                        Ok(WsMessage::Close(_)) => {
                                            info!("WebSocket connection closed by server.");
                                            break;
                                        }
                                        Err(e) => {
                                            warn!("WebSocket read error: {}", e);
                                            break;
                                        }
                                        _ => {}
                                    }
                                }
                                else => break,
                            }
                        }

                        self.is_connected.store(false, Ordering::SeqCst);
                    }
                    Err(e) => {
                        warn!(
                            "Failed to connect to {}: {}. Retrying in {}ms...",
                            self.ws_url, e, backoff_ms
                        );
                    }
                }

                tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                backoff_ms = (backoff_ms * 2).min(5000);
            }
        });
    }
}
