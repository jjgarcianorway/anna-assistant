//! Batch validator for multiple responses - v0.0.440.

use super::validator_types::ValidationResult;

/// Batch validator for multiple responses.
pub struct BatchValidator {
    /// Results by case ID.
    pub results: Vec<(String, ValidationResult)>,
    /// Valid count.
    pub valid_count: usize,
    /// Invalid count.
    pub invalid_count: usize,
}

impl BatchValidator {
    /// Create empty batch.
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            valid_count: 0,
            invalid_count: 0,
        }
    }

    /// Add a validation result.
    pub fn add(&mut self, case_id: &str, result: ValidationResult) {
        if result.is_valid() {
            self.valid_count += 1;
        } else {
            self.invalid_count += 1;
        }
        self.results.push((case_id.to_string(), result));
    }

    /// Get success rate.
    pub fn success_rate(&self) -> f64 {
        let total = self.valid_count + self.invalid_count;
        if total == 0 {
            0.0
        } else {
            self.valid_count as f64 / total as f64
        }
    }
}

impl Default for BatchValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::validator_core::SrcValidator;
    use super::*;

    #[test]
    fn test_batch_validator() {
        let mut batch = BatchValidator::new();

        let validator = SrcValidator::new("DSK-0101");
        batch.add("DSK-0101", validator.validate(r#"{"case_id": "DSK-0101", "department": "Performance", "assessment": {"summary": "Test", "confidence": 0.9, "risk": "read_only"}}"#));
        batch.add("DSK-0102", validator.validate("invalid"));

        assert_eq!(batch.valid_count, 1);
        assert_eq!(batch.invalid_count, 1);
        assert!((batch.success_rate() - 0.5).abs() < 0.01);
    }
}
