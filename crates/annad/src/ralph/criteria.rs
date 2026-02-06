//! Completion criteria and iteration state for the Ralph loop.
//! Phase 22: Integrates IntentClass for iteration limits.
//! Phase 24: Uses policy dials for iteration limits based on track record.

use anna_shared::intent_class::{classify_intent, IntentClass};
use anna_shared::policy::get_policy;

/// Completion criteria for a question
#[derive(Debug, Clone)]
pub struct CompletionCriteria {
    /// What type of answer is expected
    pub answer_type: AnswerType,
    /// Minimum confidence threshold (0.0 - 1.0)
    pub min_confidence: f32,
    /// Maximum iterations before giving up
    pub max_iterations: u32,
    /// Whether grounding in command output is required
    pub requires_grounding: bool,
    /// Phase 22: Intent classification for answer contract
    pub intent_class: IntentClass,
}

impl Default for CompletionCriteria {
    fn default() -> Self {
        Self {
            answer_type: AnswerType::Factual,
            min_confidence: 0.7,
            max_iterations: 5,
            requires_grounding: true,
            intent_class: IntentClass::ReadOnly,
        }
    }
}

/// Types of answers Anna can provide
#[derive(Debug, Clone)]
pub enum AnswerType {
    /// Factual information from the system (requires command output)
    Factual,
    /// How-to instructions (may cite wiki/docs)
    HowTo,
    /// Troubleshooting help (requires diagnosis)
    Troubleshoot,
    /// Simple acknowledgment or clarification
    Simple,
}

/// State of an iteration attempt
#[derive(Debug, Default)]
pub struct IterationState {
    /// Commands executed so far
    pub commands: Vec<String>,
    /// Outputs collected
    pub outputs: Vec<String>,
    /// Current answer draft
    pub answer: Option<String>,
    /// Confidence in current answer
    pub confidence: f32,
    /// Feedback from previous iteration
    pub feedback: Option<String>,
    /// Why we're not done yet
    pub not_done_reason: Option<String>,
}

/// Result of self-evaluation
#[derive(Debug)]
pub struct SelfEvaluation {
    /// Is the answer complete?
    pub is_complete: bool,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// What's missing if not complete
    pub missing: Option<String>,
    /// Suggestions for improvement
    pub suggestions: Option<String>,
}

/// Determine completion criteria based on the question
pub fn determine_criteria(question: &str) -> CompletionCriteria {
    let q = question.to_lowercase();

    // Phase 22: Classify intent first
    let intent_class = classify_intent(question);

    // Phase 24: Get iteration limits from policy (based on track record)
    let policy = get_policy();
    let readonly_max_iter = policy.readonly_max_iterations;
    let mutating_max_iter = policy.mutating_max_iterations;

    // HowTo questions - instructions, don't need live output
    if q.contains("how do i")
        || q.contains("how to")
        || q.contains("how can i")
        || q.starts_with("install")
        || q.starts_with("setup")
        || q.starts_with("configure")
    {
        return CompletionCriteria {
            answer_type: AnswerType::HowTo,
            min_confidence: 0.6,
            max_iterations: readonly_max_iter,
            requires_grounding: false, // Instructions don't need live data
            intent_class,
        };
    }

    // Troubleshooting - needs diagnosis
    if q.contains("not working")
        || q.contains("error")
        || q.contains("failed")
        || q.contains("problem")
        || q.contains("broken")
        || q.contains("fix")
        || q.contains("why")
    {
        // Phase 24: Use policy-driven limits
        let max_iter = match intent_class {
            IntentClass::ReadOnly => readonly_max_iter,
            IntentClass::Mutating => mutating_max_iter,
        };
        return CompletionCriteria {
            answer_type: AnswerType::Troubleshoot,
            min_confidence: 0.7,
            max_iterations: max_iter,
            requires_grounding: true,
            intent_class,
        };
    }

    // Simple questions
    if q.len() < 30 && !q.contains("?") {
        return CompletionCriteria {
            answer_type: AnswerType::Simple,
            min_confidence: 0.5,
            max_iterations: 2.min(readonly_max_iter), // Never exceed policy limit
            requires_grounding: false,
            intent_class,
        };
    }

    // Default: Factual query
    // Phase 24: Use policy-driven limits
    let max_iter = match intent_class {
        IntentClass::ReadOnly => readonly_max_iter,
        IntentClass::Mutating => mutating_max_iter,
    };
    CompletionCriteria {
        answer_type: AnswerType::Factual,
        min_confidence: 0.7,
        max_iterations: max_iter,
        requires_grounding: true,
        intent_class,
    }
}

/// Quick quality check for answers (no LLM needed)
pub fn quick_quality_check(answer: &str) -> bool {
    let answer = answer.trim();

    // Too short
    if answer.len() < 10 {
        return false;
    }

    // Obvious refusals
    let refusals = ["i cannot", "i can't", "i'm not able", "i don't know"];
    if refusals.iter().any(|r| answer.to_lowercase().contains(r)) {
        return false;
    }

    // Prompt leakage
    let leakage = ["as an ai", "as a language model", "i'm an ai"];
    if leakage.iter().any(|l| answer.to_lowercase().contains(l)) {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_criteria_factual() {
        let criteria = determine_criteria("what is my kernel version?");
        assert!(matches!(criteria.answer_type, AnswerType::Factual));
        assert!(criteria.requires_grounding);
    }

    #[test]
    fn test_determine_criteria_howto() {
        let criteria = determine_criteria("how do I install neovim?");
        assert!(matches!(criteria.answer_type, AnswerType::HowTo));
        assert!(!criteria.requires_grounding);
    }

    #[test]
    fn test_determine_criteria_troubleshoot() {
        let criteria = determine_criteria("wifi not working after update");
        assert!(matches!(criteria.answer_type, AnswerType::Troubleshoot));
        assert!(criteria.requires_grounding);
    }

    #[test]
    fn test_quick_quality_check_good() {
        assert!(quick_quality_check("Your kernel version is 6.7.0"));
    }

    #[test]
    fn test_quick_quality_check_too_short() {
        assert!(!quick_quality_check("ok"));
    }

    #[test]
    fn test_quick_quality_check_refusal() {
        assert!(!quick_quality_check("I cannot answer that question"));
    }

    // Phase 22: Intent classification and iteration limits

    #[test]
    fn test_readonly_intent_caps_iterations() {
        // READ_ONLY questions should have max 3 iterations
        let criteria = determine_criteria("what is my disk usage?");
        assert!(matches!(criteria.intent_class, IntentClass::ReadOnly));
        assert!(criteria.max_iterations <= 3);
    }

    #[test]
    fn test_mutating_intent_allows_more_iterations() {
        // MUTATING questions can have 3-6 iterations depending on success rate
        let criteria = determine_criteria("fix this bluetooth problem");
        // "fix" triggers MUTATING
        assert!(matches!(criteria.intent_class, IntentClass::Mutating));
        // Policy-driven: 3 (poor), 5 (standard), 6 (excellent)
        assert!(criteria.max_iterations >= 3 && criteria.max_iterations <= 6);
    }

    #[test]
    fn test_diagnostic_readonly_caps_iterations() {
        // Diagnostic READ_ONLY should also cap at 3
        let criteria = determine_criteria("check swap usage and swappiness");
        assert!(matches!(criteria.intent_class, IntentClass::ReadOnly));
        assert!(criteria.max_iterations <= 3);
    }
}
