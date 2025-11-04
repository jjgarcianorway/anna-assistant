//! Configurator module - unified configuration system for Anna v1.3
//!
//! This module provides:
//! - Profile management (minimal, beautiful, workstation, gaming, server)
//! - User priorities (performance, responsiveness, battery, aesthetics, etc.)
//! - Desktop/terminal preferences
//! - Safety and rollback settings
//! - Scheduler configuration
//! - Module scope management
//!
//! Configuration is stored in:
//! - ~/.config/anna/anna.yaml (master config)
//! - ~/.config/anna/priorities.yaml (user priorities)
//! - ~/.config/anna/profiles/*.yaml (custom profiles)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// ═══════════════════════════════════════════════════════════════════════════════
// Configuration Paths
// ═══════════════════════════════════════════════════════════════════════════════

/// Get ~/.config/anna directory
pub fn config_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .context("Cannot determine home directory")?;

    let mut path = PathBuf::from(home);
    path.push(".config");
    path.push("anna");

    Ok(path)
}

/// Get ~/.config/anna/anna.yaml path
pub fn anna_config_path() -> Result<PathBuf> {
    let mut path = config_dir()?;
    path.push("anna.yaml");
    Ok(path)
}

/// Get ~/.config/anna/priorities.yaml path
pub fn priorities_config_path() -> Result<PathBuf> {
    let mut path = config_dir()?;
    path.push("priorities.yaml");
    Ok(path)
}

/// Get ~/.config/anna/profiles/ directory
pub fn profiles_dir() -> Result<PathBuf> {
    let mut path = config_dir()?;
    path.push("profiles");
    Ok(path)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Master Configuration (anna.yaml)
// ═══════════════════════════════════════════════════════════════════════════════

/// Master configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterConfig {
    /// Schema version for migrations
    #[serde(default = "default_version")]
    pub version: u32,

    /// Current profile name
    #[serde(default = "default_profile")]
    pub profile: String,

    /// Autonomy level
    #[serde(default)]
    pub autonomy: AutonomyLevel,

    /// Stability preference
    #[serde(default)]
    pub stability: StabilityLevel,

    /// Privacy mode
    #[serde(default)]
    pub privacy: PrivacyMode,

    /// Desktop preferences
    #[serde(default)]
    pub desktop: DesktopConfig,

    /// Safety and rollback settings
    #[serde(default)]
    pub safety: SafetyConfig,

    /// Scheduler configuration
    #[serde(default)]
    pub scheduler: SchedulerConfig,

    /// Module scope (enabled/disabled categories)
    #[serde(default)]
    pub modules: ModulesConfig,

    /// Files to include/merge
    #[serde(default)]
    pub includes: Vec<String>,
}

fn default_version() -> u32 {
    1
}

fn default_profile() -> String {
    "default".to_string()
}

impl Default for MasterConfig {
    fn default() -> Self {
        Self {
            version: 1,
            profile: "default".to_string(),
            autonomy: AutonomyLevel::default(),
            stability: StabilityLevel::default(),
            privacy: PrivacyMode::default(),
            desktop: DesktopConfig::default(),
            safety: SafetyConfig::default(),
            scheduler: SchedulerConfig::default(),
            modules: ModulesConfig::default(),
            includes: vec!["~/.config/anna/priorities.yaml".to_string()],
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Autonomy Level
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutonomyLevel {
    /// Only provide advice, never apply automatically
    AdviceOnly,
    /// Automatically apply low-risk changes
    AutoLowRisk,
}

impl Default for AutonomyLevel {
    fn default() -> Self {
        Self::AdviceOnly
    }
}

impl std::fmt::Display for AutonomyLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AdviceOnly => write!(f, "advice_only"),
            Self::AutoLowRisk => write!(f, "auto_low_risk"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Stability Level
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StabilityLevel {
    /// Proven, LTS-like stability
    Conservative,
    /// Standard update cadence
    Balanced,
    /// Bleeding edge, latest packages
    Fast,
}

impl Default for StabilityLevel {
    fn default() -> Self {
        Self::Balanced
    }
}

impl std::fmt::Display for StabilityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Conservative => write!(f, "conservative"),
            Self::Balanced => write!(f, "balanced"),
            Self::Fast => write!(f, "fast"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Privacy Mode
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyMode {
    /// No usage tracking at all
    Strict,
    /// Only aggregate command counts
    Minimal,
}

impl Default for PrivacyMode {
    fn default() -> Self {
        Self::Strict
    }
}

impl std::fmt::Display for PrivacyMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Strict => write!(f, "strict"),
            Self::Minimal => write!(f, "minimal"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// User Priorities (priorities.yaml)
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrioritiesConfig {
    #[serde(default)]
    pub performance: Priority,

    #[serde(default)]
    pub responsiveness: Priority,

    #[serde(default)]
    pub battery_life: Priority,

    #[serde(default)]
    pub aesthetics: Priority,

    #[serde(default)]
    pub stability: Priority,

    #[serde(default)]
    pub hands_off: Priority,

    #[serde(default)]
    pub privacy: Priority,
}

impl Default for PrioritiesConfig {
    fn default() -> Self {
        Self {
            performance: Priority::Balanced,
            responsiveness: Priority::Balanced,
            battery_life: Priority::Balanced,
            aesthetics: Priority::Beautiful,
            stability: Priority::Balanced,
            hands_off: Priority::AdviceOnly,
            privacy: Priority::Strict,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    // Generic levels
    Conservative,
    Balanced,
    Maximum,

    // Aesthetics-specific
    Minimal,
    Pleasant,
    Beautiful,

    // Hands-off specific
    AdviceOnly,
    AutoLowRisk,

    // Privacy-specific
    Strict,
    MinimalPrivacy, // Renamed to avoid conflict with Minimal

    // Responsiveness-specific
    Relaxed,
    Instant,

    // Battery-specific
    Performance,
    Maxsave,

    // Stability-specific
    Cutting,
}

impl Default for Priority {
    fn default() -> Self {
        Self::Balanced
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Desktop & Terminal Preferences
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopConfig {
    /// Offer aesthetic improvements
    #[serde(default = "default_true")]
    pub beautify_desktop: bool,

    /// Apply theme automatically (requires auto_low_risk autonomy)
    #[serde(default = "default_false")]
    pub auto_apply_theme: bool,

    /// Preview diffs before changes
    #[serde(default = "default_true")]
    pub preview_diffs: bool,

    /// Suggest font improvements
    #[serde(default = "default_true")]
    pub suggest_fonts: bool,

    /// Auto-apply color schemes
    #[serde(default = "default_false")]
    pub auto_apply_colors: bool,

    /// Respect existing configuration
    #[serde(default = "default_true")]
    pub respect_existing: bool,

    /// Never suggest changes to these components
    #[serde(default)]
    pub never_suggest: Vec<String>,
}

impl Default for DesktopConfig {
    fn default() -> Self {
        Self {
            beautify_desktop: true,
            auto_apply_theme: false,
            preview_diffs: true,
            suggest_fonts: true,
            auto_apply_colors: false,
            respect_existing: true,
            never_suggest: Vec::new(),
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

// ═══════════════════════════════════════════════════════════════════════════════
// Safety & Rollback
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    /// Keep snapshots for N days
    #[serde(default = "default_snapshot_retention")]
    pub snapshot_retention_days: u32,

    /// Maximum number of snapshots to keep
    #[serde(default = "default_max_snapshots")]
    pub max_snapshots: u32,

    /// Auto-prune old snapshots
    #[serde(default = "default_true")]
    pub auto_prune: bool,

    /// Confirmation policy
    #[serde(default)]
    pub confirmation: ConfirmationPolicy,
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            snapshot_retention_days: 30,
            max_snapshots: 20,
            auto_prune: true,
            confirmation: ConfirmationPolicy::default(),
        }
    }
}

fn default_snapshot_retention() -> u32 {
    30
}

fn default_max_snapshots() -> u32 {
    20
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfirmationPolicy {
    /// Always ask before changes
    AlwaysAsk,
    /// Auto-apply low-risk only
    AutoLowRisk,
    /// Manual mode (never auto)
    Manual,
}

impl Default for ConfirmationPolicy {
    fn default() -> Self {
        Self::AlwaysAsk
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Scheduler
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// Fact collection interval in hours
    #[serde(default = "default_fact_interval")]
    pub fact_interval_hours: u32,

    /// Jitter in minutes (± random)
    #[serde(default = "default_jitter")]
    pub jitter_minutes: u32,

    /// Enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Quiet hours
    #[serde(default)]
    pub quiet_hours: Option<QuietHours>,

    /// Scheduled tasks
    #[serde(default)]
    pub tasks: Vec<ScheduledTask>,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            fact_interval_hours: 4,
            jitter_minutes: 15,
            enabled: true,
            quiet_hours: Some(QuietHours::default()),
            tasks: vec![
                ScheduledTask {
                    name: "doctor_check".to_string(),
                    enabled: true,
                    schedule: "weekly".to_string(),
                    time: "09:00".to_string(),
                },
                ScheduledTask {
                    name: "prune_snapshots".to_string(),
                    enabled: true,
                    schedule: "monthly".to_string(),
                    time: "02:00".to_string(),
                },
            ],
        }
    }
}

fn default_fact_interval() -> u32 {
    4
}

fn default_jitter() -> u32 {
    15
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuietHours {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default = "default_quiet_start")]
    pub start: String,

    #[serde(default = "default_quiet_end")]
    pub end: String,

    /// Skip collection during quiet hours (vs collect silently)
    #[serde(default = "default_true")]
    pub skip_collection: bool,
}

impl Default for QuietHours {
    fn default() -> Self {
        Self {
            enabled: true,
            start: "22:00".to_string(),
            end: "07:00".to_string(),
            skip_collection: true,
        }
    }
}

fn default_quiet_start() -> String {
    "22:00".to_string()
}

fn default_quiet_end() -> String {
    "07:00".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub name: String,
    pub enabled: bool,
    pub schedule: String, // "daily", "weekly", "monthly"
    pub time: String,      // HH:MM format
}

// ═══════════════════════════════════════════════════════════════════════════════
// Module Scope
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulesConfig {
    #[serde(default = "default_true")]
    pub editor_ux: bool,

    #[serde(default = "default_true")]
    pub desktop_env: bool,

    #[serde(default = "default_true")]
    pub graphics: bool,

    #[serde(default = "default_true")]
    pub network: bool,

    #[serde(default = "default_true")]
    pub package_hygiene: bool,

    #[serde(default = "default_true")]
    pub security: bool,

    #[serde(default = "default_false")]
    pub power: bool,

    #[serde(default = "default_true")]
    pub performance: bool,

    #[serde(default = "default_true")]
    pub storage: bool,

    #[serde(default = "default_false")]
    pub gaming: bool,
}

impl Default for ModulesConfig {
    fn default() -> Self {
        Self {
            editor_ux: true,
            desktop_env: true,
            graphics: true,
            network: true,
            package_hygiene: true,
            security: true,
            power: false,
            performance: true,
            storage: true,
            gaming: false,
        }
    }
}

impl ModulesConfig {
    /// Get list of enabled modules
    pub fn enabled_modules(&self) -> Vec<&str> {
        let mut enabled = Vec::new();
        if self.editor_ux {
            enabled.push("editor-ux");
        }
        if self.desktop_env {
            enabled.push("desktop-env");
        }
        if self.graphics {
            enabled.push("graphics");
        }
        if self.network {
            enabled.push("network");
        }
        if self.package_hygiene {
            enabled.push("package-hygiene");
        }
        if self.security {
            enabled.push("security");
        }
        if self.power {
            enabled.push("power");
        }
        if self.performance {
            enabled.push("performance");
        }
        if self.storage {
            enabled.push("storage");
        }
        if self.gaming {
            enabled.push("gaming");
        }
        enabled
    }

    /// Count enabled modules
    pub fn enabled_count(&self) -> usize {
        self.enabled_modules().len()
    }

    /// Count disabled modules
    pub fn disabled_count(&self) -> usize {
        10 - self.enabled_count()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Profile Management
// ═══════════════════════════════════════════════════════════════════════════════

/// Built-in profile templates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileTemplate {
    pub name: String,
    pub description: String,
    pub config: MasterConfig,
    pub priorities: PrioritiesConfig,
}

/// Get bundled profile templates
pub fn bundled_profiles() -> HashMap<String, ProfileTemplate> {
    let mut profiles = HashMap::new();

    // minimal profile
    profiles.insert(
        "minimal".to_string(),
        ProfileTemplate {
            name: "minimal".to_string(),
            description: "Lean, essential packages only".to_string(),
            config: MasterConfig {
                autonomy: AutonomyLevel::AdviceOnly,
                stability: StabilityLevel::Conservative,
                ..Default::default()
            },
            priorities: PrioritiesConfig {
                performance: Priority::Conservative,
                aesthetics: Priority::Minimal,
                stability: Priority::Conservative,
                ..Default::default()
            },
        },
    );

    // beautiful profile (default)
    profiles.insert(
        "beautiful".to_string(),
        ProfileTemplate {
            name: "beautiful".to_string(),
            description: "Calm, elegant aesthetics".to_string(),
            config: MasterConfig {
                autonomy: AutonomyLevel::AdviceOnly,
                stability: StabilityLevel::Balanced,
                ..Default::default()
            },
            priorities: PrioritiesConfig {
                aesthetics: Priority::Beautiful,
                ..Default::default()
            },
        },
    );

    // workstation profile
    profiles.insert(
        "workstation".to_string(),
        ProfileTemplate {
            name: "workstation".to_string(),
            description: "Development-focused tools".to_string(),
            config: MasterConfig {
                autonomy: AutonomyLevel::AutoLowRisk,
                stability: StabilityLevel::Balanced,
                modules: ModulesConfig {
                    editor_ux: true,
                    desktop_env: true,
                    graphics: true,
                    network: true,
                    package_hygiene: true,
                    security: true,
                    power: false,
                    performance: true,
                    storage: true,
                    gaming: false,
                },
                ..Default::default()
            },
            priorities: PrioritiesConfig {
                performance: Priority::Balanced,
                responsiveness: Priority::Instant,
                aesthetics: Priority::Pleasant,
                ..Default::default()
            },
        },
    );

    // gaming profile
    profiles.insert(
        "gaming".to_string(),
        ProfileTemplate {
            name: "gaming".to_string(),
            description: "Performance first, low latency".to_string(),
            config: MasterConfig {
                autonomy: AutonomyLevel::AutoLowRisk,
                stability: StabilityLevel::Fast,
                modules: ModulesConfig {
                    editor_ux: false,
                    desktop_env: true,
                    graphics: true,
                    network: true,
                    package_hygiene: true,
                    security: true,
                    power: false,
                    performance: true,
                    storage: true,
                    gaming: true,
                },
                ..Default::default()
            },
            priorities: PrioritiesConfig {
                performance: Priority::Maximum,
                responsiveness: Priority::Instant,
                aesthetics: Priority::Minimal,
                battery_life: Priority::Performance,
                ..Default::default()
            },
        },
    );

    // server profile
    profiles.insert(
        "server".to_string(),
        ProfileTemplate {
            name: "server".to_string(),
            description: "Stability and uptime focused".to_string(),
            config: MasterConfig {
                autonomy: AutonomyLevel::AdviceOnly,
                stability: StabilityLevel::Conservative,
                modules: ModulesConfig {
                    editor_ux: false,
                    desktop_env: false,
                    graphics: false,
                    network: true,
                    package_hygiene: true,
                    security: true,
                    power: false,
                    performance: true,
                    storage: true,
                    gaming: false,
                },
                ..Default::default()
            },
            priorities: PrioritiesConfig {
                performance: Priority::Conservative,
                stability: Priority::Conservative,
                aesthetics: Priority::Minimal,
                ..Default::default()
            },
        },
    );

    profiles
}

// ═══════════════════════════════════════════════════════════════════════════════
// Load/Save Functions
// ═══════════════════════════════════════════════════════════════════════════════

/// Load master configuration from anna.yaml
pub fn load_master_config() -> Result<MasterConfig> {
    let path = anna_config_path()?;

    if !path.exists() {
        return Ok(MasterConfig::default());
    }

    let content = fs::read_to_string(&path)
        .context("Failed to read anna.yaml")?;

    let config: MasterConfig = serde_yaml::from_str(&content)
        .context("Failed to parse anna.yaml")?;

    Ok(config)
}

/// Save master configuration to anna.yaml
pub fn save_master_config(config: &MasterConfig) -> Result<()> {
    let dir = config_dir()?;
    fs::create_dir_all(&dir).context("Failed to create config directory")?;

    let path = anna_config_path()?;
    let yaml = serde_yaml::to_string(config)
        .context("Failed to serialize config")?;

    fs::write(&path, yaml).context("Failed to write anna.yaml")?;

    Ok(())
}

/// Load priorities configuration from priorities.yaml
pub fn load_priorities_config() -> Result<PrioritiesConfig> {
    let path = priorities_config_path()?;

    if !path.exists() {
        return Ok(PrioritiesConfig::default());
    }

    let content = fs::read_to_string(&path)
        .context("Failed to read priorities.yaml")?;

    let config: PrioritiesConfig = serde_yaml::from_str(&content)
        .context("Failed to parse priorities.yaml")?;

    Ok(config)
}

/// Save priorities configuration to priorities.yaml
pub fn save_priorities_config(config: &PrioritiesConfig) -> Result<()> {
    let dir = config_dir()?;
    fs::create_dir_all(&dir).context("Failed to create config directory")?;

    let path = priorities_config_path()?;
    let yaml = serde_yaml::to_string(config)
        .context("Failed to serialize priorities")?;

    fs::write(&path, yaml).context("Failed to write priorities.yaml")?;

    Ok(())
}

/// Load a profile template (bundled or custom)
pub fn load_profile(name: &str) -> Result<ProfileTemplate> {
    // Check bundled profiles first
    if let Some(profile) = bundled_profiles().get(name) {
        return Ok(profile.clone());
    }

    // Check custom profile in ~/.config/anna/profiles/
    let mut path = profiles_dir()?;
    path.push(format!("{}.yaml", name));

    if !path.exists() {
        anyhow::bail!("Profile '{}' not found", name);
    }

    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read profile '{}'", name))?;

    let profile: ProfileTemplate = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse profile '{}'", name))?;

    Ok(profile)
}

/// Save a custom profile
pub fn save_profile(profile: &ProfileTemplate) -> Result<()> {
    let dir = profiles_dir()?;
    fs::create_dir_all(&dir).context("Failed to create profiles directory")?;

    let mut path = dir;
    path.push(format!("{}.yaml", profile.name));

    let yaml = serde_yaml::to_string(profile)
        .context("Failed to serialize profile")?;

    fs::write(&path, yaml)
        .with_context(|| format!("Failed to write profile '{}'", profile.name))?;

    Ok(())
}

/// List available profiles (bundled + custom)
pub fn list_profiles() -> Result<Vec<String>> {
    let mut profiles: Vec<String> = bundled_profiles().keys().cloned().collect();

    // Add custom profiles
    if let Ok(dir) = profiles_dir() {
        if dir.exists() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        if !profiles.contains(&name.to_string()) {
                            profiles.push(name.to_string());
                        }
                    }
                }
            }
        }
    }

    profiles.sort();
    Ok(profiles)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_master_config() {
        let config = MasterConfig::default();
        assert_eq!(config.version, 1);
        assert_eq!(config.profile, "default");
        assert_eq!(config.autonomy, AutonomyLevel::AdviceOnly);
        assert_eq!(config.stability, StabilityLevel::Balanced);
    }

    #[test]
    fn test_bundled_profiles() {
        let profiles = bundled_profiles();
        assert!(profiles.contains_key("minimal"));
        assert!(profiles.contains_key("beautiful"));
        assert!(profiles.contains_key("workstation"));
        assert!(profiles.contains_key("gaming"));
        assert!(profiles.contains_key("server"));
    }

    #[test]
    fn test_modules_config() {
        let modules = ModulesConfig::default();
        assert!(modules.enabled_count() > 0);
        assert!(modules.enabled_modules().contains(&"security"));
    }

    #[test]
    fn test_serialization() {
        let config = MasterConfig::default();
        let yaml = serde_yaml::to_string(&config).unwrap();
        let deserialized: MasterConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(config.version, deserialized.version);
        assert_eq!(config.profile, deserialized.profile);
    }
}
