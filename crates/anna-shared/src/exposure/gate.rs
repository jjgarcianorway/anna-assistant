//! ExposureGate - Central filtering for all dialogue emission.
//!
//! CRITICAL: This is the ONLY point where dialogue visibility is decided.
//! No specialist, no Ralph loop, no streaming path may bypass this gate.
//!
//! All dialogue emission must go through ExposureGate::filter() before rendering.

use super::levels::{DialogueClassification, ExposureLevel};
use super::sanitize::sanitize_dialogue;

/// Result of filtering a dialogue line through ExposureGate.
#[derive(Debug, Clone)]
pub struct GateResult {
    /// Whether to emit this dialogue line.
    pub emit: bool,
    /// The (possibly sanitized) content to emit.
    pub content: String,
    /// Reason for blocking (if emit is false).
    pub block_reason: Option<BlockReason>,
}

/// Reason why dialogue was blocked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockReason {
    /// Exposure level too low for this classification.
    ExposureLevelTooLow {
        current: ExposureLevel,
        required: ExposureLevel,
    },
    /// Content contains forbidden patterns.
    ForbiddenPatterns { violations: Vec<String> },
    /// Content is empty after sanitization.
    EmptyContent,
}

/// Central gate for all dialogue emission.
///
/// Invariants enforced by this gate:
/// 1. No dialogue at Silent level
/// 2. Only informational at Summary level
/// 3. Procedural requires Dialogue level
/// 4. Diagnostic requires Debug level
/// 5. All content is sanitized before emission
/// 6. Forbidden patterns block emission entirely
pub struct ExposureGate {
    level: ExposureLevel,
}

impl ExposureGate {
    /// Create a new gate for the given exposure level.
    pub fn new(level: ExposureLevel) -> Self {
        Self { level }
    }

    /// Create a gate from current config.
    pub fn from_config() -> Self {
        let level = crate::config::AnnaConfig::load()
            .map(|c| c.effective_exposure_level())
            .unwrap_or(ExposureLevel::Silent);
        Self { level }
    }

    /// Get the current exposure level.
    pub fn level(&self) -> ExposureLevel {
        self.level
    }

    /// Filter a dialogue line through the gate.
    ///
    /// Returns GateResult with emit=true if the dialogue should be shown.
    /// The content field contains the sanitized version.
    pub fn filter(&self, content: &str, classification: DialogueClassification) -> GateResult {
        // Step 1: Check exposure level allows this classification
        if !classification.visible_at(self.level) {
            return GateResult {
                emit: false,
                content: String::new(),
                block_reason: Some(BlockReason::ExposureLevelTooLow {
                    current: self.level,
                    required: classification.required_level(),
                }),
            };
        }

        // Step 2: Sanitize content
        let sanitized = sanitize_dialogue(content);

        // Step 3: Check for forbidden patterns
        if !sanitized.is_clean {
            return GateResult {
                emit: false,
                content: String::new(),
                block_reason: Some(BlockReason::ForbiddenPatterns {
                    violations: sanitized.violations.iter().map(|v| format!("{:?}: '{}'", v.pattern, v.matched)).collect(),
                }),
            };
        }

        // Step 4: Check for empty content
        if content.trim().is_empty() {
            return GateResult {
                emit: false,
                content: String::new(),
                block_reason: Some(BlockReason::EmptyContent),
            };
        }

        // All checks passed - content is clean (no sanitization needed)
        GateResult {
            emit: true,
            content: content.to_string(),
            block_reason: None,
        }
    }

    /// Check if any dialogue would be visible at current level.
    pub fn dialogue_enabled(&self) -> bool {
        self.level >= ExposureLevel::Summary
    }

    /// Check if detailed procedural dialogue is visible.
    pub fn procedural_visible(&self) -> bool {
        self.level >= ExposureLevel::Dialogue
    }

    /// Check if diagnostic output is visible.
    pub fn diagnostic_visible(&self) -> bool {
        self.level >= ExposureLevel::Debug
    }
}

/// Fallback message when FinalAnswer is blocked due to policy violations.
/// Phase 15: Anna executes actions, not the user.
const FALLBACK_ANSWER: &str = "I can help with this, but I need to handle it myself rather than providing manual instructions. Would you like me to proceed with the necessary changes? I'll explain what will be done and ask for confirmation before making any modifications.";

/// Convenience function to filter through a gate from config.
pub fn filter_dialogue(content: &str, classification: DialogueClassification) -> GateResult {
    ExposureGate::from_config().filter(content, classification)
}

/// Filter a FinalAnswer with fallback on policy violation.
/// Phase 15: FinalAnswer is NOT privileged - it must be sanitized.
/// If the answer contains forbidden patterns (manual commands), return fallback.
///
/// Note: FinalAnswer uses Summary level (not Silent) because answers should
/// always be visible. Only forbidden pattern violations cause fallback.
pub fn filter_final_answer(content: &str) -> GateResult {
    // Use Summary level to ensure answers are always visible
    // The only thing that blocks FinalAnswer is forbidden patterns
    let gate = ExposureGate::new(ExposureLevel::Summary);
    let result = gate.filter(content, DialogueClassification::Informational);

    if result.emit {
        result
    } else {
        // Answer was blocked due to forbidden patterns - return fallback
        GateResult {
            emit: true,
            content: FALLBACK_ANSWER.to_string(),
            block_reason: result.block_reason,
        }
    }
}

/// Convenience function to check if dialogue is enabled.
pub fn is_dialogue_enabled() -> bool {
    ExposureGate::from_config().dialogue_enabled()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate_silent_blocks_all() {
        let gate = ExposureGate::new(ExposureLevel::Silent);

        // All classifications blocked at Silent
        for class in [
            DialogueClassification::Informational,
            DialogueClassification::Procedural,
            DialogueClassification::Diagnostic,
        ] {
            let result = gate.filter("test message", class);
            assert!(!result.emit, "Silent should block {:?}", class);
            assert!(matches!(
                result.block_reason,
                Some(BlockReason::ExposureLevelTooLow { .. })
            ));
        }
    }

    #[test]
    fn test_gate_summary_allows_informational() {
        let gate = ExposureGate::new(ExposureLevel::Summary);

        // Informational passes
        let result = gate.filter("Request received", DialogueClassification::Informational);
        assert!(result.emit);
        assert!(result.block_reason.is_none());

        // Procedural blocked
        let result = gate.filter("Running probe", DialogueClassification::Procedural);
        assert!(!result.emit);

        // Diagnostic blocked
        let result = gate.filter("Debug output", DialogueClassification::Diagnostic);
        assert!(!result.emit);
    }

    #[test]
    fn test_gate_dialogue_allows_procedural() {
        let gate = ExposureGate::new(ExposureLevel::Dialogue);

        // Informational passes
        let result = gate.filter("Request received", DialogueClassification::Informational);
        assert!(result.emit);

        // Procedural passes
        let result = gate.filter("Running probe", DialogueClassification::Procedural);
        assert!(result.emit);

        // Diagnostic blocked
        let result = gate.filter("Debug output", DialogueClassification::Diagnostic);
        assert!(!result.emit);
    }

    #[test]
    fn test_gate_debug_allows_all() {
        let gate = ExposureGate::new(ExposureLevel::Debug);

        for class in [
            DialogueClassification::Informational,
            DialogueClassification::Procedural,
            DialogueClassification::Diagnostic,
        ] {
            let result = gate.filter("test message", class);
            assert!(result.emit, "Debug should allow {:?}", class);
        }
    }

    #[test]
    fn test_gate_blocks_forbidden_patterns() {
        let gate = ExposureGate::new(ExposureLevel::Debug);

        // Contains forbidden pattern "I think"
        let result = gate.filter("I think this is wrong", DialogueClassification::Informational);
        assert!(!result.emit);
        assert!(matches!(
            result.block_reason,
            Some(BlockReason::ForbiddenPatterns { .. })
        ));
    }

    #[test]
    fn test_gate_blocks_urgency_language() {
        let gate = ExposureGate::new(ExposureLevel::Debug);

        // Contains forbidden pattern "critical"
        let result = gate.filter("Critical error detected", DialogueClassification::Informational);
        assert!(!result.emit);
        assert!(matches!(
            result.block_reason,
            Some(BlockReason::ForbiddenPatterns { .. })
        ));
    }

    #[test]
    fn test_gate_blocks_empty_content() {
        let gate = ExposureGate::new(ExposureLevel::Debug);

        let result = gate.filter("   ", DialogueClassification::Informational);
        assert!(!result.emit);
        assert!(matches!(result.block_reason, Some(BlockReason::EmptyContent)));
    }

    #[test]
    fn test_gate_returns_sanitized_content() {
        let gate = ExposureGate::new(ExposureLevel::Debug);

        let result = gate.filter("Request processed", DialogueClassification::Informational);
        assert!(result.emit);
        assert_eq!(result.content, "Request processed");
    }

    #[test]
    fn test_dialogue_enabled_check() {
        assert!(!ExposureGate::new(ExposureLevel::Silent).dialogue_enabled());
        assert!(ExposureGate::new(ExposureLevel::Summary).dialogue_enabled());
        assert!(ExposureGate::new(ExposureLevel::Dialogue).dialogue_enabled());
        assert!(ExposureGate::new(ExposureLevel::Debug).dialogue_enabled());
    }

    #[test]
    fn test_procedural_visible_check() {
        assert!(!ExposureGate::new(ExposureLevel::Silent).procedural_visible());
        assert!(!ExposureGate::new(ExposureLevel::Summary).procedural_visible());
        assert!(ExposureGate::new(ExposureLevel::Dialogue).procedural_visible());
        assert!(ExposureGate::new(ExposureLevel::Debug).procedural_visible());
    }

    #[test]
    fn test_diagnostic_visible_check() {
        assert!(!ExposureGate::new(ExposureLevel::Silent).diagnostic_visible());
        assert!(!ExposureGate::new(ExposureLevel::Summary).diagnostic_visible());
        assert!(!ExposureGate::new(ExposureLevel::Dialogue).diagnostic_visible());
        assert!(ExposureGate::new(ExposureLevel::Debug).diagnostic_visible());
    }
}
