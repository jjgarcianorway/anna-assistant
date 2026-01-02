//! Precision translator implementation.

use super::super::evidence::EvidenceBundle;
use super::super::pipeline::AnswerType;
use super::super::reasoning::ReasoningOutput;
use super::types::{TranslatedAnswer, TranslationError, MAX_BRIEF_ANSWER, MAX_LIST_ITEMS};
use super::utils::infer_unit;
use super::validation::create_validated_answer;

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

        let answer = create_validated_answer(&text, answer_type, reasoning.confidence);

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
}
