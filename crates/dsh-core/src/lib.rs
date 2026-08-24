pub mod config;
pub mod diff_applier;
pub mod fs_tree;
pub mod mcp;
pub mod persistence;
pub mod ws_client;

use chrono::{DateTime, Utc};
use dsh_common::{AppPaths, Result};
use dsh_daemon::{DaemonConfig, DaemonManager};
use dsh_protocol::{AgentState, HarnessClientMessage, HarnessServerEvent, ToolStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

pub use config::{AppConfig, ModelConfig, ProviderType, UiConfig};
pub use diff_applier::DiffApplier;
pub use fs_tree::{FileNode, WorkspaceScanner};
pub use mcp::{McpRegistry, McpServerConfig, McpTransport};
pub use persistence::SessionPersistence;
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub title: String,
    pub workspace_path: String,
    #[serde(default = "default_session_time")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "default_session_time")]
    pub updated_at: DateTime<Utc>,
    pub messages: Vec<ChatMessage>,
    pub diffs: HashMap<String, FileDiffItem>,
    pub terminal_logs: Vec<String>,
    pub agent_state: Option<AgentState>,
}

fn default_session_time() -> DateTime<Utc> {
    Utc::now()
}

pub struct AppState {
    pub workspace_path: RwLock<PathBuf>,
    pub active_session_id: RwLock<Option<String>>,
    pub sessions: RwLock<HashMap<String, Session>>,
    pub daemon_manager: Arc<DaemonManager>,
    pub config: RwLock<AppConfig>,
    pub mcp_servers: RwLock<Vec<McpServerConfig>>,
    pub outbox_tx: mpsc::Sender<HarnessClientMessage>,
}

impl AppState {
    pub fn new(daemon_config: DaemonConfig) -> (Arc<Self>, mpsc::Receiver<HarnessClientMessage>) {
        let (outbox_tx, outbox_rx) = mpsc::channel(100);
        let daemon_manager = Arc::new(DaemonManager::new(daemon_config));
        let data_dir = AppPaths::data_dir();
        let config = AppConfig::load_or_default(&data_dir);
        let mcp_servers = McpRegistry::load_servers(&data_dir);

        let state = Arc::new(Self {
            workspace_path: RwLock::new(PathBuf::from(".")),
            active_session_id: RwLock::new(None),
            sessions: RwLock::new(HashMap::new()),
            daemon_manager,
            config: RwLock::new(config),
            mcp_servers: RwLock::new(mcp_servers),
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
        let now = Utc::now();
        let session = Session {
            id: session_id.clone(),
            title: title.to_string(),
            workspace_path: workspace.to_string(),
            created_at: now,
            updated_at: now,
            messages: Vec::new(),
            diffs: HashMap::new(),
            terminal_logs: Vec::new(),
            agent_state: Some(AgentState::Idle),
        };

        self.sessions
            .write()
            .await
            .insert(session_id.clone(), session.clone());
        *self.active_session_id.write().await = Some(session_id.clone());

        // Auto persist
        let _ = SessionPersistence::save_session(&AppPaths::data_dir(), &session);

        session_id
    }

    pub async fn session_snapshot(&self) -> Vec<Session> {
        let mut sessions = self
            .sessions
            .read()
            .await
            .values()
            .cloned()
            .collect::<Vec<_>>();
        sessions.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
        sessions
    }

    pub async fn select_session(&self, session_id: &str) -> bool {
        if self.sessions.read().await.contains_key(session_id) {
            *self.active_session_id.write().await = Some(session_id.to_string());
            true
        } else {
            false
        }
    }

    pub async fn rename_session(&self, session_id: &str, title: &str) -> Result<bool> {
        let title = title.trim();
        if title.is_empty() {
            return Ok(false);
        }

        let Some(mut updated) = self.sessions.read().await.get(session_id).cloned() else {
            return Ok(false);
        };
        updated.title = title.to_string();
        updated.updated_at = Utc::now();
        SessionPersistence::save_session(&AppPaths::data_dir(), &updated)?;
        self.sessions
            .write()
            .await
            .insert(session_id.to_string(), updated);
        Ok(true)
    }

    pub async fn duplicate_session(&self, session_id: &str) -> Result<Option<String>> {
        let Some(mut duplicate) = self.sessions.read().await.get(session_id).cloned() else {
            return Ok(None);
        };

        let now = Utc::now();
        duplicate.id = Uuid::new_v4().to_string();
        duplicate.title = format!("{} 副本", duplicate.title);
        duplicate.created_at = now;
        duplicate.updated_at = now;
        let duplicate_id = duplicate.id.clone();
        SessionPersistence::save_session(&AppPaths::data_dir(), &duplicate)?;
        self.sessions
            .write()
            .await
            .insert(duplicate_id.clone(), duplicate);
        *self.active_session_id.write().await = Some(duplicate_id.clone());
        Ok(Some(duplicate_id))
    }

    pub async fn delete_session(&self, session_id: &str) -> Result<bool> {
        if !self.sessions.read().await.contains_key(session_id) {
            return Ok(false);
        }

        SessionPersistence::delete_session(&AppPaths::data_dir(), session_id)?;
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id);
        let next_active = sessions
            .values()
            .max_by_key(|session| session.updated_at)
            .map(|session| session.id.clone());
        drop(sessions);

        if self.active_session_id.read().await.as_deref() == Some(session_id) {
            *self.active_session_id.write().await = next_active;
        }
        Ok(true)
    }

    pub async fn set_workspace_path(&self, workspace_path: PathBuf) {
        *self.workspace_path.write().await = workspace_path;
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
            session.updated_at = Utc::now();
            let _ = SessionPersistence::save_session(&AppPaths::data_dir(), session);
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

    pub async fn apply_diff(
        &self,
        session_id: &str,
        diff_id: &str,
        new_content: &str,
    ) -> Result<()> {
        let workspace = self.workspace_path.read().await.clone();
        let mut sessions = self.sessions.write().await;

        if let Some(session) = sessions.get_mut(session_id) {
            if let Some(diff) = session.diffs.get_mut(diff_id) {
                DiffApplier::apply_file_content(&workspace, &diff.file_path, new_content)?;
                diff.accepted = Some(true);
                let _ = SessionPersistence::save_session(&AppPaths::data_dir(), session);
            }
        }
        Ok(())
    }

    pub async fn reject_diff(&self, session_id: &str, diff_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            if let Some(diff) = session.diffs.get_mut(diff_id) {
                diff.accepted = Some(false);
                let _ = SessionPersistence::save_session(&AppPaths::data_dir(), session);
            }
        }
        Ok(())
    }

    pub async fn load_saved_sessions(&self) {
        if let Ok(saved) = SessionPersistence::load_all_sessions(&AppPaths::data_dir()) {
            let mut sessions = self.sessions.write().await;
            for s in saved {
                sessions.insert(s.id.clone(), s);
            }
            let active = sessions
                .values()
                .max_by_key(|session| session.updated_at)
                .map(|session| session.id.clone());
            *self.active_session_id.write().await = active;
        }
    }

    pub async fn handle_server_event(&self, event: HarnessServerEvent) {
        let session_to_save = match event {
            HarnessServerEvent::TokenChunk {
                session_id,
                message_id,
                text,
            } => {
                if let Some(session) = self.sessions.write().await.get_mut(&session_id) {
                    if let Some(last_msg) = session.messages.last_mut() {
                        if last_msg.sender == MessageSender::Assistant && last_msg.id == message_id
                        {
                            last_msg.content.push_str(&text);
                            session.updated_at = Utc::now();
                            Some(session.clone())
                        } else {
                            session.messages.push(ChatMessage {
                                id: message_id,
                                sender: MessageSender::Assistant,
                                content: text,
                                tool_calls: Vec::new(),
                                created_at: Utc::now(),
                            });
                            session.updated_at = Utc::now();
                            Some(session.clone())
                        }
                    } else {
                        session.messages.push(ChatMessage {
                            id: message_id,
                            sender: MessageSender::Assistant,
                            content: text,
                            tool_calls: Vec::new(),
                            created_at: Utc::now(),
                        });
                        session.updated_at = Utc::now();
                        Some(session.clone())
                    }
                } else {
                    None
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
                    session.updated_at = Utc::now();
                    Some(session.clone())
                } else {
                    None
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
                    session.updated_at = Utc::now();
                    Some(session.clone())
                } else {
                    None
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
                    session.updated_at = Utc::now();
                    Some(session.clone())
                } else {
                    None
                }
            }
            HarnessServerEvent::AgentStateChange { session_id, state } => {
                if let Some(session) = self.sessions.write().await.get_mut(&session_id) {
                    session.agent_state = Some(state);
                    session.updated_at = Utc::now();
                    Some(session.clone())
                } else {
                    None
                }
            }
            HarnessServerEvent::TerminalLog {
                session_id, line, ..
            } => {
                if let Some(session) = self.sessions.write().await.get_mut(&session_id) {
                    session.terminal_logs.push(line);
                    session.updated_at = Utc::now();
                    Some(session.clone())
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some(session) = session_to_save {
            let _ = SessionPersistence::save_session(&AppPaths::data_dir(), &session);
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
        assert_eq!(
            *state.active_session_id.read().await,
            Some(session_id.clone())
        );

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
    async fn session_lifecycle_uses_ids_and_updates_active_session() {
        let (state, _) = AppState::new(DaemonConfig::default());
        let first = state.create_session("相同标题", "/tmp/a").await;
        let second = state.create_session("相同标题", "/tmp/b").await;

        assert!(state.rename_session(&first, "已重命名").await.unwrap());
        let copy = state.duplicate_session(&first).await.unwrap().unwrap();
        assert_ne!(copy, first);
        assert!(state.delete_session(&second).await.unwrap());
        assert_eq!(*state.active_session_id.read().await, Some(copy));
    }
}
