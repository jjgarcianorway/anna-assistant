//! Assessment section of SRC v1.

use serde::{Deserialize, Serialize};

use super::constants::{truncate_str, MAX_SUMMARY_CHARS};
use super::types::SrcRisk;

/// Assessment section of SRC v1.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SrcAssessment {
    /// One sentence summary, no markdown, max 140 chars.
    pub summary: String,
    /// Confidence level 0.0-1.0.
    pub confidence: f64,
    /// Risk level.
    pub risk: SrcRisk,
}

impl SrcAssessment {
    /// Create a new assessment.
    pub fn new(summary: &str, confidence: f64, risk: SrcRisk) -> Self {
        Self {
            summary: truncate_str(summary, MAX_SUMMARY_CHARS),
            confidence: confidence.clamp(0.0, 1.0),
            risk,
        }
    }

    /// Validate the assessment.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.summary.is_empty() {
            errors.push("summary cannot be empty".to_string());
        }
        if self.summary.len() > MAX_SUMMARY_CHARS {
            errors.push(format!("summary exceeds {} chars", MAX_SUMMARY_CHARS));
        }
        if self.summary.contains('#') || self.summary.contains('*') || self.summary.contains('`') {
            errors.push("summary contains markdown (# * `)".to_string());
        }
        if self.confidence < 0.0 || self.confidence > 1.0 {
            errors.push("confidence must be 0.0-1.0".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_src_assessment_validation() {
        let valid = SrcAssessment::new("Boot time is 7.5 seconds.", 0.9, SrcRisk::ReadOnly);
        assert!(valid.validate().is_ok());

        let with_markdown = SrcAssessment {
            summary: "# Boot time is slow".to_string(),
            confidence: 0.9,
            risk: SrcRisk::ReadOnly,
        };
        assert!(with_markdown.validate().is_err());
    }
}
