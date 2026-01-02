//! Reasoning validation and quality checks.

use super::evidence::EvidenceBundle;
use super::reasoning_types::{ReasoningOutput, MAX_REASONING_CHARS};

/// Validator for reasoning output.
pub struct ReasoningValidator {
    /// Expected case ID.
    expected_case_id: String,
    /// Available facts from evidence.
    available_facts: Vec<String>,
}

impl ReasoningValidator {
    /// Create validator.
    pub fn new(case_id: &str, evidence: &EvidenceBundle) -> Self {
        Self {
            expected_case_id: case_id.to_string(),
            available_facts: evidence.facts.keys().cloned().collect(),
        }
    }

    /// Validate reasoning output.
    pub fn validate(&self, output: &ReasoningOutput) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check case ID
        if output.case_id != self.expected_case_id {
            errors.push(format!(
                "case_id mismatch: expected '{}', got '{}'",
                self.expected_case_id, output.case_id
            ));
        }

        // Check reasoning length
        if output.reasoning.len() > MAX_REASONING_CHARS {
            errors.push(format!(
                "reasoning too long: {} > {}",
                output.reasoning.len(),
                MAX_REASONING_CHARS
            ));
        }

        // Check confidence range
        if output.confidence < 0.0 || output.confidence > 1.0 {
            errors.push(format!("confidence out of range: {}", output.confidence));
        }

        // If can_answer=false, requires must not be empty
        if !output.can_answer && output.requires.is_empty() {
            errors.push("can_answer=false but requires is empty".to_string());
        }

        // If can_answer=true, confidence should be > 0
        if output.can_answer && output.confidence == 0.0 {
            errors.push("can_answer=true but confidence=0".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Reasoning quality check.
#[derive(Debug, Clone)]
pub struct ReasoningQuality {
    /// Is reasoning grounded in evidence?
    pub grounded: bool,
    /// Did reasoning stay within scope?
    pub within_scope: bool,
    /// Is derived data consistent with evidence?
    pub consistent: bool,
}

impl ReasoningQuality {
    /// Check reasoning quality against evidence.
    pub fn check(output: &ReasoningOutput, evidence: &EvidenceBundle) -> Self {
        // Basic checks - more sophisticated checks could use NLP
        let grounded = !output.reasoning.is_empty();
        let within_scope = output.reasoning.len() <= MAX_REASONING_CHARS;
        let consistent = output.can_answer || !output.requires.is_empty();

        Self {
            grounded,
            within_scope,
            consistent,
        }
    }

    /// Overall quality score.
    pub fn score(&self) -> f64 {
        let mut score = 0.0;
        if self.grounded {
            score += 0.4;
        }
        if self.within_scope {
            score += 0.3;
        }
        if self.consistent {
            score += 0.3;
        }
        score
    }

    /// Is quality acceptable?
    pub fn is_acceptable(&self) -> bool {
        self.grounded && self.within_scope && self.consistent
    }
}

#[cfg(test)]
mod tests {
    use super::super::evidence::FactValue;
    use super::*;

    #[test]
    fn test_reasoning_validation() {
        let mut bundle = EvidenceBundle::new("DSK-0127");
        bundle.add_fact("memory.free_gib", FactValue::Number(17.0));

        let validator = ReasoningValidator::new("DSK-0127", &bundle);

        let valid_output = ReasoningOutput::answerable("DSK-0127", "Test", 0.9);
        assert!(validator.validate(&valid_output).is_ok());

        let invalid_output = ReasoningOutput::answerable("WRONG-ID", "Test", 0.9);
        assert!(validator.validate(&invalid_output).is_err());
    }

    #[test]
    fn test_reasoning_quality() {
        let output = ReasoningOutput::answerable("DSK-0127", "Grounded reasoning.", 0.9);
        let bundle = EvidenceBundle::new("DSK-0127");

        let quality = ReasoningQuality::check(&output, &bundle);
        assert!(quality.is_acceptable());
        assert!(quality.score() > 0.9);
    }
}
