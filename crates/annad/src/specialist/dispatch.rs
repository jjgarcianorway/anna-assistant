//! Dispatch Engine - Routes intents to specialists.
//!
//! Deterministic routing based on intent, domain, and confidence.
//! Enforces escalation rules: only juniors escalate, HIGH confidence refuses.

use super::domain::Domain;
use super::registry::{get_junior, get_senior, SpecialistDefinition, SpecialistLevel};
use crate::translator::decision::TranslatorDecision;
use crate::translator::intent::UserIntent;
use serde::{Deserialize, Serialize};

/// HIGH confidence threshold - refuse escalation above this.
pub const CONFIDENCE_HIGH: f32 = 0.85;

/// Dispatch decision result.
#[derive(Debug, Clone)]
pub enum DispatchDecision {
    /// Assign to specialist.
    Assign {
        specialist: &'static SpecialistDefinition,
        intent: UserIntent,
        recipe_hint: Option<String>,
    },

    /// Escalate from junior to senior.
    Escalate {
        from: &'static SpecialistDefinition,
        to: &'static SpecialistDefinition,
        reason: EscalationReason,
        intent: UserIntent,
    },

    /// No suitable specialist found.
    NoSpecialist { intent: UserIntent, domain: Domain },

    /// HIGH confidence - execute recipe directly (skip specialist).
    DirectExecution {
        intent: UserIntent,
        recipe_id: String,
        confidence: f32,
    },
}

/// Reason for escalation from junior to senior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EscalationReason {
    /// Confidence too low for junior.
    LowConfidence { junior_confidence: f32 },
    /// Action too complex for junior.
    ComplexAction { action: String },
    /// Senior expertise required.
    ExpertiseRequired { keywords: Vec<String> },
    /// Junior failed to resolve.
    JuniorFailed { error: String },
}

/// The dispatch engine routes intents to specialists.
pub struct DispatchEngine;

impl DispatchEngine {
    /// Route a TranslatorDecision to a specialist.
    pub fn dispatch(decision: TranslatorDecision) -> DispatchDecision {
        match decision {
            // HIGH confidence + recipe = direct execution, skip specialist
            TranslatorDecision::ExecuteRecipe {
                recipe,
                intent,
                confidence,
            } if confidence >= CONFIDENCE_HIGH => DispatchDecision::DirectExecution {
                intent,
                recipe_id: recipe.id,
                confidence,
            },

            // Medium confidence with recipe = assign to junior
            TranslatorDecision::ExecuteRecipe {
                recipe,
                intent,
                confidence: _,
            } => {
                let domain = Domain::from_subject(&intent.subject);
                if let Some(junior) = get_junior(domain) {
                    DispatchDecision::Assign {
                        specialist: junior,
                        intent,
                        recipe_hint: Some(recipe.id),
                    }
                } else {
                    DispatchDecision::NoSpecialist { intent, domain }
                }
            }

            // System-modifying actions go to junior first
            TranslatorDecision::NeedsConfirmation { intent, .. } => {
                let domain = Domain::from_subject(&intent.subject);
                if let Some(junior) = get_junior(domain) {
                    DispatchDecision::Assign {
                        specialist: junior,
                        intent,
                        recipe_hint: None,
                    }
                } else {
                    DispatchDecision::NoSpecialist { intent, domain }
                }
            }

            // Unclear intents may need senior
            TranslatorDecision::NeedsClarification { intent, .. }
            | TranslatorDecision::CannotHandle { intent, .. } => {
                let domain = Domain::from_subject(&intent.subject);
                Self::route_unclear_intent(intent, domain)
            }
        }
    }

    /// Request escalation from junior to senior.
    ///
    /// Returns None if escalation should be refused:
    /// - Confidence is HIGH (critical invariant)
    /// - Requesting specialist is not a junior
    /// - No senior available in domain
    pub fn request_escalation(
        from: &'static SpecialistDefinition,
        reason: EscalationReason,
        intent: &UserIntent,
    ) -> Option<DispatchDecision> {
        // CRITICAL: Refuse escalation if confidence is HIGH
        if intent.confidence >= CONFIDENCE_HIGH {
            return None;
        }

        // Only juniors can escalate
        if from.level != SpecialistLevel::Junior {
            return None;
        }

        // Find senior in same domain
        get_senior(from.domain).map(|senior| DispatchDecision::Escalate {
            from,
            to: senior,
            reason,
            intent: intent.clone(),
        })
    }

    /// Route unclear intents - try senior first, then junior.
    fn route_unclear_intent(intent: UserIntent, domain: Domain) -> DispatchDecision {
        // Seniors handle unclear intents better
        if let Some(senior) = get_senior(domain) {
            DispatchDecision::Assign {
                specialist: senior,
                intent,
                recipe_hint: None,
            }
        } else if let Some(junior) = get_junior(domain) {
            DispatchDecision::Assign {
                specialist: junior,
                intent,
                recipe_hint: None,
            }
        } else {
            DispatchDecision::NoSpecialist { intent, domain }
        }
    }
}

impl DispatchDecision {
    /// Get the assigned specialist if any.
    pub fn specialist(&self) -> Option<&'static SpecialistDefinition> {
        match self {
            DispatchDecision::Assign { specialist, .. } => Some(specialist),
            DispatchDecision::Escalate { to, .. } => Some(to),
            _ => None,
        }
    }

    /// Get the intent.
    pub fn intent(&self) -> &UserIntent {
        match self {
            DispatchDecision::Assign { intent, .. }
            | DispatchDecision::Escalate { intent, .. }
            | DispatchDecision::NoSpecialist { intent, .. }
            | DispatchDecision::DirectExecution { intent, .. } => intent,
        }
    }

    /// Check if this is a direct execution (bypass specialist).
    pub fn is_direct_execution(&self) -> bool {
        matches!(self, DispatchDecision::DirectExecution { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::translator::intent::{IntentAction, IntentSubject};
    use anna_shared::recipe::{Recipe, RecipeSource};

    fn mock_intent(confidence: f32, subject: IntentSubject) -> UserIntent {
        UserIntent {
            action: IntentAction::Query,
            subject,
            subject_raw: "test".to_string(),
            parameters: vec![],
            original_input: "test query".to_string(),
            confidence,
            classification_method: crate::translator::intent::ClassificationMethod::PatternMatch,
        }
    }

    fn mock_recipe() -> Recipe {
        Recipe {
            id: "test-recipe".to_string(),
            name: "Test Recipe".to_string(),
            keywords: vec!["test".to_string()],
            patterns: vec!["test pattern".to_string()],
            context: Default::default(),
            commands: vec![],
            verification: None,
            source: RecipeSource::BuiltIn,
            success_count: 10,
            last_used: None,
            enabled: true,
        }
    }

    #[test]
    fn test_dispatch_high_confidence_bypass() {
        let intent = mock_intent(0.95, IntentSubject::DiskUsage);
        let decision = TranslatorDecision::ExecuteRecipe {
            recipe: mock_recipe(),
            intent: intent.clone(),
            confidence: 0.95,
        };

        let dispatch = DispatchEngine::dispatch(decision);
        assert!(matches!(dispatch, DispatchDecision::DirectExecution { .. }));
    }

    #[test]
    fn test_dispatch_medium_confidence_to_junior() {
        let intent = mock_intent(0.7, IntentSubject::DiskUsage);
        let decision = TranslatorDecision::ExecuteRecipe {
            recipe: mock_recipe(),
            intent: intent.clone(),
            confidence: 0.7,
        };

        let dispatch = DispatchEngine::dispatch(decision);
        match dispatch {
            DispatchDecision::Assign { specialist, .. } => {
                assert_eq!(specialist.domain, Domain::Storage);
                assert!(specialist.is_junior());
            }
            _ => panic!("Expected Assign to junior"),
        }
    }

    #[test]
    fn test_escalation_refused_high_confidence() {
        let intent = mock_intent(0.90, IntentSubject::NetworkStatus);
        let junior = super::super::registry::get_junior(Domain::Network).unwrap();

        let result = DispatchEngine::request_escalation(
            junior,
            EscalationReason::LowConfidence {
                junior_confidence: 0.5,
            },
            &intent,
        );

        assert!(result.is_none(), "Escalation should be refused at HIGH confidence");
    }

    #[test]
    fn test_escalation_allowed_low_confidence() {
        let intent = mock_intent(0.5, IntentSubject::NetworkStatus);
        let junior = super::super::registry::get_junior(Domain::Network).unwrap();

        let result = DispatchEngine::request_escalation(
            junior,
            EscalationReason::LowConfidence {
                junior_confidence: 0.5,
            },
            &intent,
        );

        assert!(result.is_some(), "Escalation should be allowed at low confidence");
        if let Some(DispatchDecision::Escalate { from, to, .. }) = result {
            assert!(from.is_junior());
            assert!(!to.is_junior());
        }
    }

    #[test]
    fn test_escalation_junior_only() {
        let intent = mock_intent(0.5, IntentSubject::NetworkStatus);
        let senior = super::super::registry::get_senior(Domain::Network).unwrap();

        let result = DispatchEngine::request_escalation(
            senior,
            EscalationReason::ComplexAction {
                action: "routing".to_string(),
            },
            &intent,
        );

        assert!(result.is_none(), "Seniors cannot escalate");
    }

    #[test]
    fn test_unclear_intent_routes_to_senior() {
        let intent = mock_intent(0.3, IntentSubject::NetworkStatus);
        let decision = TranslatorDecision::NeedsClarification {
            clarification: crate::translator::clarification::Clarification {
                clarification_type:
                    crate::translator::clarification::ClarificationType::IntentUnclear,
                question: "What?".to_string(),
                options: vec![],
                context: String::new(),
            },
            intent: intent.clone(),
        };

        let dispatch = DispatchEngine::dispatch(decision);
        match dispatch {
            DispatchDecision::Assign { specialist, .. } => {
                // Unclear intents go to senior
                assert!(!specialist.is_junior());
            }
            _ => panic!("Expected Assign"),
        }
    }
}
