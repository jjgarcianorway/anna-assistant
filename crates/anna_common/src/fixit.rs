//! Fix-It Mode v0.0.34
//!
//! Bounded troubleshooting loop with state machine for multi-step problem resolution.
//!
//! States:
//! 1. Understand - Extract problem statement, constraints, risk tolerance
//! 2. Evidence - Collect baseline evidence via read-only tools
//! 3. Hypothesize - Generate 1-3 hypotheses with evidence citations
//! 4. Test - Run specific read-only checks per hypothesis
//! 5. PlanFix - Produce the smallest fix plan possible
//! 6. ApplyFix - Execute mutations if confirmed
//! 7. Verify - Post-check and confirm resolution
//! 8. Close - Summarize, store recipe if reliable, save case

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::generate_request_id;

/// Maximum hypothesis cycles before giving up
pub const MAX_HYPOTHESIS_CYCLES: usize = 2;

/// Maximum tools per evidence collection phase
pub const MAX_TOOLS_PER_PHASE: usize = 5;

/// Maximum mutations in a change set
pub const MAX_MUTATIONS_PER_BATCH: usize = 5;

/// Fix-It confirmation phrase
pub const FIX_CONFIRMATION: &str = "I CONFIRM (apply fix)";

/// Fix-It Mode States
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FixItState {
    /// Initial state - extract problem statement
    Understand,
    /// Collect baseline evidence
    Evidence,
    /// Generate hypotheses from evidence
    Hypothesize,
    /// Test specific hypotheses
    Test,
    /// Plan the fix
    PlanFix,
    /// Apply the fix (requires confirmation)
    ApplyFix,
    /// Verify the fix worked
    Verify,
    /// Close and summarize
    Close,
    /// Stuck - cannot proceed
    Stuck,
    /// Completed successfully
    Completed,
    /// Failed - could not resolve
    Failed,
}

impl std::fmt::Display for FixItState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FixItState::Understand => write!(f, "understand"),
            FixItState::Evidence => write!(f, "evidence"),
            FixItState::Hypothesize => write!(f, "hypothesize"),
            FixItState::Test => write!(f, "test"),
            FixItState::PlanFix => write!(f, "plan_fix"),
            FixItState::ApplyFix => write!(f, "apply_fix"),
            FixItState::Verify => write!(f, "verify"),
            FixItState::Close => write!(f, "close"),
            FixItState::Stuck => write!(f, "stuck"),
            FixItState::Completed => write!(f, "completed"),
            FixItState::Failed => write!(f, "failed"),
        }
    }
}

/// Problem category for tool bundle selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProblemCategory {
    Networking,
    Audio,
    Performance,
    SystemdService,
    Storage,
    Graphics,
    Boot,
    Unknown,
}

impl ProblemCategory {
    /// Detect category from problem description
    pub fn detect(problem: &str) -> Self {
        let lower = problem.to_lowercase();

        if lower.contains("wifi") || lower.contains("network") || lower.contains("internet")
            || lower.contains("ethernet") || lower.contains("disconnect") || lower.contains("connection")
        {
            ProblemCategory::Networking
        } else if lower.contains("sound") || lower.contains("audio") || lower.contains("speaker")
            || lower.contains("headphone") || lower.contains("volume") || lower.contains("pulseaudio")
            || lower.contains("pipewire")
        {
            ProblemCategory::Audio
        } else if lower.contains("slow") || lower.contains("performance") || lower.contains("lag")
            || lower.contains("freeze") || lower.contains("cpu") || lower.contains("memory")
            || lower.contains("ram")
        {
            ProblemCategory::Performance
        } else if lower.contains("service") || lower.contains("systemd") || lower.contains("won't start")
            || lower.contains("failed") || lower.contains("restart")
        {
            ProblemCategory::SystemdService
        } else if lower.contains("disk") || lower.contains("storage") || lower.contains("mount")
            || lower.contains("full") || lower.contains("space")
        {
            ProblemCategory::Storage
        } else if lower.contains("display") || lower.contains("screen") || lower.contains("gpu")
            || lower.contains("graphics") || lower.contains("resolution")
        {
            ProblemCategory::Graphics
        } else if lower.contains("boot") || lower.contains("startup") || lower.contains("grub") {
            ProblemCategory::Boot
        } else {
            ProblemCategory::Unknown
        }
    }

    /// Get tool bundle for this category
    pub fn get_tool_bundle(&self) -> Vec<&'static str> {
        match self {
            ProblemCategory::Networking => vec![
                "hw_snapshot_summary",
                "service_status(name=NetworkManager)",
                "journal_warnings(service=NetworkManager, minutes=30)",
                "journal_warnings(service=wpa_supplicant, minutes=30)",
            ],
            ProblemCategory::Audio => vec![
                "hw_snapshot_summary",
                "service_status(name=pipewire)",
                "service_status(name=pulseaudio)",
                "journal_warnings(service=pipewire, minutes=30)",
            ],
            ProblemCategory::Performance => vec![
                "hw_snapshot_summary",
                "top_resource_processes(window_minutes=5)",
                "slowness_hypotheses(days=3)",
                "what_changed(days=3)",
            ],
            ProblemCategory::SystemdService => vec![
                "sw_snapshot_summary",
                "journal_warnings(minutes=60)",
            ],
            ProblemCategory::Storage => vec![
                "disk_usage",
                "hw_snapshot_summary",
            ],
            ProblemCategory::Graphics => vec![
                "hw_snapshot_summary",
                "journal_warnings(service=Xorg, minutes=30)",
            ],
            ProblemCategory::Boot => vec![
                "boot_time_trend(days=7)",
                "journal_warnings(minutes=120)",
                "what_changed(days=7)",
            ],
            ProblemCategory::Unknown => vec![
                "hw_snapshot_summary",
                "sw_snapshot_summary",
                "journal_warnings(minutes=30)",
            ],
        }
    }
}

/// A hypothesis about the problem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypothesis {
    pub id: String,
    pub description: String,
    pub evidence_refs: Vec<String>,
    pub confidence: u8,
    pub test_tools: Vec<String>,
    pub test_result: Option<HypothesisTestResult>,
}

/// Result of testing a hypothesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypothesisTestResult {
    pub confirmed: bool,
    pub evidence_refs: Vec<String>,
    pub explanation: String,
}

/// Risk level for changes (simplified from pipeline)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FixItRiskLevel {
    ReadOnly,
    Low,
    Medium,
    High,
}

impl std::fmt::Display for FixItRiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FixItRiskLevel::ReadOnly => write!(f, "read_only"),
            FixItRiskLevel::Low => write!(f, "low"),
            FixItRiskLevel::Medium => write!(f, "medium"),
            FixItRiskLevel::High => write!(f, "high"),
        }
    }
}

/// A single change in a change set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeItem {
    pub id: String,
    pub what: String,
    pub why: String,
    pub risk: FixItRiskLevel,
    pub rollback_action: String,
    pub post_check: String,
}

/// A batch of changes to apply together
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSet {
    pub id: String,
    pub changes: Vec<ChangeItem>,
    pub confirmation_required: bool,
    pub applied: bool,
    pub rolled_back: bool,
    pub results: Vec<ChangeResult>,
}

impl Default for ChangeSet {
    fn default() -> Self {
        Self::new()
    }
}

impl ChangeSet {
    pub fn new() -> Self {
        Self {
            id: generate_request_id(),
            changes: Vec::new(),
            confirmation_required: true,
            applied: false,
            rolled_back: false,
            results: Vec::new(),
        }
    }

    pub fn add_change(&mut self, change: ChangeItem) -> Result<(), &'static str> {
        if self.changes.len() >= MAX_MUTATIONS_PER_BATCH {
            return Err("Maximum mutations per batch exceeded");
        }
        self.changes.push(change);
        Ok(())
    }

    pub fn format_for_confirmation(&self) -> String {
        let mut lines = Vec::new();
        lines.push("╭─────────────────────────────────────────────────────────────────╮".to_string());
        lines.push(format!("│ Change Set: {} ({} changes)", self.id, self.changes.len()));
        lines.push("├─────────────────────────────────────────────────────────────────┤".to_string());

        for (i, change) in self.changes.iter().enumerate() {
            lines.push(format!("│ {}. {}", i + 1, change.what));
            lines.push(format!("│    Why: {}", change.why));
            lines.push(format!("│    Risk: {}", change.risk));
            lines.push(format!("│    Rollback: {}", change.rollback_action));
            if i < self.changes.len() - 1 {
                lines.push("│".to_string());
            }
        }

        lines.push("├─────────────────────────────────────────────────────────────────┤".to_string());
        lines.push(format!("│ To apply, type: {}", FIX_CONFIRMATION));
        lines.push("╰─────────────────────────────────────────────────────────────────╯".to_string());

        lines.join("\n")
    }
}

/// Result of applying a single change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeResult {
    pub change_id: String,
    pub success: bool,
    pub error: Option<String>,
    pub post_check_passed: bool,
    pub rolled_back: bool,
}

/// State transition record for fix timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    pub from: FixItState,
    pub to: FixItState,
    pub timestamp: DateTime<Utc>,
    pub evidence_ids: Vec<String>,
    pub decision: String,
}

/// Fix-It session tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixItSession {
    pub request_id: String,
    pub problem_statement: String,
    pub category: ProblemCategory,
    pub current_state: FixItState,
    pub hypothesis_cycles: usize,
    pub hypotheses: Vec<Hypothesis>,
    pub selected_hypothesis: Option<usize>,
    pub change_set: Option<ChangeSet>,
    pub evidence_ids: Vec<String>,
    pub timeline: Vec<StateTransition>,
    pub stuck_reason: Option<String>,
    pub resolution_summary: Option<String>,
}

impl FixItSession {
    pub fn new(request_id: &str, problem_statement: &str) -> Self {
        let category = ProblemCategory::detect(problem_statement);
        Self {
            request_id: request_id.to_string(),
            problem_statement: problem_statement.to_string(),
            category,
            current_state: FixItState::Understand,
            hypothesis_cycles: 0,
            hypotheses: Vec::new(),
            selected_hypothesis: None,
            change_set: None,
            evidence_ids: Vec::new(),
            timeline: Vec::new(),
            stuck_reason: None,
            resolution_summary: None,
        }
    }

    /// Record a state transition
    pub fn transition(&mut self, to: FixItState, evidence_ids: Vec<String>, decision: &str) {
        let transition = StateTransition {
            from: self.current_state,
            to,
            timestamp: Utc::now(),
            evidence_ids,
            decision: decision.to_string(),
        };
        self.timeline.push(transition);
        self.current_state = to;
    }

    /// Check if we've exceeded hypothesis cycles
    pub fn can_hypothesize(&self) -> bool {
        self.hypothesis_cycles < MAX_HYPOTHESIS_CYCLES
    }

    /// Increment hypothesis cycle
    pub fn next_cycle(&mut self) {
        self.hypothesis_cycles += 1;
    }

    /// Mark as stuck
    pub fn mark_stuck(&mut self, reason: &str) {
        self.stuck_reason = Some(reason.to_string());
        self.transition(FixItState::Stuck, vec![], reason);
    }

    /// Generate fix timeline JSON
    pub fn to_fix_timeline_json(&self) -> String {
        serde_json::to_string_pretty(&FixTimeline {
            request_id: self.request_id.clone(),
            problem_statement: self.problem_statement.clone(),
            category: format!("{:?}", self.category),
            hypothesis_cycles: self.hypothesis_cycles,
            hypotheses: self.hypotheses.clone(),
            selected_hypothesis: self.selected_hypothesis,
            change_set_id: self.change_set.as_ref().map(|cs| cs.id.clone()),
            final_state: format!("{}", self.current_state),
            stuck_reason: self.stuck_reason.clone(),
            resolution_summary: self.resolution_summary.clone(),
            transitions: self.timeline.clone(),
        })
        .unwrap_or_else(|_| "{}".to_string())
    }
}

/// Fix timeline structure for JSON serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixTimeline {
    pub request_id: String,
    pub problem_statement: String,
    pub category: String,
    pub hypothesis_cycles: usize,
    pub hypotheses: Vec<Hypothesis>,
    pub selected_hypothesis: Option<usize>,
    pub change_set_id: Option<String>,
    pub final_state: String,
    pub stuck_reason: Option<String>,
    pub resolution_summary: Option<String>,
    pub transitions: Vec<StateTransition>,
}

/// Check if a request is a Fix-It request
pub fn is_fixit_request(request: &str) -> bool {
    let lower = request.to_lowercase();

    let fix_patterns = [
        "fix my", "fix the", "repair", "troubleshoot", "debug",
        "not working", "won't work", "doesn't work", "broken",
        "keeps disconnecting", "keeps crashing", "keeps failing",
        "is slow", "is slower", "is broken", "is failing",
        "won't start", "can't connect", "cannot connect",
        "help me fix", "something wrong", "having issues",
        "having problems", "having trouble",
    ];

    fix_patterns.iter().any(|p| lower.contains(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_fixit_request() {
        assert!(is_fixit_request("WiFi keeps disconnecting"));
        assert!(is_fixit_request("My computer is slower"));
        assert!(is_fixit_request("Sound is broken"));
        assert!(is_fixit_request("My service won't start"));
        assert!(is_fixit_request("Fix my network"));

        assert!(!is_fixit_request("What CPU do I have?"));
        assert!(!is_fixit_request("Install nginx"));
        assert!(!is_fixit_request("Show me disk usage"));
    }

    #[test]
    fn test_problem_category_detection() {
        assert_eq!(ProblemCategory::detect("WiFi keeps disconnecting"), ProblemCategory::Networking);
        assert_eq!(ProblemCategory::detect("No sound from speakers"), ProblemCategory::Audio);
        assert_eq!(ProblemCategory::detect("System is very slow"), ProblemCategory::Performance);
        assert_eq!(ProblemCategory::detect("nginx service won't start"), ProblemCategory::SystemdService);
        assert_eq!(ProblemCategory::detect("Disk is full"), ProblemCategory::Storage);
    }

    #[test]
    fn test_fixit_session_creation() {
        let session = FixItSession::new("test-123", "WiFi keeps disconnecting");
        assert_eq!(session.current_state, FixItState::Understand);
        assert_eq!(session.category, ProblemCategory::Networking);
        assert_eq!(session.hypothesis_cycles, 0);
    }

    #[test]
    fn test_fixit_session_transitions() {
        let mut session = FixItSession::new("test-123", "WiFi issue");
        session.transition(FixItState::Evidence, vec!["E1".to_string()], "Starting evidence collection");

        assert_eq!(session.current_state, FixItState::Evidence);
        assert_eq!(session.timeline.len(), 1);
        assert_eq!(session.timeline[0].from, FixItState::Understand);
        assert_eq!(session.timeline[0].to, FixItState::Evidence);
    }

    #[test]
    fn test_hypothesis_cycle_limit() {
        let mut session = FixItSession::new("test-123", "Some problem");
        assert!(session.can_hypothesize());

        session.next_cycle();
        assert!(session.can_hypothesize());

        session.next_cycle();
        assert!(!session.can_hypothesize());
    }

    #[test]
    fn test_change_set_limit() {
        let mut cs = ChangeSet::new();
        for i in 0..MAX_MUTATIONS_PER_BATCH {
            let result = cs.add_change(ChangeItem {
                id: format!("C{}", i),
                what: "test".to_string(),
                why: "test".to_string(),
                risk: FixItRiskLevel::Low,
                rollback_action: "test".to_string(),
                post_check: "test".to_string(),
            });
            assert!(result.is_ok());
        }

        // Should fail on the 6th
        let result = cs.add_change(ChangeItem {
            id: "C6".to_string(),
            what: "test".to_string(),
            why: "test".to_string(),
            risk: FixItRiskLevel::Low,
            rollback_action: "test".to_string(),
            post_check: "test".to_string(),
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_tool_bundles() {
        let bundle = ProblemCategory::Networking.get_tool_bundle();
        assert!(bundle.contains(&"hw_snapshot_summary"));
        assert!(bundle.iter().any(|t| t.contains("NetworkManager")));

        let bundle = ProblemCategory::Audio.get_tool_bundle();
        assert!(bundle.iter().any(|t| t.contains("pipewire") || t.contains("pulseaudio")));
    }

    #[test]
    fn test_change_set_confirmation_format() {
        let mut cs = ChangeSet::new();
        cs.add_change(ChangeItem {
            id: "C1".to_string(),
            what: "Restart NetworkManager".to_string(),
            why: "Reset network state".to_string(),
            risk: FixItRiskLevel::Low,
            rollback_action: "Stop NetworkManager".to_string(),
            post_check: "Check network connectivity".to_string(),
        }).unwrap();

        let formatted = cs.format_for_confirmation();
        assert!(formatted.contains("Restart NetworkManager"));
        assert!(formatted.contains(FIX_CONFIRMATION));
    }
}
