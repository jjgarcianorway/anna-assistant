//! Streaming validator for real-time answer validation.

use anna_shared::rpc::ValidationWarning;

use super::checks::{check_hallucination, check_too_generic, check_uncertainty, extract_grounding_values};
use super::contradiction::check_contradiction;

/// Confidence penalties for each issue type
const PENALTY_HALLUCINATION: f32 = 0.25;
const PENALTY_CONTRADICTION: f32 = 0.30;
const PENALTY_UNCERTAINTY: f32 = 0.10;
const PENALTY_TOO_GENERIC: f32 = 0.15;
/// Threshold below which self-correction is recommended
const CORRECTION_THRESHOLD: f32 = 0.5;

/// Result of validation with confidence info
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub warnings: Vec<ValidationWarning>,
    pub confidence: f32,
    pub needs_correction: bool,
    pub correction_hint: Option<String>,
}

/// Streaming validator that accumulates tokens and checks for issues
pub struct StreamingValidator {
    accumulated: String,
    command_output: String,
    grounding_values: Vec<String>,
    warnings: Vec<ValidationWarning>,
    last_validated_len: usize,
    confidence: f32,
}

impl StreamingValidator {
    /// Create a new validator with command output to check against
    pub fn new(command_output: &str) -> Self {
        let grounding_values = extract_grounding_values(command_output);
        Self {
            accumulated: String::new(),
            command_output: command_output.to_string(),
            grounding_values,
            warnings: Vec::new(),
            last_validated_len: 0,
            confidence: 1.0,
        }
    }

    /// Add a token and check for new validation issues
    pub fn add_token(&mut self, token: &str) -> Vec<ValidationWarning> {
        let result = self.add_token_with_confidence(token);
        result.warnings
    }

    /// Add a token and return full validation result with confidence
    pub fn add_token_with_confidence(&mut self, token: &str) -> ValidationResult {
        self.accumulated.push_str(token);

        if !should_validate(&self.accumulated, self.last_validated_len) {
            return ValidationResult {
                warnings: Vec::new(),
                confidence: self.confidence,
                needs_correction: false,
                correction_hint: None,
            };
        }

        let new_text = &self.accumulated[self.last_validated_len..];
        let mut new_warnings = Vec::new();
        let mut correction_hint = None;

        if let Some(warning) = check_uncertainty(new_text) {
            if !self.has_warning(&warning) {
                self.confidence -= PENALTY_UNCERTAINTY;
                self.warnings.push(warning.clone());
                new_warnings.push(warning);
            }
        }

        if let Some(warning) =
            check_hallucination(new_text, &self.command_output, &self.grounding_values)
        {
            if !self.has_warning(&warning) {
                self.confidence -= PENALTY_HALLUCINATION;
                correction_hint = Some(format!("Verify claim: {}", warning.message));
                self.warnings.push(warning.clone());
                new_warnings.push(warning);
            }
        }

        if let Some(warning) = check_too_generic(new_text) {
            if !self.has_warning(&warning) {
                self.confidence -= PENALTY_TOO_GENERIC;
                self.warnings.push(warning.clone());
                new_warnings.push(warning);
            }
        }

        if let Some(warning) = check_contradiction(new_text, &self.command_output) {
            if !self.has_warning(&warning) {
                self.confidence -= PENALTY_CONTRADICTION;
                correction_hint = Some(format!("Contradiction detected: {}", warning.message));
                self.warnings.push(warning.clone());
                new_warnings.push(warning);
            }
        }

        self.last_validated_len = self.accumulated.len();
        self.confidence = self.confidence.max(0.0);

        ValidationResult {
            warnings: new_warnings,
            confidence: self.confidence,
            needs_correction: self.confidence < CORRECTION_THRESHOLD,
            correction_hint,
        }
    }

    /// Get current confidence score
    pub fn get_confidence(&self) -> f32 {
        self.confidence
    }

    /// Check if self-correction is recommended
    pub fn needs_correction(&self) -> bool {
        self.confidence < CORRECTION_THRESHOLD
    }

    /// Get summary of issues for correction prompt
    pub fn get_correction_summary(&self) -> Option<String> {
        if self.warnings.is_empty() {
            return None;
        }

        let issues: Vec<String> = self
            .warnings
            .iter()
            .map(|w| format!("- {:?}: {}", w.issue_type, w.message))
            .collect();

        Some(format!(
            "Issues detected (confidence: {:.0}%):\n{}",
            self.confidence * 100.0,
            issues.join("\n")
        ))
    }

    /// Get all accumulated warnings
    pub fn get_warnings(&self) -> &[ValidationWarning] {
        &self.warnings
    }

    /// Check if we already have a similar warning
    fn has_warning(&self, warning: &ValidationWarning) -> bool {
        self.warnings.iter().any(|w| {
            std::mem::discriminant(&w.issue_type) == std::mem::discriminant(&warning.issue_type)
                && w.message == warning.message
        })
    }
}

/// Determine if we should validate now (have complete sentence)
fn should_validate(text: &str, last_len: usize) -> bool {
    let new_text = &text[last_len..];

    if new_text.len() < 50 {
        return false;
    }

    new_text.contains(". ")
        || new_text.contains(".\n")
        || new_text.ends_with('.')
        || new_text.ends_with('!')
        || new_text.ends_with('?')
        || new_text.ends_with(':')
}
