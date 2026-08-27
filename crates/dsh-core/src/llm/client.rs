use super::serialize::serialize_request;
use super::sse::SseParser;
use super::translate::StreamTranslator;
use super::types::{GenerateOptions, LlmError, StreamChunk, WireError};
use futures::StreamExt;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error};

#[derive(Clone, Debug)]
pub struct LlmClientConfig {
    pub base_url: String,
    pub api_key: String,
    pub timeout_secs: u64,
}

impl Default for LlmClientConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.deepseek.com".to_string(),
            api_key: String::new(),
            timeout_secs: 180,
        }
    }
}

#[derive(Clone)]
pub struct LlmClient {
    http: reqwest::Client,
    config: LlmClientConfig,
}

impl LlmClient {
    pub fn new(config: LlmClientConfig) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .unwrap_or_default();

        Self { http, config }
    }

    pub fn config(&self) -> &LlmClientConfig {
        &self.config
    }

    pub fn set_api_key(&mut self, api_key: impl Into<String>) {
        self.config.api_key = api_key.into();
    }

    pub fn set_base_url(&mut self, base_url: impl Into<String>) {
        self.config.base_url = base_url.into();
    }

    /// Stream completion chunks from the model.
    pub async fn stream_request(
        &self,
        options: GenerateOptions,
    ) -> Result<mpsc::Receiver<Result<StreamChunk, LlmError>>, LlmError> {
        let wire_req = serialize_request(&options)?;
        let base_url = self.config.base_url.trim_end_matches('/');
        let url = if base_url.ends_with("/chat/completions") {
            base_url.to_string()
        } else {
            format!("{}/chat/completions", base_url)
        };

        let is_local = base_url.contains("localhost")
            || base_url.contains("127.0.0.1")
            || base_url.contains("ollama");
        if self.config.api_key.trim().is_empty() && !is_local {
            return Err(LlmError::MissingCredential);
        }

        let mut request_builder = self
            .http
            .post(&url)
            .header("content-type", "application/json")
            .header("accept", "text/event-stream")
            .header(
                "user-agent",
                "DeepSeek-Harness-Desktop/0.1.0 (Pure-Rust-Native)",
            );

        if !self.config.api_key.trim().is_empty() {
            request_builder = request_builder.header(
                "authorization",
                format!("Bearer {}", self.config.api_key.trim()),
            );
        }

        if let Some(session_id) = &options.session_id {
            request_builder = request_builder.header("x-deepseek-harness-session-id", session_id);
        }

        debug!(
            "Sending LLM stream request to {} with model: {}",
            url, options.model
        );

        let response = request_builder
            .json(&wire_req)
            .send()
            .await
            .map_err(|e| LlmError::Transport(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let status_code = status.as_u16();
            let body_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read response body".to_string());

            let (message, code) =
                if let Ok(err_payload) = serde_json::from_str::<WireError>(&body_text) {
                    (err_payload.error.message, err_payload.error.code)
                } else {
                    (body_text, None)
                };

            error!("LLM API error response: HTTP {} - {}", status_code, message);
            return Err(LlmError::ApiError {
                status: status_code,
                message,
                code,
            });
        }

        let (tx, rx) = mpsc::channel(64);
        let mut byte_stream = response.bytes_stream();

        tokio::spawn(async move {
            let mut sse_parser = SseParser::new();
            let mut translator = StreamTranslator::new();

            while let Some(chunk_res) = byte_stream.next().await {
                match chunk_res {
                    Ok(bytes) => {
                        let text = match std::str::from_utf8(&bytes) {
                            Ok(t) => t,
                            Err(_) => {
                                // fallback lossy string conversion
                                String::from_utf8_lossy(&bytes).to_string().leak()
                            }
                        };

                        let payloads = sse_parser.feed(text);
                        for payload in payloads {
                            match translator.process_payload(&payload) {
                                Ok(chunks) => {
                                    for chunk in chunks {
                                        if tx.send(Ok(chunk)).await.is_err() {
                                            return;
                                        }
                                    }
                                }
                                Err(err) => {
                                    let _ = tx.send(Err(err)).await;
                                    return;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(LlmError::Transport(e.to_string()))).await;
                        return;
                    }
                }
            }

            // End of stream check
            if !translator.is_finished() {
                // If stream ended without [DONE], flush remaining
                if let Ok(flushed) = sse_parser.finish() {
                    for payload in flushed {
                        if let Ok(chunks) = translator.process_payload(&payload) {
                            for chunk in chunks {
                                if tx.send(Ok(chunk)).await.is_err() {
                                    return;
                                }
                            }
                        }
                    }
                }
                if !translator.is_finished() {
                    let _ = tx
                        .send(Err(LlmError::StreamClosed(
                            "SSE stream ended before [DONE] sentinel".to_string(),
                        )))
                        .await;
                }
            }
        });

        Ok(rx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_missing_credential() {
        let config = LlmClientConfig {
            base_url: "https://api.deepseek.com".to_string(),
            api_key: "".to_string(),
            timeout_secs: 10,
        };
        let client = LlmClient::new(config);
        let opt = GenerateOptions {
            model: "gpt-5.6-luna".to_string(),
            ..Default::default()
        };
        let res = client.stream_request(opt).await;
        assert_eq!(res.err(), Some(LlmError::MissingCredential));
    }
}
