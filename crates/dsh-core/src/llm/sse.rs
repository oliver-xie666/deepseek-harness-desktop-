use super::types::LlmError;

pub const DONE: &str = "[DONE]";

#[derive(Default)]
pub struct SseParser {
    buffer: String,
    data_lines: Vec<String>,
}

impl SseParser {
    pub fn new() -> Self {
        Self::default()
    }

    /// Feed a raw text chunk into the SSE parser, returning complete data payloads.
    pub fn feed(&mut self, chunk: &str) -> Vec<String> {
        self.buffer.push_str(chunk);
        let mut payloads = Vec::new();

        while let Some(line_end) = self.buffer.find('\n') {
            let mut line = self.buffer[..line_end].to_string();
            self.buffer = self.buffer[line_end + 1..].to_string();

            if line.ends_with('\r') {
                line.pop();
            }

            if line.is_empty() {
                // Dispatch event
                if !self.data_lines.is_empty() {
                    let payload = self.data_lines.join("\n");
                    self.data_lines.clear();
                    payloads.push(payload);
                }
            } else if line.starts_with(':') {
                // SSE comment, ignore
                continue;
            } else if let Some(stripped) = line.strip_prefix("data:") {
                let data = stripped.strip_prefix(' ').unwrap_or(stripped);
                self.data_lines.push(data.to_string());
            }
        }

        payloads
    }

    /// Flush any remaining payload if terminated.
    pub fn finish(&mut self) -> Result<Vec<String>, LlmError> {
        let mut payloads = Vec::new();
        if !self.data_lines.is_empty() {
            let payload = self.data_lines.join("\n");
            self.data_lines.clear();
            payloads.push(payload);
        }
        self.buffer.clear();
        Ok(payloads)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sse_basic_parsing() {
        let mut parser = SseParser::new();
        let chunk1 = "data: {\"choices\": [{\"delta\": {\"content\": \"hello\"}}]}\n\n";
        let payloads = parser.feed(chunk1);
        assert_eq!(payloads.len(), 1);
        assert_eq!(
            payloads[0],
            "{\"choices\": [{\"delta\": {\"content\": \"hello\"}}]}"
        );

        let chunk2 = "data: [DONE]\n\n";
        let payloads2 = parser.feed(chunk2);
        assert_eq!(payloads2.len(), 1);
        assert_eq!(payloads2[0], DONE);
    }

    #[test]
    fn test_sse_split_chunks() {
        let mut parser = SseParser::new();
        assert!(parser.feed("data: {\"test\":").is_empty());
        assert!(parser.feed(" \"split\"}\n").is_empty());
        let payloads = parser.feed("\n");
        assert_eq!(payloads.len(), 1);
        assert_eq!(payloads[0], "{\"test\": \"split\"}");
    }

    #[test]
    fn test_sse_comments_and_crlf() {
        let mut parser = SseParser::new();
        let chunk = ": ping\r\ndata: 123\r\n\r\n: keepalive\r\ndata: [DONE]\r\n\r\n";
        let payloads = parser.feed(chunk);
        assert_eq!(payloads.len(), 2);
        assert_eq!(payloads[0], "123");
        assert_eq!(payloads[1], "[DONE]");
    }
}
