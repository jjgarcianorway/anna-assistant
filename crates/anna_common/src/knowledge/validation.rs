//! Fact Validation v0.16.1
//!
//! Validates facts before storing them in the knowledge store.
//! Ensures user-provided or LLM-inferred facts are double-checked against evidence.

use super::schema::{Fact, FactStatus};
use chrono::Utc;

/// Validation result for a fact
#[derive(Debug, Clone)]
pub enum ValidationResult {
    /// Fact is valid and can be stored
    Valid {
        confidence_adjustment: f64,
        notes: Option<String>,
    },
    /// Fact needs verification before storing
    NeedsVerification {
        reason: String,
        suggested_probe: Option<String>,
    },
    /// Fact conflicts with existing evidence
    Conflict {
        existing_value: String,
        reason: String,
    },
    /// Fact should be rejected
    Rejected { reason: String },
}

/// Fact validator
pub struct FactValidator {
    /// Minimum confidence for auto-acceptance
    min_auto_accept_confidence: f64,
    /// Maximum staleness hours before requiring revalidation
    max_staleness_hours: i64,
}

impl Default for FactValidator {
    fn default() -> Self {
        Self {
            min_auto_accept_confidence: 0.85,
            max_staleness_hours: 24,
        }
    }
}

impl FactValidator {
    /// Create a new fact validator
    pub fn new(min_confidence: f64, max_staleness_hours: i64) -> Self {
        Self {
            min_auto_accept_confidence: min_confidence,
            max_staleness_hours,
        }
    }

    /// Validate a fact before storing
    pub fn validate(&self, fact: &Fact, existing: Option<&Fact>) -> ValidationResult {
        // Check source type
        let is_probe_source = fact.source.starts_with("probe:");
        let is_llm_source = fact.source.starts_with("llm:");
        let is_user_source = fact.source.starts_with("user:");

        // Probe sources are auto-validated with high confidence
        if is_probe_source {
            return ValidationResult::Valid {
                confidence_adjustment: 0.0, // No adjustment for probe data
                notes: None,
            };
        }

        // Check for conflicts with existing facts
        if let Some(existing_fact) = existing {
            if existing_fact.value != fact.value {
                // Check if existing fact is more reliable
                if existing_fact.confidence > fact.confidence
                    && existing_fact.source.starts_with("probe:")
                {
                    return ValidationResult::Conflict {
                        existing_value: existing_fact.value.clone(),
                        reason: format!(
                            "Existing probe-verified value '{}' conflicts with new value '{}'",
                            existing_fact.value, fact.value
                        ),
                    };
                }

                // If existing is stale, allow update with note
                if existing_fact.is_stale_by_age(self.max_staleness_hours) {
                    return ValidationResult::Valid {
                        confidence_adjustment: -0.1, // Slight penalty for updating stale data
                        notes: Some(format!(
                            "Updated stale value '{}' -> '{}'",
                            existing_fact.value, fact.value
                        )),
                    };
                }

                // Non-stale existing fact with different value
                return ValidationResult::NeedsVerification {
                    reason: format!(
                        "Existing value '{}' differs from new value '{}'",
                        existing_fact.value, fact.value
                    ),
                    suggested_probe: self.suggest_probe_for_entity(&fact.entity),
                };
            }

            // Same value - just refresh
            return ValidationResult::Valid {
                confidence_adjustment: 0.05, // Slight boost for confirmation
                notes: Some("Confirmed existing value".to_string()),
            };
        }

        // New fact from LLM or user needs verification if confidence is low
        if is_llm_source || is_user_source {
            if fact.confidence < self.min_auto_accept_confidence {
                return ValidationResult::NeedsVerification {
                    reason: format!(
                        "Low confidence ({:.0}%) - requires verification",
                        fact.confidence * 100.0
                    ),
                    suggested_probe: self.suggest_probe_for_entity(&fact.entity),
                };
            }

            // High confidence LLM/user fact - accept with note
            return ValidationResult::Valid {
                confidence_adjustment: -0.05, // Slight penalty for non-probe source
                notes: Some(format!("Accepted from {} source", if is_llm_source { "LLM" } else { "user" })),
            };
        }

        // Unknown source type
        ValidationResult::NeedsVerification {
            reason: "Unknown source type".to_string(),
            suggested_probe: None,
        }
    }

    /// Suggest a probe to verify a fact based on entity type
    fn suggest_probe_for_entity(&self, entity: &str) -> Option<String> {
        let prefix = entity.split(':').next().unwrap_or("");
        match prefix {
            "cpu" => Some("cpu.info".to_string()),
            "gpu" => Some("gpu.info".to_string()),
            "disk" | "fs" => Some("disk.list".to_string()),
            "pkg" => Some("pkg.list".to_string()),
            "svc" => Some("service.status".to_string()),
            "net" => Some("network.info".to_string()),
            "system" => Some("os.info".to_string()),
            _ => None,
        }
    }

    /// Validate and adjust a fact, returning the adjusted fact if valid
    pub fn validate_and_adjust(&self, mut fact: Fact, existing: Option<&Fact>) -> Option<Fact> {
        match self.validate(&fact, existing) {
            ValidationResult::Valid {
                confidence_adjustment,
                notes,
            } => {
                // Adjust confidence (clamp to [0, 1])
                fact.confidence = (fact.confidence + confidence_adjustment).clamp(0.0, 1.0);

                // Add validation notes
                if let Some(note) = notes {
                    fact.notes = Some(match fact.notes {
                        Some(existing) => format!("{} | {}", existing, note),
                        None => note,
                    });
                }

                Some(fact)
            }
            ValidationResult::Conflict {
                existing_value,
                reason,
            } => {
                // Mark as conflicted
                fact.status = FactStatus::Conflicted;
                fact.notes = Some(format!("CONFLICT: {} (existing: {})", reason, existing_value));
                Some(fact) // Still store but mark conflicted
            }
            ValidationResult::NeedsVerification { reason, .. } => {
                // Lower confidence and mark as needing verification
                fact.confidence *= 0.5;
                fact.notes = Some(format!("UNVERIFIED: {}", reason));
                Some(fact) // Store with lower confidence
            }
            ValidationResult::Rejected { reason: _ } => None
        }
    }
}

/// Check if a value looks like valid system data (not hallucinated)
pub fn looks_valid(entity: &str, attribute: &str, value: &str) -> bool {
    // Empty values are suspicious
    if value.is_empty() {
        return false;
    }

    // Check for common hallucination patterns
    let hallucination_patterns = [
        "example",
        "lorem",
        "ipsum",
        "foo",
        "bar",
        "test123",
        "placeholder",
        "unknown",
        "n/a",
    ];

    let value_lower = value.to_lowercase();
    for pattern in hallucination_patterns {
        if value_lower.contains(pattern) {
            return false;
        }
    }

    // Entity-specific validation
    let prefix = entity.split(':').next().unwrap_or("");
    match (prefix, attribute) {
        ("cpu", "cores") => value.parse::<u32>().map(|n| n > 0 && n <= 1024).unwrap_or(false),
        ("cpu", "frequency_mhz") => value.parse::<f64>().map(|n| n > 100.0 && n < 10000.0).unwrap_or(false),
        ("disk", "size_bytes") => value.parse::<u64>().is_ok(),
        ("pkg", "version") => !value.is_empty() && value.len() < 100,
        ("svc", "state") => ["running", "stopped", "inactive", "active", "failed"].contains(&value),
        _ => true, // Allow unknown entity/attribute combinations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_probe_fact(entity: &str, attr: &str, value: &str, confidence: f64) -> Fact {
        Fact::from_probe(
            entity.to_string(),
            attr.to_string(),
            value.to_string(),
            "test.probe",
            confidence,
        )
    }

    fn make_llm_fact(entity: &str, attr: &str, value: &str, confidence: f64) -> Fact {
        Fact::from_llm(
            entity.to_string(),
            attr.to_string(),
            value.to_string(),
            "reasoning",
            confidence,
        )
    }

    #[test]
    fn test_probe_fact_auto_valid() {
        let validator = FactValidator::default();
        let fact = make_probe_fact("cpu:0", "cores", "8", 0.95);

        match validator.validate(&fact, None) {
            ValidationResult::Valid { confidence_adjustment, .. } => {
                assert_eq!(confidence_adjustment, 0.0);
            }
            _ => panic!("Expected Valid result"),
        }
    }

    #[test]
    fn test_llm_fact_low_confidence_needs_verification() {
        let validator = FactValidator::default();
        let fact = make_llm_fact("cpu:0", "cores", "8", 0.6);

        match validator.validate(&fact, None) {
            ValidationResult::NeedsVerification { suggested_probe, .. } => {
                assert_eq!(suggested_probe, Some("cpu.info".to_string()));
            }
            _ => panic!("Expected NeedsVerification result"),
        }
    }

    #[test]
    fn test_llm_fact_high_confidence_valid() {
        let validator = FactValidator::default();
        let fact = make_llm_fact("cpu:0", "cores", "8", 0.9);

        match validator.validate(&fact, None) {
            ValidationResult::Valid { confidence_adjustment, .. } => {
                assert!(confidence_adjustment < 0.0); // Slight penalty for LLM source
            }
            _ => panic!("Expected Valid result"),
        }
    }

    #[test]
    fn test_conflict_with_probe_data() {
        let validator = FactValidator::default();
        let existing = make_probe_fact("cpu:0", "cores", "8", 0.95);
        let new_fact = make_llm_fact("cpu:0", "cores", "16", 0.8);

        match validator.validate(&new_fact, Some(&existing)) {
            ValidationResult::Conflict { existing_value, .. } => {
                assert_eq!(existing_value, "8");
            }
            _ => panic!("Expected Conflict result"),
        }
    }

    #[test]
    fn test_same_value_confirmation() {
        let validator = FactValidator::default();
        let existing = make_probe_fact("cpu:0", "cores", "8", 0.9);
        let new_fact = make_llm_fact("cpu:0", "cores", "8", 0.8);

        match validator.validate(&new_fact, Some(&existing)) {
            ValidationResult::Valid { confidence_adjustment, notes } => {
                assert!(confidence_adjustment > 0.0); // Boost for confirmation
                assert!(notes.is_some());
            }
            _ => panic!("Expected Valid result"),
        }
    }

    #[test]
    fn test_looks_valid_cpu_cores() {
        assert!(looks_valid("cpu:0", "cores", "8"));
        assert!(looks_valid("cpu:0", "cores", "16"));
        assert!(!looks_valid("cpu:0", "cores", "0"));
        assert!(!looks_valid("cpu:0", "cores", "9999"));
        assert!(!looks_valid("cpu:0", "cores", "foo"));
    }

    #[test]
    fn test_looks_valid_rejects_hallucinations() {
        assert!(!looks_valid("pkg:test", "version", "example_version"));
        assert!(!looks_valid("pkg:test", "version", "placeholder"));
        assert!(!looks_valid("pkg:test", "version", ""));
    }

    #[test]
    fn test_validate_and_adjust() {
        let validator = FactValidator::default();
        let fact = make_llm_fact("cpu:0", "cores", "8", 0.9);

        let adjusted = validator.validate_and_adjust(fact, None);
        assert!(adjusted.is_some());

        let adjusted = adjusted.unwrap();
        assert!(adjusted.confidence < 0.9); // Should have penalty
        assert!(adjusted.notes.is_some());
    }
}
