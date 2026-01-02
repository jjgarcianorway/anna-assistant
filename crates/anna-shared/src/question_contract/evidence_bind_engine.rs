//! Evidence Binding Engine - v0.0.437.
//!
//! Core logic for binding claims to evidence with strict validation.

use super::intent::{QuestionIntent, Subject};
use super::evidence_bind_types::{
    BindingResult, BindingViolation, BoundClaim, EvidenceItem, UnboundClaim,
};

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
        let subject_match = evidence
            .iter()
            .any(|e| e.subject == intent.subject || intent.subject == Subject::Unknown);

        if !subject_match {
            // Evidence exists but for wrong subject
            return BindingResult::MismatchedEvidence {
                message: "I collected data but cannot safely answer exactly what you asked."
                    .to_string(),
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
                message: "I collected data but cannot safely answer exactly what you asked."
                    .to_string(),
                suggestion: "Try rephrasing your question or asking about what I found."
                    .to_string(),
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
