use super::types::TokenUsage;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct TokenMeterSnapshot {
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub cache_read_tokens: usize,
    pub reasoning_tokens: usize,
    pub ttft: Option<Duration>,
    pub total_duration: Duration,
    pub tokens_per_second: f64,
    pub cache_hit_rate: Option<f64>,
}

#[derive(Debug)]
pub struct TokenMeter {
    start_time: Instant,
    first_token_time: Option<Instant>,
    last_token_time: Option<Instant>,
    input_tokens: usize,
    output_tokens: usize,
    cache_read_tokens: usize,
    reasoning_tokens: usize,
    streamed_chars: usize,
}

impl Default for TokenMeter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenMeter {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            first_token_time: None,
            last_token_time: None,
            input_tokens: 0,
            output_tokens: 0,
            cache_read_tokens: 0,
            reasoning_tokens: 0,
            streamed_chars: 0,
        }
    }

    pub fn record_token_activity(&mut self, text: &str) {
        let now = Instant::now();
        if self.first_token_time.is_none() {
            self.first_token_time = Some(now);
        }
        self.last_token_time = Some(now);
        self.streamed_chars += text.chars().count();
    }

    pub fn update_usage(&mut self, usage: &TokenUsage) {
        self.input_tokens = usage.input_tokens;
        self.output_tokens = usage.output_tokens;
        if let Some(cache) = usage.cache_read_tokens {
            self.cache_read_tokens = cache;
        }
        if let Some(reasoning) = usage.reasoning_tokens {
            self.reasoning_tokens = reasoning;
        }
    }

    pub fn snapshot(&self) -> TokenMeterSnapshot {
        let now = Instant::now();
        let total_duration = now.saturating_duration_since(self.start_time);
        let ttft = self
            .first_token_time
            .map(|t| t.saturating_duration_since(self.start_time));

        let gen_duration = match (self.first_token_time, self.last_token_time) {
            (Some(first), Some(last)) if last > first => last.duration_since(first),
            (Some(_), _) => total_duration.saturating_sub(ttft.unwrap_or_default()),
            _ => Duration::from_millis(1),
        };

        let secs = gen_duration.as_secs_f64().max(0.001);
        let tokens_count = if self.output_tokens > 0 {
            self.output_tokens
        } else {
            // Rough estimation before wire usage arrives (~4 chars per token)
            (self.streamed_chars / 4).max(1)
        };
        let tokens_per_second = tokens_count as f64 / secs;

        let total_prompt = self.input_tokens + self.cache_read_tokens;
        let cache_hit_rate = if total_prompt > 0 {
            Some((self.cache_read_tokens as f64 / total_prompt as f64) * 100.0)
        } else {
            None
        };

        TokenMeterSnapshot {
            input_tokens: self.input_tokens,
            output_tokens: self.output_tokens,
            cache_read_tokens: self.cache_read_tokens,
            reasoning_tokens: self.reasoning_tokens,
            ttft,
            total_duration,
            tokens_per_second,
            cache_hit_rate,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_meter_activity_and_usage() {
        let mut meter = TokenMeter::new();
        meter.record_token_activity("Thinking");
        meter.record_token_activity(" and responding");

        let usage = TokenUsage {
            input_tokens: 100,
            output_tokens: 40,
            cache_read_tokens: Some(50),
            reasoning_tokens: Some(20),
        };
        meter.update_usage(&usage);

        let snap = meter.snapshot();
        assert_eq!(snap.input_tokens, 100);
        assert_eq!(snap.output_tokens, 40);
        assert_eq!(snap.cache_read_tokens, 50);
        assert_eq!(snap.reasoning_tokens, 20);
        assert!(snap.ttft.is_some());
        assert_eq!(snap.cache_hit_rate, Some((50.0 / 150.0) * 100.0));
    }
}
