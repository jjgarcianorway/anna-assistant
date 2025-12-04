//! Anna Helper Tracking v0.0.9
//!
//! Tracks ALL helpers Anna relies on, regardless of who installed them.
//! Provenance tracking distinguishes anna-installed vs user-installed.
//!
//! Two dimensions tracked:
//! - present/missing: Is the helper currently available?
//! - installed_by: anna | user | unknown
//!
//! Rules:
//! - If helper exists before Anna ever installed it → installed_by=user (or unknown)
//! - If Anna installs helper → installed_by=anna
//! - Only remove helpers on uninstall if installed_by=anna
//! - Provenance is tracked per-machine, not globally

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Path to helpers state file
pub const HELPERS_STATE_FILE: &str = "/var/lib/anna/helpers.json";

/// Who installed a helper
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InstalledBy {
    /// Installed by Anna
    Anna,
    /// Installed by user (was present before Anna tracked it)
    User,
    /// Unknown origin (present but can't determine)
    Unknown,
}

impl std::fmt::Display for InstalledBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstalledBy::Anna => write!(f, "Anna"),
            InstalledBy::User => write!(f, "user"),
            InstalledBy::Unknown => write!(f, "unknown"),
        }
    }
}

/// A helper package Anna relies on
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelperDefinition {
    /// Package name (e.g., "lm_sensors")
    pub name: String,
    /// Short purpose description
    pub purpose: String,
    /// Whether this helper is required (false) or optional (true)
    pub optional: bool,
    /// What command/binary this provides (for presence check)
    pub provides_command: Option<String>,
    /// v0.0.28: System relevance check (what hardware must exist for this to be useful)
    /// If None, always relevant. If Some, check must pass.
    #[serde(default)]
    pub relevance_check: Option<RelevanceCheck>,
}

/// v0.0.28: System relevance check for helpers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RelevanceCheck {
    /// Only relevant if system has ethernet interfaces
    HasEthernet,
    /// Only relevant if system has WiFi interfaces
    HasWiFi,
    /// Only relevant if system has NVMe devices
    HasNvme,
    /// Only relevant if system has SATA devices
    HasSata,
    /// Always relevant
    Always,
}

/// Current state of a helper on this machine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelperState {
    /// Package name
    pub name: String,
    /// Short purpose description
    pub purpose: String,
    /// Whether the helper is currently present
    pub present: bool,
    /// Who installed this helper
    pub installed_by: InstalledBy,
    /// When Anna first detected this helper (RFC3339)
    pub first_seen: Option<DateTime<Utc>>,
    /// When Anna installed this helper (if installed_by=anna)
    pub anna_install_timestamp: Option<DateTime<Utc>>,
    /// Package version if present
    pub version: Option<String>,
    /// Whether removal is eligible (only if installed_by=anna)
    pub removal_eligible: bool,
}

impl HelperState {
    /// Create new state for a helper that was already present
    pub fn new_user_installed(name: &str, purpose: &str, version: Option<String>) -> Self {
        Self {
            name: name.to_string(),
            purpose: purpose.to_string(),
            present: true,
            installed_by: InstalledBy::User,
            first_seen: Some(Utc::now()),
            anna_install_timestamp: None,
            version,
            removal_eligible: false,
        }
    }

    /// Create new state for a missing helper
    pub fn new_missing(name: &str, purpose: &str) -> Self {
        Self {
            name: name.to_string(),
            purpose: purpose.to_string(),
            present: false,
            installed_by: InstalledBy::Unknown,
            first_seen: None,
            anna_install_timestamp: None,
            version: None,
            removal_eligible: false,
        }
    }

    /// Mark as installed by Anna
    pub fn mark_anna_installed(&mut self, version: Option<String>) {
        self.present = true;
        self.installed_by = InstalledBy::Anna;
        self.anna_install_timestamp = Some(Utc::now());
        self.version = version;
        self.removal_eligible = true;
        if self.first_seen.is_none() {
            self.first_seen = Some(Utc::now());
        }
    }

    /// Update presence status (called when checking system state)
    pub fn update_presence(&mut self, present: bool, version: Option<String>) {
        let was_present = self.present;
        self.present = present;
        self.version = version;

        // If newly appeared and we haven't tracked it before
        if present && !was_present && self.first_seen.is_none() {
            self.first_seen = Some(Utc::now());
            // If it appeared without Anna installing it, it's user-installed
            if self.installed_by == InstalledBy::Unknown && self.anna_install_timestamp.is_none() {
                self.installed_by = InstalledBy::User;
            }
        }
    }

    /// Format for status display
    pub fn format_status(&self) -> String {
        let presence = if self.present { "present" } else { "missing" };
        let by = match self.installed_by {
            InstalledBy::Anna => "installed by Anna",
            InstalledBy::User => "installed by user",
            InstalledBy::Unknown => "unknown origin",
        };
        format!("{} ({}, {})", self.name, presence, by)
    }
}

/// Persistent state for all helpers
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HelpersManifest {
    /// Schema version
    pub schema_version: u32,
    /// Helper states by name
    pub helpers: HashMap<String, HelperState>,
    /// Last update timestamp
    pub last_updated: Option<DateTime<Utc>>,
}

impl HelpersManifest {
    const CURRENT_SCHEMA: u32 = 1;

    /// Load manifest from disk
    pub fn load() -> Self {
        let path = PathBuf::from(HELPERS_STATE_FILE);
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(manifest) = serde_json::from_str(&content) {
                    return manifest;
                }
            }
        }
        Self {
            schema_version: Self::CURRENT_SCHEMA,
            ..Default::default()
        }
    }

    /// Save manifest to disk
    pub fn save(&self) -> std::io::Result<()> {
        let path = PathBuf::from(HELPERS_STATE_FILE);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        crate::atomic_write(HELPERS_STATE_FILE, &content)
    }

    /// Get or create helper state
    pub fn get_or_create(&mut self, def: &HelperDefinition) -> &mut HelperState {
        if !self.helpers.contains_key(&def.name) {
            let state = HelperState::new_missing(&def.name, &def.purpose);
            self.helpers.insert(def.name.clone(), state);
        }
        self.helpers.get_mut(&def.name).unwrap()
    }

    /// Update helper state
    pub fn update_helper(&mut self, state: HelperState) {
        self.helpers.insert(state.name.clone(), state);
        self.last_updated = Some(Utc::now());
    }

    /// Get helper state
    pub fn get(&self, name: &str) -> Option<&HelperState> {
        self.helpers.get(name)
    }

    /// Get all helpers eligible for removal (installed_by=anna)
    pub fn removal_eligible(&self) -> Vec<&HelperState> {
        self.helpers
            .values()
            .filter(|h| h.removal_eligible && h.installed_by == InstalledBy::Anna)
            .collect()
    }

    /// Get all present helpers
    pub fn present_helpers(&self) -> Vec<&HelperState> {
        self.helpers.values().filter(|h| h.present).collect()
    }

    /// Get all missing helpers
    pub fn missing_helpers(&self) -> Vec<&HelperState> {
        self.helpers.values().filter(|h| !h.present).collect()
    }

    /// Record that Anna installed a helper
    pub fn record_anna_install(&mut self, name: &str, purpose: &str, version: Option<String>) {
        if let Some(state) = self.helpers.get_mut(name) {
            state.mark_anna_installed(version);
        } else {
            let mut state = HelperState::new_missing(name, purpose);
            state.mark_anna_installed(version);
            self.helpers.insert(name.to_string(), state);
        }
        self.last_updated = Some(Utc::now());
    }
}

/// All helpers Anna relies on for telemetry/diagnostics/execution
/// v0.0.28: Now includes relevance_check for system-appropriate filtering
pub fn get_helper_definitions() -> Vec<HelperDefinition> {
    vec![
        // Thermal/sensor monitoring - always relevant
        HelperDefinition {
            name: "lm_sensors".to_string(),
            purpose: "Temperature and fan monitoring".to_string(),
            optional: false,
            provides_command: Some("sensors".to_string()),
            relevance_check: None,
        },
        // Disk health - only if SATA devices exist
        HelperDefinition {
            name: "smartmontools".to_string(),
            purpose: "SATA/SAS disk health (SMART)".to_string(),
            optional: false,
            provides_command: Some("smartctl".to_string()),
            relevance_check: Some(RelevanceCheck::HasSata),
        },
        // NVMe - only if NVMe devices exist
        HelperDefinition {
            name: "nvme-cli".to_string(),
            purpose: "NVMe SSD health monitoring".to_string(),
            optional: false,
            provides_command: Some("nvme".to_string()),
            relevance_check: Some(RelevanceCheck::HasNvme),
        },
        // Network diagnostics - always relevant (useful for USB adapters, wifi stats too)
        // v0.0.33: Made always relevant since USB/Thunderbolt ethernet adapters are common
        HelperDefinition {
            name: "ethtool".to_string(),
            purpose: "Network interface diagnostics".to_string(),
            optional: false,
            provides_command: Some("ethtool".to_string()),
            relevance_check: None, // Always install - useful even for wifi driver stats
        },
        // WiFi - only if WiFi interfaces exist
        HelperDefinition {
            name: "iw".to_string(),
            purpose: "WiFi signal and stats".to_string(),
            optional: true,
            provides_command: Some("iw".to_string()),
            relevance_check: Some(RelevanceCheck::HasWiFi),
        },
        // Hardware enumeration - always relevant
        HelperDefinition {
            name: "usbutils".to_string(),
            purpose: "USB device enumeration".to_string(),
            optional: false,
            provides_command: Some("lsusb".to_string()),
            relevance_check: None,
        },
        HelperDefinition {
            name: "pciutils".to_string(),
            purpose: "PCI device enumeration".to_string(),
            optional: false,
            provides_command: Some("lspci".to_string()),
            relevance_check: None,
        },
        // Disk utilities - only if SATA devices exist
        HelperDefinition {
            name: "hdparm".to_string(),
            purpose: "SATA disk parameters".to_string(),
            optional: true,
            provides_command: Some("hdparm".to_string()),
            relevance_check: Some(RelevanceCheck::HasSata),
        },
        // LLM runtime - always relevant
        HelperDefinition {
            name: "ollama".to_string(),
            purpose: "Local LLM inference".to_string(),
            optional: false,
            provides_command: Some("ollama".to_string()),
            relevance_check: None,
        },
    ]
}

/// v0.0.28: Check if a helper is relevant to this system
pub fn is_helper_relevant(check: &Option<RelevanceCheck>) -> bool {
    match check {
        None => true, // No check = always relevant
        Some(RelevanceCheck::Always) => true,
        Some(RelevanceCheck::HasEthernet) => has_ethernet_interfaces(),
        Some(RelevanceCheck::HasWiFi) => has_wifi_interfaces(),
        Some(RelevanceCheck::HasNvme) => has_nvme_devices(),
        Some(RelevanceCheck::HasSata) => has_sata_devices(),
    }
}

/// Check if system has ethernet interfaces
/// v0.0.30: More inclusive detection - any non-wireless, non-loopback, non-virtual interface
fn has_ethernet_interfaces() -> bool {
    if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            // Skip loopback
            if name_str == "lo" {
                continue;
            }

            // Skip virtual interfaces (docker, veth, br, virbr, etc.)
            if name_str.starts_with("veth")
                || name_str.starts_with("docker")
                || name_str.starts_with("br-")
                || name_str.starts_with("virbr")
            {
                continue;
            }

            // Check if it's wireless by looking for /sys/class/net/<name>/wireless
            let wireless_path = format!("/sys/class/net/{}/wireless", name_str);
            if std::path::Path::new(&wireless_path).exists() {
                continue; // It's a wireless interface
            }

            // v0.0.30: If it's not wireless, not loopback, not virtual - it's likely ethernet
            // Common patterns: eth*, en*, em*, enp*, eno*, ens*
            // But ANY non-wireless physical interface counts
            return true;
        }
    }
    false
}

/// Check if system has WiFi interfaces
fn has_wifi_interfaces() -> bool {
    if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            // Check if it's wireless by looking for /sys/class/net/<name>/wireless
            let wireless_path = format!("/sys/class/net/{}/wireless", name_str);
            if std::path::Path::new(&wireless_path).exists() {
                return true;
            }

            // Also check for wlan* or wlp* naming
            if name_str.starts_with("wlan") || name_str.starts_with("wlp") {
                return true;
            }
        }
    }
    false
}

/// Check if system has NVMe devices
fn has_nvme_devices() -> bool {
    std::path::Path::new("/sys/class/nvme").exists() && {
        if let Ok(entries) = std::fs::read_dir("/sys/class/nvme") {
            entries.count() > 0
        } else {
            false
        }
    }
}

/// Check if system has SATA devices
fn has_sata_devices() -> bool {
    // Check for SATA devices via /sys/class/ata_device or block devices
    if std::path::Path::new("/sys/class/ata_device").exists() {
        if let Ok(entries) = std::fs::read_dir("/sys/class/ata_device") {
            if entries.count() > 0 {
                return true;
            }
        }
    }

    // Also check for sd* block devices that aren't USB
    if let Ok(entries) = std::fs::read_dir("/sys/block") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            // sd* devices could be SATA or USB
            if name_str.starts_with("sd") {
                // Check if it's SATA by looking at the device path
                let device_path = format!("/sys/block/{}/device", name_str);
                if let Ok(link) = std::fs::read_link(&device_path) {
                    let link_str = link.to_string_lossy();
                    // SATA devices have "ata" in their path, USB have "usb"
                    if link_str.contains("ata") && !link_str.contains("usb") {
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// Get only relevant helper definitions for this system
/// v0.0.28: Filters out helpers that aren't useful for this hardware
pub fn get_relevant_helper_definitions() -> Vec<HelperDefinition> {
    get_helper_definitions()
        .into_iter()
        .filter(|def| is_helper_relevant(&def.relevance_check))
        .collect()
}

/// Check if a package is installed on the system
pub fn is_package_present(package: &str) -> bool {
    std::process::Command::new("pacman")
        .args(["-Qi", package])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Get the current package version
pub fn get_package_version(package: &str) -> Option<String> {
    let output = std::process::Command::new("pacman")
        .args(["-Qi", package])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.starts_with("Version") {
            return line.split(':').nth(1).map(|v| v.trim().to_string());
        }
    }
    None
}

/// Result of package installation
#[derive(Debug)]
pub struct InstallResult {
    pub success: bool,
    pub version: Option<String>,
    pub error: Option<String>,
}

/// Install a package via pacman (requires root)
/// Returns the installed version on success
pub fn install_package(package: &str) -> InstallResult {
    // Check if already installed
    if is_package_present(package) {
        return InstallResult {
            success: true,
            version: get_package_version(package),
            error: None,
        };
    }

    // Install via pacman with --noconfirm
    let output = std::process::Command::new("pacman")
        .args(["-S", "--noconfirm", package])
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                InstallResult {
                    success: true,
                    version: get_package_version(package),
                    error: None,
                }
            } else {
                let stderr = String::from_utf8_lossy(&result.stderr);
                InstallResult {
                    success: false,
                    version: None,
                    error: Some(stderr.to_string()),
                }
            }
        }
        Err(e) => InstallResult {
            success: false,
            version: None,
            error: Some(e.to_string()),
        },
    }
}

/// Install Ollama using the official install script
/// This is preferred over pacman as Ollama may not be in official repos
pub fn install_ollama() -> InstallResult {
    // Check if already installed
    if is_command_available("ollama") {
        return InstallResult {
            success: true,
            version: get_ollama_version(),
            error: None,
        };
    }

    // Install using official script: curl -fsSL https://ollama.ai/install.sh | sh
    let output = std::process::Command::new("sh")
        .args(["-c", "curl -fsSL https://ollama.ai/install.sh | sh"])
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                // Verify installation
                if is_command_available("ollama") {
                    InstallResult {
                        success: true,
                        version: get_ollama_version(),
                        error: None,
                    }
                } else {
                    InstallResult {
                        success: false,
                        version: None,
                        error: Some(
                            "Install script succeeded but ollama command not found".to_string(),
                        ),
                    }
                }
            } else {
                let stderr = String::from_utf8_lossy(&result.stderr);
                InstallResult {
                    success: false,
                    version: None,
                    error: Some(stderr.to_string()),
                }
            }
        }
        Err(e) => InstallResult {
            success: false,
            version: None,
            error: Some(e.to_string()),
        },
    }
}

/// Check if a command is available in PATH
pub fn is_command_available(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// v0.0.30: Install all missing relevant helpers for this system
/// Called on daemon startup to ensure Anna has all tools she needs
/// Returns (installed_count, failed_count, messages)
pub fn install_missing_helpers() -> (usize, usize, Vec<String>) {
    let mut installed = 0;
    let mut failed = 0;
    let mut messages = Vec::new();

    let definitions = get_relevant_helper_definitions();
    let mut manifest = HelpersManifest::load();

    for def in &definitions {
        // Check if helper is present
        let (present, _version) = check_helper_presence(def);

        if present {
            continue; // Already installed
        }

        // Install the helper
        messages.push(format!("Installing {}...", def.name));

        let result = if def.name == "ollama" {
            install_ollama()
        } else {
            install_package(&def.name)
        };

        if result.success {
            installed += 1;
            messages.push(format!(
                "  + {} installed ({})",
                def.name,
                result.version.as_deref().unwrap_or("unknown")
            ));

            // Update manifest - mark as anna-installed
            let state = manifest.get_or_create(&def);
            state.present = true;
            state.installed_by = InstalledBy::Anna;
            state.anna_install_timestamp = Some(Utc::now());
            state.version = result.version;
            state.first_seen = Some(Utc::now());
        } else {
            failed += 1;
            messages.push(format!(
                "  ! {} failed: {}",
                def.name,
                result.error.as_deref().unwrap_or("unknown error")
            ));
        }
    }

    // Save manifest
    let _ = manifest.save();

    (installed, failed, messages)
}

/// Get Ollama version (from ollama --version)
pub fn get_ollama_version() -> Option<String> {
    let output = std::process::Command::new("ollama")
        .arg("--version")
        .output()
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Output is like "ollama version is 0.1.32"
        stdout.split_whitespace().last().map(|s| s.to_string())
    } else {
        None
    }
}

/// Refresh helper state from system
/// Call this periodically to update presence status
/// v0.0.27: Fix detection for helpers with provides_command (like Ollama)
/// v0.0.28: Only track helpers that are relevant to this system
pub fn refresh_helper_states() -> HelpersManifest {
    let mut manifest = HelpersManifest::load();
    // v0.0.28: Only refresh relevant helpers (filtered by system hardware)
    let definitions = get_relevant_helper_definitions();

    for def in &definitions {
        // v0.0.27: Check provides_command first (for non-pacman installs like Ollama)
        // Then fall back to package check
        let (present, version) = if let Some(ref cmd) = def.provides_command {
            if is_command_available(cmd) {
                // Command exists - try to get version
                let ver = if cmd == "ollama" {
                    get_ollama_version()
                } else {
                    get_package_version(&def.name)
                };
                (true, ver)
            } else {
                // Command not found, check if package is installed anyway
                let pkg_present = is_package_present(&def.name);
                let ver = if pkg_present {
                    get_package_version(&def.name)
                } else {
                    None
                };
                (pkg_present, ver)
            }
        } else {
            let pkg_present = is_package_present(&def.name);
            let ver = if pkg_present {
                get_package_version(&def.name)
            } else {
                None
            };
            (pkg_present, ver)
        };

        let state = manifest.get_or_create(def);

        // If this is a new helper being tracked
        if state.first_seen.is_none() && present {
            // Helper was present before we started tracking → user installed
            state.first_seen = Some(Utc::now());
            state.installed_by = InstalledBy::User;
            state.present = true;
            state.version = version;
        } else {
            state.update_presence(present, version);
        }
    }

    manifest.last_updated = Some(Utc::now());
    let _ = manifest.save();
    manifest
}

/// Get helper states for status display (without modifying)
/// v0.0.28: Use provides_command for detection (like Ollama)
/// v0.0.28: Only show helpers that are relevant to this system's hardware
pub fn get_helper_status_list() -> Vec<HelperState> {
    let manifest = HelpersManifest::load();
    // v0.0.28: Use relevant helpers only (filtered by system hardware)
    let definitions = get_relevant_helper_definitions();

    definitions
        .iter()
        .map(|def| {
            // v0.0.28: Check provides_command first for non-pacman installs
            let (present, version) = check_helper_presence(def);

            if let Some(state) = manifest.get(&def.name) {
                let mut state = state.clone();
                state.present = present;
                state.version = version;
                state
            } else {
                if present {
                    HelperState::new_user_installed(&def.name, &def.purpose, version)
                } else {
                    HelperState::new_missing(&def.name, &def.purpose)
                }
            }
        })
        .collect()
}

/// Check if a helper is present - v0.0.28: unified detection
fn check_helper_presence(def: &HelperDefinition) -> (bool, Option<String>) {
    // Check provides_command first (for non-pacman installs like Ollama)
    if let Some(ref cmd) = def.provides_command {
        if is_command_available(cmd) {
            let ver = if cmd == "ollama" {
                get_ollama_version()
            } else {
                get_package_version(&def.name)
            };
            return (true, ver);
        }
    }
    // Fall back to package check
    let pkg_present = is_package_present(&def.name);
    let ver = if pkg_present {
        get_package_version(&def.name)
    } else {
        None
    };
    (pkg_present, ver)
}

/// Summary for status snapshot
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HelpersSummary {
    /// Total helpers tracked
    pub total: usize,
    /// Helpers currently present
    pub present: usize,
    /// Helpers missing
    pub missing: usize,
    /// Helpers installed by Anna
    pub installed_by_anna: usize,
    /// Helpers eligible for removal on uninstall
    pub removal_eligible: usize,
    /// Individual helper statuses
    pub helpers: Vec<HelperStatusEntry>,
}

/// Single helper entry for status display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelperStatusEntry {
    pub name: String,
    pub purpose: String,
    pub present: bool,
    pub installed_by: InstalledBy,
    pub first_seen: Option<DateTime<Utc>>,
    pub anna_install_timestamp: Option<DateTime<Utc>>,
}

impl From<&HelperState> for HelperStatusEntry {
    fn from(state: &HelperState) -> Self {
        Self {
            name: state.name.clone(),
            purpose: state.purpose.clone(),
            present: state.present,
            installed_by: state.installed_by,
            first_seen: state.first_seen,
            anna_install_timestamp: state.anna_install_timestamp,
        }
    }
}

/// Get helper summary for status snapshot
pub fn get_helpers_summary() -> HelpersSummary {
    let states = get_helper_status_list();

    let present = states.iter().filter(|h| h.present).count();
    let missing = states.iter().filter(|h| !h.present).count();
    let installed_by_anna = states
        .iter()
        .filter(|h| h.installed_by == InstalledBy::Anna)
        .count();
    let removal_eligible = states.iter().filter(|h| h.removal_eligible).count();

    let helpers: Vec<HelperStatusEntry> = states.iter().map(|h| h.into()).collect();

    HelpersSummary {
        total: states.len(),
        present,
        missing,
        installed_by_anna,
        removal_eligible,
        helpers,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_helper_state_new_missing() {
        let state = HelperState::new_missing("test-pkg", "Testing");
        assert!(!state.present);
        assert_eq!(state.installed_by, InstalledBy::Unknown);
        assert!(state.first_seen.is_none());
        assert!(!state.removal_eligible);
    }

    #[test]
    fn test_helper_state_new_user_installed() {
        let state = HelperState::new_user_installed("test-pkg", "Testing", Some("1.0".into()));
        assert!(state.present);
        assert_eq!(state.installed_by, InstalledBy::User);
        assert!(state.first_seen.is_some());
        assert!(!state.removal_eligible);
    }

    #[test]
    fn test_helper_state_mark_anna_installed() {
        let mut state = HelperState::new_missing("test-pkg", "Testing");
        state.mark_anna_installed(Some("2.0".into()));

        assert!(state.present);
        assert_eq!(state.installed_by, InstalledBy::Anna);
        assert!(state.anna_install_timestamp.is_some());
        assert!(state.removal_eligible);
    }

    #[test]
    fn test_helper_state_already_present_not_anna() {
        // If user had it installed, Anna later "installing" it shouldn't change provenance
        // Actually, if Anna installs it, it SHOULD change to anna
        let mut state = HelperState::new_user_installed("test-pkg", "Testing", Some("1.0".into()));
        assert_eq!(state.installed_by, InstalledBy::User);

        // Anna "installs" it (e.g., runs pacman -S even though present)
        state.mark_anna_installed(Some("1.0".into()));
        // Now Anna owns it
        assert_eq!(state.installed_by, InstalledBy::Anna);
        assert!(state.removal_eligible);
    }

    #[test]
    fn test_helper_definitions() {
        let defs = get_helper_definitions();
        assert!(!defs.is_empty());

        // Check required helpers are present
        let names: Vec<&str> = defs.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"lm_sensors"));
        assert!(names.contains(&"smartmontools"));
        assert!(names.contains(&"ollama"));
    }

    #[test]
    fn test_manifest_removal_eligible() {
        let mut manifest = HelpersManifest::default();

        // Add user-installed helper
        let user_state = HelperState::new_user_installed("user-pkg", "User installed", None);
        manifest.update_helper(user_state);

        // Add anna-installed helper
        let mut anna_state = HelperState::new_missing("anna-pkg", "Anna installed");
        anna_state.mark_anna_installed(Some("1.0".into()));
        manifest.update_helper(anna_state);

        // Only anna-installed should be removal eligible
        let eligible = manifest.removal_eligible();
        assert_eq!(eligible.len(), 1);
        assert_eq!(eligible[0].name, "anna-pkg");
    }

    #[test]
    fn test_installed_by_display() {
        assert_eq!(format!("{}", InstalledBy::Anna), "Anna");
        assert_eq!(format!("{}", InstalledBy::User), "user");
        assert_eq!(format!("{}", InstalledBy::Unknown), "unknown");
    }

    #[test]
    fn test_format_status() {
        let state = HelperState::new_user_installed("ethtool", "Network diag", Some("5.0".into()));
        let status = state.format_status();
        assert!(status.contains("ethtool"));
        assert!(status.contains("present"));
        assert!(status.contains("user"));
    }
}
