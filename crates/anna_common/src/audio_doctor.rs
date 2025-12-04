//! Audio Doctor - PipeWire-focused audio diagnosis with Fix-It playbooks
//!
//! v0.0.40: Arch Audio Doctor v1
//!
//! Deterministic diagnosis flow:
//! 1. Identify active audio stack (PipeWire vs PulseAudio vs ALSA-only)
//! 2. Verify services are running (pipewire + wireplumber)
//! 3. Confirm devices exist (ALSA sees hardware, PipeWire sees nodes)
//! 4. Confirm default sink/source and volume/mute state
//! 5. Check for common conflicts (PulseAudio running alongside PipeWire)
//! 6. For Bluetooth: verify bluetooth service, adapter, connection, profile

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Audio Stack Types
// ============================================================================

/// Detected audio stack
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioStack {
    /// PipeWire with WirePlumber (modern, recommended)
    PipeWire,
    /// PulseAudio (legacy)
    PulseAudio,
    /// ALSA only (no sound server)
    AlsaOnly,
    /// No audio stack detected
    None,
    /// Unknown/mixed state
    Unknown,
}

impl std::fmt::Display for AudioStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioStack::PipeWire => write!(f, "PipeWire"),
            AudioStack::PulseAudio => write!(f, "PulseAudio"),
            AudioStack::AlsaOnly => write!(f, "ALSA-only"),
            AudioStack::None => write!(f, "None"),
            AudioStack::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Audio health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioHealth {
    /// Audio working normally
    Healthy,
    /// Some issues but audio partially functional
    Degraded,
    /// Audio not working
    Broken,
    /// Cannot determine status
    Unknown,
}

impl std::fmt::Display for AudioHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioHealth::Healthy => write!(f, "Healthy"),
            AudioHealth::Degraded => write!(f, "Degraded"),
            AudioHealth::Broken => write!(f, "Broken"),
            AudioHealth::Unknown => write!(f, "Unknown"),
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
// Service State
// ============================================================================

/// State of a systemd service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceState {
    /// Service name
    pub name: String,
    /// Whether installed (unit file exists)
    pub installed: bool,
    /// Whether enabled
    pub enabled: bool,
    /// Whether active (running)
    pub active: bool,
    /// Whether it's a user service
    pub user_service: bool,
    /// Status message
    pub status: String,
    /// Recent log lines
    pub recent_logs: Vec<String>,
}

impl ServiceState {
    /// Create a not-installed state
    pub fn not_installed(name: &str, user_service: bool) -> Self {
        Self {
            name: name.to_string(),
            installed: false,
            enabled: false,
            active: false,
            user_service,
            status: "not installed".to_string(),
            recent_logs: Vec::new(),
        }
    }

    /// Check if service is healthy (installed, enabled, active)
    pub fn is_healthy(&self) -> bool {
        self.installed && self.enabled && self.active
    }
}

// ============================================================================
// Audio Devices
// ============================================================================

/// ALSA device (from aplay/arecord -l)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlsaDevice {
    /// Card number
    pub card: u32,
    /// Device number
    pub device: u32,
    /// Card name
    pub card_name: String,
    /// Device name
    pub device_name: String,
    /// Whether it's an output (playback) device
    pub is_output: bool,
}

/// PipeWire/PulseAudio node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioNode {
    /// Node ID
    pub id: u32,
    /// Node name
    pub name: String,
    /// Description (human-readable)
    pub description: String,
    /// Whether it's a sink (output) or source (input)
    pub is_sink: bool,
    /// Whether it's the default
    pub is_default: bool,
    /// Volume percentage (0-100+)
    pub volume_percent: Option<f32>,
    /// Whether muted
    pub muted: Option<bool>,
    /// State (running, idle, suspended)
    pub state: Option<String>,
}

impl AudioNode {
    /// Check if this node is problematic
    pub fn has_issues(&self) -> bool {
        self.muted.unwrap_or(false) || self.volume_percent.map(|v| v < 5.0).unwrap_or(false)
    }
}

// ============================================================================
// Bluetooth Audio
// ============================================================================

/// Bluetooth adapter state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BluetoothAdapter {
    /// Adapter path (e.g., /org/bluez/hci0)
    pub path: String,
    /// Adapter name
    pub name: String,
    /// Whether powered on
    pub powered: bool,
    /// Whether discoverable
    pub discoverable: bool,
    /// MAC address
    pub address: String,
}

/// Bluetooth audio device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BluetoothAudioDevice {
    /// Device path
    pub path: String,
    /// Device name
    pub name: String,
    /// MAC address
    pub address: String,
    /// Whether connected
    pub connected: bool,
    /// Whether paired
    pub paired: bool,
    /// Current audio profile
    pub profile: Option<BluetoothProfile>,
    /// Available profiles
    pub available_profiles: Vec<BluetoothProfile>,
}

/// Bluetooth audio profile
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BluetoothProfile {
    /// Advanced Audio Distribution Profile (high quality)
    A2dp,
    /// Hands-Free Profile (low quality, bidirectional)
    Hfp,
    /// Headset Profile (legacy, low quality)
    Hsp,
    /// Unknown profile
    Unknown,
}

impl std::fmt::Display for BluetoothProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BluetoothProfile::A2dp => write!(f, "A2DP"),
            BluetoothProfile::Hfp => write!(f, "HFP"),
            BluetoothProfile::Hsp => write!(f, "HSP"),
            BluetoothProfile::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Bluetooth subsystem state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BluetoothState {
    /// Bluetooth service state
    pub service: ServiceState,
    /// Adapters found
    pub adapters: Vec<BluetoothAdapter>,
    /// Audio devices (headphones, speakers, etc.)
    pub audio_devices: Vec<BluetoothAudioDevice>,
}

impl BluetoothState {
    /// Check if bluetooth is available and working
    pub fn is_available(&self) -> bool {
        self.service.active && !self.adapters.is_empty() && self.adapters.iter().any(|a| a.powered)
    }

    /// Get connected audio devices
    pub fn connected_audio_devices(&self) -> Vec<&BluetoothAudioDevice> {
        self.audio_devices.iter().filter(|d| d.connected).collect()
    }
}

// ============================================================================
// User Permissions
// ============================================================================

/// User group membership for audio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioPermissions {
    /// Target user
    pub username: String,
    /// User ID
    pub uid: u32,
    /// Member of 'audio' group
    pub in_audio_group: bool,
    /// Member of 'video' group
    pub in_video_group: bool,
    /// Member of 'bluetooth' group (for BT audio)
    pub in_bluetooth_group: bool,
    /// Has access to /dev/snd/* devices
    pub has_snd_access: bool,
}

impl AudioPermissions {
    /// Check if permissions look okay
    pub fn is_okay(&self) -> bool {
        // Modern systems with PipeWire/logind don't strictly need audio group
        // But we check device access
        self.has_snd_access
    }
}

// ============================================================================
// Evidence Bundle
// ============================================================================

/// Complete audio evidence bundle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioEvidence {
    /// Collection timestamp
    pub collected_at: DateTime<Utc>,
    /// Target user (for user services)
    pub target_user: String,
    /// Detected audio stack
    pub stack: AudioStack,
    /// PipeWire service state
    pub pipewire: ServiceState,
    /// WirePlumber service state
    pub wireplumber: ServiceState,
    /// PulseAudio service state (for conflict detection)
    pub pulseaudio: ServiceState,
    /// ALSA devices
    pub alsa_devices: Vec<AlsaDevice>,
    /// Audio nodes (sinks/sources)
    pub nodes: Vec<AudioNode>,
    /// Default sink (output)
    pub default_sink: Option<AudioNode>,
    /// Default source (input)
    pub default_source: Option<AudioNode>,
    /// Bluetooth state
    pub bluetooth: Option<BluetoothState>,
    /// User permissions
    pub permissions: AudioPermissions,
    /// Recent journal errors/warnings
    pub journal_errors: Vec<String>,
    /// Collection errors
    pub collection_errors: Vec<String>,
}

impl AudioEvidence {
    /// Create new empty evidence
    pub fn new(target_user: String) -> Self {
        Self {
            collected_at: Utc::now(),
            target_user: target_user.clone(),
            stack: AudioStack::Unknown,
            pipewire: ServiceState::not_installed("pipewire", true),
            wireplumber: ServiceState::not_installed("wireplumber", true),
            pulseaudio: ServiceState::not_installed("pulseaudio", true),
            alsa_devices: Vec::new(),
            nodes: Vec::new(),
            default_sink: None,
            default_source: None,
            bluetooth: None,
            permissions: AudioPermissions {
                username: target_user,
                uid: 0,
                in_audio_group: false,
                in_video_group: false,
                in_bluetooth_group: false,
                has_snd_access: false,
            },
            journal_errors: Vec::new(),
            collection_errors: Vec::new(),
        }
    }

    /// Check if PipeWire is the active stack
    pub fn is_pipewire(&self) -> bool {
        self.stack == AudioStack::PipeWire
    }

    /// Check if there's a PulseAudio conflict
    pub fn has_pulseaudio_conflict(&self) -> bool {
        self.is_pipewire() && self.pulseaudio.active
    }

    /// Check if services are healthy
    pub fn services_healthy(&self) -> bool {
        match self.stack {
            AudioStack::PipeWire => self.pipewire.active && self.wireplumber.active,
            AudioStack::PulseAudio => self.pulseaudio.active,
            AudioStack::AlsaOnly => true,
            _ => false,
        }
    }

    /// Check if there are output devices
    pub fn has_output_devices(&self) -> bool {
        !self
            .nodes
            .iter()
            .filter(|n| n.is_sink)
            .collect::<Vec<_>>()
            .is_empty()
            || !self
                .alsa_devices
                .iter()
                .filter(|d| d.is_output)
                .collect::<Vec<_>>()
                .is_empty()
    }

    /// Check if default sink is okay
    pub fn default_sink_okay(&self) -> bool {
        if let Some(sink) = &self.default_sink {
            !sink.muted.unwrap_or(false) && sink.volume_percent.map(|v| v >= 10.0).unwrap_or(true)
        } else {
            false
        }
    }

    /// Get overall health
    pub fn health(&self) -> AudioHealth {
        if self.stack == AudioStack::None {
            return AudioHealth::Broken;
        }

        if !self.services_healthy() {
            return AudioHealth::Broken;
        }

        if !self.has_output_devices() {
            return AudioHealth::Broken;
        }

        if self.has_pulseaudio_conflict() {
            return AudioHealth::Degraded;
        }

        if !self.default_sink_okay() {
            return AudioHealth::Degraded;
        }

        AudioHealth::Healthy
    }
}

// ============================================================================
// Diagnosis Flow
// ============================================================================

/// Diagnosis step result
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

/// A diagnosis step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisStep {
    /// Step name
    pub name: String,
    /// Step description
    pub description: String,
    /// Step number (1-based)
    pub step_number: u32,
    /// Result
    pub result: StepResult,
    /// Details/findings
    pub details: String,
    /// Implication
    pub implication: String,
    /// Evidence IDs referenced
    pub evidence_ids: Vec<String>,
}

/// A finding from diagnosis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Unique ID
    pub id: String,
    /// Summary
    pub summary: String,
    /// Detail
    pub detail: String,
    /// Risk level
    pub risk: RiskLevel,
    /// Evidence ID
    pub evidence_id: String,
}

/// A hypothesis about an audio issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioHypothesis {
    /// Hypothesis ID
    pub id: String,
    /// Summary
    pub summary: String,
    /// Explanation
    pub explanation: String,
    /// Confidence (0-100)
    pub confidence: u8,
    /// Supporting evidence IDs
    pub supporting_evidence: Vec<String>,
    /// Suggested playbook
    pub suggested_playbook: Option<String>,
}

/// Diagnosis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisResult {
    /// Overall health
    pub health: AudioHealth,
    /// Steps executed
    pub steps: Vec<DiagnosisStep>,
    /// Findings
    pub findings: Vec<Finding>,
    /// Hypotheses (max 3)
    pub hypotheses: Vec<AudioHypothesis>,
    /// Summary message
    pub summary: String,
    /// Timestamp
    pub diagnosed_at: DateTime<Utc>,
}

// ============================================================================
// Fix Playbooks
// ============================================================================

/// Fix playbook type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybookType {
    /// Restart user services
    RestartServices,
    /// Set default sink/source
    SetDefault,
    /// Unmute and set volume
    UnmuteVolume,
    /// Stop conflicting service
    StopConflict,
    /// Restart bluetooth
    RestartBluetooth,
    /// Set bluetooth profile
    SetBluetoothProfile,
}

impl std::fmt::Display for PlaybookType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlaybookType::RestartServices => write!(f, "Restart Services"),
            PlaybookType::SetDefault => write!(f, "Set Default"),
            PlaybookType::UnmuteVolume => write!(f, "Unmute/Volume"),
            PlaybookType::StopConflict => write!(f, "Stop Conflict"),
            PlaybookType::RestartBluetooth => write!(f, "Restart Bluetooth"),
            PlaybookType::SetBluetoothProfile => write!(f, "Set BT Profile"),
        }
    }
}

/// A command in a playbook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookCommand {
    /// Command to run
    pub command: String,
    /// Description
    pub description: String,
    /// Whether to run as target user
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
    /// Whether to run as user
    pub as_user: bool,
    /// Error message if fails
    pub error_message: String,
}

/// Post-execution check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostCheck {
    /// Check name
    pub name: String,
    /// Command to run
    pub command: String,
    /// Whether to run as user
    pub as_user: bool,
    /// Wait time before check
    pub wait_secs: u32,
    /// Expected result description
    pub expected: String,
}

/// A fix playbook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixPlaybook {
    /// Playbook ID
    pub id: String,
    /// Playbook type
    pub playbook_type: PlaybookType,
    /// Name
    pub name: String,
    /// Description
    pub description: String,
    /// Risk level
    pub risk: RiskLevel,
    /// Target user (for user services)
    pub target_user: String,
    /// Preflight checks
    pub preflight: Vec<PreflightCheck>,
    /// Commands to execute
    pub commands: Vec<PlaybookCommand>,
    /// Post-execution checks
    pub post_checks: Vec<PostCheck>,
    /// Rollback commands
    pub rollback: Vec<PlaybookCommand>,
    /// Whether blocked by policy
    pub policy_blocked: bool,
    /// Policy block reason
    pub policy_block_reason: Option<String>,
    /// Confirmation phrase required
    pub confirmation_phrase: String,
    /// Hypothesis this addresses
    pub addresses_hypothesis: Option<String>,
}

/// Result of executing a playbook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookResult {
    /// Playbook ID
    pub playbook_id: String,
    /// Whether successful
    pub success: bool,
    /// Commands executed with results
    pub command_results: Vec<CommandResult>,
    /// Post-check results
    pub post_check_results: Vec<CheckResult>,
    /// Error if failed
    pub error: Option<String>,
    /// Execution timestamp
    pub executed_at: DateTime<Utc>,
    /// Reliability score (for recipe capture)
    pub reliability_score: Option<u8>,
}

/// Result of a command
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
    /// Duration in ms
    pub duration_ms: u64,
}

/// Result of a check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    /// Check name
    pub name: String,
    /// Whether passed
    pub passed: bool,
    /// Output
    pub output: String,
}

// ============================================================================
// Recipe Capture
// ============================================================================

/// Recipe creation request from Audio Doctor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeCaptureRequest {
    /// Problem description
    pub problem: String,
    /// Solution description
    pub solution: String,
    /// Playbook that worked
    pub playbook_id: String,
    /// Preconditions
    pub preconditions: Vec<String>,
    /// Evidence patterns that indicate this problem
    pub evidence_patterns: Vec<String>,
    /// Reliability score
    pub reliability_score: u8,
}

// ============================================================================
// Case File
// ============================================================================

/// Audio doctor case file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDoctorCase {
    /// Case ID
    pub case_id: String,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Updated at
    pub updated_at: DateTime<Utc>,
    /// Target user
    pub target_user: String,
    /// Evidence collected
    pub evidence: AudioEvidence,
    /// Diagnosis result
    pub diagnosis: Option<DiagnosisResult>,
    /// Available playbooks
    pub playbooks: Vec<FixPlaybook>,
    /// Executed playbooks
    pub executed_playbooks: Vec<PlaybookResult>,
    /// Recipe capture requests (for successful fixes)
    pub recipe_requests: Vec<RecipeCaptureRequest>,
    /// Case status
    pub status: CaseStatus,
    /// Notes
    pub notes: Vec<CaseNote>,
}

/// Case status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaseStatus {
    EvidenceCollected,
    Diagnosed,
    PlaybookPending,
    PlaybookApplied,
    Resolved,
    Closed,
}

/// Case note
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseNote {
    pub timestamp: DateTime<Utc>,
    pub content: String,
    pub author: String,
}

impl AudioDoctorCase {
    /// Create new case
    pub fn new(case_id: String, evidence: AudioEvidence) -> Self {
        let target_user = evidence.target_user.clone();
        let now = Utc::now();
        Self {
            case_id,
            created_at: now,
            updated_at: now,
            target_user,
            evidence,
            diagnosis: None,
            playbooks: Vec::new(),
            executed_playbooks: Vec::new(),
            recipe_requests: Vec::new(),
            status: CaseStatus::EvidenceCollected,
            notes: Vec::new(),
        }
    }

    /// Set diagnosis
    pub fn set_diagnosis(&mut self, diagnosis: DiagnosisResult) {
        self.diagnosis = Some(diagnosis);
        self.status = CaseStatus::Diagnosed;
        self.updated_at = Utc::now();
    }

    /// Set available playbooks
    pub fn set_playbooks(&mut self, playbooks: Vec<FixPlaybook>) {
        self.playbooks = playbooks;
        if !self.playbooks.is_empty() {
            self.status = CaseStatus::PlaybookPending;
        }
        self.updated_at = Utc::now();
    }

    /// Record playbook execution
    pub fn record_playbook(&mut self, result: PlaybookResult) {
        let success = result.success;
        let reliability = result.reliability_score;

        // Check for recipe capture
        if success {
            if let Some(score) = reliability {
                if score >= 80 {
                    // Find the playbook and create recipe request
                    if let Some(pb) = self.playbooks.iter().find(|p| p.id == result.playbook_id) {
                        let request = RecipeCaptureRequest {
                            problem: format!(
                                "Audio issue: {}",
                                self.diagnosis
                                    .as_ref()
                                    .map(|d| d.summary.as_str())
                                    .unwrap_or("unknown")
                            ),
                            solution: pb.description.clone(),
                            playbook_id: pb.id.clone(),
                            preconditions: vec![
                                format!("Audio stack: {}", self.evidence.stack),
                                format!("Target user: {}", self.target_user),
                            ],
                            evidence_patterns: self
                                .diagnosis
                                .as_ref()
                                .map(|d| {
                                    d.hypotheses
                                        .iter()
                                        .flat_map(|h| h.supporting_evidence.clone())
                                        .collect()
                                })
                                .unwrap_or_default(),
                            reliability_score: score,
                        };
                        self.recipe_requests.push(request);
                    }
                }
            }
        }

        self.executed_playbooks.push(result);
        self.status = if success {
            CaseStatus::Resolved
        } else {
            CaseStatus::PlaybookApplied
        };
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

    /// Close case
    pub fn close(&mut self) {
        self.status = CaseStatus::Closed;
        self.updated_at = Utc::now();
    }
}

// ============================================================================
// Audio Doctor Engine
// ============================================================================

/// Confirmation phrase for fix playbooks
pub const FIX_CONFIRMATION: &str = "I CONFIRM";

/// Audio Doctor engine
pub struct AudioDoctor {
    /// Whether to allow stopping PulseAudio
    pub allow_stop_pulseaudio: bool,
    /// Whether to allow bluetooth service restart
    pub allow_bluetooth_restart: bool,
}

impl Default for AudioDoctor {
    fn default() -> Self {
        Self {
            allow_stop_pulseaudio: false,  // Higher risk, blocked by default
            allow_bluetooth_restart: true, // Allowed by default
        }
    }
}

impl AudioDoctor {
    /// Create new audio doctor
    pub fn new() -> Self {
        Self::default()
    }

    /// Run deterministic diagnosis flow
    pub fn diagnose(&self, evidence: &AudioEvidence) -> DiagnosisResult {
        let mut steps = Vec::new();
        let mut findings = Vec::new();

        // Step 1: Identify audio stack
        let step1 = self.step_identify_stack(evidence);
        findings.extend(self.findings_from_step(&step1, evidence));
        steps.push(step1);

        // Step 2: Verify services
        let step2 = self.step_verify_services(evidence);
        findings.extend(self.findings_from_step(&step2, evidence));
        steps.push(step2);

        // Step 3: Confirm devices exist
        let step3 = self.step_confirm_devices(evidence);
        findings.extend(self.findings_from_step(&step3, evidence));
        steps.push(step3);

        // Step 4: Check default sink/source
        let step4 = self.step_check_defaults(evidence);
        findings.extend(self.findings_from_step(&step4, evidence));
        steps.push(step4);

        // Step 5: Check for conflicts
        let step5 = self.step_check_conflicts(evidence);
        findings.extend(self.findings_from_step(&step5, evidence));
        steps.push(step5);

        // Step 6: Bluetooth (if relevant)
        let step6 = self.step_check_bluetooth(evidence);
        findings.extend(self.findings_from_step(&step6, evidence));
        steps.push(step6);

        // Generate hypotheses
        let hypotheses = self.generate_hypotheses(&findings, evidence);
        let health = self.determine_health(&steps);
        let summary = self.generate_summary(&health, &hypotheses, evidence);

        DiagnosisResult {
            health,
            steps,
            findings,
            hypotheses,
            summary,
            diagnosed_at: Utc::now(),
        }
    }

    /// Step 1: Identify audio stack
    fn step_identify_stack(&self, evidence: &AudioEvidence) -> DiagnosisStep {
        let (result, details, implication) = match evidence.stack {
            AudioStack::PipeWire => (
                StepResult::Pass,
                "PipeWire detected as active audio stack".to_string(),
                "Modern audio system. Will check PipeWire and WirePlumber services.".to_string(),
            ),
            AudioStack::PulseAudio => (
                StepResult::Pass,
                "PulseAudio detected as active audio stack".to_string(),
                "Legacy audio system. Consider migrating to PipeWire.".to_string(),
            ),
            AudioStack::AlsaOnly => (
                StepResult::Partial,
                "ALSA-only detected (no sound server)".to_string(),
                "Basic audio without session management. Some features limited.".to_string(),
            ),
            AudioStack::None => (
                StepResult::Fail,
                "No audio stack detected".to_string(),
                "Audio services not installed. Need to install PipeWire.".to_string(),
            ),
            AudioStack::Unknown => (
                StepResult::Fail,
                "Cannot determine audio stack".to_string(),
                "Mixed or broken configuration detected.".to_string(),
            ),
        };

        DiagnosisStep {
            name: "Identify Audio Stack".to_string(),
            description: "Detect PipeWire vs PulseAudio vs ALSA-only".to_string(),
            step_number: 1,
            result,
            details,
            implication,
            evidence_ids: vec![format!("ev-stack-{}", Utc::now().timestamp())],
        }
    }

    /// Step 2: Verify services
    fn step_verify_services(&self, evidence: &AudioEvidence) -> DiagnosisStep {
        let (result, details, implication) = match evidence.stack {
            AudioStack::PipeWire => {
                let pw_ok = evidence.pipewire.active;
                let wp_ok = evidence.wireplumber.active;

                if pw_ok && wp_ok {
                    (
                        StepResult::Pass,
                        "PipeWire and WirePlumber services running".to_string(),
                        "Audio services healthy.".to_string(),
                    )
                } else if pw_ok && !wp_ok {
                    (
                        StepResult::Partial,
                        format!(
                            "PipeWire running but WirePlumber {}",
                            if evidence.wireplumber.installed {
                                "stopped"
                            } else {
                                "not installed"
                            }
                        ),
                        "Session/policy management missing. Audio may not route correctly."
                            .to_string(),
                    )
                } else {
                    (
                        StepResult::Fail,
                        format!(
                            "PipeWire {} (installed: {})",
                            if evidence.pipewire.active {
                                "running"
                            } else {
                                "stopped"
                            },
                            evidence.pipewire.installed
                        ),
                        "Core audio service not running. Audio will not work.".to_string(),
                    )
                }
            }
            AudioStack::PulseAudio => {
                if evidence.pulseaudio.active {
                    (
                        StepResult::Pass,
                        "PulseAudio service running".to_string(),
                        "Legacy audio service healthy.".to_string(),
                    )
                } else {
                    (
                        StepResult::Fail,
                        "PulseAudio not running".to_string(),
                        "Audio service stopped.".to_string(),
                    )
                }
            }
            AudioStack::AlsaOnly => (
                StepResult::Skipped,
                "ALSA-only mode, no services to check".to_string(),
                "Direct ALSA access.".to_string(),
            ),
            _ => (
                StepResult::Fail,
                "No audio stack to verify".to_string(),
                "Cannot verify services.".to_string(),
            ),
        };

        DiagnosisStep {
            name: "Verify Services".to_string(),
            description: "Check if audio services are running".to_string(),
            step_number: 2,
            result,
            details,
            implication,
            evidence_ids: vec![format!("ev-services-{}", Utc::now().timestamp())],
        }
    }

    /// Step 3: Confirm devices exist
    fn step_confirm_devices(&self, evidence: &AudioEvidence) -> DiagnosisStep {
        let alsa_outputs = evidence.alsa_devices.iter().filter(|d| d.is_output).count();
        let alsa_inputs = evidence
            .alsa_devices
            .iter()
            .filter(|d| !d.is_output)
            .count();
        let node_sinks = evidence.nodes.iter().filter(|n| n.is_sink).count();
        let node_sources = evidence.nodes.iter().filter(|n| !n.is_sink).count();

        let (result, details, implication) = if alsa_outputs == 0 && node_sinks == 0 {
            (
                StepResult::Fail,
                "No output devices found".to_string(),
                "No speakers/headphones detected. Check hardware connection.".to_string(),
            )
        } else if node_sinks == 0 && evidence.is_pipewire() {
            (
                StepResult::Partial,
                format!(
                    "ALSA sees {} outputs but PipeWire has no sinks",
                    alsa_outputs
                ),
                "Hardware detected but PipeWire not exposing it. Service issue likely.".to_string(),
            )
        } else {
            (
                StepResult::Pass,
                format!(
                    "Found {} sink(s), {} source(s) (ALSA: {} out, {} in)",
                    node_sinks, node_sources, alsa_outputs, alsa_inputs
                ),
                "Audio hardware detected and accessible.".to_string(),
            )
        };

        DiagnosisStep {
            name: "Confirm Devices".to_string(),
            description: "Check if hardware is detected".to_string(),
            step_number: 3,
            result,
            details,
            implication,
            evidence_ids: vec![format!("ev-devices-{}", Utc::now().timestamp())],
        }
    }

    /// Step 4: Check default sink/source
    fn step_check_defaults(&self, evidence: &AudioEvidence) -> DiagnosisStep {
        let (result, details, implication) = match &evidence.default_sink {
            Some(sink) => {
                let muted = sink.muted.unwrap_or(false);
                let volume = sink.volume_percent.unwrap_or(100.0);

                if muted {
                    (
                        StepResult::Fail,
                        format!("Default sink '{}' is MUTED", sink.description),
                        "Output is muted. Unmute to hear audio.".to_string(),
                    )
                } else if volume < 10.0 {
                    (
                        StepResult::Partial,
                        format!(
                            "Default sink '{}' volume very low ({:.0}%)",
                            sink.description, volume
                        ),
                        "Volume too low to hear. Increase volume.".to_string(),
                    )
                } else {
                    (
                        StepResult::Pass,
                        format!(
                            "Default sink '{}' at {:.0}% volume, not muted",
                            sink.description, volume
                        ),
                        "Output configured correctly.".to_string(),
                    )
                }
            }
            None => (
                StepResult::Fail,
                "No default sink configured".to_string(),
                "No output device selected. Need to set default.".to_string(),
            ),
        };

        DiagnosisStep {
            name: "Check Default Output".to_string(),
            description: "Verify default sink and volume/mute state".to_string(),
            step_number: 4,
            result,
            details,
            implication,
            evidence_ids: vec![format!("ev-defaults-{}", Utc::now().timestamp())],
        }
    }

    /// Step 5: Check for conflicts
    fn step_check_conflicts(&self, evidence: &AudioEvidence) -> DiagnosisStep {
        let (result, details, implication) = if evidence.has_pulseaudio_conflict() {
            (
                StepResult::Fail,
                "PulseAudio running alongside PipeWire".to_string(),
                "Both audio servers running causes conflicts. Stop one.".to_string(),
            )
        } else if evidence.is_pipewire()
            && evidence.pulseaudio.installed
            && !evidence.pulseaudio.active
        {
            (
                StepResult::Pass,
                "PulseAudio installed but not running (no conflict)".to_string(),
                "PipeWire can provide PulseAudio compatibility.".to_string(),
            )
        } else {
            (
                StepResult::Pass,
                "No conflicts detected".to_string(),
                "Audio stack configuration clean.".to_string(),
            )
        };

        DiagnosisStep {
            name: "Check Conflicts".to_string(),
            description: "Detect PulseAudio/PipeWire conflicts".to_string(),
            step_number: 5,
            result,
            details,
            implication,
            evidence_ids: vec![format!("ev-conflicts-{}", Utc::now().timestamp())],
        }
    }

    /// Step 6: Check Bluetooth
    fn step_check_bluetooth(&self, evidence: &AudioEvidence) -> DiagnosisStep {
        let bt = match &evidence.bluetooth {
            Some(bt) => bt,
            None => {
                return DiagnosisStep {
                    name: "Check Bluetooth".to_string(),
                    description: "Verify Bluetooth audio".to_string(),
                    step_number: 6,
                    result: StepResult::Skipped,
                    details: "Bluetooth not checked (no BT evidence)".to_string(),
                    implication: "Bluetooth audio not in scope.".to_string(),
                    evidence_ids: vec![],
                };
            }
        };

        let (result, details, implication) = if !bt.service.active {
            (
                StepResult::Fail,
                "Bluetooth service not running".to_string(),
                "Bluetooth audio unavailable. Start bluetooth service.".to_string(),
            )
        } else if bt.adapters.is_empty() {
            (
                StepResult::Fail,
                "No Bluetooth adapters found".to_string(),
                "No BT hardware detected.".to_string(),
            )
        } else if !bt.adapters.iter().any(|a| a.powered) {
            (
                StepResult::Partial,
                "Bluetooth adapter(s) found but not powered".to_string(),
                "Turn on Bluetooth adapter.".to_string(),
            )
        } else {
            let connected = bt.connected_audio_devices();
            if connected.is_empty() {
                (
                    StepResult::Partial,
                    format!(
                        "Bluetooth active, {} audio device(s) paired but none connected",
                        bt.audio_devices.len()
                    ),
                    "Connect to a paired audio device.".to_string(),
                )
            } else {
                let dev = &connected[0];
                let profile_info = dev
                    .profile
                    .map(|p| format!(", profile: {}", p))
                    .unwrap_or_default();
                (
                    StepResult::Pass,
                    format!(
                        "Bluetooth audio device '{}' connected{}",
                        dev.name, profile_info
                    ),
                    "Bluetooth audio available.".to_string(),
                )
            }
        };

        DiagnosisStep {
            name: "Check Bluetooth".to_string(),
            description: "Verify Bluetooth audio".to_string(),
            step_number: 6,
            result,
            details,
            implication,
            evidence_ids: vec![format!("ev-bluetooth-{}", Utc::now().timestamp())],
        }
    }

    /// Extract findings from a step
    fn findings_from_step(&self, step: &DiagnosisStep, _evidence: &AudioEvidence) -> Vec<Finding> {
        let mut findings = Vec::new();

        if step.result == StepResult::Fail || step.result == StepResult::Partial {
            let risk = if step.result == StepResult::Fail {
                RiskLevel::High
            } else {
                RiskLevel::Medium
            };

            findings.push(Finding {
                id: format!(
                    "find-{}-{}",
                    step.name.to_lowercase().replace(' ', "-"),
                    Utc::now().timestamp()
                ),
                summary: step.details.clone(),
                detail: step.implication.clone(),
                risk,
                evidence_id: step.evidence_ids.first().cloned().unwrap_or_default(),
            });
        }

        findings
    }

    /// Generate hypotheses
    fn generate_hypotheses(
        &self,
        findings: &[Finding],
        evidence: &AudioEvidence,
    ) -> Vec<AudioHypothesis> {
        let mut hypotheses = Vec::new();

        // Hypothesis: PipeWire service not running
        if evidence.is_pipewire() && !evidence.pipewire.active {
            hypotheses.push(AudioHypothesis {
                id: format!("hyp-pw-stopped-{}", Utc::now().timestamp()),
                summary: "PipeWire user service not running".to_string(),
                explanation: "The PipeWire audio service is not active. This is required for \
                              audio to work on modern Arch systems."
                    .to_string(),
                confidence: 95,
                supporting_evidence: vec![format!("ev-services-{}", Utc::now().timestamp())],
                suggested_playbook: Some("restart_pipewire".to_string()),
            });
        }

        // Hypothesis: WirePlumber not running
        if evidence.is_pipewire() && evidence.pipewire.active && !evidence.wireplumber.active {
            hypotheses.push(AudioHypothesis {
                id: format!("hyp-wp-stopped-{}", Utc::now().timestamp()),
                summary: "WirePlumber session manager not running".to_string(),
                explanation:
                    "PipeWire is running but WirePlumber (session/policy manager) is not. \
                              Audio routing and device management require WirePlumber."
                        .to_string(),
                confidence: 90,
                supporting_evidence: vec![format!("ev-services-{}", Utc::now().timestamp())],
                suggested_playbook: Some("restart_wireplumber".to_string()),
            });
        }

        // Hypothesis: Muted or low volume
        if let Some(sink) = &evidence.default_sink {
            if sink.muted.unwrap_or(false) {
                hypotheses.push(AudioHypothesis {
                    id: format!("hyp-muted-{}", Utc::now().timestamp()),
                    summary: "Default output is muted".to_string(),
                    explanation: format!(
                        "The default sink '{}' is muted. Unmuting will restore audio.",
                        sink.description
                    ),
                    confidence: 95,
                    supporting_evidence: vec![format!("ev-defaults-{}", Utc::now().timestamp())],
                    suggested_playbook: Some("unmute_volume".to_string()),
                });
            } else if sink.volume_percent.map(|v| v < 10.0).unwrap_or(false) {
                hypotheses.push(AudioHypothesis {
                    id: format!("hyp-low-volume-{}", Utc::now().timestamp()),
                    summary: "Volume too low".to_string(),
                    explanation: format!(
                        "Default sink volume at {:.0}%, too low to hear.",
                        sink.volume_percent.unwrap_or(0.0)
                    ),
                    confidence: 85,
                    supporting_evidence: vec![format!("ev-defaults-{}", Utc::now().timestamp())],
                    suggested_playbook: Some("set_volume".to_string()),
                });
            }
        }

        // Hypothesis: No default sink
        if evidence.default_sink.is_none() && evidence.has_output_devices() {
            hypotheses.push(AudioHypothesis {
                id: format!("hyp-no-default-{}", Utc::now().timestamp()),
                summary: "No default output device set".to_string(),
                explanation: "Output devices exist but none is set as default. \
                              Setting a default sink should restore audio."
                    .to_string(),
                confidence: 85,
                supporting_evidence: vec![format!("ev-defaults-{}", Utc::now().timestamp())],
                suggested_playbook: Some("set_default_sink".to_string()),
            });
        }

        // Hypothesis: PulseAudio conflict
        if evidence.has_pulseaudio_conflict() {
            hypotheses.push(AudioHypothesis {
                id: format!("hyp-pa-conflict-{}", Utc::now().timestamp()),
                summary: "PulseAudio running alongside PipeWire".to_string(),
                explanation: "Both audio servers are running, causing conflicts. \
                              Stop PulseAudio to let PipeWire work properly."
                    .to_string(),
                confidence: 90,
                supporting_evidence: vec![format!("ev-conflicts-{}", Utc::now().timestamp())],
                suggested_playbook: Some("stop_pulseaudio".to_string()),
            });
        }

        // Hypothesis: Bluetooth profile wrong
        if let Some(bt) = &evidence.bluetooth {
            for dev in bt.connected_audio_devices() {
                if dev.profile == Some(BluetoothProfile::Hsp)
                    || dev.profile == Some(BluetoothProfile::Hfp)
                {
                    if dev.available_profiles.contains(&BluetoothProfile::A2dp) {
                        hypotheses.push(AudioHypothesis {
                            id: format!("hyp-bt-profile-{}", Utc::now().timestamp()),
                            summary: "Bluetooth using low-quality profile".to_string(),
                            explanation: format!(
                                "Device '{}' is using {} (low quality) instead of A2DP (high quality). \
                                 Switch to A2DP for better audio.",
                                dev.name, dev.profile.unwrap()),
                            confidence: 80,
                            supporting_evidence: vec![format!("ev-bluetooth-{}", Utc::now().timestamp())],
                            suggested_playbook: Some("set_bt_a2dp".to_string()),
                        });
                    }
                }
            }
        }

        // Hypothesis: Permission issue
        if !evidence.permissions.is_okay() {
            hypotheses.push(AudioHypothesis {
                id: format!("hyp-permissions-{}", Utc::now().timestamp()),
                summary: "Audio device permission issue".to_string(),
                explanation: format!(
                    "User '{}' may lack access to audio devices. Check group membership.",
                    evidence.permissions.username
                ),
                confidence: 60, // Lower confidence, modern systems often don't need it
                supporting_evidence: vec![format!("ev-permissions-{}", Utc::now().timestamp())],
                suggested_playbook: None, // No automatic fix for permissions
            });
        }

        // Sort by confidence, limit to 3
        hypotheses.sort_by(|a, b| b.confidence.cmp(&a.confidence));
        hypotheses.truncate(3);
        hypotheses
    }

    /// Determine overall health
    fn determine_health(&self, steps: &[DiagnosisStep]) -> AudioHealth {
        let fails = steps
            .iter()
            .filter(|s| s.result == StepResult::Fail)
            .count();
        let partials = steps
            .iter()
            .filter(|s| s.result == StepResult::Partial)
            .count();

        // Check for critical failures (services down = broken audio)
        let service_step_failed = steps
            .iter()
            .any(|s| s.step_number == 2 && s.result == StepResult::Fail);

        if fails >= 2 || service_step_failed {
            // Core service down or multiple failures = broken
            AudioHealth::Broken
        } else if fails == 1 {
            AudioHealth::Degraded
        } else if partials > 0 {
            AudioHealth::Degraded
        } else {
            AudioHealth::Healthy
        }
    }

    /// Generate summary
    fn generate_summary(
        &self,
        health: &AudioHealth,
        hypotheses: &[AudioHypothesis],
        evidence: &AudioEvidence,
    ) -> String {
        match health {
            AudioHealth::Healthy => format!(
                "Audio healthy. {} stack with {} output device(s).",
                evidence.stack,
                evidence.nodes.iter().filter(|n| n.is_sink).count()
            ),
            AudioHealth::Degraded => {
                if let Some(h) = hypotheses.first() {
                    format!(
                        "Audio degraded. Likely cause: {} ({}% confidence)",
                        h.summary, h.confidence
                    )
                } else {
                    "Audio degraded. Check findings for details.".to_string()
                }
            }
            AudioHealth::Broken => {
                if let Some(h) = hypotheses.first() {
                    format!(
                        "Audio NOT WORKING. Primary issue: {} ({}% confidence)",
                        h.summary, h.confidence
                    )
                } else {
                    "Audio not working. No clear hypothesis.".to_string()
                }
            }
            AudioHealth::Unknown => "Cannot determine audio health.".to_string(),
        }
    }

    /// Generate fix playbooks based on diagnosis
    pub fn generate_playbooks(
        &self,
        diagnosis: &DiagnosisResult,
        evidence: &AudioEvidence,
    ) -> Vec<FixPlaybook> {
        let mut playbooks = Vec::new();

        for hyp in &diagnosis.hypotheses {
            if let Some(pb_id) = &hyp.suggested_playbook {
                if let Some(pb) = self.create_playbook(pb_id, evidence, Some(&hyp.id)) {
                    playbooks.push(pb);
                }
            }
        }

        playbooks
    }

    /// Create a specific playbook
    fn create_playbook(
        &self,
        playbook_id: &str,
        evidence: &AudioEvidence,
        hypothesis_id: Option<&str>,
    ) -> Option<FixPlaybook> {
        let user = &evidence.target_user;

        match playbook_id {
            "restart_pipewire" => Some(FixPlaybook {
                id: "restart_pipewire".to_string(),
                playbook_type: PlaybookType::RestartServices,
                name: "Restart PipeWire Services".to_string(),
                description: "Restart pipewire and wireplumber user services".to_string(),
                risk: RiskLevel::Low,
                target_user: user.clone(),
                preflight: vec![PreflightCheck {
                    name: "PipeWire installed".to_string(),
                    command: "pacman -Q pipewire".to_string(),
                    as_user: false,
                    error_message: "PipeWire not installed".to_string(),
                }],
                commands: vec![PlaybookCommand {
                    command: "systemctl --user restart pipewire pipewire-pulse wireplumber"
                        .to_string(),
                    description: "Restart PipeWire stack".to_string(),
                    as_user: true,
                    timeout_secs: 10,
                }],
                post_checks: vec![
                    PostCheck {
                        name: "PipeWire running".to_string(),
                        command: "systemctl --user is-active pipewire".to_string(),
                        as_user: true,
                        wait_secs: 2,
                        expected: "active".to_string(),
                    },
                    PostCheck {
                        name: "WirePlumber running".to_string(),
                        command: "systemctl --user is-active wireplumber".to_string(),
                        as_user: true,
                        wait_secs: 1,
                        expected: "active".to_string(),
                    },
                ],
                rollback: vec![], // Restart is its own rollback
                policy_blocked: false,
                policy_block_reason: None,
                confirmation_phrase: FIX_CONFIRMATION.to_string(),
                addresses_hypothesis: hypothesis_id.map(String::from),
            }),

            "restart_wireplumber" => Some(FixPlaybook {
                id: "restart_wireplumber".to_string(),
                playbook_type: PlaybookType::RestartServices,
                name: "Restart WirePlumber".to_string(),
                description: "Restart wireplumber session manager".to_string(),
                risk: RiskLevel::Low,
                target_user: user.clone(),
                preflight: vec![PreflightCheck {
                    name: "WirePlumber installed".to_string(),
                    command: "pacman -Q wireplumber".to_string(),
                    as_user: false,
                    error_message: "WirePlumber not installed".to_string(),
                }],
                commands: vec![PlaybookCommand {
                    command: "systemctl --user restart wireplumber".to_string(),
                    description: "Restart WirePlumber".to_string(),
                    as_user: true,
                    timeout_secs: 10,
                }],
                post_checks: vec![PostCheck {
                    name: "WirePlumber running".to_string(),
                    command: "systemctl --user is-active wireplumber".to_string(),
                    as_user: true,
                    wait_secs: 2,
                    expected: "active".to_string(),
                }],
                rollback: vec![],
                policy_blocked: false,
                policy_block_reason: None,
                confirmation_phrase: FIX_CONFIRMATION.to_string(),
                addresses_hypothesis: hypothesis_id.map(String::from),
            }),

            "unmute_volume" | "set_volume" => Some(FixPlaybook {
                id: "unmute_volume".to_string(),
                playbook_type: PlaybookType::UnmuteVolume,
                name: "Unmute and Set Volume".to_string(),
                description: "Unmute default sink and set volume to 50%".to_string(),
                risk: RiskLevel::Low,
                target_user: user.clone(),
                preflight: vec![],
                commands: vec![
                    PlaybookCommand {
                        command: "wpctl set-mute @DEFAULT_AUDIO_SINK@ 0".to_string(),
                        description: "Unmute default sink".to_string(),
                        as_user: true,
                        timeout_secs: 5,
                    },
                    PlaybookCommand {
                        command: "wpctl set-volume @DEFAULT_AUDIO_SINK@ 0.5".to_string(),
                        description: "Set volume to 50%".to_string(),
                        as_user: true,
                        timeout_secs: 5,
                    },
                ],
                post_checks: vec![PostCheck {
                    name: "Volume check".to_string(),
                    command: "wpctl get-volume @DEFAULT_AUDIO_SINK@".to_string(),
                    as_user: true,
                    wait_secs: 1,
                    expected: "Volume: 0.50".to_string(),
                }],
                rollback: vec![],
                policy_blocked: false,
                policy_block_reason: None,
                confirmation_phrase: FIX_CONFIRMATION.to_string(),
                addresses_hypothesis: hypothesis_id.map(String::from),
            }),

            "set_default_sink" => {
                // Find first available sink
                let sink = evidence.nodes.iter().find(|n| n.is_sink)?;
                Some(FixPlaybook {
                    id: "set_default_sink".to_string(),
                    playbook_type: PlaybookType::SetDefault,
                    name: "Set Default Output".to_string(),
                    description: format!("Set '{}' as default output", sink.description),
                    risk: RiskLevel::Low,
                    target_user: user.clone(),
                    preflight: vec![],
                    commands: vec![PlaybookCommand {
                        command: format!("wpctl set-default {}", sink.id),
                        description: format!("Set sink {} as default", sink.id),
                        as_user: true,
                        timeout_secs: 5,
                    }],
                    post_checks: vec![PostCheck {
                        name: "Default set".to_string(),
                        command: "wpctl status | grep -A1 'Audio/Sink'".to_string(),
                        as_user: true,
                        wait_secs: 1,
                        expected: sink.description.clone(),
                    }],
                    rollback: vec![],
                    policy_blocked: false,
                    policy_block_reason: None,
                    confirmation_phrase: FIX_CONFIRMATION.to_string(),
                    addresses_hypothesis: hypothesis_id.map(String::from),
                })
            }

            "stop_pulseaudio" => Some(FixPlaybook {
                id: "stop_pulseaudio".to_string(),
                playbook_type: PlaybookType::StopConflict,
                name: "Stop PulseAudio".to_string(),
                description: "Stop and disable PulseAudio user service (conflict with PipeWire)"
                    .to_string(),
                risk: RiskLevel::Medium,
                target_user: user.clone(),
                preflight: vec![PreflightCheck {
                    name: "PulseAudio running".to_string(),
                    command: "systemctl --user is-active pulseaudio".to_string(),
                    as_user: true,
                    error_message: "PulseAudio not running".to_string(),
                }],
                commands: vec![
                    PlaybookCommand {
                        command: "systemctl --user stop pulseaudio.socket pulseaudio.service"
                            .to_string(),
                        description: "Stop PulseAudio".to_string(),
                        as_user: true,
                        timeout_secs: 10,
                    },
                    PlaybookCommand {
                        command: "systemctl --user mask pulseaudio.socket pulseaudio.service"
                            .to_string(),
                        description: "Mask PulseAudio to prevent restart".to_string(),
                        as_user: true,
                        timeout_secs: 5,
                    },
                ],
                post_checks: vec![PostCheck {
                    name: "PulseAudio stopped".to_string(),
                    command: "systemctl --user is-active pulseaudio || echo inactive".to_string(),
                    as_user: true,
                    wait_secs: 1,
                    expected: "inactive".to_string(),
                }],
                rollback: vec![PlaybookCommand {
                    command: "systemctl --user unmask pulseaudio.socket pulseaudio.service"
                        .to_string(),
                    description: "Unmask PulseAudio".to_string(),
                    as_user: true,
                    timeout_secs: 5,
                }],
                policy_blocked: !self.allow_stop_pulseaudio,
                policy_block_reason: if !self.allow_stop_pulseaudio {
                    Some("Stopping PulseAudio blocked by policy (medium risk)".to_string())
                } else {
                    None
                },
                confirmation_phrase: FIX_CONFIRMATION.to_string(),
                addresses_hypothesis: hypothesis_id.map(String::from),
            }),

            "set_bt_a2dp" => {
                let bt = evidence.bluetooth.as_ref()?;
                let connected = bt.connected_audio_devices();
                let dev = connected.first()?;
                Some(FixPlaybook {
                    id: "set_bt_a2dp".to_string(),
                    playbook_type: PlaybookType::SetBluetoothProfile,
                    name: "Set Bluetooth A2DP Profile".to_string(),
                    description: format!("Switch '{}' to high-quality A2DP profile", dev.name),
                    risk: RiskLevel::Low,
                    target_user: user.clone(),
                    preflight: vec![],
                    commands: vec![
                        PlaybookCommand {
                            // Find the node ID for this BT device and set profile
                            command: format!("wpctl set-profile $(wpctl status | grep -i '{}' | head -1 | awk '{{print $1}}' | tr -d '.') 1",
                                dev.name.replace(' ', ".*")),
                            description: "Set A2DP profile".to_string(),
                            as_user: true,
                            timeout_secs: 5,
                        },
                    ],
                    post_checks: vec![
                        PostCheck {
                            name: "Profile set".to_string(),
                            command: "wpctl status".to_string(),
                            as_user: true,
                            wait_secs: 2,
                            expected: "a2dp".to_string(),
                        },
                    ],
                    rollback: vec![],
                    policy_blocked: false,
                    policy_block_reason: None,
                    confirmation_phrase: FIX_CONFIRMATION.to_string(),
                    addresses_hypothesis: hypothesis_id.map(String::from),
                })
            }

            _ => None,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_evidence(user: &str) -> AudioEvidence {
        AudioEvidence::new(user.to_string())
    }

    fn create_healthy_pipewire_evidence() -> AudioEvidence {
        let mut evidence = create_test_evidence("testuser");
        evidence.stack = AudioStack::PipeWire;
        evidence.pipewire = ServiceState {
            name: "pipewire".to_string(),
            installed: true,
            enabled: true,
            active: true,
            user_service: true,
            status: "active".to_string(),
            recent_logs: vec![],
        };
        evidence.wireplumber = ServiceState {
            name: "wireplumber".to_string(),
            installed: true,
            enabled: true,
            active: true,
            user_service: true,
            status: "active".to_string(),
            recent_logs: vec![],
        };
        evidence.nodes.push(AudioNode {
            id: 1,
            name: "alsa_output.pci-0000_00_1f.3.analog-stereo".to_string(),
            description: "Built-in Audio Analog Stereo".to_string(),
            is_sink: true,
            is_default: true,
            volume_percent: Some(50.0),
            muted: Some(false),
            state: Some("running".to_string()),
        });
        evidence.default_sink = Some(evidence.nodes[0].clone());
        evidence.permissions.has_snd_access = true;
        evidence.collection_errors.clear();
        evidence
    }

    #[test]
    fn test_audio_stack_display() {
        assert_eq!(AudioStack::PipeWire.to_string(), "PipeWire");
        assert_eq!(AudioStack::PulseAudio.to_string(), "PulseAudio");
        assert_eq!(AudioStack::AlsaOnly.to_string(), "ALSA-only");
    }

    #[test]
    fn test_audio_health_display() {
        assert_eq!(AudioHealth::Healthy.to_string(), "Healthy");
        assert_eq!(AudioHealth::Broken.to_string(), "Broken");
    }

    #[test]
    fn test_service_state_healthy() {
        let mut svc = ServiceState::not_installed("test", true);
        assert!(!svc.is_healthy());

        svc.installed = true;
        svc.enabled = true;
        svc.active = true;
        assert!(svc.is_healthy());
    }

    #[test]
    fn test_audio_node_issues() {
        let mut node = AudioNode {
            id: 1,
            name: "test".to_string(),
            description: "Test".to_string(),
            is_sink: true,
            is_default: false,
            volume_percent: Some(50.0),
            muted: Some(false),
            state: None,
        };
        assert!(!node.has_issues());

        node.muted = Some(true);
        assert!(node.has_issues());

        node.muted = Some(false);
        node.volume_percent = Some(2.0);
        assert!(node.has_issues());
    }

    #[test]
    fn test_evidence_pulseaudio_conflict() {
        let mut evidence = create_healthy_pipewire_evidence();
        assert!(!evidence.has_pulseaudio_conflict());

        evidence.pulseaudio.active = true;
        assert!(evidence.has_pulseaudio_conflict());
    }

    #[test]
    fn test_evidence_health() {
        let evidence = create_healthy_pipewire_evidence();
        assert_eq!(evidence.health(), AudioHealth::Healthy);

        let mut broken = create_test_evidence("test");
        broken.stack = AudioStack::None;
        assert_eq!(broken.health(), AudioHealth::Broken);
    }

    #[test]
    fn test_diagnosis_healthy() {
        let doctor = AudioDoctor::new();
        let evidence = create_healthy_pipewire_evidence();
        let result = doctor.diagnose(&evidence);

        assert_eq!(result.health, AudioHealth::Healthy);
        assert!(result.hypotheses.is_empty());
    }

    #[test]
    fn test_diagnosis_pipewire_stopped() {
        let doctor = AudioDoctor::new();
        let mut evidence = create_healthy_pipewire_evidence();
        evidence.pipewire.active = false;

        let result = doctor.diagnose(&evidence);
        assert_eq!(result.health, AudioHealth::Broken);
        assert!(!result.hypotheses.is_empty());
        assert!(result.hypotheses[0].summary.contains("PipeWire"));
    }

    #[test]
    fn test_diagnosis_muted() {
        let doctor = AudioDoctor::new();
        let mut evidence = create_healthy_pipewire_evidence();
        evidence.default_sink.as_mut().unwrap().muted = Some(true);

        let result = doctor.diagnose(&evidence);
        assert_eq!(result.health, AudioHealth::Degraded);
        assert!(result
            .hypotheses
            .iter()
            .any(|h| h.summary.contains("muted")));
    }

    #[test]
    fn test_diagnosis_pulseaudio_conflict() {
        let doctor = AudioDoctor::new();
        let mut evidence = create_healthy_pipewire_evidence();
        evidence.pulseaudio = ServiceState {
            name: "pulseaudio".to_string(),
            installed: true,
            enabled: true,
            active: true,
            user_service: true,
            status: "active".to_string(),
            recent_logs: vec![],
        };

        let result = doctor.diagnose(&evidence);
        assert_eq!(result.health, AudioHealth::Degraded);
        assert!(result
            .hypotheses
            .iter()
            .any(|h| h.summary.contains("PulseAudio")));
    }

    #[test]
    fn test_playbook_generation() {
        let doctor = AudioDoctor::new();
        let mut evidence = create_healthy_pipewire_evidence();
        evidence.pipewire.active = false;

        let diagnosis = doctor.diagnose(&evidence);
        let playbooks = doctor.generate_playbooks(&diagnosis, &evidence);

        assert!(!playbooks.is_empty());
        assert!(playbooks.iter().any(|p| p.id == "restart_pipewire"));
    }

    #[test]
    fn test_playbook_pulseaudio_blocked() {
        let doctor = AudioDoctor::new();
        let mut evidence = create_healthy_pipewire_evidence();
        evidence.pulseaudio.active = true;

        let diagnosis = doctor.diagnose(&evidence);
        let playbooks = doctor.generate_playbooks(&diagnosis, &evidence);

        let pa_playbook = playbooks.iter().find(|p| p.id == "stop_pulseaudio");
        assert!(pa_playbook.is_some());
        assert!(pa_playbook.unwrap().policy_blocked);
    }

    #[test]
    fn test_case_file_workflow() {
        let evidence = create_healthy_pipewire_evidence();
        let mut case = AudioDoctorCase::new("case-001".to_string(), evidence);

        assert_eq!(case.status, CaseStatus::EvidenceCollected);

        let doctor = AudioDoctor::new();
        let diagnosis = doctor.diagnose(&case.evidence);
        case.set_diagnosis(diagnosis);
        assert_eq!(case.status, CaseStatus::Diagnosed);

        case.add_note("Test note".to_string(), "anna".to_string());
        assert_eq!(case.notes.len(), 1);

        case.close();
        assert_eq!(case.status, CaseStatus::Closed);
    }

    #[test]
    fn test_recipe_capture_on_success() {
        let mut evidence = create_healthy_pipewire_evidence();
        evidence.pipewire.active = false;

        let mut case = AudioDoctorCase::new("case-002".to_string(), evidence.clone());

        let doctor = AudioDoctor::new();
        let diagnosis = doctor.diagnose(&evidence);
        case.set_diagnosis(diagnosis.clone());

        let playbooks = doctor.generate_playbooks(&diagnosis, &evidence);
        case.set_playbooks(playbooks);

        // Simulate successful playbook with high reliability
        let result = PlaybookResult {
            playbook_id: "restart_pipewire".to_string(),
            success: true,
            command_results: vec![],
            post_check_results: vec![CheckResult {
                name: "PipeWire running".to_string(),
                passed: true,
                output: "active".to_string(),
            }],
            error: None,
            executed_at: Utc::now(),
            reliability_score: Some(85),
        };
        case.record_playbook(result);

        // Should have recipe capture request
        assert!(!case.recipe_requests.is_empty());
        assert!(case.recipe_requests[0].reliability_score >= 80);
    }

    #[test]
    fn test_bluetooth_profile_hypothesis() {
        let doctor = AudioDoctor::new();
        let mut evidence = create_healthy_pipewire_evidence();

        evidence.bluetooth = Some(BluetoothState {
            service: ServiceState {
                name: "bluetooth".to_string(),
                installed: true,
                enabled: true,
                active: true,
                user_service: false,
                status: "active".to_string(),
                recent_logs: vec![],
            },
            adapters: vec![BluetoothAdapter {
                path: "/org/bluez/hci0".to_string(),
                name: "hci0".to_string(),
                powered: true,
                discoverable: false,
                address: "00:11:22:33:44:55".to_string(),
            }],
            audio_devices: vec![BluetoothAudioDevice {
                path: "/org/bluez/hci0/dev_AA_BB_CC_DD_EE_FF".to_string(),
                name: "My Headphones".to_string(),
                address: "AA:BB:CC:DD:EE:FF".to_string(),
                connected: true,
                paired: true,
                profile: Some(BluetoothProfile::Hsp),
                available_profiles: vec![BluetoothProfile::Hsp, BluetoothProfile::A2dp],
            }],
        });

        let result = doctor.diagnose(&evidence);
        assert!(result
            .hypotheses
            .iter()
            .any(|h| h.summary.contains("low-quality profile")));
    }

    #[test]
    fn test_max_three_hypotheses() {
        let doctor = AudioDoctor::new();
        let mut evidence = create_healthy_pipewire_evidence();

        // Create multiple issues
        evidence.pipewire.active = false;
        evidence.wireplumber.active = false;
        evidence.default_sink.as_mut().unwrap().muted = Some(true);
        evidence.pulseaudio.active = true;
        evidence.permissions.has_snd_access = false;

        let result = doctor.diagnose(&evidence);
        assert!(result.hypotheses.len() <= 3);
    }
}
