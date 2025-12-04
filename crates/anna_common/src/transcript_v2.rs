//! Transcript Renderer v0.0.59 - Departmental IT Org Dialogue
//!
//! Realistic but professional IT department dialogue. No jokes, no emojis, no theater.
//!
//! Participants:
//! - [you]: User input (literal)
//! - [anna]: Service Desk lead, calm, structured
//! - [translator]: Intake analyst, terse
//! - [junior]: QA/reliability, skeptical, calls out missing evidence
//! - [annad]: Operator, reports facts only
//! - [networking], [storage], [boot], [audio], [graphics]: Specialists, concise

use crate::case_lifecycle::{
    CaseFileV2, CaseStatus, Department, Participant, ProposedAction,
};
use owo_colors::OwoColorize;

// ============================================================================
// Persona Style Guide
// ============================================================================

/// Get actor display name with bracket formatting
fn actor_display(participant: &Participant) -> String {
    format!("[{}]", participant.actor_name())
}

/// Style text based on actor
fn style_actor(participant: &Participant, text: &str) -> String {
    match participant {
        Participant::You => text.white().to_string(),
        Participant::Anna => text.cyan().to_string(),
        Participant::Translator => text.yellow().to_string(),
        Participant::Junior => text.magenta().to_string(),
        Participant::Annad => text.dimmed().to_string(),
        Participant::Specialist(_) => text.green().to_string(),
    }
}

// ============================================================================
// Transcript Lines
// ============================================================================

/// A rendered transcript line
#[derive(Debug, Clone)]
pub struct TranscriptLine {
    pub actor: String,
    pub content: String,
    pub styled: bool,
}

impl TranscriptLine {
    pub fn new(actor: &str, content: &str) -> Self {
        Self {
            actor: actor.to_string(),
            content: content.to_string(),
            styled: true,
        }
    }

    pub fn plain(content: &str) -> Self {
        Self {
            actor: String::new(),
            content: content.to_string(),
            styled: false,
        }
    }

    pub fn separator(label: &str) -> Self {
        Self {
            actor: String::new(),
            content: format!("----- {} -----", label),
            styled: false,
        }
    }

    pub fn render(&self) -> String {
        if self.actor.is_empty() {
            self.content.dimmed().to_string()
        } else {
            format!("{} {}", self.actor.cyan(), self.content)
        }
    }
}

// ============================================================================
// Transcript Builder
// ============================================================================

/// Builds a transcript from case events
pub struct TranscriptBuilder {
    lines: Vec<TranscriptLine>,
}

impl TranscriptBuilder {
    pub fn new() -> Self {
        Self { lines: Vec::new() }
    }

    /// Add a line from an actor
    pub fn add_line(&mut self, actor: &Participant, content: &str) {
        self.lines
            .push(TranscriptLine::new(&actor_display(actor), content));
    }

    /// Add a separator
    pub fn add_separator(&mut self, label: &str) {
        self.lines.push(TranscriptLine::separator(label));
    }

    /// Add plain text
    pub fn add_plain(&mut self, content: &str) {
        self.lines.push(TranscriptLine::plain(content));
    }

    /// Build the full transcript
    pub fn build(&self) -> String {
        self.lines.iter().map(|l| l.render()).collect::<Vec<_>>().join("\n")
    }
}

impl Default for TranscriptBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Full Case Transcript Rendering
// ============================================================================

/// Render a complete case transcript
pub fn render_case_transcript(case: &CaseFileV2) -> String {
    let mut builder = TranscriptBuilder::new();

    // Header
    builder.add_plain(&format!("=== Case {} ===", case.case_id));
    builder.add_plain("");

    // Intake phase
    builder.add_separator("intake");
    builder.add_line(&Participant::You, &case.request);

    // Triage phase
    builder.add_separator("triage");
    render_triage_phase(&mut builder, case);

    // Investigation phase (if beyond triaged)
    if case.status != CaseStatus::New && case.status != CaseStatus::Triaged {
        builder.add_separator("investigation");
        render_investigation_phase(&mut builder, case);
    }

    // Diagnosis/Plan phase
    if !case.findings.is_empty() || !case.hypotheses.is_empty() {
        builder.add_separator("diagnosis");
        render_diagnosis_phase(&mut builder, case);
    }

    // Verification phase
    if case.reliability_pct > 0 {
        builder.add_separator("verification");
        render_verification_phase(&mut builder, case);
    }

    // Response phase
    if case.final_answer.is_some() {
        builder.add_separator("response");
        render_response_phase(&mut builder, case);
    }

    // Footer with metrics
    builder.add_plain("");
    render_metrics_footer(&mut builder, case);

    builder.build()
}

/// Render triage phase
fn render_triage_phase(builder: &mut TranscriptBuilder, case: &CaseFileV2) {
    // Translator classification
    builder.add_line(
        &Participant::Translator,
        &format!(
            "Intent: {}. Targets: [{}].",
            case.intent,
            case.targets.join(", ")
        ),
    );

    // Department assignment with handoff
    if case.assigned_department != Department::ServiceDesk {
        builder.add_line(
            &Participant::Anna,
            &format!(
                "Assigning to [{}] due to targets: {}",
                case.assigned_department.actor_name(),
                case.targets.join(", ")
            ),
        );
    }

    // Alert linkage
    if !case.linked_alert_ids.is_empty() {
        builder.add_line(
            &Participant::Anna,
            &format!(
                "Linked to {} active alert(s): {}",
                case.linked_alert_ids.len(),
                case.linked_alert_ids.join(", ")
            ),
        );
    }
}

/// Render investigation phase
fn render_investigation_phase(builder: &mut TranscriptBuilder, case: &CaseFileV2) {
    // Evidence collection summary
    if case.evidence_count > 0 {
        builder.add_line(
            &Participant::Annad,
            &format!(
                "Collected {} evidence item(s): [{}]",
                case.evidence_count,
                case.evidence_ids.join(", ")
            ),
        );
    }

    // Coverage check
    if case.coverage_pct < 90 {
        builder.add_line(
            &Participant::Junior,
            &format!(
                "Coverage {}%. Below threshold. May need additional evidence.",
                case.coverage_pct
            ),
        );
    }
}

/// Render diagnosis phase
fn render_diagnosis_phase(builder: &mut TranscriptBuilder, case: &CaseFileV2) {
    let specialist = Participant::Specialist(case.assigned_department);

    // Findings
    if !case.findings.is_empty() {
        builder.add_line(&specialist, "Findings:");
        for finding in &case.findings {
            builder.add_plain(&format!("  - {}", finding));
        }
    }

    // Hypotheses
    if !case.hypotheses.is_empty() {
        builder.add_line(&specialist, "Hypotheses:");
        for (hyp, conf) in &case.hypotheses {
            builder.add_plain(&format!("  - {} ({}% confidence)", hyp, conf));
        }
    }

    // Proposed actions
    if !case.actions_proposed.is_empty() {
        builder.add_line(&specialist, "Action plan:");
        for action in &case.actions_proposed {
            let risk_label = match action.risk {
                crate::case_lifecycle::ActionRisk::ReadOnly => "read-only",
                crate::case_lifecycle::ActionRisk::Low => "low-risk",
                crate::case_lifecycle::ActionRisk::Medium => "medium-risk",
                crate::case_lifecycle::ActionRisk::High => "HIGH-RISK",
            };
            builder.add_plain(&format!(
                "  [{}] {} [{}]",
                action.id, action.description, risk_label
            ));
        }
    }
}

/// Render verification phase
fn render_verification_phase(builder: &mut TranscriptBuilder, case: &CaseFileV2) {
    let verdict = if case.reliability_pct >= 90 {
        "Solid evidence. Ship it."
    } else if case.reliability_pct >= 75 {
        "Acceptable. Ship it."
    } else {
        "Not good enough. Missing evidence."
    };

    builder.add_line(
        &Participant::Junior,
        &format!(
            "Reliability {}%. Coverage {}%. {}",
            case.reliability_pct, case.coverage_pct, verdict
        ),
    );

    // Show disagreement if low confidence
    if case.reliability_pct < 75 {
        builder.add_line(
            &Participant::Junior,
            "I can't approve this conclusion without stronger evidence.",
        );
    }
}

/// Render response phase
fn render_response_phase(builder: &mut TranscriptBuilder, case: &CaseFileV2) {
    if let Some(answer) = &case.final_answer {
        builder.add_line(&Participant::Anna, answer);
    }
}

/// Render metrics footer
fn render_metrics_footer(builder: &mut TranscriptBuilder, case: &CaseFileV2) {
    builder.add_plain(&format!(
        "Case: {} | Status: {} | Coverage: {}% | Reliability: {}%",
        case.case_id, case.status, case.coverage_pct, case.reliability_pct
    ));
}

// ============================================================================
// Structured Department Output
// ============================================================================

/// Structured output from a specialist department
#[derive(Debug, Clone)]
pub struct DepartmentOutput {
    /// Department that produced this output
    pub department: Department,
    /// Key findings (bullets)
    pub findings: Vec<String>,
    /// Evidence IDs used
    pub evidence_ids: Vec<String>,
    /// Hypotheses with confidence (labeled, evidence-tied)
    pub hypotheses: Vec<Hypothesis>,
    /// Next checks (read-only, auto-run)
    pub next_checks: Vec<String>,
    /// Action plan (mutations gated)
    pub action_plan: Vec<ProposedAction>,
}

/// A hypothesis with confidence and evidence backing
#[derive(Debug, Clone)]
pub struct Hypothesis {
    /// Hypothesis label (e.g., "H1", "H2")
    pub label: String,
    /// Description
    pub description: String,
    /// Confidence (0-100)
    pub confidence: u8,
    /// Evidence IDs that support this hypothesis
    pub supporting_evidence: Vec<String>,
}

impl DepartmentOutput {
    pub fn new(department: Department) -> Self {
        Self {
            department,
            findings: Vec::new(),
            evidence_ids: Vec::new(),
            hypotheses: Vec::new(),
            next_checks: Vec::new(),
            action_plan: Vec::new(),
        }
    }

    /// Render as structured transcript lines
    pub fn render(&self, builder: &mut TranscriptBuilder) {
        let specialist = Participant::Specialist(self.department);

        // Findings
        if !self.findings.is_empty() {
            builder.add_line(&specialist, "Findings:");
            for finding in &self.findings {
                builder.add_plain(&format!("  - {}", finding));
            }
        }

        // Evidence
        if !self.evidence_ids.is_empty() {
            builder.add_line(
                &specialist,
                &format!("Evidence: [{}]", self.evidence_ids.join(", ")),
            );
        }

        // Hypotheses
        if !self.hypotheses.is_empty() {
            builder.add_line(&specialist, "Hypotheses:");
            for hyp in &self.hypotheses {
                builder.add_plain(&format!(
                    "  [{}] {} ({}% confidence, backed by [{}])",
                    hyp.label,
                    hyp.description,
                    hyp.confidence,
                    hyp.supporting_evidence.join(", ")
                ));
            }
        }

        // Next checks
        if !self.next_checks.is_empty() {
            builder.add_line(&specialist, "Next checks (auto-run):");
            for check in &self.next_checks {
                builder.add_plain(&format!("  - {}", check));
            }
        }

        // Action plan
        if !self.action_plan.is_empty() {
            builder.add_line(&specialist, "Action plan:");
            for action in &self.action_plan {
                let risk_label = match action.risk {
                    crate::case_lifecycle::ActionRisk::ReadOnly => "read-only",
                    crate::case_lifecycle::ActionRisk::Low => "low-risk",
                    crate::case_lifecycle::ActionRisk::Medium => "medium-risk",
                    crate::case_lifecycle::ActionRisk::High => "HIGH-RISK",
                };
                builder.add_plain(&format!(
                    "  [{}] {} [{}] (evidence: [{}])",
                    action.id,
                    action.description,
                    risk_label,
                    action.evidence_ids.join(", ")
                ));
            }
        }
    }
}

// ============================================================================
// Handoff Lines
// ============================================================================

/// Generate a handoff line when routing to a department
pub fn render_handoff(
    from: &Participant,
    to_department: Department,
    reason: &str,
) -> TranscriptLine {
    TranscriptLine::new(
        &actor_display(from),
        &format!(
            "Assigning to [{}]: {}",
            to_department.actor_name(),
            reason
        ),
    )
}

/// Generate a disagreement line from Junior
pub fn render_junior_disagreement(missing: &[String]) -> TranscriptLine {
    if missing.is_empty() {
        TranscriptLine::new(
            &actor_display(&Participant::Junior),
            "Evidence is insufficient. Cannot approve.",
        )
    } else {
        TranscriptLine::new(
            &actor_display(&Participant::Junior),
            &format!(
                "I can't approve this conclusion. Missing: {} evidence.",
                missing.join("/")
            ),
        )
    }
}

/// Generate a collaboration line when multiple doctors consult
pub fn render_collaboration(primary: Department, secondary: Department, topic: &str) -> Vec<TranscriptLine> {
    vec![
        TranscriptLine::new(
            &actor_display(&Participant::Specialist(primary)),
            &format!("Consulting [{}] on {}.", secondary.actor_name(), topic),
        ),
        TranscriptLine::new(
            &actor_display(&Participant::Specialist(secondary)),
            "Acknowledged. Reviewing relevant evidence.",
        ),
    ]
}

// ============================================================================
// Status Line for annactl status
// ============================================================================

/// Render active cases count for status display
pub fn render_active_cases_status() -> String {
    let count = crate::case_lifecycle::count_active_cases();
    if count == 0 {
        "No active cases".to_string()
    } else {
        format!("{} active case(s)", count)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transcript_builder() {
        let mut builder = TranscriptBuilder::new();
        builder.add_line(&Participant::You, "test request");
        builder.add_separator("triage");
        builder.add_line(&Participant::Anna, "acknowledged");

        let result = builder.build();
        assert!(result.contains("[you]"));
        assert!(result.contains("[anna]"));
        assert!(result.contains("triage"));
    }

    #[test]
    fn test_department_output_render() {
        let mut output = DepartmentOutput::new(Department::Networking);
        output.findings.push("DNS server not responding".to_string());
        output.evidence_ids.push("E1".to_string());
        output.hypotheses.push(Hypothesis {
            label: "H1".to_string(),
            description: "DNS misconfiguration".to_string(),
            confidence: 75,
            supporting_evidence: vec!["E1".to_string()],
        });

        let mut builder = TranscriptBuilder::new();
        output.render(&mut builder);

        let result = builder.build();
        assert!(result.contains("Findings"));
        assert!(result.contains("DNS server not responding"));
        assert!(result.contains("H1"));
        assert!(result.contains("75%"));
    }

    #[test]
    fn test_handoff_line() {
        let line = render_handoff(
            &Participant::Anna,
            Department::Networking,
            "network-related keywords detected",
        );
        assert!(line.content.contains("[networking]"));
    }

    #[test]
    fn test_junior_disagreement() {
        let line = render_junior_disagreement(&["dns".to_string(), "route".to_string()]);
        assert!(line.content.contains("Missing"));
        assert!(line.content.contains("dns/route"));
    }
}
