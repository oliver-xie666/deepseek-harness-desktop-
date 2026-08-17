pub mod ws_client;

use chrono::{DateTime, Utc};
use dsh_common::Result;
use dsh_daemon::{DaemonConfig, DaemonManager};
use dsh_protocol::{
    AgentState, HarnessClientMessage, HarnessServerEvent, ToolStatus,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;
pub use ws_client::HarnessWsClient;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub sender: MessageSender,
    pub content: String,
    pub tool_calls: Vec<ToolCallItem>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessageSender {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCallItem {
    pub id: String,
    pub tool_name: String,
    pub input: serde_json::Value,
    pub output: Option<serde_json::Value>,
    pub status: ToolStatus,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileDiffItem {
    pub id: String,
    pub file_path: String,
    pub diff_content: String,
    pub accepted: Option<bool>,
}

#[derive(Debug, Clone, Default)]
pub struct Session {
    pub id: String,
    pub title: String,
    pub workspace_path: String,
    pub messages: Vec<ChatMessage>,
    pub diffs: HashMap<String, FileDiffItem>,
    pub terminal_logs: Vec<String>,
    pub agent_state: Option<AgentState>,
}

pub struct AppState {
    pub workspace_path: RwLock<PathBuf>,
    pub active_session_id: RwLock<Option<String>>,
    pub sessions: RwLock<HashMap<String, Session>>,
    pub daemon_manager: Arc<DaemonManager>,
    pub outbox_tx: mpsc::Sender<HarnessClientMessage>,
}

impl AppState {
    pub fn new(daemon_config: DaemonConfig) -> (Arc<Self>, mpsc::Receiver<HarnessClientMessage>) {
        let (outbox_tx, outbox_rx) = mpsc::channel(100);
        let daemon_manager = Arc::new(DaemonManager::new(daemon_config));

        let state = Arc::new(Self {
            workspace_path: RwLock::new(PathBuf::from(".")),
            active_session_id: RwLock::new(None),
            sessions: RwLock::new(HashMap::new()),
            daemon_manager,
            outbox_tx,
        });

        (state, outbox_rx)
    }

    pub fn start_background_client(
        state: Arc<Self>,
        outbox_rx: mpsc::Receiver<HarnessClientMessage>,
    ) -> Arc<HarnessWsClient> {
        let ws_url = state.daemon_manager.ws_url();
        let client = Arc::new(HarnessWsClient::new(&ws_url, state.clone()));
        client.clone().start(outbox_rx);
        client
    }

    pub async fn create_session(&self, title: &str, workspace: &str) -> String {
        let session_id = Uuid::new_v4().to_string();
        let session = Session {
            id: session_id.clone(),
            title: title.to_string(),
            workspace_path: workspace.to_string(),
            messages: Vec::new(),
            diffs: HashMap::new(),
            terminal_logs: Vec::new(),
            agent_state: Some(AgentState::Idle),
        };

        self.sessions.write().await.insert(session_id.clone(), session);
        *self.active_session_id.write().await = Some(session_id.clone());

        session_id
    }

    pub async fn add_user_message(&self, session_id: &str, text: &str) -> Result<()> {
        let msg = ChatMessage {
            id: Uuid::new_v4().to_string(),
            sender: MessageSender::User,
            content: text.to_string(),
            tool_calls: Vec::new(),
            created_at: Utc::now(),
        };

        if let Some(session) = self.sessions.write().await.get_mut(session_id) {
            session.messages.push(msg);
        }

        self.outbox_tx
            .send(HarnessClientMessage::SendPrompt {
                session_id: session_id.to_string(),
                text: text.to_string(),
                attachments: vec![],
            })
            .await
            .map_err(|e| dsh_common::DshError::Other(e.to_string()))?;

        Ok(())
    }

    pub async fn handle_server_event(&self, event: HarnessServerEvent) {
        match event {
            HarnessServerEvent::TokenChunk {
                session_id,
                message_id,
                text,
            } => {
                if let Some(session) = self.sessions.write().await.get_mut(&session_id) {
                    if let Some(last_msg) = session.messages.last_mut() {
                        if last_msg.sender == MessageSender::Assistant && last_msg.id == message_id {
                            last_msg.content.push_str(&text);
                            return;
                        }
                    }
                    session.messages.push(ChatMessage {
                        id: message_id,
                        sender: MessageSender::Assistant,
                        content: text,
                        tool_calls: Vec::new(),
                        created_at: Utc::now(),
                    });
                }
            }
            HarnessServerEvent::ToolCallStart {
                session_id,
                call_id,
                tool_name,
                input,
            } => {
                if let Some(session) = self.sessions.write().await.get_mut(&session_id) {
                    if let Some(last_msg) = session.messages.last_mut() {
                        last_msg.tool_calls.push(ToolCallItem {
                            id: call_id,
                            tool_name,
                            input,
                            output: None,
                            status: ToolStatus::Running,
                            duration_ms: 0,
                        });
                    }
                }
            }
            HarnessServerEvent::ToolCallEnd {
                session_id,
                call_id,
                output,
                status,
                duration_ms,
            } => {
                if let Some(session) = self.sessions.write().await.get_mut(&session_id) {
                    for msg in &mut session.messages {
                        if let Some(tc) = msg.tool_calls.iter_mut().find(|t| t.id == call_id) {
                            tc.output = Some(output.clone());
                            tc.status = status;
                            tc.duration_ms = duration_ms;
                            break;
                        }
                    }
                }
            }
            HarnessServerEvent::FileDiffReady {
                session_id,
                diff_id,
                file_path,
                diff_content,
            } => {
                if let Some(session) = self.sessions.write().await.get_mut(&session_id) {
                    session.diffs.insert(
                        diff_id.clone(),
                        FileDiffItem {
                            id: diff_id,
                            file_path,
                            diff_content,
                            accepted: None,
                        },
                    );
                }
            }
            HarnessServerEvent::AgentStateChange { session_id, state } => {
                if let Some(session) = self.sessions.write().await.get_mut(&session_id) {
                    session.agent_state = Some(state);
                }
            }
            HarnessServerEvent::TerminalLog {
                session_id,
                line,
                ..
            } => {
                if let Some(session) = self.sessions.write().await.get_mut(&session_id) {
                    session.terminal_logs.push(line);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_state_session() {
        let (state, _) = AppState::new(DaemonConfig::default());
        let session_id = state.create_session("Test Session", "/tmp").await;
        assert_eq!(*state.active_session_id.read().await, Some(session_id.clone()));

        state
            .handle_server_event(HarnessServerEvent::TokenChunk {
                session_id: session_id.clone(),
                message_id: "msg-1".to_string(),
                text: "Hello".to_string(),
            })
            .await;

        let sessions = state.sessions.read().await;
        let session = sessions.get(&session_id).unwrap();
        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.messages[0].content, "Hello");
    }

    #[tokio::test]
    async fn test_end_to_end_ws_client_mock() {
        let port = dsh_daemon::DaemonManager::find_available_port(3700, 3800).unwrap();
        let config = DaemonConfig {
            port,
            use_embedded_mock: true,
            ..Default::default()
        };

        let (state, outbox_rx) = AppState::new(config);
        state.daemon_manager.start().await.unwrap();

        let session_id = state.create_session("Mock Chat", "/tmp").await;
        let _client = AppState::start_background_client(state.clone(), outbox_rx);

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Send a prompt
        state.add_user_message(&session_id, "Hello Rust").await.unwrap();

        // Wait for mock stream response
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        let sessions = state.sessions.read().await;
        let session = sessions.get(&session_id).unwrap();
        assert!(session.messages.len() >= 2); // 1 User + 1 Assistant
    }
}
