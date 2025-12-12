//! No Streaming for Parsed Calls (Part C) - v0.0.438.
//!
//! Disable streaming on any model call expected to be parsed.
//! Streaming is allowed ONLY for the Translator-to-User natural language layer.
//!
//! Rules:
//! - translator_intent: no streaming (parsed)
//! - specialist: no streaming (parsed)
//! - renderer: streaming allowed (not parsed, user-facing)

use serde::{Deserialize, Serialize};

/// Policy for streaming behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CallPolicy {
    /// No streaming - wait for complete response.
    NoStream,
    /// Streaming allowed - tokens arrive incrementally.
    StreamAllowed,
}

impl CallPolicy {
    /// Check if streaming is disabled.
    pub fn is_no_stream(&self) -> bool {
        matches!(self, Self::NoStream)
    }

    /// Check if streaming is allowed.
    pub fn is_stream_allowed(&self) -> bool {
        matches!(self, Self::StreamAllowed)
    }
}

/// Call type for determining streaming policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CallType {
    /// Translator intent extraction (parsed).
    TranslatorIntent,
    /// Junior specialist analysis (parsed).
    JuniorSpecialist,
    /// Senior specialist analysis (parsed).
    SeniorSpecialist,
    /// Renderer output (user-facing, not parsed).
    Renderer,
    /// Probe execution (not LLM, but included for completeness).
    Probe,
}

impl CallType {
    /// Get streaming policy for this call type.
    pub fn policy(&self) -> CallPolicy {
        match self {
            // Parsed calls - no streaming
            Self::TranslatorIntent => CallPolicy::NoStream,
            Self::JuniorSpecialist => CallPolicy::NoStream,
            Self::SeniorSpecialist => CallPolicy::NoStream,
            // User-facing - streaming allowed
            Self::Renderer => CallPolicy::StreamAllowed,
            // Probes are not LLM calls
            Self::Probe => CallPolicy::NoStream,
        }
    }

    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::TranslatorIntent => "translator_intent",
            Self::JuniorSpecialist => "junior_specialist",
            Self::SeniorSpecialist => "senior_specialist",
            Self::Renderer => "renderer",
            Self::Probe => "probe",
        }
    }

    /// Whether this call type expects JSON output.
    pub fn expects_json(&self) -> bool {
        match self {
            Self::TranslatorIntent => true,
            Self::JuniorSpecialist => true,
            Self::SeniorSpecialist => true,
            Self::Renderer => false,
            Self::Probe => false,
        }
    }
}

/// Configuration for a model call.
#[derive(Debug, Clone)]
pub struct ModelCallConfig {
    /// Call type.
    pub call_type: CallType,
    /// Streaming policy (derived from call type).
    pub policy: CallPolicy,
    /// Timeout in milliseconds.
    pub timeout_ms: u64,
    /// Max tokens for response.
    pub max_tokens: usize,
    /// Whether response is parsed.
    pub is_parsed: bool,
}

impl ModelCallConfig {
    /// Create config for translator intent.
    pub fn translator_intent(timeout_ms: u64) -> Self {
        Self {
            call_type: CallType::TranslatorIntent,
            policy: CallPolicy::NoStream,
            timeout_ms,
            max_tokens: 150,
            is_parsed: true,
        }
    }

    /// Create config for junior specialist.
    pub fn junior_specialist(timeout_ms: u64) -> Self {
        Self {
            call_type: CallType::JuniorSpecialist,
            policy: CallPolicy::NoStream,
            timeout_ms,
            max_tokens: 220,
            is_parsed: true,
        }
    }

    /// Create config for senior specialist.
    pub fn senior_specialist(timeout_ms: u64) -> Self {
        Self {
            call_type: CallType::SeniorSpecialist,
            policy: CallPolicy::NoStream,
            timeout_ms,
            max_tokens: 220,
            is_parsed: true,
        }
    }

    /// Create config for renderer.
    pub fn renderer(timeout_ms: u64) -> Self {
        Self {
            call_type: CallType::Renderer,
            policy: CallPolicy::StreamAllowed,
            timeout_ms,
            max_tokens: 500,
            is_parsed: false,
        }
    }

    /// Check if streaming should be disabled.
    pub fn should_disable_stream(&self) -> bool {
        self.policy.is_no_stream()
    }
}

/// Enforce no-streaming policy on a call.
///
/// Returns `true` if streaming should be disabled for this call type.
pub fn enforce_no_stream(call_type: CallType) -> bool {
    call_type.policy().is_no_stream()
}

/// Reason why streaming is disabled.
#[derive(Debug, Clone)]
pub struct NoStreamReason {
    /// Call type.
    pub call_type: CallType,
    /// Human-readable reason.
    pub reason: &'static str,
}

impl NoStreamReason {
    /// Get reason for a call type.
    pub fn for_call(call_type: CallType) -> Option<Self> {
        if call_type.policy().is_no_stream() {
            let reason = match call_type {
                CallType::TranslatorIntent => "Intent extraction requires complete JSON parsing",
                CallType::JuniorSpecialist => "Specialist output is parsed JSON contract",
                CallType::SeniorSpecialist => "Specialist output is parsed JSON contract",
                CallType::Probe => "Probes are not LLM calls",
                CallType::Renderer => unreachable!(),
            };
            Some(Self { call_type, reason })
        } else {
            None
        }
    }
}

/// Stream configuration builder.
#[derive(Debug, Clone, Default)]
pub struct StreamConfigBuilder {
    /// Disable streaming globally.
    pub force_no_stream: bool,
    /// Allow streaming for renderer even if force_no_stream.
    pub allow_renderer_stream: bool,
}

impl StreamConfigBuilder {
    /// Create new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Force no streaming for all calls.
    pub fn force_no_stream(mut self) -> Self {
        self.force_no_stream = true;
        self
    }

    /// Allow renderer to stream even when forced.
    pub fn allow_renderer(mut self) -> Self {
        self.allow_renderer_stream = true;
        self
    }

    /// Build final policy for a call type.
    pub fn policy_for(&self, call_type: CallType) -> CallPolicy {
        if self.force_no_stream {
            if self.allow_renderer_stream && call_type == CallType::Renderer {
                CallPolicy::StreamAllowed
            } else {
                CallPolicy::NoStream
            }
        } else {
            call_type.policy()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_policy() {
        assert!(CallPolicy::NoStream.is_no_stream());
        assert!(CallPolicy::StreamAllowed.is_stream_allowed());
    }

    #[test]
    fn test_call_type_policies() {
        assert_eq!(CallType::TranslatorIntent.policy(), CallPolicy::NoStream);
        assert_eq!(CallType::JuniorSpecialist.policy(), CallPolicy::NoStream);
        assert_eq!(CallType::SeniorSpecialist.policy(), CallPolicy::NoStream);
        assert_eq!(CallType::Renderer.policy(), CallPolicy::StreamAllowed);
    }

    #[test]
    fn test_enforce_no_stream() {
        assert!(enforce_no_stream(CallType::TranslatorIntent));
        assert!(enforce_no_stream(CallType::JuniorSpecialist));
        assert!(!enforce_no_stream(CallType::Renderer));
    }

    #[test]
    fn test_model_call_config() {
        let config = ModelCallConfig::junior_specialist(1500);
        assert!(config.should_disable_stream());
        assert!(config.is_parsed);

        let renderer = ModelCallConfig::renderer(200);
        assert!(!renderer.should_disable_stream());
        assert!(!renderer.is_parsed);
    }

    #[test]
    fn test_stream_config_builder() {
        let builder = StreamConfigBuilder::new()
            .force_no_stream()
            .allow_renderer();

        assert_eq!(
            builder.policy_for(CallType::JuniorSpecialist),
            CallPolicy::NoStream
        );
        assert_eq!(
            builder.policy_for(CallType::Renderer),
            CallPolicy::StreamAllowed
        );
    }

    #[test]
    fn test_expects_json() {
        assert!(CallType::TranslatorIntent.expects_json());
        assert!(CallType::JuniorSpecialist.expects_json());
        assert!(!CallType::Renderer.expects_json());
    }
}
