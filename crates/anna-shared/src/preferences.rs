//! User preferences for customizing Anna's behavior.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// User preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// What to include in morning briefings.
    pub briefing: BriefingPreferences,
    /// Alert preferences.
    pub alerts: AlertPreferences,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            briefing: BriefingPreferences::default(),
            alerts: AlertPreferences::default(),
        }
    }
}

/// What to include in morning briefings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefingPreferences {
    pub updates: bool,
    pub security: bool,
    pub errors: bool,
    pub disk: bool,
    pub memory: bool,
    pub services: bool,
    pub load: bool,
    pub network: bool,
    pub hardware: bool,
}

impl Default for BriefingPreferences {
    fn default() -> Self {
        Self {
            updates: true,
            security: true,
            errors: true,
            disk: true,
            memory: true,
            services: true,
            load: true,
            network: false, // Off by default, can be noisy
            hardware: false, // Off by default, needs smartctl
        }
    }
}

/// Alert preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertPreferences {
    /// Alert on RAM anomalies.
    pub ram_anomaly: bool,
    /// Alert on CPU anomalies.
    pub cpu_anomaly: bool,
    /// Alert on disk anomalies.
    pub disk_anomaly: bool,
    /// Alert on network anomalies.
    pub network_anomaly: bool,
    /// Alert on failed services.
    pub service_failures: bool,
    /// Alert on security events.
    pub security_events: bool,
}

impl Default for AlertPreferences {
    fn default() -> Self {
        Self {
            ram_anomaly: true,
            cpu_anomaly: true,
            disk_anomaly: true,
            network_anomaly: false, // Off by default
            service_failures: true,
            security_events: true,
        }
    }
}

impl UserPreferences {
    fn path() -> PathBuf {
        PathBuf::from("/var/lib/anna/preferences.json")
    }

    pub fn load() -> Self {
        std::fs::read_to_string(Self::path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> std::io::Result<()> {
        if let Some(parent) = Self::path().parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(Self::path(), json)
    }
}

/// Parse preference update requests.
/// e.g., "add network to my briefing", "disable security alerts"
pub fn parse_preference_update(input: &str) -> Option<PreferenceUpdate> {
    let lower = input.to_lowercase();

    // Briefing modifications
    if lower.contains("briefing") || lower.contains("morning") || lower.contains("report") {
        if lower.contains("add") || lower.contains("include") || lower.contains("enable") {
            if lower.contains("network") {
                return Some(PreferenceUpdate::Briefing("network".into(), true));
            }
            if lower.contains("hardware") || lower.contains("smart") {
                return Some(PreferenceUpdate::Briefing("hardware".into(), true));
            }
        }
        if lower.contains("remove") || lower.contains("disable") || lower.contains("hide") {
            if lower.contains("update") {
                return Some(PreferenceUpdate::Briefing("updates".into(), false));
            }
            if lower.contains("security") {
                return Some(PreferenceUpdate::Briefing("security".into(), false));
            }
            if lower.contains("error") {
                return Some(PreferenceUpdate::Briefing("errors".into(), false));
            }
        }
    }

    // Alert modifications
    if lower.contains("alert") {
        if lower.contains("enable") || lower.contains("add") {
            if lower.contains("network") {
                return Some(PreferenceUpdate::Alert("network_anomaly".into(), true));
            }
        }
        if lower.contains("disable") || lower.contains("remove") {
            if lower.contains("ram") || lower.contains("memory") {
                return Some(PreferenceUpdate::Alert("ram_anomaly".into(), false));
            }
        }
    }

    None
}

/// A preference update action.
#[derive(Debug, Clone)]
pub enum PreferenceUpdate {
    Briefing(String, bool),
    Alert(String, bool),
}

impl PreferenceUpdate {
    /// Apply this update to preferences.
    pub fn apply(&self, prefs: &mut UserPreferences) -> String {
        match self {
            PreferenceUpdate::Briefing(key, enabled) => {
                let status = if *enabled { "enabled" } else { "disabled" };
                match key.as_str() {
                    "updates" => prefs.briefing.updates = *enabled,
                    "security" => prefs.briefing.security = *enabled,
                    "errors" => prefs.briefing.errors = *enabled,
                    "disk" => prefs.briefing.disk = *enabled,
                    "memory" => prefs.briefing.memory = *enabled,
                    "services" => prefs.briefing.services = *enabled,
                    "load" => prefs.briefing.load = *enabled,
                    "network" => prefs.briefing.network = *enabled,
                    "hardware" => prefs.briefing.hardware = *enabled,
                    _ => return format!("Unknown briefing option: {}", key),
                }
                format!("{} {} in morning briefing", status, key)
            }
            PreferenceUpdate::Alert(key, enabled) => {
                let status = if *enabled { "enabled" } else { "disabled" };
                match key.as_str() {
                    "ram_anomaly" => prefs.alerts.ram_anomaly = *enabled,
                    "cpu_anomaly" => prefs.alerts.cpu_anomaly = *enabled,
                    "disk_anomaly" => prefs.alerts.disk_anomaly = *enabled,
                    "network_anomaly" => prefs.alerts.network_anomaly = *enabled,
                    "service_failures" => prefs.alerts.service_failures = *enabled,
                    "security_events" => prefs.alerts.security_events = *enabled,
                    _ => return format!("Unknown alert option: {}", key),
                }
                format!("{} {} alerts", status, key.replace("_", " "))
            }
        }
    }
}
