//! LLM Protocol v0.15.0
//!
//! Junior LLM-A / Senior LLM-B architecture with:
//! - Dynamic checks instead of hardcoded probes
//! - Interactive user questions through annactl
//! - Risk classification and approval flow
//! - Measured vs user-asserted facts
//! - Detailed reasoning traces

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============================================================================
// Protocol Version
// ============================================================================

pub const PROTOCOL_VERSION: &str = "0.15.0";

// ============================================================================
// Check Risk Classification
// ============================================================================

/// Risk level for dynamic checks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckRisk {
    /// Safe read-only command with no side effects
    ReadOnlyLow,
    /// Read-only but may be slow or access sensitive data
    ReadOnlyMedium,
    /// Low-risk write (e.g., creating backup, writing to user config)
    WriteLow,
    /// Medium-risk write (e.g., modifying config files)
    WriteMedium,
    /// High-risk write (e.g., system config, package operations)
    WriteHigh,
}

impl CheckRisk {
    pub fn as_str(&self) -> &'static str {
        match self {
            CheckRisk::ReadOnlyLow => "read_only_low",
            CheckRisk::ReadOnlyMedium => "read_only_medium",
            CheckRisk::WriteLow => "write_low",
            CheckRisk::WriteMedium => "write_medium",
            CheckRisk::WriteHigh => "write_high",
        }
    }

    pub fn is_read_only(&self) -> bool {
        matches!(self, CheckRisk::ReadOnlyLow | CheckRisk::ReadOnlyMedium)
    }

    pub fn requires_user_confirm(&self) -> bool {
        matches!(
            self,
            CheckRisk::WriteMedium | CheckRisk::WriteHigh | CheckRisk::ReadOnlyMedium
        )
    }

    pub fn is_high_risk(&self) -> bool {
        matches!(self, CheckRisk::WriteHigh)
    }
}

// ============================================================================
// Dynamic Check Definition
// ============================================================================

/// A learned or proposed dynamic check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicCheck {
    /// Internal identifier (UUID)
    pub id: String,
    /// Human-readable name (e.g., "vim_installed_check")
    pub name: String,
    /// Shell command to execute
    pub command: String,
    /// Optional working directory
    #[serde(default)]
    pub working_dir: Option<String>,
    /// Risk classification
    pub risk: CheckRisk,
    /// Description of what this check does and why
    pub description: String,
    /// When this check was first created
    pub created_at: DateTime<Utc>,
    /// Last time this check was used
    #[serde(default)]
    pub last_used_at: Option<DateTime<Utc>>,
    /// Last exit code from running this check
    #[serde(default)]
    pub last_status: Option<i32>,
    /// Sample of last output (truncated)
    #[serde(default)]
    pub last_output_sample: Option<String>,
    /// Tags for filtering (e.g., ["vim", "editor", "package_check"])
    #[serde(default)]
    pub tags: Vec<String>,
}

impl DynamicCheck {
    pub fn new(name: String, command: String, risk: CheckRisk, description: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            command,
            working_dir: None,
            risk,
            description,
            created_at: Utc::now(),
            last_used_at: None,
            last_status: None,
            last_output_sample: None,
            tags: vec![],
        }
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn record_execution(&mut self, exit_code: i32, output_sample: Option<String>) {
        self.last_used_at = Some(Utc::now());
        self.last_status = Some(exit_code);
        self.last_output_sample = output_sample;
    }
}

// ============================================================================
// Core Probes (Minimal Set)
// ============================================================================

/// Core probe identifier - minimal universal set
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoreProbeId {
    /// CPU info from lscpu -J
    CpuInfo,
    /// Memory info from /proc/meminfo
    MemInfo,
    /// Block device layout from lsblk -J
    DiskLayout,
    /// Root filesystem usage from df -P /
    FsUsageRoot,
    /// Network links from ip -j link show
    NetLinks,
    /// Network addresses from ip -j addr
    NetAddr,
    /// DNS config from /etc/resolv.conf
    DnsResolv,
}

impl CoreProbeId {
    pub fn as_str(&self) -> &'static str {
        match self {
            CoreProbeId::CpuInfo => "core.cpu_info",
            CoreProbeId::MemInfo => "core.mem_info",
            CoreProbeId::DiskLayout => "core.disk_layout",
            CoreProbeId::FsUsageRoot => "core.fs_usage_root",
            CoreProbeId::NetLinks => "core.net_links",
            CoreProbeId::NetAddr => "core.net_addr",
            CoreProbeId::DnsResolv => "core.dns_resolv",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            CoreProbeId::CpuInfo => "CPU information (model, cores, flags) from lscpu",
            CoreProbeId::MemInfo => "Memory usage from /proc/meminfo",
            CoreProbeId::DiskLayout => "Block device layout from lsblk",
            CoreProbeId::FsUsageRoot => "Root filesystem usage from df",
            CoreProbeId::NetLinks => "Network interface list from ip link",
            CoreProbeId::NetAddr => "Network addresses from ip addr",
            CoreProbeId::DnsResolv => "DNS configuration from /etc/resolv.conf",
        }
    }

    pub fn command(&self) -> &'static str {
        match self {
            CoreProbeId::CpuInfo => "lscpu -J",
            CoreProbeId::MemInfo => "cat /proc/meminfo",
            CoreProbeId::DiskLayout => "lsblk -J -b -o NAME,SIZE,TYPE,FSTYPE,MOUNTPOINT",
            CoreProbeId::FsUsageRoot => "df -P /",
            CoreProbeId::NetLinks => "ip -j link show",
            CoreProbeId::NetAddr => "ip -j addr",
            CoreProbeId::DnsResolv => "cat /etc/resolv.conf",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            CoreProbeId::CpuInfo,
            CoreProbeId::MemInfo,
            CoreProbeId::DiskLayout,
            CoreProbeId::FsUsageRoot,
            CoreProbeId::NetLinks,
            CoreProbeId::NetAddr,
            CoreProbeId::DnsResolv,
        ]
    }
}

// ============================================================================
// User Question (Interactive Flow)
// ============================================================================

/// Style of user question
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestionStyle {
    /// User picks exactly one option
    SingleChoice,
    /// User picks one or more options
    MultiChoice,
    /// User types free text (no options)
    FreeText,
}

/// An option in a user question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionOption {
    /// Short identifier (e.g., "vim", "nano")
    pub id: String,
    /// Display label (e.g., "Vim or Neovim")
    pub label: String,
}

/// A structured question for the user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserQuestion {
    /// Why this question is being asked
    pub reason: String,
    /// Question style
    pub style: QuestionStyle,
    /// The question text
    pub question: String,
    /// Available options (for single_choice or multi_choice)
    #[serde(default)]
    pub options: Vec<QuestionOption>,
    /// Allow free text even with options?
    #[serde(default)]
    pub allow_free_text: bool,
}

impl UserQuestion {
    pub fn single_choice(question: &str, reason: &str, options: Vec<QuestionOption>) -> Self {
        Self {
            reason: reason.to_string(),
            style: QuestionStyle::SingleChoice,
            question: question.to_string(),
            options,
            allow_free_text: false,
        }
    }

    pub fn multi_choice(question: &str, reason: &str, options: Vec<QuestionOption>) -> Self {
        Self {
            reason: reason.to_string(),
            style: QuestionStyle::MultiChoice,
            question: question.to_string(),
            options,
            allow_free_text: false,
        }
    }

    pub fn free_text(question: &str, reason: &str) -> Self {
        Self {
            reason: reason.to_string(),
            style: QuestionStyle::FreeText,
            question: question.to_string(),
            options: vec![],
            allow_free_text: true,
        }
    }

    pub fn with_free_text_fallback(mut self) -> Self {
        self.allow_free_text = true;
        self
    }
}

/// User's response to a question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAnswer {
    /// Original question ID or hash
    pub question_ref: String,
    /// Selected option ID(s) or free text
    pub answer: UserAnswerValue,
    /// Timestamp of response
    pub answered_at: DateTime<Utc>,
}

/// Value of user's answer
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UserAnswerValue {
    /// Single option selected
    Single(String),
    /// Multiple options selected
    Multiple(Vec<String>),
    /// Free text response
    Text(String),
}

// ============================================================================
// Fact Source Types
// ============================================================================

/// Source of a fact - measured vs user-asserted
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactSource {
    /// Confirmed by command output on this machine
    Measured,
    /// Asserted by the user (may need verification)
    UserAsserted,
    /// Inferred by LLM from other facts
    Inferred,
}

impl FactSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            FactSource::Measured => "measured",
            FactSource::UserAsserted => "user_asserted",
            FactSource::Inferred => "inferred",
        }
    }

    pub fn trust_level(&self) -> f64 {
        match self {
            FactSource::Measured => 1.0,
            FactSource::UserAsserted => 0.7,
            FactSource::Inferred => 0.5,
        }
    }
}

/// A fact with source tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedFact {
    /// Entity (e.g., "editor:vim", "location:vim.config")
    pub entity: String,
    /// Attribute (e.g., "installed", "path")
    pub attribute: String,
    /// Value
    pub value: String,
    /// Source type
    pub source: FactSource,
    /// Confidence [0.0, 1.0]
    pub confidence: f64,
    /// When this was last verified
    pub last_verified: Option<DateTime<Utc>>,
    /// Optional tags for filtering
    #[serde(default)]
    pub tags: Vec<String>,
}

// ============================================================================
// LLM-A (Junior Planner) Protocol
// ============================================================================

/// Intent category detected by LLM-A
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Intent {
    EditorConfig,
    PackageInstall,
    PackageRemove,
    NetworkDebug,
    StorageCleanup,
    MetaStatus,
    MetaHelp,
    SystemInfo,
    ConfigLocation,
    ServiceManagement,
    Other(String),
}

/// A check request from LLM-A
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CheckRequest {
    /// Run a core probe
    #[serde(rename = "core_probe")]
    CoreProbe { probe_id: String, reason: String },
    /// Reuse an existing dynamic check
    #[serde(rename = "reuse_check")]
    ReuseCheck { check_id: String, reason: String },
    /// Propose a new dynamic check
    #[serde(rename = "new_check")]
    NewCheck {
        name: String,
        command: String,
        risk: CheckRisk,
        reason: String,
        tags: Vec<String>,
    },
}

/// Request from engine to LLM-A
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmARequestV15 {
    /// Protocol version
    pub version: String,
    /// User's request text
    pub user_request: String,
    /// Available core probes
    pub core_probes: Vec<CoreProbeInfo>,
    /// Existing dynamic checks relevant to this request
    #[serde(default)]
    pub available_checks: Vec<DynamicCheck>,
    /// Known facts from knowledge store
    #[serde(default)]
    pub known_facts: Vec<TrackedFact>,
    /// System mode (dev or normal)
    pub mode: String,
    /// Current iteration (1-based)
    pub iteration: usize,
    /// Mentor feedback from LLM-B (if retry)
    #[serde(default)]
    pub mentor_feedback: Option<String>,
    /// Evidence collected so far
    #[serde(default)]
    pub evidence: Vec<CheckResult>,
}

/// Core probe info for LLM context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreProbeInfo {
    pub id: String,
    pub description: String,
}

/// Response from LLM-A (strict JSON)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmAResponseV15 {
    /// Detected intent category
    pub intent: String,
    /// Ordered conceptual plan steps
    pub plan_steps: Vec<String>,
    /// Checks requested (core probes, reuse, or new)
    #[serde(default)]
    pub check_requests: Vec<CheckRequest>,
    /// Optional question for the user if info needed
    #[serde(default)]
    pub user_question: Option<UserQuestion>,
    /// Draft answer (may be incomplete)
    #[serde(default)]
    pub draft_answer: Option<String>,
    /// Safety notes: what could go wrong
    #[serde(default)]
    pub safety_notes: Vec<String>,
    /// Does LLM-A need mentor review?
    pub needs_mentor: bool,
    /// Why mentor review is needed
    #[serde(default)]
    pub needs_mentor_reason: Option<String>,
    /// Self-assessed confidence [0.0, 1.0]
    pub self_confidence: f64,
}

// ============================================================================
// LLM-B (Senior Reviewer) Protocol
// ============================================================================

/// Verdict from LLM-B
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LlmBVerdict {
    /// Plan and answer are safe and correct
    Accept,
    /// B corrects the answer and accepts
    FixAndAccept,
    /// More checks needed before conclusion
    NeedsMoreChecks,
    /// A should retry with feedback
    MentorRetry,
    /// Request cannot be satisfied
    Refuse,
}

impl LlmBVerdict {
    pub fn as_str(&self) -> &'static str {
        match self {
            LlmBVerdict::Accept => "accept",
            LlmBVerdict::FixAndAccept => "fix_and_accept",
            LlmBVerdict::NeedsMoreChecks => "needs_more_checks",
            LlmBVerdict::MentorRetry => "mentor_retry",
            LlmBVerdict::Refuse => "refuse",
        }
    }
}

/// Check approval status from LLM-B
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckApproval {
    /// Safe to run now
    AllowNow,
    /// Run after user confirms
    AllowAfterUserConfirm,
    /// Do not run
    Deny,
}

/// Per-check risk evaluation from LLM-B
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckRiskEval {
    /// Check ID or name
    pub check_ref: String,
    /// Updated risk rating
    pub risk: CheckRisk,
    /// Approval decision
    pub approval: CheckApproval,
    /// Explanation
    pub explanation: String,
}

/// Suggested learning update from LLM-B
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LearningUpdate {
    /// Store a new dynamic check
    #[serde(rename = "store_check")]
    StoreCheck { check: DynamicCheck },
    /// Store a known location
    #[serde(rename = "store_location")]
    StoreLocation {
        entity: String,
        path: String,
        description: String,
    },
    /// Store a measured fact
    #[serde(rename = "store_fact")]
    StoreFact {
        entity: String,
        attribute: String,
        value: String,
        source: FactSource,
    },
    /// Invalidate a stale fact
    #[serde(rename = "invalidate_fact")]
    InvalidateFact { entity: String, reason: String },
}

/// Request from engine to LLM-B
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmBRequestV15 {
    /// Protocol version
    pub version: String,
    /// Original user request
    pub user_request: String,
    /// Full LLM-A output
    pub llm_a_output: LlmAResponseV15,
    /// Results from checks already executed
    #[serde(default)]
    pub check_results: Vec<CheckResult>,
    /// Known facts including user facts
    #[serde(default)]
    pub known_facts: Vec<TrackedFact>,
    /// System mode
    pub mode: String,
}

/// Result of executing a check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    /// Check ID or core probe ID
    pub check_id: String,
    /// Command that was run
    pub command: String,
    /// Exit code
    pub exit_code: i32,
    /// Stdout (truncated if large)
    #[serde(default)]
    pub stdout: Option<String>,
    /// Stderr (truncated if large)
    #[serde(default)]
    pub stderr: Option<String>,
    /// Execution timestamp
    pub executed_at: DateTime<Utc>,
}

/// Response from LLM-B (strict JSON)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmBResponseV15 {
    /// Final verdict
    pub verdict: LlmBVerdict,
    /// Per-check risk evaluation
    #[serde(default)]
    pub risk_evaluation: Vec<CheckRiskEval>,
    /// Approved checks with their approval status
    #[serde(default)]
    pub approved_checks: Vec<CheckRiskEval>,
    /// Corrected answer if fix_and_accept
    #[serde(default)]
    pub corrected_answer: Option<String>,
    /// Feedback for LLM-A if mentor_retry
    #[serde(default)]
    pub mentor_feedback: Option<String>,
    /// Numeric score for LLM-A [0.0, 1.0]
    #[serde(default)]
    pub mentor_score: Option<f64>,
    /// Suggested learning updates
    #[serde(default)]
    pub learning_updates: Vec<LearningUpdate>,
    /// Question for user if B needs more info
    #[serde(default)]
    pub user_question: Option<UserQuestion>,
    /// Problems identified
    #[serde(default)]
    pub problems: Vec<String>,
    /// Final confidence score [0.0, 1.0]
    pub confidence: f64,
}

// ============================================================================
// Reasoning Trace (for debugging)
// ============================================================================

/// A step in the reasoning trace
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "step_type")]
pub enum TraceStep {
    /// User initiated request
    #[serde(rename = "user_request")]
    UserRequest {
        text: String,
        timestamp: DateTime<Utc>,
    },
    /// LLM-A planning output
    #[serde(rename = "llm_a_plan")]
    LlmAPlan {
        response: LlmAResponseV15,
        timestamp: DateTime<Utc>,
    },
    /// Check execution
    #[serde(rename = "check_executed")]
    CheckExecuted {
        result: CheckResult,
        approved_by: String,
    },
    /// User question asked
    #[serde(rename = "user_question")]
    UserQuestionAsked {
        question: UserQuestion,
        timestamp: DateTime<Utc>,
    },
    /// User answer received
    #[serde(rename = "user_answer")]
    UserAnswerReceived {
        answer: UserAnswer,
        stored_as_fact: bool,
    },
    /// LLM-B evaluation
    #[serde(rename = "llm_b_eval")]
    LlmBEval {
        response: LlmBResponseV15,
        timestamp: DateTime<Utc>,
    },
    /// Final answer delivered
    #[serde(rename = "final_answer")]
    FinalAnswer {
        answer: String,
        confidence: f64,
        timestamp: DateTime<Utc>,
    },
}

/// Complete reasoning trace for a request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningTrace {
    /// Unique trace ID
    pub trace_id: String,
    /// All steps in order
    pub steps: Vec<TraceStep>,
    /// Start time
    pub started_at: DateTime<Utc>,
    /// End time
    #[serde(default)]
    pub ended_at: Option<DateTime<Utc>>,
    /// Number of A/B iterations
    pub iterations: usize,
}

impl ReasoningTrace {
    pub fn new() -> Self {
        Self {
            trace_id: uuid::Uuid::new_v4().to_string(),
            steps: vec![],
            started_at: Utc::now(),
            ended_at: None,
            iterations: 0,
        }
    }

    pub fn add_step(&mut self, step: TraceStep) {
        self.steps.push(step);
    }

    pub fn finish(&mut self) {
        self.ended_at = Some(Utc::now());
    }
}

impl Default for ReasoningTrace {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_risk() {
        assert!(CheckRisk::ReadOnlyLow.is_read_only());
        assert!(!CheckRisk::WriteMedium.is_read_only());
        assert!(CheckRisk::WriteHigh.requires_user_confirm());
        assert!(CheckRisk::WriteHigh.is_high_risk());
    }

    #[test]
    fn test_core_probes() {
        let probes = CoreProbeId::all();
        assert_eq!(probes.len(), 7);
        assert_eq!(CoreProbeId::CpuInfo.as_str(), "core.cpu_info");
    }

    #[test]
    fn test_user_question() {
        let q = UserQuestion::single_choice(
            "Which editor do you use?",
            "need to know for config location",
            vec![
                QuestionOption {
                    id: "vim".to_string(),
                    label: "Vim".to_string(),
                },
                QuestionOption {
                    id: "nano".to_string(),
                    label: "Nano".to_string(),
                },
            ],
        );
        assert_eq!(q.style, QuestionStyle::SingleChoice);
        assert_eq!(q.options.len(), 2);
    }

    #[test]
    fn test_fact_source() {
        assert_eq!(FactSource::Measured.trust_level(), 1.0);
        assert!(FactSource::UserAsserted.trust_level() < FactSource::Measured.trust_level());
    }

    #[test]
    fn test_dynamic_check() {
        let mut check = DynamicCheck::new(
            "vim_installed".to_string(),
            "pacman -Qi vim".to_string(),
            CheckRisk::ReadOnlyLow,
            "Check if vim is installed".to_string(),
        );
        check.record_execution(0, Some("Name: vim\nVersion: 9.0".to_string()));
        assert_eq!(check.last_status, Some(0));
    }

    #[test]
    fn test_llm_b_verdict() {
        assert_eq!(LlmBVerdict::Accept.as_str(), "accept");
        assert_eq!(LlmBVerdict::MentorRetry.as_str(), "mentor_retry");
    }

    #[test]
    fn test_check_request_serialization() {
        let req = CheckRequest::NewCheck {
            name: "test_check".to_string(),
            command: "echo test".to_string(),
            risk: CheckRisk::ReadOnlyLow,
            reason: "testing".to_string(),
            tags: vec!["test".to_string()],
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("new_check"));
    }

    #[test]
    fn test_reasoning_trace() {
        let mut trace = ReasoningTrace::new();
        trace.add_step(TraceStep::UserRequest {
            text: "test".to_string(),
            timestamp: Utc::now(),
        });
        trace.finish();
        assert!(trace.ended_at.is_some());
    }
}
