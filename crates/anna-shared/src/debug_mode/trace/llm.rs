//! LLM call trace information.
//!
//! Captures details about LLM calls, prompts, responses, and parsing.

use crate::debug_mode::redact::Redactor;
use serde::{Deserialize, Serialize};

/// LLM call trace info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmTrace {
    /// Role (translator/specialist/verifier)
    pub role: String,
    /// Model name
    pub model: String,
    /// Duration in ms
    pub duration_ms: u64,
    /// Input token estimate
    pub input_tokens_est: u32,
    /// Output token estimate
    pub output_tokens_est: u32,
    /// Temperature used
    pub temperature: f32,
    /// Max tokens setting
    pub max_tokens: u32,
    /// Parse success
    pub parse_success: bool,
    /// Parse error details (if any)
    pub parse_error: Option<ParseErrorInfo>,
    /// Prompt digest (level 2)
    pub prompt_digest: Option<PromptDigest>,
    /// Full prompt (level 3, redacted)
    pub full_prompt: Option<String>,
    /// Full response (level 3, redacted, even if invalid JSON)
    pub full_response: Option<String>,
}

impl LlmTrace {
    pub fn new(role: &str, model: &str) -> Self {
        Self {
            role: role.to_string(),
            model: model.to_string(),
            duration_ms: 0,
            input_tokens_est: 0,
            output_tokens_est: 0,
            temperature: 0.0,
            max_tokens: 0,
            parse_success: false,
            parse_error: None,
            prompt_digest: None,
            full_prompt: None,
            full_response: None,
        }
    }

    /// Set timing.
    pub fn with_timing(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }

    /// Set token estimates.
    pub fn with_tokens(mut self, input: u32, output: u32) -> Self {
        self.input_tokens_est = input;
        self.output_tokens_est = output;
        self
    }

    /// Set model params.
    pub fn with_params(mut self, temperature: f32, max_tokens: u32) -> Self {
        self.temperature = temperature;
        self.max_tokens = max_tokens;
        self
    }

    /// Set parse result.
    pub fn with_parse(mut self, success: bool, error: Option<ParseErrorInfo>) -> Self {
        self.parse_success = success;
        self.parse_error = error;
        self
    }

    /// Set prompt digest (level 2).
    pub fn with_digest(mut self, system: &str, user: &str) -> Self {
        self.prompt_digest = Some(PromptDigest::new(system, user));
        self
    }

    /// Set full prompt/response (level 3, redacted).
    pub fn with_full(mut self, prompt: &str, response: &str, redactor: &Redactor) -> Self {
        self.full_prompt = Some(redactor.redact(prompt));
        self.full_response = Some(redactor.redact(response));
        self
    }
}

/// Parse error details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseErrorInfo {
    /// Error message
    pub message: String,
    /// Byte offset where error occurred
    pub byte_offset: Option<usize>,
    /// Field name that failed
    pub field_name: Option<String>,
    /// Raw output snippet around error
    pub context: Option<String>,
}

impl ParseErrorInfo {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
            byte_offset: None,
            field_name: None,
            context: None,
        }
    }

    pub fn with_location(mut self, offset: usize, field: &str) -> Self {
        self.byte_offset = Some(offset);
        self.field_name = Some(field.to_string());
        self
    }

    pub fn with_context(mut self, raw: &str, offset: usize) -> Self {
        // Extract ~100 chars around the error
        let start = offset.saturating_sub(50);
        let end = (offset + 50).min(raw.len());
        if let Some(slice) = raw.get(start..end) {
            self.context = Some(slice.to_string());
        }
        self
    }
}

/// Compact prompt digest (level 2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptDigest {
    /// Hash of system prompt
    pub system_hash: String,
    /// Hash of user prompt
    pub user_hash: String,
    /// First 200 chars of system prompt
    pub system_preview: String,
    /// First 200 chars of user prompt
    pub user_preview: String,
    /// Total prompt length
    pub total_chars: usize,
}

impl PromptDigest {
    pub fn new(system: &str, user: &str) -> Self {
        Self {
            system_hash: simple_hash(system),
            user_hash: simple_hash(user),
            system_preview: system.chars().take(200).collect(),
            user_preview: user.chars().take(200).collect(),
            total_chars: system.len() + user.len(),
        }
    }
}

/// Simple hash for prompt digests.
fn simple_hash(s: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    format!("{:016x}", h.finish())
}
