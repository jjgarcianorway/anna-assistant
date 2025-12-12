//! Translator Precision Rules (Part D) - v0.0.441.
//!
//! Translator converts reasoning → user-facing answer.
//!
//! PRECISION RULES (non-negotiable):
//! - Numeric question → numeric answer only
//! - Yes/No question → yes/no + 1 sentence max
//! - List question → list only
//! - Entity question → entity name only
//!
//! Violations = BUG.

use super::evidence::{EvidenceBundle, FactValue};
use super::pipeline::AnswerType;
use super::reasoning::{DerivedValues, ReasoningOutput};

/// Maximum answer lengths by type.
pub const MAX_NUMERIC_ANSWER: usize = 30;
pub const MAX_BOOLEAN_ANSWER: usize = 80;
pub const MAX_LIST_ITEMS: usize = 10;
pub const MAX_ENTITY_ANSWER: usize = 100;
pub const MAX_BRIEF_ANSWER: usize = 200;

/// Translated answer.
#[derive(Debug, Clone)]
pub struct TranslatedAnswer {
    /// The answer text.
    pub text: String,
    /// Answer type.
    pub answer_type: AnswerType,
    /// Confidence (inherited from reasoning).
    pub confidence: f64,
    /// Whether answer matches expected type.
    pub type_match: bool,
}

impl TranslatedAnswer {
    /// Create new answer.
    pub fn new(text: &str, answer_type: AnswerType, confidence: f64) -> Self {
        let type_match = validate_answer_type(text, answer_type);
        Self {
            text: text.to_string(),
            answer_type,
            confidence,
            type_match,
        }
    }

    /// Check if answer is valid.
    pub fn is_valid(&self) -> bool {
        self.type_match && !self.text.is_empty()
    }
}

/// Validate answer matches expected type.
fn validate_answer_type(text: &str, expected: AnswerType) -> bool {
    match expected {
        AnswerType::Numeric => {
            // Should be primarily numeric
            let has_number = text.chars().any(|c| c.is_ascii_digit());
            let short_enough = text.len() <= MAX_NUMERIC_ANSWER;
            has_number && short_enough
        }
        AnswerType::Boolean => {
            let lower = text.to_lowercase();
            let starts_with_yesno = lower.starts_with("yes") || lower.starts_with("no");
            let short_enough = text.len() <= MAX_BOOLEAN_ANSWER;
            starts_with_yesno && short_enough
        }
        AnswerType::List => {
            // Lists should have commas, newlines, or bullet points
            text.contains(',') || text.contains('\n') || text.contains('•') || text.contains('-')
        }
        AnswerType::Entity => {
            // Entity should be concise
            text.len() <= MAX_ENTITY_ANSWER && !text.contains('\n')
        }
        AnswerType::Brief => text.len() <= MAX_BRIEF_ANSWER,
    }
}

/// Precision translator.
pub struct PrecisionTranslator {
    /// Whether to enforce strict type matching.
    strict: bool,
}

impl PrecisionTranslator {
    /// Create new translator.
    pub fn new() -> Self {
        Self { strict: true }
    }

    /// Create lenient translator.
    pub fn lenient() -> Self {
        Self { strict: false }
    }

    /// Translate reasoning to user answer.
    pub fn translate(
        &self,
        reasoning: &ReasoningOutput,
        evidence: &EvidenceBundle,
        answer_type: AnswerType,
    ) -> Result<TranslatedAnswer, TranslationError> {
        // Cannot translate if reasoning says can't answer
        if !reasoning.can_answer {
            return Err(TranslationError::CannotAnswer {
                requires: reasoning.requires.clone(),
            });
        }

        // Build answer based on type
        let text = match answer_type {
            AnswerType::Numeric => self.build_numeric_answer(reasoning, evidence)?,
            AnswerType::Boolean => self.build_boolean_answer(reasoning, evidence)?,
            AnswerType::List => self.build_list_answer(reasoning, evidence)?,
            AnswerType::Entity => self.build_entity_answer(reasoning, evidence)?,
            AnswerType::Brief => self.build_brief_answer(reasoning, evidence)?,
        };

        let answer = TranslatedAnswer::new(&text, answer_type, reasoning.confidence);

        // Strict mode enforces type match
        if self.strict && !answer.type_match {
            return Err(TranslationError::TypeMismatch {
                expected: answer_type.label().to_string(),
                got: text,
            });
        }

        Ok(answer)
    }

    /// Build numeric answer (e.g., "17.0 GiB").
    fn build_numeric_answer(
        &self,
        reasoning: &ReasoningOutput,
        evidence: &EvidenceBundle,
    ) -> Result<String, TranslationError> {
        // Try derived metric first
        if let Some(ref metric) = reasoning.derived.metric {
            return Ok(metric.clone());
        }

        // Look for numeric facts in evidence
        for (name, value) in &evidence.facts {
            if let Some(n) = value.as_number() {
                // Format with unit if identifiable
                let unit = infer_unit(name);
                if n.fract() == 0.0 {
                    return Ok(format!("{}{}", n as i64, unit));
                } else {
                    return Ok(format!("{:.1}{}", n, unit));
                }
            }
        }

        Err(TranslationError::NoNumericValue)
    }

    /// Build boolean answer (e.g., "Yes." or "No, trim is not enabled.").
    fn build_boolean_answer(
        &self,
        reasoning: &ReasoningOutput,
        evidence: &EvidenceBundle,
    ) -> Result<String, TranslationError> {
        // Try derived metric (often "yes"/"no")
        if let Some(ref metric) = reasoning.derived.metric {
            let lower = metric.to_lowercase();
            if lower.contains("yes") || lower.contains("true") || lower.contains("enabled") {
                return Ok("Yes.".to_string());
            } else if lower.contains("no") || lower.contains("false") || lower.contains("disabled")
            {
                return Ok("No.".to_string());
            }
        }

        // Look for boolean facts
        for (_, value) in &evidence.facts {
            if let Some(b) = value.as_bool() {
                return Ok(if b {
                    "Yes.".to_string()
                } else {
                    "No.".to_string()
                });
            }
        }

        // Default based on reasoning confidence
        if reasoning.confidence > 0.7 {
            Ok("Yes.".to_string())
        } else {
            Err(TranslationError::NoBooleanValue)
        }
    }

    /// Build list answer (e.g., "service1 (3.2s), service2 (2.1s)").
    fn build_list_answer(
        &self,
        reasoning: &ReasoningOutput,
        evidence: &EvidenceBundle,
    ) -> Result<String, TranslationError> {
        // Try derived metric if it's a list-like string
        if let Some(ref metric) = reasoning.derived.metric {
            if metric.contains(',') || metric.contains('\n') {
                return Ok(metric.clone());
            }
        }

        // Look for list facts
        for (_, value) in &evidence.facts {
            if let Some(list) = value.as_list() {
                let items: Vec<_> = list.iter().take(MAX_LIST_ITEMS).cloned().collect();
                return Ok(items.join(", "));
            }
        }

        // Single item is still a valid list
        if let Some(ref metric) = reasoning.derived.metric {
            return Ok(metric.clone());
        }

        Err(TranslationError::NoListValue)
    }

    /// Build entity answer (e.g., "AMD Ryzen 9 5900X").
    fn build_entity_answer(
        &self,
        reasoning: &ReasoningOutput,
        evidence: &EvidenceBundle,
    ) -> Result<String, TranslationError> {
        // Try derived metric first
        if let Some(ref metric) = reasoning.derived.metric {
            return Ok(metric.clone());
        }

        // Try root cause
        if let Some(ref cause) = reasoning.derived.root_cause {
            return Ok(cause.clone());
        }

        // Look for string facts
        for (_, value) in &evidence.facts {
            if let Some(s) = value.as_string() {
                return Ok(s.to_string());
            }
        }

        Err(TranslationError::NoEntityValue)
    }

    /// Build brief answer (2-3 sentences).
    fn build_brief_answer(
        &self,
        reasoning: &ReasoningOutput,
        evidence: &EvidenceBundle,
    ) -> Result<String, TranslationError> {
        // Use reasoning as base
        let mut answer = reasoning.reasoning.clone();

        // Add metric if available
        if let Some(ref metric) = reasoning.derived.metric {
            if !answer.contains(metric) {
                answer = format!("{} ({})", answer, metric);
            }
        }

        // Truncate if too long
        if answer.len() > MAX_BRIEF_ANSWER {
            answer = format!("{}...", &answer[..MAX_BRIEF_ANSWER - 3]);
        }

        Ok(answer)
    }
}

impl Default for PrecisionTranslator {
    fn default() -> Self {
        Self::new()
    }
}

/// Translation error.
#[derive(Debug, Clone)]
pub enum TranslationError {
    /// Cannot answer - requires more facts.
    CannotAnswer { requires: Vec<String> },
    /// Answer type mismatch.
    TypeMismatch { expected: String, got: String },
    /// No numeric value found.
    NoNumericValue,
    /// No boolean value found.
    NoBooleanValue,
    /// No list value found.
    NoListValue,
    /// No entity value found.
    NoEntityValue,
}

impl TranslationError {
    /// Get error message.
    pub fn message(&self) -> String {
        match self {
            Self::CannotAnswer { requires } => {
                format!("Cannot answer. Requires: {}", requires.join(", "))
            }
            Self::TypeMismatch { expected, got } => {
                format!("Type mismatch: expected {}, got '{}'", expected, got)
            }
            Self::NoNumericValue => "No numeric value in evidence".to_string(),
            Self::NoBooleanValue => "No boolean value in evidence".to_string(),
            Self::NoListValue => "No list value in evidence".to_string(),
            Self::NoEntityValue => "No entity value in evidence".to_string(),
        }
    }
}

/// Infer unit from fact name.
fn infer_unit(fact_name: &str) -> &'static str {
    let lower = fact_name.to_lowercase();
    if lower.contains("gib") || lower.contains("_gib") {
        " GiB"
    } else if lower.contains("mib") || lower.contains("_mib") {
        " MiB"
    } else if lower.contains("pct") || lower.contains("percent") {
        "%"
    } else if lower.contains("_s") || lower.contains("time_s") || lower.contains("seconds") {
        "s"
    } else if lower.contains("_ms") {
        "ms"
    } else if lower.contains("temp") || lower.contains("_c") {
        "°C"
    } else if lower.contains("count") {
        ""
    } else {
        ""
    }
}

/// Direct answer builder for deterministic cases.
pub struct DirectAnswerBuilder;

impl DirectAnswerBuilder {
    /// Build answer directly from evidence (no reasoning needed).
    pub fn build(
        fact_name: &str,
        evidence: &EvidenceBundle,
        answer_type: AnswerType,
    ) -> Option<String> {
        let value = evidence.get(fact_name)?;

        match answer_type {
            AnswerType::Numeric => {
                let n = value.as_number()?;
                let unit = infer_unit(fact_name);
                if n.fract() == 0.0 {
                    Some(format!("{}{}", n as i64, unit))
                } else {
                    Some(format!("{:.1}{}", n, unit))
                }
            }
            AnswerType::Boolean => {
                let b = value.as_bool()?;
                Some(if b {
                    "Yes.".to_string()
                } else {
                    "No.".to_string()
                })
            }
            AnswerType::List => {
                let list = value.as_list()?;
                Some(
                    list.iter()
                        .take(MAX_LIST_ITEMS)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", "),
                )
            }
            AnswerType::Entity => Some(value.as_string()?.to_string()),
            AnswerType::Brief => Some(value.display()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::era_pipeline::evidence::EvidenceBundleBuilder;

    #[test]
    fn test_numeric_answer() {
        let evidence = EvidenceBundleBuilder::new("DSK-0127")
            .fact_number("memory.free_gib", 17.0)
            .build();

        let reasoning = ReasoningOutput::answerable("DSK-0127", "Memory is 17 GiB", 0.95)
            .with_metric("17.0 GiB");

        let translator = PrecisionTranslator::new();
        let answer = translator
            .translate(&reasoning, &evidence, AnswerType::Numeric)
            .unwrap();

        assert_eq!(answer.text, "17.0 GiB");
        assert!(answer.type_match);
    }

    #[test]
    fn test_boolean_answer() {
        let evidence = EvidenceBundleBuilder::new("DSK-0127")
            .fact_bool("disk.trim_enabled", true)
            .build();

        let reasoning =
            ReasoningOutput::answerable("DSK-0127", "Trim is enabled", 0.9).with_metric("enabled");

        let translator = PrecisionTranslator::new();
        let answer = translator
            .translate(&reasoning, &evidence, AnswerType::Boolean)
            .unwrap();

        assert!(answer.text.starts_with("Yes"));
    }

    #[test]
    fn test_list_answer() {
        let evidence = EvidenceBundleBuilder::new("DSK-0127")
            .fact_list(
                "boot.blame",
                vec![
                    "NetworkManager.service (2.5s)".to_string(),
                    "systemd-udev-settle.service (1.2s)".to_string(),
                ],
            )
            .build();

        let reasoning = ReasoningOutput::answerable("DSK-0127", "Services listed", 0.9);

        let translator = PrecisionTranslator::new();
        let answer = translator
            .translate(&reasoning, &evidence, AnswerType::List)
            .unwrap();

        assert!(answer.text.contains("NetworkManager"));
    }

    #[test]
    fn test_entity_answer() {
        let evidence = EvidenceBundleBuilder::new("DSK-0127")
            .fact_string("gpu.model", "NVIDIA GeForce RTX 3080")
            .build();

        let reasoning = ReasoningOutput::answerable("DSK-0127", "GPU identified", 0.95)
            .with_metric("NVIDIA GeForce RTX 3080");

        let translator = PrecisionTranslator::new();
        let answer = translator
            .translate(&reasoning, &evidence, AnswerType::Entity)
            .unwrap();

        assert_eq!(answer.text, "NVIDIA GeForce RTX 3080");
    }

    #[test]
    fn test_cannot_answer() {
        let evidence = EvidenceBundle::new("DSK-0127");
        let reasoning =
            ReasoningOutput::unanswerable("DSK-0127", "Missing boot data", vec!["boot.blame"]);

        let translator = PrecisionTranslator::new();
        let result = translator.translate(&reasoning, &evidence, AnswerType::List);

        assert!(result.is_err());
        if let Err(TranslationError::CannotAnswer { requires }) = result {
            assert!(requires.contains(&"boot.blame".to_string()));
        }
    }

    #[test]
    fn test_direct_answer_builder() {
        let evidence = EvidenceBundleBuilder::new("DSK-0127")
            .fact_number("memory.free_gib", 17.5)
            .build();

        let answer = DirectAnswerBuilder::build("memory.free_gib", &evidence, AnswerType::Numeric);
        assert_eq!(answer, Some("17.5 GiB".to_string()));
    }

    #[test]
    fn test_type_validation() {
        assert!(validate_answer_type("17.0 GiB", AnswerType::Numeric));
        assert!(!validate_answer_type("hello world", AnswerType::Numeric));

        assert!(validate_answer_type("Yes.", AnswerType::Boolean));
        assert!(validate_answer_type(
            "No, it is not enabled.",
            AnswerType::Boolean
        ));
        assert!(!validate_answer_type("Maybe", AnswerType::Boolean));
    }
}
