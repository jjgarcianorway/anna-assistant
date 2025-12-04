//! Boot Doctor - Slow boot diagnosis with "what changed" correlation
//!
//! v0.0.41: Arch Boot Doctor v1
//!
//! Deterministic diagnosis flow:
//! 1. Establish current boot time summary (systemd-analyze time)
//! 2. Identify top offenders (blame + critical chain)
//! 3. Check if offenders are new/regressed (compare with baseline)
//! 4. Correlate with changes (packages, services, configs)
//! 5. Produce findings (facts) vs hypotheses (clearly labeled)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

// ============================================================================
// Boot Health Types
// ============================================================================

/// Boot health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BootHealth {
    /// Boot time within normal range
    Healthy,
    /// Boot slower than baseline but functional
    Degraded,
    /// Severely slow or boot issues detected
    Broken,
    /// Cannot determine status (no baseline)
    Unknown,
}

impl std::fmt::Display for BootHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BootHealth::Healthy => write!(f, "Healthy"),
            BootHealth::Degraded => write!(f, "Degraded"),
            BootHealth::Broken => write!(f, "Broken"),
            BootHealth::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Risk level for findings and playbooks
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Informational only
    Info,
    /// Low risk, easily reversible
    Low,
    /// Medium risk, requires confirmation
    Medium,
    /// High risk, significant changes
    High,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Info => write!(f, "Info"),
            RiskLevel::Low => write!(f, "Low"),
            RiskLevel::Medium => write!(f, "Medium"),
            RiskLevel::High => write!(f, "High"),
        }
    }
}

// ============================================================================
// Boot Timing Types
// ============================================================================

/// Boot time breakdown from systemd-analyze
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootTiming {
    /// Firmware time (if available)
    pub firmware_ms: Option<u64>,
    /// Bootloader time (if available)
    pub loader_ms: Option<u64>,
    /// Kernel time
    pub kernel_ms: u64,
    /// Initrd time
    pub initrd_ms: u64,
    /// Userspace time
    pub userspace_ms: u64,
    /// Total boot time
    pub total_ms: u64,
    /// Graphical target reached time (if applicable)
    pub graphical_ms: Option<u64>,
}

impl BootTiming {
    /// Check if boot is considered slow (> 30 seconds userspace)
    pub fn is_slow(&self) -> bool {
        self.userspace_ms > 30_000
    }

    /// Check if boot is very slow (> 60 seconds userspace)
    pub fn is_very_slow(&self) -> bool {
        self.userspace_ms > 60_000
    }

    /// Format as human-readable string
    pub fn to_summary(&self) -> String {
        let mut parts = vec![];
        if let Some(fw) = self.firmware_ms {
            parts.push(format!("firmware {}ms", fw));
        }
        if let Some(loader) = self.loader_ms {
            parts.push(format!("loader {}ms", loader));
        }
        parts.push(format!("kernel {}ms", self.kernel_ms));
        parts.push(format!("initrd {}ms", self.initrd_ms));
        parts.push(format!("userspace {}ms", self.userspace_ms));
        format!("Total: {}ms ({})", self.total_ms, parts.join(" + "))
    }
}

/// A service/unit that delays boot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootOffender {
    /// Unit name (e.g., "NetworkManager-wait-online.service")
    pub unit: String,
    /// Time taken in milliseconds
    pub time_ms: u64,
    /// Whether this is a new offender (not in baseline)
    pub is_new: bool,
    /// Whether this has regressed (slower than baseline)
    pub is_regressed: bool,
    /// Previous time if regressed
    pub previous_ms: Option<u64>,
    /// Correlation with changes (package name, etc.)
    pub correlation: Option<String>,
    /// Is this a critical chain item
    pub in_critical_chain: bool,
}

impl BootOffender {
    /// Calculate regression percentage
    pub fn regression_percent(&self) -> Option<f64> {
        self.previous_ms.map(|prev| {
            if prev > 0 {
                ((self.time_ms as f64 - prev as f64) / prev as f64) * 100.0
            } else {
                0.0
            }
        })
    }
}

/// A change that might correlate with boot issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeEvent {
    /// When the change occurred
    pub timestamp: DateTime<Utc>,
    /// Type of change
    pub change_type: ChangeType,
    /// Description of the change
    pub description: String,
    /// Related package (if applicable)
    pub package: Option<String>,
    /// Related unit (if applicable)
    pub unit: Option<String>,
    /// Evidence ID
    pub evidence_id: String,
}

/// Types of changes tracked
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    /// Package installed
    PackageInstall,
    /// Package updated
    PackageUpdate,
    /// Package removed
    PackageRemove,
    /// Service enabled
    ServiceEnabled,
    /// Service disabled
    ServiceDisabled,
    /// Config file changed
    ConfigChange,
    /// Kernel update
    KernelUpdate,
    /// Anna update
    AnnaUpdate,
}

impl std::fmt::Display for ChangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeType::PackageInstall => write!(f, "Package Install"),
            ChangeType::PackageUpdate => write!(f, "Package Update"),
            ChangeType::PackageRemove => write!(f, "Package Remove"),
            ChangeType::ServiceEnabled => write!(f, "Service Enabled"),
            ChangeType::ServiceDisabled => write!(f, "Service Disabled"),
            ChangeType::ConfigChange => write!(f, "Config Change"),
            ChangeType::KernelUpdate => write!(f, "Kernel Update"),
            ChangeType::AnnaUpdate => write!(f, "Anna Update"),
        }
    }
}

// ============================================================================
// Boot Evidence
// ============================================================================

/// Complete evidence bundle for boot diagnosis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootEvidence {
    /// Collected timestamp
    pub collected_at: DateTime<Utc>,
    /// Current boot timing
    pub timing: BootTiming,
    /// Top offenders from blame (sorted by time)
    pub blame: Vec<BootOffender>,
    /// Critical chain items
    pub critical_chain: Vec<BootOffender>,
    /// Enabled units snapshot
    pub enabled_units: Vec<String>,
    /// Recent journal warnings/errors during boot
    pub journal_boot_errors: Vec<JournalEntry>,
    /// Changes in the last N days
    pub recent_changes: Vec<ChangeEvent>,
    /// Anna telemetry boot time trend (if available)
    pub boot_trend: Option<BootTrend>,
    /// Baseline for comparison (if available)
    pub baseline: Option<BootBaseline>,
    /// Correlation lookback days
    pub lookback_days: u32,
    /// Evidence ID prefix
    pub evidence_id: String,
}

impl BootEvidence {
    /// Determine overall boot health
    pub fn health(&self) -> BootHealth {
        // Check for severe issues
        if self.timing.is_very_slow() {
            return BootHealth::Broken;
        }

        // Check for regression against baseline
        if let Some(baseline) = &self.baseline {
            let regression_pct = self.timing.userspace_ms as f64 / baseline.userspace_ms as f64;
            if regression_pct > 2.0 {
                return BootHealth::Broken;
            }
            if regression_pct > 1.5 {
                return BootHealth::Degraded;
            }
        }

        // Check for slow boot without baseline
        if self.timing.is_slow() {
            return BootHealth::Degraded;
        }

        // Check for new offenders
        let new_offenders = self.blame.iter().filter(|o| o.is_new).count();
        if new_offenders > 0 {
            return BootHealth::Degraded;
        }

        BootHealth::Healthy
    }
}

/// Journal entry during boot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Priority (err, warning, etc.)
    pub priority: String,
    /// Unit/identifier
    pub unit: String,
    /// Message
    pub message: String,
}

/// Boot time trend from telemetry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootTrend {
    /// Average boot time over last N boots
    pub avg_ms: u64,
    /// Min boot time
    pub min_ms: u64,
    /// Max boot time
    pub max_ms: u64,
    /// Number of samples
    pub samples: u32,
    /// Trend direction
    pub trend: TrendDirection,
}

/// Trend direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Stable,
    Degrading,
    Unknown,
}

impl std::fmt::Display for TrendDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrendDirection::Improving => write!(f, "Improving"),
            TrendDirection::Stable => write!(f, "Stable"),
            TrendDirection::Degrading => write!(f, "Degrading"),
            TrendDirection::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Baseline for comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootBaseline {
    /// Recorded timestamp
    pub recorded_at: DateTime<Utc>,
    /// Baseline userspace time
    pub userspace_ms: u64,
    /// Baseline total time
    pub total_ms: u64,
    /// Top offenders at baseline
    pub top_offenders: Vec<(String, u64)>,
}

// ============================================================================
// Diagnosis Types
// ============================================================================

/// Result of a diagnosis step
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepResult {
    Pass,
    Fail,
    Partial,
    Skipped,
}

impl std::fmt::Display for StepResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StepResult::Pass => write!(f, "PASS"),
            StepResult::Fail => write!(f, "FAIL"),
            StepResult::Partial => write!(f, "PARTIAL"),
            StepResult::Skipped => write!(f, "SKIPPED"),
        }
    }
}

/// A single diagnosis step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisStep {
    /// Step name
    pub name: String,
    /// Step description
    pub description: String,
    /// Step number (1-5)
    pub step_number: u32,
    /// Result
    pub result: StepResult,
    /// Detailed findings
    pub details: String,
    /// What this means for the user
    pub implication: String,
    /// Evidence IDs cited
    pub evidence_ids: Vec<String>,
}

/// A finding (fact) from diagnosis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Finding ID
    pub id: String,
    /// What was found
    pub description: String,
    /// Risk level
    pub risk: RiskLevel,
    /// Related unit (if applicable)
    pub unit: Option<String>,
    /// Evidence IDs
    pub evidence_ids: Vec<String>,
}

/// A hypothesis about the cause
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootHypothesis {
    /// Hypothesis ID
    pub id: String,
    /// One-line summary
    pub summary: String,
    /// Detailed explanation
    pub explanation: String,
    /// Confidence (0-100)
    pub confidence: u32,
    /// Supporting evidence IDs
    pub supporting_evidence: Vec<String>,
    /// Suggested playbook to test/fix
    pub suggested_playbook: Option<String>,
    /// Correlated change (if any)
    pub correlated_change: Option<String>,
}

/// Complete diagnosis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisResult {
    /// Overall health
    pub health: BootHealth,
    /// Diagnosis steps
    pub steps: Vec<DiagnosisStep>,
    /// Findings (facts)
    pub findings: Vec<Finding>,
    /// Hypotheses (theories)
    pub hypotheses: Vec<BootHypothesis>,
    /// Summary text
    pub summary: String,
    /// Recommended playbooks
    pub recommended_playbooks: Vec<String>,
}

// ============================================================================
// Playbook Types
// ============================================================================

/// Confirmation phrase for fixes
pub const FIX_CONFIRMATION: &str = "I CONFIRM";

/// Type of playbook
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybookType {
    /// Restart a service
    RestartService,
    /// Reload a service
    ReloadService,
    /// Disable a service
    DisableService,
    /// Reduce timeout value
    ReduceTimeout,
    /// Remove conflict
    RemoveConflict,
}

/// A command in a playbook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookCommand {
    /// The command to run
    pub command: String,
    /// Human description
    pub description: String,
    /// Run as target user (vs root)
    pub as_user: bool,
    /// Timeout in seconds
    pub timeout_secs: u32,
}

/// Preflight check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightCheck {
    /// Check name
    pub name: String,
    /// Command to run
    pub command: String,
    /// Expected output pattern (regex)
    pub expected_pattern: Option<String>,
    /// Error message if check fails
    pub error_message: String,
}

/// Post-execution check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostCheck {
    /// Check name
    pub name: String,
    /// Command to run
    pub command: String,
    /// Expected output pattern (regex)
    pub expected_pattern: Option<String>,
    /// Wait time before check (seconds)
    pub wait_secs: u32,
    /// What to verify after reboot
    pub verify_on_reboot: Option<String>,
}

/// A fix playbook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixPlaybook {
    /// Playbook ID
    pub id: String,
    /// Type
    pub playbook_type: PlaybookType,
    /// Human name
    pub name: String,
    /// Description
    pub description: String,
    /// Risk level
    pub risk: RiskLevel,
    /// Target user (for user-scoped operations)
    pub target_user: Option<String>,
    /// Preflight checks
    pub preflight: Vec<PreflightCheck>,
    /// Commands to execute
    pub commands: Vec<PlaybookCommand>,
    /// Post-checks
    pub post_checks: Vec<PostCheck>,
    /// Rollback commands
    pub rollback: Vec<PlaybookCommand>,
    /// Required confirmation phrase
    pub confirmation_phrase: String,
    /// Which hypothesis this addresses
    pub addresses_hypothesis: Option<String>,
    /// Policy blocked (if true, show warning)
    pub policy_blocked: bool,
    /// Policy block reason
    pub policy_block_reason: Option<String>,
    /// Verification pending note (for post-reboot)
    pub verification_pending: Option<String>,
}

/// Result of playbook execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookResult {
    /// Playbook ID
    pub playbook_id: String,
    /// Success
    pub success: bool,
    /// Commands executed
    pub commands_executed: Vec<CommandResult>,
    /// Post-checks passed
    pub post_checks_passed: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Verification pending note
    pub verification_pending: Option<String>,
    /// Suggested reliability (capped until verified)
    pub reliability: u32,
}

/// Result of a command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    /// Command
    pub command: String,
    /// Exit code
    pub exit_code: i32,
    /// Stdout
    pub stdout: String,
    /// Stderr
    pub stderr: String,
    /// Duration ms
    pub duration_ms: u64,
}

/// Result of a check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    /// Check name
    pub name: String,
    /// Passed
    pub passed: bool,
    /// Output
    pub output: String,
}

// ============================================================================
// Recipe Capture
// ============================================================================

/// Request to capture a recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeCaptureRequest {
    /// Suggested recipe name
    pub name: String,
    /// Problem description
    pub problem: String,
    /// Solution description
    pub solution: String,
    /// Preconditions
    pub preconditions: Vec<String>,
    /// Playbook that worked
    pub playbook_id: String,
    /// Reliability achieved
    pub reliability: u32,
    /// Evidence patterns for matching
    pub evidence_patterns: Vec<String>,
}

// ============================================================================
// Case File
// ============================================================================

/// Boot doctor case file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootDoctorCase {
    /// Case ID
    pub case_id: String,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated
    pub updated_at: DateTime<Utc>,
    /// Status
    pub status: CaseStatus,
    /// Evidence bundle
    pub evidence: BootEvidence,
    /// Diagnosis result
    pub diagnosis: Option<DiagnosisResult>,
    /// Applied playbook
    pub applied_playbook: Option<PlaybookResult>,
    /// Recipe capture request
    pub recipe_capture: Option<RecipeCaptureRequest>,
    /// Notes
    pub notes: Vec<CaseNote>,
    /// Verification pending items
    pub verification_pending: Vec<String>,
}

/// Case status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaseStatus {
    /// Evidence collected
    EvidenceCollected,
    /// Diagnosed
    Diagnosed,
    /// Fix applied
    FixApplied,
    /// Verified (after reboot)
    Verified,
    /// Closed
    Closed,
}

/// Case note
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseNote {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Note text
    pub note: String,
}

impl BootDoctorCase {
    /// Create new case
    pub fn new(evidence: BootEvidence) -> Self {
        let case_id = format!("boot-{}", Utc::now().format("%Y%m%d-%H%M%S"));
        Self {
            case_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: CaseStatus::EvidenceCollected,
            evidence,
            diagnosis: None,
            applied_playbook: None,
            recipe_capture: None,
            notes: vec![],
            verification_pending: vec![],
        }
    }

    /// Add diagnosis
    pub fn add_diagnosis(&mut self, diagnosis: DiagnosisResult) {
        self.diagnosis = Some(diagnosis);
        self.status = CaseStatus::Diagnosed;
        self.updated_at = Utc::now();
    }

    /// Add playbook result
    pub fn add_playbook_result(&mut self, result: PlaybookResult) {
        if let Some(ref pending) = result.verification_pending {
            self.verification_pending.push(pending.clone());
        }
        self.applied_playbook = Some(result);
        self.status = CaseStatus::FixApplied;
        self.updated_at = Utc::now();
    }

    /// Add recipe capture request
    pub fn add_recipe_capture(&mut self, request: RecipeCaptureRequest) {
        self.recipe_capture = Some(request);
        self.updated_at = Utc::now();
    }

    /// Add note
    pub fn add_note(&mut self, note: &str) {
        self.notes.push(CaseNote {
            timestamp: Utc::now(),
            note: note.to_string(),
        });
        self.updated_at = Utc::now();
    }
}

// ============================================================================
// Boot Doctor Engine
// ============================================================================

/// Boot Doctor - diagnoses slow boot and correlates with changes
pub struct BootDoctor {
    /// Default lookback days for correlation
    lookback_days: u32,
    /// Max offenders to analyze
    max_offenders: usize,
    /// Max hypotheses to generate
    max_hypotheses: usize,
}

impl BootDoctor {
    /// Create new Boot Doctor
    pub fn new() -> Self {
        Self {
            lookback_days: 14,
            max_offenders: 20,
            max_hypotheses: 3,
        }
    }

    /// Create with custom settings
    pub fn with_settings(lookback_days: u32, max_offenders: usize, max_hypotheses: usize) -> Self {
        Self {
            lookback_days,
            max_offenders,
            max_hypotheses,
        }
    }

    /// Collect boot evidence
    pub fn collect_evidence(&self, baseline: Option<BootBaseline>) -> BootEvidence {
        let evidence_id = format!("ev-boot-{}", Utc::now().timestamp());

        // Collect timing
        let timing = self.collect_timing();

        // Collect blame
        let blame = self.collect_blame(&baseline);

        // Collect critical chain
        let critical_chain = self.collect_critical_chain(&baseline);

        // Collect enabled units
        let enabled_units = self.collect_enabled_units();

        // Collect journal boot errors
        let journal_boot_errors = self.collect_journal_boot_errors();

        // Collect recent changes
        let recent_changes = self.collect_recent_changes(self.lookback_days);

        // Collect boot trend from telemetry
        let boot_trend = self.collect_boot_trend();

        BootEvidence {
            collected_at: Utc::now(),
            timing,
            blame,
            critical_chain,
            enabled_units,
            journal_boot_errors,
            recent_changes,
            boot_trend,
            baseline,
            lookback_days: self.lookback_days,
            evidence_id,
        }
    }

    /// Collect timing from systemd-analyze
    fn collect_timing(&self) -> BootTiming {
        let output = Command::new("systemd-analyze").arg("time").output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                self.parse_timing(&stdout)
            }
            Err(_) => BootTiming {
                firmware_ms: None,
                loader_ms: None,
                kernel_ms: 0,
                initrd_ms: 0,
                userspace_ms: 0,
                total_ms: 0,
                graphical_ms: None,
            },
        }
    }

    /// Parse systemd-analyze time output
    fn parse_timing(&self, output: &str) -> BootTiming {
        let mut firmware_ms = None;
        let mut loader_ms = None;
        let mut kernel_ms = 0u64;
        let mut initrd_ms = 0u64;
        let mut userspace_ms = 0u64;
        let mut graphical_ms = None;

        // Parse output like:
        // Startup finished in 2.134s (firmware) + 1.234s (loader) + 1.982s (kernel) + 3.456s (initrd) + 25.678s (userspace) = 34.484s
        // or without firmware/loader on VMs

        for segment in output.split('+') {
            let segment = segment.trim();
            if let Some(time) = self.extract_time_ms(segment) {
                if segment.contains("firmware") {
                    firmware_ms = Some(time);
                } else if segment.contains("loader") {
                    loader_ms = Some(time);
                } else if segment.contains("kernel") {
                    kernel_ms = time;
                } else if segment.contains("initrd") {
                    initrd_ms = time;
                } else if segment.contains("userspace") {
                    userspace_ms = time;
                }
            }
        }

        // Check for graphical.target
        if output.contains("graphical.target") {
            if let Some(time) = self.extract_time_ms(output) {
                graphical_ms = Some(time);
            }
        }

        let total_ms = firmware_ms.unwrap_or(0)
            + loader_ms.unwrap_or(0)
            + kernel_ms
            + initrd_ms
            + userspace_ms;

        BootTiming {
            firmware_ms,
            loader_ms,
            kernel_ms,
            initrd_ms,
            userspace_ms,
            total_ms,
            graphical_ms,
        }
    }

    /// Extract time in milliseconds from a string like "2.134s" or "234ms"
    fn extract_time_ms(&self, s: &str) -> Option<u64> {
        // Look for patterns like "2.134s" or "234ms"
        let re_sec = regex::Regex::new(r"(\d+\.?\d*)s").ok()?;
        let re_ms = regex::Regex::new(r"(\d+)ms").ok()?;
        let re_min = regex::Regex::new(r"(\d+)min").ok()?;

        if let Some(caps) = re_min.captures(s) {
            let mins: f64 = caps[1].parse().ok()?;
            return Some((mins * 60_000.0) as u64);
        }

        if let Some(caps) = re_sec.captures(s) {
            let secs: f64 = caps[1].parse().ok()?;
            return Some((secs * 1000.0) as u64);
        }

        if let Some(caps) = re_ms.captures(s) {
            let ms: u64 = caps[1].parse().ok()?;
            return Some(ms);
        }

        None
    }

    /// Collect blame (top offenders)
    fn collect_blame(&self, baseline: &Option<BootBaseline>) -> Vec<BootOffender> {
        let output = Command::new("systemd-analyze").arg("blame").output();

        let baseline_map: HashMap<String, u64> = baseline
            .as_ref()
            .map(|b| b.top_offenders.iter().cloned().collect())
            .unwrap_or_default();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                self.parse_blame(&stdout, &baseline_map)
                    .into_iter()
                    .take(self.max_offenders)
                    .collect()
            }
            Err(_) => vec![],
        }
    }

    /// Parse blame output
    fn parse_blame(&self, output: &str, baseline: &HashMap<String, u64>) -> Vec<BootOffender> {
        let mut offenders = vec![];

        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Lines like: "25.456s NetworkManager-wait-online.service"
            let parts: Vec<&str> = line.splitn(2, char::is_whitespace).collect();
            if parts.len() != 2 {
                continue;
            }

            let time_str = parts[0].trim();
            let unit = parts[1].trim().to_string();

            if let Some(time_ms) = self.extract_time_ms(time_str) {
                let previous_ms = baseline.get(&unit).copied();
                let is_new = previous_ms.is_none() && !baseline.is_empty();
                let is_regressed = previous_ms.map(|p| time_ms > p + 1000).unwrap_or(false);

                offenders.push(BootOffender {
                    unit,
                    time_ms,
                    is_new,
                    is_regressed,
                    previous_ms,
                    correlation: None,
                    in_critical_chain: false,
                });
            }
        }

        offenders
    }

    /// Collect critical chain
    fn collect_critical_chain(&self, baseline: &Option<BootBaseline>) -> Vec<BootOffender> {
        let output = Command::new("systemd-analyze")
            .arg("critical-chain")
            .output();

        let baseline_map: HashMap<String, u64> = baseline
            .as_ref()
            .map(|b| b.top_offenders.iter().cloned().collect())
            .unwrap_or_default();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                self.parse_critical_chain(&stdout, &baseline_map)
                    .into_iter()
                    .take(10)
                    .collect()
            }
            Err(_) => vec![],
        }
    }

    /// Parse critical chain output
    fn parse_critical_chain(
        &self,
        output: &str,
        baseline: &HashMap<String, u64>,
    ) -> Vec<BootOffender> {
        let mut offenders = vec![];

        // Critical chain format is more complex with tree structure
        // We'll extract units and their times
        let re = regex::Regex::new(r"@(\d+\.?\d*)s\s+\+?(\d+\.?\d*)?s?\s*$").ok();

        for line in output.lines() {
            // Extract unit name (before @)
            if let Some(at_pos) = line.find('@') {
                let unit_part = line[..at_pos].trim();
                // Get the actual unit name (last part after any tree characters)
                let unit = unit_part
                    .chars()
                    .skip_while(|c| !c.is_alphanumeric())
                    .collect::<String>();

                if unit.is_empty() || !unit.ends_with(".service") && !unit.ends_with(".target") {
                    continue;
                }

                // Extract time
                if let Some(ref re) = re {
                    if let Some(caps) = re.captures(line) {
                        let time_ms = caps
                            .get(2)
                            .and_then(|m| self.extract_time_ms(&format!("{}s", m.as_str())))
                            .or_else(|| {
                                caps.get(1)
                                    .and_then(|m| self.extract_time_ms(&format!("{}s", m.as_str())))
                            })
                            .unwrap_or(0);

                        if time_ms > 0 {
                            let previous_ms = baseline.get(&unit).copied();
                            let is_new = previous_ms.is_none() && !baseline.is_empty();
                            let is_regressed =
                                previous_ms.map(|p| time_ms > p + 1000).unwrap_or(false);

                            offenders.push(BootOffender {
                                unit,
                                time_ms,
                                is_new,
                                is_regressed,
                                previous_ms,
                                correlation: None,
                                in_critical_chain: true,
                            });
                        }
                    }
                }
            }
        }

        offenders
    }

    /// Collect enabled units
    fn collect_enabled_units(&self) -> Vec<String> {
        let output = Command::new("systemctl")
            .args(["list-unit-files", "--state=enabled", "--no-legend"])
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                stdout
                    .lines()
                    .filter_map(|line| {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        parts.first().map(|s| s.to_string())
                    })
                    .collect()
            }
            Err(_) => vec![],
        }
    }

    /// Collect journal boot errors
    fn collect_journal_boot_errors(&self) -> Vec<JournalEntry> {
        let output = Command::new("journalctl")
            .args(["-b", "-p", "err", "--no-pager", "-n", "50"])
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                self.parse_journal_entries(&stdout)
            }
            Err(_) => vec![],
        }
    }

    /// Parse journal entries
    fn parse_journal_entries(&self, output: &str) -> Vec<JournalEntry> {
        let mut entries = vec![];

        for line in output.lines().take(50) {
            // Simple parsing - journal format varies
            let parts: Vec<&str> = line.splitn(4, ' ').collect();
            if parts.len() >= 4 {
                entries.push(JournalEntry {
                    timestamp: Utc::now(), // Would need proper parsing
                    priority: "err".to_string(),
                    unit: parts.get(2).unwrap_or(&"").to_string(),
                    message: parts.get(3).unwrap_or(&"").to_string(),
                });
            }
        }

        entries
    }

    /// Collect recent changes from pacman log and other sources
    fn collect_recent_changes(&self, days: u32) -> Vec<ChangeEvent> {
        let mut changes = vec![];

        // Parse pacman log for package changes
        changes.extend(self.parse_pacman_log(days));

        // Check for service changes via systemd
        changes.extend(self.collect_service_changes(days));

        // Sort by timestamp descending
        changes.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        changes
    }

    /// Parse pacman log for recent changes
    fn parse_pacman_log(&self, days: u32) -> Vec<ChangeEvent> {
        let mut changes = vec![];

        // Read pacman log
        let log_path = "/var/log/pacman.log";
        let content = std::fs::read_to_string(log_path).unwrap_or_default();

        let cutoff = Utc::now() - chrono::Duration::days(days as i64);
        let evidence_base = format!("ev-pacman-{}", Utc::now().timestamp());
        let mut idx = 0;

        for line in content.lines().rev().take(500) {
            // Lines like: [2024-01-15T10:30:00+0000] [ALPM] installed networkmanager (1.44.0-1)
            if line.contains("[ALPM]") {
                let change_type = if line.contains("installed") {
                    Some(ChangeType::PackageInstall)
                } else if line.contains("upgraded") {
                    Some(ChangeType::PackageUpdate)
                } else if line.contains("removed") {
                    Some(ChangeType::PackageRemove)
                } else {
                    None
                };

                if let Some(ct) = change_type {
                    // Extract package name
                    let package = self.extract_package_from_log_line(line);

                    // Check if kernel update
                    let actual_type = if package
                        .as_ref()
                        .map(|p| p.starts_with("linux"))
                        .unwrap_or(false)
                    {
                        ChangeType::KernelUpdate
                    } else {
                        ct
                    };

                    changes.push(ChangeEvent {
                        timestamp: Utc::now() - chrono::Duration::hours(idx as i64), // Approximation
                        change_type: actual_type,
                        description: line.to_string(),
                        package,
                        unit: None,
                        evidence_id: format!("{}-{}", evidence_base, idx),
                    });
                    idx += 1;
                }
            }
        }

        changes
    }

    /// Extract package name from pacman log line
    fn extract_package_from_log_line(&self, line: &str) -> Option<String> {
        // Format: "... installed/upgraded/removed packagename (version)"
        let parts: Vec<&str> = line.split_whitespace().collect();
        for (i, part) in parts.iter().enumerate() {
            if *part == "installed" || *part == "upgraded" || *part == "removed" {
                return parts.get(i + 1).map(|s| s.to_string());
            }
        }
        None
    }

    /// Collect service enable/disable changes
    fn collect_service_changes(&self, _days: u32) -> Vec<ChangeEvent> {
        // Would need to parse journal for service state changes
        // For now, return empty
        vec![]
    }

    /// Collect boot trend from Anna telemetry
    fn collect_boot_trend(&self) -> Option<BootTrend> {
        // Would read from telemetry DB if available
        // For now, return None
        None
    }

    /// Run diagnosis
    pub fn diagnose(&self, evidence: &BootEvidence) -> DiagnosisResult {
        let mut steps = vec![];
        let mut findings = vec![];

        // Step 1: Boot time summary
        steps.push(self.step_boot_time_summary(evidence));

        // Step 2: Top offenders
        steps.push(self.step_top_offenders(evidence, &mut findings));

        // Step 3: Regression check
        steps.push(self.step_regression_check(evidence, &mut findings));

        // Step 4: Correlation with changes
        steps.push(self.step_correlation(evidence, &mut findings));

        // Step 5: Produce hypotheses
        let hypotheses = self.generate_hypotheses(&findings, evidence);
        steps.push(self.step_hypotheses(&hypotheses));

        // Determine health
        let health = self.determine_health(&steps, evidence);

        // Generate summary
        let summary = self.generate_summary(&health, &hypotheses, evidence);

        // Recommend playbooks
        let recommended_playbooks = self.recommend_playbooks(&hypotheses, evidence);

        DiagnosisResult {
            health,
            steps,
            findings,
            hypotheses,
            summary,
            recommended_playbooks,
        }
    }

    /// Step 1: Boot time summary
    fn step_boot_time_summary(&self, evidence: &BootEvidence) -> DiagnosisStep {
        let timing = &evidence.timing;

        let (result, details, implication) = if timing.is_very_slow() {
            (
                StepResult::Fail,
                format!(
                    "Boot time: {} (userspace: {}ms > 60s threshold)",
                    timing.to_summary(),
                    timing.userspace_ms
                ),
                "Boot is severely slow. Investigation needed.".to_string(),
            )
        } else if timing.is_slow() {
            (
                StepResult::Partial,
                format!(
                    "Boot time: {} (userspace: {}ms > 30s threshold)",
                    timing.to_summary(),
                    timing.userspace_ms
                ),
                "Boot is slower than ideal. Optimization possible.".to_string(),
            )
        } else {
            (
                StepResult::Pass,
                format!("Boot time: {}", timing.to_summary()),
                "Boot time is within normal range.".to_string(),
            )
        };

        DiagnosisStep {
            name: "Boot Time Summary".to_string(),
            description: "Analyze systemd-analyze time output".to_string(),
            step_number: 1,
            result,
            details,
            implication,
            evidence_ids: vec![format!("{}-timing", evidence.evidence_id)],
        }
    }

    /// Step 2: Top offenders
    fn step_top_offenders(
        &self,
        evidence: &BootEvidence,
        findings: &mut Vec<Finding>,
    ) -> DiagnosisStep {
        let slow_threshold_ms = 5000; // 5 seconds
        let slow_offenders: Vec<_> = evidence
            .blame
            .iter()
            .filter(|o| o.time_ms > slow_threshold_ms)
            .collect();

        let (result, details, implication) = if slow_offenders.is_empty() {
            (
                StepResult::Pass,
                "No units taking > 5s to start".to_string(),
                "No obvious slow units.".to_string(),
            )
        } else {
            // Add findings for slow offenders
            for offender in &slow_offenders {
                findings.push(Finding {
                    id: format!("slow-{}", offender.unit.replace('.', "-")),
                    description: format!("{} takes {}ms to start", offender.unit, offender.time_ms),
                    risk: if offender.time_ms > 20000 {
                        RiskLevel::High
                    } else {
                        RiskLevel::Medium
                    },
                    unit: Some(offender.unit.clone()),
                    evidence_ids: vec![format!("{}-blame", evidence.evidence_id)],
                });
            }

            let top3: Vec<String> = slow_offenders
                .iter()
                .take(3)
                .map(|o| format!("{} ({}ms)", o.unit, o.time_ms))
                .collect();

            (
                StepResult::Partial,
                format!(
                    "Found {} slow units: {}",
                    slow_offenders.len(),
                    top3.join(", ")
                ),
                "These units are delaying boot.".to_string(),
            )
        };

        DiagnosisStep {
            name: "Top Offenders".to_string(),
            description: "Identify units taking longest to start".to_string(),
            step_number: 2,
            result,
            details,
            implication,
            evidence_ids: vec![format!("{}-blame", evidence.evidence_id)],
        }
    }

    /// Step 3: Regression check
    fn step_regression_check(
        &self,
        evidence: &BootEvidence,
        findings: &mut Vec<Finding>,
    ) -> DiagnosisStep {
        if evidence.baseline.is_none() {
            return DiagnosisStep {
                name: "Regression Check".to_string(),
                description: "Compare with baseline".to_string(),
                step_number: 3,
                result: StepResult::Skipped,
                details: "No baseline available for comparison".to_string(),
                implication: "Cannot detect regressions without baseline.".to_string(),
                evidence_ids: vec![],
            };
        }

        let baseline = evidence.baseline.as_ref().unwrap();
        let new_offenders: Vec<_> = evidence.blame.iter().filter(|o| o.is_new).collect();
        let regressed_offenders: Vec<_> =
            evidence.blame.iter().filter(|o| o.is_regressed).collect();

        // Add findings
        for offender in &new_offenders {
            findings.push(Finding {
                id: format!("new-{}", offender.unit.replace('.', "-")),
                description: format!(
                    "NEW: {} ({}ms) not in baseline",
                    offender.unit, offender.time_ms
                ),
                risk: RiskLevel::Medium,
                unit: Some(offender.unit.clone()),
                evidence_ids: vec![format!("{}-baseline", evidence.evidence_id)],
            });
        }

        for offender in &regressed_offenders {
            if let Some(pct) = offender.regression_percent() {
                findings.push(Finding {
                    id: format!("regressed-{}", offender.unit.replace('.', "-")),
                    description: format!(
                        "REGRESSED: {} now {}ms (was {}ms, +{:.0}%)",
                        offender.unit,
                        offender.time_ms,
                        offender.previous_ms.unwrap_or(0),
                        pct
                    ),
                    risk: if pct > 100.0 {
                        RiskLevel::High
                    } else {
                        RiskLevel::Medium
                    },
                    unit: Some(offender.unit.clone()),
                    evidence_ids: vec![format!("{}-baseline", evidence.evidence_id)],
                });
            }
        }

        let (result, details, implication) =
            if new_offenders.is_empty() && regressed_offenders.is_empty() {
                (
                    StepResult::Pass,
                    format!(
                        "Boot time stable vs baseline (was {}ms)",
                        baseline.userspace_ms
                    ),
                    "No new slow units or regressions detected.".to_string(),
                )
            } else {
                (
                    StepResult::Fail,
                    format!(
                        "{} new slow units, {} regressed units",
                        new_offenders.len(),
                        regressed_offenders.len()
                    ),
                    "Boot has degraded since baseline.".to_string(),
                )
            };

        DiagnosisStep {
            name: "Regression Check".to_string(),
            description: "Compare with baseline".to_string(),
            step_number: 3,
            result,
            details,
            implication,
            evidence_ids: vec![format!("{}-baseline", evidence.evidence_id)],
        }
    }

    /// Step 4: Correlation with changes
    fn step_correlation(
        &self,
        evidence: &BootEvidence,
        findings: &mut Vec<Finding>,
    ) -> DiagnosisStep {
        if evidence.recent_changes.is_empty() {
            return DiagnosisStep {
                name: "Correlation Analysis".to_string(),
                description: "Correlate with recent changes".to_string(),
                step_number: 4,
                result: StepResult::Skipped,
                details: "No recent changes found in lookback period".to_string(),
                implication: "Cannot correlate issues with changes.".to_string(),
                evidence_ids: vec![],
            };
        }

        let mut correlations = vec![];

        // Try to correlate slow offenders with package changes
        for offender in &evidence.blame {
            if offender.time_ms < 5000 {
                continue;
            }

            // Look for related package
            let unit_base = offender
                .unit
                .trim_end_matches(".service")
                .trim_end_matches(".target");

            for change in &evidence.recent_changes {
                if let Some(pkg) = &change.package {
                    // Check if package name relates to unit
                    if unit_base.to_lowercase().contains(&pkg.to_lowercase())
                        || pkg.to_lowercase().contains(&unit_base.to_lowercase())
                    {
                        correlations.push((offender.unit.clone(), change.clone()));
                        findings.push(Finding {
                            id: format!("corr-{}-{}", offender.unit.replace('.', "-"), pkg),
                            description: format!(
                                "CORRELATION: {} slow after {} {}",
                                offender.unit, change.change_type, pkg
                            ),
                            risk: RiskLevel::Info,
                            unit: Some(offender.unit.clone()),
                            evidence_ids: vec![change.evidence_id.clone()],
                        });
                    }
                }
            }
        }

        // Check for kernel updates
        let kernel_updates: Vec<_> = evidence
            .recent_changes
            .iter()
            .filter(|c| c.change_type == ChangeType::KernelUpdate)
            .collect();

        if !kernel_updates.is_empty() {
            findings.push(Finding {
                id: "kernel-update".to_string(),
                description: format!(
                    "Kernel updated {} time(s) in last {} days",
                    kernel_updates.len(),
                    evidence.lookback_days
                ),
                risk: RiskLevel::Info,
                unit: None,
                evidence_ids: kernel_updates
                    .iter()
                    .map(|c| c.evidence_id.clone())
                    .collect(),
            });
        }

        let (result, details, implication) = if correlations.is_empty() {
            (
                StepResult::Partial,
                format!(
                    "Found {} changes in last {} days, no direct correlations",
                    evidence.recent_changes.len(),
                    evidence.lookback_days
                ),
                "Changes found but no obvious link to slow units.".to_string(),
            )
        } else {
            (
                StepResult::Pass,
                format!(
                    "Found {} correlation(s) between slow units and recent changes",
                    correlations.len()
                ),
                "Potential causes identified.".to_string(),
            )
        };

        DiagnosisStep {
            name: "Correlation Analysis".to_string(),
            description: "Correlate with recent changes".to_string(),
            step_number: 4,
            result,
            details,
            implication,
            evidence_ids: evidence
                .recent_changes
                .iter()
                .map(|c| c.evidence_id.clone())
                .collect(),
        }
    }

    /// Step 5: Generate hypotheses
    fn generate_hypotheses(
        &self,
        findings: &[Finding],
        evidence: &BootEvidence,
    ) -> Vec<BootHypothesis> {
        let mut hypotheses = vec![];

        // Hypothesis: NetworkManager-wait-online is slow
        for offender in &evidence.blame {
            if offender.unit.contains("wait-online") && offender.time_ms > 10000 {
                hypotheses.push(BootHypothesis {
                    id: "wait-online-slow".to_string(),
                    summary: format!("{} is delaying boot", offender.unit),
                    explanation: format!(
                        "The {} unit waits for network to be fully online before proceeding. \
                         This took {}ms. This service can often be disabled if network-dependent \
                         services don't need to start at boot.",
                        offender.unit, offender.time_ms
                    ),
                    confidence: 90,
                    supporting_evidence: vec![format!("{}-blame", evidence.evidence_id)],
                    suggested_playbook: Some("disable_wait_online".to_string()),
                    correlated_change: None,
                });
            }
        }

        // Hypothesis: New service added
        for finding in findings {
            if finding.id.starts_with("new-") {
                if let Some(unit) = &finding.unit {
                    hypotheses.push(BootHypothesis {
                        id: format!("new-service-{}", unit.replace('.', "-")),
                        summary: format!("New service {} is slowing boot", unit),
                        explanation: format!(
                            "The unit {} was not present in the baseline but is now taking \
                             significant time to start. This might be from a recent installation.",
                            unit
                        ),
                        confidence: 85,
                        supporting_evidence: finding.evidence_ids.clone(),
                        suggested_playbook: Some(format!("disable_{}", unit.replace('.', "_"))),
                        correlated_change: None,
                    });
                }
            }
        }

        // Hypothesis: Correlated package update
        for finding in findings {
            if finding.id.starts_with("corr-") {
                hypotheses.push(BootHypothesis {
                    id: finding.id.clone(),
                    summary: finding.description.clone(),
                    explanation: format!(
                        "A package update correlates with this slow unit. \
                         Consider checking if the update changed the service configuration."
                    ),
                    confidence: 75,
                    supporting_evidence: finding.evidence_ids.clone(),
                    suggested_playbook: None,
                    correlated_change: Some(finding.description.clone()),
                });
            }
        }

        // Hypothesis: Service stuck post-boot
        for offender in &evidence.blame {
            if offender.time_ms > 30000 && offender.in_critical_chain {
                hypotheses.push(BootHypothesis {
                    id: format!("stuck-{}", offender.unit.replace('.', "-")),
                    summary: format!("{} may be stuck or timing out", offender.unit),
                    explanation: format!(
                        "The unit {} took {}ms which suggests it may be stuck waiting \
                         for a resource or timing out. Check the unit's logs for details.",
                        offender.unit, offender.time_ms
                    ),
                    confidence: 70,
                    supporting_evidence: vec![format!("{}-chain", evidence.evidence_id)],
                    suggested_playbook: Some(format!(
                        "restart_{}",
                        offender.unit.replace('.', "_")
                    )),
                    correlated_change: None,
                });
            }
        }

        // Sort by confidence and limit
        hypotheses.sort_by(|a, b| b.confidence.cmp(&a.confidence));
        hypotheses.truncate(self.max_hypotheses);
        hypotheses
    }

    /// Step 5 result
    fn step_hypotheses(&self, hypotheses: &[BootHypothesis]) -> DiagnosisStep {
        let (result, details, implication) = if hypotheses.is_empty() {
            (
                StepResult::Pass,
                "No specific hypotheses generated".to_string(),
                "Boot appears normal or causes unclear.".to_string(),
            )
        } else {
            let top = hypotheses.first().unwrap();
            (
                StepResult::Pass,
                format!(
                    "Generated {} hypothesis(es). Top: {} ({}% confidence)",
                    hypotheses.len(),
                    top.summary,
                    top.confidence
                ),
                "Potential causes identified for investigation.".to_string(),
            )
        };

        DiagnosisStep {
            name: "Hypothesis Generation".to_string(),
            description: "Generate theories about causes".to_string(),
            step_number: 5,
            result,
            details,
            implication,
            evidence_ids: vec![],
        }
    }

    /// Determine overall health
    fn determine_health(&self, steps: &[DiagnosisStep], evidence: &BootEvidence) -> BootHealth {
        // Direct health from evidence
        let evidence_health = evidence.health();

        // Check steps
        let fails = steps
            .iter()
            .filter(|s| s.result == StepResult::Fail)
            .count();

        if evidence_health == BootHealth::Broken || fails >= 2 {
            BootHealth::Broken
        } else if evidence_health == BootHealth::Degraded || fails == 1 {
            BootHealth::Degraded
        } else if evidence.baseline.is_none() {
            BootHealth::Unknown
        } else {
            BootHealth::Healthy
        }
    }

    /// Generate summary
    fn generate_summary(
        &self,
        health: &BootHealth,
        hypotheses: &[BootHypothesis],
        evidence: &BootEvidence,
    ) -> String {
        match health {
            BootHealth::Healthy => format!(
                "Boot healthy. Userspace: {}ms.",
                evidence.timing.userspace_ms
            ),
            BootHealth::Degraded => {
                if let Some(h) = hypotheses.first() {
                    format!(
                        "Boot degraded. Likely cause: {} ({}% confidence)",
                        h.summary, h.confidence
                    )
                } else {
                    format!(
                        "Boot slower than ideal ({}ms userspace)",
                        evidence.timing.userspace_ms
                    )
                }
            }
            BootHealth::Broken => {
                if let Some(h) = hypotheses.first() {
                    format!(
                        "Boot severely slow. Top issue: {} ({}% confidence)",
                        h.summary, h.confidence
                    )
                } else {
                    format!(
                        "Boot severely slow ({}ms userspace). Investigation needed.",
                        evidence.timing.userspace_ms
                    )
                }
            }
            BootHealth::Unknown => {
                format!(
                    "Boot time: {}ms. No baseline for comparison.",
                    evidence.timing.userspace_ms
                )
            }
        }
    }

    /// Recommend playbooks based on hypotheses
    fn recommend_playbooks(
        &self,
        hypotheses: &[BootHypothesis],
        _evidence: &BootEvidence,
    ) -> Vec<String> {
        hypotheses
            .iter()
            .filter_map(|h| h.suggested_playbook.clone())
            .collect()
    }

    /// Create a playbook for a given action
    pub fn create_playbook(
        &self,
        playbook_id: &str,
        evidence: &BootEvidence,
        hypothesis_id: Option<&str>,
    ) -> Option<FixPlaybook> {
        match playbook_id {
            "disable_wait_online" => {
                // Find the wait-online unit
                let unit = evidence
                    .blame
                    .iter()
                    .find(|o| o.unit.contains("wait-online"))
                    .map(|o| o.unit.clone())?;

                Some(FixPlaybook {
                    id: "disable_wait_online".to_string(),
                    playbook_type: PlaybookType::DisableService,
                    name: "Disable Network Wait Online".to_string(),
                    description: format!(
                        "Disable {} to speed up boot. Network will still work, \
                        but network-dependent services may start before network is fully ready.",
                        unit
                    ),
                    risk: RiskLevel::Medium,
                    target_user: None,
                    preflight: vec![PreflightCheck {
                        name: "Unit exists".to_string(),
                        command: format!("systemctl cat {}", unit),
                        expected_pattern: Some("\\[Unit\\]".to_string()),
                        error_message: format!("Unit {} not found", unit),
                    }],
                    commands: vec![PlaybookCommand {
                        command: format!("systemctl disable --now {}", unit),
                        description: format!("Disable {}", unit),
                        as_user: false,
                        timeout_secs: 30,
                    }],
                    post_checks: vec![PostCheck {
                        name: "Unit disabled".to_string(),
                        command: format!("systemctl is-enabled {}", unit),
                        expected_pattern: Some("disabled".to_string()),
                        wait_secs: 2,
                        verify_on_reboot: Some("Check boot time with systemd-analyze".to_string()),
                    }],
                    rollback: vec![PlaybookCommand {
                        command: format!("systemctl enable --now {}", unit),
                        description: format!("Re-enable {}", unit),
                        as_user: false,
                        timeout_secs: 30,
                    }],
                    confirmation_phrase: FIX_CONFIRMATION.to_string(),
                    addresses_hypothesis: hypothesis_id.map(String::from),
                    policy_blocked: false,
                    policy_block_reason: None,
                    verification_pending: Some(
                        "Boot time improvement will be verified on next boot".to_string(),
                    ),
                })
            }

            id if id.starts_with("restart_") => {
                let unit = id.strip_prefix("restart_")?.replace('_', ".");

                Some(FixPlaybook {
                    id: id.to_string(),
                    playbook_type: PlaybookType::RestartService,
                    name: format!("Restart {}", unit),
                    description: format!("Restart {} to clear any stuck state.", unit),
                    risk: RiskLevel::Low,
                    target_user: None,
                    preflight: vec![PreflightCheck {
                        name: "Unit exists".to_string(),
                        command: format!("systemctl cat {}", unit),
                        expected_pattern: Some("\\[Unit\\]".to_string()),
                        error_message: format!("Unit {} not found", unit),
                    }],
                    commands: vec![PlaybookCommand {
                        command: format!("systemctl restart {}", unit),
                        description: format!("Restart {}", unit),
                        as_user: false,
                        timeout_secs: 60,
                    }],
                    post_checks: vec![PostCheck {
                        name: "Unit running".to_string(),
                        command: format!("systemctl is-active {}", unit),
                        expected_pattern: Some("active".to_string()),
                        wait_secs: 5,
                        verify_on_reboot: None,
                    }],
                    rollback: vec![],
                    confirmation_phrase: FIX_CONFIRMATION.to_string(),
                    addresses_hypothesis: hypothesis_id.map(String::from),
                    policy_blocked: false,
                    policy_block_reason: None,
                    verification_pending: None,
                })
            }

            id if id.starts_with("disable_") => {
                let unit = id.strip_prefix("disable_")?.replace('_', ".");

                // Check if this is a critical service that should be blocked
                let is_critical = unit.contains("systemd")
                    || unit.contains("dbus")
                    || unit.contains("login")
                    || unit.contains("udev");

                Some(FixPlaybook {
                    id: id.to_string(),
                    playbook_type: PlaybookType::DisableService,
                    name: format!("Disable {}", unit),
                    description: format!(
                        "Disable {} to speed up boot. WARNING: This may affect \
                        system functionality. Only disable if you understand the implications.",
                        unit
                    ),
                    risk: RiskLevel::Medium,
                    target_user: None,
                    preflight: vec![PreflightCheck {
                        name: "Unit exists".to_string(),
                        command: format!("systemctl cat {}", unit),
                        expected_pattern: Some("\\[Unit\\]".to_string()),
                        error_message: format!("Unit {} not found", unit),
                    }],
                    commands: vec![PlaybookCommand {
                        command: format!("systemctl disable --now {}", unit),
                        description: format!("Disable {}", unit),
                        as_user: false,
                        timeout_secs: 30,
                    }],
                    post_checks: vec![PostCheck {
                        name: "Unit disabled".to_string(),
                        command: format!("systemctl is-enabled {}", unit),
                        expected_pattern: Some("disabled".to_string()),
                        wait_secs: 2,
                        verify_on_reboot: Some("Check boot time with systemd-analyze".to_string()),
                    }],
                    rollback: vec![PlaybookCommand {
                        command: format!("systemctl enable --now {}", unit),
                        description: format!("Re-enable {}", unit),
                        as_user: false,
                        timeout_secs: 30,
                    }],
                    confirmation_phrase: FIX_CONFIRMATION.to_string(),
                    addresses_hypothesis: hypothesis_id.map(String::from),
                    policy_blocked: is_critical,
                    policy_block_reason: if is_critical {
                        Some("Critical system service - disabling may break the system".to_string())
                    } else {
                        None
                    },
                    verification_pending: Some(
                        "Boot time improvement will be verified on next boot".to_string(),
                    ),
                })
            }

            _ => None,
        }
    }

    /// Execute a playbook
    pub fn execute_playbook(&self, playbook: &FixPlaybook) -> PlaybookResult {
        // Check if policy blocked
        if playbook.policy_blocked {
            return PlaybookResult {
                playbook_id: playbook.id.clone(),
                success: false,
                commands_executed: vec![],
                post_checks_passed: false,
                error: playbook.policy_block_reason.clone(),
                verification_pending: None,
                reliability: 0,
            };
        }

        let mut commands_executed = vec![];

        // Run preflight checks
        for check in &playbook.preflight {
            let start = std::time::Instant::now();
            let output = Command::new("sh").arg("-c").arg(&check.command).output();

            match output {
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                    let success = if let Some(pattern) = &check.expected_pattern {
                        regex::Regex::new(pattern)
                            .map(|re| re.is_match(&stdout))
                            .unwrap_or(false)
                    } else {
                        out.status.success()
                    };

                    if !success {
                        return PlaybookResult {
                            playbook_id: playbook.id.clone(),
                            success: false,
                            commands_executed,
                            post_checks_passed: false,
                            error: Some(check.error_message.clone()),
                            verification_pending: None,
                            reliability: 0,
                        };
                    }

                    commands_executed.push(CommandResult {
                        command: check.command.clone(),
                        exit_code: out.status.code().unwrap_or(-1),
                        stdout,
                        stderr: String::from_utf8_lossy(&out.stderr).to_string(),
                        duration_ms: start.elapsed().as_millis() as u64,
                    });
                }
                Err(e) => {
                    return PlaybookResult {
                        playbook_id: playbook.id.clone(),
                        success: false,
                        commands_executed,
                        post_checks_passed: false,
                        error: Some(format!("Preflight check failed: {}", e)),
                        verification_pending: None,
                        reliability: 0,
                    };
                }
            }
        }

        // Execute commands
        for cmd in &playbook.commands {
            let start = std::time::Instant::now();
            let output = if cmd.as_user {
                // Would need to run as target user
                Command::new("sh").arg("-c").arg(&cmd.command).output()
            } else {
                Command::new("sh").arg("-c").arg(&cmd.command).output()
            };

            match output {
                Ok(out) => {
                    commands_executed.push(CommandResult {
                        command: cmd.command.clone(),
                        exit_code: out.status.code().unwrap_or(-1),
                        stdout: String::from_utf8_lossy(&out.stdout).to_string(),
                        stderr: String::from_utf8_lossy(&out.stderr).to_string(),
                        duration_ms: start.elapsed().as_millis() as u64,
                    });

                    if !out.status.success() {
                        return PlaybookResult {
                            playbook_id: playbook.id.clone(),
                            success: false,
                            commands_executed,
                            post_checks_passed: false,
                            error: Some(format!("Command failed: {}", cmd.command)),
                            verification_pending: None,
                            reliability: 0,
                        };
                    }
                }
                Err(e) => {
                    return PlaybookResult {
                        playbook_id: playbook.id.clone(),
                        success: false,
                        commands_executed,
                        post_checks_passed: false,
                        error: Some(format!("Command execution failed: {}", e)),
                        verification_pending: None,
                        reliability: 0,
                    };
                }
            }
        }

        // Run post-checks
        let mut post_checks_passed = true;
        for check in &playbook.post_checks {
            if check.wait_secs > 0 {
                std::thread::sleep(std::time::Duration::from_secs(check.wait_secs as u64));
            }

            let output = Command::new("sh").arg("-c").arg(&check.command).output();

            if let Ok(out) = output {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                if let Some(pattern) = &check.expected_pattern {
                    if !regex::Regex::new(pattern)
                        .map(|re| re.is_match(&stdout))
                        .unwrap_or(false)
                    {
                        post_checks_passed = false;
                    }
                }
            } else {
                post_checks_passed = false;
            }
        }

        // Determine reliability (capped if verification pending)
        let reliability = if playbook.verification_pending.is_some() {
            75 // Capped until verified on reboot
        } else if post_checks_passed {
            85
        } else {
            60
        };

        PlaybookResult {
            playbook_id: playbook.id.clone(),
            success: post_checks_passed,
            commands_executed,
            post_checks_passed,
            error: None,
            verification_pending: playbook.verification_pending.clone(),
            reliability,
        }
    }

    /// Create recipe capture request if appropriate
    pub fn maybe_capture_recipe(
        &self,
        playbook: &FixPlaybook,
        result: &PlaybookResult,
        evidence: &BootEvidence,
    ) -> Option<RecipeCaptureRequest> {
        if !result.success || result.reliability < 80 {
            return None;
        }

        // Find related hypothesis
        let problem = if let Some(hyp_id) = &playbook.addresses_hypothesis {
            format!(
                "Boot slow due to {} (hypothesis: {})",
                playbook.name, hyp_id
            )
        } else {
            format!("Boot slow due to {}", playbook.name)
        };

        Some(RecipeCaptureRequest {
            name: format!(
                "Fix: slow boot due to {}",
                playbook.name.to_lowercase().replace(' ', "_")
            ),
            problem,
            solution: playbook.description.clone(),
            preconditions: vec![
                format!("Boot time > 30s userspace"),
                format!("Target unit present"),
            ],
            playbook_id: playbook.id.clone(),
            reliability: result.reliability,
            evidence_patterns: vec![evidence.evidence_id.clone()],
        })
    }
}

impl Default for BootDoctor {
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

    fn create_healthy_evidence() -> BootEvidence {
        BootEvidence {
            collected_at: Utc::now(),
            timing: BootTiming {
                firmware_ms: Some(2000),
                loader_ms: Some(1000),
                kernel_ms: 2000,
                initrd_ms: 1500,
                userspace_ms: 15000, // 15s - healthy
                total_ms: 21500,
                graphical_ms: Some(20000),
            },
            blame: vec![BootOffender {
                unit: "NetworkManager.service".to_string(),
                time_ms: 2500,
                is_new: false,
                is_regressed: false,
                previous_ms: Some(2400),
                correlation: None,
                in_critical_chain: false,
            }],
            critical_chain: vec![],
            enabled_units: vec!["NetworkManager.service".to_string()],
            journal_boot_errors: vec![],
            recent_changes: vec![],
            boot_trend: None,
            baseline: Some(BootBaseline {
                recorded_at: Utc::now() - chrono::Duration::days(7),
                userspace_ms: 14000,
                total_ms: 20000,
                top_offenders: vec![("NetworkManager.service".to_string(), 2400)],
            }),
            lookback_days: 14,
            evidence_id: "ev-test-123".to_string(),
        }
    }

    fn create_slow_evidence() -> BootEvidence {
        BootEvidence {
            collected_at: Utc::now(),
            timing: BootTiming {
                firmware_ms: Some(2000),
                loader_ms: Some(1000),
                kernel_ms: 2000,
                initrd_ms: 1500,
                userspace_ms: 45000, // 45s - slow
                total_ms: 51500,
                graphical_ms: Some(50000),
            },
            blame: vec![
                BootOffender {
                    unit: "NetworkManager-wait-online.service".to_string(),
                    time_ms: 30000,
                    is_new: false,
                    is_regressed: true,
                    previous_ms: Some(5000),
                    correlation: None,
                    in_critical_chain: true,
                },
                BootOffender {
                    unit: "docker.service".to_string(),
                    time_ms: 8000,
                    is_new: true,
                    is_regressed: false,
                    previous_ms: None,
                    correlation: None,
                    in_critical_chain: false,
                },
            ],
            critical_chain: vec![BootOffender {
                unit: "NetworkManager-wait-online.service".to_string(),
                time_ms: 30000,
                is_new: false,
                is_regressed: true,
                previous_ms: Some(5000),
                correlation: None,
                in_critical_chain: true,
            }],
            enabled_units: vec![
                "NetworkManager.service".to_string(),
                "NetworkManager-wait-online.service".to_string(),
                "docker.service".to_string(),
            ],
            journal_boot_errors: vec![],
            recent_changes: vec![ChangeEvent {
                timestamp: Utc::now() - chrono::Duration::days(2),
                change_type: ChangeType::PackageInstall,
                description: "Installed docker".to_string(),
                package: Some("docker".to_string()),
                unit: None,
                evidence_id: "ev-pacman-1".to_string(),
            }],
            boot_trend: None,
            baseline: Some(BootBaseline {
                recorded_at: Utc::now() - chrono::Duration::days(7),
                userspace_ms: 14000,
                total_ms: 20000,
                top_offenders: vec![("NetworkManager-wait-online.service".to_string(), 5000)],
            }),
            lookback_days: 14,
            evidence_id: "ev-test-456".to_string(),
        }
    }

    #[test]
    fn test_boot_health_display() {
        assert_eq!(BootHealth::Healthy.to_string(), "Healthy");
        assert_eq!(BootHealth::Degraded.to_string(), "Degraded");
        assert_eq!(BootHealth::Broken.to_string(), "Broken");
        assert_eq!(BootHealth::Unknown.to_string(), "Unknown");
    }

    #[test]
    fn test_risk_level_display() {
        assert_eq!(RiskLevel::Low.to_string(), "Low");
        assert_eq!(RiskLevel::Medium.to_string(), "Medium");
        assert_eq!(RiskLevel::High.to_string(), "High");
    }

    #[test]
    fn test_timing_is_slow() {
        let fast = BootTiming {
            firmware_ms: None,
            loader_ms: None,
            kernel_ms: 1000,
            initrd_ms: 1000,
            userspace_ms: 15000,
            total_ms: 17000,
            graphical_ms: None,
        };
        assert!(!fast.is_slow());

        let slow = BootTiming {
            firmware_ms: None,
            loader_ms: None,
            kernel_ms: 1000,
            initrd_ms: 1000,
            userspace_ms: 35000,
            total_ms: 37000,
            graphical_ms: None,
        };
        assert!(slow.is_slow());
        assert!(!slow.is_very_slow());

        let very_slow = BootTiming {
            firmware_ms: None,
            loader_ms: None,
            kernel_ms: 1000,
            initrd_ms: 1000,
            userspace_ms: 65000,
            total_ms: 67000,
            graphical_ms: None,
        };
        assert!(very_slow.is_very_slow());
    }

    #[test]
    fn test_evidence_health_healthy() {
        let evidence = create_healthy_evidence();
        assert_eq!(evidence.health(), BootHealth::Healthy);
    }

    #[test]
    fn test_evidence_health_degraded() {
        let mut evidence = create_healthy_evidence();
        evidence.baseline = None; // No baseline, so pure timing check
        evidence.timing.userspace_ms = 35000; // > 30s threshold
        assert_eq!(evidence.health(), BootHealth::Degraded);
    }

    #[test]
    fn test_evidence_health_broken() {
        let mut evidence = create_healthy_evidence();
        evidence.timing.userspace_ms = 70000; // > 60s
        assert_eq!(evidence.health(), BootHealth::Broken);
    }

    #[test]
    fn test_diagnosis_healthy() {
        let doctor = BootDoctor::new();
        let evidence = create_healthy_evidence();
        let result = doctor.diagnose(&evidence);

        assert_eq!(result.health, BootHealth::Healthy);
        assert!(result.hypotheses.is_empty());
    }

    #[test]
    fn test_diagnosis_slow_boot() {
        let doctor = BootDoctor::new();
        let evidence = create_slow_evidence();
        let result = doctor.diagnose(&evidence);

        // 45000ms / 14000ms baseline = 3.2x regression -> Broken
        assert_eq!(result.health, BootHealth::Broken);
        assert!(!result.hypotheses.is_empty());
        // Should have wait-online hypothesis
        assert!(result.hypotheses.iter().any(|h| h.id == "wait-online-slow"));
    }

    #[test]
    fn test_diagnosis_new_offender() {
        let doctor = BootDoctor::new();
        let evidence = create_slow_evidence();
        let result = doctor.diagnose(&evidence);

        // Should detect new docker.service
        assert!(result.findings.iter().any(|f| f.id.contains("new-")));
    }

    #[test]
    fn test_diagnosis_regression() {
        let doctor = BootDoctor::new();
        let evidence = create_slow_evidence();
        let result = doctor.diagnose(&evidence);

        // Should detect wait-online regression
        assert!(result.findings.iter().any(|f| f.id.contains("regressed-")));
    }

    #[test]
    fn test_correlation_engine() {
        let doctor = BootDoctor::new();
        let evidence = create_slow_evidence();
        let result = doctor.diagnose(&evidence);

        // Should correlate docker install with docker.service being slow
        assert!(result
            .findings
            .iter()
            .any(|f| f.id.starts_with("corr-") && f.description.contains("docker")));
    }

    #[test]
    fn test_playbook_generation_wait_online() {
        let doctor = BootDoctor::new();
        let evidence = create_slow_evidence();

        let playbook = doctor.create_playbook("disable_wait_online", &evidence, None);
        assert!(playbook.is_some());

        let pb = playbook.unwrap();
        assert_eq!(pb.id, "disable_wait_online");
        assert_eq!(pb.risk, RiskLevel::Medium);
        assert!(!pb.policy_blocked);
        assert!(pb.verification_pending.is_some());
    }

    #[test]
    fn test_playbook_restart_service() {
        let doctor = BootDoctor::new();
        let evidence = create_slow_evidence();

        let playbook = doctor.create_playbook("restart_NetworkManager.service", &evidence, None);
        assert!(playbook.is_some());

        let pb = playbook.unwrap();
        assert_eq!(pb.playbook_type, PlaybookType::RestartService);
        assert_eq!(pb.risk, RiskLevel::Low);
    }

    #[test]
    fn test_playbook_disable_critical_blocked() {
        let doctor = BootDoctor::new();
        let evidence = create_slow_evidence();

        let playbook = doctor.create_playbook("disable_systemd-journald.service", &evidence, None);
        assert!(playbook.is_some());

        let pb = playbook.unwrap();
        assert!(pb.policy_blocked);
        assert!(pb.policy_block_reason.is_some());
    }

    #[test]
    fn test_max_three_hypotheses() {
        let doctor = BootDoctor::new();

        // Create evidence with many issues
        let mut evidence = create_slow_evidence();
        for i in 0..5 {
            evidence.blame.push(BootOffender {
                unit: format!("slow-service-{}.service", i),
                time_ms: 10000,
                is_new: true,
                is_regressed: false,
                previous_ms: None,
                correlation: None,
                in_critical_chain: false,
            });
        }

        let result = doctor.diagnose(&evidence);
        assert!(result.hypotheses.len() <= 3);
    }

    #[test]
    fn test_case_file_workflow() {
        let evidence = create_slow_evidence();
        let mut case_file = BootDoctorCase::new(evidence);

        assert_eq!(case_file.status, CaseStatus::EvidenceCollected);

        let doctor = BootDoctor::new();
        let diagnosis = doctor.diagnose(&case_file.evidence);
        case_file.add_diagnosis(diagnosis);

        assert_eq!(case_file.status, CaseStatus::Diagnosed);

        case_file.add_note("Investigating wait-online issue");
        assert_eq!(case_file.notes.len(), 1);
    }

    #[test]
    fn test_recipe_capture_on_success() {
        let doctor = BootDoctor::new();
        let evidence = create_slow_evidence();

        let playbook = doctor
            .create_playbook("disable_wait_online", &evidence, Some("wait-online-slow"))
            .unwrap();

        // Simulate successful result
        let result = PlaybookResult {
            playbook_id: playbook.id.clone(),
            success: true,
            commands_executed: vec![],
            post_checks_passed: true,
            error: None,
            verification_pending: None,
            reliability: 85,
        };

        let recipe = doctor.maybe_capture_recipe(&playbook, &result, &evidence);
        assert!(recipe.is_some());

        let r = recipe.unwrap();
        assert!(r.name.contains("slow boot"));
        assert_eq!(r.reliability, 85);
    }

    #[test]
    fn test_regression_percent() {
        let offender = BootOffender {
            unit: "test.service".to_string(),
            time_ms: 10000,
            is_new: false,
            is_regressed: true,
            previous_ms: Some(5000),
            correlation: None,
            in_critical_chain: false,
        };

        let pct = offender.regression_percent().unwrap();
        assert!((pct - 100.0).abs() < 0.1); // 100% increase
    }

    #[test]
    fn test_timing_summary() {
        let timing = BootTiming {
            firmware_ms: Some(2000),
            loader_ms: Some(1000),
            kernel_ms: 2000,
            initrd_ms: 1500,
            userspace_ms: 15000,
            total_ms: 21500,
            graphical_ms: None,
        };

        let summary = timing.to_summary();
        assert!(summary.contains("firmware"));
        assert!(summary.contains("loader"));
        assert!(summary.contains("kernel"));
        assert!(summary.contains("initrd"));
        assert!(summary.contains("userspace"));
        assert!(summary.contains("21500"));
    }
}
