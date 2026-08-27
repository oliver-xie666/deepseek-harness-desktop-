use super::types::{
    ContentBlock, GenerateOptions, LlmError, LlmMessage, LlmRole, WireFunctionCall, WireMessage,
    WireRequest, WireStreamOptions, WireThinking, WireToolCall,
};

fn flatten_text(blocks: &[ContentBlock]) -> String {
    blocks
        .iter()
        .filter_map(|b| match b {
            ContentBlock::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("")
}

fn serialize_assistant(message: &LlmMessage) -> WireMessage {
    let text = flatten_text(&message.content);
    let reasoning = message
        .content
        .iter()
        .filter_map(|b| match b {
            ContentBlock::Reasoning { text } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("");

    let tool_calls: Vec<WireToolCall> = message
        .content
        .iter()
        .filter_map(|b| match b {
            ContentBlock::ToolCall {
                id,
                name,
                arguments,
            } => Some(WireToolCall {
                id: id.clone(),
                r#type: "function".to_string(),
                function: WireFunctionCall {
                    name: name.clone(),
                    arguments: arguments.clone(),
                },
            }),
            _ => None,
        })
        .collect();

    let has_tools = !tool_calls.is_empty();
    let has_reasoning = !reasoning.is_empty();

    WireMessage {
        role: "assistant".to_string(),
        content: text,
        reasoning_content: if has_tools && has_reasoning {
            Some(reasoning)
        } else {
            None
        },
        tool_calls: if has_tools { Some(tool_calls) } else { None },
        tool_call_id: None,
    }
}

pub fn serialize_messages(messages: &[LlmMessage]) -> Vec<WireMessage> {
    let mut wire = Vec::new();
    for msg in messages {
        match msg.role {
            LlmRole::System => {
                wire.push(WireMessage {
                    role: "system".to_string(),
                    content: flatten_text(&msg.content),
                    reasoning_content: None,
                    tool_calls: None,
                    tool_call_id: None,
                });
            }
            LlmRole::Assistant => {
                wire.push(serialize_assistant(msg));
            }
            LlmRole::User => {
                let tool_results: Vec<&ContentBlock> = msg
                    .content
                    .iter()
                    .filter(|b| matches!(b, ContentBlock::ToolResult { .. }))
                    .collect();

                let text = flatten_text(&msg.content);
                if !text.is_empty() || tool_results.is_empty() {
                    wire.push(WireMessage {
                        role: "user".to_string(),
                        content: text,
                        reasoning_content: None,
                        tool_calls: None,
                        tool_call_id: None,
                    });
                }

                for res in tool_results {
                    if let ContentBlock::ToolResult {
                        tool_call_id,
                        content,
                        ..
                    } = res
                    {
                        let output = if content.is_empty() {
                            "(no output)".to_string()
                        } else {
                            content.clone()
                        };
                        wire.push(WireMessage {
                            role: "tool".to_string(),
                            content: output,
                            reasoning_content: None,
                            tool_calls: None,
                            tool_call_id: Some(tool_call_id.clone()),
                        });
                    }
                }
            }
            LlmRole::Tool => {
                for block in &msg.content {
                    if let ContentBlock::ToolResult {
                        tool_call_id,
                        content,
                        ..
                    } = block
                    {
                        let output = if content.is_empty() {
                            "(no output)".to_string()
                        } else {
                            content.clone()
                        };
                        wire.push(WireMessage {
                            role: "tool".to_string(),
                            content: output,
                            reasoning_content: None,
                            tool_calls: None,
                            tool_call_id: Some(tool_call_id.clone()),
                        });
                    }
                }
            }
        }
    }
    wire
}

pub fn serialize_request(options: &GenerateOptions) -> Result<WireRequest, LlmError> {
    let mut messages = Vec::new();
    if let Some(sys) = &options.system {
        if !sys.is_empty() {
            messages.push(WireMessage {
                role: "system".to_string(),
                content: sys.clone(),
                reasoning_content: None,
                tool_calls: None,
                tool_call_id: None,
            });
        }
    }
    messages.extend(serialize_messages(&options.messages));

    let tools = if options.tools.is_empty() {
        None
    } else {
        Some(options.tools.clone())
    };

    let (thinking, reasoning_effort) = match options.reasoning_effort.as_deref() {
        Some("off") => (
            Some(WireThinking {
                r#type: "disabled".to_string(),
            }),
            None,
        ),
        Some("high") | Some("max") => (
            Some(WireThinking {
                r#type: "enabled".to_string(),
            }),
            options.reasoning_effort.clone(),
        ),
        Some(other) => {
            return Err(LlmError::UnsupportedReasoningEffort(other.to_string()));
        }
        None => {
            if let Some(th) = &options.thinking {
                (Some(WireThinking { r#type: th.clone() }), None)
            } else {
                (None, None)
            }
        }
    };

    Ok(WireRequest {
        model: options.model.clone(),
        messages,
        stream: true,
        stream_options: Some(WireStreamOptions {
            include_usage: true,
        }),
        thinking,
        reasoning_effort,
        tools,
        temperature: options.temperature,
        max_tokens: options.max_tokens,
        stop: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::types::*;

    #[test]
    fn test_serialize_user_and_assistant_message() {
        let msgs = vec![
            LlmMessage::user("Hello DeepSeek"),
            LlmMessage::assistant("Hello! How can I help?"),
        ];
        let wire = serialize_messages(&msgs);
        assert_eq!(wire.len(), 2);
        assert_eq!(wire[0].role, "user");
        assert_eq!(wire[0].content, "Hello DeepSeek");
        assert_eq!(wire[1].role, "assistant");
        assert_eq!(wire[1].content, "Hello! How can I help?");
    }

    #[test]
    fn test_serialize_assistant_tool_call_passback_rule() {
        let msg = LlmMessage::assistant_with_blocks(vec![
            ContentBlock::reasoning("Let's read the file first"),
            ContentBlock::tool_call("call_1", "read_file", r#"{"path":"main.rs"}"#),
        ]);
        let wire = serialize_messages(&[msg]);
        assert_eq!(wire.len(), 1);
        assert_eq!(wire[0].role, "assistant");
        assert_eq!(wire[0].content, "");
        assert_eq!(
            wire[0].reasoning_content.as_deref(),
            Some("Let's read the file first")
        );
        assert!(wire[0].tool_calls.is_some());
        let tc = wire[0].tool_calls.as_ref().unwrap();
        assert_eq!(tc[0].id, "call_1");
        assert_eq!(tc[0].function.name, "read_file");
    }

    #[test]
    fn test_serialize_tool_result_messages() {
        let msg = LlmMessage {
            role: LlmRole::User,
            content: vec![
                ContentBlock::text("Here is the tool output:"),
                ContentBlock::tool_result("call_1", "file content xyz"),
            ],
        };
        let wire = serialize_messages(&[msg]);
        assert_eq!(wire.len(), 2);
        assert_eq!(wire[0].role, "user");
        assert_eq!(wire[0].content, "Here is the tool output:");
        assert_eq!(wire[1].role, "tool");
        assert_eq!(wire[1].tool_call_id.as_deref(), Some("call_1"));
        assert_eq!(wire[1].content, "file content xyz");
    }

    #[test]
    fn test_serialize_request_with_thinking() {
        let options = GenerateOptions {
            model: "deepseek-reasoner".to_string(),
            messages: vec![LlmMessage::user("Think deeply")],
            system: Some("You are a helpful assistant".to_string()),
            reasoning_effort: Some("high".to_string()),
            ..Default::default()
        };
        let req = serialize_request(&options).unwrap();
        assert_eq!(req.model, "deepseek-reasoner");
        assert_eq!(req.messages.len(), 2);
        assert_eq!(req.messages[0].role, "system");
        assert_eq!(req.thinking.as_ref().unwrap().r#type, "enabled");
        assert_eq!(req.reasoning_effort.as_deref(), Some("high"));
        assert!(req.stream);
        assert!(req.stream_options.as_ref().unwrap().include_usage);
    }
}
