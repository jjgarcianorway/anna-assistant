//! Protocol v0.19.0 - Subproblem Decomposition
//!
//! Key changes from v0.18.0:
//! - Junior decomposes questions into subproblems
//! - Fact-aware planning (check fact store first)
//! - Senior as mentor (refines subproblem list)
//! - Small concrete JSON messages

use serde::{Deserialize, Serialize};

/// A subproblem that Junior identifies to answer the main question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subproblem {
    /// Unique ID for this subproblem
    pub id: String,
    /// Human-readable description
    pub description: String,
    /// Required probes to answer this subproblem
    pub required_probes: Vec<String>,
    /// Facts from store that might help (if any)
    pub relevant_facts: Vec<String>,
    /// Status of this subproblem
    pub status: SubproblemStatus,
    /// Evidence collected for this subproblem
    pub evidence: Vec<String>,
    /// Partial answer for this subproblem (if solved)
    pub partial_answer: Option<String>,
}

/// Status of a subproblem
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubproblemStatus {
    /// Not yet started
    Pending,
    /// Currently gathering evidence
    InProgress,
    /// Solved with partial answer
    Solved,
    /// Cannot be solved with available probes
    Blocked,
}

impl Default for SubproblemStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// Junior's decomposition of the question into subproblems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JuniorDecomposition {
    /// The subproblems identified
    pub subproblems: Vec<Subproblem>,
    /// Facts from store that are relevant to the whole question
    pub known_facts: Vec<KnownFact>,
    /// Whether decomposition is complete
    pub decomposition_complete: bool,
    /// Reasoning for the decomposition
    pub reasoning: String,
}

/// A fact from the knowledge store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnownFact {
    /// Fact key (e.g., "cpu.cores")
    pub key: String,
    /// Fact value
    pub value: String,
    /// Trust level (0.0 - 1.0)
    pub trust: f32,
    /// When the fact was last verified
    pub last_verified: Option<String>,
}

/// Junior's action for v0.19.0 - now includes subproblem context
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum JuniorStepV19 {
    /// Decompose the question into subproblems
    #[serde(rename = "decompose")]
    Decompose {
        decomposition: JuniorDecomposition,
    },

    /// Work on a specific subproblem by running a probe
    #[serde(rename = "work_subproblem")]
    WorkSubproblem {
        subproblem_id: String,
        probe_id: String,
        reason: String,
    },

    /// Mark a subproblem as solved
    #[serde(rename = "solve_subproblem")]
    SolveSubproblem {
        subproblem_id: String,
        partial_answer: String,
        confidence: u8,
    },

    /// Synthesize final answer from solved subproblems
    #[serde(rename = "synthesize")]
    Synthesize {
        text: String,
        subproblem_summaries: Vec<SubproblemSummary>,
        scores: JuniorScoresV19,
    },

    /// Ask Senior for mentoring on decomposition or approach
    #[serde(rename = "ask_mentor")]
    AskMentor {
        question: String,
        current_state: MentorContext,
    },
}

/// Summary of a solved subproblem for synthesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubproblemSummary {
    pub id: String,
    pub description: String,
    pub answer: String,
    pub probes_used: Vec<String>,
}

/// Context provided to Senior for mentoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MentorContext {
    pub original_question: String,
    pub current_subproblems: Vec<Subproblem>,
    pub solved_count: usize,
    pub blocked_count: usize,
    pub specific_issue: String,
}

/// Junior's self-assessment scores for v0.19.0
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JuniorScoresV19 {
    /// How complete is the evidence coverage (0-100)
    pub evidence_coverage: u8,
    /// How confident in the reasoning (0-100)
    pub reasoning_confidence: u8,
    /// How well subproblems were solved (0-100)
    pub subproblem_coverage: u8,
    /// Overall score (min of above)
    pub overall: u8,
}

impl Default for JuniorScoresV19 {
    fn default() -> Self {
        Self {
            evidence_coverage: 0,
            reasoning_confidence: 0,
            subproblem_coverage: 0,
            overall: 0,
        }
    }
}

/// Senior's mentoring response for v0.19.0
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "response")]
pub enum SeniorMentor {
    /// Approve the current approach
    #[serde(rename = "approve_approach")]
    ApproveApproach {
        feedback: String,
    },

    /// Suggest refinements to the subproblem list
    #[serde(rename = "refine_subproblems")]
    RefineSubproblems {
        feedback: String,
        suggested_additions: Vec<SuggestedSubproblem>,
        suggested_removals: Vec<String>,
        suggested_merges: Vec<SubproblemMerge>,
    },

    /// Suggest a different approach entirely
    #[serde(rename = "suggest_approach")]
    SuggestApproach {
        feedback: String,
        new_approach: String,
        key_subproblems: Vec<SuggestedSubproblem>,
    },

    /// Approve the final answer
    #[serde(rename = "approve_answer")]
    ApproveAnswer {
        scores: SeniorScoresV19,
    },

    /// Correct the final answer
    #[serde(rename = "correct_answer")]
    CorrectAnswer {
        corrected_text: String,
        corrections: Vec<String>,
        scores: SeniorScoresV19,
    },
}

/// A subproblem suggested by Senior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedSubproblem {
    pub description: String,
    pub suggested_probes: Vec<String>,
    pub reason: String,
}

/// A merge suggestion for subproblems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubproblemMerge {
    pub merge_ids: Vec<String>,
    pub merged_description: String,
    pub reason: String,
}

/// Senior's scores for v0.19.0
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeniorScoresV19 {
    /// Evidence grounding (0-100)
    pub evidence: u8,
    /// Reasoning quality (0-100)
    pub reasoning: u8,
    /// Answer completeness (0-100)
    pub completeness: u8,
    /// Overall reliability (min of above)
    pub overall: u8,
    /// Human-readable reliability note
    pub reliability_note: String,
}

impl Default for SeniorScoresV19 {
    fn default() -> Self {
        Self {
            evidence: 0,
            reasoning: 0,
            completeness: 0,
            overall: 0,
            reliability_note: String::new(),
        }
    }
}

/// State for v0.19.0 question processing
#[derive(Debug, Clone, Default)]
pub struct QuestionStateV19 {
    /// Original question
    pub question: String,
    /// Current iteration
    pub iteration: usize,
    /// Decomposition from Junior
    pub decomposition: Option<JuniorDecomposition>,
    /// Current subproblems with their states
    pub subproblems: Vec<Subproblem>,
    /// All probes executed
    pub probes_executed: Vec<ProbeResultV19>,
    /// Whether Senior has been consulted
    pub senior_consulted: bool,
    /// Final answer if complete
    pub final_answer: Option<FinalAnswerV19>,
}

/// Probe result for v0.19.0
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResultV19 {
    pub probe_id: String,
    pub subproblem_id: Option<String>,
    pub output: String,
    pub success: bool,
}

/// Final answer for v0.19.0
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalAnswerV19 {
    pub text: String,
    pub reliability: u8,
    pub reliability_note: String,
    pub subproblems_solved: usize,
    pub subproblems_total: usize,
    pub probes_used: Vec<String>,
    pub facts_used: Vec<String>,
    pub iterations: usize,
    pub senior_mentored: bool,
}

/// Constants for v0.19.0
pub const MAX_ITERATIONS_V19: usize = 8;
pub const MAX_SUBPROBLEMS: usize = 5;
pub const MIN_CONFIDENCE_FOR_SYNTHESIS: u8 = 70;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subproblem_status_default() {
        let status = SubproblemStatus::default();
        assert_eq!(status, SubproblemStatus::Pending);
    }

    #[test]
    fn test_junior_scores_default() {
        let scores = JuniorScoresV19::default();
        assert_eq!(scores.overall, 0);
    }

    #[test]
    fn test_serialize_junior_step() {
        let step = JuniorStepV19::WorkSubproblem {
            subproblem_id: "sp1".to_string(),
            probe_id: "cpu.info".to_string(),
            reason: "Need CPU info".to_string(),
        };
        let json = serde_json::to_string(&step).unwrap();
        assert!(json.contains("work_subproblem"));
        assert!(json.contains("cpu.info"));
    }
}
