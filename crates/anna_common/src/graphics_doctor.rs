//! Graphics Doctor - GPU/Wayland/X11 diagnosis with portal health checks
//!
//! v0.0.42: Arch GPU/Graphics Doctor v1
//!
//! Deterministic diagnosis flow:
//! 1. Identify session type (Wayland vs X11) and compositor
//! 2. Identify GPU vendor and loaded driver module
//! 3. Confirm key packages exist for that stack (mesa/nvidia + Vulkan)
//! 4. Check portal stack health (for screen share issues)
//! 5. Check logs for common failures (crash loops, missing backend)
//! 6. Produce findings vs hypotheses (max 3)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::process::Command;

// ============================================================================
// Session and Display Types
// ============================================================================

/// Display session type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionType {
    /// Wayland session
    Wayland,
    /// X11/Xorg session
    X11,
    /// TTY (no graphical session)
    Tty,
    /// Unknown/mixed
    Unknown,
}

impl std::fmt::Display for SessionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionType::Wayland => write!(f, "Wayland"),
            SessionType::X11 => write!(f, "X11"),
            SessionType::Tty => write!(f, "TTY"),
            SessionType::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Known compositors
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Compositor {
    /// Hyprland
    Hyprland,
    /// Sway
    Sway,
    /// KDE Plasma (kwin_wayland)
    KwinWayland,
    /// GNOME (mutter)
    Mutter,
    /// wlroots-based (generic)
    Wlroots,
    /// X11 window manager
    X11Wm(String),
    /// Unknown compositor
    Unknown,
    /// None (TTY)
    None,
}

impl std::fmt::Display for Compositor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Compositor::Hyprland => write!(f, "Hyprland"),
            Compositor::Sway => write!(f, "Sway"),
            Compositor::KwinWayland => write!(f, "KDE Plasma (kwin_wayland)"),
            Compositor::Mutter => write!(f, "GNOME (Mutter)"),
            Compositor::Wlroots => write!(f, "wlroots-based"),
            Compositor::X11Wm(name) => write!(f, "X11 WM: {}", name),
            Compositor::Unknown => write!(f, "Unknown"),
            Compositor::None => write!(f, "None"),
        }
    }
}

// ============================================================================
// GPU and Driver Types
// ============================================================================

/// GPU vendor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Other,
    Unknown,
}

impl std::fmt::Display for GpuVendor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpuVendor::Nvidia => write!(f, "NVIDIA"),
            GpuVendor::Amd => write!(f, "AMD"),
            GpuVendor::Intel => write!(f, "Intel"),
            GpuVendor::Other => write!(f, "Other"),
            GpuVendor::Unknown => write!(f, "Unknown"),
        }
    }
}

/// GPU information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    /// Vendor
    pub vendor: GpuVendor,
    /// Device name from lspci
    pub device_name: String,
    /// PCI slot
    pub pci_slot: String,
    /// Loaded kernel module
    pub kernel_module: Option<String>,
    /// Is this the primary GPU
    pub is_primary: bool,
}

/// Driver stack type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriverStack {
    /// NVIDIA proprietary
    NvidiaProprietary,
    /// NVIDIA open kernel modules
    NvidiaOpen,
    /// Nouveau (open source NVIDIA)
    Nouveau,
    /// AMD AMDGPU
    Amdgpu,
    /// AMD Radeon (legacy)
    Radeon,
    /// Intel i915
    I915,
    /// Intel Xe
    Xe,
    /// Unknown
    Unknown,
}

impl std::fmt::Display for DriverStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DriverStack::NvidiaProprietary => write!(f, "NVIDIA Proprietary"),
            DriverStack::NvidiaOpen => write!(f, "NVIDIA Open"),
            DriverStack::Nouveau => write!(f, "Nouveau"),
            DriverStack::Amdgpu => write!(f, "AMDGPU"),
            DriverStack::Radeon => write!(f, "Radeon"),
            DriverStack::I915 => write!(f, "Intel i915"),
            DriverStack::Xe => write!(f, "Intel Xe"),
            DriverStack::Unknown => write!(f, "Unknown"),
        }
    }
}

// ============================================================================
// Portal Types
// ============================================================================

/// XDG portal backend
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortalBackend {
    /// GTK portal
    Gtk,
    /// KDE portal
    Kde,
    /// GNOME portal
    Gnome,
    /// wlr (wlroots) portal
    Wlr,
    /// Hyprland portal
    Hyprland,
    /// LXQt portal
    Lxqt,
    /// Unknown
    Unknown,
    /// None detected
    None,
}

impl std::fmt::Display for PortalBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortalBackend::Gtk => write!(f, "GTK"),
            PortalBackend::Kde => write!(f, "KDE"),
            PortalBackend::Gnome => write!(f, "GNOME"),
            PortalBackend::Wlr => write!(f, "wlr"),
            PortalBackend::Hyprland => write!(f, "Hyprland"),
            PortalBackend::Lxqt => write!(f, "LXQt"),
            PortalBackend::Unknown => write!(f, "Unknown"),
            PortalBackend::None => write!(f, "None"),
        }
    }
}

/// Portal service state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortalState {
    /// Service name
    pub service: String,
    /// Is installed
    pub installed: bool,
    /// Is running
    pub running: bool,
    /// Is enabled
    pub enabled: bool,
    /// Error message if any
    pub error: Option<String>,
}

// ============================================================================
// Health and Risk Types
// ============================================================================

/// Graphics health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphicsHealth {
    /// Graphics working normally
    Healthy,
    /// Some issues but functional
    Degraded,
    /// Major issues detected
    Broken,
    /// Cannot determine status
    Unknown,
}

impl std::fmt::Display for GraphicsHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphicsHealth::Healthy => write!(f, "Healthy"),
            GraphicsHealth::Degraded => write!(f, "Degraded"),
            GraphicsHealth::Broken => write!(f, "Broken"),
            GraphicsHealth::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Risk level for findings and playbooks
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    Info,
    Low,
    Medium,
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
// Evidence Types
// ============================================================================

/// Session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    /// Session type
    pub session_type: SessionType,
    /// Compositor
    pub compositor: Compositor,
    /// XDG_SESSION_TYPE env var
    pub xdg_session_type: Option<String>,
    /// WAYLAND_DISPLAY env var
    pub wayland_display: Option<String>,
    /// DISPLAY env var (X11)
    pub display: Option<String>,
    /// XDG_CURRENT_DESKTOP
    pub xdg_current_desktop: Option<String>,
    /// Hyprland instance signature
    pub hyprland_instance: Option<String>,
}

/// Driver packages information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverPackages {
    /// NVIDIA packages
    pub nvidia: Vec<String>,
    /// Mesa packages
    pub mesa: Vec<String>,
    /// Vulkan packages
    pub vulkan: Vec<String>,
    /// LibVA packages
    pub libva: Vec<String>,
    /// VDPAU packages
    pub vdpau: Vec<String>,
}

/// Monitor information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorInfo {
    /// Monitor name/identifier
    pub name: String,
    /// Resolution
    pub resolution: String,
    /// Refresh rate
    pub refresh_rate: Option<String>,
    /// Is primary
    pub is_primary: bool,
    /// Is enabled
    pub enabled: bool,
}

/// Log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Priority
    pub priority: String,
    /// Unit/source
    pub unit: String,
    /// Message
    pub message: String,
}

/// Complete graphics evidence bundle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsEvidence {
    /// Collected timestamp
    pub collected_at: DateTime<Utc>,
    /// Session information
    pub session: SessionInfo,
    /// GPU inventory
    pub gpus: Vec<GpuInfo>,
    /// Primary driver stack
    pub driver_stack: DriverStack,
    /// Driver packages
    pub packages: DriverPackages,
    /// Portal states
    pub portals: Vec<PortalState>,
    /// Active portal backend
    pub portal_backend: PortalBackend,
    /// PipeWire status (for screen sharing)
    pub pipewire_running: bool,
    /// WirePlumber status
    pub wireplumber_running: bool,
    /// Monitors
    pub monitors: Vec<MonitorInfo>,
    /// Recent compositor logs
    pub compositor_logs: Vec<LogEntry>,
    /// Recent portal logs
    pub portal_logs: Vec<LogEntry>,
    /// Environment variables relevant to graphics
    pub env_vars: Vec<(String, String)>,
    /// Evidence ID prefix
    pub evidence_id: String,
    /// Target user
    pub target_user: Option<String>,
}

impl GraphicsEvidence {
    /// Determine overall graphics health
    pub fn health(&self) -> GraphicsHealth {
        // Check for broken conditions
        if self.session.session_type == SessionType::Unknown {
            return GraphicsHealth::Unknown;
        }

        // No GPU detected
        if self.gpus.is_empty() {
            return GraphicsHealth::Broken;
        }

        // Check portal health for Wayland
        if self.session.session_type == SessionType::Wayland {
            let portal_running = self.portals.iter().any(|p| p.running);
            if !portal_running {
                return GraphicsHealth::Degraded;
            }

            // Wrong portal backend for compositor
            if !self.is_portal_backend_correct() {
                return GraphicsHealth::Degraded;
            }
        }

        // Check for driver loaded
        if self.gpus.iter().all(|g| g.kernel_module.is_none()) {
            return GraphicsHealth::Broken;
        }

        // Check for crash indicators in logs
        let crash_keywords = ["crash", "segfault", "SIGSEGV", "fatal", "GPU hang"];
        let has_crashes = self.compositor_logs.iter().any(|l| {
            crash_keywords
                .iter()
                .any(|k| l.message.to_lowercase().contains(&k.to_lowercase()))
        });
        if has_crashes {
            return GraphicsHealth::Degraded;
        }

        GraphicsHealth::Healthy
    }

    /// Check if portal backend matches compositor
    pub fn is_portal_backend_correct(&self) -> bool {
        match (&self.session.compositor, &self.portal_backend) {
            // Hyprland: needs hyprland or wlr portal
            (Compositor::Hyprland, PortalBackend::Hyprland) => true,
            (Compositor::Hyprland, PortalBackend::Wlr) => true,
            (Compositor::Hyprland, _) => false, // Other portals don't work with Hyprland
            // Sway: needs wlr portal
            (Compositor::Sway, PortalBackend::Wlr) => true,
            (Compositor::Sway, _) => false,
            // KDE: needs kde portal
            (Compositor::KwinWayland, PortalBackend::Kde) => true,
            (Compositor::KwinWayland, _) => false,
            // GNOME: needs gnome or gtk portal
            (Compositor::Mutter, PortalBackend::Gnome) => true,
            (Compositor::Mutter, PortalBackend::Gtk) => true,
            (Compositor::Mutter, _) => false,
            // wlroots-based: needs wlr portal
            (Compositor::Wlroots, PortalBackend::Wlr) => true,
            (Compositor::Wlroots, _) => false,
            // X11: gtk or kde work
            (Compositor::X11Wm(_), PortalBackend::Gtk) => true,
            (Compositor::X11Wm(_), PortalBackend::Kde) => true,
            (Compositor::X11Wm(_), _) => false,
            // Unknown/None portal is always wrong
            (_, PortalBackend::Unknown) => false,
            (_, PortalBackend::None) => false,
            // Unknown compositor - be lenient
            (Compositor::Unknown, _) => true,
            (Compositor::None, _) => true, // TTY doesn't need portals
        }
    }

    /// Get expected portal backend for compositor
    pub fn expected_portal_backend(&self) -> PortalBackend {
        match &self.session.compositor {
            Compositor::Hyprland => PortalBackend::Hyprland,
            Compositor::Sway => PortalBackend::Wlr,
            Compositor::KwinWayland => PortalBackend::Kde,
            Compositor::Mutter => PortalBackend::Gnome,
            Compositor::Wlroots => PortalBackend::Wlr,
            Compositor::X11Wm(_) => PortalBackend::Gtk,
            _ => PortalBackend::Unknown,
        }
    }
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
    /// Step number
    pub step_number: u32,
    /// Result
    pub result: StepResult,
    /// Detailed findings
    pub details: String,
    /// What this means
    pub implication: String,
    /// Evidence IDs cited
    pub evidence_ids: Vec<String>,
}

/// A finding (fact) from diagnosis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Finding ID
    pub id: String,
    /// Description
    pub description: String,
    /// Risk level
    pub risk: RiskLevel,
    /// Related component
    pub component: Option<String>,
    /// Evidence IDs
    pub evidence_ids: Vec<String>,
}

/// A hypothesis about the cause
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsHypothesis {
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
    /// Suggested playbook
    pub suggested_playbook: Option<String>,
}

/// Complete diagnosis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisResult {
    /// Overall health
    pub health: GraphicsHealth,
    /// Diagnosis steps
    pub steps: Vec<DiagnosisStep>,
    /// Findings (facts)
    pub findings: Vec<Finding>,
    /// Hypotheses (theories)
    pub hypotheses: Vec<GraphicsHypothesis>,
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
    /// Restart portal services
    RestartPortals,
    /// Restart PipeWire
    RestartPipewire,
    /// Restart compositor (high risk)
    RestartCompositor,
    /// Restart display manager (high risk)
    RestartDisplayManager,
    /// Set environment variable
    SetEnvVar,
    /// Collect crash report
    CollectCrashReport,
}

/// A command in a playbook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookCommand {
    /// The command
    pub command: String,
    /// Description
    pub description: String,
    /// Run as target user
    pub as_user: bool,
    /// Timeout seconds
    pub timeout_secs: u32,
}

/// Preflight check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightCheck {
    /// Check name
    pub name: String,
    /// Command to run
    pub command: String,
    /// Expected pattern
    pub expected_pattern: Option<String>,
    /// Error message
    pub error_message: String,
}

/// Post-execution check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostCheck {
    /// Check name
    pub name: String,
    /// Command to run
    pub command: String,
    /// Expected pattern
    pub expected_pattern: Option<String>,
    /// Wait time before check
    pub wait_secs: u32,
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
    /// Target user
    pub target_user: Option<String>,
    /// Preflight checks
    pub preflight: Vec<PreflightCheck>,
    /// Commands
    pub commands: Vec<PlaybookCommand>,
    /// Post-checks
    pub post_checks: Vec<PostCheck>,
    /// Rollback commands
    pub rollback: Vec<PlaybookCommand>,
    /// Confirmation phrase
    pub confirmation_phrase: String,
    /// Which hypothesis this addresses
    pub addresses_hypothesis: Option<String>,
    /// Policy blocked
    pub policy_blocked: bool,
    /// Policy block reason
    pub policy_block_reason: Option<String>,
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
    /// Error message
    pub error: Option<String>,
    /// Reliability score
    pub reliability: u32,
}

/// Result of a command
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
    /// Recipe name
    pub name: String,
    /// Problem description
    pub problem: String,
    /// Solution description
    pub solution: String,
    /// Preconditions
    pub preconditions: Vec<String>,
    /// Playbook ID
    pub playbook_id: String,
    /// Reliability achieved
    pub reliability: u32,
    /// Evidence patterns
    pub evidence_patterns: Vec<String>,
}

// ============================================================================
// Case File
// ============================================================================

/// Graphics doctor case file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsDoctorCase {
    /// Case ID
    pub case_id: String,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated
    pub updated_at: DateTime<Utc>,
    /// Status
    pub status: CaseStatus,
    /// Evidence bundle
    pub evidence: GraphicsEvidence,
    /// Diagnosis result
    pub diagnosis: Option<DiagnosisResult>,
    /// Applied playbook
    pub applied_playbook: Option<PlaybookResult>,
    /// Recipe capture request
    pub recipe_capture: Option<RecipeCaptureRequest>,
    /// Notes
    pub notes: Vec<CaseNote>,
}

/// Case status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaseStatus {
    EvidenceCollected,
    Diagnosed,
    FixApplied,
    Verified,
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

impl GraphicsDoctorCase {
    /// Create new case
    pub fn new(evidence: GraphicsEvidence) -> Self {
        let case_id = format!("graphics-{}", Utc::now().format("%Y%m%d-%H%M%S"));
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
        self.applied_playbook = Some(result);
        self.status = CaseStatus::FixApplied;
        self.updated_at = Utc::now();
    }

    /// Add recipe capture
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
// Graphics Doctor Engine
// ============================================================================

/// Graphics Doctor - diagnoses GPU, compositor, and portal issues
pub struct GraphicsDoctor {
    /// Max hypotheses to generate
    max_hypotheses: usize,
}

impl GraphicsDoctor {
    /// Create new Graphics Doctor
    pub fn new() -> Self {
        Self { max_hypotheses: 3 }
    }

    /// Collect graphics evidence
    pub fn collect_evidence(&self, target_user: Option<&str>) -> GraphicsEvidence {
        let evidence_id = format!("ev-graphics-{}", Utc::now().timestamp());

        // Collect session info
        let session = self.collect_session_info();

        // Collect GPU info
        let gpus = self.collect_gpu_info();

        // Determine driver stack
        let driver_stack = self.determine_driver_stack(&gpus);

        // Collect packages
        let packages = self.collect_driver_packages();

        // Collect portal states
        let portals = self.collect_portal_states(target_user);

        // Determine portal backend
        let portal_backend = self.determine_portal_backend(&portals);

        // Check PipeWire/WirePlumber
        let (pipewire_running, wireplumber_running) = self.check_audio_services(target_user);

        // Collect monitors
        let monitors = self.collect_monitors(&session);

        // Collect logs
        let compositor_logs = self.collect_compositor_logs(&session);
        let portal_logs = self.collect_portal_logs(target_user);

        // Collect relevant env vars
        let env_vars = self.collect_env_vars();

        GraphicsEvidence {
            collected_at: Utc::now(),
            session,
            gpus,
            driver_stack,
            packages,
            portals,
            portal_backend,
            pipewire_running,
            wireplumber_running,
            monitors,
            compositor_logs,
            portal_logs,
            env_vars,
            evidence_id,
            target_user: target_user.map(String::from),
        }
    }

    /// Collect session information
    fn collect_session_info(&self) -> SessionInfo {
        // Get environment variables
        let xdg_session_type = std::env::var("XDG_SESSION_TYPE").ok();
        let wayland_display = std::env::var("WAYLAND_DISPLAY").ok();
        let display = std::env::var("DISPLAY").ok();
        let xdg_current_desktop = std::env::var("XDG_CURRENT_DESKTOP").ok();
        let hyprland_instance = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").ok();

        // Determine session type
        let session_type = match xdg_session_type.as_deref() {
            Some("wayland") => SessionType::Wayland,
            Some("x11") => SessionType::X11,
            Some("tty") => SessionType::Tty,
            _ => {
                if wayland_display.is_some() {
                    SessionType::Wayland
                } else if display.is_some() {
                    SessionType::X11
                } else {
                    SessionType::Unknown
                }
            }
        };

        // Determine compositor
        let compositor = if hyprland_instance.is_some() {
            Compositor::Hyprland
        } else if std::env::var("SWAYSOCK").is_ok() {
            Compositor::Sway
        } else {
            match xdg_current_desktop.as_deref() {
                Some(d) if d.to_lowercase().contains("kde") => Compositor::KwinWayland,
                Some(d) if d.to_lowercase().contains("gnome") => Compositor::Mutter,
                Some(d) if d.to_lowercase().contains("hyprland") => Compositor::Hyprland,
                Some(d) if d.to_lowercase().contains("sway") => Compositor::Sway,
                _ => {
                    if session_type == SessionType::Wayland {
                        Compositor::Wlroots // Assume wlroots-based if unknown Wayland
                    } else if session_type == SessionType::X11 {
                        // Try to detect X11 WM
                        let wm = self.detect_x11_wm();
                        if wm.is_empty() {
                            Compositor::Unknown
                        } else {
                            Compositor::X11Wm(wm)
                        }
                    } else {
                        Compositor::None
                    }
                }
            }
        };

        SessionInfo {
            session_type,
            compositor,
            xdg_session_type,
            wayland_display,
            display,
            xdg_current_desktop,
            hyprland_instance,
        }
    }

    /// Detect X11 window manager
    fn detect_x11_wm(&self) -> String {
        // Try wmctrl
        if let Ok(output) = Command::new("wmctrl").arg("-m").output() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.starts_with("Name:") {
                    return line.trim_start_matches("Name:").trim().to_string();
                }
            }
        }
        String::new()
    }

    /// Collect GPU information from lspci
    fn collect_gpu_info(&self) -> Vec<GpuInfo> {
        let mut gpus = vec![];

        let output = Command::new("lspci")
            .args(["-nn", "-d", "::0300"]) // VGA compatible controllers
            .output();

        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                if let Some(gpu) = self.parse_lspci_gpu_line(line, true) {
                    gpus.push(gpu);
                }
            }
        }

        // Also check 3D controllers (NVIDIA discrete usually)
        let output = Command::new("lspci").args(["-nn", "-d", "::0302"]).output();

        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                if let Some(gpu) = self.parse_lspci_gpu_line(line, false) {
                    gpus.push(gpu);
                }
            }
        }

        // Detect loaded kernel modules for each GPU
        for gpu in &mut gpus {
            gpu.kernel_module = self.detect_gpu_kernel_module(&gpu.vendor);
        }

        gpus
    }

    /// Parse lspci GPU line
    fn parse_lspci_gpu_line(&self, line: &str, is_primary: bool) -> Option<GpuInfo> {
        // Format: "00:02.0 VGA compatible controller [0300]: Intel Corporation ..."
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        if parts.len() < 2 {
            return None;
        }

        let pci_slot = parts[0].to_string();
        let rest = parts[1];

        // Determine vendor
        let vendor = if rest.to_lowercase().contains("nvidia") {
            GpuVendor::Nvidia
        } else if rest.to_lowercase().contains("amd") || rest.to_lowercase().contains("radeon") {
            GpuVendor::Amd
        } else if rest.to_lowercase().contains("intel") {
            GpuVendor::Intel
        } else {
            GpuVendor::Other
        };

        // Extract device name (after the class description)
        let device_name = if let Some(idx) = rest.find("]:") {
            rest[idx + 2..].trim().to_string()
        } else {
            rest.to_string()
        };

        Some(GpuInfo {
            vendor,
            device_name,
            pci_slot,
            kernel_module: None,
            is_primary,
        })
    }

    /// Detect which kernel module is loaded for a GPU vendor
    fn detect_gpu_kernel_module(&self, vendor: &GpuVendor) -> Option<String> {
        let modules_to_check = match vendor {
            GpuVendor::Nvidia => vec!["nvidia", "nvidia_drm", "nouveau"],
            GpuVendor::Amd => vec!["amdgpu", "radeon"],
            GpuVendor::Intel => vec!["i915", "xe"],
            _ => vec![],
        };

        for module in modules_to_check {
            let output = Command::new("lsmod").output().ok()?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.lines().any(|l| l.starts_with(module)) {
                return Some(module.to_string());
            }
        }

        None
    }

    /// Determine driver stack from GPUs
    fn determine_driver_stack(&self, gpus: &[GpuInfo]) -> DriverStack {
        for gpu in gpus {
            if let Some(module) = &gpu.kernel_module {
                return match module.as_str() {
                    "nvidia" | "nvidia_drm" => DriverStack::NvidiaProprietary,
                    "nvidia_open" => DriverStack::NvidiaOpen,
                    "nouveau" => DriverStack::Nouveau,
                    "amdgpu" => DriverStack::Amdgpu,
                    "radeon" => DriverStack::Radeon,
                    "i915" => DriverStack::I915,
                    "xe" => DriverStack::Xe,
                    _ => DriverStack::Unknown,
                };
            }
        }
        DriverStack::Unknown
    }

    /// Collect driver packages
    fn collect_driver_packages(&self) -> DriverPackages {
        let check_packages = |patterns: &[&str]| -> Vec<String> {
            let mut found = vec![];
            for pattern in patterns {
                let output = Command::new("pacman").args(["-Qq", pattern]).output();
                if let Ok(out) = output {
                    if out.status.success() {
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        for line in stdout.lines() {
                            if !line.is_empty() {
                                found.push(line.to_string());
                            }
                        }
                    }
                }
            }
            found
        };

        DriverPackages {
            nvidia: check_packages(&["nvidia", "nvidia-open", "nvidia-dkms", "nvidia-utils"]),
            mesa: check_packages(&["mesa", "lib32-mesa"]),
            vulkan: check_packages(&[
                "vulkan-icd-loader",
                "vulkan-intel",
                "vulkan-radeon",
                "nvidia-utils",
            ]),
            libva: check_packages(&[
                "libva",
                "libva-mesa-driver",
                "libva-intel-driver",
                "libva-nvidia-driver",
            ]),
            vdpau: check_packages(&["libvdpau", "mesa-vdpau"]),
        }
    }

    /// Collect portal states
    fn collect_portal_states(&self, target_user: Option<&str>) -> Vec<PortalState> {
        let portals = [
            "xdg-desktop-portal",
            "xdg-desktop-portal-gtk",
            "xdg-desktop-portal-kde",
            "xdg-desktop-portal-gnome",
            "xdg-desktop-portal-wlr",
            "xdg-desktop-portal-hyprland",
        ];

        let mut states = vec![];

        for portal in portals {
            let service = format!("{}.service", portal);

            // Check if installed
            let installed = Command::new("pacman")
                .args(["-Qq", portal])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            // Check service status (user service)
            let (running, enabled, error) = self.check_user_service(&service, target_user);

            states.push(PortalState {
                service,
                installed,
                running,
                enabled,
                error,
            });
        }

        states
    }

    /// Check user service status
    fn check_user_service(
        &self,
        service: &str,
        target_user: Option<&str>,
    ) -> (bool, bool, Option<String>) {
        let user_arg = target_user.map(|u| format!("--user={}", u));

        // Check if active
        let mut cmd = Command::new("systemctl");
        cmd.arg("--user").arg("is-active").arg(service);
        if let Some(ref ua) = user_arg {
            cmd.arg(ua);
        }
        let running = cmd.output().map(|o| o.status.success()).unwrap_or(false);

        // Check if enabled
        let mut cmd = Command::new("systemctl");
        cmd.arg("--user").arg("is-enabled").arg(service);
        if let Some(ref ua) = user_arg {
            cmd.arg(ua);
        }
        let enabled = cmd.output().map(|o| o.status.success()).unwrap_or(false);

        // Get any error from status
        let mut cmd = Command::new("systemctl");
        cmd.arg("--user")
            .arg("status")
            .arg(service)
            .arg("--no-pager");
        if let Some(ref ua) = user_arg {
            cmd.arg(ua);
        }
        let error = cmd.output().ok().and_then(|o| {
            if !o.status.success() {
                let stderr = String::from_utf8_lossy(&o.stderr);
                if !stderr.is_empty() {
                    Some(stderr.to_string())
                } else {
                    None
                }
            } else {
                None
            }
        });

        (running, enabled, error)
    }

    /// Determine active portal backend
    fn determine_portal_backend(&self, portals: &[PortalState]) -> PortalBackend {
        // Check which portal-specific backend is running
        for portal in portals {
            if portal.running {
                if portal.service.contains("hyprland") {
                    return PortalBackend::Hyprland;
                } else if portal.service.contains("wlr") {
                    return PortalBackend::Wlr;
                } else if portal.service.contains("kde") {
                    return PortalBackend::Kde;
                } else if portal.service.contains("gnome") {
                    return PortalBackend::Gnome;
                } else if portal.service.contains("gtk") {
                    return PortalBackend::Gtk;
                } else if portal.service.contains("lxqt") {
                    return PortalBackend::Lxqt;
                }
            }
        }

        // Check if main portal is running but no backend
        let main_running = portals
            .iter()
            .any(|p| p.service == "xdg-desktop-portal.service" && p.running);

        if main_running {
            PortalBackend::Unknown
        } else {
            PortalBackend::None
        }
    }

    /// Check PipeWire and WirePlumber services
    fn check_audio_services(&self, target_user: Option<&str>) -> (bool, bool) {
        let (pw_running, _, _) = self.check_user_service("pipewire.service", target_user);
        let (wp_running, _, _) = self.check_user_service("wireplumber.service", target_user);
        (pw_running, wp_running)
    }

    /// Collect monitor information
    fn collect_monitors(&self, session: &SessionInfo) -> Vec<MonitorInfo> {
        let mut monitors = vec![];

        if session.session_type == SessionType::Wayland {
            // Try hyprctl for Hyprland
            if matches!(session.compositor, Compositor::Hyprland) {
                if let Ok(output) = Command::new("hyprctl").arg("monitors").arg("-j").output() {
                    if output.status.success() {
                        // Parse JSON output
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                            if let Some(arr) = json.as_array() {
                                for m in arr {
                                    let name = m
                                        .get("name")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("unknown")
                                        .to_string();
                                    let width =
                                        m.get("width").and_then(|v| v.as_u64()).unwrap_or(0);
                                    let height =
                                        m.get("height").and_then(|v| v.as_u64()).unwrap_or(0);
                                    let refresh = m.get("refreshRate").and_then(|v| v.as_f64());
                                    let focused =
                                        m.get("focused").and_then(|v| v.as_bool()).unwrap_or(false);

                                    monitors.push(MonitorInfo {
                                        name,
                                        resolution: format!("{}x{}", width, height),
                                        refresh_rate: refresh.map(|r| format!("{:.2}Hz", r)),
                                        is_primary: focused,
                                        enabled: true,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        } else if session.session_type == SessionType::X11 {
            // Use xrandr
            if let Ok(output) = Command::new("xrandr").arg("--query").output() {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    // Parse xrandr output
                    for line in stdout.lines() {
                        if line.contains(" connected") {
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if let Some(name) = parts.first() {
                                let is_primary = line.contains("primary");
                                let resolution = parts
                                    .iter()
                                    .find(|p| {
                                        p.contains('x')
                                            && p.chars()
                                                .all(|c| c.is_numeric() || c == 'x' || c == '+')
                                    })
                                    .map(|r| r.split('+').next().unwrap_or(*r))
                                    .unwrap_or("unknown")
                                    .to_string();

                                monitors.push(MonitorInfo {
                                    name: name.to_string(),
                                    resolution,
                                    refresh_rate: None,
                                    is_primary,
                                    enabled: !line.contains("disconnected"),
                                });
                            }
                        }
                    }
                }
            }
        }

        monitors
    }

    /// Collect compositor logs
    fn collect_compositor_logs(&self, session: &SessionInfo) -> Vec<LogEntry> {
        let mut logs = vec![];

        // Collect from journal for compositor
        let unit = match &session.compositor {
            Compositor::Hyprland => Some("hyprland"),
            Compositor::Sway => Some("sway"),
            Compositor::KwinWayland => Some("kwin_wayland"),
            Compositor::Mutter => Some("gnome-shell"),
            _ => None,
        };

        if let Some(u) = unit {
            let output = Command::new("journalctl")
                .args(["--user", "-u", u, "-n", "20", "--no-pager", "-p", "warning"])
                .output();

            if let Ok(out) = output {
                let stdout = String::from_utf8_lossy(&out.stdout);
                for line in stdout.lines().take(20) {
                    logs.push(LogEntry {
                        timestamp: Utc::now(),
                        priority: "warning".to_string(),
                        unit: u.to_string(),
                        message: line.to_string(),
                    });
                }
            }
        }

        logs
    }

    /// Collect portal logs
    fn collect_portal_logs(&self, _target_user: Option<&str>) -> Vec<LogEntry> {
        let mut logs = vec![];

        let output = Command::new("journalctl")
            .args([
                "--user",
                "-u",
                "xdg-desktop-portal*",
                "-n",
                "20",
                "--no-pager",
                "-p",
                "warning",
            ])
            .output();

        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines().take(20) {
                logs.push(LogEntry {
                    timestamp: Utc::now(),
                    priority: "warning".to_string(),
                    unit: "xdg-desktop-portal".to_string(),
                    message: line.to_string(),
                });
            }
        }

        logs
    }

    /// Collect relevant environment variables
    fn collect_env_vars(&self) -> Vec<(String, String)> {
        let vars = [
            "XDG_SESSION_TYPE",
            "XDG_CURRENT_DESKTOP",
            "WAYLAND_DISPLAY",
            "DISPLAY",
            "GDK_BACKEND",
            "QT_QPA_PLATFORM",
            "MOZ_ENABLE_WAYLAND",
            "LIBVA_DRIVER_NAME",
            "VDPAU_DRIVER",
            "__GLX_VENDOR_LIBRARY_NAME",
            "WLR_NO_HARDWARE_CURSORS",
            "WLR_RENDERER",
            "HYPRLAND_INSTANCE_SIGNATURE",
        ];

        vars.iter()
            .filter_map(|&v| std::env::var(v).ok().map(|val| (v.to_string(), val)))
            .collect()
    }

    /// Run diagnosis
    pub fn diagnose(&self, evidence: &GraphicsEvidence) -> DiagnosisResult {
        let mut steps = vec![];
        let mut findings = vec![];

        // Step 1: Session and compositor
        steps.push(self.step_session_compositor(evidence, &mut findings));

        // Step 2: GPU and driver
        steps.push(self.step_gpu_driver(evidence, &mut findings));

        // Step 3: Packages
        steps.push(self.step_packages(evidence, &mut findings));

        // Step 4: Portal health
        steps.push(self.step_portal_health(evidence, &mut findings));

        // Step 5: Logs
        steps.push(self.step_check_logs(evidence, &mut findings));

        // Step 6: Generate hypotheses
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

    /// Step 1: Session and compositor
    fn step_session_compositor(
        &self,
        evidence: &GraphicsEvidence,
        findings: &mut Vec<Finding>,
    ) -> DiagnosisStep {
        let session = &evidence.session;

        let (result, details, implication) = match session.session_type {
            SessionType::Wayland => {
                findings.push(Finding {
                    id: "session-wayland".to_string(),
                    description: format!("Wayland session with {} compositor", session.compositor),
                    risk: RiskLevel::Info,
                    component: Some("session".to_string()),
                    evidence_ids: vec![format!("{}-session", evidence.evidence_id)],
                });

                (
                    StepResult::Pass,
                    format!(
                        "Wayland session detected. Compositor: {}",
                        session.compositor
                    ),
                    "Modern Wayland session running.".to_string(),
                )
            }
            SessionType::X11 => {
                findings.push(Finding {
                    id: "session-x11".to_string(),
                    description: format!("X11 session with {}", session.compositor),
                    risk: RiskLevel::Info,
                    component: Some("session".to_string()),
                    evidence_ids: vec![format!("{}-session", evidence.evidence_id)],
                });

                (
                    StepResult::Pass,
                    format!(
                        "X11 session detected. WM/Compositor: {}",
                        session.compositor
                    ),
                    "Legacy X11 session running.".to_string(),
                )
            }
            SessionType::Tty => (
                StepResult::Partial,
                "Running in TTY (no graphical session)".to_string(),
                "No graphical session active.".to_string(),
            ),
            SessionType::Unknown => (
                StepResult::Fail,
                "Cannot determine session type".to_string(),
                "Session type detection failed.".to_string(),
            ),
        };

        DiagnosisStep {
            name: "Session Detection".to_string(),
            description: "Identify session type and compositor".to_string(),
            step_number: 1,
            result,
            details,
            implication,
            evidence_ids: vec![format!("{}-session", evidence.evidence_id)],
        }
    }

    /// Step 2: GPU and driver
    fn step_gpu_driver(
        &self,
        evidence: &GraphicsEvidence,
        findings: &mut Vec<Finding>,
    ) -> DiagnosisStep {
        if evidence.gpus.is_empty() {
            return DiagnosisStep {
                name: "GPU Detection".to_string(),
                description: "Identify GPU and loaded driver".to_string(),
                step_number: 2,
                result: StepResult::Fail,
                details: "No GPU detected".to_string(),
                implication: "Graphics hardware not found or not accessible.".to_string(),
                evidence_ids: vec![],
            };
        }

        let mut all_loaded = true;
        for gpu in &evidence.gpus {
            findings.push(Finding {
                id: format!("gpu-{}", gpu.pci_slot.replace('.', "-").replace(':', "-")),
                description: format!(
                    "{} GPU: {} (module: {:?})",
                    gpu.vendor, gpu.device_name, gpu.kernel_module
                ),
                risk: RiskLevel::Info,
                component: Some("gpu".to_string()),
                evidence_ids: vec![format!("{}-gpu", evidence.evidence_id)],
            });

            if gpu.kernel_module.is_none() {
                all_loaded = false;
            }
        }

        let primary_gpu = evidence
            .gpus
            .iter()
            .find(|g| g.is_primary)
            .or(evidence.gpus.first());
        let (result, details, implication) = if let Some(gpu) = primary_gpu {
            if gpu.kernel_module.is_some() {
                (
                    StepResult::Pass,
                    format!(
                        "{} GPU with {} driver loaded",
                        gpu.vendor, evidence.driver_stack
                    ),
                    "GPU driver loaded successfully.".to_string(),
                )
            } else {
                (
                    StepResult::Fail,
                    format!("{} GPU detected but no driver loaded", gpu.vendor),
                    "GPU driver not loaded. Graphics may not work.".to_string(),
                )
            }
        } else {
            (
                StepResult::Fail,
                "No primary GPU identified".to_string(),
                "GPU detection incomplete.".to_string(),
            )
        };

        DiagnosisStep {
            name: "GPU Detection".to_string(),
            description: "Identify GPU and loaded driver".to_string(),
            step_number: 2,
            result: if all_loaded {
                result
            } else {
                StepResult::Partial
            },
            details,
            implication,
            evidence_ids: vec![format!("{}-gpu", evidence.evidence_id)],
        }
    }

    /// Step 3: Packages
    fn step_packages(
        &self,
        evidence: &GraphicsEvidence,
        findings: &mut Vec<Finding>,
    ) -> DiagnosisStep {
        let packages = &evidence.packages;

        // Check for required packages based on GPU
        let mut missing = vec![];

        // Mesa is required for AMD/Intel
        let needs_mesa = evidence
            .gpus
            .iter()
            .any(|g| g.vendor == GpuVendor::Amd || g.vendor == GpuVendor::Intel);
        if needs_mesa && packages.mesa.is_empty() {
            missing.push("mesa");
        }

        // NVIDIA needs nvidia packages
        let needs_nvidia = evidence.gpus.iter().any(|g| g.vendor == GpuVendor::Nvidia);
        if needs_nvidia
            && packages.nvidia.is_empty()
            && evidence.driver_stack != DriverStack::Nouveau
        {
            missing.push("nvidia or nouveau");
        }

        // Vulkan loader is generally recommended
        if packages.vulkan.is_empty() {
            findings.push(Finding {
                id: "missing-vulkan".to_string(),
                description: "Vulkan packages not detected".to_string(),
                risk: RiskLevel::Low,
                component: Some("packages".to_string()),
                evidence_ids: vec![format!("{}-packages", evidence.evidence_id)],
            });
        }

        let (result, details, implication) = if missing.is_empty() {
            (
                StepResult::Pass,
                format!(
                    "Required packages present: mesa={}, nvidia={}, vulkan={}",
                    packages.mesa.len(),
                    packages.nvidia.len(),
                    packages.vulkan.len()
                ),
                "Graphics packages properly installed.".to_string(),
            )
        } else {
            findings.push(Finding {
                id: "missing-packages".to_string(),
                description: format!("Missing packages: {}", missing.join(", ")),
                risk: RiskLevel::Medium,
                component: Some("packages".to_string()),
                evidence_ids: vec![format!("{}-packages", evidence.evidence_id)],
            });

            (
                StepResult::Partial,
                format!("Missing packages: {}", missing.join(", ")),
                "Some graphics packages may need to be installed.".to_string(),
            )
        };

        DiagnosisStep {
            name: "Package Check".to_string(),
            description: "Verify required graphics packages".to_string(),
            step_number: 3,
            result,
            details,
            implication,
            evidence_ids: vec![format!("{}-packages", evidence.evidence_id)],
        }
    }

    /// Step 4: Portal health
    fn step_portal_health(
        &self,
        evidence: &GraphicsEvidence,
        findings: &mut Vec<Finding>,
    ) -> DiagnosisStep {
        // Only relevant for Wayland
        if evidence.session.session_type != SessionType::Wayland {
            return DiagnosisStep {
                name: "Portal Health".to_string(),
                description: "Check XDG portal stack".to_string(),
                step_number: 4,
                result: StepResult::Skipped,
                details: "Not a Wayland session, portals less critical".to_string(),
                implication: "Portal check skipped for X11.".to_string(),
                evidence_ids: vec![],
            };
        }

        let main_portal = evidence
            .portals
            .iter()
            .find(|p| p.service == "xdg-desktop-portal.service");

        let backend_portal = evidence
            .portals
            .iter()
            .find(|p| p.running && p.service != "xdg-desktop-portal.service");

        let (result, details, implication) = match (main_portal, backend_portal) {
            (Some(main), Some(backend)) if main.running && backend.running => {
                // Check if correct backend for compositor
                if !evidence.is_portal_backend_correct() {
                    findings.push(Finding {
                        id: "wrong-portal-backend".to_string(),
                        description: format!(
                            "Portal backend {} may not be optimal for {}",
                            evidence.portal_backend, evidence.session.compositor
                        ),
                        risk: RiskLevel::Medium,
                        component: Some("portal".to_string()),
                        evidence_ids: vec![format!("{}-portal", evidence.evidence_id)],
                    });

                    (
                        StepResult::Partial,
                        format!(
                            "Portals running but {} may not be optimal for {}",
                            evidence.portal_backend, evidence.session.compositor
                        ),
                        format!(
                            "Consider installing xdg-desktop-portal-{} for best compatibility.",
                            evidence
                                .expected_portal_backend()
                                .to_string()
                                .to_lowercase()
                        ),
                    )
                } else {
                    (
                        StepResult::Pass,
                        format!(
                            "Portal stack healthy: {} backend for {}",
                            evidence.portal_backend, evidence.session.compositor
                        ),
                        "Screen sharing and file dialogs should work.".to_string(),
                    )
                }
            }
            (Some(main), _) if main.running => {
                findings.push(Finding {
                    id: "portal-no-backend".to_string(),
                    description: "Main portal running but no backend detected".to_string(),
                    risk: RiskLevel::Medium,
                    component: Some("portal".to_string()),
                    evidence_ids: vec![format!("{}-portal", evidence.evidence_id)],
                });

                (
                    StepResult::Partial,
                    "Main portal running but no compositor-specific backend".to_string(),
                    "Screen sharing may not work without proper backend.".to_string(),
                )
            }
            (Some(main), _) if !main.running => {
                findings.push(Finding {
                    id: "portal-not-running".to_string(),
                    description: "XDG Desktop Portal is not running".to_string(),
                    risk: RiskLevel::High,
                    component: Some("portal".to_string()),
                    evidence_ids: vec![format!("{}-portal", evidence.evidence_id)],
                });

                (
                    StepResult::Fail,
                    "XDG Desktop Portal not running".to_string(),
                    "Screen sharing and native file dialogs will not work.".to_string(),
                )
            }
            _ => {
                findings.push(Finding {
                    id: "portal-not-installed".to_string(),
                    description: "XDG Desktop Portal not detected".to_string(),
                    risk: RiskLevel::High,
                    component: Some("portal".to_string()),
                    evidence_ids: vec![format!("{}-portal", evidence.evidence_id)],
                });

                (
                    StepResult::Fail,
                    "Portal stack not properly configured".to_string(),
                    "Wayland features like screen sharing require portals.".to_string(),
                )
            }
        };

        // Also check PipeWire for screen sharing
        if !evidence.pipewire_running {
            findings.push(Finding {
                id: "pipewire-not-running".to_string(),
                description: "PipeWire not running (needed for screen sharing)".to_string(),
                risk: RiskLevel::Medium,
                component: Some("pipewire".to_string()),
                evidence_ids: vec![format!("{}-pipewire", evidence.evidence_id)],
            });
        }

        DiagnosisStep {
            name: "Portal Health".to_string(),
            description: "Check XDG portal stack".to_string(),
            step_number: 4,
            result,
            details,
            implication,
            evidence_ids: vec![format!("{}-portal", evidence.evidence_id)],
        }
    }

    /// Step 5: Check logs
    fn step_check_logs(
        &self,
        evidence: &GraphicsEvidence,
        findings: &mut Vec<Finding>,
    ) -> DiagnosisStep {
        let crash_keywords = [
            "crash", "segfault", "SIGSEGV", "fatal", "GPU hang", "error", "failed",
        ];

        let compositor_issues: Vec<_> = evidence
            .compositor_logs
            .iter()
            .filter(|l| {
                crash_keywords
                    .iter()
                    .any(|k| l.message.to_lowercase().contains(&k.to_lowercase()))
            })
            .collect();

        let portal_issues: Vec<_> = evidence
            .portal_logs
            .iter()
            .filter(|l| {
                crash_keywords
                    .iter()
                    .any(|k| l.message.to_lowercase().contains(&k.to_lowercase()))
            })
            .collect();

        let (result, details, implication) = if compositor_issues.is_empty()
            && portal_issues.is_empty()
        {
            (
                StepResult::Pass,
                "No recent errors in compositor or portal logs".to_string(),
                "Graphics stack appears stable.".to_string(),
            )
        } else {
            if !compositor_issues.is_empty() {
                findings.push(Finding {
                    id: "compositor-errors".to_string(),
                    description: format!("{} error(s) in compositor logs", compositor_issues.len()),
                    risk: RiskLevel::Medium,
                    component: Some("compositor".to_string()),
                    evidence_ids: vec![format!("{}-logs", evidence.evidence_id)],
                });
            }

            if !portal_issues.is_empty() {
                findings.push(Finding {
                    id: "portal-errors".to_string(),
                    description: format!("{} error(s) in portal logs", portal_issues.len()),
                    risk: RiskLevel::Medium,
                    component: Some("portal".to_string()),
                    evidence_ids: vec![format!("{}-logs", evidence.evidence_id)],
                });
            }

            (
                StepResult::Partial,
                format!(
                    "Found {} compositor and {} portal issues in logs",
                    compositor_issues.len(),
                    portal_issues.len()
                ),
                "Recent errors detected. May indicate instability.".to_string(),
            )
        };

        DiagnosisStep {
            name: "Log Analysis".to_string(),
            description: "Check for errors in compositor and portal logs".to_string(),
            step_number: 5,
            result,
            details,
            implication,
            evidence_ids: vec![format!("{}-logs", evidence.evidence_id)],
        }
    }

    /// Generate hypotheses
    fn generate_hypotheses(
        &self,
        findings: &[Finding],
        evidence: &GraphicsEvidence,
    ) -> Vec<GraphicsHypothesis> {
        let mut hypotheses = vec![];

        // Hypothesis: Portal not running
        if findings.iter().any(|f| f.id == "portal-not-running") {
            hypotheses.push(GraphicsHypothesis {
                id: "portal-restart".to_string(),
                summary: "XDG Desktop Portal services are not running".to_string(),
                explanation: "The XDG Desktop Portal is required for screen sharing, file dialogs, \
                    and other Wayland features. Restarting the portal services may resolve the issue.".to_string(),
                confidence: 90,
                supporting_evidence: vec![format!("{}-portal", evidence.evidence_id)],
                suggested_playbook: Some("restart_portals".to_string()),
            });
        }

        // Hypothesis: Wrong portal backend
        if findings.iter().any(|f| f.id == "wrong-portal-backend") {
            let expected = evidence.expected_portal_backend();
            hypotheses.push(GraphicsHypothesis {
                id: "wrong-backend".to_string(),
                summary: format!(
                    "Portal backend {} may not work well with {}",
                    evidence.portal_backend, evidence.session.compositor
                ),
                explanation: format!(
                    "Your compositor ({}) works best with the {} portal backend. \
                     Consider installing xdg-desktop-portal-{} for proper screen sharing.",
                    evidence.session.compositor,
                    expected,
                    expected.to_string().to_lowercase()
                ),
                confidence: 80,
                supporting_evidence: vec![format!("{}-portal", evidence.evidence_id)],
                suggested_playbook: None, // Don't auto-install packages
            });
        }

        // Hypothesis: PipeWire not running (screen sharing)
        if findings.iter().any(|f| f.id == "pipewire-not-running") {
            hypotheses.push(GraphicsHypothesis {
                id: "pipewire-screen-share".to_string(),
                summary: "PipeWire not running - screen sharing will fail".to_string(),
                explanation: "PipeWire is required for screen sharing on Wayland. \
                    Restarting PipeWire and portal services may help."
                    .to_string(),
                confidence: 85,
                supporting_evidence: vec![format!("{}-pipewire", evidence.evidence_id)],
                suggested_playbook: Some("restart_pipewire_portals".to_string()),
            });
        }

        // Hypothesis: NVIDIA on Wayland issues
        if evidence.gpus.iter().any(|g| g.vendor == GpuVendor::Nvidia)
            && evidence.session.session_type == SessionType::Wayland
        {
            // Check for common NVIDIA+Wayland issues
            let has_drm_modeset = evidence
                .env_vars
                .iter()
                .any(|(k, _)| k.contains("NVIDIA") || k.contains("__GLX"));

            if !has_drm_modeset {
                hypotheses.push(GraphicsHypothesis {
                    id: "nvidia-wayland-config".to_string(),
                    summary: "NVIDIA Wayland may need additional configuration".to_string(),
                    explanation: "NVIDIA on Wayland often requires specific kernel parameters \
                        (nvidia_drm.modeset=1) and environment variables. Check the Arch Wiki \
                        for NVIDIA Wayland setup."
                        .to_string(),
                    confidence: 70,
                    supporting_evidence: vec![format!("{}-gpu", evidence.evidence_id)],
                    suggested_playbook: None, // Read-only guidance
                });
            }
        }

        // Hypothesis: Compositor crash
        if findings.iter().any(|f| f.id == "compositor-errors") {
            hypotheses.push(GraphicsHypothesis {
                id: "compositor-unstable".to_string(),
                summary: "Compositor may be experiencing crashes".to_string(),
                explanation: "Errors in compositor logs suggest instability. \
                    This could be due to driver issues, configuration problems, or bugs."
                    .to_string(),
                confidence: 75,
                supporting_evidence: vec![format!("{}-logs", evidence.evidence_id)],
                suggested_playbook: Some("collect_crash_report".to_string()),
            });
        }

        // Sort by confidence and limit
        hypotheses.sort_by(|a, b| b.confidence.cmp(&a.confidence));
        hypotheses.truncate(self.max_hypotheses);
        hypotheses
    }

    /// Step 6: Hypotheses summary
    fn step_hypotheses(&self, hypotheses: &[GraphicsHypothesis]) -> DiagnosisStep {
        let (result, details, implication) = if hypotheses.is_empty() {
            (
                StepResult::Pass,
                "No specific issues identified".to_string(),
                "Graphics stack appears healthy.".to_string(),
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
                "Potential issues identified.".to_string(),
            )
        };

        DiagnosisStep {
            name: "Hypothesis Generation".to_string(),
            description: "Generate theories about issues".to_string(),
            step_number: 6,
            result,
            details,
            implication,
            evidence_ids: vec![],
        }
    }

    /// Determine overall health
    fn determine_health(
        &self,
        steps: &[DiagnosisStep],
        evidence: &GraphicsEvidence,
    ) -> GraphicsHealth {
        let evidence_health = evidence.health();
        let fails = steps
            .iter()
            .filter(|s| s.result == StepResult::Fail)
            .count();
        let partials = steps
            .iter()
            .filter(|s| s.result == StepResult::Partial)
            .count();

        if evidence_health == GraphicsHealth::Broken || fails >= 2 {
            GraphicsHealth::Broken
        } else if evidence_health == GraphicsHealth::Degraded || fails == 1 || partials >= 2 {
            GraphicsHealth::Degraded
        } else if evidence.session.session_type == SessionType::Unknown {
            GraphicsHealth::Unknown
        } else {
            GraphicsHealth::Healthy
        }
    }

    /// Generate summary
    fn generate_summary(
        &self,
        health: &GraphicsHealth,
        hypotheses: &[GraphicsHypothesis],
        evidence: &GraphicsEvidence,
    ) -> String {
        let gpu_str = evidence
            .gpus
            .first()
            .map(|g| format!("{} ({})", g.vendor, evidence.driver_stack))
            .unwrap_or_else(|| "Unknown GPU".to_string());

        match health {
            GraphicsHealth::Healthy => format!(
                "Graphics healthy. {} session with {} compositor on {}.",
                evidence.session.session_type, evidence.session.compositor, gpu_str
            ),
            GraphicsHealth::Degraded => {
                if let Some(h) = hypotheses.first() {
                    format!(
                        "Graphics degraded. Likely cause: {} ({}% confidence)",
                        h.summary, h.confidence
                    )
                } else {
                    "Graphics degraded. Some issues detected.".to_string()
                }
            }
            GraphicsHealth::Broken => {
                if let Some(h) = hypotheses.first() {
                    format!(
                        "Graphics broken. Top issue: {} ({}% confidence)",
                        h.summary, h.confidence
                    )
                } else {
                    "Graphics broken. Critical issues detected.".to_string()
                }
            }
            GraphicsHealth::Unknown => "Cannot determine graphics health.".to_string(),
        }
    }

    /// Recommend playbooks
    fn recommend_playbooks(
        &self,
        hypotheses: &[GraphicsHypothesis],
        _evidence: &GraphicsEvidence,
    ) -> Vec<String> {
        hypotheses
            .iter()
            .filter_map(|h| h.suggested_playbook.clone())
            .collect()
    }

    /// Create a playbook
    pub fn create_playbook(
        &self,
        playbook_id: &str,
        evidence: &GraphicsEvidence,
        hypothesis_id: Option<&str>,
    ) -> Option<FixPlaybook> {
        let user = evidence.target_user.clone();

        match playbook_id {
            "restart_portals" => Some(FixPlaybook {
                id: "restart_portals".to_string(),
                playbook_type: PlaybookType::RestartPortals,
                name: "Restart Portal Services".to_string(),
                description: "Restart XDG Desktop Portal services to restore screen sharing and file dialogs.".to_string(),
                risk: RiskLevel::Low,
                target_user: user,
                preflight: vec![],
                commands: vec![
                    PlaybookCommand {
                        command: "systemctl --user restart xdg-desktop-portal".to_string(),
                        description: "Restart main portal service".to_string(),
                        as_user: true,
                        timeout_secs: 10,
                    },
                    PlaybookCommand {
                        command: format!("systemctl --user restart xdg-desktop-portal-{}",
                            evidence.expected_portal_backend().to_string().to_lowercase()),
                        description: "Restart backend portal".to_string(),
                        as_user: true,
                        timeout_secs: 10,
                    },
                ],
                post_checks: vec![
                    PostCheck {
                        name: "Portal running".to_string(),
                        command: "systemctl --user is-active xdg-desktop-portal".to_string(),
                        expected_pattern: Some("active".to_string()),
                        wait_secs: 2,
                    },
                ],
                rollback: vec![],
                confirmation_phrase: FIX_CONFIRMATION.to_string(),
                addresses_hypothesis: hypothesis_id.map(String::from),
                policy_blocked: false,
                policy_block_reason: None,
            }),

            "restart_pipewire_portals" => Some(FixPlaybook {
                id: "restart_pipewire_portals".to_string(),
                playbook_type: PlaybookType::RestartPipewire,
                name: "Restart PipeWire and Portals".to_string(),
                description: "Restart PipeWire and portal services for screen sharing.".to_string(),
                risk: RiskLevel::Low,
                target_user: user,
                preflight: vec![],
                commands: vec![
                    PlaybookCommand {
                        command: "systemctl --user restart pipewire pipewire-pulse wireplumber".to_string(),
                        description: "Restart PipeWire stack".to_string(),
                        as_user: true,
                        timeout_secs: 10,
                    },
                    PlaybookCommand {
                        command: "systemctl --user restart xdg-desktop-portal".to_string(),
                        description: "Restart portal service".to_string(),
                        as_user: true,
                        timeout_secs: 10,
                    },
                ],
                post_checks: vec![
                    PostCheck {
                        name: "PipeWire running".to_string(),
                        command: "systemctl --user is-active pipewire".to_string(),
                        expected_pattern: Some("active".to_string()),
                        wait_secs: 2,
                    },
                    PostCheck {
                        name: "Portal running".to_string(),
                        command: "systemctl --user is-active xdg-desktop-portal".to_string(),
                        expected_pattern: Some("active".to_string()),
                        wait_secs: 2,
                    },
                ],
                rollback: vec![],
                confirmation_phrase: FIX_CONFIRMATION.to_string(),
                addresses_hypothesis: hypothesis_id.map(String::from),
                policy_blocked: false,
                policy_block_reason: None,
            }),

            "collect_crash_report" => Some(FixPlaybook {
                id: "collect_crash_report".to_string(),
                playbook_type: PlaybookType::CollectCrashReport,
                name: "Collect Crash Information".to_string(),
                description: "Gather compositor crash logs for analysis (read-only).".to_string(),
                risk: RiskLevel::Info,
                target_user: user,
                preflight: vec![],
                commands: vec![
                    PlaybookCommand {
                        command: format!("journalctl --user -u {} -n 100 --no-pager",
                            match &evidence.session.compositor {
                                Compositor::Hyprland => "hyprland",
                                Compositor::Sway => "sway",
                                _ => "*",
                            }),
                        description: "Collect compositor logs".to_string(),
                        as_user: true,
                        timeout_secs: 10,
                    },
                ],
                post_checks: vec![],
                rollback: vec![],
                confirmation_phrase: FIX_CONFIRMATION.to_string(),
                addresses_hypothesis: hypothesis_id.map(String::from),
                policy_blocked: false,
                policy_block_reason: None,
            }),

            "restart_display_manager" => Some(FixPlaybook {
                id: "restart_display_manager".to_string(),
                playbook_type: PlaybookType::RestartDisplayManager,
                name: "Restart Display Manager".to_string(),
                description: "Restart the display manager (WARNING: will log you out!)".to_string(),
                risk: RiskLevel::High,
                target_user: None,
                preflight: vec![],
                commands: vec![
                    PlaybookCommand {
                        command: "systemctl restart display-manager".to_string(),
                        description: "Restart display manager".to_string(),
                        as_user: false,
                        timeout_secs: 30,
                    },
                ],
                post_checks: vec![],
                rollback: vec![],
                confirmation_phrase: FIX_CONFIRMATION.to_string(),
                addresses_hypothesis: hypothesis_id.map(String::from),
                policy_blocked: true,
                policy_block_reason: Some("High risk: will terminate your session".to_string()),
            }),

            _ => None,
        }
    }

    /// Execute a playbook
    pub fn execute_playbook(&self, playbook: &FixPlaybook) -> PlaybookResult {
        if playbook.policy_blocked {
            return PlaybookResult {
                playbook_id: playbook.id.clone(),
                success: false,
                commands_executed: vec![],
                post_checks_passed: false,
                error: playbook.policy_block_reason.clone(),
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
                        error: Some(format!("Preflight failed: {}", e)),
                        reliability: 0,
                    };
                }
            }
        }

        // Execute commands
        for cmd in &playbook.commands {
            let start = std::time::Instant::now();
            let output = Command::new("sh").arg("-c").arg(&cmd.command).output();

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

        let reliability = if post_checks_passed { 85 } else { 60 };

        PlaybookResult {
            playbook_id: playbook.id.clone(),
            success: post_checks_passed,
            commands_executed,
            post_checks_passed,
            error: None,
            reliability,
        }
    }

    /// Maybe capture recipe
    pub fn maybe_capture_recipe(
        &self,
        playbook: &FixPlaybook,
        result: &PlaybookResult,
        evidence: &GraphicsEvidence,
    ) -> Option<RecipeCaptureRequest> {
        if !result.success || result.reliability < 80 {
            return None;
        }

        let problem = if let Some(hyp_id) = &playbook.addresses_hypothesis {
            format!("Graphics issue: {} (hypothesis: {})", playbook.name, hyp_id)
        } else {
            format!("Graphics issue: {}", playbook.name)
        };

        Some(RecipeCaptureRequest {
            name: format!(
                "Fix: {} on {}",
                playbook.name.to_lowercase().replace(' ', "_"),
                evidence.session.compositor
            ),
            problem,
            solution: playbook.description.clone(),
            preconditions: vec![
                format!("Session type: {}", evidence.session.session_type),
                format!("Compositor: {}", evidence.session.compositor),
            ],
            playbook_id: playbook.id.clone(),
            reliability: result.reliability,
            evidence_patterns: vec![evidence.evidence_id.clone()],
        })
    }
}

impl Default for GraphicsDoctor {
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

    fn create_healthy_wayland_evidence() -> GraphicsEvidence {
        GraphicsEvidence {
            collected_at: Utc::now(),
            session: SessionInfo {
                session_type: SessionType::Wayland,
                compositor: Compositor::Hyprland,
                xdg_session_type: Some("wayland".to_string()),
                wayland_display: Some("wayland-1".to_string()),
                display: None,
                xdg_current_desktop: Some("Hyprland".to_string()),
                hyprland_instance: Some("abc123".to_string()),
            },
            gpus: vec![GpuInfo {
                vendor: GpuVendor::Amd,
                device_name: "AMD Radeon RX 6800".to_string(),
                pci_slot: "03:00.0".to_string(),
                kernel_module: Some("amdgpu".to_string()),
                is_primary: true,
            }],
            driver_stack: DriverStack::Amdgpu,
            packages: DriverPackages {
                nvidia: vec![],
                mesa: vec!["mesa".to_string()],
                vulkan: vec!["vulkan-radeon".to_string()],
                libva: vec!["libva-mesa-driver".to_string()],
                vdpau: vec!["mesa-vdpau".to_string()],
            },
            portals: vec![
                PortalState {
                    service: "xdg-desktop-portal.service".to_string(),
                    installed: true,
                    running: true,
                    enabled: true,
                    error: None,
                },
                PortalState {
                    service: "xdg-desktop-portal-hyprland.service".to_string(),
                    installed: true,
                    running: true,
                    enabled: true,
                    error: None,
                },
            ],
            portal_backend: PortalBackend::Hyprland,
            pipewire_running: true,
            wireplumber_running: true,
            monitors: vec![MonitorInfo {
                name: "DP-1".to_string(),
                resolution: "2560x1440".to_string(),
                refresh_rate: Some("165Hz".to_string()),
                is_primary: true,
                enabled: true,
            }],
            compositor_logs: vec![],
            portal_logs: vec![],
            env_vars: vec![
                ("XDG_SESSION_TYPE".to_string(), "wayland".to_string()),
                ("WAYLAND_DISPLAY".to_string(), "wayland-1".to_string()),
            ],
            evidence_id: "ev-test-123".to_string(),
            target_user: Some("testuser".to_string()),
        }
    }

    fn create_portal_broken_evidence() -> GraphicsEvidence {
        let mut evidence = create_healthy_wayland_evidence();
        evidence.portals = vec![
            PortalState {
                service: "xdg-desktop-portal.service".to_string(),
                installed: true,
                running: false,
                enabled: true,
                error: Some("Service failed".to_string()),
            },
            PortalState {
                service: "xdg-desktop-portal-hyprland.service".to_string(),
                installed: true,
                running: false,
                enabled: true,
                error: None,
            },
        ];
        evidence.portal_backend = PortalBackend::None;
        evidence
    }

    #[test]
    fn test_graphics_health_display() {
        assert_eq!(GraphicsHealth::Healthy.to_string(), "Healthy");
        assert_eq!(GraphicsHealth::Degraded.to_string(), "Degraded");
        assert_eq!(GraphicsHealth::Broken.to_string(), "Broken");
    }

    #[test]
    fn test_session_type_display() {
        assert_eq!(SessionType::Wayland.to_string(), "Wayland");
        assert_eq!(SessionType::X11.to_string(), "X11");
    }

    #[test]
    fn test_gpu_vendor_display() {
        assert_eq!(GpuVendor::Nvidia.to_string(), "NVIDIA");
        assert_eq!(GpuVendor::Amd.to_string(), "AMD");
        assert_eq!(GpuVendor::Intel.to_string(), "Intel");
    }

    #[test]
    fn test_driver_stack_display() {
        assert_eq!(
            DriverStack::NvidiaProprietary.to_string(),
            "NVIDIA Proprietary"
        );
        assert_eq!(DriverStack::Amdgpu.to_string(), "AMDGPU");
        assert_eq!(DriverStack::I915.to_string(), "Intel i915");
    }

    #[test]
    fn test_evidence_health_healthy() {
        let evidence = create_healthy_wayland_evidence();
        assert_eq!(evidence.health(), GraphicsHealth::Healthy);
    }

    #[test]
    fn test_evidence_health_portal_broken() {
        let evidence = create_portal_broken_evidence();
        assert_eq!(evidence.health(), GraphicsHealth::Degraded);
    }

    #[test]
    fn test_portal_backend_correct() {
        let evidence = create_healthy_wayland_evidence();
        assert!(evidence.is_portal_backend_correct());
    }

    #[test]
    fn test_portal_backend_wrong() {
        let mut evidence = create_healthy_wayland_evidence();
        evidence.portal_backend = PortalBackend::Kde; // Wrong for Hyprland
        assert!(!evidence.is_portal_backend_correct());
    }

    #[test]
    fn test_expected_portal_backend() {
        let evidence = create_healthy_wayland_evidence();
        assert_eq!(evidence.expected_portal_backend(), PortalBackend::Hyprland);
    }

    #[test]
    fn test_diagnosis_healthy() {
        let doctor = GraphicsDoctor::new();
        let evidence = create_healthy_wayland_evidence();
        let result = doctor.diagnose(&evidence);

        assert_eq!(result.health, GraphicsHealth::Healthy);
        assert!(result.hypotheses.is_empty());
    }

    #[test]
    fn test_diagnosis_portal_broken() {
        let doctor = GraphicsDoctor::new();
        let evidence = create_portal_broken_evidence();
        let result = doctor.diagnose(&evidence);

        assert_eq!(result.health, GraphicsHealth::Degraded);
        assert!(!result.hypotheses.is_empty());
        assert!(result.hypotheses.iter().any(|h| h.id == "portal-restart"));
    }

    #[test]
    fn test_playbook_restart_portals() {
        let doctor = GraphicsDoctor::new();
        let evidence = create_portal_broken_evidence();

        let playbook = doctor.create_playbook("restart_portals", &evidence, Some("portal-restart"));
        assert!(playbook.is_some());

        let pb = playbook.unwrap();
        assert_eq!(pb.id, "restart_portals");
        assert_eq!(pb.risk, RiskLevel::Low);
        assert!(!pb.policy_blocked);
    }

    #[test]
    fn test_playbook_restart_dm_blocked() {
        let doctor = GraphicsDoctor::new();
        let evidence = create_healthy_wayland_evidence();

        let playbook = doctor.create_playbook("restart_display_manager", &evidence, None);
        assert!(playbook.is_some());

        let pb = playbook.unwrap();
        assert!(pb.policy_blocked);
        assert_eq!(pb.risk, RiskLevel::High);
    }

    #[test]
    fn test_max_three_hypotheses() {
        let doctor = GraphicsDoctor::new();
        let mut evidence = create_portal_broken_evidence();
        // Add more issues
        evidence.pipewire_running = false;
        evidence.compositor_logs.push(LogEntry {
            timestamp: Utc::now(),
            priority: "error".to_string(),
            unit: "hyprland".to_string(),
            message: "GPU hang detected".to_string(),
        });

        let result = doctor.diagnose(&evidence);
        assert!(result.hypotheses.len() <= 3);
    }

    #[test]
    fn test_case_file_workflow() {
        let evidence = create_healthy_wayland_evidence();
        let mut case_file = GraphicsDoctorCase::new(evidence);

        assert_eq!(case_file.status, CaseStatus::EvidenceCollected);

        let doctor = GraphicsDoctor::new();
        let diagnosis = doctor.diagnose(&case_file.evidence);
        case_file.add_diagnosis(diagnosis);

        assert_eq!(case_file.status, CaseStatus::Diagnosed);

        case_file.add_note("Investigating portal issue");
        assert_eq!(case_file.notes.len(), 1);
    }

    #[test]
    fn test_recipe_capture_on_success() {
        let doctor = GraphicsDoctor::new();
        let evidence = create_portal_broken_evidence();

        let playbook = doctor
            .create_playbook("restart_portals", &evidence, Some("portal-restart"))
            .unwrap();

        let result = PlaybookResult {
            playbook_id: playbook.id.clone(),
            success: true,
            commands_executed: vec![],
            post_checks_passed: true,
            error: None,
            reliability: 85,
        };

        let recipe = doctor.maybe_capture_recipe(&playbook, &result, &evidence);
        assert!(recipe.is_some());

        let r = recipe.unwrap();
        assert!(r.name.contains("restart_portal"));
        assert_eq!(r.reliability, 85);
    }

    #[test]
    fn test_wrong_backend_hypothesis() {
        let doctor = GraphicsDoctor::new();
        let mut evidence = create_healthy_wayland_evidence();
        evidence.portal_backend = PortalBackend::Gtk; // Wrong for Hyprland

        let result = doctor.diagnose(&evidence);
        assert!(result.hypotheses.iter().any(|h| h.id == "wrong-backend"));
    }

    #[test]
    fn test_pipewire_not_running_hypothesis() {
        let doctor = GraphicsDoctor::new();
        let mut evidence = create_healthy_wayland_evidence();
        evidence.pipewire_running = false;

        let result = doctor.diagnose(&evidence);
        assert!(result
            .hypotheses
            .iter()
            .any(|h| h.id == "pipewire-screen-share"));
    }

    #[test]
    fn test_nvidia_wayland_check() {
        let doctor = GraphicsDoctor::new();
        let mut evidence = create_healthy_wayland_evidence();
        evidence.gpus = vec![GpuInfo {
            vendor: GpuVendor::Nvidia,
            device_name: "NVIDIA GeForce RTX 4090".to_string(),
            pci_slot: "01:00.0".to_string(),
            kernel_module: Some("nvidia".to_string()),
            is_primary: true,
        }];
        evidence.driver_stack = DriverStack::NvidiaProprietary;
        evidence.env_vars.clear(); // No NVIDIA env vars

        let result = doctor.diagnose(&evidence);
        // Should suggest NVIDIA Wayland config
        assert!(result
            .hypotheses
            .iter()
            .any(|h| h.id == "nvidia-wayland-config"));
    }
}
