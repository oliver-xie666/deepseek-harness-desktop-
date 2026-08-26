use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Client -> Server messages
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum HarnessClientMessage {
    /// Initialize or switch workspace
    InitSession {
        workspace_path: String,
        mode: String,
    },
    /// Send user prompt to active session
    SendPrompt {
        session_id: String,
        text: String,
        #[serde(default)]
        attachments: Vec<String>,
    },
    /// Cancel in-flight generation
    CancelExecution { session_id: String },
    /// User accepts generated file diff
    AcceptDiff { session_id: String, diff_id: String },
    /// User rejects generated file diff
    RejectDiff { session_id: String, diff_id: String },
    /// Toggle Plan mode for session
    TogglePlanMode { session_id: String, enabled: bool },
    /// User answers an interactive question prompt
    AnswerQuestion {
        session_id: String,
        question_id: String,
        selected: Vec<String>,
        #[serde(default)]
        custom_text: Option<String>,
    },
    /// Update or change session goal
    UpdateGoal {
        session_id: String,
        objective: String,
        phase: GoalPhase,
    },
    /// Clear or remove session goal
    ClearGoal { session_id: String },
    /// Stop or kill background job
    KillJob { session_id: String, job_id: String },
    /// Ping daemon for health status
    Ping,
}

/// Server -> Client events
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum HarnessServerEvent {
    /// Session created or restored
    SessionCreated {
        session_id: String,
        workspace_path: String,
    },
    /// Incremental token chunk from LLM
    TokenChunk {
        session_id: String,
        message_id: String,
        text: String,
    },
    /// Tool call has begun execution
    ToolCallStart {
        session_id: String,
        call_id: String,
        tool_name: String,
        input: serde_json::Value,
    },
    /// Tool call execution finished
    ToolCallEnd {
        session_id: String,
        call_id: String,
        output: serde_json::Value,
        status: ToolStatus,
        duration_ms: u64,
    },
    /// Diff ready for user inspection
    FileDiffReady {
        session_id: String,
        diff_id: String,
        file_path: String,
        diff_content: String,
    },
    /// Agent execution state change
    AgentStateChange {
        session_id: String,
        state: AgentState,
    },
    /// Plan mode status and markdown plan updates
    PlanUpdate {
        session_id: String,
        active: bool,
        plan_markdown: String,
    },
    /// Interactive question prompt asking for user decision
    QuestionPrompt {
        session_id: String,
        question_id: String,
        prompt: String,
        options: Vec<String>,
        multi_select: bool,
    },
    /// Goal status or objective update
    GoalUpdate {
        session_id: String,
        goal: Option<GoalItem>,
    },
    /// Single job status update
    JobUpdate { session_id: String, job: JobItem },
    /// Full job list snapshot update
    JobListUpdate {
        session_id: String,
        jobs: Vec<JobItem>,
    },
    /// Realtime terminal/shell execution log
    TerminalLog {
        session_id: String,
        line: String,
        is_stderr: bool,
    },
    /// Pong health response
    Pong,
    /// Error notification
    Error { code: String, message: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolStatus {
    Success,
    Failed,
    Running,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentState {
    Idle,
    Thinking,
    ExecutingTool,
    WaitingForApproval,
    Completed,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalPhase {
    Active,
    Paused,
    Blocked,
    Complete,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoalItem {
    pub id: String,
    pub objective: String,
    pub phase: GoalPhase,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Running,
    Stopping,
    Completed,
    Killed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JobItem {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub status: JobStatus,
    #[serde(default = "Utc::now")]
    pub started_at: DateTime<Utc>,
    #[serde(default)]
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PlanState {
    pub active: bool,
    pub plan_markdown: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuestionItem {
    pub id: String,
    pub prompt: String,
    pub options: Vec<String>,
    pub multi_select: bool,
    #[serde(default)]
    pub answered: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub id: String,
    pub title: String,
    pub workspace_path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_message_roundtrip() {
        let msg = HarnessClientMessage::SendPrompt {
            session_id: "sess-123".to_string(),
            text: "Hello DeepSeek".to_string(),
            attachments: vec!["src/main.rs".to_string()],
        };

        let json = serde_json::to_string(&msg).unwrap();
        let parsed: HarnessClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);
    }

    #[test]
    fn test_client_plan_and_question_roundtrip() {
        let plan_msg = HarnessClientMessage::TogglePlanMode {
            session_id: "sess-123".to_string(),
            enabled: true,
        };
        let json = serde_json::to_string(&plan_msg).unwrap();
        let parsed: HarnessClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(plan_msg, parsed);

        let answer_msg = HarnessClientMessage::AnswerQuestion {
            session_id: "sess-123".to_string(),
            question_id: "q-1".to_string(),
            selected: vec!["Option A".to_string()],
            custom_text: Some("Note".to_string()),
        };
        let json = serde_json::to_string(&answer_msg).unwrap();
        let parsed: HarnessClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(answer_msg, parsed);
    }

    #[test]
    fn test_client_goal_and_job_roundtrip() {
        let goal_msg = HarnessClientMessage::UpdateGoal {
            session_id: "sess-123".into(),
            objective: "Build parity".into(),
            phase: GoalPhase::Active,
        };
        let json = serde_json::to_string(&goal_msg).unwrap();
        let parsed: HarnessClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(goal_msg, parsed);

        let kill_msg = HarnessClientMessage::KillJob {
            session_id: "sess-123".into(),
            job_id: "job-1".into(),
        };
        let json2 = serde_json::to_string(&kill_msg).unwrap();
        let parsed2: HarnessClientMessage = serde_json::from_str(&json2).unwrap();
        assert_eq!(kill_msg, parsed2);
    }

    #[test]
    fn test_server_event_roundtrip() {
        let event = HarnessServerEvent::TokenChunk {
            session_id: "sess-123".to_string(),
            message_id: "msg-456".to_string(),
            text: "fn main() {".to_string(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let parsed: HarnessServerEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, parsed);

        let plan_event = HarnessServerEvent::PlanUpdate {
            session_id: "sess-123".to_string(),
            active: true,
            plan_markdown: "1. Step 1\n2. Step 2".to_string(),
        };
        let json = serde_json::to_string(&plan_event).unwrap();
        let parsed: HarnessServerEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(plan_event, parsed);

        let q_event = HarnessServerEvent::QuestionPrompt {
            session_id: "sess-123".to_string(),
            question_id: "q-1".to_string(),
            prompt: "Choose an approach".to_string(),
            options: vec!["Approach 1".to_string(), "Approach 2".to_string()],
            multi_select: false,
        };
        let json = serde_json::to_string(&q_event).unwrap();
        let parsed: HarnessServerEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(q_event, parsed);

        let goal_event = HarnessServerEvent::GoalUpdate {
            session_id: "sess-123".into(),
            goal: Some(GoalItem {
                id: "g-1".into(),
                objective: "Ship product".into(),
                phase: GoalPhase::Active,
                error: None,
            }),
        };
        let json = serde_json::to_string(&goal_event).unwrap();
        let parsed: HarnessServerEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(goal_event, parsed);

        let job_event = HarnessServerEvent::JobUpdate {
            session_id: "sess-123".into(),
            job: JobItem {
                id: "j-1".into(),
                kind: "agent".into(),
                label: "worker-subtask".into(),
                status: JobStatus::Running,
                started_at: Utc::now(),
                duration_ms: Some(1500),
            },
        };
        let json2 = serde_json::to_string(&job_event).unwrap();
        let parsed2: HarnessServerEvent = serde_json::from_str(&json2).unwrap();
        assert_eq!(job_event, parsed2);
    }
}
