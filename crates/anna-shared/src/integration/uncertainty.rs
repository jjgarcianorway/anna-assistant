//! Uncertainty Detection - "I don't know" as first-class control state.
//!
//! Trigger conditions are NUMERIC, not vibes:
//! - Novelty score above threshold
//! - Low evidence confidence
//! - Conflicting signals
//!
//! When triggered:
//! - Investigator mode generates EXPERIMENTS, not guesses
//! - Experiments have bounded cost and clear stopping conditions
//! - This state is emitted even if the LLM tries to bluff

use super::retrieval::RetrievedKnowledge;
use serde::{Deserialize, Serialize};

/// Uncertainty detection thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncertaintyThresholds {
    /// Novelty score above this triggers investigation
    pub novelty_threshold: f32,
    /// Confidence below this triggers investigation
    pub confidence_threshold: f32,
    /// Conflict ratio above this triggers investigation
    pub conflict_threshold: f32,
    /// Minimum knowledge items needed
    pub min_knowledge_items: usize,
}

impl Default for UncertaintyThresholds {
    fn default() -> Self {
        Self {
            novelty_threshold: 0.7,
            confidence_threshold: 0.4,
            conflict_threshold: 0.3,
            min_knowledge_items: 1,
        }
    }
}

/// The uncertainty state - a first-class control state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncertaintyState {
    /// Why we're uncertain
    pub reason: String,
    /// Novelty score (0.0 = familiar, 1.0 = completely novel)
    pub novelty_score: f32,
    /// Evidence confidence (0.0-1.0)
    pub evidence_confidence: f32,
    /// Are there conflicting signals?
    pub has_conflicts: bool,
    /// Conflict details
    pub conflicts: Vec<ConflictDetail>,
    /// Overall confidence after analysis
    pub confidence: f32,
    /// Recommended action
    pub recommendation: UncertaintyAction,
}

/// Detail about a conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictDetail {
    /// Source A
    pub source_a: String,
    /// Source B
    pub source_b: String,
    /// What conflicts
    pub description: String,
}

/// What to do when uncertain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UncertaintyAction {
    /// Can proceed with caution
    ProceedWithCaution,
    /// Need to investigate
    Investigate,
    /// Must say "I don't know"
    AdmitUncertainty,
}

impl UncertaintyState {
    /// Create new uncertainty state
    pub fn new(reason: &str, novelty: f32) -> Self {
        Self {
            reason: reason.to_string(),
            novelty_score: novelty,
            evidence_confidence: 0.0,
            has_conflicts: false,
            conflicts: Vec::new(),
            confidence: 0.0,
            recommendation: UncertaintyAction::AdmitUncertainty,
        }
    }

    /// Should we switch to investigator mode?
    pub fn should_investigate(&self) -> bool {
        matches!(self.recommendation, UncertaintyAction::Investigate)
    }

    /// Generate an experiment plan to resolve uncertainty
    pub fn generate_experiment_plan(&self) -> Vec<ExperimentStep> {
        let mut steps = Vec::new();

        // If novel, gather more information
        if self.novelty_score > 0.5 {
            steps.push(ExperimentStep {
                description: "Search for similar patterns in documentation".to_string(),
                command: None,
                expected_outcome: "Find relevant documentation".to_string(),
                cost_estimate: 1,
                stop_condition: "Found matching documentation or exhausted search".to_string(),
            });
        }

        // If low confidence, run diagnostic probes
        if self.evidence_confidence < 0.5 {
            steps.push(ExperimentStep {
                description: "Run diagnostic probes to gather system state".to_string(),
                command: None, // Commands determined by context
                expected_outcome: "Gather current system state".to_string(),
                cost_estimate: 3,
                stop_condition: "Sufficient state gathered or budget exceeded".to_string(),
            });
        }

        // If conflicts, resolve them
        if self.has_conflicts {
            steps.push(ExperimentStep {
                description: "Run targeted probes to resolve conflicting information".to_string(),
                command: None,
                expected_outcome: "Determine which information is current".to_string(),
                cost_estimate: 2,
                stop_condition: "Conflict resolved or determined irresolvable".to_string(),
            });
        }

        // Always have a stop condition
        if steps.is_empty() {
            steps.push(ExperimentStep {
                description: "Unable to determine experiment plan".to_string(),
                command: None,
                expected_outcome: "Admit uncertainty".to_string(),
                cost_estimate: 0,
                stop_condition: "Immediately".to_string(),
            });
        }

        steps
    }
}

/// A step in an experiment plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentStep {
    /// What this step does
    pub description: String,
    /// Specific command (if known)
    pub command: Option<String>,
    /// What we expect to learn
    pub expected_outcome: String,
    /// Cost estimate (1-10)
    pub cost_estimate: u32,
    /// When to stop
    pub stop_condition: String,
}

/// The uncertainty detector
#[derive(Debug, Clone, Default)]
pub struct UncertaintyDetector {
    /// Detection thresholds
    pub thresholds: UncertaintyThresholds,
    /// Known patterns (for novelty detection)
    known_patterns: Vec<String>,
}

impl UncertaintyDetector {
    /// Create new detector
    pub fn new() -> Self {
        Self::default()
    }

    /// With custom thresholds
    pub fn with_thresholds(thresholds: UncertaintyThresholds) -> Self {
        Self {
            thresholds,
            known_patterns: Vec::new(),
        }
    }

    /// Assess uncertainty for retrieved knowledge
    pub fn assess(&self, knowledge: &[RetrievedKnowledge], question: &str) -> Option<UncertaintyState> {
        // 1. Calculate novelty score
        let novelty = self.calculate_novelty(question);

        // 2. Calculate evidence confidence
        let evidence_confidence = self.calculate_evidence_confidence(knowledge);

        // 3. Detect conflicts
        let (has_conflicts, conflicts) = self.detect_conflicts(knowledge);

        // 4. Check if any threshold is breached
        let novelty_breach = novelty > self.thresholds.novelty_threshold;
        let confidence_breach = evidence_confidence < self.thresholds.confidence_threshold;
        let conflict_breach = has_conflicts
            && (conflicts.len() as f32 / knowledge.len().max(1) as f32)
                > self.thresholds.conflict_threshold;
        let insufficient_knowledge = knowledge.len() < self.thresholds.min_knowledge_items;

        if !novelty_breach && !confidence_breach && !conflict_breach && !insufficient_knowledge {
            return None; // No uncertainty detected
        }

        // Build reason string
        let mut reasons = Vec::new();
        if novelty_breach {
            reasons.push(format!("High novelty ({:.0}%)", novelty * 100.0));
        }
        if confidence_breach {
            reasons.push(format!(
                "Low evidence confidence ({:.0}%)",
                evidence_confidence * 100.0
            ));
        }
        if conflict_breach {
            reasons.push(format!("{} conflicting signals", conflicts.len()));
        }
        if insufficient_knowledge {
            reasons.push("Insufficient knowledge".to_string());
        }

        // Determine recommendation
        let recommendation = if novelty > 0.9 || evidence_confidence < 0.2 {
            UncertaintyAction::AdmitUncertainty
        } else if novelty > 0.7 || evidence_confidence < 0.4 || has_conflicts {
            UncertaintyAction::Investigate
        } else {
            UncertaintyAction::ProceedWithCaution
        };

        // Calculate overall confidence
        let confidence = evidence_confidence * (1.0 - novelty * 0.5)
            * if has_conflicts { 0.7 } else { 1.0 };

        Some(UncertaintyState {
            reason: reasons.join("; "),
            novelty_score: novelty,
            evidence_confidence,
            has_conflicts,
            conflicts,
            confidence,
            recommendation,
        })
    }

    /// Calculate novelty score (how unfamiliar is this question?)
    fn calculate_novelty(&self, question: &str) -> f32 {
        if self.known_patterns.is_empty() {
            return 0.8; // Default to somewhat novel
        }

        let q_lower = question.to_lowercase();
        let words: Vec<&str> = q_lower.split_whitespace().collect();

        // Check overlap with known patterns
        let mut max_overlap: f32 = 0.0;
        for pattern in &self.known_patterns {
            let pattern_words: Vec<&str> = pattern.split_whitespace().collect();
            let matching = words
                .iter()
                .filter(|w| pattern_words.contains(w))
                .count();
            let overlap =
                matching as f32 / words.len().max(pattern_words.len()).max(1) as f32;
            max_overlap = max_overlap.max(overlap);
        }

        // Novelty is inverse of familiarity
        1.0 - max_overlap
    }

    /// Calculate evidence confidence from knowledge
    fn calculate_evidence_confidence(&self, knowledge: &[RetrievedKnowledge]) -> f32 {
        if knowledge.is_empty() {
            return 0.0;
        }

        // Weight by source reliability
        let total_weighted: f32 = knowledge
            .iter()
            .map(|k| k.confidence * k.source.reliability_weight())
            .sum();

        let total_weight: f32 = knowledge
            .iter()
            .map(|k| k.source.reliability_weight())
            .sum();

        if total_weight == 0.0 {
            return 0.0;
        }

        total_weighted / total_weight
    }

    /// Detect conflicts between knowledge items
    fn detect_conflicts(&self, knowledge: &[RetrievedKnowledge]) -> (bool, Vec<ConflictDetail>) {
        let mut conflicts = Vec::new();

        // Simple conflict detection: look for contradictory statements
        for (i, k1) in knowledge.iter().enumerate() {
            for k2 in knowledge.iter().skip(i + 1) {
                if let Some(conflict) = self.check_conflict(k1, k2) {
                    conflicts.push(conflict);
                }
            }
        }

        (!conflicts.is_empty(), conflicts)
    }

    /// Check if two knowledge items conflict
    fn check_conflict(&self, k1: &RetrievedKnowledge, k2: &RetrievedKnowledge) -> Option<ConflictDetail> {
        // Check for obvious contradictions
        let c1 = k1.content.to_lowercase();
        let c2 = k2.content.to_lowercase();

        // Simple patterns that indicate conflict
        let conflict_pairs = [
            ("is installed", "is not installed"),
            ("is running", "is stopped"),
            ("is enabled", "is disabled"),
            ("exists", "does not exist"),
            ("succeeded", "failed"),
        ];

        for (pos, neg) in &conflict_pairs {
            if (c1.contains(pos) && c2.contains(neg)) || (c1.contains(neg) && c2.contains(pos)) {
                return Some(ConflictDetail {
                    source_a: format!("{:?}", k1.source),
                    source_b: format!("{:?}", k2.source),
                    description: format!("Conflicting state: {} vs {}", pos, neg),
                });
            }
        }

        None
    }

    /// Add a known pattern (for novelty detection)
    pub fn add_known_pattern(&mut self, pattern: &str) {
        self.known_patterns.push(pattern.to_lowercase());
    }
}

/// Acceptance test: when presented with an unknown error signature,
/// Anna must switch to investigator mode and produce an experiment plan
/// instead of a confident answer
#[cfg(test)]
mod tests {
    use super::*;
    use crate::integration::provenance::{ProvenanceRecord, ProvenanceSource};
    use crate::integration::retrieval::KnowledgeSource;

    fn make_knowledge(content: &str, source: KnowledgeSource, confidence: f32) -> RetrievedKnowledge {
        RetrievedKnowledge {
            content: content.to_string(),
            source,
            confidence,
            provenance: ProvenanceRecord::new(
                ProvenanceSource::LiveProbe {
                    command: "test".to_string(),
                    timestamp: "now".to_string(),
                },
                confidence,
            ),
            stale: false,
            age_secs: None,
            citation: None,
        }
    }

    #[test]
    fn test_high_novelty_triggers_uncertainty() {
        let detector = UncertaintyDetector::new();

        // Empty knowledge = unknown
        let result = detector.assess(&[], "some completely novel question");
        assert!(result.is_some());

        let state = result.unwrap();
        assert!(state.novelty_score > 0.5);
    }

    #[test]
    fn test_low_confidence_triggers_investigation() {
        let detector = UncertaintyDetector::new();

        let knowledge = vec![make_knowledge(
            "maybe something",
            KnowledgeSource::EpisodicMemory, // Low reliability
            0.3,                              // Low confidence
        )];

        let result = detector.assess(&knowledge, "test question");
        assert!(result.is_some());

        let state = result.unwrap();
        assert!(state.evidence_confidence < 0.5);
    }

    #[test]
    fn test_conflicts_detected() {
        let detector = UncertaintyDetector::new();

        let knowledge = vec![
            make_knowledge("service is running", KnowledgeSource::LiveProbe, 0.9),
            make_knowledge("service is stopped", KnowledgeSource::EpisodicMemory, 0.8),
        ];

        let result = detector.assess(&knowledge, "test question");
        assert!(result.is_some());

        let state = result.unwrap();
        assert!(state.has_conflicts);
    }

    #[test]
    fn test_experiment_plan_generated() {
        let state = UncertaintyState {
            reason: "High novelty".to_string(),
            novelty_score: 0.9,
            evidence_confidence: 0.3,
            has_conflicts: false,
            conflicts: Vec::new(),
            confidence: 0.2,
            recommendation: UncertaintyAction::Investigate,
        };

        let plan = state.generate_experiment_plan();
        assert!(!plan.is_empty());

        // Each step should have a stop condition
        for step in &plan {
            assert!(!step.stop_condition.is_empty());
        }
    }

    #[test]
    fn test_investigate_mode_triggered() {
        let detector = UncertaintyDetector::new();

        // Unknown question with no knowledge
        let result = detector.assess(&[], "completely unknown error xyz123");
        assert!(result.is_some());

        let state = result.unwrap();
        // Should recommend investigation, not a confident answer
        assert!(matches!(
            state.recommendation,
            UncertaintyAction::Investigate | UncertaintyAction::AdmitUncertainty
        ));
    }
}
