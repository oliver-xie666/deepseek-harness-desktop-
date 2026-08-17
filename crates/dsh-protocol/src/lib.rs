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
    CancelExecution {
        session_id: String,
    },
    /// User accepts generated file diff
    AcceptDiff {
        session_id: String,
        diff_id: String,
    },
    /// User rejects generated file diff
    RejectDiff {
        session_id: String,
        diff_id: String,
    },
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
    /// Realtime terminal/shell execution log
    TerminalLog {
        session_id: String,
        line: String,
        is_stderr: bool,
    },
    /// Pong health response
    Pong,
    /// Error notification
    Error {
        code: String,
        message: String,
    },
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
    fn test_server_event_roundtrip() {
        let event = HarnessServerEvent::TokenChunk {
            session_id: "sess-123".to_string(),
            message_id: "msg-456".to_string(),
            text: "fn main() {".to_string(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let parsed: HarnessServerEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, parsed);
    }
}
