//! Doctor Registry - Unified entry flow for Anna's diagnostic doctors
//!
//! v0.0.43: Doctor Registry + Unified Entry Flow
//!
//! This module provides:
//! - Data-driven doctor registration from TOML config
//! - Automatic doctor selection based on keywords, intent tags, and context
//! - Unified lifecycle for all doctor runs
//! - Consistent output schemas (doctor_run.json)
//! - Junior verification integration

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// ============================================================================
// Registry Configuration Types
// ============================================================================

/// Doctor registry configuration (loaded from TOML)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorRegistryConfig {
    /// Schema version
    pub schema_version: u32,
    /// Last modified timestamp
    pub last_modified: Option<String>,
    /// Doctor entries
    pub doctors: Vec<DoctorEntry>,
}

impl Default for DoctorRegistryConfig {
    fn default() -> Self {
        Self {
            schema_version: 1,
            last_modified: None,
            doctors: default_doctors(),
        }
    }
}

/// A single doctor entry in the registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorEntry {
    /// Unique identifier (e.g., "network_doctor")
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Brief description
    pub description: String,
    /// Domain this doctor handles
    pub domain: DoctorDomain,
    /// Keywords that trigger this doctor (lowercase)
    pub keywords: Vec<String>,
    /// Intent tags that match this doctor
    pub intent_tags: Vec<String>,
    /// Symptom patterns this doctor handles
    pub symptoms: Vec<String>,
    /// Required evidence bundles
    pub required_evidence: Vec<String>,
    /// Optional evidence bundles
    pub optional_evidence: Vec<String>,
    /// Tools required for this doctor
    pub required_tools: Vec<String>,
    /// Allowed playbook types
    pub allowed_playbooks: Vec<String>,
    /// Case file output filename
    pub case_file_name: String,
    /// Priority weight (higher = preferred when tied)
    pub priority: u32,
    /// Is this doctor enabled?
    pub enabled: bool,
}

/// Doctor domain categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DoctorDomain {
    Network,
    Storage,
    Audio,
    Boot,
    Graphics,
    System,
}

impl std::fmt::Display for DoctorDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DoctorDomain::Network => write!(f, "Network"),
            DoctorDomain::Storage => write!(f, "Storage"),
            DoctorDomain::Audio => write!(f, "Audio"),
            DoctorDomain::Boot => write!(f, "Boot"),
            DoctorDomain::Graphics => write!(f, "Graphics"),
            DoctorDomain::System => write!(f, "System"),
        }
    }
}

// ============================================================================
// Selection Types
// ============================================================================

/// Result of doctor selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorSelection {
    /// Primary doctor selected
    pub primary: SelectedDoctor,
    /// Optional secondary doctor (max 1 in v0.0.43)
    pub secondary: Option<SelectedDoctor>,
    /// Selection reasoning
    pub reasoning: String,
    /// Confidence in selection (0-100)
    pub confidence: u32,
    /// Keywords matched
    pub matched_keywords: Vec<String>,
    /// Intent tags matched
    pub matched_tags: Vec<String>,
}

/// A selected doctor with match details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectedDoctor {
    /// Doctor ID
    pub doctor_id: String,
    /// Doctor name
    pub doctor_name: String,
    /// Match score (0-100)
    pub match_score: u32,
    /// Why this doctor was selected
    pub match_reason: String,
    /// Evidence bundles to collect
    pub evidence_bundles: Vec<String>,
}

/// Selection match for ranking
#[derive(Debug, Clone)]
struct SelectionMatch {
    doctor_id: String,
    doctor_name: String,
    score: u32,
    keyword_matches: Vec<String>,
    tag_matches: Vec<String>,
    symptom_matches: Vec<String>,
    priority: u32,
    evidence_bundles: Vec<String>,
}

// ============================================================================
// Doctor Run Lifecycle Types
// ============================================================================

/// Stages of a doctor run
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DoctorRunStage {
    /// Doctor selection
    SelectDoctor,
    /// Collecting evidence bundle
    CollectEvidence,
    /// Running diagnosis flow
    DiagnosisFlow,
    /// Offering playbook
    PlaybookOffer,
    /// Applying fix
    ApplyFix,
    /// Verifying fix
    Verify,
    /// Closing and recipe capture
    Close,
}

impl std::fmt::Display for DoctorRunStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DoctorRunStage::SelectDoctor => write!(f, "select_doctor"),
            DoctorRunStage::CollectEvidence => write!(f, "collect_evidence"),
            DoctorRunStage::DiagnosisFlow => write!(f, "diagnosis_flow"),
            DoctorRunStage::PlaybookOffer => write!(f, "playbook_offer"),
            DoctorRunStage::ApplyFix => write!(f, "apply_fix"),
            DoctorRunStage::Verify => write!(f, "verify"),
            DoctorRunStage::Close => write!(f, "close"),
        }
    }
}

/// Timing for a stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageTiming {
    /// Stage name
    pub stage: DoctorRunStage,
    /// Start timestamp
    pub started_at: DateTime<Utc>,
    /// End timestamp
    pub ended_at: Option<DateTime<Utc>>,
    /// Duration in milliseconds
    pub duration_ms: Option<u64>,
    /// Status
    pub status: StageStatus,
    /// Notes
    pub notes: Option<String>,
}

/// Stage completion status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StageStatus {
    Pending,
    InProgress,
    Completed,
    Skipped,
    Failed,
}

impl std::fmt::Display for StageStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StageStatus::Pending => write!(f, "pending"),
            StageStatus::InProgress => write!(f, "in_progress"),
            StageStatus::Completed => write!(f, "completed"),
            StageStatus::Skipped => write!(f, "skipped"),
            StageStatus::Failed => write!(f, "failed"),
        }
    }
}

/// Doctor run result
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DoctorRunResult {
    /// Run completed successfully
    Success,
    /// Run completed but issue persists
    Partial,
    /// Run failed
    Failed,
    /// User cancelled
    Cancelled,
    /// Verification pending (needs reboot, etc.)
    VerificationPending,
}

impl std::fmt::Display for DoctorRunResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DoctorRunResult::Success => write!(f, "Success"),
            DoctorRunResult::Partial => write!(f, "Partial"),
            DoctorRunResult::Failed => write!(f, "Failed"),
            DoctorRunResult::Cancelled => write!(f, "Cancelled"),
            DoctorRunResult::VerificationPending => write!(f, "Verification Pending"),
        }
    }
}

/// Key finding from a doctor run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyFinding {
    /// Finding ID
    pub id: String,
    /// Description
    pub description: String,
    /// Severity
    pub severity: FindingSeverity,
    /// Evidence IDs supporting this finding
    pub evidence_ids: Vec<String>,
}

/// Finding severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl std::fmt::Display for FindingSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FindingSeverity::Info => write!(f, "Info"),
            FindingSeverity::Warning => write!(f, "Warning"),
            FindingSeverity::Error => write!(f, "Error"),
            FindingSeverity::Critical => write!(f, "Critical"),
        }
    }
}

// ============================================================================
// Doctor Run Output Schema
// ============================================================================

/// Schema version for doctor_run.json
pub const DOCTOR_RUN_SCHEMA_VERSION: u32 = 1;

/// Unified doctor run output (doctor_run.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorRun {
    /// Schema version
    pub schema_version: u32,
    /// Run ID
    pub run_id: String,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated
    pub updated_at: DateTime<Utc>,
    /// User request (original)
    pub user_request: String,
    /// Doctor selection
    pub selection: DoctorSelection,
    /// Current stage
    pub current_stage: DoctorRunStage,
    /// Stage timings
    pub stage_timings: Vec<StageTiming>,
    /// Key findings
    pub key_findings: Vec<KeyFinding>,
    /// Chosen playbook ID (if any)
    pub chosen_playbook: Option<String>,
    /// Playbook result (if applied)
    pub playbook_result: Option<PlaybookRunResult>,
    /// Verification status
    pub verification_status: VerificationStatus,
    /// Overall result
    pub result: Option<DoctorRunResult>,
    /// Reliability score (0-100)
    pub reliability: Option<u32>,
    /// Recipe captured (if applicable)
    pub recipe_captured: bool,
    /// Related case file
    pub case_file: Option<String>,
    /// Junior verification
    pub junior_verification: Option<JuniorVerification>,
}

/// Playbook run result (subset of doctor-specific result)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookRunResult {
    /// Playbook ID
    pub playbook_id: String,
    /// Success
    pub success: bool,
    /// Commands executed count
    pub commands_executed: u32,
    /// Post-checks passed
    pub post_checks_passed: bool,
    /// Error if any
    pub error: Option<String>,
}

/// Verification status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationStatus {
    /// Is verification complete?
    pub verified: bool,
    /// Pending verification reason
    pub pending_reason: Option<String>,
    /// Verification timestamp
    pub verified_at: Option<DateTime<Utc>>,
    /// Verification evidence IDs
    pub evidence_ids: Vec<String>,
}

/// Junior verification of doctor run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JuniorVerification {
    /// Did Junior approve the doctor choice?
    pub doctor_choice_approved: bool,
    /// Were diagnosis steps followed?
    pub diagnosis_steps_followed: bool,
    /// Is fix policy-compliant?
    pub fix_policy_compliant: Option<bool>,
    /// Is fix minimal?
    pub fix_minimal: Option<bool>,
    /// Is final claim evidence-backed?
    pub claim_evidence_backed: bool,
    /// Overall score
    pub score: u32,
    /// Critique
    pub critique: String,
}

impl DoctorRun {
    /// Create a new doctor run
    pub fn new(run_id: String, user_request: String, selection: DoctorSelection) -> Self {
        let now = Utc::now();
        Self {
            schema_version: DOCTOR_RUN_SCHEMA_VERSION,
            run_id,
            created_at: now,
            updated_at: now,
            user_request,
            selection,
            current_stage: DoctorRunStage::SelectDoctor,
            stage_timings: vec![StageTiming {
                stage: DoctorRunStage::SelectDoctor,
                started_at: now,
                ended_at: Some(now),
                duration_ms: Some(0),
                status: StageStatus::Completed,
                notes: None,
            }],
            key_findings: vec![],
            chosen_playbook: None,
            playbook_result: None,
            verification_status: VerificationStatus {
                verified: false,
                pending_reason: None,
                verified_at: None,
                evidence_ids: vec![],
            },
            result: None,
            reliability: None,
            recipe_captured: false,
            case_file: None,
            junior_verification: None,
        }
    }

    /// Start a new stage
    pub fn start_stage(&mut self, stage: DoctorRunStage) {
        self.current_stage = stage;
        self.stage_timings.push(StageTiming {
            stage,
            started_at: Utc::now(),
            ended_at: None,
            duration_ms: None,
            status: StageStatus::InProgress,
            notes: None,
        });
        self.updated_at = Utc::now();
    }

    /// Complete current stage
    pub fn complete_stage(&mut self, status: StageStatus, notes: Option<String>) {
        if let Some(timing) = self.stage_timings.last_mut() {
            let now = Utc::now();
            timing.ended_at = Some(now);
            timing.duration_ms = Some(
                (now - timing.started_at).num_milliseconds().max(0) as u64
            );
            timing.status = status;
            timing.notes = notes;
        }
        self.updated_at = Utc::now();
    }

    /// Add a key finding
    pub fn add_finding(&mut self, finding: KeyFinding) {
        self.key_findings.push(finding);
        self.updated_at = Utc::now();
    }

    /// Set playbook choice
    pub fn set_playbook(&mut self, playbook_id: String) {
        self.chosen_playbook = Some(playbook_id);
        self.updated_at = Utc::now();
    }

    /// Set playbook result
    pub fn set_playbook_result(&mut self, result: PlaybookRunResult) {
        self.playbook_result = Some(result);
        self.updated_at = Utc::now();
    }

    /// Set verification pending
    pub fn set_verification_pending(&mut self, reason: String) {
        self.verification_status.verified = false;
        self.verification_status.pending_reason = Some(reason);
        self.updated_at = Utc::now();
    }

    /// Mark as verified
    pub fn mark_verified(&mut self, evidence_ids: Vec<String>) {
        self.verification_status.verified = true;
        self.verification_status.verified_at = Some(Utc::now());
        self.verification_status.evidence_ids = evidence_ids;
        self.verification_status.pending_reason = None;
        self.updated_at = Utc::now();
    }

    /// Complete the run
    pub fn complete(&mut self, result: DoctorRunResult, reliability: u32) {
        self.result = Some(result);
        self.reliability = Some(reliability);
        self.updated_at = Utc::now();
    }

    /// Set Junior verification
    pub fn set_junior_verification(&mut self, verification: JuniorVerification) {
        self.junior_verification = Some(verification);
        self.updated_at = Utc::now();
    }

    /// Get total duration in milliseconds
    pub fn total_duration_ms(&self) -> u64 {
        self.stage_timings.iter()
            .filter_map(|t| t.duration_ms)
            .sum()
    }

    /// Save to file
    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Load from file
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let json = fs::read_to_string(path)?;
        let run: Self = serde_json::from_str(&json)?;
        Ok(run)
    }
}

// ============================================================================
// Doctor Registry
// ============================================================================

/// Path to the doctor registry config
pub const REGISTRY_CONFIG_PATH: &str = "/etc/anna/policy/doctors.toml";

/// Fallback path in user config
pub const REGISTRY_CONFIG_PATH_USER: &str = ".config/anna/doctors.toml";

/// Doctor run output directory
pub const DOCTOR_RUNS_DIR: &str = "/var/lib/anna/doctor_runs";

/// Doctor Registry - manages doctor entries and selection
pub struct DoctorRegistry {
    /// Registry configuration
    config: DoctorRegistryConfig,
    /// Doctor entries by ID
    doctors_by_id: HashMap<String, DoctorEntry>,
}

impl DoctorRegistry {
    /// Create a new registry with default doctors
    pub fn new() -> Self {
        let config = DoctorRegistryConfig::default();
        let doctors_by_id = config.doctors.iter()
            .map(|d| (d.id.clone(), d.clone()))
            .collect();
        Self { config, doctors_by_id }
    }

    /// Load registry from config file
    pub fn load() -> anyhow::Result<Self> {
        // Try system config first
        let config = if Path::new(REGISTRY_CONFIG_PATH).exists() {
            let toml_str = fs::read_to_string(REGISTRY_CONFIG_PATH)?;
            toml::from_str(&toml_str)?
        } else if let Some(home) = dirs::home_dir() {
            let user_path = home.join(REGISTRY_CONFIG_PATH_USER);
            if user_path.exists() {
                let toml_str = fs::read_to_string(user_path)?;
                toml::from_str(&toml_str)?
            } else {
                // Use defaults
                DoctorRegistryConfig::default()
            }
        } else {
            DoctorRegistryConfig::default()
        };

        let doctors_by_id = config.doctors.iter()
            .filter(|d| d.enabled)
            .map(|d| (d.id.clone(), d.clone()))
            .collect();

        Ok(Self { config, doctors_by_id })
    }

    /// Load from a specific path (for testing)
    pub fn load_from(path: &Path) -> anyhow::Result<Self> {
        let toml_str = fs::read_to_string(path)?;
        let config: DoctorRegistryConfig = toml::from_str(&toml_str)?;

        let doctors_by_id = config.doctors.iter()
            .filter(|d| d.enabled)
            .map(|d| (d.id.clone(), d.clone()))
            .collect();

        Ok(Self { config, doctors_by_id })
    }

    /// Get a doctor by ID
    pub fn get_doctor(&self, id: &str) -> Option<&DoctorEntry> {
        self.doctors_by_id.get(id)
    }

    /// Get all enabled doctors
    pub fn get_all_doctors(&self) -> Vec<&DoctorEntry> {
        self.doctors_by_id.values().collect()
    }

    /// Get doctors for a domain
    pub fn get_doctors_by_domain(&self, domain: DoctorDomain) -> Vec<&DoctorEntry> {
        self.doctors_by_id.values()
            .filter(|d| d.domain == domain)
            .collect()
    }

    /// Select doctor(s) for a user request
    ///
    /// Returns the best matching doctor as primary, and optionally a secondary
    /// if another domain is clearly relevant.
    pub fn select_doctors(&self, request: &str, intent_tags: &[String]) -> Option<DoctorSelection> {
        let request_lower = request.to_lowercase();
        let words: Vec<&str> = request_lower.split_whitespace().collect();

        // Score each doctor
        let mut matches: Vec<SelectionMatch> = self.doctors_by_id.values()
            .map(|doctor| self.score_doctor(doctor, &request_lower, &words, intent_tags))
            .filter(|m| m.score > 0)
            .collect();

        // Sort by score (descending), then priority (descending)
        matches.sort_by(|a, b| {
            b.score.cmp(&a.score)
                .then_with(|| b.priority.cmp(&a.priority))
        });

        if matches.is_empty() {
            return None;
        }

        // Primary doctor is the best match
        let primary = &matches[0];

        // Check for secondary (only if from different domain and score > 30)
        let secondary = matches.get(1)
            .filter(|m| {
                let primary_doctor = self.get_doctor(&primary.doctor_id);
                let secondary_doctor = self.get_doctor(&m.doctor_id);
                if let (Some(pd), Some(sd)) = (primary_doctor, secondary_doctor) {
                    pd.domain != sd.domain && m.score >= 30
                } else {
                    false
                }
            });

        // Build reasoning
        let reasoning = if let Some(sec) = secondary {
            format!(
                "Selected {} (score: {}) as primary based on keywords: {}. \
                 Also selected {} (score: {}) as secondary for related {} domain issues.",
                primary.doctor_name, primary.score,
                primary.keyword_matches.join(", "),
                sec.doctor_name, sec.score,
                self.get_doctor(&sec.doctor_id).map(|d| d.domain.to_string()).unwrap_or_default()
            )
        } else {
            format!(
                "Selected {} (score: {}) as primary based on keywords: {}{}",
                primary.doctor_name, primary.score,
                primary.keyword_matches.join(", "),
                if !primary.symptom_matches.is_empty() {
                    format!(" and symptoms: {}", primary.symptom_matches.join(", "))
                } else {
                    String::new()
                }
            )
        };

        // Collect all matched keywords and tags
        let mut all_keywords: Vec<String> = primary.keyword_matches.clone();
        let mut all_tags: Vec<String> = primary.tag_matches.clone();
        if let Some(sec) = secondary {
            all_keywords.extend(sec.keyword_matches.clone());
            all_tags.extend(sec.tag_matches.clone());
        }

        Some(DoctorSelection {
            primary: SelectedDoctor {
                doctor_id: primary.doctor_id.clone(),
                doctor_name: primary.doctor_name.clone(),
                match_score: primary.score,
                match_reason: format!("Matched keywords: {}", primary.keyword_matches.join(", ")),
                evidence_bundles: primary.evidence_bundles.clone(),
            },
            secondary: secondary.map(|s| SelectedDoctor {
                doctor_id: s.doctor_id.clone(),
                doctor_name: s.doctor_name.clone(),
                match_score: s.score,
                match_reason: format!("Matched keywords: {}", s.keyword_matches.join(", ")),
                evidence_bundles: s.evidence_bundles.clone(),
            }),
            reasoning,
            confidence: primary.score.min(100),
            matched_keywords: all_keywords,
            matched_tags: all_tags,
        })
    }

    /// Score a doctor against a request
    fn score_doctor(
        &self,
        doctor: &DoctorEntry,
        request_lower: &str,
        words: &[&str],
        intent_tags: &[String],
    ) -> SelectionMatch {
        let mut score: u32 = 0;
        let mut keyword_matches = vec![];
        let mut tag_matches = vec![];
        let mut symptom_matches = vec![];

        // Keyword matching (10 points per keyword)
        for keyword in &doctor.keywords {
            if words.iter().any(|w| w.contains(keyword) || keyword.contains(*w)) {
                score += 10;
                keyword_matches.push(keyword.clone());
            } else if request_lower.contains(keyword) {
                score += 5; // Partial match
                keyword_matches.push(keyword.clone());
            }
        }

        // Intent tag matching (15 points per tag)
        for tag in &doctor.intent_tags {
            if intent_tags.iter().any(|t| t == tag) {
                score += 15;
                tag_matches.push(tag.clone());
            }
        }

        // Symptom matching (20 points per symptom)
        for symptom in &doctor.symptoms {
            if request_lower.contains(symptom) {
                score += 20;
                symptom_matches.push(symptom.clone());
            }
        }

        // Only add priority bonus if we have at least one match
        // Otherwise, pure priority would make all doctors match anything
        if !keyword_matches.is_empty() || !tag_matches.is_empty() || !symptom_matches.is_empty() {
            score += doctor.priority / 10;
        }

        SelectionMatch {
            doctor_id: doctor.id.clone(),
            doctor_name: doctor.name.clone(),
            score,
            keyword_matches,
            tag_matches,
            symptom_matches,
            priority: doctor.priority,
            evidence_bundles: [
                doctor.required_evidence.clone(),
                doctor.optional_evidence.clone(),
            ].concat(),
        }
    }

    /// Generate a run ID
    pub fn generate_run_id(&self) -> String {
        format!("dr-{}", Utc::now().format("%Y%m%d-%H%M%S-%3f"))
    }

    /// Create a new doctor run
    pub fn create_run(&self, request: &str, selection: DoctorSelection) -> DoctorRun {
        let run_id = self.generate_run_id();
        DoctorRun::new(run_id, request.to_string(), selection)
    }
}

impl Default for DoctorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Default Doctor Entries
// ============================================================================

/// Generate default doctor entries
fn default_doctors() -> Vec<DoctorEntry> {
    vec![
        DoctorEntry {
            id: "network_doctor".to_string(),
            name: "Network Doctor".to_string(),
            description: "Diagnoses network connectivity, WiFi, DNS, and routing issues".to_string(),
            domain: DoctorDomain::Network,
            keywords: vec![
                "network".to_string(),
                "wifi".to_string(),
                "wireless".to_string(),
                "internet".to_string(),
                "ethernet".to_string(),
                "connection".to_string(),
                "dns".to_string(),
                "ping".to_string(),
                "router".to_string(),
                "ip".to_string(),
                "dhcp".to_string(),
                "vpn".to_string(),
                "firewall".to_string(),
                "disconnect".to_string(),
                "offline".to_string(),
            ],
            intent_tags: vec![
                "network_diagnosis".to_string(),
                "connectivity_issue".to_string(),
                "wifi_problem".to_string(),
            ],
            symptoms: vec![
                "no internet".to_string(),
                "wifi disconnecting".to_string(),
                "can't connect".to_string(),
                "network slow".to_string(),
                "dns not resolving".to_string(),
                "no connection".to_string(),
            ],
            required_evidence: vec![
                "interface_status".to_string(),
                "ip_addresses".to_string(),
                "routes".to_string(),
                "dns_config".to_string(),
            ],
            optional_evidence: vec![
                "wifi_signal".to_string(),
                "network_manager_status".to_string(),
                "firewall_rules".to_string(),
            ],
            required_tools: vec!["ip".to_string(), "ping".to_string()],
            allowed_playbooks: vec![
                "restart_networkmanager".to_string(),
                "renew_dhcp".to_string(),
                "flush_dns".to_string(),
            ],
            case_file_name: "networking_doctor.json".to_string(),
            priority: 80,
            enabled: true,
        },
        DoctorEntry {
            id: "storage_doctor".to_string(),
            name: "Storage Doctor".to_string(),
            description: "Diagnoses disk, BTRFS, filesystem, and SMART health issues".to_string(),
            domain: DoctorDomain::Storage,
            keywords: vec![
                "disk".to_string(),
                "storage".to_string(),
                "btrfs".to_string(),
                "filesystem".to_string(),
                "mount".to_string(),
                "partition".to_string(),
                "ssd".to_string(),
                "nvme".to_string(),
                "hdd".to_string(),
                "smart".to_string(),
                "raid".to_string(),
                "snapshot".to_string(),
                "space".to_string(),
                "full".to_string(),
            ],
            intent_tags: vec![
                "storage_diagnosis".to_string(),
                "disk_issue".to_string(),
                "filesystem_problem".to_string(),
            ],
            symptoms: vec![
                "disk full".to_string(),
                "no space".to_string(),
                "disk slow".to_string(),
                "mount failed".to_string(),
                "filesystem error".to_string(),
                "btrfs error".to_string(),
                "io error".to_string(),
            ],
            required_evidence: vec![
                "mount_points".to_string(),
                "disk_usage".to_string(),
                "block_devices".to_string(),
            ],
            optional_evidence: vec![
                "smart_status".to_string(),
                "btrfs_status".to_string(),
                "io_stats".to_string(),
            ],
            required_tools: vec!["lsblk".to_string(), "df".to_string()],
            allowed_playbooks: vec![
                "btrfs_scrub".to_string(),
                "btrfs_balance".to_string(),
                "clean_cache".to_string(),
            ],
            case_file_name: "storage_doctor.json".to_string(),
            priority: 70,
            enabled: true,
        },
        DoctorEntry {
            id: "audio_doctor".to_string(),
            name: "Audio Doctor".to_string(),
            description: "Diagnoses PipeWire, ALSA, Bluetooth audio, and sound issues".to_string(),
            domain: DoctorDomain::Audio,
            keywords: vec![
                "audio".to_string(),
                "sound".to_string(),
                "speaker".to_string(),
                "headphone".to_string(),
                "microphone".to_string(),
                "mic".to_string(),
                "volume".to_string(),
                "mute".to_string(),
                "pipewire".to_string(),
                "pulseaudio".to_string(),
                "alsa".to_string(),
                "bluetooth".to_string(),
                "airpods".to_string(),
            ],
            intent_tags: vec![
                "audio_diagnosis".to_string(),
                "sound_issue".to_string(),
                "audio_problem".to_string(),
            ],
            symptoms: vec![
                "no sound".to_string(),
                "no audio".to_string(),
                "sound not working".to_string(),
                "audio broken".to_string(),
                "can't hear".to_string(),
                "bluetooth audio".to_string(),
                "crackling".to_string(),
                "audio stuttering".to_string(),
            ],
            required_evidence: vec![
                "pipewire_status".to_string(),
                "audio_devices".to_string(),
                "default_sink".to_string(),
            ],
            optional_evidence: vec![
                "bluetooth_status".to_string(),
                "alsa_status".to_string(),
                "wireplumber_status".to_string(),
            ],
            required_tools: vec!["pactl".to_string(), "wpctl".to_string()],
            allowed_playbooks: vec![
                "restart_pipewire".to_string(),
                "restart_wireplumber".to_string(),
                "reconnect_bluetooth".to_string(),
            ],
            case_file_name: "audio_doctor.json".to_string(),
            priority: 75,
            enabled: true,
        },
        DoctorEntry {
            id: "boot_doctor".to_string(),
            name: "Boot Doctor".to_string(),
            description: "Diagnoses slow boot, service regressions, and startup issues".to_string(),
            domain: DoctorDomain::Boot,
            keywords: vec![
                "boot".to_string(),
                "startup".to_string(),
                "slow boot".to_string(),
                "systemd".to_string(),
                "service".to_string(),
                "reboot".to_string(),
                "shutdown".to_string(),
                "initramfs".to_string(),
                "grub".to_string(),
                "bootloader".to_string(),
            ],
            intent_tags: vec![
                "boot_diagnosis".to_string(),
                "startup_issue".to_string(),
                "boot_problem".to_string(),
            ],
            symptoms: vec![
                "slow boot".to_string(),
                "boot takes long".to_string(),
                "slow startup".to_string(),
                "stuck at boot".to_string(),
                "service failed".to_string(),
                "boot regression".to_string(),
            ],
            required_evidence: vec![
                "boot_timing".to_string(),
                "boot_blame".to_string(),
                "failed_units".to_string(),
            ],
            optional_evidence: vec![
                "boot_baseline".to_string(),
                "recent_changes".to_string(),
                "journal_boot".to_string(),
            ],
            required_tools: vec!["systemd-analyze".to_string(), "systemctl".to_string()],
            allowed_playbooks: vec![
                "disable_wait_online".to_string(),
                "restart_service".to_string(),
                "mask_service".to_string(),
            ],
            case_file_name: "boot_doctor.json".to_string(),
            priority: 65,
            enabled: true,
        },
        DoctorEntry {
            id: "graphics_doctor".to_string(),
            name: "Graphics Doctor".to_string(),
            description: "Diagnoses GPU, Wayland/X11, compositor, and portal issues".to_string(),
            domain: DoctorDomain::Graphics,
            keywords: vec![
                "graphics".to_string(),
                "gpu".to_string(),
                "display".to_string(),
                "screen".to_string(),
                "monitor".to_string(),
                "wayland".to_string(),
                "x11".to_string(),
                "xorg".to_string(),
                "nvidia".to_string(),
                "amd".to_string(),
                "intel".to_string(),
                "compositor".to_string(),
                "hyprland".to_string(),
                "sway".to_string(),
                "kde".to_string(),
                "gnome".to_string(),
                "portal".to_string(),
                "screen share".to_string(),
            ],
            intent_tags: vec![
                "graphics_diagnosis".to_string(),
                "display_issue".to_string(),
                "screen_problem".to_string(),
            ],
            symptoms: vec![
                "black screen".to_string(),
                "screen tearing".to_string(),
                "screen flickering".to_string(),
                "no display".to_string(),
                "screen share broken".to_string(),
                "can't share screen".to_string(),
                "graphics stutter".to_string(),
                "compositor crash".to_string(),
            ],
            required_evidence: vec![
                "session_type".to_string(),
                "compositor".to_string(),
                "gpu_info".to_string(),
                "driver_stack".to_string(),
            ],
            optional_evidence: vec![
                "portal_status".to_string(),
                "pipewire_status".to_string(),
                "compositor_logs".to_string(),
            ],
            required_tools: vec!["lspci".to_string()],
            allowed_playbooks: vec![
                "restart_portals".to_string(),
                "restart_pipewire_portals".to_string(),
                "collect_crash_report".to_string(),
            ],
            case_file_name: "graphics_doctor.json".to_string(),
            priority: 70,
            enabled: true,
        },
    ]
}

/// Generate default doctors.toml content
pub fn generate_default_config() -> String {
    let config = DoctorRegistryConfig::default();
    toml::to_string_pretty(&config).unwrap_or_default()
}

// ============================================================================
// Status Integration
// ============================================================================

/// Last doctor run summary (for annactl status)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastDoctorRunSummary {
    /// Doctor ID
    pub doctor_id: String,
    /// Doctor name
    pub doctor_name: String,
    /// Run timestamp
    pub run_at: DateTime<Utc>,
    /// Result
    pub result: DoctorRunResult,
    /// Reliability score
    pub reliability: u32,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

/// Doctor runs statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorRunStats {
    /// Last run summary
    pub last_run: Option<LastDoctorRunSummary>,
    /// Runs today count
    pub runs_today: u32,
    /// Success rate today (percentage)
    pub success_rate_today: u32,
}

/// Get doctor run stats from saved runs
pub fn get_doctor_run_stats() -> DoctorRunStats {
    let runs_dir = Path::new(DOCTOR_RUNS_DIR);

    if !runs_dir.exists() {
        return DoctorRunStats {
            last_run: None,
            runs_today: 0,
            success_rate_today: 0,
        };
    }

    let today = Utc::now().format("%Y%m%d").to_string();
    let mut runs_today = 0;
    let mut successes_today = 0;
    let mut last_run: Option<DoctorRun> = None;
    let mut last_run_time: Option<DateTime<Utc>> = None;

    // Read all doctor run files
    if let Ok(entries) = fs::read_dir(runs_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(run) = DoctorRun::load(&path) {
                    // Check if today
                    if run.created_at.format("%Y%m%d").to_string() == today {
                        runs_today += 1;
                        if matches!(run.result, Some(DoctorRunResult::Success)) {
                            successes_today += 1;
                        }
                    }

                    // Track latest run
                    if last_run_time.is_none() || run.created_at > last_run_time.unwrap() {
                        last_run_time = Some(run.created_at);
                        last_run = Some(run);
                    }
                }
            }
        }
    }

    let success_rate = if runs_today > 0 {
        ((successes_today as f32 / runs_today as f32) * 100.0) as u32
    } else {
        0
    };

    DoctorRunStats {
        last_run: last_run.map(|r| {
            let duration = r.total_duration_ms();
            LastDoctorRunSummary {
                doctor_id: r.selection.primary.doctor_id,
                doctor_name: r.selection.primary.doctor_name,
                run_at: r.created_at,
                result: r.result.unwrap_or(DoctorRunResult::Failed),
                reliability: r.reliability.unwrap_or(0),
                duration_ms: duration,
            }
        }),
        runs_today,
        success_rate_today: success_rate,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_registry() {
        let registry = DoctorRegistry::new();
        assert!(!registry.doctors_by_id.is_empty());
        assert!(registry.get_doctor("network_doctor").is_some());
        assert!(registry.get_doctor("audio_doctor").is_some());
        assert!(registry.get_doctor("boot_doctor").is_some());
        assert!(registry.get_doctor("graphics_doctor").is_some());
        assert!(registry.get_doctor("storage_doctor").is_some());
    }

    #[test]
    fn test_select_network_doctor() {
        let registry = DoctorRegistry::new();
        let selection = registry.select_doctors("wifi disconnecting", &[]);

        assert!(selection.is_some());
        let sel = selection.unwrap();
        assert_eq!(sel.primary.doctor_id, "network_doctor");
        assert!(sel.confidence > 0);
    }

    #[test]
    fn test_select_audio_doctor() {
        let registry = DoctorRegistry::new();
        let selection = registry.select_doctors("no sound", &[]);

        assert!(selection.is_some());
        let sel = selection.unwrap();
        assert_eq!(sel.primary.doctor_id, "audio_doctor");
    }

    #[test]
    fn test_select_boot_doctor() {
        let registry = DoctorRegistry::new();
        let selection = registry.select_doctors("boot is slow", &[]);

        assert!(selection.is_some());
        let sel = selection.unwrap();
        assert_eq!(sel.primary.doctor_id, "boot_doctor");
    }

    #[test]
    fn test_select_graphics_doctor() {
        let registry = DoctorRegistry::new();
        let selection = registry.select_doctors("screen share broken", &[]);

        assert!(selection.is_some());
        let sel = selection.unwrap();
        assert_eq!(sel.primary.doctor_id, "graphics_doctor");
    }

    #[test]
    fn test_select_storage_doctor() {
        let registry = DoctorRegistry::new();
        let selection = registry.select_doctors("disk full", &[]);

        assert!(selection.is_some());
        let sel = selection.unwrap();
        assert_eq!(sel.primary.doctor_id, "storage_doctor");
    }

    #[test]
    fn test_ambiguous_request_selects_one() {
        let registry = DoctorRegistry::new();
        // This could match multiple, but should pick best
        let selection = registry.select_doctors("my system is slow", &[]);

        // Should still pick one
        assert!(selection.is_some());
        let sel = selection.unwrap();
        assert!(!sel.primary.doctor_id.is_empty());
        assert!(!sel.reasoning.is_empty());
    }

    #[test]
    fn test_selection_with_intent_tags() {
        let registry = DoctorRegistry::new();
        let selection = registry.select_doctors(
            "help me fix this",
            &["audio_diagnosis".to_string()]
        );

        assert!(selection.is_some());
        let sel = selection.unwrap();
        assert_eq!(sel.primary.doctor_id, "audio_doctor");
    }

    #[test]
    fn test_no_match_returns_none() {
        let registry = DoctorRegistry::new();
        // Use something that definitely doesn't match any doctor keywords
        let selection = registry.select_doctors("make coffee please", &[]);

        // Random unrelated requests shouldn't match any doctor
        assert!(selection.is_none());
    }

    #[test]
    fn test_doctor_run_lifecycle() {
        let registry = DoctorRegistry::new();
        let selection = registry.select_doctors("no sound", &[]).unwrap();
        let mut run = registry.create_run("no sound", selection);

        // Start evidence collection
        run.start_stage(DoctorRunStage::CollectEvidence);
        assert_eq!(run.current_stage, DoctorRunStage::CollectEvidence);

        // Complete evidence collection
        run.complete_stage(StageStatus::Completed, Some("Collected 5 evidence items".to_string()));

        // Start diagnosis
        run.start_stage(DoctorRunStage::DiagnosisFlow);
        run.add_finding(KeyFinding {
            id: "pipewire-stopped".to_string(),
            description: "PipeWire service is not running".to_string(),
            severity: FindingSeverity::Error,
            evidence_ids: vec!["ev-audio-1".to_string()],
        });
        run.complete_stage(StageStatus::Completed, None);

        // Set playbook
        run.start_stage(DoctorRunStage::PlaybookOffer);
        run.set_playbook("restart_pipewire".to_string());
        run.complete_stage(StageStatus::Completed, None);

        // Complete
        run.complete(DoctorRunResult::Success, 85);

        assert_eq!(run.result, Some(DoctorRunResult::Success));
        assert_eq!(run.reliability, Some(85));
        assert_eq!(run.key_findings.len(), 1);
        assert!(run.total_duration_ms() >= 0);
    }

    #[test]
    fn test_doctor_domain_display() {
        assert_eq!(DoctorDomain::Network.to_string(), "Network");
        assert_eq!(DoctorDomain::Audio.to_string(), "Audio");
        assert_eq!(DoctorDomain::Graphics.to_string(), "Graphics");
    }

    #[test]
    fn test_stage_status_display() {
        assert_eq!(StageStatus::Completed.to_string(), "completed");
        assert_eq!(StageStatus::Failed.to_string(), "failed");
    }

    #[test]
    fn test_doctor_run_result_display() {
        assert_eq!(DoctorRunResult::Success.to_string(), "Success");
        assert_eq!(DoctorRunResult::VerificationPending.to_string(), "Verification Pending");
    }

    #[test]
    fn test_generate_run_id() {
        let registry = DoctorRegistry::new();
        let id1 = registry.generate_run_id();
        // Sleep a bit to ensure different timestamp
        std::thread::sleep(std::time::Duration::from_millis(2));
        let id2 = registry.generate_run_id();

        assert!(id1.starts_with("dr-"));
        assert!(id2.starts_with("dr-"));
        // IDs should be unique (different timestamps)
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_junior_verification() {
        let registry = DoctorRegistry::new();
        let selection = registry.select_doctors("no audio", &[]).unwrap();
        let mut run = registry.create_run("no audio", selection);

        run.set_junior_verification(JuniorVerification {
            doctor_choice_approved: true,
            diagnosis_steps_followed: true,
            fix_policy_compliant: Some(true),
            fix_minimal: Some(true),
            claim_evidence_backed: true,
            score: 90,
            critique: "Doctor selection appropriate. Fix is minimal and safe.".to_string(),
        });

        assert!(run.junior_verification.is_some());
        let jv = run.junior_verification.as_ref().unwrap();
        assert!(jv.doctor_choice_approved);
        assert_eq!(jv.score, 90);
    }

    #[test]
    fn test_verification_pending() {
        let registry = DoctorRegistry::new();
        let selection = registry.select_doctors("slow boot", &[]).unwrap();
        let mut run = registry.create_run("slow boot", selection);

        run.set_verification_pending("Requires reboot to verify boot time improvement".to_string());

        assert!(!run.verification_status.verified);
        assert!(run.verification_status.pending_reason.is_some());
    }

    #[test]
    fn test_mark_verified() {
        let registry = DoctorRegistry::new();
        let selection = registry.select_doctors("slow boot", &[]).unwrap();
        let mut run = registry.create_run("slow boot", selection);

        run.set_verification_pending("Requires reboot".to_string());
        run.mark_verified(vec!["ev-boot-post-1".to_string()]);

        assert!(run.verification_status.verified);
        assert!(run.verification_status.pending_reason.is_none());
        assert_eq!(run.verification_status.evidence_ids.len(), 1);
    }

    #[test]
    fn test_config_serialization() {
        let config = DoctorRegistryConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();

        assert!(toml_str.contains("network_doctor"));
        assert!(toml_str.contains("audio_doctor"));

        // Verify it can be parsed back
        let parsed: DoctorRegistryConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.doctors.len(), config.doctors.len());
    }

    #[test]
    fn test_secondary_doctor_different_domain() {
        let registry = DoctorRegistry::new();
        // A request that touches both audio and graphics (screen share with audio)
        let selection = registry.select_doctors("screen share audio not working", &[]);

        assert!(selection.is_some());
        let sel = selection.unwrap();
        // Primary should be one of them
        assert!(
            sel.primary.doctor_id == "graphics_doctor" ||
            sel.primary.doctor_id == "audio_doctor"
        );
    }

    #[test]
    fn test_doctors_by_domain() {
        let registry = DoctorRegistry::new();

        let network_docs = registry.get_doctors_by_domain(DoctorDomain::Network);
        assert_eq!(network_docs.len(), 1);
        assert_eq!(network_docs[0].id, "network_doctor");
    }

    #[test]
    fn test_selection_explains_why() {
        let registry = DoctorRegistry::new();
        let selection = registry.select_doctors("wifi keeps disconnecting", &[]).unwrap();

        // Reasoning should explain the selection
        assert!(selection.reasoning.contains("Network Doctor"));
        assert!(!selection.matched_keywords.is_empty());
    }
}
