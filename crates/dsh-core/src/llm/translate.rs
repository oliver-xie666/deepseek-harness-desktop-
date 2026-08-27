use super::sse::DONE;
use super::types::{
    ContentBlock, FinishReason, LlmError, StreamChunk, TokenUsage, WireChunk, WireUsage,
};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
enum BlockKind {
    Text,
    Reasoning,
    ToolCall,
}

#[derive(Debug, Clone)]
struct OpenBlock {
    index: usize,
    kind: BlockKind,
    text: String,
    call_id: Option<String>,
    name: Option<String>,
}

impl OpenBlock {
    fn to_content_block(&self) -> ContentBlock {
        match self.kind {
            BlockKind::Text => ContentBlock::Text {
                text: self.text.clone(),
            },
            BlockKind::Reasoning => ContentBlock::Reasoning {
                text: self.text.clone(),
            },
            BlockKind::ToolCall => ContentBlock::ToolCall {
                id: self.call_id.clone().unwrap_or_default(),
                name: self.name.clone().unwrap_or_default(),
                arguments: self.text.clone(),
            },
        }
    }
}

pub fn map_finish_reason(reason: &str) -> FinishReason {
    match reason {
        "stop" => FinishReason::Stop,
        "tool_calls" => FinishReason::ToolCalls,
        "length" => FinishReason::MaxTokens,
        other => FinishReason::Error {
            message: format!("model stopped: {}", other),
            code: other.to_uppercase(),
        },
    }
}

pub fn map_usage(usage: &WireUsage) -> TokenUsage {
    let cache_read = usage
        .prompt_tokens_details
        .as_ref()
        .and_then(|d| d.cached_tokens)
        .or(usage.prompt_cache_hit_tokens);

    let reasoning = usage
        .completion_tokens_details
        .as_ref()
        .and_then(|d| d.reasoning_tokens);

    let input_tokens = usage.prompt_tokens.saturating_sub(cache_read.unwrap_or(0));

    TokenUsage {
        input_tokens,
        output_tokens: usage.completion_tokens,
        cache_read_tokens: cache_read,
        reasoning_tokens: reasoning,
    }
}

pub struct StreamTranslator {
    next_index: usize,
    text_block: Option<usize>,
    reasoning_block: Option<usize>,
    tool_blocks: HashMap<usize, usize>, // wire index -> block order index
    order: Vec<OpenBlock>,
    pending_finish: Option<FinishReason>,
    pending_usage: Option<TokenUsage>,
    finished: bool,
}

impl Default for StreamTranslator {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamTranslator {
    pub fn new() -> Self {
        Self {
            next_index: 0,
            text_block: None,
            reasoning_block: None,
            tool_blocks: HashMap::new(),
            order: Vec::new(),
            pending_finish: None,
            pending_usage: None,
            finished: false,
        }
    }

    fn open(&mut self, kind: BlockKind) -> usize {
        let idx = self.next_index;
        self.next_index += 1;
        self.order.push(OpenBlock {
            index: idx,
            kind,
            text: String::new(),
            call_id: None,
            name: None,
        });
        idx
    }

    /// Process one data payload from SSE.
    pub fn process_payload(&mut self, payload: &str) -> Result<Vec<StreamChunk>, LlmError> {
        let mut chunks = Vec::new();

        if payload == DONE {
            self.finished = true;
            for block in &self.order {
                chunks.push(StreamChunk::BlockEnd {
                    index: block.index,
                    block: block.to_content_block(),
                });
            }
            if let Some(usage) = self.pending_usage.take() {
                chunks.push(StreamChunk::Usage { usage });
            }
            let reason = self.pending_finish.take().unwrap_or(FinishReason::Stop);

            if reason == FinishReason::Stop && self.order.is_empty() {
                chunks.push(StreamChunk::Finish {
                    reason: FinishReason::Error {
                        message: "model returned a completed response with no content".to_string(),
                        code: "EMPTY_RESPONSE".to_string(),
                    },
                });
            } else {
                chunks.push(StreamChunk::Finish { reason });
            }
            return Ok(chunks);
        }

        let wire_chunk: WireChunk = serde_json::from_str(payload).map_err(|e| {
            LlmError::MalformedResponse(format!("JSON error: {}, payload: {}", e, payload))
        })?;

        if let Some(choices) = wire_chunk.choices {
            for choice in choices {
                if let Some(delta) = choice.delta {
                    // 1. Reasoning delta
                    if let Some(reasoning) = delta.reasoning_content {
                        if !reasoning.is_empty() {
                            let block_idx = match self.reasoning_block {
                                Some(idx) => idx,
                                None => {
                                    let idx = self.open(BlockKind::Reasoning);
                                    self.reasoning_block = Some(idx);
                                    chunks.push(StreamChunk::BlockStart {
                                        index: idx,
                                        block_type: "reasoning".to_string(),
                                    });
                                    idx
                                }
                            };
                            if let Some(b) = self.order.iter_mut().find(|b| b.index == block_idx) {
                                b.text.push_str(&reasoning);
                            }
                            chunks.push(StreamChunk::ReasoningDelta {
                                index: block_idx,
                                text: reasoning,
                            });
                        }
                    }

                    // 2. Content text delta
                    if let Some(content) = delta.content {
                        if !content.is_empty() {
                            let block_idx = match self.text_block {
                                Some(idx) => idx,
                                None => {
                                    let idx = self.open(BlockKind::Text);
                                    self.text_block = Some(idx);
                                    chunks.push(StreamChunk::BlockStart {
                                        index: idx,
                                        block_type: "text".to_string(),
                                    });
                                    idx
                                }
                            };
                            if let Some(b) = self.order.iter_mut().find(|b| b.index == block_idx) {
                                b.text.push_str(&content);
                            }
                            chunks.push(StreamChunk::TextDelta {
                                index: block_idx,
                                text: content,
                            });
                        }
                    }

                    // 3. Tool call delta
                    if let Some(tool_calls) = delta.tool_calls {
                        for call in tool_calls {
                            let wire_idx = call.index;
                            let block_idx = match self.tool_blocks.get(&wire_idx) {
                                Some(&idx) => idx,
                                None => {
                                    let idx = self.open(BlockKind::ToolCall);
                                    self.tool_blocks.insert(wire_idx, idx);
                                    chunks.push(StreamChunk::BlockStart {
                                        index: idx,
                                        block_type: "tool-call".to_string(),
                                    });
                                    idx
                                }
                            };

                            let fragment = call
                                .function
                                .as_ref()
                                .and_then(|f| f.arguments.as_deref())
                                .unwrap_or_default()
                                .to_string();

                            let mut id_out = None;
                            let mut name_out = None;

                            if let Some(b) = self.order.iter_mut().find(|b| b.index == block_idx) {
                                if let Some(id) = call.id {
                                    b.call_id = Some(id.clone());
                                    id_out = Some(id);
                                }
                                if let Some(func) = call.function {
                                    if let Some(name) = func.name {
                                        b.name = Some(name.clone());
                                        name_out = Some(name);
                                    }
                                }
                                b.text.push_str(&fragment);
                            }

                            chunks.push(StreamChunk::ToolCallDelta {
                                index: block_idx,
                                id: id_out,
                                name: name_out,
                                arguments_delta: fragment,
                            });
                        }
                    }
                }

                if let Some(reason_str) = choice.finish_reason {
                    self.pending_finish = Some(map_finish_reason(&reason_str));
                }
            }
        }

        if let Some(usage) = wire_chunk.usage {
            self.pending_usage = Some(map_usage(&usage));
        }

        Ok(chunks)
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_text_stream() {
        let mut translator = StreamTranslator::new();
        let c1 = r#"{"choices":[{"delta":{"content":"Hello "}}]}"#;
        let c2 = r#"{"choices":[{"delta":{"content":"World!"},"finish_reason":"stop"}],"usage":{"prompt_tokens":10,"completion_tokens":5,"total_tokens":15}}"#;

        let res1 = translator.process_payload(c1).unwrap();
        assert_eq!(res1.len(), 2);
        assert!(
            matches!(&res1[0], StreamChunk::BlockStart { block_type, .. } if block_type == "text")
        );
        assert!(matches!(&res1[1], StreamChunk::TextDelta { text, .. } if text == "Hello "));

        let res2 = translator.process_payload(c2).unwrap();
        assert_eq!(res2.len(), 1);
        assert!(matches!(&res2[0], StreamChunk::TextDelta { text, .. } if text == "World!"));

        let res_done = translator.process_payload(DONE).unwrap();
        assert_eq!(res_done.len(), 3); // BlockEnd, Usage, Finish
        assert!(
            matches!(&res_done[0], StreamChunk::BlockEnd { block: ContentBlock::Text { text }, .. } if text == "Hello World!")
        );
        assert!(
            matches!(&res_done[1], StreamChunk::Usage { usage } if usage.input_tokens == 10 && usage.output_tokens == 5)
        );
        assert!(matches!(
            &res_done[2],
            StreamChunk::Finish {
                reason: FinishReason::Stop
            }
        ));
    }

    #[test]
    fn test_translate_reasoning_stream() {
        let mut translator = StreamTranslator::new();
        let c1 = r#"{"choices":[{"delta":{"reasoning_content":"Thinking deeply..."}}]}"#;
        let c2 = r#"{"choices":[{"delta":{"content":"Result."},"finish_reason":"stop"}]}"#;

        let res1 = translator.process_payload(c1).unwrap();
        assert_eq!(res1.len(), 2);
        assert!(
            matches!(&res1[0], StreamChunk::BlockStart { block_type, .. } if block_type == "reasoning")
        );
        assert!(
            matches!(&res1[1], StreamChunk::ReasoningDelta { text, .. } if text == "Thinking deeply...")
        );

        let res2 = translator.process_payload(c2).unwrap();
        assert_eq!(res2.len(), 2);
        assert!(
            matches!(&res2[0], StreamChunk::BlockStart { block_type, .. } if block_type == "text")
        );
        assert!(matches!(&res2[1], StreamChunk::TextDelta { text, .. } if text == "Result."));

        let res_done = translator.process_payload(DONE).unwrap();
        assert_eq!(res_done.len(), 3); // 2 BlockEnds (reasoning + text), 1 Finish
    }

    #[test]
    fn test_translate_tool_call_stream() {
        let mut translator = StreamTranslator::new();
        let c1 = r#"{"choices":[{"delta":{"tool_calls":[{"index":0,"id":"call_99","type":"function","function":{"name":"write_file","arguments":"{\"path\":"}}]}}]}"#;
        let c2 = r#"{"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"arguments":"\"app.rs\"}"}}]}}]}"#;
        let c3 = r#"{"choices":[{"finish_reason":"tool_calls"}]}"#;

        let res1 = translator.process_payload(c1).unwrap();
        assert_eq!(res1.len(), 2); // BlockStart + ToolCallDelta

        let res2 = translator.process_payload(c2).unwrap();
        assert_eq!(res2.len(), 1); // ToolCallDelta

        let res3 = translator.process_payload(c3).unwrap();
        assert_eq!(res3.len(), 0);

        let res_done = translator.process_payload(DONE).unwrap();
        assert_eq!(res_done.len(), 2); // BlockEnd + Finish(ToolCalls)
        if let StreamChunk::BlockEnd {
            block:
                ContentBlock::ToolCall {
                    id,
                    name,
                    arguments,
                },
            ..
        } = &res_done[0]
        {
            assert_eq!(id, "call_99");
            assert_eq!(name, "write_file");
            assert_eq!(arguments, r#"{"path":"app.rs"}"#);
        } else {
            panic!("Expected ToolCall block");
        }
        assert!(matches!(
            &res_done[1],
            StreamChunk::Finish {
                reason: FinishReason::ToolCalls
            }
        ));
    }
}
