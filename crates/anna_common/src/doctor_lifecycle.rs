//! Doctor Lifecycle v0.0.64 - First-Class Doctor Flows
//!
//! v0.0.64: Doctors are first-class flows with explicit lifecycle stages:
//! 1. Intake: What symptoms? What we're checking now
//! 2. EvidenceGathering: Target specific evidence topics
//! 3. Diagnosis: Ranked hypotheses
//! 4. Plan: Read-only suggestions or mutation plan with risk tier
//! 5. Verify: Confirm evidence coherence (read-only) or post-check (actions)
//! 6. HandOff: If needs another doctor
//!
//! This module provides:
//! - Doctor trait with lifecycle methods
//! - DoctorLifecycleStage enum for tracking progress
//! - DoctorRunner for orchestrating diagnosis flows
//! - Structured output formats for findings and reports
//! - Integration with knowledge learning system

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::doctor_registry::{
    DoctorDomain, DoctorRun, DoctorRunResult, DoctorRunStage, DoctorSelection, FindingSeverity,
    KeyFinding, PlaybookRunResult, StageStatus, StageTiming, VerificationStatus,
};
use crate::evidence_topic::EvidenceTopic;
use crate::service_desk::Ticket;

// =============================================================================
// v0.0.64: Doctor Lifecycle Stages
// =============================================================================

/// Lifecycle stage for doctor flow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DoctorLifecycleStage {
    /// Initial intake - understanding symptoms
    Intake,
    /// Gathering evidence from tools
    EvidenceGathering,
    /// Running diagnosis on evidence
    Diagnosis,
    /// Creating action plan
    Planning,
    /// Verifying results
    Verification,
    /// Handing off to another doctor
    HandOff,
    /// Completed
    Complete,
}

impl DoctorLifecycleStage {
    pub fn as_str(&self) -> &'static str {
        match self {
            DoctorLifecycleStage::Intake => "intake",
            DoctorLifecycleStage::EvidenceGathering => "evidence_gathering",
            DoctorLifecycleStage::Diagnosis => "diagnosis",
            DoctorLifecycleStage::Planning => "planning",
            DoctorLifecycleStage::Verification => "verification",
            DoctorLifecycleStage::HandOff => "hand_off",
            DoctorLifecycleStage::Complete => "complete",
        }
    }

    pub fn human_label(&self) -> &'static str {
        match self {
            DoctorLifecycleStage::Intake => "Understanding symptoms",
            DoctorLifecycleStage::EvidenceGathering => "Gathering evidence",
            DoctorLifecycleStage::Diagnosis => "Analyzing findings",
            DoctorLifecycleStage::Planning => "Creating plan",
            DoctorLifecycleStage::Verification => "Verifying results",
            DoctorLifecycleStage::HandOff => "Consulting specialist",
            DoctorLifecycleStage::Complete => "Complete",
        }
    }
}

impl std::fmt::Display for DoctorLifecycleStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.human_label())
    }
}

/// Intake result from doctor - what symptoms they identified
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntakeResult {
    /// Symptoms identified from request
    pub symptoms: Vec<String>,
    /// What the doctor will check
    pub checking: Vec<String>,
    /// Human-readable intake summary
    pub summary: String,
    /// Evidence topics to gather
    pub evidence_topics: Vec<EvidenceTopic>,
}

/// Lifecycle state tracking for a doctor flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorLifecycleState {
    /// Current stage
    pub current_stage: DoctorLifecycleStage,
    /// Stages completed
    pub completed_stages: Vec<DoctorLifecycleStage>,
    /// Stage timings (milliseconds)
    pub stage_timings: HashMap<DoctorLifecycleStage, u64>,
    /// Intake result (if completed)
    pub intake: Option<IntakeResult>,
    /// Whether hand-off to another doctor is needed
    pub needs_handoff: bool,
    /// Doctor to hand off to (if any)
    pub handoff_doctor: Option<String>,
}

impl DoctorLifecycleState {
    pub fn new() -> Self {
        Self {
            current_stage: DoctorLifecycleStage::Intake,
            completed_stages: Vec::new(),
            stage_timings: HashMap::new(),
            intake: None,
            needs_handoff: false,
            handoff_doctor: None,
        }
    }

    pub fn advance_to(&mut self, stage: DoctorLifecycleStage, duration_ms: u64) {
        self.completed_stages.push(self.current_stage);
        self.stage_timings.insert(self.current_stage, duration_ms);
        self.current_stage = stage;
    }

    pub fn complete(&mut self, duration_ms: u64) {
        self.advance_to(DoctorLifecycleStage::Complete, duration_ms);
    }
}

impl Default for DoctorLifecycleState {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Doctor Trait - The Lifecycle Contract
// =============================================================================

/// Check in a doctor's diagnostic plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticCheck {
    /// Check ID (e.g., "physical_link", "default_route")
    pub id: String,
    /// Human-readable description
    pub description: String,
    /// Tool to invoke for this check
    pub tool_name: String,
    /// Parameters for the tool
    pub tool_params: HashMap<String, serde_json::Value>,
    /// Is this check required or optional?
    pub required: bool,
    /// Order in the plan (lower = earlier)
    pub order: u32,
}

/// Evidence collected from a diagnostic check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectedEvidence {
    /// Evidence ID (e.g., "N1", "N2")
    pub evidence_id: String,
    /// Check that produced this evidence
    pub check_id: String,
    /// Tool that was invoked
    pub tool_name: String,
    /// Raw data returned
    pub data: serde_json::Value,
    /// Human-readable summary
    pub summary: String,
    /// Collection timestamp
    pub collected_at: DateTime<Utc>,
    /// Success flag
    pub success: bool,
}

/// A finding from diagnosis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisFinding {
    /// Finding ID
    pub id: String,
    /// What was found
    pub description: String,
    /// Severity level
    pub severity: FindingSeverity,
    /// Evidence IDs that support this finding
    pub evidence_ids: Vec<String>,
    /// Confidence in this finding (0-100)
    pub confidence: u32,
    /// Tags for knowledge search
    pub tags: Vec<String>,
}

/// A proposed action (not executed automatically)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposedAction {
    /// Action ID
    pub id: String,
    /// Description of what will be done
    pub description: String,
    /// Commands that would be executed
    pub commands: Vec<String>,
    /// Risk level
    pub risk: ActionRisk,
    /// Requires confirmation phrase
    pub confirmation_required: bool,
    /// Confirmation phrase needed
    pub confirmation_phrase: Option<String>,
    /// Evidence IDs justifying this action
    pub evidence_ids: Vec<String>,
    /// Rollback commands if action fails
    pub rollback: Option<Vec<String>>,
}

/// Risk level for actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionRisk {
    /// Read-only or very safe
    Low,
    /// Service restart, config change
    Medium,
    /// Destructive or hard to undo
    High,
}

impl std::fmt::Display for ActionRisk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionRisk::Low => write!(f, "Low"),
            ActionRisk::Medium => write!(f, "Medium"),
            ActionRisk::High => write!(f, "High"),
        }
    }
}

/// Safe next step (read-only suggestion)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafeNextStep {
    /// Step description
    pub description: String,
    /// Why this step is suggested
    pub rationale: String,
    /// Evidence supporting this suggestion
    pub evidence_ids: Vec<String>,
}

/// Result of diagnosis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisResult {
    /// Human-readable summary
    pub summary: String,
    /// Most likely cause (if determined)
    pub most_likely_cause: Option<String>,
    /// Findings from diagnosis
    pub findings: Vec<DiagnosisFinding>,
    /// Overall confidence (0-100)
    pub confidence: u32,
    /// Safe next steps (read-only)
    pub next_steps: Vec<SafeNextStep>,
    /// Proposed actions (require confirmation)
    pub proposed_actions: Vec<ProposedAction>,
    /// Is the issue resolved based on evidence?
    pub issue_resolved: bool,
    /// Keywords for learning/recipe tagging
    pub symptom_keywords: Vec<String>,
}

impl DiagnosisResult {
    /// Create a "no fault found" result
    pub fn no_fault_found(summary: &str, next_steps: Vec<SafeNextStep>) -> Self {
        Self {
            summary: summary.to_string(),
            most_likely_cause: None,
            findings: vec![],
            confidence: 50,
            next_steps,
            proposed_actions: vec![],
            issue_resolved: false,
            symptom_keywords: vec![],
        }
    }
}

/// Doctor report - the final structured output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorReport {
    /// Report ID
    pub report_id: String,
    /// Doctor that generated this report
    pub doctor_id: String,
    /// Doctor name
    pub doctor_name: String,
    /// Domain
    pub domain: DoctorDomain,
    /// User's original request
    pub user_request: String,
    /// Timestamp
    pub generated_at: DateTime<Utc>,
    /// What was checked (plan summary)
    pub checks_performed: Vec<String>,
    /// Evidence collected
    pub evidence: Vec<CollectedEvidence>,
    /// Diagnosis result
    pub diagnosis: DiagnosisResult,
    /// Total duration in milliseconds
    pub duration_ms: u64,
}

impl DoctorReport {
    /// Render as human-readable text
    pub fn render(&self) -> String {
        let mut lines = Vec::new();

        // Header
        lines.push(format!("{} Report", self.doctor_name));
        lines.push("=".repeat(60));
        lines.push(String::new());

        // Summary
        lines.push(format!("Summary: {}", self.diagnosis.summary));
        lines.push(String::new());

        // What I checked
        lines.push("What I Checked:".to_string());
        for check in &self.checks_performed {
            lines.push(format!("  - {}", check));
        }
        lines.push(String::new());

        // Findings
        if !self.diagnosis.findings.is_empty() {
            lines.push("Findings:".to_string());
            for finding in &self.diagnosis.findings {
                let evidence = if finding.evidence_ids.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", finding.evidence_ids.join(", "))
                };
                lines.push(format!(
                    "  [{:?}] {}{}",
                    finding.severity, finding.description, evidence
                ));
            }
            lines.push(String::new());
        }

        // Most likely cause
        if let Some(cause) = &self.diagnosis.most_likely_cause {
            lines.push(format!(
                "Most Likely Cause: {} ({}% confidence)",
                cause, self.diagnosis.confidence
            ));
            lines.push(String::new());
        }

        // Safe next steps
        if !self.diagnosis.next_steps.is_empty() {
            lines.push("Safe Next Steps:".to_string());
            for step in &self.diagnosis.next_steps {
                lines.push(format!("  - {}", step.description));
            }
            lines.push(String::new());
        }

        // Proposed actions
        if !self.diagnosis.proposed_actions.is_empty() {
            lines.push("Actions I Can Do (requires confirmation):".to_string());
            for action in &self.diagnosis.proposed_actions {
                lines.push(format!("  [{:?} risk] {}", action.risk, action.description));
                if action.confirmation_required {
                    if let Some(phrase) = &action.confirmation_phrase {
                        lines.push(format!("    Confirm with: \"{}\"", phrase));
                    }
                }
            }
            lines.push(String::new());
        }

        // Footer
        lines.push(format!(
            "Report ID: {} | Duration: {}ms",
            self.report_id, self.duration_ms
        ));

        lines.join("\n")
    }

    /// Check if this report qualifies for recipe learning
    pub fn qualifies_for_learning(&self, reliability_score: u32) -> bool {
        reliability_score >= 90 && self.diagnosis.confidence >= 80 && self.evidence.len() >= 3
    }

    /// Get tools used in this diagnosis for recipe creation
    pub fn tools_used(&self) -> Vec<String> {
        self.evidence
            .iter()
            .map(|e| e.tool_name.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect()
    }

    /// Get targets for recipe creation
    pub fn targets(&self) -> Vec<String> {
        self.diagnosis.symptom_keywords.clone()
    }
}

// =============================================================================
// Doctor Trait
// =============================================================================

/// The Doctor trait defines the lifecycle contract for all diagnostic doctors
///
/// v0.0.64: Enhanced with lifecycle stage methods:
/// - intake(): Understand symptoms, return what we're checking
/// - evidence_plan(): Return evidence topics to collect
/// - diagnose(): Analyze evidence and produce findings
/// - plan(): Create action plan from diagnosis
/// - human_dialogue(): Get human-readable messages for stages
pub trait Doctor: Send + Sync {
    /// Stable identifier for this doctor
    fn id(&self) -> &str;

    /// Human-readable name
    fn name(&self) -> &str;

    /// Domain this doctor handles
    fn domain(&self) -> DoctorDomain;

    /// Domains this doctor can diagnose (may be multiple)
    fn domains(&self) -> Vec<&str>;

    /// Score how well this doctor matches the given request (0-100)
    ///
    /// Parameters:
    /// - intent: The classified intent type
    /// - targets: Extracted targets from translator
    /// - raw_text: Original user request
    fn matches(&self, intent: &str, targets: &[String], raw_text: &str) -> u32;

    /// Generate the diagnostic plan - ordered list of checks
    fn plan(&self) -> Vec<DiagnosticCheck>;

    /// Run diagnosis on collected evidence
    fn diagnose(&self, evidence: &[CollectedEvidence]) -> DiagnosisResult;

    // =========================================================================
    // v0.0.64: Lifecycle Stage Methods
    // =========================================================================

    /// Intake stage: understand symptoms and return what we'll check
    fn intake(&self, ticket: &Ticket) -> IntakeResult {
        // Default implementation extracts keywords from ticket
        let symptoms = ticket.matched_keywords.clone();
        let checking = self.plan().iter().map(|c| c.description.clone()).collect();

        IntakeResult {
            symptoms,
            checking,
            summary: format!("{} reviewing symptoms", self.name()),
            evidence_topics: self.evidence_topics(),
        }
    }

    /// Return the evidence topics this doctor needs
    fn evidence_topics(&self) -> Vec<EvidenceTopic> {
        // Default: map domain to evidence topic
        match self.domain() {
            DoctorDomain::Network => vec![EvidenceTopic::NetworkStatus],
            DoctorDomain::Storage => vec![EvidenceTopic::DiskFree],
            DoctorDomain::Audio => vec![EvidenceTopic::AudioStatus],
            DoctorDomain::Boot => vec![EvidenceTopic::BootTime],
            DoctorDomain::Graphics => vec![EvidenceTopic::GraphicsStatus],
            DoctorDomain::System => vec![EvidenceTopic::Unknown],
        }
    }

    /// Human dialogue for current stage
    fn human_dialogue(&self, stage: DoctorLifecycleStage) -> String {
        match stage {
            DoctorLifecycleStage::Intake => {
                format!("I'm looking at your {} issue.", self.domain_label())
            }
            DoctorLifecycleStage::EvidenceGathering => {
                format!("Checking {}.", self.checking_what())
            }
            DoctorLifecycleStage::Diagnosis => "Analyzing what I found.".to_string(),
            DoctorLifecycleStage::Planning => "Preparing recommendations.".to_string(),
            DoctorLifecycleStage::Verification => "Verifying the findings.".to_string(),
            DoctorLifecycleStage::HandOff => "Consulting with another specialist.".to_string(),
            DoctorLifecycleStage::Complete => "Analysis complete.".to_string(),
        }
    }

    /// Human-readable domain label
    fn domain_label(&self) -> &'static str {
        match self.domain() {
            DoctorDomain::Network => "network",
            DoctorDomain::Storage => "storage",
            DoctorDomain::Audio => "audio",
            DoctorDomain::Boot => "boot",
            DoctorDomain::Graphics => "graphics",
            DoctorDomain::System => "system",
        }
    }

    /// Human-readable description of what this doctor checks
    fn checking_what(&self) -> String {
        match self.domain() {
            DoctorDomain::Network => "link state, IP, default route, and DNS".to_string(),
            DoctorDomain::Storage => "disk usage, filesystems, and mount points".to_string(),
            DoctorDomain::Audio => "audio services, devices, and mixer state".to_string(),
            DoctorDomain::Boot => "boot time, service delays, and startup issues".to_string(),
            DoctorDomain::Graphics => {
                "GPU status, display configuration, and compositor".to_string()
            }
            DoctorDomain::System => "general system state".to_string(),
        }
    }
}

// =============================================================================
// Doctor Runner
// =============================================================================

/// Orchestrates the doctor lifecycle
pub struct DoctorRunner {
    /// Evidence prefix for this run
    evidence_prefix: String,
    /// Evidence counter
    evidence_counter: u32,
}

impl DoctorRunner {
    /// Create a new runner with given evidence prefix
    pub fn new(evidence_prefix: &str) -> Self {
        Self {
            evidence_prefix: evidence_prefix.to_string(),
            evidence_counter: 0,
        }
    }

    /// Generate next evidence ID
    pub fn next_evidence_id(&mut self) -> String {
        self.evidence_counter += 1;
        format!("{}{}", self.evidence_prefix, self.evidence_counter)
    }

    /// Execute a diagnostic check and collect evidence
    ///
    /// This is called by the pipeline for each check in the plan.
    /// The actual tool execution happens in the daemon.
    pub fn record_evidence(
        &mut self,
        check: &DiagnosticCheck,
        tool_result: serde_json::Value,
        summary: &str,
        success: bool,
    ) -> CollectedEvidence {
        CollectedEvidence {
            evidence_id: self.next_evidence_id(),
            check_id: check.id.clone(),
            tool_name: check.tool_name.clone(),
            data: tool_result,
            summary: summary.to_string(),
            collected_at: Utc::now(),
            success,
        }
    }

    /// Build a doctor report from the run
    pub fn build_report(
        &self,
        doctor: &dyn Doctor,
        user_request: &str,
        evidence: Vec<CollectedEvidence>,
        diagnosis: DiagnosisResult,
        start_time: DateTime<Utc>,
    ) -> DoctorReport {
        let plan = doctor.plan();
        let checks_performed: Vec<String> = plan.iter().map(|c| c.description.clone()).collect();

        let duration_ms = (Utc::now() - start_time).num_milliseconds() as u64;

        DoctorReport {
            report_id: crate::generate_request_id(),
            doctor_id: doctor.id().to_string(),
            doctor_name: doctor.name().to_string(),
            domain: doctor.domain(),
            user_request: user_request.to_string(),
            generated_at: Utc::now(),
            checks_performed,
            evidence,
            diagnosis,
            duration_ms,
        }
    }
}

// =============================================================================
// Network Evidence Helpers (used by NetworkingDoctor)
// =============================================================================

/// Network interface summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetInterfaceSummary {
    pub name: String,
    pub state: String,     // UP, DOWN, UNKNOWN
    pub operstate: String, // up, down, unknown
    pub carrier: bool,
    pub mac: Option<String>,
    pub ip4: Vec<String>,
    pub ip6: Vec<String>,
    pub is_wireless: bool,
}

/// Route summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteSummary {
    pub has_default_route: bool,
    pub default_gateway: Option<String>,
    pub default_interface: Option<String>,
    pub route_count: usize,
}

/// DNS summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsSummary {
    pub servers: Vec<String>,
    pub source: String, // resolv.conf, systemd-resolved, NetworkManager
    pub is_stub_resolver: bool,
    pub domains: Vec<String>,
}

/// NetworkManager summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NmSummary {
    pub installed: bool,
    pub running: bool,
    pub connectivity: String, // full, limited, portal, none, unknown
    pub primary_connection: Option<String>,
    pub wifi_enabled: bool,
}

/// Wireless summary (iw)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiSummary {
    pub connected: bool,
    pub ssid: Option<String>,
    pub signal_dbm: Option<i32>,
    pub signal_quality: Option<String>, // excellent, good, fair, poor, none
    pub frequency: Option<String>,
    pub bitrate: Option<String>,
}

/// Network errors summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkErrorsSummary {
    pub error_count: usize,
    pub warning_count: usize,
    pub recent_errors: Vec<String>,
    pub recent_warnings: Vec<String>,
    pub time_range_minutes: u32,
}

/// Ping check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingCheckResult {
    pub target: String,
    pub success: bool,
    pub packets_sent: u32,
    pub packets_received: u32,
    pub avg_latency_ms: Option<f64>,
    pub error: Option<String>,
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnosis_result_no_fault() {
        let result = DiagnosisResult::no_fault_found(
            "No obvious network issues detected",
            vec![SafeNextStep {
                description: "Check if ISP has outage".to_string(),
                rationale: "External connectivity could be the issue".to_string(),
                evidence_ids: vec![],
            }],
        );

        assert!(result.most_likely_cause.is_none());
        assert_eq!(result.confidence, 50);
        assert_eq!(result.next_steps.len(), 1);
    }

    #[test]
    fn test_doctor_report_render() {
        let report = DoctorReport {
            report_id: "dr-test-001".to_string(),
            doctor_id: "network_doctor".to_string(),
            doctor_name: "Network Doctor".to_string(),
            domain: DoctorDomain::Network,
            user_request: "my wifi keeps disconnecting".to_string(),
            generated_at: Utc::now(),
            checks_performed: vec![
                "Check physical link".to_string(),
                "Check IP address".to_string(),
            ],
            evidence: vec![],
            diagnosis: DiagnosisResult {
                summary: "WiFi signal is weak".to_string(),
                most_likely_cause: Some("Low signal strength".to_string()),
                findings: vec![DiagnosisFinding {
                    id: "f1".to_string(),
                    description: "WiFi signal at -78 dBm (poor)".to_string(),
                    severity: FindingSeverity::Warning,
                    evidence_ids: vec!["N3".to_string()],
                    confidence: 85,
                    tags: vec!["wifi".to_string(), "signal".to_string()],
                }],
                confidence: 85,
                next_steps: vec![SafeNextStep {
                    description: "Move closer to the access point".to_string(),
                    rationale: "Signal strength improves with proximity".to_string(),
                    evidence_ids: vec!["N3".to_string()],
                }],
                proposed_actions: vec![],
                issue_resolved: false,
                symptom_keywords: vec!["wifi".to_string(), "disconnecting".to_string()],
            },
            duration_ms: 1500,
        };

        let rendered = report.render();
        assert!(rendered.contains("Network Doctor Report"));
        assert!(rendered.contains("WiFi signal is weak"));
        assert!(rendered.contains("[N3]"));
    }

    #[test]
    fn test_report_qualifies_for_learning() {
        let report = DoctorReport {
            report_id: "dr-test-001".to_string(),
            doctor_id: "network_doctor".to_string(),
            doctor_name: "Network Doctor".to_string(),
            domain: DoctorDomain::Network,
            user_request: "test".to_string(),
            generated_at: Utc::now(),
            checks_performed: vec![],
            evidence: vec![
                CollectedEvidence {
                    evidence_id: "N1".to_string(),
                    check_id: "c1".to_string(),
                    tool_name: "test".to_string(),
                    data: serde_json::Value::Null,
                    summary: "test".to_string(),
                    collected_at: Utc::now(),
                    success: true,
                },
                CollectedEvidence {
                    evidence_id: "N2".to_string(),
                    check_id: "c2".to_string(),
                    tool_name: "test".to_string(),
                    data: serde_json::Value::Null,
                    summary: "test".to_string(),
                    collected_at: Utc::now(),
                    success: true,
                },
                CollectedEvidence {
                    evidence_id: "N3".to_string(),
                    check_id: "c3".to_string(),
                    tool_name: "test".to_string(),
                    data: serde_json::Value::Null,
                    summary: "test".to_string(),
                    collected_at: Utc::now(),
                    success: true,
                },
            ],
            diagnosis: DiagnosisResult {
                summary: "test".to_string(),
                most_likely_cause: None,
                findings: vec![],
                confidence: 85,
                next_steps: vec![],
                proposed_actions: vec![],
                issue_resolved: false,
                symptom_keywords: vec![],
            },
            duration_ms: 1000,
        };

        // 3 evidence, 85% confidence, but reliability 80 = not qualified
        assert!(!report.qualifies_for_learning(80));

        // 3 evidence, 85% confidence, reliability 90 = qualified
        assert!(report.qualifies_for_learning(90));
    }
}
