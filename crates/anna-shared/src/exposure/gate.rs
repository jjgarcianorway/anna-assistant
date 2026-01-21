//! ExposureGate - Central filtering for all dialogue emission.
//!
//! CRITICAL: This is the ONLY point where dialogue visibility is decided.
//! No specialist, no Ralph loop, no streaming path may bypass this gate.
//!
//! All dialogue emission must go through ExposureGate::filter() before rendering.
//!
//! Phase 24: Confidence phrasing modulation based on track record.

use super::levels::{DialogueClassification, ExposureLevel};
use super::sanitize::sanitize_dialogue;
use crate::intent_class::IntentClass;
use crate::policy::{get_policy, ConfidenceLevel};

/// Result of filtering a dialogue line through ExposureGate.
#[derive(Debug, Clone)]
pub struct GateResult {
    /// Whether to emit this dialogue line.
    pub emit: bool,
    /// The (possibly sanitized) content to emit.
    pub content: String,
    /// Reason for blocking (if emit is false).
    pub block_reason: Option<BlockReason>,
    /// Additive warnings attached as metadata (never block content).
    /// Phase 29: Warnings are additive, not terminal.
    pub warnings: Vec<String>,
}

impl GateResult {
    /// Create a result with no warnings.
    pub fn new(emit: bool, content: String, block_reason: Option<BlockReason>) -> Self {
        Self {
            emit,
            content,
            block_reason,
            warnings: Vec::new(),
        }
    }

    /// Attach a warning as metadata (does not block content).
    pub fn with_warning(mut self, warning: String) -> Self {
        self.warnings.push(warning);
        self
    }

    /// Attach multiple warnings as metadata.
    pub fn with_warnings(mut self, warnings: Vec<String>) -> Self {
        self.warnings.extend(warnings);
        self
    }

    /// Check if any warnings are attached.
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
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
    /// Phase 29: Warnings are attached as metadata, never block content.
    pub fn filter(&self, content: &str, classification: DialogueClassification) -> GateResult {
        // Step 1: Check exposure level allows this classification
        if !classification.visible_at(self.level) {
            return GateResult::new(
                false,
                String::new(),
                Some(BlockReason::ExposureLevelTooLow {
                    current: self.level,
                    required: classification.required_level(),
                }),
            );
        }

        // Step 2: Sanitize content
        let sanitized = sanitize_dialogue(content);

        // Step 3: Check for forbidden patterns
        if !sanitized.is_clean {
            return GateResult::new(
                false,
                String::new(),
                Some(BlockReason::ForbiddenPatterns {
                    violations: sanitized.violations.iter().map(|v| format!("{:?}: '{}'", v.pattern, v.matched)).collect(),
                }),
            );
        }

        // Step 4: Check for empty content
        if content.trim().is_empty() {
            return GateResult::new(false, String::new(), Some(BlockReason::EmptyContent));
        }

        // All checks passed - content is clean (no sanitization needed)
        GateResult::new(true, content.to_string(), None)
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

/// Fallback for MUTATING intents - ActionPlan flow with confirmation.
/// Phase 15/22: MUTATING operations require confirmation before execution.
const FALLBACK_MUTATING: &str = "This operation requires changes. An action plan will be prepared for your approval before any modifications are made.";

/// Convenience function to filter through a gate from config.
pub fn filter_dialogue(content: &str, classification: DialogueClassification) -> GateResult {
    ExposureGate::from_config().filter(content, classification)
}

/// Filter a FinalAnswer with intent-aware fallback on policy violation.
/// Phase 15/22: FinalAnswer is NOT privileged - it must be sanitized.
/// Phase 24: Applies confidence phrasing based on track record.
///
/// If the answer contains forbidden patterns (manual commands), return capability-aware fallback.
///
/// READ_ONLY: Gets capability-routed response (Abstained with hints, not generic fallback)
/// MUTATING: Gets ActionPlan confirmation fallback
///
/// Note: FinalAnswer uses Summary level (not Silent) because answers should
/// always be visible. Only forbidden pattern violations cause fallback.
pub fn filter_final_answer(content: &str, intent: IntentClass) -> GateResult {
    filter_final_answer_with_request(content, intent, None)
}

/// Filter a FinalAnswer with original request for capability routing.
/// When the original request is provided, blocked output produces a capability-aware
/// response instead of a generic fallback.
/// Phase 29: Warnings are preserved as metadata, not lost in fallback.
pub fn filter_final_answer_with_request(
    content: &str,
    intent: IntentClass,
    original_request: Option<&str>,
) -> GateResult {
    use crate::capability::{build_policy_violation_response, format_outcome_to_string};

    // Use Summary level to ensure answers are always visible
    // The only thing that blocks FinalAnswer is forbidden patterns
    let gate = ExposureGate::new(ExposureLevel::Summary);
    let result = gate.filter(content, DialogueClassification::Informational);

    if result.emit {
        // Phase 24: Apply confidence phrasing modulation
        let policy = get_policy();
        let content_with_phrasing = apply_confidence_phrasing(&result.content, policy.confidence_level);

        // Phase 29: Preserve any warnings from the original result
        GateResult::new(true, content_with_phrasing, None)
            .with_warnings(result.warnings)
    } else {
        // Answer was blocked - use intent-appropriate fallback
        let fallback = match intent {
            IntentClass::ReadOnly => {
                // Use capability routing to provide structured fallback
                let request = original_request.unwrap_or("");
                let response = build_policy_violation_response(request);
                format_outcome_to_string(&response)
            }
            IntentClass::Mutating => FALLBACK_MUTATING.to_string(),
        };
        // Phase 29: Even on fallback, preserve warnings as metadata
        GateResult::new(true, fallback, result.block_reason)
            .with_warnings(result.warnings)
    }
}

/// Filter a FinalAnswer and attach pending system warnings as metadata.
/// Phase 29: Warnings are additive - they accompany the primary response,
/// never replace it.
pub fn filter_final_answer_with_warnings(
    content: &str,
    intent: IntentClass,
    original_request: Option<&str>,
    pending_warnings: Vec<String>,
) -> GateResult {
    filter_final_answer_with_request(content, intent, original_request)
        .with_warnings(pending_warnings)
}

/// Phase 24: Apply confidence phrasing to answer based on track record.
/// High confidence: no change (confident language allowed)
/// Medium confidence: neutral phrasing (unchanged)
/// Low/Unknown confidence: prepend hedge phrase
fn apply_confidence_phrasing(content: &str, confidence: ConfidenceLevel) -> String {
    // Don't modify very short content or empty content
    if content.trim().len() < 20 {
        return content.to_string();
    }

    // Check if content already starts with a hedge phrase
    let lower = content.to_lowercase();
    let already_hedged = lower.starts_with("based on")
        || lower.starts_with("without")
        || lower.starts_with("from the")
        || lower.starts_with("according to");

    if already_hedged {
        return content.to_string();
    }

    match confidence {
        ConfidenceLevel::High | ConfidenceLevel::Medium => {
            // No modification needed
            content.to_string()
        }
        ConfidenceLevel::Low => {
            // Low confidence: subtle hedge
            format!("Based on available information, {}", lowercase_first(content))
        }
        ConfidenceLevel::Unknown => {
            // Unknown: explicit caveat (cold start)
            content.to_string() // Don't add noise at cold start, just be neutral
        }
    }
}

/// Lowercase the first character of a string (for natural phrasing).
fn lowercase_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_lowercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// Filter a FinalAnswer without intent context (defaults to ReadOnly).
/// Backwards-compatible wrapper for existing call sites.
pub fn filter_final_answer_default(content: &str) -> GateResult {
    filter_final_answer(content, IntentClass::ReadOnly)
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

    // Phase 22: Intent-aware fallback tests

    #[test]
    fn test_filter_final_answer_readonly_fallback_no_generic() {
        // Content with forbidden patterns should get capability-aware fallback
        let result = filter_final_answer("sudo pacman -Syu", IntentClass::ReadOnly);
        assert!(result.emit);
        // CRITICAL: Must NOT contain the generic fallback message
        assert!(
            !result.content.contains("could not format a valid response"),
            "Must not use generic fallback. Got: {}",
            result.content
        );
        // Should have capability-aware response with hints
        assert!(
            result.content.contains("does not match any known capability")
                || result.content.contains("Things I can help with"),
            "Should have structured capability-aware response. Got: {}",
            result.content
        );
        assert!(!result.content.contains("would you like"));
        assert!(!result.content.contains("Would you like"));
    }

    #[test]
    fn test_filter_final_answer_mutating_fallback() {
        // Content with forbidden patterns should get MUTATING fallback
        let result = filter_final_answer("sudo pacman -Syu", IntentClass::Mutating);
        assert!(result.emit);
        assert!(result.content.contains("requires changes"));
        assert!(result.content.contains("approval"));
    }

    #[test]
    fn test_filter_final_answer_clean_content_passes() {
        // Clean content passes through regardless of intent
        let clean = "The disk usage is at 45%.";
        let result_ro = filter_final_answer(clean, IntentClass::ReadOnly);
        let result_mut = filter_final_answer(clean, IntentClass::Mutating);

        assert!(result_ro.emit);
        assert_eq!(result_ro.content, clean);
        assert!(result_mut.emit);
        assert_eq!(result_mut.content, clean);
    }

    #[test]
    fn test_filter_final_answer_default_no_generic_fallback() {
        // Default function should use capability-aware fallback
        let result = filter_final_answer_default("sudo pacman -Syu");
        assert!(result.emit);
        // CRITICAL: Must NOT contain the generic fallback message
        assert!(
            !result.content.contains("could not format a valid response"),
            "Must not use generic fallback. Got: {}",
            result.content
        );
    }

    #[test]
    fn test_filter_final_answer_with_request_routing() {
        // When request is provided, should route to matching capability
        let result = filter_final_answer_with_request(
            "sudo pacman -Syu",
            IntentClass::ReadOnly,
            Some("scale my gdm please"),
        );
        assert!(result.emit);
        // Should NOT contain generic fallback
        assert!(
            !result.content.contains("could not format a valid response"),
            "Must not use generic fallback. Got: {}",
            result.content
        );
        // Should have capability-aware output (matched but blocked by policy)
        assert!(
            result.content.contains("matching capability")
                || result.content.contains("cannot be displayed"),
            "Should route to display.scale.gdm capability. Got: {}",
            result.content
        );
    }

    #[test]
    fn test_filter_final_answer_includes_hints() {
        // Unknown request should include capability hints
        let result = filter_final_answer_with_request(
            "sudo do something",
            IntentClass::ReadOnly,
            Some("tell me a joke"),
        );
        assert!(result.emit);
        // Should have hints for available capabilities (human-readable format)
        assert!(
            result.content.contains("Things I can help with")
                || result.content.contains("status")
                || result.content.contains("disk"),
            "Should include capability hints. Got: {}",
            result.content
        );
    }

    // Phase 29: Additive warning tests

    #[test]
    fn test_gate_result_warnings_default_empty() {
        let result = GateResult::new(true, "content".to_string(), None);
        assert!(!result.has_warnings());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_gate_result_with_warning() {
        let result = GateResult::new(true, "content".to_string(), None)
            .with_warning("Config changed: group".to_string());

        assert!(result.has_warnings());
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.warnings[0], "Config changed: group");
    }

    #[test]
    fn test_gate_result_with_multiple_warnings() {
        let warnings = vec![
            "Config changed: group".to_string(),
            "Config changed: passwd".to_string(),
        ];
        let result = GateResult::new(true, "content".to_string(), None)
            .with_warnings(warnings);

        assert!(result.has_warnings());
        assert_eq!(result.warnings.len(), 2);
    }

    #[test]
    fn test_warnings_preserved_on_clean_content() {
        // Clean content should preserve warnings as metadata
        let result = filter_final_answer_with_warnings(
            "The disk usage is at 45%.",
            IntentClass::ReadOnly,
            None,
            vec!["Config changed: group".to_string()],
        );

        assert!(result.emit);
        assert_eq!(result.content, "The disk usage is at 45%.");
        assert!(result.has_warnings());
        assert_eq!(result.warnings[0], "Config changed: group");
    }

    #[test]
    fn test_warnings_preserved_on_blocked_content() {
        // Even when content is blocked, warnings should be preserved as metadata
        let result = filter_final_answer_with_warnings(
            "sudo pacman -Syu",
            IntentClass::ReadOnly,
            Some("scale my gdm please"),
            vec!["Config changed: group".to_string()],
        );

        assert!(result.emit);
        // Content is replaced with capability-aware fallback (matched but blocked by policy)
        assert!(
            result.content.contains("matching capability")
                || result.content.contains("cannot be displayed"),
            "Should have capability-aware fallback. Got: {}",
            result.content
        );
        // But warnings are STILL preserved
        assert!(
            result.has_warnings(),
            "Warnings should be preserved even on blocked content"
        );
        assert_eq!(result.warnings[0], "Config changed: group");
    }

    #[test]
    fn test_warnings_additive_not_terminal() {
        // Key invariant: warnings never block primary content
        let clean_content = "GDM scaling has been configured to 2x.";
        let warnings = vec!["Config changed: group".to_string()];

        let result = filter_final_answer_with_warnings(
            clean_content,
            IntentClass::ReadOnly,
            Some("scale my gdm please"),
            warnings,
        );

        // Primary content passes through
        assert!(result.emit);
        assert_eq!(result.content, clean_content);
        // Warnings are metadata, not blockers
        assert!(result.has_warnings());
        // Block reason should be None (warnings don't block)
        assert!(result.block_reason.is_none());
    }
}
