//! Completion criteria and iteration state for the Ralph loop.

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
}

impl Default for CompletionCriteria {
    fn default() -> Self {
        Self {
            answer_type: AnswerType::Factual,
            min_confidence: 0.7,
            max_iterations: 5,
            requires_grounding: true,
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
            max_iterations: 3,
            requires_grounding: false, // Instructions don't need live data
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
        return CompletionCriteria {
            answer_type: AnswerType::Troubleshoot,
            min_confidence: 0.7,
            max_iterations: 5,
            requires_grounding: true,
        };
    }

    // Simple questions
    if q.len() < 30 && !q.contains("?") {
        return CompletionCriteria {
            answer_type: AnswerType::Simple,
            min_confidence: 0.5,
            max_iterations: 2,
            requires_grounding: false,
        };
    }

    // Default: Factual query
    CompletionCriteria {
        answer_type: AnswerType::Factual,
        min_confidence: 0.7,
        max_iterations: 5,
        requires_grounding: true,
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
}
