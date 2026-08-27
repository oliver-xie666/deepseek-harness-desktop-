use super::context::{convert_session_messages_to_llm, SystemPromptBuilder};
use crate::llm::token_meter::TokenMeter;
use crate::llm::types::{ContentBlock, GenerateOptions, LlmError, StreamChunk};
use crate::llm::LlmClient;
use crate::tools::{ToolExecutionResult, ToolRegistry};
use crate::{ChatMessage, FileDiffItem, MessageSender, Session, ToolCallItem};
use chrono::Utc;
use dsh_protocol::{AgentState, HarnessServerEvent};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

pub struct AgentLoopConfig {
    pub max_turns: usize,
    pub model_name: String,
    pub temperature: f32,
    pub reasoning_effort: String,
}

impl Default for AgentLoopConfig {
    fn default() -> Self {
        Self {
            max_turns: 12,
            model_name: "gpt-5.6-luna".to_string(),
            temperature: 0.7,
            reasoning_effort: "high".to_string(),
        }
    }
}

pub struct NativeAgentLoop;

impl NativeAgentLoop {
    pub async fn run_turn(
        session: &mut Session,
        workspace: &Path,
        llm: Arc<LlmClient>,
        tools: Arc<ToolRegistry>,
        config: AgentLoopConfig,
        skills_prompt: Option<&str>,
        events_tx: mpsc::Sender<HarnessServerEvent>,
    ) -> Result<(), LlmError> {
        let mut turn_count = 0;
        let mut meter = TokenMeter::new();

        let plan_mode = session
            .plan_state
            .as_ref()
            .map(|p| p.active)
            .unwrap_or(false);

        loop {
            turn_count += 1;
            if turn_count > config.max_turns {
                info!("Agent loop reached maximum turns ({})", config.max_turns);
                break;
            }

            let system_prompt = SystemPromptBuilder::build(workspace, skills_prompt, plan_mode);
            let llm_messages = convert_session_messages_to_llm(&session.messages);
            let tool_defs = tools.list_tools();

            let assistant_msg_id = format!("msg-{}", uuid::Uuid::new_v4());

            // Notify thinking state
            let _ = events_tx
                .send(HarnessServerEvent::AgentStateChange {
                    session_id: session.id.clone(),
                    state: AgentState::Thinking,
                })
                .await;

            let options = GenerateOptions {
                model: config.model_name.clone(),
                messages: llm_messages,
                system: Some(system_prompt),
                tools: tool_defs,
                temperature: Some(config.temperature),
                reasoning_effort: Some(config.reasoning_effort.clone()),
                session_id: Some(session.id.clone()),
                ..Default::default()
            };

            let mut stream_rx = match llm.stream_request(options).await {
                Ok(rx) => rx,
                Err(e) => {
                    error!("Failed to start LLM stream: {}", e);
                    let _ = events_tx
                        .send(HarnessServerEvent::Error {
                            code: "LLM_REQUEST_FAILED".to_string(),
                            message: e.to_string(),
                        })
                        .await;
                    let _ = events_tx
                        .send(HarnessServerEvent::AgentStateChange {
                            session_id: session.id.clone(),
                            state: AgentState::Error,
                        })
                        .await;
                    return Err(e);
                }
            };

            let mut accumulated_reasoning = String::new();
            let mut accumulated_content = String::new();
            let mut assembled_tool_calls: Vec<ContentBlock> = Vec::new();

            while let Some(chunk_res) = stream_rx.recv().await {
                match chunk_res {
                    Ok(chunk) => match chunk {
                        StreamChunk::ReasoningDelta { text, .. } => {
                            meter.record_token_activity(&text);
                            accumulated_reasoning.push_str(&text);
                            let _ = events_tx
                                .send(HarnessServerEvent::ReasoningChunk {
                                    session_id: session.id.clone(),
                                    message_id: assistant_msg_id.clone(),
                                    text,
                                })
                                .await;
                        }
                        StreamChunk::TextDelta { text, .. } => {
                            meter.record_token_activity(&text);
                            accumulated_content.push_str(&text);
                            let _ = events_tx
                                .send(HarnessServerEvent::TokenChunk {
                                    session_id: session.id.clone(),
                                    message_id: assistant_msg_id.clone(),
                                    text,
                                })
                                .await;
                        }
                        StreamChunk::BlockEnd {
                            block:
                                ContentBlock::ToolCall {
                                    id,
                                    name,
                                    arguments,
                                },
                            ..
                        } => {
                            assembled_tool_calls.push(ContentBlock::ToolCall {
                                id,
                                name,
                                arguments,
                            });
                        }
                        StreamChunk::Usage { usage } => {
                            meter.update_usage(&usage);
                        }
                        StreamChunk::Finish { .. } => {}
                        _ => {}
                    },
                    Err(e) => {
                        warn!("Stream chunk error: {}", e);
                        let _ = events_tx
                            .send(HarnessServerEvent::Error {
                                code: "STREAM_ERROR".to_string(),
                                message: e.to_string(),
                            })
                            .await;
                    }
                }
            }

            // Create assistant message
            let mut executed_tools = Vec::new();

            if !assembled_tool_calls.is_empty() {
                let _ = events_tx
                    .send(HarnessServerEvent::AgentStateChange {
                        session_id: session.id.clone(),
                        state: AgentState::ExecutingTool,
                    })
                    .await;

                for block in assembled_tool_calls {
                    if let ContentBlock::ToolCall {
                        id,
                        name,
                        arguments,
                    } = block
                    {
                        let input_val: serde_json::Value = serde_json::from_str(&arguments)
                            .unwrap_or_else(|_| serde_json::json!({ "raw": arguments }));

                        let _ = events_tx
                            .send(HarnessServerEvent::ToolCallStart {
                                session_id: session.id.clone(),
                                call_id: id.clone(),
                                tool_name: name.clone(),
                                input: input_val.clone(),
                            })
                            .await;

                        let result: ToolExecutionResult =
                            tools.dispatch(workspace, &name, &input_val).await;

                        if let Some(diff_content) = result.diff {
                            let diff_id = format!("diff-{}", uuid::Uuid::new_v4());
                            let file_path = result
                                .produced_file
                                .clone()
                                .unwrap_or_else(|| "modified_file".to_string());

                            session.diffs.insert(
                                diff_id.clone(),
                                FileDiffItem {
                                    id: diff_id.clone(),
                                    file_path: file_path.clone(),
                                    diff_content: diff_content.clone(),
                                    accepted: None,
                                },
                            );

                            let _ = events_tx
                                .send(HarnessServerEvent::FileDiffReady {
                                    session_id: session.id.clone(),
                                    diff_id,
                                    file_path,
                                    diff_content,
                                })
                                .await;
                        }

                        let _ = events_tx
                            .send(HarnessServerEvent::ToolCallEnd {
                                session_id: session.id.clone(),
                                call_id: id.clone(),
                                output: result.output.clone(),
                                status: result.status,
                                duration_ms: result.duration_ms,
                            })
                            .await;

                        executed_tools.push(ToolCallItem {
                            id,
                            tool_name: name,
                            input: input_val,
                            output: Some(result.output),
                            status: result.status,
                            duration_ms: result.duration_ms,
                        });
                    }
                }
            }

            let assistant_message = ChatMessage {
                id: assistant_msg_id,
                sender: MessageSender::Assistant,
                content: accumulated_content,
                reasoning: if accumulated_reasoning.is_empty() {
                    None
                } else {
                    Some(accumulated_reasoning)
                },
                tool_calls: executed_tools.clone(),
                created_at: Utc::now(),
            };

            session.messages.push(assistant_message);

            // If no tools were called in this turn, the loop completes!
            if executed_tools.is_empty() {
                break;
            }
        }

        session.updated_at = Utc::now();
        session.agent_state = Some(AgentState::Completed);

        let _ = events_tx
            .send(HarnessServerEvent::AgentStateChange {
                session_id: session.id.clone(),
                state: AgentState::Completed,
            })
            .await;

        Ok(())
    }
}
