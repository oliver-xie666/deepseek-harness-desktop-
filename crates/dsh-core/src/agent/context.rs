use crate::llm::types::{ContentBlock, LlmMessage, LlmRole};
use crate::ChatMessage;
use chrono::Utc;
use std::path::Path;

pub struct SystemPromptBuilder;

impl SystemPromptBuilder {
    pub fn build(workspace: &Path, skills_prompt: Option<&str>, plan_mode: bool) -> String {
        let mut prompt = String::new();
        prompt.push_str("You are DeepSeek Harness Native Agent, an expert AI software engineer and coding assistant.\n\n");
        prompt.push_str(&format!("## Environment Context\n- Workspace Root: {}\n- Operating System: {}\n- Current Date: {}\n\n",
            workspace.display(),
            std::env::consts::OS,
            Utc::now().format("%Y-%m-%d")
        ));

        prompt.push_str("## Core Capabilities & Tool Guidelines\n");
        prompt.push_str(
            "- Always examine existing code with `read_file` or `grep_search` before modifying.\n",
        );
        prompt.push_str("- When creating new files, use `write_file`.\n");
        prompt.push_str("- When editing existing files, prefer `edit_file` (unique string replacement) or `apply_patch`.\n");
        prompt.push_str("- When running commands or tests, use `exec_command`.\n");
        prompt.push_str(
            "- Be precise, surgical, and avoid unnecessary destructive modifications.\n\n",
        );

        if plan_mode {
            prompt.push_str("## Plan Mode Active\n");
            prompt.push_str("- Break down complex tasks into numbered, actionable steps.\n");
            prompt.push_str("- Provide clear intermediate checkpoints and explain your architectural rationale.\n\n");
        }

        if let Some(skills) = skills_prompt {
            if !skills.is_empty() {
                prompt.push_str("## Available Skills & Extended Plugins\n");
                prompt.push_str(skills);
                prompt.push_str("\n\n");
            }
        }

        prompt
    }
}

pub fn convert_session_messages_to_llm(messages: &[ChatMessage]) -> Vec<LlmMessage> {
    let mut llm_messages = Vec::new();

    for msg in messages {
        match msg.sender {
            crate::MessageSender::User => {
                llm_messages.push(LlmMessage::user(&msg.content));
            }
            crate::MessageSender::System => {
                llm_messages.push(LlmMessage::system(&msg.content));
            }
            crate::MessageSender::Assistant => {
                let mut blocks = Vec::new();

                if let Some(r) = &msg.reasoning {
                    if !r.is_empty() {
                        blocks.push(ContentBlock::reasoning(r.clone()));
                    }
                }

                if !msg.content.is_empty() {
                    blocks.push(ContentBlock::text(msg.content.clone()));
                }

                for tc in &msg.tool_calls {
                    let args_str = serde_json::to_string(&tc.input).unwrap_or_default();
                    blocks.push(ContentBlock::tool_call(
                        tc.id.clone(),
                        tc.tool_name.clone(),
                        args_str,
                    ));
                }

                llm_messages.push(LlmMessage {
                    role: LlmRole::Assistant,
                    content: blocks,
                });

                // Follow with tool results
                for tc in &msg.tool_calls {
                    if let Some(output) = &tc.output {
                        let out_str = serde_json::to_string(output).unwrap_or_default();
                        llm_messages.push(LlmMessage::tool_result(tc.id.clone(), out_str));
                    }
                }
            }
        }
    }

    llm_messages
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ToolCallItem;
    use dsh_protocol::ToolStatus;

    #[test]
    fn test_convert_session_messages() {
        let msgs = vec![
            ChatMessage {
                id: "1".into(),
                sender: crate::MessageSender::User,
                content: "Read main.rs".into(),
                reasoning: None,
                tool_calls: vec![],
                created_at: Utc::now(),
            },
            ChatMessage {
                id: "2".into(),
                sender: crate::MessageSender::Assistant,
                content: "Let me check".into(),
                reasoning: Some("Need to inspect main".into()),
                tool_calls: vec![ToolCallItem {
                    id: "call_1".into(),
                    tool_name: "read_file".into(),
                    input: serde_json::json!({ "path": "main.rs" }),
                    output: Some(serde_json::json!({ "lines": 10 })),
                    status: ToolStatus::Success,
                    duration_ms: 15,
                }],
                created_at: Utc::now(),
            },
        ];

        let converted = convert_session_messages_to_llm(&msgs);
        assert_eq!(converted.len(), 3); // User, Assistant, ToolResult
        assert_eq!(converted[0].role, LlmRole::User);
        assert_eq!(converted[1].role, LlmRole::Assistant);
        assert_eq!(converted[2].role, LlmRole::Tool);
    }
}
