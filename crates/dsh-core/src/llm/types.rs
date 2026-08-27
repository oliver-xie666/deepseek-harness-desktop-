use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum LlmError {
    #[error("Missing API credential or key")]
    MissingCredential,

    #[error("Transport error: {0}")]
    Transport(String),

    #[error("Stream closed unexpectedly: {0}")]
    StreamClosed(String),

    #[error("Malformed SSE or JSON response: {0}")]
    MalformedResponse(String),

    #[error("Model returned empty response")]
    EmptyResponse,

    #[error("Unsupported reasoning effort: {0}")]
    UnsupportedReasoningEffort(String),

    #[error("Unsupported content block: {0}")]
    UnsupportedContent(String),

    #[error("API error (HTTP {status}): {message}")]
    ApiError {
        status: u16,
        message: String,
        code: Option<String>,
    },

    #[error("Request timed out")]
    Timeout,

    #[error("Request aborted")]
    Aborted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LlmRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ContentBlock {
    Text {
        text: String,
    },
    Reasoning {
        text: String,
    },
    ToolCall {
        id: String,
        name: String,
        arguments: String,
    },
    ToolResult {
        tool_call_id: String,
        content: String,
        #[serde(default)]
        is_error: bool,
    },
}

impl ContentBlock {
    pub fn text(t: impl Into<String>) -> Self {
        Self::Text { text: t.into() }
    }

    pub fn reasoning(r: impl Into<String>) -> Self {
        Self::Reasoning { text: r.into() }
    }

    pub fn tool_call(
        id: impl Into<String>,
        name: impl Into<String>,
        arguments: impl Into<String>,
    ) -> Self {
        Self::ToolCall {
            id: id.into(),
            name: name.into(),
            arguments: arguments.into(),
        }
    }

    pub fn tool_result(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self::ToolResult {
            tool_call_id: tool_call_id.into(),
            content: content.into(),
            is_error: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LlmMessage {
    pub role: LlmRole,
    pub content: Vec<ContentBlock>,
}

impl LlmMessage {
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: LlmRole::User,
            content: vec![ContentBlock::text(text)],
        }
    }

    pub fn system(text: impl Into<String>) -> Self {
        Self {
            role: LlmRole::System,
            content: vec![ContentBlock::text(text)],
        }
    }

    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            role: LlmRole::Assistant,
            content: vec![ContentBlock::text(text)],
        }
    }

    pub fn assistant_with_blocks(blocks: Vec<ContentBlock>) -> Self {
        Self {
            role: LlmRole::Assistant,
            content: blocks,
        }
    }

    pub fn tool_result(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: LlmRole::Tool,
            content: vec![ContentBlock::tool_result(tool_call_id, content)],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub r#type: String,
    pub function: ToolFunction,
}

impl ToolDefinition {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: serde_json::Value,
    ) -> Self {
        Self {
            r#type: "function".to_string(),
            function: ToolFunction {
                name: name.into(),
                description: description.into(),
                parameters,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct GenerateOptions {
    pub model: String,
    pub messages: Vec<LlmMessage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<ToolDefinition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    pub input_tokens: usize,
    pub output_tokens: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_read_tokens: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum FinishReason {
    Stop,
    ToolCalls,
    MaxTokens,
    Error { message: String, code: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum StreamChunk {
    BlockStart {
        index: usize,
        block_type: String,
    },
    BlockEnd {
        index: usize,
        block: ContentBlock,
    },
    TextDelta {
        index: usize,
        text: String,
    },
    ReasoningDelta {
        index: usize,
        text: String,
    },
    ToolCallDelta {
        index: usize,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        arguments_delta: String,
    },
    Usage {
        usage: TokenUsage,
    },
    Finish {
        reason: FinishReason,
    },
}

// ---------------- Wire types for OpenAI / DeepSeek format ----------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WireMessage {
    pub role: String,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<WireToolCall>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WireToolCall {
    pub id: String,
    pub r#type: String,
    pub function: WireFunctionCall,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WireFunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WireThinking {
    pub r#type: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WireStreamOptions {
    pub include_usage: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WireRequest {
    pub model: String,
    pub messages: Vec<WireMessage>,
    pub stream: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<WireStreamOptions>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thinking: Option<WireThinking>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct WireDeltaFunction {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub arguments: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct WireDeltaToolCall {
    #[serde(default)]
    pub index: usize,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub function: Option<WireDeltaFunction>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct WireDelta {
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub reasoning_content: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<Vec<WireDeltaToolCall>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct WireChoice {
    #[serde(default)]
    pub index: usize,
    #[serde(default)]
    pub delta: Option<WireDelta>,
    #[serde(default)]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct WirePromptTokensDetails {
    #[serde(default)]
    pub cached_tokens: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct WireCompletionTokensDetails {
    #[serde(default)]
    pub reasoning_tokens: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct WireUsage {
    #[serde(default)]
    pub prompt_tokens: usize,
    #[serde(default)]
    pub completion_tokens: usize,
    #[serde(default)]
    pub total_tokens: usize,
    #[serde(default)]
    pub prompt_cache_hit_tokens: Option<usize>,
    #[serde(default)]
    pub prompt_cache_miss_tokens: Option<usize>,
    #[serde(default)]
    pub prompt_tokens_details: Option<WirePromptTokensDetails>,
    #[serde(default)]
    pub completion_tokens_details: Option<WireCompletionTokensDetails>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct WireChunk {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub choices: Option<Vec<WireChoice>>,
    #[serde(default)]
    pub usage: Option<WireUsage>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WireErrorDetail {
    pub message: String,
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WireError {
    pub error: WireErrorDetail,
}
