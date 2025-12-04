//! Doctor Flow v0.0.53 - Orchestrates diagnostic flows with evidence collection

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::doctor_lifecycle::{
    CollectedEvidence, DiagnosisResult, DiagnosticCheck, Doctor, DoctorReport, DoctorRunner,
};
use crate::doctor_registry::{DoctorDomain, DoctorSelection, FindingSeverity};
use crate::generate_request_id;

/// Keywords indicating a problem/diagnostic query
const PROBLEM_PHRASES: &[&str] = &[
    "not working", "doesn't work", "isn't working", "won't work",
    "broken", "fails", "failed", "failing", "error", "crash", "crashed",
    "disconnecting", "disconnects", "can't connect", "cannot connect",
    "no sound", "no audio", "no internet", "no network", "no wifi",
    "slow", "lag", "lagging", "stuttering", "freezing", "frozen",
    "black screen", "blank screen", "can't mount", "won't mount",
    "slow boot", "takes forever", "hangs", "hanging", "stuck",
    "keeps disconnecting", "keeps crashing", "keeps failing",
];

/// Single-word problem indicators
const PROBLEM_WORDS: &[&str] = &[
    "broken", "failed", "error", "crash", "slow", "lag", "stuck",
    "disconnecting", "stuttering", "freezing", "hanging", "offline",
];

/// Detect if request expresses a problem (0-100 confidence)
pub fn detect_problem_phrase(request: &str) -> (bool, u8, Vec<String>) {
    let lower = request.to_lowercase();
    let mut matched = Vec::new();
    let mut score: u32 = 0;

    // Check multi-word phrases first (higher confidence)
    for phrase in PROBLEM_PHRASES {
        if lower.contains(phrase) {
            matched.push(phrase.to_string());
            score += 30;
        }
    }

    // Check single words
    for word in PROBLEM_WORDS {
        if lower.split_whitespace().any(|w| w == *word || w.trim_matches(|c: char| !c.is_alphanumeric()) == *word) {
            if !matched.iter().any(|m| m.contains(word)) {
                matched.push(word.to_string());
                score += 15;
            }
        }
    }

    let confidence = score.min(100) as u8;
    let is_problem = confidence >= 25;

    (is_problem, confidence, matched)
}

/// A step in doctor flow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorFlowStep {
    pub step_number: u32,
    pub check: DiagnosticCheck,
    pub status: StepStatus,
    pub evidence: Option<CollectedEvidence>,
    pub duration_ms: u64,
    pub dialogue: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Pending,
    Running,
    Success,
    Failed,
    Skipped,
}

/// Result of a doctor flow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorFlowResult {
    pub flow_id: String,
    pub doctor_id: String,
    pub doctor_name: String,
    pub domain: DoctorDomain,
    pub user_request: String,
    pub selection_score: u32,
    pub selection_reason: String,
    pub steps: Vec<DoctorFlowStep>,
    pub evidence: Vec<CollectedEvidence>,
    pub diagnosis: DiagnosisResult,
    pub report: DoctorReport,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub total_duration_ms: u64,
}

impl DoctorFlowResult {
    /// Render human-readable summary for transcript
    pub fn render_summary(&self) -> String {
        let mut lines = vec![format!("Diagnosis Confidence: {}%\n", self.diagnosis.confidence)];

        if !self.diagnosis.findings.is_empty() {
            lines.push("Findings:".into());
            for f in &self.diagnosis.findings {
                let ev = if f.evidence_ids.is_empty() { String::new() } else { format!(" [{}]", f.evidence_ids.join(", ")) };
                let sev = match f.severity { FindingSeverity::Critical => "CRITICAL", FindingSeverity::Error => "ERROR", FindingSeverity::Warning => "WARNING", FindingSeverity::Info => "INFO" };
                lines.push(format!("  [{}] {}{}", sev, f.description, ev));
            }
            lines.push(String::new());
        }
        if let Some(cause) = &self.diagnosis.most_likely_cause {
            lines.push(format!("Most Likely Cause: {}\n", cause));
        }
        if !self.diagnosis.next_steps.is_empty() {
            lines.push("Suggested Next Steps:".into());
            for s in &self.diagnosis.next_steps { lines.push(format!("  - {}", s.description)); }
            lines.push(String::new());
        }
        if !self.diagnosis.proposed_actions.is_empty() {
            lines.push("Optional Fixes (require confirmation):".into());
            for a in &self.diagnosis.proposed_actions {
                lines.push(format!("  [{} risk] {}", a.risk, a.description));
                if let Some(p) = &a.confirmation_phrase { lines.push(format!("    Say: \"{}\"", p)); }
            }
        }
        lines.join("\n")
    }

    pub fn what_i_checked(&self) -> Vec<String> {
        self.steps.iter().filter(|s| s.status == StepStatus::Success).map(|s| s.check.description.clone()).collect()
    }

    pub fn what_i_found(&self) -> Vec<String> {
        self.diagnosis.findings.iter().map(|f| {
            let ev = if f.evidence_ids.is_empty() { String::new() } else { format!(" [{}]", f.evidence_ids.join(", ")) };
            format!("{}{}", f.description, ev)
        }).collect()
    }
}

/// Executes a doctor diagnostic flow
pub struct DoctorFlowExecutor {
    runner: DoctorRunner,
    steps: Vec<DoctorFlowStep>,
    evidence: Vec<CollectedEvidence>,
    start_time: DateTime<Utc>,
}

impl DoctorFlowExecutor {
    /// Create executor for a doctor domain
    pub fn new(domain: DoctorDomain) -> Self {
        let prefix = match domain {
            DoctorDomain::Network => "N",
            DoctorDomain::Storage => "S",
            DoctorDomain::Audio => "A",
            DoctorDomain::Boot => "B",
            DoctorDomain::Graphics => "G",
            DoctorDomain::System => "X",
        };

        Self {
            runner: DoctorRunner::new(prefix),
            steps: Vec::new(),
            evidence: Vec::new(),
            start_time: Utc::now(),
        }
    }

    /// Get the diagnostic plan as flow steps
    pub fn prepare_steps(&mut self, doctor: &dyn Doctor) -> Vec<DoctorFlowStep> {
        let plan = doctor.plan();
        self.steps = plan
            .into_iter()
            .enumerate()
            .map(|(i, check)| DoctorFlowStep {
                step_number: i as u32 + 1,
                check,
                status: StepStatus::Pending,
                evidence: None,
                duration_ms: 0,
                dialogue: String::new(),
            })
            .collect();
        self.steps.clone()
    }

    /// Record evidence for a step (called after tool execution)
    pub fn record_step_evidence(
        &mut self,
        step_index: usize,
        tool_result: serde_json::Value,
        summary: &str,
        success: bool,
        duration_ms: u64,
    ) -> Option<CollectedEvidence> {
        if step_index >= self.steps.len() {
            return None;
        }

        let step = &mut self.steps[step_index];
        let evidence = self.runner.record_evidence(&step.check, tool_result, summary, success);

        step.status = if success { StepStatus::Success } else { StepStatus::Failed };
        step.evidence = Some(evidence.clone());
        step.duration_ms = duration_ms;
        step.dialogue = format!(
            "[{}] {}: {}",
            evidence.evidence_id,
            step.check.tool_name,
            summary
        );

        self.evidence.push(evidence.clone());
        Some(evidence)
    }

    /// Skip a step
    pub fn skip_step(&mut self, step_index: usize, reason: &str) {
        if step_index < self.steps.len() {
            self.steps[step_index].status = StepStatus::Skipped;
            self.steps[step_index].dialogue = format!("Skipped: {}", reason);
        }
    }

    /// Run diagnosis on collected evidence
    pub fn diagnose(&self, doctor: &dyn Doctor) -> DiagnosisResult {
        doctor.diagnose(&self.evidence)
    }

    /// Build the final flow result
    pub fn build_result(
        &self,
        doctor: &dyn Doctor,
        selection: &DoctorSelection,
        user_request: &str,
        diagnosis: DiagnosisResult,
    ) -> DoctorFlowResult {
        let completed_at = Utc::now();
        let total_duration_ms = (completed_at - self.start_time).num_milliseconds() as u64;

        let report = self.runner.build_report(
            doctor,
            user_request,
            self.evidence.clone(),
            diagnosis.clone(),
            self.start_time,
        );

        DoctorFlowResult {
            flow_id: generate_request_id(),
            doctor_id: doctor.id().to_string(),
            doctor_name: doctor.name().to_string(),
            domain: doctor.domain(),
            user_request: user_request.to_string(),
            selection_score: selection.confidence,
            selection_reason: selection.reasoning.clone(),
            steps: self.steps.clone(),
            evidence: self.evidence.clone(),
            diagnosis,
            report,
            started_at: self.start_time,
            completed_at,
            total_duration_ms,
        }
    }

    /// Get evidence IDs collected so far
    pub fn evidence_ids(&self) -> Vec<String> {
        self.evidence.iter().map(|e| e.evidence_id.clone()).collect()
    }

    /// Get count of successful steps
    pub fn successful_steps(&self) -> usize {
        self.steps.iter().filter(|s| s.status == StepStatus::Success).count()
    }
}

/// Case file for a doctor run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorCaseFile {
    pub case_id: String,
    pub doctor_id: String,
    pub doctor_name: String,
    pub domain: DoctorDomain,
    pub user_request: String,
    pub selection_score: u32,
    pub selection_reason: String,
    pub steps_executed: Vec<CaseStep>,
    pub evidence_ids: Vec<String>,
    pub findings_count: usize,
    pub most_likely_cause: Option<String>,
    pub confidence: u32,
    pub reliability_score: u32,
    pub suggested_actions: Vec<CaseSuggestedAction>,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseStep {
    pub step_number: u32,
    pub check_id: String,
    pub tool_name: String,
    pub status: String,
    pub evidence_id: Option<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseSuggestedAction {
    pub id: String,
    pub description: String,
    pub risk: String,
    pub executed: bool,
}

impl DoctorCaseFile {
    /// Create from flow result
    pub fn from_flow_result(result: &DoctorFlowResult, reliability_score: u32) -> Self {
        Self {
            case_id: result.flow_id.clone(),
            doctor_id: result.doctor_id.clone(),
            doctor_name: result.doctor_name.clone(),
            domain: result.domain.clone(),
            user_request: result.user_request.clone(),
            selection_score: result.selection_score,
            selection_reason: result.selection_reason.clone(),
            steps_executed: result
                .steps
                .iter()
                .map(|s| CaseStep {
                    step_number: s.step_number,
                    check_id: s.check.id.clone(),
                    tool_name: s.check.tool_name.clone(),
                    status: format!("{:?}", s.status),
                    evidence_id: s.evidence.as_ref().map(|e| e.evidence_id.clone()),
                    duration_ms: s.duration_ms,
                })
                .collect(),
            evidence_ids: result.evidence.iter().map(|e| e.evidence_id.clone()).collect(),
            findings_count: result.diagnosis.findings.len(),
            most_likely_cause: result.diagnosis.most_likely_cause.clone(),
            confidence: result.diagnosis.confidence,
            reliability_score,
            suggested_actions: result
                .diagnosis
                .proposed_actions
                .iter()
                .map(|a| CaseSuggestedAction {
                    id: a.id.clone(),
                    description: a.description.clone(),
                    risk: format!("{}", a.risk),
                    executed: false,
                })
                .collect(),
            started_at: result.started_at,
            completed_at: result.completed_at,
            duration_ms: result.total_duration_ms,
        }
    }

    /// Save to case file directory
    pub fn save(&self) -> anyhow::Result<String> {
        let cases_dir = std::path::Path::new("/var/lib/anna/cases");
        std::fs::create_dir_all(cases_dir)?;

        let case_path = cases_dir.join(format!("{}", self.case_id));
        std::fs::create_dir_all(&case_path)?;

        let file_path = case_path.join("doctor.json");
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&file_path, json)?;

        Ok(file_path.to_string_lossy().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_problem_phrases() {
        // Problem queries should be detected
        assert!(detect_problem_phrase("wifi keeps disconnecting").0);
        assert!(detect_problem_phrase("no sound from speakers").0);
        assert!(detect_problem_phrase("my system has slow boot").0);
        assert!(detect_problem_phrase("bluetooth is not working").0);
        // Normal queries should not be detected as problems
        assert!(!detect_problem_phrase("what CPU do I have").0);
        assert!(!detect_problem_phrase("how much disk space").0);
    }
}
