//! Evidence Binding (Part C) - v0.0.437.
//!
//! Every claim in the final answer must map to:
//! - One or more EvidenceRef IDs
//! - And must satisfy the intent's subject and allowed_fields
//!
//! If evidence exists but does not map cleanly:
//! - Anna must say "I collected data but cannot safely answer exactly what you asked."
//! - Then propose a clarification or alternative phrasing
//!
//! This prevents hallucinated glue text.

use super::intent::{QuestionIntent, Subject};
use serde::{Deserialize, Serialize};

/// A claim that is bound to evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundClaim {
    /// The claim text.
    pub text: String,
    /// Evidence IDs that support this claim.
    pub evidence_ids: Vec<String>,
    /// Field this claim corresponds to.
    pub field: String,
    /// Whether the binding is valid.
    pub binding_valid: bool,
}

impl BoundClaim {
    /// Create a new bound claim.
    pub fn new(text: &str, field: &str, evidence_ids: Vec<String>) -> Self {
        Self {
            text: text.to_string(),
            evidence_ids,
            field: field.to_string(),
            binding_valid: true,
        }
    }

    /// Check if claim has evidence.
    pub fn has_evidence(&self) -> bool {
        !self.evidence_ids.is_empty()
    }

    /// Mark as invalid binding.
    pub fn invalidate(&mut self, reason: &str) {
        self.binding_valid = false;
        self.text = format!("[Invalid: {}] {}", reason, self.text);
    }
}

/// Result of evidence binding.
#[derive(Debug, Clone)]
pub enum BindingResult {
    /// All claims are properly bound to evidence.
    Valid {
        claims: Vec<BoundClaim>,
    },
    /// Some claims could not be bound.
    PartialBind {
        valid_claims: Vec<BoundClaim>,
        unbound_claims: Vec<String>,
    },
    /// Evidence exists but doesn't match the question.
    MismatchedEvidence {
        message: String,
        suggestion: String,
    },
    /// No evidence at all.
    NoEvidence,
}

impl BindingResult {
    /// Check if binding is fully valid.
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid { .. })
    }

    /// Get valid claims if any.
    pub fn valid_claims(&self) -> Vec<&BoundClaim> {
        match self {
            Self::Valid { claims } => claims.iter().collect(),
            Self::PartialBind { valid_claims, .. } => valid_claims.iter().collect(),
            _ => Vec::new(),
        }
    }

    /// Get the fallback message for mismatched evidence.
    pub fn fallback_message(&self) -> Option<String> {
        match self {
            Self::MismatchedEvidence { message, suggestion } => {
                Some(format!("{}\n\nSuggestion: {}", message, suggestion))
            }
            Self::NoEvidence => {
                Some("I could not find evidence to answer this question.".to_string())
            }
            Self::PartialBind { unbound_claims, .. } if !unbound_claims.is_empty() => {
                Some(format!(
                    "I could not verify: {}",
                    unbound_claims.join(", ")
                ))
            }
            _ => None,
        }
    }
}

/// Evidence binding engine.
pub struct EvidenceBinding;

impl EvidenceBinding {
    /// Bind claims to evidence with strict validation.
    pub fn bind(
        intent: &QuestionIntent,
        claims: Vec<UnboundClaim>,
        evidence: &[EvidenceItem],
    ) -> BindingResult {
        if evidence.is_empty() {
            return BindingResult::NoEvidence;
        }

        // Check if evidence matches the subject
        let subject_match = evidence.iter().any(|e| {
            e.subject == intent.subject || intent.subject == Subject::Unknown
        });

        if !subject_match {
            // Evidence exists but for wrong subject
            return BindingResult::MismatchedEvidence {
                message: "I collected data but cannot safely answer exactly what you asked.".to_string(),
                suggestion: Self::suggest_alternative(intent, evidence),
            };
        }

        let mut valid_claims = Vec::new();
        let mut unbound_claims = Vec::new();

        for claim in claims {
            // Try to bind claim to evidence
            let bound = Self::try_bind_claim(&claim, evidence, intent);

            match bound {
                Some(bc) if bc.binding_valid && bc.has_evidence() => {
                    valid_claims.push(bc);
                }
                Some(bc) => {
                    // Claim exists but binding is weak
                    if intent.requires_evidence {
                        unbound_claims.push(claim.text);
                    } else {
                        valid_claims.push(bc);
                    }
                }
                None => {
                    unbound_claims.push(claim.text);
                }
            }
        }

        if valid_claims.is_empty() && !unbound_claims.is_empty() {
            BindingResult::MismatchedEvidence {
                message: "I collected data but cannot safely answer exactly what you asked.".to_string(),
                suggestion: "Try rephrasing your question or asking about what I found.".to_string(),
            }
        } else if !unbound_claims.is_empty() {
            BindingResult::PartialBind {
                valid_claims,
                unbound_claims,
            }
        } else {
            BindingResult::Valid {
                claims: valid_claims,
            }
        }
    }

    /// Try to bind a single claim to evidence.
    fn try_bind_claim(
        claim: &UnboundClaim,
        evidence: &[EvidenceItem],
        intent: &QuestionIntent,
    ) -> Option<BoundClaim> {
        // Find evidence that matches the claim's field
        let matching_evidence: Vec<String> = evidence
            .iter()
            .filter(|e| {
                // Check if evidence field matches claim field
                e.fields.contains(&claim.field) ||
                // Or if evidence covers the claim text
                claim.text.to_lowercase().contains(&e.summary.to_lowercase()) ||
                e.summary.to_lowercase().contains(&claim.text.to_lowercase())
            })
            .map(|e| e.id.clone())
            .collect();

        if matching_evidence.is_empty() && intent.requires_evidence {
            return None;
        }

        let mut bound = BoundClaim::new(&claim.text, &claim.field, matching_evidence);

        // Validate field is allowed
        if !intent.is_field_allowed(&claim.field) && !intent.allows_extras() {
            bound.invalidate("field not allowed");
        }

        Some(bound)
    }

    /// Suggest an alternative question based on available evidence.
    fn suggest_alternative(intent: &QuestionIntent, evidence: &[EvidenceItem]) -> String {
        let available_subjects: Vec<_> = evidence
            .iter()
            .map(|e| e.subject.label())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        if available_subjects.is_empty() {
            return "Try asking a more specific question.".to_string();
        }

        let asked_subject = intent.subject.label();
        format!(
            "You asked about {}. I have data about: {}. Would you like to know about one of those instead?",
            asked_subject,
            available_subjects.join(", ")
        )
    }

    /// Validate that all claims have proper evidence support.
    pub fn validate_binding(claims: &[BoundClaim]) -> Vec<BindingViolation> {
        let mut violations = Vec::new();

        for claim in claims {
            if !claim.has_evidence() {
                violations.push(BindingViolation::NoEvidence {
                    claim: claim.text.clone(),
                });
            }

            if !claim.binding_valid {
                violations.push(BindingViolation::InvalidBinding {
                    claim: claim.text.clone(),
                });
            }
        }

        violations
    }
}

/// An unbound claim from specialist output.
#[derive(Debug, Clone)]
pub struct UnboundClaim {
    /// The claim text.
    pub text: String,
    /// Field this claim corresponds to.
    pub field: String,
}

impl UnboundClaim {
    /// Create a new unbound claim.
    pub fn new(text: &str, field: &str) -> Self {
        Self {
            text: text.to_string(),
            field: field.to_string(),
        }
    }
}

/// An evidence item available for binding.
#[derive(Debug, Clone)]
pub struct EvidenceItem {
    /// Evidence ID.
    pub id: String,
    /// Subject this evidence relates to.
    pub subject: Subject,
    /// Fields this evidence provides.
    pub fields: Vec<String>,
    /// Brief summary.
    pub summary: String,
}

impl EvidenceItem {
    /// Create a new evidence item.
    pub fn new(id: &str, subject: Subject, fields: Vec<&str>, summary: &str) -> Self {
        Self {
            id: id.to_string(),
            subject,
            fields: fields.into_iter().map(String::from).collect(),
            summary: summary.to_string(),
        }
    }
}

/// Binding violation types.
#[derive(Debug, Clone)]
pub enum BindingViolation {
    /// Claim has no evidence support.
    NoEvidence { claim: String },
    /// Binding is marked invalid.
    InvalidBinding { claim: String },
    /// Evidence doesn't match subject.
    SubjectMismatch { expected: Subject, found: Subject },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::question_contract::intent::{IntentBuilder, IntentCategory};

    #[test]
    fn test_valid_binding() {
        let intent = IntentBuilder::new("int_001")
            .category(IntentCategory::Fact)
            .subject(Subject::Memory)
            .allow_fields(vec!["free"])
            .build();

        let claims = vec![
            UnboundClaim::new("4.2 GB free", "free"),
        ];

        let evidence = vec![
            EvidenceItem::new("ev_mem", Subject::Memory, vec!["free", "total"], "Memory usage"),
        ];

        let result = EvidenceBinding::bind(&intent, claims, &evidence);

        assert!(result.is_valid());
        assert_eq!(result.valid_claims().len(), 1);
    }

    #[test]
    fn test_mismatched_evidence() {
        let intent = IntentBuilder::new("int_002")
            .category(IntentCategory::Fact)
            .subject(Subject::Memory)
            .build();

        let claims = vec![
            UnboundClaim::new("Intel Core i7", "cpu_model"),
        ];

        // Evidence is about CPU, not memory
        let evidence = vec![
            EvidenceItem::new("ev_cpu", Subject::Cpu, vec!["model"], "CPU info"),
        ];

        let result = EvidenceBinding::bind(&intent, claims, &evidence);

        match result {
            BindingResult::MismatchedEvidence { message, .. } => {
                assert!(message.contains("cannot safely answer"));
            }
            _ => panic!("Expected MismatchedEvidence"),
        }
    }

    #[test]
    fn test_no_evidence() {
        let intent = IntentBuilder::new("int_003")
            .category(IntentCategory::Fact)
            .subject(Subject::Memory)
            .build();

        let claims = vec![
            UnboundClaim::new("4.2 GB free", "free"),
        ];

        let result = EvidenceBinding::bind(&intent, claims, &[]);

        assert!(matches!(result, BindingResult::NoEvidence));
    }

    #[test]
    fn test_partial_binding() {
        let intent = IntentBuilder::new("int_004")
            .category(IntentCategory::Fact)
            .subject(Subject::Memory)
            .allow_fields(vec!["free", "total"])
            .build();

        let claims = vec![
            UnboundClaim::new("4.2 GB free", "free"),
            UnboundClaim::new("Some unverified claim", "unknown"),
        ];

        let evidence = vec![
            EvidenceItem::new("ev_mem", Subject::Memory, vec!["free"], "Memory free"),
        ];

        let result = EvidenceBinding::bind(&intent, claims, &evidence);

        match result {
            BindingResult::PartialBind { valid_claims, unbound_claims } => {
                assert_eq!(valid_claims.len(), 1);
                assert_eq!(unbound_claims.len(), 1);
            }
            _ => panic!("Expected PartialBind"),
        }
    }

    #[test]
    fn test_binding_validation() {
        let claims = vec![
            BoundClaim::new("Valid claim", "field", vec!["ev_1".to_string()]),
            BoundClaim::new("No evidence claim", "field", vec![]),
        ];

        let violations = EvidenceBinding::validate_binding(&claims);

        assert_eq!(violations.len(), 1);
        assert!(matches!(violations[0], BindingViolation::NoEvidence { .. }));
    }

    #[test]
    fn test_fallback_message() {
        let result = BindingResult::MismatchedEvidence {
            message: "Cannot answer".to_string(),
            suggestion: "Try rephrasing".to_string(),
        };

        let msg = result.fallback_message().unwrap();
        assert!(msg.contains("Cannot answer"));
        assert!(msg.contains("Try rephrasing"));
    }
}
