//! Case Coordinator v0.0.69 - Service Desk Coordination Layer
//!
//! The Case Coordinator runs for every request, orchestrating:
//! 1. open_case(request) -> case_id with owner assignment
//! 2. triage(request) -> primary + supporting departments, intent, risk
//! 3. dispatch(dept, context) -> DepartmentReport
//! 4. merge_reports(reports) -> ConsolidatedAssessment
//! 5. compose_user_answer(assessment) -> final human answer
//! 6. log(case) -> persist both human and debug transcripts
//!
//! Human Mode shows believable IT team coordination without internals.
//! Debug Mode shows full triage rationale and department report details.

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::case_lifecycle::{ActionRisk, CaseFileV2, CaseStatus};
use crate::department_protocol::{DepartmentFinding, DepartmentName, DepartmentResult};
use crate::evidence_topic::EvidenceTopic;
use crate::service_desk::{detect_ticket_type, is_problem_report};

// ============================================================================
// Evidence Topic Mapping
// ============================================================================

/// Maps query targets to required evidence topics
pub fn get_evidence_topics_for_target(target: &str) -> Vec<EvidenceTopic> {
    let lower = target.to_lowercase();

    // Memory queries
    if lower.contains("memory") || lower.contains("ram") {
        return vec![EvidenceTopic::MemoryInfo];
    }

    // Disk/storage queries
    if lower.contains("disk")
        || lower.contains("space")
        || lower.contains("storage")
        || lower.contains("mount")
        || lower.contains("filesystem")
    {
        return vec![EvidenceTopic::DiskFree];
    }

    // Kernel queries
    if lower.contains("kernel") || lower.contains("linux version") {
        return vec![EvidenceTopic::KernelVersion];
    }

    // Network queries
    if lower.contains("network")
        || lower.contains("wifi")
        || lower.contains("internet")
        || lower.contains("connection")
        || lower.contains("ip")
    {
        return vec![EvidenceTopic::NetworkStatus];
    }

    // Audio queries
    if lower.contains("audio") || lower.contains("sound") || lower.contains("speaker") {
        return vec![EvidenceTopic::AudioStatus];
    }

    // CPU queries
    if lower.contains("cpu") || lower.contains("processor") {
        return vec![EvidenceTopic::CpuInfo];
    }

    // Boot queries
    if lower.contains("boot") || lower.contains("startup") || lower.contains("systemd") {
        return vec![EvidenceTopic::BootTime, EvidenceTopic::ServiceState];
    }

    // Graphics queries
    if lower.contains("graphics") || lower.contains("gpu") || lower.contains("display") {
        return vec![EvidenceTopic::GraphicsStatus];
    }

    // Generic - unknown, requires LLM classification
    vec![EvidenceTopic::Unknown]
}

// ============================================================================
// Intent Classification (Hard Rules)
// ============================================================================

/// Intent type for the request
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestIntent {
    /// Asking about system state (read-only)
    SystemQuery,
    /// Reporting a problem/incident
    ProblemReport,
    /// Requesting a change/action
    ActionRequest,
}

impl RequestIntent {
    pub fn as_str(&self) -> &'static str {
        match self {
            RequestIntent::SystemQuery => "system_query",
            RequestIntent::ProblemReport => "problem_report",
            RequestIntent::ActionRequest => "action_request",
        }
    }
}

/// Classify request intent with hard rules to prevent misclassification
pub fn classify_intent(request: &str) -> RequestIntent {
    let lower = request.to_lowercase();
    let trimmed = lower.trim();

    // Hard rule: questions about system state are ALWAYS SystemQuery
    let query_starters = [
        "how much",
        "how many",
        "what is",
        "what's",
        "what are",
        "which",
        "show me",
        "show",
        "is my",
        "are my",
        "tell me",
        "list",
        "do i have",
        "does my",
    ];
    for starter in &query_starters {
        if trimmed.starts_with(starter) {
            // Unless followed by action verbs
            let action_verbs = [
                "install", "remove", "delete", "disable", "enable", "edit", "change", "add", "fix",
            ];
            let has_action = action_verbs.iter().any(|v| lower.contains(v));
            if !has_action {
                return RequestIntent::SystemQuery;
            }
        }
    }

    // Hard rule: imperative action verbs = ActionRequest
    let action_starters = [
        "install",
        "remove",
        "uninstall",
        "delete",
        "add",
        "edit",
        "modify",
        "change",
        "update",
        "upgrade",
        "enable",
        "disable",
        "start",
        "stop",
        "restart",
        "configure",
        "setup",
        "set up",
        "write",
        "create",
    ];
    for starter in &action_starters {
        if trimmed.starts_with(starter) {
            return RequestIntent::ActionRequest;
        }
    }

    // Check for problem indicators
    if is_problem_report(request) {
        return RequestIntent::ProblemReport;
    }

    // "Can you X" or "Could you X" with action verbs = ActionRequest
    if trimmed.starts_with("can you")
        || trimmed.starts_with("could you")
        || trimmed.starts_with("please")
    {
        let has_action = action_starters.iter().any(|v| lower.contains(v));
        if has_action {
            return RequestIntent::ActionRequest;
        }
    }

    // Default to SystemQuery for safety
    RequestIntent::SystemQuery
}

// ============================================================================
// Triage Result
// ============================================================================

/// Result of triage phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriageDecision {
    /// Primary department to handle the case
    pub primary_dept: DepartmentName,
    /// Supporting departments (up to 2)
    pub supporting_depts: Vec<DepartmentName>,
    /// Detected intent
    pub intent: RequestIntent,
    /// Risk level
    pub risk: ActionRisk,
    /// Confidence in routing (0-100)
    pub confidence: u8,
    /// Keywords that triggered this routing
    pub matched_keywords: Vec<String>,
    /// Human-readable rationale
    pub rationale: String,
    /// Required evidence topics
    pub evidence_topics: Vec<EvidenceTopic>,
}

impl TriageDecision {
    pub fn new(primary: DepartmentName, intent: RequestIntent) -> Self {
        Self {
            primary_dept: primary,
            supporting_depts: Vec::new(),
            intent,
            risk: ActionRisk::ReadOnly,
            confidence: 50,
            matched_keywords: Vec::new(),
            rationale: String::new(),
            evidence_topics: Vec::new(),
        }
    }

    pub fn with_support(mut self, depts: Vec<DepartmentName>) -> Self {
        self.supporting_depts = depts.into_iter().take(2).collect();
        self
    }

    pub fn with_confidence(mut self, confidence: u8) -> Self {
        self.confidence = confidence;
        self
    }

    pub fn with_keywords(mut self, keywords: Vec<String>) -> Self {
        self.matched_keywords = keywords;
        self
    }

    pub fn with_rationale(mut self, rationale: &str) -> Self {
        self.rationale = rationale.to_string();
        self
    }

    pub fn with_topics(mut self, topics: Vec<EvidenceTopic>) -> Self {
        self.evidence_topics = topics;
        self
    }

    pub fn with_risk(mut self, risk: ActionRisk) -> Self {
        self.risk = risk;
        self
    }

    /// Get all involved departments
    pub fn all_departments(&self) -> Vec<DepartmentName> {
        let mut all = vec![self.primary_dept];
        all.extend(self.supporting_depts.clone());
        all
    }
}

// ============================================================================
// Department Report
// ============================================================================

/// Structured report from a department investigation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepartmentReport {
    /// Department that produced this report
    pub department: DepartmentName,
    /// Human-readable summary (for Human Mode)
    pub summary_human: String,
    /// Structured findings (internal)
    pub findings: Vec<DepartmentFinding>,
    /// Evidence topics used with human summaries
    pub evidence_topics: Vec<EvidenceTopicSummary>,
    /// Confidence in the report (0-100)
    pub confidence: u8,
    /// Recommended next steps (read-only suggestions)
    pub recommended_next_steps: Vec<String>,
    /// Action plan (only if mutation is appropriate)
    pub action_plan: Option<ActionPlan>,
    /// Policy notes (e.g., "needs confirmation", "protected path")
    pub policy_notes: Vec<String>,
    /// Duration of investigation in milliseconds
    pub duration_ms: u64,
}

/// Evidence topic with human + debug summaries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceTopicSummary {
    /// Topic title (human readable)
    pub title: String,
    /// Human summary (no tool names, no IDs)
    pub summary_human: String,
    /// Debug summary (tool names, evidence IDs)
    pub summary_debug: String,
    /// Evidence IDs collected
    pub evidence_ids: Vec<String>,
}

/// Action plan for mutations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPlan {
    /// Description of the action
    pub description: String,
    /// Steps in human terms
    pub steps_human: Vec<String>,
    /// Steps in debug terms (commands)
    pub steps_debug: Vec<String>,
    /// Risk level
    pub risk: ActionRisk,
    /// Confirmation phrase required
    pub confirmation_phrase: Option<String>,
    /// Rollback description
    pub rollback_human: String,
}

impl DepartmentReport {
    pub fn new(department: DepartmentName, summary: &str) -> Self {
        Self {
            department,
            summary_human: summary.to_string(),
            findings: Vec::new(),
            evidence_topics: Vec::new(),
            confidence: 50,
            recommended_next_steps: Vec::new(),
            action_plan: None,
            policy_notes: Vec::new(),
            duration_ms: 0,
        }
    }

    pub fn with_findings(mut self, findings: Vec<DepartmentFinding>) -> Self {
        self.findings = findings;
        self
    }

    pub fn with_evidence(mut self, topics: Vec<EvidenceTopicSummary>) -> Self {
        self.evidence_topics = topics;
        self
    }

    pub fn with_confidence(mut self, confidence: u8) -> Self {
        self.confidence = confidence;
        self
    }

    pub fn with_next_steps(mut self, steps: Vec<String>) -> Self {
        self.recommended_next_steps = steps;
        self
    }

    pub fn with_action_plan(mut self, plan: ActionPlan) -> Self {
        self.action_plan = Some(plan);
        self
    }

    pub fn with_policy_note(mut self, note: &str) -> Self {
        self.policy_notes.push(note.to_string());
        self
    }

    pub fn with_duration(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }

    /// Convert from DepartmentResult (legacy bridge)
    pub fn from_department_result(result: &DepartmentResult) -> Self {
        Self {
            department: result.department,
            summary_human: result.summary.clone(),
            findings: result.findings.clone(),
            evidence_topics: Vec::new(),
            confidence: result.reliability_hint,
            recommended_next_steps: result
                .recommended_actions
                .iter()
                .map(|a| a.description.clone())
                .collect(),
            action_plan: None,
            policy_notes: Vec::new(),
            duration_ms: 0,
        }
    }
}

// ============================================================================
// Consolidated Assessment
// ============================================================================

/// Merged assessment from all department reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidatedAssessment {
    /// All department reports
    pub reports: Vec<DepartmentReport>,
    /// Overall summary (human)
    pub summary_human: String,
    /// All findings merged
    pub all_findings: Vec<DepartmentFinding>,
    /// Combined evidence topics
    pub evidence_topics: Vec<EvidenceTopicSummary>,
    /// Overall confidence (weighted average)
    pub confidence: u8,
    /// Overall risk level
    pub risk: ActionRisk,
    /// Whether any action needs confirmation
    pub needs_confirmation: bool,
    /// Combined action plan (if any)
    pub action_plan: Option<ActionPlan>,
    /// All policy notes
    pub policy_notes: Vec<String>,
}

impl ConsolidatedAssessment {
    pub fn merge(reports: Vec<DepartmentReport>) -> Self {
        if reports.is_empty() {
            return Self {
                reports: Vec::new(),
                summary_human: "No department reports available.".to_string(),
                all_findings: Vec::new(),
                evidence_topics: Vec::new(),
                confidence: 0,
                risk: ActionRisk::ReadOnly,
                needs_confirmation: false,
                action_plan: None,
                policy_notes: Vec::new(),
            };
        }

        // Merge all findings
        let all_findings: Vec<_> = reports.iter().flat_map(|r| r.findings.clone()).collect();

        // Merge all evidence topics
        let evidence_topics: Vec<_> = reports
            .iter()
            .flat_map(|r| r.evidence_topics.clone())
            .collect();

        // Weighted average confidence
        let total_conf: u32 = reports.iter().map(|r| r.confidence as u32).sum();
        let avg_conf = (total_conf / reports.len() as u32) as u8;

        // Highest risk level
        let max_risk = reports
            .iter()
            .filter_map(|r| r.action_plan.as_ref().map(|p| p.risk))
            .max()
            .unwrap_or(ActionRisk::ReadOnly);

        // Check if confirmation needed
        let needs_confirmation = reports.iter().any(|r| {
            r.action_plan
                .as_ref()
                .map(|p| p.confirmation_phrase.is_some())
                .unwrap_or(false)
        });

        // Use primary department's action plan if any
        let action_plan = reports.iter().find_map(|r| r.action_plan.clone());

        // Collect policy notes
        let policy_notes: Vec<_> = reports
            .iter()
            .flat_map(|r| r.policy_notes.clone())
            .collect();

        // Build summary from all reports
        let summary_parts: Vec<_> = reports.iter().map(|r| r.summary_human.clone()).collect();
        let summary_human = summary_parts.join(" ");

        Self {
            reports,
            summary_human,
            all_findings,
            evidence_topics,
            confidence: avg_conf,
            risk: max_risk,
            needs_confirmation,
            action_plan,
            policy_notes,
        }
    }
}

// ============================================================================
// Case Coordinator
// ============================================================================

/// Coordinator for all case processing
#[derive(Debug, Clone)]
pub struct CaseCoordinator {
    /// Current case being processed
    pub case_id: String,
    /// Case file
    pub case: CaseFileV2,
    /// Triage decision
    pub triage: Option<TriageDecision>,
    /// Department reports collected
    pub reports: Vec<DepartmentReport>,
    /// Consolidated assessment
    pub assessment: Option<ConsolidatedAssessment>,
    /// Human transcript lines
    pub transcript_human: Vec<String>,
    /// Debug transcript lines
    pub transcript_debug: Vec<String>,
}

impl CaseCoordinator {
    /// Open a new case for a request
    pub fn open_case(request: &str) -> Self {
        let case = CaseFileV2::new(&generate_case_id(), request);
        let case_id = case.case_id.clone();

        let mut coordinator = Self {
            case_id,
            case,
            triage: None,
            reports: Vec::new(),
            assessment: None,
            transcript_human: Vec::new(),
            transcript_debug: Vec::new(),
        };

        // Log case opening
        coordinator.log_human("[service-desk] Opening case and reviewing request.");
        coordinator.log_debug(&format!(
            "case_id={}, request=\"{}\"",
            coordinator.case_id, request
        ));

        coordinator
    }

    /// Triage the request
    pub fn triage(&mut self, request: &str) -> &TriageDecision {
        let intent = classify_intent(request);
        let ticket_type = detect_ticket_type(request);

        // Determine primary department based on keywords
        let (primary, keywords) = self.detect_primary_department(request);

        // Determine supporting departments for compound queries
        let supporting = self.detect_supporting_departments(request, primary);

        // Determine evidence topics
        let topics = get_evidence_topics_for_target(request);

        // Determine risk
        let risk = match intent {
            RequestIntent::SystemQuery => ActionRisk::ReadOnly,
            RequestIntent::ProblemReport => ActionRisk::ReadOnly,
            RequestIntent::ActionRequest => ActionRisk::Medium,
        };

        // Build rationale
        let rationale = format!(
            "Intent: {:?}, Keywords: {:?}, Primary: {}, Support: {:?}",
            intent, keywords, primary, supporting
        );

        let triage = TriageDecision::new(primary, intent)
            .with_support(supporting.clone())
            .with_confidence(if keywords.is_empty() { 50 } else { 85 })
            .with_keywords(keywords)
            .with_rationale(&rationale)
            .with_topics(topics)
            .with_risk(risk);

        // Update case
        self.case.set_status(CaseStatus::Triaged);
        self.case.ticket_type = ticket_type;
        self.case
            .assign_department(primary.to_department(), &rationale);

        // Log triage (human shows simplified version)
        if supporting.is_empty() {
            self.log_human(&format!(
                "[service-desk] I'll have {} look into this.",
                primary.human_label()
            ));
        } else {
            let support_names: Vec<_> = supporting.iter().map(|d| d.human_label()).collect();
            self.log_human(&format!(
                "[service-desk] I'll open a case and pull in {} (primary) and {} (support).",
                primary.human_label(),
                support_names.join(", ")
            ));
        }

        self.log_debug(&format!("TRIAGE: {}", rationale));

        self.triage = Some(triage);
        self.triage.as_ref().unwrap()
    }

    /// Detect primary department from keywords
    fn detect_primary_department(&self, request: &str) -> (DepartmentName, Vec<String>) {
        let lower = request.to_lowercase();
        let mut matched = Vec::new();

        // Check each department's keywords
        let dept_keywords = [
            (
                DepartmentName::Networking,
                &[
                    "wifi",
                    "network",
                    "internet",
                    "dns",
                    "connection",
                    "ip",
                    "ethernet",
                ][..],
            ),
            (
                DepartmentName::Storage,
                &[
                    "disk",
                    "storage",
                    "space",
                    "mount",
                    "btrfs",
                    "filesystem",
                    "drive",
                ][..],
            ),
            (
                DepartmentName::Audio,
                &[
                    "audio",
                    "sound",
                    "speaker",
                    "microphone",
                    "pipewire",
                    "volume",
                ][..],
            ),
            (
                DepartmentName::Boot,
                &["boot", "startup", "systemd", "slow boot", "service"][..],
            ),
            (
                DepartmentName::Graphics,
                &[
                    "graphics", "gpu", "display", "monitor", "screen", "nvidia", "wayland",
                ][..],
            ),
            (
                DepartmentName::Performance,
                &["slow", "performance", "cpu", "memory", "ram", "lag"][..],
            ),
        ];

        let mut best_dept = DepartmentName::InfoDesk;
        let mut best_count = 0;

        for (dept, keywords) in dept_keywords {
            let mut count = 0;
            for kw in keywords {
                if lower.contains(kw) {
                    count += 1;
                    matched.push(kw.to_string());
                }
            }
            if count > best_count {
                best_count = count;
                best_dept = dept;
            }
        }

        (best_dept, matched)
    }

    /// Detect supporting departments for compound queries
    fn detect_supporting_departments(
        &self,
        request: &str,
        primary: DepartmentName,
    ) -> Vec<DepartmentName> {
        let lower = request.to_lowercase();
        let mut supporting = Vec::new();

        // Common compound patterns
        let patterns = [
            // "wifi disconnecting AND sound crackling"
            (vec!["wifi", "network"], DepartmentName::Networking),
            (vec!["sound", "audio"], DepartmentName::Audio),
            (vec!["boot", "startup"], DepartmentName::Boot),
            (vec!["disk", "storage"], DepartmentName::Storage),
            (vec!["graphics", "display"], DepartmentName::Graphics),
        ];

        for (keywords, dept) in patterns {
            if dept != primary
                && keywords.iter().any(|k| lower.contains(k))
                && !supporting.contains(&dept)
            {
                supporting.push(dept);
            }
        }

        // Limit to 2 supporting departments
        supporting.truncate(2);
        supporting
    }

    /// Add a department report
    pub fn add_report(&mut self, report: DepartmentReport) {
        // Log human-readable summary
        self.log_human(&format!(
            "[{}] {}",
            report.department.as_str(),
            report.summary_human
        ));

        // Log evidence topics
        for topic in &report.evidence_topics {
            self.log_human(&format!(
                "[{}] {}: {}",
                report.department.as_str(),
                topic.title,
                topic.summary_human
            ));
        }

        // Debug log with full details
        self.log_debug(&format!(
            "REPORT from {}: confidence={}, findings={}, duration={}ms",
            report.department.as_str(),
            report.confidence,
            report.findings.len(),
            report.duration_ms
        ));

        self.reports.push(report);
    }

    /// Merge all reports into consolidated assessment
    pub fn merge_reports(&mut self) -> &ConsolidatedAssessment {
        let assessment = ConsolidatedAssessment::merge(self.reports.clone());

        self.log_debug(&format!(
            "MERGED: {} reports, confidence={}, risk={:?}",
            assessment.reports.len(),
            assessment.confidence,
            assessment.risk
        ));

        self.assessment = Some(assessment);
        self.assessment.as_ref().unwrap()
    }

    /// Compose final user answer
    pub fn compose_user_answer(&mut self) -> String {
        // Clone values we need before borrowing self mutably
        let (answer, confidence, risk) = {
            let assessment = self
                .assessment
                .as_ref()
                .expect("Must call merge_reports before compose_user_answer");
            (
                assessment.summary_human.clone(),
                assessment.confidence,
                assessment.risk,
            )
        };

        // Add reliability footer
        let reliability_desc = if confidence >= 80 {
            "good evidence coverage"
        } else if confidence >= 60 {
            "some evidence gaps"
        } else {
            "limited evidence"
        };

        // Log final answer
        self.log_human(&format!("[service-desk] {}", answer));
        self.log_human("");
        self.log_human(&format!(
            "Reliability: {}% ({})",
            confidence, reliability_desc
        ));

        self.log_debug(&format!(
            "FINAL: reliability={}, risk={:?}",
            confidence, risk
        ));

        answer
    }

    /// Get human transcript
    pub fn human_transcript(&self) -> Vec<String> {
        self.transcript_human.clone()
    }

    /// Get debug transcript
    pub fn debug_transcript(&self) -> Vec<String> {
        self.transcript_debug.clone()
    }

    /// Log to human transcript
    fn log_human(&mut self, line: &str) {
        self.transcript_human.push(line.to_string());
    }

    /// Log to debug transcript
    fn log_debug(&mut self, line: &str) {
        let ts = Utc::now().format("%H:%M:%S%.3f");
        self.transcript_debug.push(format!("{} {}", ts, line));
    }
}

/// Generate a unique case ID
fn generate_case_id() -> String {
    let now = Utc::now();
    format!("C-{}-{:04}", now.format("%Y%m%d"), rand_suffix())
}

fn rand_suffix() -> u16 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    (nanos % 10000) as u16
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_intent_system_query() {
        assert_eq!(
            classify_intent("how much memory do I have"),
            RequestIntent::SystemQuery
        );
        assert_eq!(
            classify_intent("what is my kernel version"),
            RequestIntent::SystemQuery
        );
        assert_eq!(
            classify_intent("show me disk space"),
            RequestIntent::SystemQuery
        );
        assert_eq!(
            classify_intent("which packages are installed"),
            RequestIntent::SystemQuery
        );
    }

    #[test]
    fn test_classify_intent_action_request() {
        assert_eq!(
            classify_intent("install firefox"),
            RequestIntent::ActionRequest
        );
        assert_eq!(
            classify_intent("please remove vim"),
            RequestIntent::ActionRequest
        );
        assert_eq!(
            classify_intent("restart nginx"),
            RequestIntent::ActionRequest
        );
        assert_eq!(
            classify_intent("enable ssh service"),
            RequestIntent::ActionRequest
        );
    }

    #[test]
    fn test_classify_intent_problem_report() {
        assert_eq!(
            classify_intent("wifi keeps disconnecting"),
            RequestIntent::ProblemReport
        );
        assert_eq!(
            classify_intent("no sound from speakers"),
            RequestIntent::ProblemReport
        );
        assert_eq!(
            classify_intent("my system is slow"),
            RequestIntent::ProblemReport
        );
    }

    #[test]
    fn test_evidence_topics_for_memory() {
        let topics = get_evidence_topics_for_target("how much memory do I have");
        assert!(topics.contains(&EvidenceTopic::MemoryInfo));
        assert!(!topics.contains(&EvidenceTopic::CpuInfo));
    }

    #[test]
    fn test_evidence_topics_for_disk() {
        let topics = get_evidence_topics_for_target("how much disk space is free");
        assert!(topics.contains(&EvidenceTopic::DiskFree));
    }

    #[test]
    fn test_case_coordinator_triage() {
        let mut coord = CaseCoordinator::open_case("wifi keeps disconnecting");
        let triage = coord.triage("wifi keeps disconnecting");

        assert_eq!(triage.primary_dept, DepartmentName::Networking);
        assert_eq!(triage.intent, RequestIntent::ProblemReport);
        assert!(triage.matched_keywords.contains(&"wifi".to_string()));
    }

    #[test]
    fn test_multi_dept_detection() {
        let mut coord = CaseCoordinator::open_case("wifi disconnecting and sound crackling");
        let triage = coord.triage("wifi disconnecting and sound crackling");

        // Should have both networking and audio
        let all_depts = triage.all_departments();
        assert!(
            all_depts.contains(&DepartmentName::Networking)
                || all_depts.contains(&DepartmentName::Audio)
        );
    }

    #[test]
    fn test_department_report_merge() {
        let report1 = DepartmentReport::new(DepartmentName::Networking, "Network is stable.")
            .with_confidence(90);
        let report2 = DepartmentReport::new(DepartmentName::Storage, "Disk usage is normal.")
            .with_confidence(80);

        let assessment = ConsolidatedAssessment::merge(vec![report1, report2]);

        assert_eq!(assessment.reports.len(), 2);
        assert_eq!(assessment.confidence, 85); // Average
        assert!(assessment.summary_human.contains("Network"));
        assert!(assessment.summary_human.contains("Disk"));
    }

    #[test]
    fn test_human_transcript_no_internals() {
        let mut coord = CaseCoordinator::open_case("what cpu do I have");
        coord.triage("what cpu do I have");

        let transcript = coord.human_transcript().join("\n");

        // Human transcript should not contain debug info
        assert!(!transcript.contains("case_id="));
        assert!(!transcript.contains("TRIAGE:"));
        assert!(transcript.contains("[service-desk]"));
    }

    // v0.0.71: Additional tests for misclassification guardrails

    #[test]
    fn test_systemd_query_evidence_topics() {
        let topics = get_evidence_topics_for_target("is systemd running");
        // Should include both boot time and service state evidence
        assert!(
            topics.contains(&EvidenceTopic::BootTime)
                || topics.contains(&EvidenceTopic::ServiceState)
        );
    }

    #[test]
    fn test_memory_query_not_cpu() {
        // Memory query should never get CPU evidence
        let topics = get_evidence_topics_for_target("how much ram do i have");
        assert!(topics.contains(&EvidenceTopic::MemoryInfo));
        assert!(!topics.contains(&EvidenceTopic::CpuInfo));
    }

    #[test]
    fn test_disk_query_not_cpu() {
        // Disk query should never get CPU evidence
        let topics = get_evidence_topics_for_target("how much storage do I have");
        assert!(topics.contains(&EvidenceTopic::DiskFree));
        assert!(!topics.contains(&EvidenceTopic::CpuInfo));
    }
}
