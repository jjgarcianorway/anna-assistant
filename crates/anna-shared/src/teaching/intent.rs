//! Teaching Mode Intent Classification
//!
//! v0.3.71: Teaching Mode specification implementation.
//!
//! Hard constraints remain:
//! - No new execution capabilities
//! - No unsolicited actions
//! - No guessing
//! - No hallucinated system state
//! - No shell commands unless explicitly allowed by user opt-in
//!
//! Teaching Mode adds explanation capabilities WITHOUT loosening safety.

use regex::Regex;
use std::sync::LazyLock;

/// Teaching intent classification - 5 distinct intents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TeachingIntent {
    /// Status: Answer from current snapshot.
    /// - Direct data retrieval
    /// - No interpretation beyond facts
    #[default]
    Status,

    /// Change Analysis: Answer from diffs and history.
    /// - What changed, when, evidence
    /// - No speculation about causes
    ChangeAnalysis,

    /// Explanation: Explain what something is and why it matters.
    /// - May explain concepts, reasoning, tradeoffs
    /// - Must NOT give shell commands by default
    /// - Must NOT instruct user what to type
    Explanation,

    /// Service Desk Handling: Explain how an experienced Linux admin would reason.
    /// - Diagnostic reasoning steps
    /// - Escalation logic
    /// - What evidence would be requested and why
    ServiceDesk,

    /// Action Request: Must follow existing confirmation and capability rules.
    /// - Routes to existing Mutating flow
    /// - No changes to action handling
    ActionRequest,
}

impl TeachingIntent {
    /// Whether this intent allows explanation/teaching output.
    pub fn allows_teaching(&self) -> bool {
        matches!(self, TeachingIntent::Explanation | TeachingIntent::ServiceDesk)
    }

    /// Whether this intent requires data grounding (evidence).
    pub fn requires_evidence(&self) -> bool {
        matches!(
            self,
            TeachingIntent::Status | TeachingIntent::ChangeAnalysis | TeachingIntent::ServiceDesk
        )
    }

    /// Whether commands can appear in the response (never by default).
    pub fn allows_commands(&self) -> bool {
        false // Never show commands by default - Teaching Mode rule
    }

    /// Whether this intent routes to the action/mutation flow.
    pub fn is_action_request(&self) -> bool {
        matches!(self, TeachingIntent::ActionRequest)
    }
}

// Status patterns - asking about current state
static STATUS_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new(r"(?i)^(show|display|list|check|report)\s+(me\s+)?(the\s+)?(current|my)?\s*(status|state|usage|info)").unwrap(),
        Regex::new(r"(?i)^what('?s| is| are)\s+(my|the|current)\s").unwrap(),
        Regex::new(r"(?i)^how (much|many)\s").unwrap(),
        Regex::new(r"(?i)\b(status|uptime|version|running|active)\b.*\?$").unwrap(),
        Regex::new(r"(?i)^is\s+(my|the|this)\s+\w+\s+(running|active|enabled|working)").unwrap(),
    ]
});

// Change analysis patterns - asking about diffs/history
static CHANGE_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new(r"(?i)\b(what|which)\s+(changed|was modified|was updated|is different)").unwrap(),
        Regex::new(r"(?i)\b(when|why) did .* (change|modify|update)").unwrap(),
        Regex::new(r"(?i)\b(diff|difference|compare|history|log|timeline)\b").unwrap(),
        Regex::new(r"(?i)\b(before|after|since|between)\b.*\b(change|update|modify)").unwrap(),
        Regex::new(r"(?i)\bwhat (happened|occurred)\b").unwrap(),
    ]
});

// Explanation patterns - asking for understanding
static EXPLANATION_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new(r"(?i)^what (is|are|does)\s+\w+\s*\??$").unwrap(), // "what is X?"
        Regex::new(r"(?i)^(explain|describe|tell me about)\s").unwrap(),
        Regex::new(r"(?i)\b(what does .* mean|what is .* for|purpose of)\b").unwrap(),
        Regex::new(r"(?i)\b(why is .* important|why does .* matter)\b").unwrap(),
        Regex::new(r"(?i)^how does\s+\w+\s+work").unwrap(),
        Regex::new(r"(?i)\b(concept|understand|meaning|definition)\b").unwrap(),
    ]
});

// Service desk patterns - asking for diagnostic reasoning
static SERVICEDESK_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new(r"(?i)\b(how would|what would)\s+(you|an admin|a sysadmin)\s+(diagnose|troubleshoot|investigate|approach)").unwrap(),
        Regex::new(r"(?i)\b(why might|what could cause|what are possible causes)\b").unwrap(),
        Regex::new(r"(?i)\b(diagnose|troubleshoot|debug|investigate)\s+(this|the|my)\b").unwrap(),
        Regex::new(r"(?i)\b(next steps|what should i check|where do i look)\b").unwrap(),
        Regex::new(r"(?i)\b(escalat|triage|prioriti)\b").unwrap(),
        Regex::new(r"(?i)\bwhat (evidence|data|info) (would|should|do) (you|i|we) need\b").unwrap(),
    ]
});

// Action request patterns - explicit change requests
static ACTION_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new(r"(?i)\b(install|uninstall|remove|upgrade|update)\s+\w").unwrap(),
        Regex::new(r"(?i)\b(enable|disable|start|stop|restart|mask)\s+(the\s+)?\w").unwrap(),
        Regex::new(r"(?i)\b(change|set|configure|modify|edit|create|delete)\s").unwrap(),
        Regex::new(r"(?i)\b(fix|repair|resolve)\s+(this|the|it|my)\b").unwrap(),
        Regex::new(r"(?i)\b(do it|apply|proceed|make the change|go ahead)\b").unwrap(),
        Regex::new(r"(?i)\b(prevent|ensure|guarantee)\b").unwrap(),
    ]
});

/// Classify a user question into a TeachingIntent.
pub fn classify_teaching_intent(question: &str) -> TeachingIntent {
    let q = question.trim();

    // Check ACTION first (highest priority for safety)
    for pattern in ACTION_PATTERNS.iter() {
        if pattern.is_match(q) {
            return TeachingIntent::ActionRequest;
        }
    }

    // Check SERVICE_DESK (diagnostic reasoning)
    for pattern in SERVICEDESK_PATTERNS.iter() {
        if pattern.is_match(q) {
            return TeachingIntent::ServiceDesk;
        }
    }

    // Check EXPLANATION (conceptual understanding)
    for pattern in EXPLANATION_PATTERNS.iter() {
        if pattern.is_match(q) {
            return TeachingIntent::Explanation;
        }
    }

    // Check CHANGE_ANALYSIS (diffs/history)
    for pattern in CHANGE_PATTERNS.iter() {
        if pattern.is_match(q) {
            return TeachingIntent::ChangeAnalysis;
        }
    }

    // Check STATUS (current state)
    for pattern in STATUS_PATTERNS.iter() {
        if pattern.is_match(q) {
            return TeachingIntent::Status;
        }
    }

    // Default: Status for questions (safe, fact-based)
    TeachingIntent::Status
}

/// Teaching output rules for explanation intents.
pub struct TeachingResponse {
    /// The structured explanation (why before how)
    pub explanation: String,
    /// System state evidence used (if any)
    pub evidence_used: Vec<String>,
    /// What evidence is missing (if any)
    pub evidence_missing: Vec<String>,
    /// Whether this came from cached data
    pub from_cache: bool,
}

impl TeachingResponse {
    /// Create a new teaching response.
    pub fn new(explanation: String) -> Self {
        Self {
            explanation,
            evidence_used: Vec::new(),
            evidence_missing: Vec::new(),
            from_cache: false,
        }
    }

    /// Add evidence that was used in the explanation.
    pub fn with_evidence(mut self, evidence: Vec<String>) -> Self {
        self.evidence_used = evidence;
        self
    }

    /// Add missing evidence that would help.
    pub fn with_missing(mut self, missing: Vec<String>) -> Self {
        self.evidence_missing = missing;
        self
    }

    /// Mark as coming from cache.
    pub fn cached(mut self) -> Self {
        self.from_cache = true;
        self
    }

    /// Format the teaching response for output.
    pub fn format(&self) -> String {
        let mut output = self.explanation.clone();

        if !self.evidence_used.is_empty() {
            output.push_str("\n\n[Evidence used: ");
            output.push_str(&self.evidence_used.join(", "));
            output.push(']');
        }

        if !self.evidence_missing.is_empty() {
            output.push_str("\n\n[Evidence a service desk would request: ");
            output.push_str(&self.evidence_missing.join(", "));
            output.push(']');
        }

        output
    }
}

/// Format a service desk reasoning response.
/// Explains how an experienced Linux admin would reason about the issue.
pub fn format_servicedesk_reasoning(
    issue_summary: &str,
    reasoning_steps: &[&str],
    evidence_needed: &[&str],
) -> String {
    let mut output = String::new();

    output.push_str("SERVICE DESK REASONING\n");
    output.push_str("----------------------\n\n");
    output.push_str(&format!("Issue: {}\n\n", issue_summary));

    output.push_str("How an experienced admin would approach this:\n\n");
    for (i, step) in reasoning_steps.iter().enumerate() {
        output.push_str(&format!("{}. {}\n", i + 1, step));
    }

    if !evidence_needed.is_empty() {
        output.push_str("\nEvidence needed for diagnosis:\n");
        for evidence in evidence_needed {
            output.push_str(&format!("- {}\n", evidence));
        }
    }

    output.push_str("\n[Teaching Mode: No commands provided by default]\n");

    output
}

/// Format an explanation response.
/// Explains why before how, grounded in evidence.
pub fn format_explanation(
    concept: &str,
    why_it_matters: &str,
    system_context: Option<&str>,
) -> String {
    let mut output = String::new();

    output.push_str("EXPLANATION\n");
    output.push_str("-----------\n\n");

    // Why it matters (first)
    output.push_str(&format!("Why this matters: {}\n\n", why_it_matters));

    // What it is
    output.push_str(&format!("What is {}: ", concept));
    // Note: actual explanation would come from LLM, this is structure only

    // Tie to system state if available
    if let Some(context) = system_context {
        output.push_str(&format!("\n\nOn your system: {}\n", context));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_intent() {
        let status_questions = [
            "what is my disk usage?",
            "show me the current status",
            "how much RAM do I have?",
            "is my bluetooth running?",
        ];

        for q in status_questions {
            assert_eq!(
                classify_teaching_intent(q),
                TeachingIntent::Status,
                "Expected Status for: {}",
                q
            );
        }
    }

    #[test]
    fn test_change_analysis_intent() {
        let change_questions = [
            "what changed in /etc/group?",
            "when did the config change?",
            "show me the diff",
            "what happened since yesterday?",
        ];

        for q in change_questions {
            assert_eq!(
                classify_teaching_intent(q),
                TeachingIntent::ChangeAnalysis,
                "Expected ChangeAnalysis for: {}",
                q
            );
        }
    }

    #[test]
    fn test_explanation_intent() {
        let explanation_questions = [
            "what is systemd?",
            "explain pipewire",
            "what does swappiness mean?",
            "how does btrfs work?",
        ];

        for q in explanation_questions {
            assert_eq!(
                classify_teaching_intent(q),
                TeachingIntent::Explanation,
                "Expected Explanation for: {}",
                q
            );
        }
    }

    #[test]
    fn test_servicedesk_intent() {
        let servicedesk_questions = [
            "how would an admin diagnose this?",
            "what could cause the fan to spin?",
            "troubleshoot my network issue",
            "what evidence do I need?",
        ];

        for q in servicedesk_questions {
            assert_eq!(
                classify_teaching_intent(q),
                TeachingIntent::ServiceDesk,
                "Expected ServiceDesk for: {}",
                q
            );
        }
    }

    #[test]
    fn test_action_request_intent() {
        let action_questions = [
            "install neovim",
            "disable bluetooth",
            "fix this error",
            "do it",
        ];

        for q in action_questions {
            assert_eq!(
                classify_teaching_intent(q),
                TeachingIntent::ActionRequest,
                "Expected ActionRequest for: {}",
                q
            );
        }
    }

    #[test]
    fn test_intent_properties() {
        assert!(!TeachingIntent::Status.allows_teaching());
        assert!(TeachingIntent::Explanation.allows_teaching());
        assert!(TeachingIntent::ServiceDesk.allows_teaching());
        assert!(!TeachingIntent::ActionRequest.allows_teaching());

        assert!(!TeachingIntent::Status.allows_commands());
        assert!(!TeachingIntent::Explanation.allows_commands());
        assert!(!TeachingIntent::ServiceDesk.allows_commands());
        assert!(!TeachingIntent::ActionRequest.allows_commands());
    }
}
