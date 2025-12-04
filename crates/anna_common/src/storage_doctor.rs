//! Storage Doctor - BTRFS-focused storage diagnosis with safe repair plans
//!
//! v0.0.39: Arch Storage Doctor v1
//!
//! Deterministic diagnosis flow:
//! 1. Identify filesystem types (BTRFS vs EXT4/XFS)
//! 2. Check free space and metadata space (BTRFS-specific)
//! 3. Check device errors and SMART data
//! 4. Check scrub/balance status (BTRFS)
//! 5. Check I/O error logs
//! 6. Risk rating and hypothesis generation

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Storage health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageHealth {
    /// All storage healthy
    Healthy,
    /// Some issues but storage operational
    Degraded,
    /// Critical issues, data at risk
    Critical,
    /// Unknown state (evidence collection failed)
    Unknown,
}

impl std::fmt::Display for StorageHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageHealth::Healthy => write!(f, "Healthy"),
            StorageHealth::Degraded => write!(f, "Degraded"),
            StorageHealth::Critical => write!(f, "Critical"),
            StorageHealth::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Risk level for findings and repair plans
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Informational only
    Info,
    /// Warning, should investigate
    Warning,
    /// Critical, immediate action needed
    Critical,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Info => write!(f, "Info"),
            RiskLevel::Warning => write!(f, "Warning"),
            RiskLevel::Critical => write!(f, "Critical"),
        }
    }
}

/// Filesystem type detected on the system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilesystemType {
    Btrfs,
    Ext4,
    Xfs,
    Zfs,
    Other(String),
}

impl std::fmt::Display for FilesystemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FilesystemType::Btrfs => write!(f, "btrfs"),
            FilesystemType::Ext4 => write!(f, "ext4"),
            FilesystemType::Xfs => write!(f, "xfs"),
            FilesystemType::Zfs => write!(f, "zfs"),
            FilesystemType::Other(s) => write!(f, "{}", s),
        }
    }
}

impl From<&str> for FilesystemType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "btrfs" => FilesystemType::Btrfs,
            "ext4" => FilesystemType::Ext4,
            "xfs" => FilesystemType::Xfs,
            "zfs" => FilesystemType::Zfs,
            other => FilesystemType::Other(other.to_string()),
        }
    }
}

/// Mount point information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountInfo {
    /// Device path (e.g., /dev/sda1)
    pub device: String,
    /// Mount point (e.g., /home)
    pub mount_point: String,
    /// Filesystem type
    pub fs_type: FilesystemType,
    /// Mount options
    pub options: Vec<String>,
    /// Total size in bytes
    pub total_bytes: u64,
    /// Used bytes
    pub used_bytes: u64,
    /// Available bytes
    pub available_bytes: u64,
    /// Usage percentage
    pub usage_percent: f64,
}

impl MountInfo {
    /// Check if this is a BTRFS mount
    pub fn is_btrfs(&self) -> bool {
        self.fs_type == FilesystemType::Btrfs
    }

    /// Check if space is critically low (>90%)
    pub fn is_space_critical(&self) -> bool {
        self.usage_percent >= 90.0
    }

    /// Check if space is warning level (>80%)
    pub fn is_space_warning(&self) -> bool {
        self.usage_percent >= 80.0
    }
}

/// BTRFS device statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BtrfsDeviceStats {
    /// Device path
    pub device: String,
    /// Write I/O errors
    pub write_io_errs: u64,
    /// Read I/O errors
    pub read_io_errs: u64,
    /// Flush I/O errors
    pub flush_io_errs: u64,
    /// Corruption errors
    pub corruption_errs: u64,
    /// Generation errors
    pub generation_errs: u64,
}

impl BtrfsDeviceStats {
    /// Check if any errors exist
    pub fn has_errors(&self) -> bool {
        self.write_io_errs > 0
            || self.read_io_errs > 0
            || self.flush_io_errs > 0
            || self.corruption_errs > 0
            || self.generation_errs > 0
    }

    /// Get total error count
    pub fn total_errors(&self) -> u64 {
        self.write_io_errs
            + self.read_io_errs
            + self.flush_io_errs
            + self.corruption_errs
            + self.generation_errs
    }

    /// Determine risk level based on errors
    pub fn risk_level(&self) -> RiskLevel {
        if self.corruption_errs > 0 || self.generation_errs > 0 {
            RiskLevel::Critical
        } else if self.total_errors() > 10 {
            RiskLevel::Critical
        } else if self.has_errors() {
            RiskLevel::Warning
        } else {
            RiskLevel::Info
        }
    }
}

/// BTRFS usage information (data vs metadata)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BtrfsUsage {
    /// Mount point this applies to
    pub mount_point: String,
    /// Data space total
    pub data_total: u64,
    /// Data space used
    pub data_used: u64,
    /// Metadata space total
    pub metadata_total: u64,
    /// Metadata space used
    pub metadata_used: u64,
    /// System space total
    pub system_total: u64,
    /// System space used
    pub system_used: u64,
    /// Unallocated space
    pub unallocated: u64,
}

impl BtrfsUsage {
    /// Get metadata usage percentage
    pub fn metadata_usage_percent(&self) -> f64 {
        if self.metadata_total == 0 {
            0.0
        } else {
            (self.metadata_used as f64 / self.metadata_total as f64) * 100.0
        }
    }

    /// Get data usage percentage
    pub fn data_usage_percent(&self) -> f64 {
        if self.data_total == 0 {
            0.0
        } else {
            (self.data_used as f64 / self.data_total as f64) * 100.0
        }
    }

    /// Check if metadata space is critical (>90%)
    pub fn is_metadata_critical(&self) -> bool {
        self.metadata_usage_percent() >= 90.0
    }

    /// Check if metadata space is warning level (>80%)
    pub fn is_metadata_warning(&self) -> bool {
        self.metadata_usage_percent() >= 80.0
    }
}

/// BTRFS scrub status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScrubStatus {
    /// Never run
    Never,
    /// Currently running
    Running {
        started: Option<DateTime<Utc>>,
        progress_percent: f64,
    },
    /// Completed successfully
    Completed {
        finished: Option<DateTime<Utc>>,
        errors_found: u64,
        errors_corrected: u64,
    },
    /// Aborted or failed
    Failed { reason: String },
}

impl ScrubStatus {
    /// Check if scrub found uncorrected errors
    pub fn has_uncorrected_errors(&self) -> bool {
        match self {
            ScrubStatus::Completed {
                errors_found,
                errors_corrected,
                ..
            } => errors_found > errors_corrected,
            _ => false,
        }
    }
}

/// BTRFS balance status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BalanceStatus {
    /// Not running
    Idle,
    /// Currently running
    Running {
        started: Option<DateTime<Utc>>,
        progress_percent: f64,
    },
    /// Paused
    Paused,
}

/// BTRFS filesystem information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtrfsInfo {
    /// Filesystem UUID
    pub uuid: String,
    /// Label if set
    pub label: Option<String>,
    /// Mount point
    pub mount_point: String,
    /// Device statistics
    pub device_stats: Vec<BtrfsDeviceStats>,
    /// Usage information
    pub usage: Option<BtrfsUsage>,
    /// Scrub status
    pub scrub_status: ScrubStatus,
    /// Balance status
    pub balance_status: BalanceStatus,
    /// RAID profile for data
    pub data_profile: Option<String>,
    /// RAID profile for metadata
    pub metadata_profile: Option<String>,
}

impl BtrfsInfo {
    /// Check if any device has errors
    pub fn has_device_errors(&self) -> bool {
        self.device_stats.iter().any(|d| d.has_errors())
    }

    /// Get total device errors
    pub fn total_device_errors(&self) -> u64 {
        self.device_stats.iter().map(|d| d.total_errors()).sum()
    }

    /// Check overall health
    pub fn health_status(&self) -> StorageHealth {
        // Critical conditions
        if self
            .device_stats
            .iter()
            .any(|d| d.corruption_errs > 0 || d.generation_errs > 0)
        {
            return StorageHealth::Critical;
        }
        if let Some(usage) = &self.usage {
            if usage.is_metadata_critical() {
                return StorageHealth::Critical;
            }
        }
        if self.scrub_status.has_uncorrected_errors() {
            return StorageHealth::Critical;
        }

        // Degraded conditions
        if self.has_device_errors() {
            return StorageHealth::Degraded;
        }
        if let Some(usage) = &self.usage {
            if usage.is_metadata_warning() {
                return StorageHealth::Degraded;
            }
        }

        StorageHealth::Healthy
    }
}

/// SMART health status for a drive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartHealth {
    /// Device path
    pub device: String,
    /// Model name
    pub model: Option<String>,
    /// Serial number
    pub serial: Option<String>,
    /// Overall health status
    pub passed: bool,
    /// Temperature in Celsius
    pub temperature_c: Option<i32>,
    /// Power on hours
    pub power_on_hours: Option<u64>,
    /// Reallocated sector count
    pub reallocated_sectors: Option<u64>,
    /// Pending sector count
    pub pending_sectors: Option<u64>,
    /// Uncorrectable sector count
    pub uncorrectable_sectors: Option<u64>,
    /// Raw read error rate
    pub raw_read_error_rate: Option<u64>,
    /// Is SSD
    pub is_ssd: bool,
    /// SSD wear percentage (if applicable)
    pub wear_percentage: Option<f64>,
}

impl SmartHealth {
    /// Check if drive has concerning attributes
    pub fn has_warnings(&self) -> bool {
        !self.passed
            || self.reallocated_sectors.unwrap_or(0) > 0
            || self.pending_sectors.unwrap_or(0) > 0
            || self.uncorrectable_sectors.unwrap_or(0) > 0
            || self.wear_percentage.map(|w| w > 80.0).unwrap_or(false)
    }

    /// Get risk level
    pub fn risk_level(&self) -> RiskLevel {
        if !self.passed {
            return RiskLevel::Critical;
        }
        if self.uncorrectable_sectors.unwrap_or(0) > 0 {
            return RiskLevel::Critical;
        }
        if self.reallocated_sectors.unwrap_or(0) > 100 {
            return RiskLevel::Critical;
        }
        if self.wear_percentage.map(|w| w > 90.0).unwrap_or(false) {
            return RiskLevel::Critical;
        }
        if self.has_warnings() {
            return RiskLevel::Warning;
        }
        RiskLevel::Info
    }
}

/// I/O error log entry from kernel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoErrorLog {
    /// Timestamp of error
    pub timestamp: Option<DateTime<Utc>>,
    /// Device involved
    pub device: Option<String>,
    /// Error message
    pub message: String,
    /// Error type classification
    pub error_type: IoErrorType,
}

/// Type of I/O error
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IoErrorType {
    /// Read error
    Read,
    /// Write error
    Write,
    /// Timeout
    Timeout,
    /// Medium error (bad sector)
    Medium,
    /// Controller error
    Controller,
    /// Filesystem error
    Filesystem,
    /// Other/unknown
    Other,
}

/// Complete storage evidence bundle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageEvidence {
    /// Collection timestamp
    pub collected_at: DateTime<Utc>,
    /// Mount point information
    pub mounts: Vec<MountInfo>,
    /// BTRFS filesystem information
    pub btrfs_filesystems: Vec<BtrfsInfo>,
    /// SMART health data
    pub smart_health: Vec<SmartHealth>,
    /// Recent I/O errors from kernel log
    pub io_errors: Vec<IoErrorLog>,
    /// Block device list (lsblk output)
    pub block_devices: Vec<BlockDevice>,
    /// Evidence collection errors
    pub collection_errors: Vec<String>,
}

/// Block device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDevice {
    /// Device name (e.g., sda)
    pub name: String,
    /// Device path (e.g., /dev/sda)
    pub path: String,
    /// Size in bytes
    pub size: u64,
    /// Device type (disk, part, lvm, etc.)
    pub device_type: String,
    /// Filesystem type if any
    pub fs_type: Option<String>,
    /// Mount point if mounted
    pub mount_point: Option<String>,
    /// Is rotational (HDD vs SSD)
    pub rotational: bool,
    /// Model name
    pub model: Option<String>,
    /// Parent device if partition
    pub parent: Option<String>,
}

impl StorageEvidence {
    /// Create new empty evidence bundle
    pub fn new() -> Self {
        Self {
            collected_at: Utc::now(),
            mounts: Vec::new(),
            btrfs_filesystems: Vec::new(),
            smart_health: Vec::new(),
            io_errors: Vec::new(),
            block_devices: Vec::new(),
            collection_errors: Vec::new(),
        }
    }

    /// Check if any BTRFS filesystems exist
    pub fn has_btrfs(&self) -> bool {
        !self.btrfs_filesystems.is_empty()
    }

    /// Get overall storage health
    pub fn overall_health(&self) -> StorageHealth {
        // Check BTRFS health
        for btrfs in &self.btrfs_filesystems {
            match btrfs.health_status() {
                StorageHealth::Critical => return StorageHealth::Critical,
                StorageHealth::Degraded => continue, // Check all before deciding
                _ => {}
            }
        }

        // Check SMART health
        for smart in &self.smart_health {
            if smart.risk_level() == RiskLevel::Critical {
                return StorageHealth::Critical;
            }
        }

        // Check for recent I/O errors
        if !self.io_errors.is_empty() {
            return StorageHealth::Degraded;
        }

        // Check for BTRFS degraded
        for btrfs in &self.btrfs_filesystems {
            if btrfs.health_status() == StorageHealth::Degraded {
                return StorageHealth::Degraded;
            }
        }

        // Check SMART warnings
        for smart in &self.smart_health {
            if smart.has_warnings() {
                return StorageHealth::Degraded;
            }
        }

        // Check space warnings
        for mount in &self.mounts {
            if mount.is_space_critical() {
                return StorageHealth::Degraded;
            }
        }

        if self.collection_errors.is_empty() {
            StorageHealth::Healthy
        } else {
            StorageHealth::Unknown
        }
    }
}

impl Default for StorageEvidence {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Diagnosis Flow
// ============================================================================

/// Diagnosis step in the deterministic flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisStep {
    /// Step name
    pub name: String,
    /// Step description
    pub description: String,
    /// Step number (1-based)
    pub step_number: u32,
    /// Whether step passed
    pub passed: bool,
    /// Findings from this step
    pub findings: Vec<Finding>,
    /// Raw evidence collected
    pub evidence_ids: Vec<String>,
}

/// A finding from diagnosis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Unique ID for this finding
    pub id: String,
    /// Short summary
    pub summary: String,
    /// Detailed description
    pub detail: String,
    /// Risk level
    pub risk: RiskLevel,
    /// Evidence ID this finding is based on
    pub evidence_id: String,
    /// Related device/mount if applicable
    pub location: Option<String>,
}

/// A hypothesis about a storage issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageHypothesis {
    /// Hypothesis ID
    pub id: String,
    /// Short description
    pub summary: String,
    /// Detailed explanation
    pub explanation: String,
    /// Confidence percentage (0-100)
    pub confidence: u8,
    /// Evidence IDs supporting this hypothesis
    pub supporting_evidence: Vec<String>,
    /// Criteria to confirm this hypothesis
    pub confirm_criteria: Vec<String>,
    /// Criteria to refute this hypothesis
    pub refute_criteria: Vec<String>,
    /// Suggested repair plan if applicable
    pub suggested_repair: Option<String>,
}

/// Diagnosis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisResult {
    /// Overall health status
    pub status: StorageHealth,
    /// Steps executed
    pub steps: Vec<DiagnosisStep>,
    /// Hypotheses generated (max 3)
    pub hypotheses: Vec<StorageHypothesis>,
    /// Summary message
    pub summary: String,
    /// Timestamp
    pub diagnosed_at: DateTime<Utc>,
}

// ============================================================================
// Repair Plans
// ============================================================================

/// Type of repair plan
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepairPlanType {
    /// Read-only diagnostic (no confirmation needed)
    ReadOnly,
    /// Mutation that modifies state (confirmation needed)
    Mutation,
}

/// A repair plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairPlan {
    /// Plan ID
    pub id: String,
    /// Short name
    pub name: String,
    /// Description
    pub description: String,
    /// Plan type
    pub plan_type: RepairPlanType,
    /// Risk level
    pub risk: RiskLevel,
    /// Commands to execute
    pub commands: Vec<RepairCommand>,
    /// Preflight checks
    pub preflight_checks: Vec<PreflightCheck>,
    /// Post-execution checks
    pub post_checks: Vec<PostCheck>,
    /// Rollback instructions (if applicable)
    pub rollback: Option<RollbackPlan>,
    /// Whether this plan is blocked by policy
    pub policy_blocked: bool,
    /// Reason for policy block
    pub policy_block_reason: Option<String>,
    /// Confirmation phrase required (for mutations)
    pub confirmation_phrase: Option<String>,
    /// Hypothesis this addresses
    pub addresses_hypothesis: Option<String>,
}

/// A command in a repair plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairCommand {
    /// Command to run
    pub command: String,
    /// Description of what it does
    pub description: String,
    /// Expected output pattern (regex)
    pub expected_output: Option<String>,
    /// Timeout in seconds
    pub timeout_secs: u32,
    /// Whether failure is fatal
    pub fatal_on_failure: bool,
}

/// Preflight check before repair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightCheck {
    /// Check name
    pub name: String,
    /// Command to run
    pub command: String,
    /// Expected output pattern (regex)
    pub expected_pattern: String,
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
    pub expected_pattern: String,
    /// Wait time before check (seconds)
    pub wait_secs: u32,
}

/// Rollback plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackPlan {
    /// Description
    pub description: String,
    /// Commands to rollback
    pub commands: Vec<String>,
    /// Whether automatic rollback is possible
    pub automatic: bool,
}

/// Result of executing a repair plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairResult {
    /// Plan ID
    pub plan_id: String,
    /// Whether repair succeeded
    pub success: bool,
    /// Commands executed
    pub commands_executed: Vec<CommandResult>,
    /// Post-check results
    pub post_check_results: Vec<CheckResult>,
    /// Error message if failed
    pub error: Option<String>,
    /// Timestamp
    pub executed_at: DateTime<Utc>,
}

/// Result of a single command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    /// Command run
    pub command: String,
    /// Exit code
    pub exit_code: i32,
    /// Stdout
    pub stdout: String,
    /// Stderr
    pub stderr: String,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

/// Result of a check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    /// Check name
    pub name: String,
    /// Whether check passed
    pub passed: bool,
    /// Actual output
    pub output: String,
}

// ============================================================================
// Case File
// ============================================================================

/// Storage doctor case file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageDoctorCase {
    /// Case ID
    pub case_id: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Evidence collected
    pub evidence: StorageEvidence,
    /// Diagnosis result
    pub diagnosis: Option<DiagnosisResult>,
    /// Repair plans available
    pub repair_plans: Vec<RepairPlan>,
    /// Repairs executed
    pub repairs_executed: Vec<RepairResult>,
    /// Case status
    pub status: CaseStatus,
    /// Notes
    pub notes: Vec<CaseNote>,
}

/// Case status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaseStatus {
    /// Evidence collected, awaiting diagnosis
    EvidenceCollected,
    /// Diagnosis complete
    Diagnosed,
    /// Repair in progress
    RepairInProgress,
    /// Repair complete
    RepairComplete,
    /// Closed
    Closed,
}

/// A note on the case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseNote {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Note content
    pub content: String,
    /// Author (anna, junior, senior)
    pub author: String,
}

impl StorageDoctorCase {
    /// Create new case
    pub fn new(case_id: String, evidence: StorageEvidence) -> Self {
        let now = Utc::now();
        Self {
            case_id,
            created_at: now,
            updated_at: now,
            evidence,
            diagnosis: None,
            repair_plans: Vec::new(),
            repairs_executed: Vec::new(),
            status: CaseStatus::EvidenceCollected,
            notes: Vec::new(),
        }
    }

    /// Add diagnosis to case
    pub fn set_diagnosis(&mut self, diagnosis: DiagnosisResult) {
        self.diagnosis = Some(diagnosis);
        self.status = CaseStatus::Diagnosed;
        self.updated_at = Utc::now();
    }

    /// Add repair plans
    pub fn set_repair_plans(&mut self, plans: Vec<RepairPlan>) {
        self.repair_plans = plans;
        self.updated_at = Utc::now();
    }

    /// Record repair execution
    pub fn record_repair(&mut self, result: RepairResult) {
        if self.status != CaseStatus::RepairInProgress {
            self.status = CaseStatus::RepairInProgress;
        }
        self.repairs_executed.push(result);
        self.updated_at = Utc::now();
    }

    /// Mark repair complete
    pub fn complete_repair(&mut self) {
        self.status = CaseStatus::RepairComplete;
        self.updated_at = Utc::now();
    }

    /// Close case
    pub fn close(&mut self) {
        self.status = CaseStatus::Closed;
        self.updated_at = Utc::now();
    }

    /// Add note
    pub fn add_note(&mut self, content: String, author: String) {
        self.notes.push(CaseNote {
            timestamp: Utc::now(),
            content,
            author,
        });
        self.updated_at = Utc::now();
    }
}

// ============================================================================
// Diagnosis Engine
// ============================================================================

/// Storage doctor diagnosis engine
pub struct StorageDoctor {
    /// Policy: whether balance operations are allowed
    pub allow_balance: bool,
    /// Policy: whether scrub operations are allowed
    pub allow_scrub: bool,
}

impl Default for StorageDoctor {
    fn default() -> Self {
        Self {
            allow_balance: false, // Blocked by default (can take hours)
            allow_scrub: true,    // Allowed by default
        }
    }
}

impl StorageDoctor {
    /// Create new storage doctor
    pub fn new() -> Self {
        Self::default()
    }

    /// Run deterministic diagnosis flow
    pub fn diagnose(&self, evidence: &StorageEvidence) -> DiagnosisResult {
        let mut steps = Vec::new();
        let mut all_findings = Vec::new();

        // Step 1: Identify filesystem types
        let step1 = self.step_identify_filesystems(evidence);
        all_findings.extend(step1.findings.clone());
        steps.push(step1);

        // Step 2: Check free space and metadata space
        let step2 = self.step_check_space(evidence);
        all_findings.extend(step2.findings.clone());
        steps.push(step2);

        // Step 3: Check device errors and SMART
        let step3 = self.step_check_device_health(evidence);
        all_findings.extend(step3.findings.clone());
        steps.push(step3);

        // Step 4: Check scrub/balance status
        let step4 = self.step_check_btrfs_maintenance(evidence);
        all_findings.extend(step4.findings.clone());
        steps.push(step4);

        // Step 5: Check I/O error logs
        let step5 = self.step_check_io_errors(evidence);
        all_findings.extend(step5.findings.clone());
        steps.push(step5);

        // Step 6: Generate hypotheses and risk rating
        let hypotheses = self.generate_hypotheses(&all_findings, evidence);
        let status = self.determine_status(&steps);
        let summary = self.generate_summary(&status, &hypotheses);

        DiagnosisResult {
            status,
            steps,
            hypotheses,
            summary,
            diagnosed_at: Utc::now(),
        }
    }

    /// Step 1: Identify filesystem types
    fn step_identify_filesystems(&self, evidence: &StorageEvidence) -> DiagnosisStep {
        let mut findings = Vec::new();
        let mut evidence_ids = Vec::new();

        let btrfs_count = evidence.mounts.iter().filter(|m| m.is_btrfs()).count();
        let ext4_count = evidence
            .mounts
            .iter()
            .filter(|m| m.fs_type == FilesystemType::Ext4)
            .count();
        let xfs_count = evidence
            .mounts
            .iter()
            .filter(|m| m.fs_type == FilesystemType::Xfs)
            .count();

        let evidence_id = format!("ev-fs-types-{}", Utc::now().timestamp());
        evidence_ids.push(evidence_id.clone());

        if btrfs_count > 0 {
            findings.push(Finding {
                id: format!("find-btrfs-detected-{}", Utc::now().timestamp()),
                summary: format!("{} BTRFS filesystem(s) detected", btrfs_count),
                detail: format!(
                    "Found {} BTRFS mount(s). BTRFS-specific diagnostics will be performed.",
                    btrfs_count
                ),
                risk: RiskLevel::Info,
                evidence_id: evidence_id.clone(),
                location: None,
            });
        }

        if ext4_count > 0 {
            findings.push(Finding {
                id: format!("find-ext4-detected-{}", Utc::now().timestamp()),
                summary: format!("{} EXT4 filesystem(s) detected", ext4_count),
                detail: format!(
                    "Found {} EXT4 mount(s). Basic health checks will be performed.",
                    ext4_count
                ),
                risk: RiskLevel::Info,
                evidence_id: evidence_id.clone(),
                location: None,
            });
        }

        if xfs_count > 0 {
            findings.push(Finding {
                id: format!("find-xfs-detected-{}", Utc::now().timestamp()),
                summary: format!("{} XFS filesystem(s) detected", xfs_count),
                detail: format!(
                    "Found {} XFS mount(s). Basic health checks will be performed.",
                    xfs_count
                ),
                risk: RiskLevel::Info,
                evidence_id,
                location: None,
            });
        }

        DiagnosisStep {
            name: "Identify Filesystem Types".to_string(),
            description: "Detect BTRFS, EXT4, XFS and other filesystem types".to_string(),
            step_number: 1,
            passed: true,
            findings,
            evidence_ids,
        }
    }

    /// Step 2: Check free space and metadata space
    fn step_check_space(&self, evidence: &StorageEvidence) -> DiagnosisStep {
        let mut findings = Vec::new();
        let mut evidence_ids = Vec::new();
        let mut passed = true;

        // Check mount point space
        for mount in &evidence.mounts {
            let evidence_id = format!(
                "ev-space-{}-{}",
                mount.mount_point.replace('/', "-"),
                Utc::now().timestamp()
            );
            evidence_ids.push(evidence_id.clone());

            if mount.is_space_critical() {
                passed = false;
                findings.push(Finding {
                    id: format!("find-space-critical-{}", Utc::now().timestamp()),
                    summary: format!(
                        "{} is {:.1}% full (CRITICAL)",
                        mount.mount_point, mount.usage_percent
                    ),
                    detail: format!(
                        "Mount point {} has {:.1}% disk usage. Available: {} bytes. \
                         Critical threshold is 90%.",
                        mount.mount_point, mount.usage_percent, mount.available_bytes
                    ),
                    risk: RiskLevel::Critical,
                    evidence_id: evidence_id.clone(),
                    location: Some(mount.mount_point.clone()),
                });
            } else if mount.is_space_warning() {
                findings.push(Finding {
                    id: format!("find-space-warning-{}", Utc::now().timestamp()),
                    summary: format!("{} is {:.1}% full", mount.mount_point, mount.usage_percent),
                    detail: format!(
                        "Mount point {} has {:.1}% disk usage. Warning threshold is 80%.",
                        mount.mount_point, mount.usage_percent
                    ),
                    risk: RiskLevel::Warning,
                    evidence_id,
                    location: Some(mount.mount_point.clone()),
                });
            }
        }

        // Check BTRFS metadata space
        for btrfs in &evidence.btrfs_filesystems {
            if let Some(usage) = &btrfs.usage {
                let evidence_id = format!(
                    "ev-metadata-{}-{}",
                    btrfs.mount_point.replace('/', "-"),
                    Utc::now().timestamp()
                );
                evidence_ids.push(evidence_id.clone());

                if usage.is_metadata_critical() {
                    passed = false;
                    findings.push(Finding {
                        id: format!("find-metadata-critical-{}", Utc::now().timestamp()),
                        summary: format!(
                            "BTRFS metadata space critical at {:.1}%",
                            usage.metadata_usage_percent()
                        ),
                        detail: format!(
                            "BTRFS filesystem at {} has {:.1}% metadata usage. \
                             This can cause write failures even with data space available. \
                             Consider running balance to redistribute metadata.",
                            btrfs.mount_point,
                            usage.metadata_usage_percent()
                        ),
                        risk: RiskLevel::Critical,
                        evidence_id: evidence_id.clone(),
                        location: Some(btrfs.mount_point.clone()),
                    });
                } else if usage.is_metadata_warning() {
                    findings.push(Finding {
                        id: format!("find-metadata-warning-{}", Utc::now().timestamp()),
                        summary: format!(
                            "BTRFS metadata space at {:.1}%",
                            usage.metadata_usage_percent()
                        ),
                        detail: format!(
                            "BTRFS filesystem at {} has elevated metadata usage at {:.1}%. \
                             Monitor this metric.",
                            btrfs.mount_point,
                            usage.metadata_usage_percent()
                        ),
                        risk: RiskLevel::Warning,
                        evidence_id,
                        location: Some(btrfs.mount_point.clone()),
                    });
                }
            }
        }

        DiagnosisStep {
            name: "Check Space Usage".to_string(),
            description: "Check free space and BTRFS metadata space".to_string(),
            step_number: 2,
            passed,
            findings,
            evidence_ids,
        }
    }

    /// Step 3: Check device errors and SMART
    fn step_check_device_health(&self, evidence: &StorageEvidence) -> DiagnosisStep {
        let mut findings = Vec::new();
        let mut evidence_ids = Vec::new();
        let mut passed = true;

        // Check BTRFS device stats
        for btrfs in &evidence.btrfs_filesystems {
            for stats in &btrfs.device_stats {
                if stats.has_errors() {
                    let evidence_id = format!(
                        "ev-btrfs-stats-{}-{}",
                        stats.device.replace('/', "-"),
                        Utc::now().timestamp()
                    );
                    evidence_ids.push(evidence_id.clone());

                    let risk = stats.risk_level();
                    if risk >= RiskLevel::Warning {
                        passed = false;
                    }

                    findings.push(Finding {
                        id: format!("find-btrfs-errors-{}", Utc::now().timestamp()),
                        summary: format!(
                            "BTRFS device {} has {} error(s)",
                            stats.device,
                            stats.total_errors()
                        ),
                        detail: format!(
                            "Device {} shows: write_io_errs={}, read_io_errs={}, \
                             flush_io_errs={}, corruption_errs={}, generation_errs={}",
                            stats.device,
                            stats.write_io_errs,
                            stats.read_io_errs,
                            stats.flush_io_errs,
                            stats.corruption_errs,
                            stats.generation_errs
                        ),
                        risk,
                        evidence_id,
                        location: Some(stats.device.clone()),
                    });
                }
            }
        }

        // Check SMART health
        for smart in &evidence.smart_health {
            let evidence_id = format!(
                "ev-smart-{}-{}",
                smart.device.replace('/', "-"),
                Utc::now().timestamp()
            );
            evidence_ids.push(evidence_id.clone());

            if !smart.passed {
                passed = false;
                findings.push(Finding {
                    id: format!("find-smart-failed-{}", Utc::now().timestamp()),
                    summary: format!("SMART health check FAILED for {}", smart.device),
                    detail: format!(
                        "Drive {} ({}) failed SMART self-assessment. \
                         This indicates imminent drive failure. Back up data immediately.",
                        smart.device,
                        smart.model.as_deref().unwrap_or("unknown model")
                    ),
                    risk: RiskLevel::Critical,
                    evidence_id: evidence_id.clone(),
                    location: Some(smart.device.clone()),
                });
            } else if smart.has_warnings() {
                passed = false; // Any warnings means step didn't fully pass
                let mut warnings = Vec::new();
                if let Some(realloc) = smart.reallocated_sectors {
                    if realloc > 0 {
                        warnings.push(format!("{} reallocated sectors", realloc));
                    }
                }
                if let Some(pending) = smart.pending_sectors {
                    if pending > 0 {
                        warnings.push(format!("{} pending sectors", pending));
                    }
                }
                if let Some(wear) = smart.wear_percentage {
                    if wear > 80.0 {
                        warnings.push(format!("{:.1}% SSD wear", wear));
                    }
                }

                findings.push(Finding {
                    id: format!("find-smart-warning-{}", Utc::now().timestamp()),
                    summary: format!("SMART warnings for {}", smart.device),
                    detail: format!(
                        "Drive {} shows warnings: {}",
                        smart.device,
                        warnings.join(", ")
                    ),
                    risk: smart.risk_level(),
                    evidence_id,
                    location: Some(smart.device.clone()),
                });
            }
        }

        DiagnosisStep {
            name: "Check Device Health".to_string(),
            description: "Check BTRFS device stats and SMART data".to_string(),
            step_number: 3,
            passed,
            findings,
            evidence_ids,
        }
    }

    /// Step 4: Check scrub/balance status
    fn step_check_btrfs_maintenance(&self, evidence: &StorageEvidence) -> DiagnosisStep {
        let mut findings = Vec::new();
        let mut evidence_ids = Vec::new();
        let mut passed = true;

        for btrfs in &evidence.btrfs_filesystems {
            let evidence_id = format!(
                "ev-maint-{}-{}",
                btrfs.mount_point.replace('/', "-"),
                Utc::now().timestamp()
            );
            evidence_ids.push(evidence_id.clone());

            // Check scrub status
            match &btrfs.scrub_status {
                ScrubStatus::Never => {
                    findings.push(Finding {
                        id: format!("find-scrub-never-{}", Utc::now().timestamp()),
                        summary: format!("Scrub never run on {}", btrfs.mount_point),
                        detail: format!(
                            "BTRFS filesystem at {} has never been scrubbed. \
                             Consider running a scrub to verify data integrity.",
                            btrfs.mount_point
                        ),
                        risk: RiskLevel::Warning,
                        evidence_id: evidence_id.clone(),
                        location: Some(btrfs.mount_point.clone()),
                    });
                }
                ScrubStatus::Completed {
                    errors_found,
                    errors_corrected,
                    ..
                } => {
                    if errors_found > errors_corrected {
                        passed = false;
                        findings.push(Finding {
                            id: format!("find-scrub-uncorrected-{}", Utc::now().timestamp()),
                            summary: format!(
                                "Scrub found {} uncorrected errors on {}",
                                errors_found - errors_corrected,
                                btrfs.mount_point
                            ),
                            detail: format!(
                                "Last scrub found {} errors, only {} corrected. \
                                 Uncorrectable errors indicate data loss or hardware issues.",
                                errors_found, errors_corrected
                            ),
                            risk: RiskLevel::Critical,
                            evidence_id: evidence_id.clone(),
                            location: Some(btrfs.mount_point.clone()),
                        });
                    }
                }
                ScrubStatus::Failed { reason } => {
                    findings.push(Finding {
                        id: format!("find-scrub-failed-{}", Utc::now().timestamp()),
                        summary: format!("Last scrub failed on {}", btrfs.mount_point),
                        detail: format!("Scrub failed with reason: {}", reason),
                        risk: RiskLevel::Warning,
                        evidence_id: evidence_id.clone(),
                        location: Some(btrfs.mount_point.clone()),
                    });
                }
                ScrubStatus::Running { .. } => {
                    findings.push(Finding {
                        id: format!("find-scrub-running-{}", Utc::now().timestamp()),
                        summary: format!("Scrub currently running on {}", btrfs.mount_point),
                        detail: "A scrub operation is in progress.".to_string(),
                        risk: RiskLevel::Info,
                        evidence_id: evidence_id.clone(),
                        location: Some(btrfs.mount_point.clone()),
                    });
                }
            }

            // Check balance status
            if let BalanceStatus::Running { .. } = btrfs.balance_status {
                findings.push(Finding {
                    id: format!("find-balance-running-{}", Utc::now().timestamp()),
                    summary: format!("Balance running on {}", btrfs.mount_point),
                    detail: "A balance operation is in progress. This may affect performance."
                        .to_string(),
                    risk: RiskLevel::Info,
                    evidence_id,
                    location: Some(btrfs.mount_point.clone()),
                });
            }
        }

        DiagnosisStep {
            name: "Check BTRFS Maintenance".to_string(),
            description: "Check scrub and balance status".to_string(),
            step_number: 4,
            passed,
            findings,
            evidence_ids,
        }
    }

    /// Step 5: Check I/O error logs
    fn step_check_io_errors(&self, evidence: &StorageEvidence) -> DiagnosisStep {
        let mut findings = Vec::new();
        let mut evidence_ids = Vec::new();
        let passed = evidence.io_errors.is_empty();

        if !evidence.io_errors.is_empty() {
            let evidence_id = format!("ev-io-errors-{}", Utc::now().timestamp());
            evidence_ids.push(evidence_id.clone());

            // Group errors by device
            let mut by_device: HashMap<String, Vec<&IoErrorLog>> = HashMap::new();
            for err in &evidence.io_errors {
                let device = err.device.clone().unwrap_or_else(|| "unknown".to_string());
                by_device.entry(device).or_default().push(err);
            }

            for (device, errors) in by_device {
                let risk = if errors.iter().any(|e| e.error_type == IoErrorType::Medium) {
                    RiskLevel::Critical
                } else {
                    RiskLevel::Warning
                };

                findings.push(Finding {
                    id: format!(
                        "find-io-errors-{}-{}",
                        device.replace('/', "-"),
                        Utc::now().timestamp()
                    ),
                    summary: format!("{} I/O error(s) on {}", errors.len(), device),
                    detail: format!(
                        "Found {} I/O errors in kernel log for {}. Latest: {}",
                        errors.len(),
                        device,
                        errors
                            .first()
                            .map(|e| e.message.as_str())
                            .unwrap_or("unknown")
                    ),
                    risk,
                    evidence_id: evidence_id.clone(),
                    location: Some(device),
                });
            }
        }

        DiagnosisStep {
            name: "Check I/O Error Logs".to_string(),
            description: "Scan kernel logs for storage I/O errors".to_string(),
            step_number: 5,
            passed,
            findings,
            evidence_ids,
        }
    }

    /// Determine overall status from steps
    fn determine_status(&self, steps: &[DiagnosisStep]) -> StorageHealth {
        let fails = steps.iter().filter(|s| !s.passed).count();
        let critical_findings = steps
            .iter()
            .flat_map(|s| &s.findings)
            .any(|f| f.risk == RiskLevel::Critical);

        if critical_findings {
            StorageHealth::Critical
        } else if fails > 0 {
            StorageHealth::Degraded
        } else {
            StorageHealth::Healthy
        }
    }

    /// Generate hypotheses from findings
    fn generate_hypotheses(
        &self,
        findings: &[Finding],
        evidence: &StorageEvidence,
    ) -> Vec<StorageHypothesis> {
        let mut hypotheses = Vec::new();

        // Check for dying drive hypothesis
        let smart_critical = findings
            .iter()
            .any(|f| f.summary.contains("SMART") && f.risk == RiskLevel::Critical);
        let io_errors = findings.iter().any(|f| f.summary.contains("I/O error"));

        if smart_critical || (io_errors && evidence.smart_health.iter().any(|s| s.has_warnings())) {
            hypotheses.push(StorageHypothesis {
                id: format!("hyp-dying-drive-{}", Utc::now().timestamp()),
                summary: "Failing storage device".to_string(),
                explanation: "SMART data and/or I/O errors suggest a storage device is failing. \
                              This requires immediate attention to prevent data loss."
                    .to_string(),
                confidence: if smart_critical { 90 } else { 70 },
                supporting_evidence: findings
                    .iter()
                    .filter(|f| f.summary.contains("SMART") || f.summary.contains("I/O error"))
                    .map(|f| f.evidence_id.clone())
                    .collect(),
                confirm_criteria: vec![
                    "SMART self-test fails".to_string(),
                    "Increasing reallocated sector count".to_string(),
                    "Drive audible clicking or grinding".to_string(),
                ],
                refute_criteria: vec![
                    "Errors clear after cable reseat".to_string(),
                    "Errors correlate with specific workload only".to_string(),
                ],
                suggested_repair: Some("backup_and_replace".to_string()),
            });
        }

        // Check for BTRFS metadata pressure
        let metadata_critical = findings
            .iter()
            .any(|f| f.summary.contains("metadata") && f.risk == RiskLevel::Critical);

        if metadata_critical {
            hypotheses.push(StorageHypothesis {
                id: format!("hyp-metadata-pressure-{}", Utc::now().timestamp()),
                summary: "BTRFS metadata space exhaustion".to_string(),
                explanation: "BTRFS metadata space is critically low. This can cause write failures \
                              even with data space available. A balance operation may help redistribute \
                              metadata, or you may need to delete snapshots/files.".to_string(),
                confidence: 85,
                supporting_evidence: findings
                    .iter()
                    .filter(|f| f.summary.contains("metadata"))
                    .map(|f| f.evidence_id.clone())
                    .collect(),
                confirm_criteria: vec![
                    "Write operations fail with ENOSPC".to_string(),
                    "'btrfs fi usage' shows high metadata allocation".to_string(),
                ],
                refute_criteria: vec![
                    "Writes succeed normally".to_string(),
                    "Deleting files frees metadata space".to_string(),
                ],
                suggested_repair: Some("btrfs_balance_metadata".to_string()),
            });
        }

        // Check for corruption
        let corruption = findings.iter().any(|f| {
            f.detail.contains("corruption_errs") && !f.detail.contains("corruption_errs=0")
        }) || findings
            .iter()
            .any(|f| f.summary.contains("uncorrected errors"));

        if corruption {
            hypotheses.push(StorageHypothesis {
                id: format!("hyp-data-corruption-{}", Utc::now().timestamp()),
                summary: "Data corruption detected".to_string(),
                explanation: "BTRFS has detected data corruption that could not be repaired. \
                              This may indicate hardware issues, cosmic ray bit flips, or filesystem bugs. \
                              Verify affected files and restore from backup if needed.".to_string(),
                confidence: 95,
                supporting_evidence: findings
                    .iter()
                    .filter(|f| f.detail.contains("corruption") || f.summary.contains("uncorrected"))
                    .map(|f| f.evidence_id.clone())
                    .collect(),
                confirm_criteria: vec![
                    "Affected files have incorrect checksums".to_string(),
                    "Files fail to read or show garbage".to_string(),
                ],
                refute_criteria: vec![
                    "Scrub clears after running again".to_string(),
                    "Memory test (memtest86+) reveals RAM issues".to_string(),
                ],
                suggested_repair: Some("restore_from_backup".to_string()),
            });
        }

        // Limit to 3 hypotheses, sorted by confidence
        hypotheses.sort_by(|a, b| b.confidence.cmp(&a.confidence));
        hypotheses.truncate(3);
        hypotheses
    }

    /// Generate summary message
    fn generate_summary(&self, status: &StorageHealth, hypotheses: &[StorageHypothesis]) -> String {
        match status {
            StorageHealth::Healthy => {
                "All storage systems healthy. No issues detected.".to_string()
            }
            StorageHealth::Degraded => {
                if let Some(h) = hypotheses.first() {
                    format!("Storage degraded. Primary hypothesis: {}", h.summary)
                } else {
                    "Storage degraded. Review findings for details.".to_string()
                }
            }
            StorageHealth::Critical => {
                if let Some(h) = hypotheses.first() {
                    format!(
                        "CRITICAL storage issues. Primary hypothesis: {} ({}% confidence)",
                        h.summary, h.confidence
                    )
                } else {
                    "CRITICAL storage issues detected. Immediate action required.".to_string()
                }
            }
            StorageHealth::Unknown => {
                "Unable to determine storage health. Evidence collection failed.".to_string()
            }
        }
    }

    /// Generate repair plans based on diagnosis
    pub fn generate_repair_plans(
        &self,
        diagnosis: &DiagnosisResult,
        evidence: &StorageEvidence,
    ) -> Vec<RepairPlan> {
        let mut plans = Vec::new();

        // Read-only plans (always available)
        plans.extend(self.generate_readonly_plans(evidence));

        // Mutation plans based on findings
        plans.extend(self.generate_mutation_plans(diagnosis, evidence));

        plans
    }

    /// Generate read-only diagnostic plans
    fn generate_readonly_plans(&self, evidence: &StorageEvidence) -> Vec<RepairPlan> {
        let mut plans = Vec::new();

        // SMART extended test for all drives
        for smart in &evidence.smart_health {
            plans.push(RepairPlan {
                id: format!("plan-smart-test-{}", smart.device.replace('/', "-")),
                name: format!("SMART Extended Test ({})", smart.device),
                description: format!(
                    "Run extended SMART self-test on {}. This is read-only and safe.",
                    smart.device
                ),
                plan_type: RepairPlanType::ReadOnly,
                risk: RiskLevel::Info,
                commands: vec![RepairCommand {
                    command: format!("smartctl -t long {}", smart.device),
                    description: "Start extended SMART self-test".to_string(),
                    expected_output: Some("Testing has begun".to_string()),
                    timeout_secs: 30,
                    fatal_on_failure: false,
                }],
                preflight_checks: vec![PreflightCheck {
                    name: "Drive supports SMART".to_string(),
                    command: format!(
                        "smartctl -i {} | grep -q 'SMART support is: Enabled'",
                        smart.device
                    ),
                    expected_pattern: "".to_string(), // Just check exit code
                    error_message: "SMART not enabled on this drive".to_string(),
                }],
                post_checks: vec![PostCheck {
                    name: "Test started".to_string(),
                    command: format!(
                        "smartctl -a {} | grep -E 'Self-test.*in progress'",
                        smart.device
                    ),
                    expected_pattern: "in progress".to_string(),
                    wait_secs: 5,
                }],
                rollback: None,
                policy_blocked: false,
                policy_block_reason: None,
                confirmation_phrase: None,
                addresses_hypothesis: None,
            });
        }

        // BTRFS device stats (read-only)
        for btrfs in &evidence.btrfs_filesystems {
            plans.push(RepairPlan {
                id: format!("plan-btrfs-stats-{}", btrfs.mount_point.replace('/', "-")),
                name: format!("BTRFS Device Stats ({})", btrfs.mount_point),
                description: "View current BTRFS device error statistics".to_string(),
                plan_type: RepairPlanType::ReadOnly,
                risk: RiskLevel::Info,
                commands: vec![RepairCommand {
                    command: format!("btrfs device stats {}", btrfs.mount_point),
                    description: "Show device statistics".to_string(),
                    expected_output: None,
                    timeout_secs: 30,
                    fatal_on_failure: false,
                }],
                preflight_checks: vec![],
                post_checks: vec![],
                rollback: None,
                policy_blocked: false,
                policy_block_reason: None,
                confirmation_phrase: None,
                addresses_hypothesis: None,
            });
        }

        plans
    }

    /// Generate mutation plans
    fn generate_mutation_plans(
        &self,
        diagnosis: &DiagnosisResult,
        evidence: &StorageEvidence,
    ) -> Vec<RepairPlan> {
        let mut plans = Vec::new();

        // Check hypotheses for suggested repairs
        for hyp in &diagnosis.hypotheses {
            if let Some(repair) = &hyp.suggested_repair {
                match repair.as_str() {
                    "btrfs_balance_metadata" => {
                        for btrfs in &evidence.btrfs_filesystems {
                            plans.push(self.create_balance_metadata_plan(btrfs, &hyp.id));
                        }
                    }
                    _ => {}
                }
            }
        }

        // Scrub plan for BTRFS with errors
        for btrfs in &evidence.btrfs_filesystems {
            if btrfs.has_device_errors() || matches!(btrfs.scrub_status, ScrubStatus::Never) {
                plans.push(self.create_scrub_plan(btrfs));
            }
        }

        // Clear device stats plan
        for btrfs in &evidence.btrfs_filesystems {
            if btrfs.has_device_errors() {
                plans.push(self.create_clear_stats_plan(btrfs));
            }
        }

        plans
    }

    /// Create BTRFS balance metadata plan
    fn create_balance_metadata_plan(&self, btrfs: &BtrfsInfo, hypothesis_id: &str) -> RepairPlan {
        RepairPlan {
            id: format!("plan-balance-meta-{}", btrfs.mount_point.replace('/', "-")),
            name: format!("Balance Metadata ({})", btrfs.mount_point),
            description: format!(
                "Run BTRFS balance targeting metadata chunks on {}. \
                 This redistributes metadata across devices and can take hours.",
                btrfs.mount_point
            ),
            plan_type: RepairPlanType::Mutation,
            risk: RiskLevel::Warning,
            commands: vec![RepairCommand {
                command: format!(
                    "btrfs balance start -musage=50 -dusage=50 {}",
                    btrfs.mount_point
                ),
                description: "Balance metadata and data chunks with <50% usage".to_string(),
                expected_output: Some("Done".to_string()),
                timeout_secs: 3600 * 4, // 4 hours
                fatal_on_failure: false,
            }],
            preflight_checks: vec![
                PreflightCheck {
                    name: "No balance running".to_string(),
                    command: format!(
                        "btrfs balance status {} | grep -q 'No balance found'",
                        btrfs.mount_point
                    ),
                    expected_pattern: "".to_string(),
                    error_message: "A balance is already running".to_string(),
                },
                PreflightCheck {
                    name: "Filesystem mounted read-write".to_string(),
                    command: format!("mount | grep {} | grep -qv 'ro,'", btrfs.mount_point),
                    expected_pattern: "".to_string(),
                    error_message: "Filesystem is mounted read-only".to_string(),
                },
            ],
            post_checks: vec![PostCheck {
                name: "Balance completed".to_string(),
                command: format!("btrfs balance status {}", btrfs.mount_point),
                expected_pattern: "No balance found".to_string(),
                wait_secs: 10,
            }],
            rollback: Some(RollbackPlan {
                description: "Cancel balance if still running".to_string(),
                commands: vec![format!("btrfs balance cancel {}", btrfs.mount_point)],
                automatic: true,
            }),
            policy_blocked: !self.allow_balance,
            policy_block_reason: if !self.allow_balance {
                Some("Balance operations blocked by policy (can take hours)".to_string())
            } else {
                None
            },
            confirmation_phrase: Some("I understand balance can take hours".to_string()),
            addresses_hypothesis: Some(hypothesis_id.to_string()),
        }
    }

    /// Create BTRFS scrub plan
    fn create_scrub_plan(&self, btrfs: &BtrfsInfo) -> RepairPlan {
        RepairPlan {
            id: format!("plan-scrub-{}", btrfs.mount_point.replace('/', "-")),
            name: format!("Start Scrub ({})", btrfs.mount_point),
            description: format!(
                "Run BTRFS scrub on {} to verify data integrity. \
                 Scrub reads all data and checksums, correcting errors when possible.",
                btrfs.mount_point
            ),
            plan_type: RepairPlanType::Mutation,
            risk: RiskLevel::Info,
            commands: vec![RepairCommand {
                command: format!("btrfs scrub start {}", btrfs.mount_point),
                description: "Start background scrub".to_string(),
                expected_output: Some("scrub started".to_string()),
                timeout_secs: 60,
                fatal_on_failure: false,
            }],
            preflight_checks: vec![PreflightCheck {
                name: "No scrub running".to_string(),
                command: format!(
                    "btrfs scrub status {} 2>&1 | grep -qE '(no stats|finished|aborted)'",
                    btrfs.mount_point
                ),
                expected_pattern: "".to_string(),
                error_message: "A scrub is already running".to_string(),
            }],
            post_checks: vec![PostCheck {
                name: "Scrub started".to_string(),
                command: format!("btrfs scrub status {}", btrfs.mount_point),
                expected_pattern: "running".to_string(),
                wait_secs: 5,
            }],
            rollback: Some(RollbackPlan {
                description: "Cancel scrub if needed".to_string(),
                commands: vec![format!("btrfs scrub cancel {}", btrfs.mount_point)],
                automatic: true,
            }),
            policy_blocked: !self.allow_scrub,
            policy_block_reason: if !self.allow_scrub {
                Some("Scrub operations blocked by policy".to_string())
            } else {
                None
            },
            confirmation_phrase: Some("Start BTRFS scrub".to_string()),
            addresses_hypothesis: None,
        }
    }

    /// Create clear device stats plan
    fn create_clear_stats_plan(&self, btrfs: &BtrfsInfo) -> RepairPlan {
        RepairPlan {
            id: format!("plan-clear-stats-{}", btrfs.mount_point.replace('/', "-")),
            name: format!("Clear Device Stats ({})", btrfs.mount_point),
            description: format!(
                "Reset BTRFS device error counters on {}. \
                 Use this after investigating and resolving the underlying issue.",
                btrfs.mount_point
            ),
            plan_type: RepairPlanType::Mutation,
            risk: RiskLevel::Warning,
            commands: vec![RepairCommand {
                command: format!("btrfs device stats --reset {}", btrfs.mount_point),
                description: "Reset device statistics to zero".to_string(),
                expected_output: None,
                timeout_secs: 30,
                fatal_on_failure: false,
            }],
            preflight_checks: vec![],
            post_checks: vec![PostCheck {
                name: "Stats cleared".to_string(),
                command: format!(
                    "btrfs device stats {} | grep -E '[1-9]' | wc -l",
                    btrfs.mount_point
                ),
                expected_pattern: "^0$".to_string(),
                wait_secs: 2,
            }],
            rollback: None, // Cannot un-clear stats
            policy_blocked: false,
            policy_block_reason: None,
            confirmation_phrase: Some("Clear error counters".to_string()),
            addresses_hypothesis: None,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_evidence() -> StorageEvidence {
        StorageEvidence::new()
    }

    fn create_healthy_btrfs() -> BtrfsInfo {
        BtrfsInfo {
            uuid: "test-uuid".to_string(),
            label: Some("root".to_string()),
            mount_point: "/".to_string(),
            device_stats: vec![BtrfsDeviceStats {
                device: "/dev/sda1".to_string(),
                ..Default::default()
            }],
            usage: Some(BtrfsUsage {
                mount_point: "/".to_string(),
                data_total: 100_000_000_000,
                data_used: 50_000_000_000,
                metadata_total: 1_000_000_000,
                metadata_used: 500_000_000,
                system_total: 100_000_000,
                system_used: 50_000_000,
                unallocated: 10_000_000_000,
            }),
            scrub_status: ScrubStatus::Completed {
                finished: Some(Utc::now()),
                errors_found: 0,
                errors_corrected: 0,
            },
            balance_status: BalanceStatus::Idle,
            data_profile: Some("single".to_string()),
            metadata_profile: Some("dup".to_string()),
        }
    }

    #[test]
    fn test_storage_health_display() {
        assert_eq!(StorageHealth::Healthy.to_string(), "Healthy");
        assert_eq!(StorageHealth::Degraded.to_string(), "Degraded");
        assert_eq!(StorageHealth::Critical.to_string(), "Critical");
        assert_eq!(StorageHealth::Unknown.to_string(), "Unknown");
    }

    #[test]
    fn test_filesystem_type_from_str() {
        assert_eq!(FilesystemType::from("btrfs"), FilesystemType::Btrfs);
        assert_eq!(FilesystemType::from("BTRFS"), FilesystemType::Btrfs);
        assert_eq!(FilesystemType::from("ext4"), FilesystemType::Ext4);
        assert_eq!(FilesystemType::from("xfs"), FilesystemType::Xfs);
        assert_eq!(FilesystemType::from("zfs"), FilesystemType::Zfs);
        assert!(matches!(
            FilesystemType::from("ntfs"),
            FilesystemType::Other(_)
        ));
    }

    #[test]
    fn test_mount_info_space_checks() {
        let mount = MountInfo {
            device: "/dev/sda1".to_string(),
            mount_point: "/".to_string(),
            fs_type: FilesystemType::Btrfs,
            options: vec!["rw".to_string()],
            total_bytes: 100_000_000_000,
            used_bytes: 95_000_000_000,
            available_bytes: 5_000_000_000,
            usage_percent: 95.0,
        };
        assert!(mount.is_space_critical());
        assert!(mount.is_space_warning());

        let mount2 = MountInfo {
            usage_percent: 85.0,
            ..mount.clone()
        };
        assert!(!mount2.is_space_critical());
        assert!(mount2.is_space_warning());

        let mount3 = MountInfo {
            usage_percent: 50.0,
            ..mount
        };
        assert!(!mount3.is_space_critical());
        assert!(!mount3.is_space_warning());
    }

    #[test]
    fn test_btrfs_device_stats_errors() {
        let mut stats = BtrfsDeviceStats::default();
        assert!(!stats.has_errors());
        assert_eq!(stats.total_errors(), 0);
        assert_eq!(stats.risk_level(), RiskLevel::Info);

        stats.write_io_errs = 5;
        assert!(stats.has_errors());
        assert_eq!(stats.total_errors(), 5);
        assert_eq!(stats.risk_level(), RiskLevel::Warning);

        stats.corruption_errs = 1;
        assert_eq!(stats.risk_level(), RiskLevel::Critical);
    }

    #[test]
    fn test_btrfs_usage_metadata_pressure() {
        let mut usage = BtrfsUsage {
            mount_point: "/".to_string(),
            metadata_total: 1_000_000_000,
            metadata_used: 500_000_000,
            ..Default::default()
        };
        assert_eq!(usage.metadata_usage_percent(), 50.0);
        assert!(!usage.is_metadata_warning());
        assert!(!usage.is_metadata_critical());

        usage.metadata_used = 850_000_000;
        assert_eq!(usage.metadata_usage_percent(), 85.0);
        assert!(usage.is_metadata_warning());
        assert!(!usage.is_metadata_critical());

        usage.metadata_used = 950_000_000;
        assert_eq!(usage.metadata_usage_percent(), 95.0);
        assert!(usage.is_metadata_warning());
        assert!(usage.is_metadata_critical());
    }

    #[test]
    fn test_smart_health_warnings() {
        let mut smart = SmartHealth {
            device: "/dev/sda".to_string(),
            model: Some("Test Drive".to_string()),
            serial: Some("12345".to_string()),
            passed: true,
            temperature_c: Some(35),
            power_on_hours: Some(10000),
            reallocated_sectors: Some(0),
            pending_sectors: Some(0),
            uncorrectable_sectors: Some(0),
            raw_read_error_rate: None,
            is_ssd: false,
            wear_percentage: None,
        };
        assert!(!smart.has_warnings());
        assert_eq!(smart.risk_level(), RiskLevel::Info);

        smart.passed = false;
        assert!(smart.has_warnings());
        assert_eq!(smart.risk_level(), RiskLevel::Critical);

        smart.passed = true;
        smart.reallocated_sectors = Some(5);
        assert!(smart.has_warnings());
        assert_eq!(smart.risk_level(), RiskLevel::Warning);

        smart.reallocated_sectors = Some(150);
        assert_eq!(smart.risk_level(), RiskLevel::Critical);
    }

    #[test]
    fn test_diagnosis_healthy_system() {
        let doctor = StorageDoctor::new();
        let mut evidence = create_test_evidence();
        evidence.btrfs_filesystems.push(create_healthy_btrfs());

        let result = doctor.diagnose(&evidence);
        assert_eq!(result.status, StorageHealth::Healthy);
        assert!(result.hypotheses.is_empty());
    }

    #[test]
    fn test_diagnosis_btrfs_device_errors() {
        let doctor = StorageDoctor::new();
        let mut evidence = create_test_evidence();
        let mut btrfs = create_healthy_btrfs();
        btrfs.device_stats[0].write_io_errs = 10;
        evidence.btrfs_filesystems.push(btrfs);

        let result = doctor.diagnose(&evidence);
        assert_eq!(result.status, StorageHealth::Degraded);
    }

    #[test]
    fn test_diagnosis_btrfs_corruption() {
        let doctor = StorageDoctor::new();
        let mut evidence = create_test_evidence();
        let mut btrfs = create_healthy_btrfs();
        btrfs.device_stats[0].corruption_errs = 1;
        evidence.btrfs_filesystems.push(btrfs);

        let result = doctor.diagnose(&evidence);
        assert_eq!(result.status, StorageHealth::Critical);
        assert!(!result.hypotheses.is_empty());
        assert!(result
            .hypotheses
            .iter()
            .any(|h| h.summary.contains("corruption")));
    }

    #[test]
    fn test_diagnosis_metadata_pressure() {
        let doctor = StorageDoctor::new();
        let mut evidence = create_test_evidence();
        let mut btrfs = create_healthy_btrfs();
        if let Some(usage) = &mut btrfs.usage {
            usage.metadata_used = usage.metadata_total - 10_000; // 99% full
        }
        evidence.btrfs_filesystems.push(btrfs);

        let result = doctor.diagnose(&evidence);
        assert_eq!(result.status, StorageHealth::Critical);
        assert!(result
            .hypotheses
            .iter()
            .any(|h| h.summary.contains("metadata")));
    }

    #[test]
    fn test_repair_plans_balance_blocked() {
        let doctor = StorageDoctor::new();
        let mut evidence = create_test_evidence();
        let mut btrfs = create_healthy_btrfs();
        if let Some(usage) = &mut btrfs.usage {
            usage.metadata_used = usage.metadata_total - 10_000;
        }
        evidence.btrfs_filesystems.push(btrfs);

        let diagnosis = doctor.diagnose(&evidence);
        let plans = doctor.generate_repair_plans(&diagnosis, &evidence);

        let balance_plan = plans.iter().find(|p| p.name.contains("Balance"));
        assert!(balance_plan.is_some());
        let plan = balance_plan.unwrap();
        assert!(plan.policy_blocked);
        assert!(plan.policy_block_reason.is_some());
    }

    #[test]
    fn test_repair_plans_scrub_allowed() {
        let mut doctor = StorageDoctor::new();
        doctor.allow_scrub = true;

        let mut evidence = create_test_evidence();
        let mut btrfs = create_healthy_btrfs();
        btrfs.scrub_status = ScrubStatus::Never;
        evidence.btrfs_filesystems.push(btrfs);

        let diagnosis = doctor.diagnose(&evidence);
        let plans = doctor.generate_repair_plans(&diagnosis, &evidence);

        let scrub_plan = plans.iter().find(|p| p.name.contains("Scrub"));
        assert!(scrub_plan.is_some());
        let plan = scrub_plan.unwrap();
        assert!(!plan.policy_blocked);
    }

    #[test]
    fn test_case_file_workflow() {
        let evidence = create_test_evidence();
        let mut case = StorageDoctorCase::new("case-001".to_string(), evidence);

        assert_eq!(case.status, CaseStatus::EvidenceCollected);

        let doctor = StorageDoctor::new();
        let diagnosis = doctor.diagnose(&case.evidence);
        case.set_diagnosis(diagnosis);
        assert_eq!(case.status, CaseStatus::Diagnosed);

        case.add_note("Reviewed findings".to_string(), "anna".to_string());
        assert_eq!(case.notes.len(), 1);

        case.complete_repair();
        assert_eq!(case.status, CaseStatus::RepairComplete);

        case.close();
        assert_eq!(case.status, CaseStatus::Closed);
    }

    #[test]
    fn test_scrub_status_uncorrected_errors() {
        let scrub_ok = ScrubStatus::Completed {
            finished: Some(Utc::now()),
            errors_found: 5,
            errors_corrected: 5,
        };
        assert!(!scrub_ok.has_uncorrected_errors());

        let scrub_bad = ScrubStatus::Completed {
            finished: Some(Utc::now()),
            errors_found: 5,
            errors_corrected: 3,
        };
        assert!(scrub_bad.has_uncorrected_errors());
    }

    #[test]
    fn test_evidence_overall_health() {
        let mut evidence = create_test_evidence();
        // Empty evidence with no collection errors is Healthy (nothing wrong detected)
        assert_eq!(evidence.overall_health(), StorageHealth::Healthy);

        // With collection errors, status is Unknown
        evidence
            .collection_errors
            .push("Failed to read SMART".to_string());
        assert_eq!(evidence.overall_health(), StorageHealth::Unknown);

        // Clear errors and add healthy BTRFS
        evidence.collection_errors.clear();
        evidence.btrfs_filesystems.push(create_healthy_btrfs());
        assert_eq!(evidence.overall_health(), StorageHealth::Healthy);

        evidence.io_errors.push(IoErrorLog {
            timestamp: Some(Utc::now()),
            device: Some("/dev/sda".to_string()),
            message: "I/O error".to_string(),
            error_type: IoErrorType::Read,
        });
        assert_eq!(evidence.overall_health(), StorageHealth::Degraded);
    }

    #[test]
    fn test_diagnosis_determines_correct_status() {
        let doctor = StorageDoctor::new();

        // All passed -> Healthy
        let mut evidence = create_test_evidence();
        evidence.collection_errors.clear();
        evidence.btrfs_filesystems.push(create_healthy_btrfs());
        let result = doctor.diagnose(&evidence);
        assert_eq!(result.status, StorageHealth::Healthy);

        // Non-critical failures -> Degraded
        let mut evidence2 = create_test_evidence();
        let mut btrfs = create_healthy_btrfs();
        btrfs.device_stats[0].read_io_errs = 5; // Warning level, not critical
        evidence2.btrfs_filesystems.push(btrfs);
        let result2 = doctor.diagnose(&evidence2);
        assert_eq!(result2.status, StorageHealth::Degraded);
    }

    #[test]
    fn test_max_three_hypotheses() {
        let doctor = StorageDoctor::new();
        let mut evidence = create_test_evidence();

        // Create conditions that would generate multiple hypotheses
        let mut btrfs = create_healthy_btrfs();
        btrfs.device_stats[0].corruption_errs = 1; // Corruption hypothesis
        if let Some(usage) = &mut btrfs.usage {
            usage.metadata_used = usage.metadata_total - 10_000; // Metadata hypothesis
        }
        btrfs.scrub_status = ScrubStatus::Completed {
            finished: Some(Utc::now()),
            errors_found: 10,
            errors_corrected: 5, // Uncorrected errors
        };
        evidence.btrfs_filesystems.push(btrfs);

        // Add SMART failure
        evidence.smart_health.push(SmartHealth {
            device: "/dev/sda".to_string(),
            model: None,
            serial: None,
            passed: false,
            temperature_c: None,
            power_on_hours: None,
            reallocated_sectors: None,
            pending_sectors: None,
            uncorrectable_sectors: None,
            raw_read_error_rate: None,
            is_ssd: false,
            wear_percentage: None,
        });

        let result = doctor.diagnose(&evidence);
        assert!(result.hypotheses.len() <= 3);
    }
}
