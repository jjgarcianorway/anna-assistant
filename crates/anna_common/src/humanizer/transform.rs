//! Message Transformation for Humanizer v0.0.73
//!
//! Transforms internal events into natural human-readable messages.
//! Uses role-appropriate phrasing per department (v0.0.73).
//!
//! Can introduce slight disagreement/clarifying exchanges ONLY when real
//! uncertainty exists (translator confidence low, tool returned no evidence).
//!
//! NEVER fabricates evidence or actions.

use super::labels::{EvidenceSummary, HumanLabel};
use super::phrases;
use super::roles::{ConfidenceHint, DepartmentTag, MessageTone, StaffRole};

/// A humanized message ready for transcript
#[derive(Debug, Clone)]
pub struct HumanizedMessage {
    /// Role tag for display (e.g., "service desk", "network")
    pub tag: String,
    /// The humanized message text
    pub text: String,
    /// Tone used
    pub tone: MessageTone,
    /// Whether this is a side-thread message
    pub is_side_thread: bool,
}

impl HumanizedMessage {
    pub fn new(tag: &str, text: &str) -> Self {
        Self {
            tag: tag.to_string(),
            text: text.to_string(),
            tone: MessageTone::Neutral,
            is_side_thread: false,
        }
    }

    pub fn with_tone(mut self, tone: MessageTone) -> Self {
        self.tone = tone;
        self
    }

    pub fn as_side_thread(mut self) -> Self {
        self.is_side_thread = true;
        self
    }

    /// Format for display
    pub fn format(&self) -> String {
        if self.is_side_thread {
            format!("  [{}] {}", self.tag, self.text)
        } else {
            format!("[{}] {}", self.tag, self.text)
        }
    }
}

/// Humanizer context for transformation
#[derive(Debug, Clone, Default)]
pub struct HumanizerContext {
    /// Current confidence level (affects tone)
    pub confidence: u8,
    /// Whether evidence is missing
    pub evidence_missing: bool,
    /// Whether this is a complex/multi-step case
    pub is_complex: bool,
    /// Active department
    pub active_department: Option<DepartmentTag>,
    /// Parse warnings occurred
    pub has_parse_warnings: bool,
    /// Retries occurred
    pub has_retries: bool,
}

impl HumanizerContext {
    pub fn tone(&self) -> MessageTone {
        if self.evidence_missing {
            MessageTone::Skeptical
        } else if self.has_parse_warnings || self.has_retries {
            MessageTone::Cautious
        } else {
            MessageTone::from_confidence(self.confidence)
        }
    }

    pub fn confidence_hint(&self) -> ConfidenceHint {
        ConfidenceHint::from_score(self.confidence)
    }
}

/// Humanize service desk opening message (v0.0.73: role-appropriate phrasing)
pub fn humanize_case_open(_request: &str, _ctx: &HumanizerContext) -> HumanizedMessage {
    let text = phrases::phrase_case_open(None);
    HumanizedMessage::new("service desk", text)
}

/// Humanize triage/routing decision
pub fn humanize_triage(
    primary: DepartmentTag,
    supporting: &[DepartmentTag],
    ctx: &HumanizerContext,
) -> HumanizedMessage {
    let text = if supporting.is_empty() {
        format!("I'll have {} look into this.", primary.display_name())
    } else {
        let support_names: Vec<_> = supporting.iter().map(|d| d.display_name()).collect();
        format!(
            "I'll have {} look into this, with help from {}.",
            primary.display_name(),
            support_names.join(" and ")
        )
    };
    HumanizedMessage::new("service desk", &text).with_tone(ctx.tone())
}

/// Humanize evidence gathering message (v0.0.73: topic + source, no IDs)
pub fn humanize_evidence_gather(
    dept: DepartmentTag,
    label: &HumanLabel,
    summary: &str,
    ctx: &HumanizerContext,
) -> HumanizedMessage {
    // v0.0.73: Show topic with source context, no evidence IDs
    let source = match label {
        HumanLabel::HardwareInventory => "hardware snapshot",
        HumanLabel::SoftwareServices => "service snapshot",
        HumanLabel::ServiceStatus => "service snapshot",
        HumanLabel::NetworkSignals => "network snapshot",
        HumanLabel::StorageStatus => "storage snapshot",
        HumanLabel::AudioStack => "audio snapshot",
        HumanLabel::GraphicsStatus => "graphics snapshot",
        HumanLabel::BootTimeline => "boot timeline",
        HumanLabel::ErrorJournal => "error journal",
        HumanLabel::SystemLoad => "hardware snapshot",
        _ => "latest snapshot",
    };
    let text = format!("{} (from {}): {}", label.short_name(), source, summary);
    HumanizedMessage::new(dept.tag(), &text)
        .with_tone(ctx.tone())
        .as_side_thread()
}

/// Humanize doctor/department selection in Human mode (v0.0.73)
/// Shows which department took ownership and what they will check first.
pub fn humanize_doctor_selection(dept: DepartmentTag) -> Vec<HumanizedMessage> {
    let mut messages = Vec::new();

    // Line 1: Who is taking the case
    let ownership = phrases::phrase_taking_ownership(dept);
    messages.push(HumanizedMessage::new(dept.tag(), &ownership));

    // Line 2: What they will check first
    let first_check = phrases::phrase_first_check(dept);
    messages.push(
        HumanizedMessage::new(dept.tag(), first_check)
            .with_tone(MessageTone::Neutral)
            .as_side_thread(),
    );

    messages
}

/// Humanize department finding (v0.0.73: confidence-based prefix)
pub fn humanize_finding(
    dept: DepartmentTag,
    finding: &str,
    ctx: &HumanizerContext,
) -> HumanizedMessage {
    let tone = ctx.tone();
    // v0.0.73: Use confidence-based prefix instead of tone prefix
    let prefix = phrases::phrase_finding_prefix(ctx.confidence);
    let text = format!("{}{}", prefix, finding);
    HumanizedMessage::new(dept.tag(), &text).with_tone(tone)
}

/// Humanize "no evidence" situation - ALLOWED realism (v0.0.73: role-specific phrasing)
pub fn humanize_missing_evidence(
    dept: DepartmentTag,
    topic: &str,
    _ctx: &HumanizerContext,
) -> HumanizedMessage {
    // This is allowed because it reflects real uncertainty
    // v0.0.73: Use role-specific phrasing
    let text = phrases::phrase_missing_evidence(dept, topic);
    HumanizedMessage::new(dept.tag(), &text)
        .with_tone(MessageTone::Skeptical)
        .as_side_thread()
}

/// Humanize service desk caution message - ALLOWED realism
pub fn humanize_caution(reason: &str) -> HumanizedMessage {
    let text = format!("Let's keep it read-only for now and {}.", reason);
    HumanizedMessage::new("service desk", &text).with_tone(MessageTone::Cautious)
}

/// Humanize junior critique - ALLOWED realism
pub fn humanize_junior_critique(missing_topic: &str, _ctx: &HumanizerContext) -> HumanizedMessage {
    // Junior speaks like a reviewer - short, specific
    let text = format!(
        "Your answer is okay, but you didn't ground it in the {}.",
        missing_topic
    );
    // Junior is internal, but critique is shown in debug mode
    HumanizedMessage::new("junior", &text).with_tone(MessageTone::Skeptical)
}

/// Humanize final answer from service desk
pub fn humanize_final_answer(answer: &str, reliability: u8, reason: &str) -> Vec<HumanizedMessage> {
    let mut messages = Vec::new();

    // Main answer
    messages.push(HumanizedMessage::new("service desk", answer));

    // Reliability footer
    let reliability_text = format_reliability(reliability, reason);
    messages.push(HumanizedMessage::new("", &reliability_text));

    messages
}

fn format_reliability(score: u8, reason: &str) -> String {
    let desc = if score >= 80 {
        "good evidence coverage"
    } else if score >= 60 {
        "some evidence gaps"
    } else {
        "limited evidence"
    };
    format!("Reliability: {}% ({}, {})", score, desc, reason)
}

/// Generate micro-thread for evidence gathering (human mode only)
pub fn generate_micro_thread(
    dept: DepartmentTag,
    evidence_summaries: &[(HumanLabel, String)],
    ctx: &HumanizerContext,
) -> Vec<HumanizedMessage> {
    let mut messages = Vec::new();

    if evidence_summaries.is_empty() {
        messages.push(humanize_missing_evidence(dept, "status", ctx));
        return messages;
    }

    for (label, summary) in evidence_summaries {
        messages.push(humanize_evidence_gather(dept, label, summary, ctx));
    }

    messages
}

/// Check if this is a semantically correct answer for the query type
pub fn validate_answer_relevance(query: &str, answer: &str) -> AnswerValidation {
    let query_lower = query.to_lowercase();
    let answer_lower = answer.to_lowercase();

    // Memory query should not get CPU answer
    if (query_lower.contains("memory") || query_lower.contains("ram"))
        && !answer_lower.contains("memory")
        && !answer_lower.contains("ram")
        && !answer_lower.contains("gib")
        && !answer_lower.contains("gb")
    {
        return AnswerValidation::WrongTopic {
            expected: "memory",
            got_hint: if answer_lower.contains("cpu") {
                "cpu"
            } else {
                "other"
            },
        };
    }

    // Disk query should not get CPU answer
    if (query_lower.contains("disk")
        || query_lower.contains("space")
        || query_lower.contains("storage"))
        && !answer_lower.contains("disk")
        && !answer_lower.contains("storage")
        && !answer_lower.contains("free")
        && !answer_lower.contains("mount")
    {
        return AnswerValidation::WrongTopic {
            expected: "disk/storage",
            got_hint: if answer_lower.contains("cpu") {
                "cpu"
            } else {
                "other"
            },
        };
    }

    // Systemd query should have specific answer
    if query_lower.contains("systemd") && query_lower.contains("running") {
        if !answer_lower.contains("systemd")
            && !answer_lower.contains("running")
            && !answer_lower.contains("pid 1")
        {
            return AnswerValidation::TooGeneric {
                expected: "systemd running status",
            };
        }
    }

    AnswerValidation::Ok
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnswerValidation {
    Ok,
    WrongTopic {
        expected: &'static str,
        got_hint: &'static str,
    },
    TooGeneric {
        expected: &'static str,
    },
    MissingEvidence,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_humanize_case_open() {
        let ctx = HumanizerContext::default();
        let msg = humanize_case_open("what is my cpu", &ctx);
        assert_eq!(msg.tag, "service desk");
        // v0.0.73: Uses role-appropriate phrasing
        assert!(msg.text.contains("triaging"));
    }

    #[test]
    fn test_humanize_doctor_selection() {
        let messages = humanize_doctor_selection(DepartmentTag::Network);
        assert_eq!(messages.len(), 2);
        // First message: ownership
        assert!(messages[0].text.contains("Network team"));
        assert!(messages[0].text.contains("case"));
        // Second message: first check (side thread)
        assert!(messages[1].is_side_thread);
        assert!(
            messages[1].text.contains("interface") || messages[1].text.contains("connectivity")
        );
    }

    #[test]
    fn test_humanize_evidence_gather_v073() {
        let ctx = HumanizerContext::default();
        let msg = humanize_evidence_gather(
            DepartmentTag::Performance,
            &HumanLabel::HardwareInventory,
            "Intel i9-14900HX",
            &ctx,
        );
        // v0.0.73: Should show topic with source context
        assert!(msg.text.contains("hardware inventory"));
        assert!(msg.text.contains("hardware snapshot"));
        assert!(msg.text.contains("Intel i9-14900HX"));
        // Should NOT have evidence IDs
        assert!(!msg.text.contains("[E"));
    }

    #[test]
    fn test_humanize_triage_single() {
        let ctx = HumanizerContext {
            confidence: 80,
            ..Default::default()
        };
        let msg = humanize_triage(DepartmentTag::Performance, &[], &ctx);
        assert!(msg.text.contains("performance"));
        assert!(!msg.text.contains("help from"));
    }

    #[test]
    fn test_humanize_triage_multi() {
        let ctx = HumanizerContext::default();
        let msg = humanize_triage(DepartmentTag::Network, &[DepartmentTag::Audio], &ctx);
        assert!(msg.text.contains("network"));
        assert!(msg.text.contains("audio"));
    }

    #[test]
    fn test_humanize_missing_evidence() {
        let ctx = HumanizerContext {
            evidence_missing: true,
            ..Default::default()
        };
        let msg = humanize_missing_evidence(DepartmentTag::Network, "link state", &ctx);
        assert!(msg.text.contains("can't confirm"));
        assert!(msg.is_side_thread);
    }

    #[test]
    fn test_validate_answer_memory_cpu_mismatch() {
        let result = validate_answer_relevance(
            "how much memory do I have",
            "You have an Intel i9-14900HX CPU",
        );
        assert!(matches!(
            result,
            AnswerValidation::WrongTopic {
                expected: "memory",
                ..
            }
        ));
    }

    #[test]
    fn test_validate_answer_disk_cpu_mismatch() {
        let result = validate_answer_relevance(
            "how much disk space is free",
            "Your CPU is Intel i9-14900HX",
        );
        assert!(matches!(
            result,
            AnswerValidation::WrongTopic {
                expected: "disk/storage",
                ..
            }
        ));
    }

    #[test]
    fn test_validate_answer_ok() {
        let result = validate_answer_relevance(
            "how much memory do I have",
            "You have 32 GiB of RAM, 45% used",
        );
        assert_eq!(result, AnswerValidation::Ok);
    }
}
