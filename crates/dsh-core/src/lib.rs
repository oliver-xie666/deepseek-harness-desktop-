pub mod config;
pub mod diff_applier;
pub mod fs_tree;
pub mod mcp;
pub mod persistence;
pub mod ws_client;

use chrono::{DateTime, Utc};
use dsh_common::{AppPaths, Result};
use dsh_daemon::{DaemonConfig, DaemonManager};
use dsh_protocol::{
    AgentState, GoalItem, GoalPhase, HarnessClientMessage, HarnessServerEvent, JobItem, JobStatus,
    PlanState, QuestionItem, ToolStatus,
};
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

/// Extract produced / modified file paths from tool calls (e.g. deliverables)
pub fn extract_produced_files(tool_calls: &[ToolCallItem]) -> Vec<String> {
    let mut paths = Vec::new();
    for tool in tool_calls {
        if tool.status == ToolStatus::Failed {
            continue;
        }
        let name = tool.tool_name.to_lowercase();
        if name.contains("write")
            || name.contains("edit")
            || name.contains("patch")
            || name.contains("create")
            || name.contains("save")
            || name.contains("deliverable")
        {
            if let Some(path_val) = tool
                .input
                .get("path")
                .or_else(|| tool.input.get("filepath"))
                .or_else(|| tool.input.get("file"))
                .or_else(|| tool.input.get("target"))
            {
                if let Some(path_str) = path_val.as_str() {
                    let trimmed = path_str.trim();
                    if !trimmed.is_empty() && !paths.contains(&trimmed.to_string()) {
                        paths.push(trimmed.to_string());
                    }
                }
            }
        }
    }
    paths
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
    #[serde(default)]
    pub plan_state: Option<PlanState>,
    #[serde(default)]
    pub pending_question: Option<QuestionItem>,
    #[serde(default)]
    pub goal: Option<GoalItem>,
    #[serde(default)]
    pub jobs: Vec<JobItem>,
}

fn default_session_time() -> DateTime<Utc> {
    Utc::now()
}

pub struct AppState {
    pub workspace_path: RwLock<PathBuf>,
    pub storage_dir: PathBuf,
    pub active_session_id: RwLock<Option<String>>,
    pub sessions: RwLock<HashMap<String, Session>>,
    pub daemon_manager: Arc<DaemonManager>,
    pub config: RwLock<AppConfig>,
    pub mcp_servers: RwLock<Vec<McpServerConfig>>,
    pub outbox_tx: mpsc::Sender<HarnessClientMessage>,
}

impl AppState {
    pub fn new(daemon_config: DaemonConfig) -> (Arc<Self>, mpsc::Receiver<HarnessClientMessage>) {
        Self::new_with_storage(daemon_config, AppPaths::data_dir())
    }

    pub fn new_with_storage(
        daemon_config: DaemonConfig,
        storage_dir: PathBuf,
    ) -> (Arc<Self>, mpsc::Receiver<HarnessClientMessage>) {
        let (outbox_tx, outbox_rx) = mpsc::channel(100);
        let daemon_manager = Arc::new(DaemonManager::new(daemon_config));
        let config = AppConfig::load_or_default(&storage_dir);
        let mcp_servers = McpRegistry::load_servers(&storage_dir);

        let state = Arc::new(Self {
            workspace_path: RwLock::new(PathBuf::from(".")),
            storage_dir,
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
            plan_state: None,
            pending_question: None,
            goal: None,
            jobs: Vec::new(),
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
        SessionPersistence::save_session(&self.storage_dir, &updated)?;
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
        SessionPersistence::save_session(&self.storage_dir, &duplicate)?;
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

        SessionPersistence::delete_session(&self.storage_dir, session_id)?;
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

    pub async fn toggle_plan_mode(&self, session_id: &str, enabled: bool) -> Result<()> {
        if let Some(session) = self.sessions.write().await.get_mut(session_id) {
            let current = session.plan_state.clone().unwrap_or_default();
            session.plan_state = Some(PlanState {
                active: enabled,
                plan_markdown: current.plan_markdown,
            });
            session.updated_at = Utc::now();
            let _ = SessionPersistence::save_session(&self.storage_dir, session);
        }
        let _ = self
            .outbox_tx
            .send(HarnessClientMessage::TogglePlanMode {
                session_id: session_id.to_string(),
                enabled,
            })
            .await;
        Ok(())
    }

    pub async fn answer_question(
        &self,
        session_id: &str,
        question_id: &str,
        selected: Vec<String>,
        custom_text: Option<String>,
    ) -> Result<()> {
        if let Some(session) = self.sessions.write().await.get_mut(session_id) {
            if let Some(q) = &mut session.pending_question {
                if q.id == question_id {
                    q.answered = Some(selected.clone());
                }
            }
            session.updated_at = Utc::now();
            let _ = SessionPersistence::save_session(&self.storage_dir, session);
        }
        let _ = self
            .outbox_tx
            .send(HarnessClientMessage::AnswerQuestion {
                session_id: session_id.to_string(),
                question_id: question_id.to_string(),
                selected,
                custom_text,
            })
            .await;
        Ok(())
    }

    pub async fn add_user_message(&self, session_id: &str, text: &str) -> Result<()> {
        self.add_user_message_with_attachments(session_id, text, Vec::new())
            .await
    }

    pub async fn add_user_message_with_attachments(
        &self,
        session_id: &str,
        text: &str,
        attachments: Vec<String>,
    ) -> Result<()> {
        let mut full_content = text.to_string();
        if !attachments.is_empty() {
            full_content.push_str("\n\n**附件:**\n");
            for att in &attachments {
                full_content.push_str(&format!("- `{}`\n", att));
            }
        }
        let msg = ChatMessage {
            id: Uuid::new_v4().to_string(),
            sender: MessageSender::User,
            content: full_content,
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
                attachments,
            })
            .await
            .map_err(|e| dsh_common::DshError::Other(e.to_string()))?;

        Ok(())
    }

    #[doc(hidden)]
    pub async fn _legacy_add_user_message(&self, session_id: &str, text: &str) -> Result<()> {
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
        diff_content: &str,
    ) -> Result<()> {
        let workspace = self.workspace_path.read().await.clone();
        let mut sessions = self.sessions.write().await;

        if let Some(session) = sessions.get_mut(session_id) {
            if let Some(diff) = session.diffs.get_mut(diff_id) {
                DiffApplier::apply_unified_diff(&workspace, &diff.file_path, diff_content)?;
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

    pub async fn update_goal(
        &self,
        session_id: &str,
        objective: &str,
        phase: GoalPhase,
    ) -> Result<()> {
        {
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(session_id) {
                let goal_id = session
                    .goal
                    .as_ref()
                    .map(|g| g.id.clone())
                    .unwrap_or_else(|| Uuid::new_v4().to_string());
                session.goal = Some(GoalItem {
                    id: goal_id,
                    objective: objective.to_string(),
                    phase,
                    error: None,
                });
                session.updated_at = Utc::now();
            }
        }
        let _ = self
            .outbox_tx
            .send(HarnessClientMessage::UpdateGoal {
                session_id: session_id.to_string(),
                objective: objective.to_string(),
                phase,
            })
            .await;
        Ok(())
    }

    pub async fn clear_goal(&self, session_id: &str) -> Result<()> {
        {
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(session_id) {
                session.goal = None;
                session.updated_at = Utc::now();
            }
        }
        let _ = self
            .outbox_tx
            .send(HarnessClientMessage::ClearGoal {
                session_id: session_id.to_string(),
            })
            .await;
        Ok(())
    }

    pub async fn kill_job(&self, session_id: &str, job_id: &str) -> Result<()> {
        {
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(session_id) {
                if let Some(job) = session.jobs.iter_mut().find(|j| j.id == job_id) {
                    job.status = JobStatus::Stopping;
                }
                session.updated_at = Utc::now();
            }
        }
        let _ = self
            .outbox_tx
            .send(HarnessClientMessage::KillJob {
                session_id: session_id.to_string(),
                job_id: job_id.to_string(),
            })
            .await;
        Ok(())
    }

    pub fn export_session_markdown(session: &Session) -> String {
        let mut md = String::new();
        md.push_str(&format!("# 会话导出: {}\n\n", session.title));
        md.push_str(&format!("- **会话 ID**: `{}`\n", session.id));
        md.push_str(&format!("- **工作区**: `{}`\n", session.workspace_path));
        md.push_str(&format!(
            "- **创建时间**: {}\n",
            session.created_at.to_rfc3339()
        ));
        md.push_str(&format!(
            "- **更新时间**: {}\n\n",
            session.updated_at.to_rfc3339()
        ));

        if let Some(goal) = &session.goal {
            let phase_str = match goal.phase {
                GoalPhase::Active => "进行中",
                GoalPhase::Paused => "已暂停",
                GoalPhase::Blocked => "阻塞",
                GoalPhase::Complete => "已完成",
            };
            md.push_str(&format!("## 目标: {} ({})\n\n", goal.objective, phase_str));
        }

        if let Some(plan) = &session.plan_state {
            if plan.active && !plan.plan_markdown.is_empty() {
                md.push_str("## 规划 (Plan)\n\n");
                md.push_str(&plan.plan_markdown);
                md.push_str("\n\n");
            }
        }

        md.push_str("## 消息记录\n\n");
        for msg in &session.messages {
            let sender = match msg.sender {
                MessageSender::User => "🧑 **User**",
                MessageSender::Assistant => "🤖 **Assistant**",
                MessageSender::System => "⚙️ **System**",
            };
            md.push_str(&format!("### {}\n\n", sender));
            md.push_str(&msg.content);
            md.push_str("\n\n");

            if !msg.tool_calls.is_empty() {
                md.push_str("#### 工具调用\n\n");
                for tc in &msg.tool_calls {
                    md.push_str(&format!(
                        "- **`{}`** (耗时 {}ms, 状态: {:?})\n",
                        tc.tool_name, tc.duration_ms, tc.status
                    ));
                    md.push_str("  ```json\n");
                    md.push_str(&format!("  // Input\n  {}\n", tc.input));
                    if let Some(out) = &tc.output {
                        md.push_str(&format!("  // Output\n  {}\n", out));
                    }
                    md.push_str("  ```\n\n");
                }
            }
        }

        if !session.terminal_logs.is_empty() {
            md.push_str("## 终端执行日志\n\n```text\n");
            for log in &session.terminal_logs {
                md.push_str(log);
                md.push('\n');
            }
            md.push_str("```\n");
        }

        md
    }

    pub fn export_session_json(session: &Session) -> serde_json::Result<String> {
        serde_json::to_string_pretty(session)
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
            HarnessServerEvent::PlanUpdate {
                session_id,
                active,
                plan_markdown,
            } => {
                if let Some(session) = self.sessions.write().await.get_mut(&session_id) {
                    session.plan_state = Some(PlanState {
                        active,
                        plan_markdown,
                    });
                    session.updated_at = Utc::now();
                    Some(session.clone())
                } else {
                    None
                }
            }
            HarnessServerEvent::QuestionPrompt {
                session_id,
                question_id,
                prompt,
                options,
                multi_select,
            } => {
                if let Some(session) = self.sessions.write().await.get_mut(&session_id) {
                    session.pending_question = Some(QuestionItem {
                        id: question_id,
                        prompt,
                        options,
                        multi_select,
                        answered: None,
                    });
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
            HarnessServerEvent::GoalUpdate { session_id, goal } => {
                if let Some(session) = self.sessions.write().await.get_mut(&session_id) {
                    session.goal = goal;
                    session.updated_at = Utc::now();
                    Some(session.clone())
                } else {
                    None
                }
            }
            HarnessServerEvent::JobUpdate { session_id, job } => {
                if let Some(session) = self.sessions.write().await.get_mut(&session_id) {
                    if let Some(idx) = session.jobs.iter().position(|j| j.id == job.id) {
                        session.jobs[idx] = job;
                    } else {
                        session.jobs.push(job);
                    }
                    session.updated_at = Utc::now();
                    Some(session.clone())
                } else {
                    None
                }
            }
            HarnessServerEvent::JobListUpdate { session_id, jobs } => {
                if let Some(session) = self.sessions.write().await.get_mut(&session_id) {
                    session.jobs = jobs;
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
        let temp_dir =
            std::env::temp_dir().join(format!("dsh_test_session_{}", uuid::Uuid::new_v4()));
        let (state, _) = AppState::new_with_storage(DaemonConfig::default(), temp_dir.clone());
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
        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[tokio::test]
    async fn session_lifecycle_uses_ids_and_updates_active_session() {
        let temp_dir =
            std::env::temp_dir().join(format!("dsh_test_lifecycle_{}", uuid::Uuid::new_v4()));
        let (state, _) = AppState::new_with_storage(DaemonConfig::default(), temp_dir.clone());
        let first = state.create_session("相同标题", "/tmp/a").await;
        let second = state.create_session("相同标题", "/tmp/b").await;

        assert!(state.rename_session(&first, "已重命名").await.unwrap());
        let copy = state.duplicate_session(&first).await.unwrap().unwrap();
        assert_ne!(copy, first);
        assert!(state.delete_session(&second).await.unwrap());
        assert_eq!(*state.active_session_id.read().await, Some(copy));
        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[tokio::test]
    async fn workspace_path_can_be_updated() {
        let (state, _) = AppState::new(DaemonConfig::default());
        state
            .set_workspace_path(std::path::PathBuf::from("/tmp/workspace"))
            .await;
        assert_eq!(
            *state.workspace_path.read().await,
            std::path::PathBuf::from("/tmp/workspace")
        );
    }
    #[tokio::test]
    async fn test_plan_and_question_events() {
        let temp_dir = std::env::temp_dir().join(format!("dsh_test_plan_{}", uuid::Uuid::new_v4()));
        let (state, mut outbox_rx) =
            AppState::new_with_storage(DaemonConfig::default(), temp_dir.clone());
        let session_id = state.create_session("Plan Session", "/tmp").await;

        state
            .handle_server_event(HarnessServerEvent::PlanUpdate {
                session_id: session_id.clone(),
                active: true,
                plan_markdown: "1. Step 1
2. Step 2"
                    .into(),
            })
            .await;

        let sessions = state.sessions.read().await;
        let session = sessions.get(&session_id).unwrap();
        assert!(session.plan_state.as_ref().unwrap().active);
        assert_eq!(
            session.plan_state.as_ref().unwrap().plan_markdown,
            "1. Step 1
2. Step 2"
        );
        drop(sessions);

        state.toggle_plan_mode(&session_id, false).await.unwrap();
        let msg = outbox_rx.try_recv().unwrap();
        assert_eq!(
            msg,
            HarnessClientMessage::TogglePlanMode {
                session_id: session_id.clone(),
                enabled: false,
            }
        );

        state
            .handle_server_event(HarnessServerEvent::QuestionPrompt {
                session_id: session_id.clone(),
                question_id: "q-1".into(),
                prompt: "Select mode".into(),
                options: vec!["Mode A".into(), "Mode B".into()],
                multi_select: false,
            })
            .await;

        let sessions = state.sessions.read().await;
        let session = sessions.get(&session_id).unwrap();
        assert_eq!(
            session.pending_question.as_ref().unwrap().prompt,
            "Select mode"
        );
        drop(sessions);

        state
            .answer_question(&session_id, "q-1", vec!["Mode A".into()], None)
            .await
            .unwrap();
        let msg2 = outbox_rx.try_recv().unwrap();
        assert_eq!(
            msg2,
            HarnessClientMessage::AnswerQuestion {
                session_id: session_id.clone(),
                question_id: "q-1".into(),
                selected: vec!["Mode A".into()],
                custom_text: None,
            }
        );

        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[tokio::test]
    async fn test_goal_and_job_events_and_export() {
        let temp_dir = std::env::temp_dir().join(format!("dsh_test_goal_{}", uuid::Uuid::new_v4()));
        let (state, mut outbox_rx) =
            AppState::new_with_storage(DaemonConfig::default(), temp_dir.clone());
        let session_id = state.create_session("Goal Session", "/tmp").await;

        state
            .handle_server_event(HarnessServerEvent::GoalUpdate {
                session_id: session_id.clone(),
                goal: Some(GoalItem {
                    id: "g-1".into(),
                    objective: "Test objective".into(),
                    phase: GoalPhase::Active,
                    error: None,
                }),
            })
            .await;

        let sessions = state.sessions.read().await;
        let session = sessions.get(&session_id).unwrap();
        assert_eq!(session.goal.as_ref().unwrap().objective, "Test objective");
        drop(sessions);

        state
            .handle_server_event(HarnessServerEvent::JobUpdate {
                session_id: session_id.clone(),
                job: JobItem {
                    id: "j-1".into(),
                    kind: "agent".into(),
                    label: "code-worker".into(),
                    status: JobStatus::Running,
                    started_at: Utc::now(),
                    duration_ms: Some(2500),
                },
            })
            .await;

        let sessions = state.sessions.read().await;
        let session = sessions.get(&session_id).unwrap();
        assert_eq!(session.jobs.len(), 1);
        assert_eq!(session.jobs[0].label, "code-worker");

        // Test markdown & json export
        let md = AppState::export_session_markdown(session);
        assert!(md.contains("# 会话导出: Goal Session"));
        assert!(md.contains("## 目标: Test objective (进行中)"));

        let json = AppState::export_session_json(session).unwrap();
        assert!(json.contains("Test objective"));
        drop(sessions);

        state.clear_goal(&session_id).await.unwrap();
        let msg = outbox_rx.try_recv().unwrap();
        assert_eq!(
            msg,
            HarnessClientMessage::ClearGoal {
                session_id: session_id.clone()
            }
        );

        state.kill_job(&session_id, "j-1").await.unwrap();
        let msg2 = outbox_rx.try_recv().unwrap();
        assert_eq!(
            msg2,
            HarnessClientMessage::KillJob {
                session_id: session_id.clone(),
                job_id: "j-1".into()
            }
        );

        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_extract_produced_files_from_tool_calls() {
        let tools = vec![
            ToolCallItem {
                id: "t1".into(),
                tool_name: "write_file".into(),
                input: serde_json::json!({ "path": "src/main.rs", "content": "fn main() {}" }),
                output: Some(serde_json::json!({ "success": true })),
                status: ToolStatus::Success,
                duration_ms: 25,
            },
            ToolCallItem {
                id: "t2".into(),
                tool_name: "read_file".into(),
                input: serde_json::json!({ "path": "Cargo.toml" }),
                output: Some(serde_json::json!({ "content": "[package]" })),
                status: ToolStatus::Success,
                duration_ms: 10,
            },
            ToolCallItem {
                id: "t3".into(),
                tool_name: "apply_patch".into(),
                input: serde_json::json!({ "path": "src/lib.rs", "patch": "..." }),
                output: None,
                status: ToolStatus::Failed,
                duration_ms: 5,
            },
            ToolCallItem {
                id: "t4".into(),
                tool_name: "apply_patch".into(),
                input: serde_json::json!({ "path": "src/config.rs", "patch": "..." }),
                output: Some(serde_json::json!({ "success": true })),
                status: ToolStatus::Success,
                duration_ms: 30,
            },
        ];

        let files = extract_produced_files(&tools);
        assert_eq!(
            files,
            vec!["src/main.rs".to_string(), "src/config.rs".to_string()]
        );
    }
}
