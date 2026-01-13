//! Integration Layer - Wires all learning subsystems into the decision loop.
//!
//! This module enforces:
//! 1. Retrieval precedence (live probes → docs → skills → semantic → episodic)
//! 2. Provenance tracking for all knowledge
//! 3. Validation gatekeeping for skill promotion
//! 4. "I don't know" as first-class control state
//! 5. Active falsification
//! 6. Budget/stopping policies

pub mod budget;
pub mod falsification;
pub mod provenance;
pub mod retrieval;
pub mod skill_gate;
pub mod uncertainty;

pub use budget::*;
pub use falsification::*;
pub use provenance::*;
pub use retrieval::*;
pub use skill_gate::*;
pub use uncertainty::*;

use serde::{Deserialize, Serialize};

/// The integrated decision engine
#[derive(Debug, Clone, Default)]
pub struct DecisionEngine {
    /// Knowledge retrieval system with precedence
    pub retrieval: KnowledgeRetrieval,
    /// Skill validation gatekeeper
    pub skill_gate: SkillGatekeeper,
    /// Uncertainty detector
    pub uncertainty: UncertaintyDetector,
    /// Falsification engine
    pub falsification: FalsificationEngine,
    /// Budget controller
    pub budget: BudgetController,
}

impl DecisionEngine {
    /// Create a new decision engine
    pub fn new() -> Self {
        Self::default()
    }

    /// Process a question through the full decision loop
    pub fn process_question(&mut self, question: &str, mode: OperationMode) -> DecisionResult {
        // 1. Check budget
        let budget = self.budget.get_budget(mode);
        if !budget.can_proceed() {
            return DecisionResult {
                action: DecisionAction::BudgetExceeded,
                knowledge: Vec::new(),
                confidence: 0.0,
                uncertainty_state: Some(UncertaintyState::new(
                    "Budget exceeded before processing",
                    1.0,
                )),
                provenance: Vec::new(),
            };
        }

        // 2. Retrieve knowledge with precedence
        let knowledge = self.retrieval.retrieve(question);

        // 3. Check uncertainty
        let uncertainty_state = self.uncertainty.assess(&knowledge, question);

        // 4. If uncertain, switch to investigator mode
        if let Some(ref state) = uncertainty_state {
            if state.should_investigate() {
                return DecisionResult {
                    action: DecisionAction::Investigate(state.generate_experiment_plan()),
                    knowledge,
                    confidence: state.confidence,
                    uncertainty_state: Some(state.clone()),
                    provenance: self.retrieval.get_provenance(),
                };
            }
        }

        // 5. Calculate overall confidence
        let confidence = self.calculate_confidence(&knowledge, &uncertainty_state);

        // 6. Determine action
        let action = if confidence > 0.7 {
            DecisionAction::Answer
        } else if confidence > 0.4 {
            DecisionAction::AnswerWithCaution
        } else {
            DecisionAction::Investigate(
                uncertainty_state
                    .as_ref()
                    .map(|s| s.generate_experiment_plan())
                    .unwrap_or_default(),
            )
        };

        DecisionResult {
            action,
            knowledge,
            confidence,
            uncertainty_state,
            provenance: self.retrieval.get_provenance(),
        }
    }

    /// Calculate overall confidence from knowledge sources
    fn calculate_confidence(
        &self,
        knowledge: &[RetrievedKnowledge],
        uncertainty: &Option<UncertaintyState>,
    ) -> f32 {
        if knowledge.is_empty() {
            return 0.0;
        }

        // Base confidence from knowledge sources
        let mut confidence: f32 = knowledge
            .iter()
            .map(|k| k.confidence * k.source.reliability_weight())
            .sum::<f32>()
            / knowledge.len() as f32;

        // Penalty for uncertainty signals
        if let Some(state) = uncertainty {
            confidence *= 1.0 - (state.novelty_score * 0.3);
            if state.has_conflicts {
                confidence *= 0.7;
            }
        }

        confidence.clamp(0.0, 1.0)
    }
}

/// Result of decision processing
#[derive(Debug, Clone)]
pub struct DecisionResult {
    /// Recommended action
    pub action: DecisionAction,
    /// Retrieved knowledge
    pub knowledge: Vec<RetrievedKnowledge>,
    /// Overall confidence
    pub confidence: f32,
    /// Uncertainty state if detected
    pub uncertainty_state: Option<UncertaintyState>,
    /// Provenance for all knowledge
    pub provenance: Vec<ProvenanceRecord>,
}

/// Actions the decision engine can recommend
#[derive(Debug, Clone)]
pub enum DecisionAction {
    /// Confident enough to answer
    Answer,
    /// Can answer but with reduced confidence
    AnswerWithCaution,
    /// Need to investigate - includes experiment plan
    Investigate(Vec<ExperimentStep>),
    /// Budget exceeded
    BudgetExceeded,
    /// Explicitly don't know
    DontKnow(String),
}

/// Operation modes with different budgets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum OperationMode {
    /// Fast problem solving - tight budget
    #[default]
    Solver,
    /// Investigation mode - medium budget
    Investigator,
    /// Lab/experiment mode - high budget
    Lab,
}

impl OperationMode {
    pub fn name(&self) -> &'static str {
        match self {
            OperationMode::Solver => "solver",
            OperationMode::Investigator => "investigator",
            OperationMode::Lab => "lab",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decision_engine_creation() {
        let engine = DecisionEngine::new();
        assert!(engine.budget.get_budget(OperationMode::Solver).can_proceed());
    }

    #[test]
    fn test_mode_names() {
        assert_eq!(OperationMode::Solver.name(), "solver");
        assert_eq!(OperationMode::Investigator.name(), "investigator");
        assert_eq!(OperationMode::Lab.name(), "lab");
    }
}
