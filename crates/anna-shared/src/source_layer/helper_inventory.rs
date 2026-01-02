//! Helper Inventory - v0.0.443.
//!
//! Helper registry for attribution:
//! - Track which helpers Anna installed vs preexisting

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::inventory_common::InstalledBy;

/// Helper installation method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallMethod {
    /// Via pacman.
    Pacman,
    /// Via AUR helper.
    Aur,
    /// Via script.
    Script,
    /// Manual installation.
    Manual,
    /// Unknown.
    Unknown,
}

/// Helper entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelperEntry {
    /// Helper name (e.g., "ollama").
    pub name: String,
    /// Whether installed.
    pub installed: bool,
    /// Who installed it.
    pub installed_by: InstalledBy,
    /// Installation method.
    pub install_method: InstallMethod,
    /// Installation evidence (commands run).
    pub install_evidence: Vec<String>,
    /// Version.
    pub version: Option<String>,
    /// Last checked timestamp.
    pub last_checked_at: Option<String>,
    /// Whether required by Anna.
    pub required: bool,
}

impl HelperEntry {
    /// Create new entry.
    pub fn new(name: &str, required: bool) -> Self {
        Self {
            name: name.to_string(),
            installed: false,
            installed_by: InstalledBy::Unknown,
            install_method: InstallMethod::Unknown,
            install_evidence: Vec::new(),
            version: None,
            last_checked_at: None,
            required,
        }
    }

    /// Mark as installed by Anna.
    pub fn installed_by_anna(mut self, method: InstallMethod, evidence: Vec<String>) -> Self {
        self.installed = true;
        self.installed_by = InstalledBy::Anna;
        self.install_method = method;
        self.install_evidence = evidence;
        self
    }

    /// Mark as preexisting (user).
    pub fn preexisting(mut self) -> Self {
        self.installed = true;
        self.installed_by = InstalledBy::User;
        self
    }
}

/// Helper registry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HelperRegistry {
    /// Helpers by name.
    pub helpers: HashMap<String, HelperEntry>,
}

impl HelperRegistry {
    /// Registry file path.
    pub const FILE_PATH: &'static str = "/var/lib/anna/helpers/registry.json";

    /// Create empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with default helpers.
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();

        // Required helpers
        registry.upsert(HelperEntry::new("ollama", true));

        // Optional helpers
        registry.upsert(HelperEntry::new("paru", false));
        registry.upsert(HelperEntry::new("yay", false));

        registry
    }

    /// Add or update helper.
    pub fn upsert(&mut self, entry: HelperEntry) {
        self.helpers.insert(entry.name.clone(), entry);
    }

    /// Get helper.
    pub fn get(&self, name: &str) -> Option<&HelperEntry> {
        self.helpers.get(name)
    }

    /// Get mutable helper.
    pub fn get_mut(&mut self, name: &str) -> Option<&mut HelperEntry> {
        self.helpers.get_mut(name)
    }

    /// Record installation by Anna.
    pub fn record_anna_install(&mut self, name: &str, method: InstallMethod, command: &str) {
        if let Some(entry) = self.helpers.get_mut(name) {
            entry.installed = true;
            entry.installed_by = InstalledBy::Anna;
            entry.install_method = method;
            entry.install_evidence.push(command.to_string());
        }
    }

    /// Get status summary.
    pub fn status_summary(&self) -> Vec<HelperStatus> {
        self.helpers
            .values()
            .map(|h| HelperStatus {
                name: h.name.clone(),
                installed: h.installed,
                installed_by: h.installed_by,
                version: h.version.clone(),
                required: h.required,
            })
            .collect()
    }

    /// Save to file.
    pub fn save(&self) -> Result<(), String> {
        let dir = std::path::Path::new(Self::FILE_PATH).parent().unwrap();
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;

        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(Self::FILE_PATH, json).map_err(|e| e.to_string())
    }

    /// Load from file.
    pub fn load() -> Result<Self, String> {
        let content = std::fs::read_to_string(Self::FILE_PATH).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    }
}

/// Helper status for display.
#[derive(Debug, Clone)]
pub struct HelperStatus {
    /// Helper name.
    pub name: String,
    /// Installed?
    pub installed: bool,
    /// Who installed.
    pub installed_by: InstalledBy,
    /// Version.
    pub version: Option<String>,
    /// Required?
    pub required: bool,
}

impl HelperStatus {
    /// Format for display.
    pub fn display(&self) -> String {
        let status = if self.installed {
            "installed"
        } else {
            "not installed"
        };
        let by = match self.installed_by {
            InstalledBy::Anna => "anna",
            InstalledBy::User => "user",
            InstalledBy::Unknown => "unknown",
        };
        let req = if self.required {
            "required"
        } else {
            "optional"
        };
        let ver = self.version.as_deref().unwrap_or("?");

        format!("{}: {} (by {}, {}, v{})", self.name, status, by, req, ver)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_helper_registry() {
        let mut registry = HelperRegistry::with_defaults();

        registry.record_anna_install("ollama", InstallMethod::Pacman, "sudo pacman -S ollama");

        let ollama = registry.get("ollama").unwrap();
        assert_eq!(ollama.installed_by, InstalledBy::Anna);
        assert!(!ollama.install_evidence.is_empty());
    }
}
