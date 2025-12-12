//! Answer Contract (v0.0.445).
//!
//! Enforces: answer shape = question shape.
//!
//! Examples:
//! - "how much free ram" → single number + unit
//! - "is X enabled" → Yes/No + one sentence why
//! - "where is config" → single path
//! - "why is X slow" → short explanation + named cause(s)

use super::claim_evidence::{ClaimType, EvidenceBinding};
use serde::{Deserialize, Serialize};

/// Expected answer shape based on question type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnswerShape {
    /// Single number + unit (e.g., "17.0 GiB")
    SingleMetric,
    /// Yes/No + one sentence explanation
    BooleanWithReason,
    /// Single path or location
    SinglePath,
    /// Short explanation with named cause(s)
    Explanation,
    /// List of items (when explicitly asked for list)
    List,
    /// Status check (running/stopped/etc)
    Status,
}

impl AnswerShape {
    /// Infer answer shape from question.
    pub fn from_question(question: &str) -> Self {
        let q = question.to_lowercase();

        // Boolean questions
        if q.starts_with("is ")
            || q.starts_with("are ")
            || q.starts_with("does ")
            || q.starts_with("do ")
            || q.starts_with("can ")
            || q.starts_with("has ")
            || q.starts_with("have ")
            || q.contains(" enabled")
            || q.contains(" installed")
            || q.contains(" running")
            || q.contains(" active")
        {
            return Self::BooleanWithReason;
        }

        // Metric questions
        if q.contains("how much")
            || q.contains("how many")
            || q.contains("what size")
            || q.contains("how big")
            || q.starts_with("what is the")
            || q.contains(" usage")
            || q.contains(" free ")
            || q.contains(" available ")
        {
            return Self::SingleMetric;
        }

        // Path questions
        if q.starts_with("where ")
            || q.contains("what path")
            || q.contains("which file")
            || q.contains("which folder")
            || q.contains("location of")
        {
            return Self::SinglePath;
        }

        // List questions
        if q.starts_with("list ")
            || q.starts_with("show all")
            || q.contains("what are the")
            || q.contains("which services")
            || q.contains("which packages")
        {
            return Self::List;
        }

        // Status questions
        if q.contains("status of")
            || q.contains("state of")
            || (q.contains("is ") && q.contains(" service"))
        {
            return Self::Status;
        }

        // Explanation questions (why, how come, etc.)
        Self::Explanation
    }

    /// Get allowed claim types for this shape.
    pub fn allowed_claim_types(&self) -> Vec<ClaimType> {
        match self {
            Self::SingleMetric => vec![ClaimType::Metric],
            Self::BooleanWithReason => vec![ClaimType::Boolean],
            Self::SinglePath => vec![ClaimType::Path],
            Self::Explanation => vec![ClaimType::Diagnosis],
            Self::List => vec![ClaimType::List],
            Self::Status => vec![ClaimType::Status],
        }
    }

    /// Maximum expected claims for this shape.
    pub fn max_claims(&self) -> usize {
        match self {
            Self::SingleMetric => 1,
            Self::BooleanWithReason => 2, // yes/no + reason
            Self::SinglePath => 1,
            Self::Explanation => 3, // explanation can have multiple causes
            Self::List => 10,       // lists can be longer
            Self::Status => 1,
        }
    }
}

/// Contract violation types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractViolation {
    /// Too many claims for expected shape
    TooManyClaims { expected: usize, got: usize },
    /// Wrong claim type for expected shape
    WrongClaimType {
        expected: Vec<ClaimType>,
        got: ClaimType,
    },
    /// Missing required claim type
    MissingRequiredClaim { expected: ClaimType },
    /// Extra unrelated content detected
    ExtraContent { description: String },
}

/// Answer contract inferred from question.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerContract {
    /// Original question
    pub question: String,
    /// Expected answer shape
    pub shape: AnswerShape,
    /// Domain (e.g., "system", "storage")
    pub domain: String,
}

impl AnswerContract {
    /// Create contract from question.
    pub fn from_question(question: &str, domain: &str) -> Self {
        Self {
            question: question.to_string(),
            shape: AnswerShape::from_question(question),
            domain: domain.to_string(),
        }
    }

    /// Create with explicit shape.
    pub fn with_shape(question: &str, domain: &str, shape: AnswerShape) -> Self {
        Self {
            question: question.to_string(),
            shape,
            domain: domain.to_string(),
        }
    }

    /// Validate answer against contract.
    /// Returns None if valid, Some(violation) if invalid.
    pub fn validate_answer(&self, binding: &EvidenceBinding) -> Option<ContractViolation> {
        // Check claim count
        let max_claims = self.shape.max_claims();
        if binding.claims.len() > max_claims {
            return Some(ContractViolation::TooManyClaims {
                expected: max_claims,
                got: binding.claims.len(),
            });
        }

        // Check claim types
        let allowed = self.shape.allowed_claim_types();
        for claim in &binding.claims {
            if !allowed.contains(&claim.claim_type) {
                return Some(ContractViolation::WrongClaimType {
                    expected: allowed.clone(),
                    got: claim.claim_type,
                });
            }
        }

        // Check for required claim type (at least one)
        if !binding.claims.is_empty() {
            let has_required = binding
                .claims
                .iter()
                .any(|c| allowed.contains(&c.claim_type));
            if !has_required {
                return Some(ContractViolation::MissingRequiredClaim {
                    expected: allowed[0],
                });
            }
        }

        None
    }

    /// Check if answer is about the right domain.
    pub fn is_relevant_domain(&self, answer_domain: &str) -> bool {
        // Same domain
        if self.domain == answer_domain {
            return true;
        }

        // Related domains
        let related = match self.domain.as_str() {
            "system" => vec!["memory", "cpu", "kernel", "uptime"],
            "storage" => vec!["disk", "filesystem", "mount"],
            "network" => vec!["interface", "ip", "dns"],
            "services" => vec!["systemd", "daemon"],
            _ => vec![],
        };

        related.contains(&answer_domain)
    }
}

/// Detect if answer contains generic/irrelevant content.
pub fn detect_generic_content(answer: &str, question_domain: &str) -> Option<String> {
    let answer_lower = answer.to_lowercase();

    // Generic system info patterns that are often irrelevant
    let generic_patterns = [
        ("system information", "generic system dump"),
        ("cpu info", "unrequested CPU info"),
        ("memory info", "unrequested memory info"),
        ("disk usage", "unrequested disk info"),
        ("here's your system", "generic overview"),
    ];

    // Check if answer contains generic patterns unrelated to question
    for (pattern, description) in generic_patterns {
        if answer_lower.contains(pattern) {
            // Check if pattern is related to question domain
            let is_relevant = match question_domain {
                "system" | "performance" => pattern.contains("cpu") || pattern.contains("memory"),
                "storage" => pattern.contains("disk"),
                _ => false,
            };

            if !is_relevant {
                return Some(description.to_string());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reliability_gate::claim_evidence::{ClaimType, StrictClaim};

    #[test]
    fn test_infer_boolean_shape() {
        assert_eq!(
            AnswerShape::from_question("is swap enabled"),
            AnswerShape::BooleanWithReason
        );
        assert_eq!(
            AnswerShape::from_question("do I have vim installed"),
            AnswerShape::BooleanWithReason
        );
        assert_eq!(
            AnswerShape::from_question("is nginx running"),
            AnswerShape::BooleanWithReason
        );
    }

    #[test]
    fn test_infer_metric_shape() {
        assert_eq!(
            AnswerShape::from_question("how much free ram do I have"),
            AnswerShape::SingleMetric
        );
        assert_eq!(
            AnswerShape::from_question("what is my disk usage"),
            AnswerShape::SingleMetric
        );
    }

    #[test]
    fn test_infer_path_shape() {
        assert_eq!(
            AnswerShape::from_question("where is the nginx config"),
            AnswerShape::SinglePath
        );
        assert_eq!(
            AnswerShape::from_question("which file contains ssh settings"),
            AnswerShape::SinglePath
        );
    }

    #[test]
    fn test_contract_validation_too_many_claims() {
        let contract = AnswerContract::from_question("is swap enabled", "system");
        let mut binding = EvidenceBinding::new("REQ-001");

        // Add too many claims for boolean shape (max 2)
        for i in 0..5 {
            binding.add_claim(StrictClaim::new(
                &format!("C{}", i),
                "claim",
                ClaimType::Boolean,
                "system",
            ));
        }

        let violation = contract.validate_answer(&binding);
        assert!(matches!(
            violation,
            Some(ContractViolation::TooManyClaims { .. })
        ));
    }

    #[test]
    fn test_contract_validation_wrong_type() {
        let contract = AnswerContract::from_question("is swap enabled", "system");
        let mut binding = EvidenceBinding::new("REQ-001");

        // Add metric claim to boolean question
        binding.add_claim(StrictClaim::new(
            "C1",
            "swap is 4 GiB",
            ClaimType::Metric, // Wrong - should be Boolean
            "system",
        ));

        let violation = contract.validate_answer(&binding);
        assert!(matches!(
            violation,
            Some(ContractViolation::WrongClaimType { .. })
        ));
    }

    #[test]
    fn test_detect_generic_content() {
        // When asking about services, generic system info is irrelevant
        let result = detect_generic_content(
            "Here's your System Information:\nCPU: Intel i7\nMemory: 16GB",
            "services",
        );
        assert!(result.is_some());

        // When asking about performance, CPU info is relevant
        let result = detect_generic_content("CPU Info: Intel i7 at 3.4GHz", "performance");
        assert!(result.is_none());
    }
}
