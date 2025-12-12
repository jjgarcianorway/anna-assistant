//! Accurate model and probe inventory (v0.0.444).
//!
//! Fixes:
//! - No more "and 22 more" with duplicates
//! - Track model ownership (user vs anna installed)
//! - Clean probe inventory with commands

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Model ownership - who installed this model?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelOwner {
    /// User installed this model (pre-existing or manual pull)
    User,
    /// Anna installed this model (auto-pulled for operation)
    Anna,
    /// Unknown ownership (legacy or unclear)
    Unknown,
}

/// A single model in the inventory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    /// Model name (e.g., "qwen2.5:7b")
    pub name: String,
    /// Normalized name for deduplication (lowercase, no tag variants)
    pub normalized: String,
    /// Owner (user/anna/unknown)
    pub owner: ModelOwner,
    /// Size in MB (if known)
    pub size_mb: Option<u64>,
    /// Quantization (if known)
    pub quantization: Option<String>,
    /// Last used timestamp (Unix ms)
    pub last_used_ms: Option<u64>,
    /// Is this configured for a role?
    pub is_configured: bool,
    /// Role it's configured for (if any)
    pub configured_role: Option<String>,
}

impl ModelEntry {
    /// Create a new model entry.
    pub fn new(name: impl Into<String>, owner: ModelOwner) -> Self {
        let name = name.into();
        let normalized = normalize_model_name(&name);
        Self {
            name,
            normalized,
            owner,
            size_mb: None,
            quantization: None,
            last_used_ms: None,
            is_configured: false,
            configured_role: None,
        }
    }

    /// Set size.
    pub fn with_size(mut self, size_mb: u64) -> Self {
        self.size_mb = Some(size_mb);
        self
    }

    /// Mark as configured for a role.
    pub fn with_role(mut self, role: &str) -> Self {
        self.is_configured = true;
        self.configured_role = Some(role.to_string());
        self
    }
}

/// Normalize model name for deduplication.
/// "qwen2.5:7b" and "qwen2.5:7b-instruct" are different
/// but "qwen2.5:7b" and "QWEN2.5:7B" are the same.
fn normalize_model_name(name: &str) -> String {
    name.to_lowercase().trim().to_string()
}

/// Accurate model inventory.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelInventory {
    /// All discovered models (deduplicated).
    pub models: HashMap<String, ModelEntry>,

    /// Configured models by role.
    pub configured: ConfiguredModels,

    /// Anna-installed model names.
    pub anna_installed: HashSet<String>,
}

/// Models configured for specific roles.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfiguredModels {
    pub translator: Option<String>,
    pub junior: Option<String>,
    pub senior: Option<String>,
}

impl ModelInventory {
    /// Create empty inventory.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a discovered model.
    pub fn add_discovered(&mut self, name: &str, owner: ModelOwner) {
        let entry = ModelEntry::new(name, owner);
        let key = entry.normalized.clone();
        self.models.entry(key).or_insert(entry);
    }

    /// Add a model that Anna installed.
    pub fn add_anna_installed(&mut self, name: &str) {
        let entry = ModelEntry::new(name, ModelOwner::Anna);
        let key = entry.normalized.clone();
        self.models.insert(key.clone(), entry);
        self.anna_installed.insert(key);
    }

    /// Set configured models.
    pub fn set_configured(
        &mut self,
        translator: Option<&str>,
        junior: Option<&str>,
        senior: Option<&str>,
    ) {
        self.configured = ConfiguredModels {
            translator: translator.map(String::from),
            junior: junior.map(String::from),
            senior: senior.map(String::from),
        };

        // Mark configured models in inventory
        if let Some(t) = &self.configured.translator {
            if let Some(m) = self.models.get_mut(&normalize_model_name(t)) {
                m.is_configured = true;
                m.configured_role = Some("translator".into());
            }
        }
        if let Some(j) = &self.configured.junior {
            if let Some(m) = self.models.get_mut(&normalize_model_name(j)) {
                m.is_configured = true;
                m.configured_role = Some("junior".into());
            }
        }
        if let Some(s) = &self.configured.senior {
            if let Some(m) = self.models.get_mut(&normalize_model_name(s)) {
                m.is_configured = true;
                m.configured_role = Some("senior".into());
            }
        }
    }

    /// Get total discovered model count.
    pub fn discovered_count(&self) -> usize {
        self.models.len()
    }

    /// Get count of Anna-installed models.
    pub fn anna_installed_count(&self) -> usize {
        self.anna_installed.len()
    }

    /// Get count of user-installed models.
    pub fn user_installed_count(&self) -> usize {
        self.models
            .values()
            .filter(|m| m.owner == ModelOwner::User)
            .count()
    }

    /// Get configured models that are actually present.
    pub fn configured_present(&self) -> Vec<(&str, &str)> {
        let mut result = Vec::new();
        if let Some(t) = &self.configured.translator {
            if self.models.contains_key(&normalize_model_name(t)) {
                result.push(("translator", t.as_str()));
            }
        }
        if let Some(j) = &self.configured.junior {
            if self.models.contains_key(&normalize_model_name(j)) {
                result.push(("junior", j.as_str()));
            }
        }
        if let Some(s) = &self.configured.senior {
            if self.models.contains_key(&normalize_model_name(s)) {
                result.push(("senior", s.as_str()));
            }
        }
        result
    }

    /// Get configured models that are missing.
    pub fn configured_missing(&self) -> Vec<(&str, &str)> {
        let mut result = Vec::new();
        if let Some(t) = &self.configured.translator {
            if !self.models.contains_key(&normalize_model_name(t)) {
                result.push(("translator", t.as_str()));
            }
        }
        if let Some(j) = &self.configured.junior {
            if !self.models.contains_key(&normalize_model_name(j)) {
                result.push(("junior", j.as_str()));
            }
        }
        if let Some(s) = &self.configured.senior {
            if !self.models.contains_key(&normalize_model_name(s)) {
                result.push(("senior", s.as_str()));
            }
        }
        result
    }

    /// Get top N models by name (sorted alphabetically).
    pub fn top_models(&self, n: usize) -> Vec<&ModelEntry> {
        let mut models: Vec<_> = self.models.values().collect();
        models.sort_by(|a, b| a.name.cmp(&b.name));
        models.truncate(n);
        models
    }

    /// Format for display in annactl status.
    pub fn display(&self, max_models: usize) -> String {
        let mut out = String::new();

        out.push_str("[models]\n");

        // Configured models (always show fully)
        out.push_str("  configured:\n");
        if let Some(t) = &self.configured.translator {
            let status = if self.models.contains_key(&normalize_model_name(t)) {
                "✓"
            } else {
                "✗ (missing)"
            };
            out.push_str(&format!("    translator: {} {}\n", t, status));
        }
        if let Some(j) = &self.configured.junior {
            let status = if self.models.contains_key(&normalize_model_name(j)) {
                "✓"
            } else {
                "✗ (missing)"
            };
            out.push_str(&format!("    junior:     {} {}\n", j, status));
        }
        if let Some(s) = &self.configured.senior {
            let status = if self.models.contains_key(&normalize_model_name(s)) {
                "✓"
            } else {
                "✗ (missing)"
            };
            out.push_str(&format!("    senior:     {} {}\n", s, status));
        }

        // Counts
        out.push_str(&format!(
            "  discovered_total:   {}\n",
            self.discovered_count()
        ));
        out.push_str(&format!(
            "  anna_installed:     {}\n",
            self.anna_installed_count()
        ));
        out.push_str(&format!(
            "  user_installed:     {}\n",
            self.user_installed_count()
        ));

        // Top models (no duplicates)
        if !self.models.is_empty() {
            out.push_str("  available:\n");
            let top = self.top_models(max_models);
            for m in &top {
                let owner = match m.owner {
                    ModelOwner::User => "[user]",
                    ModelOwner::Anna => "[anna]",
                    ModelOwner::Unknown => "",
                };
                out.push_str(&format!("    {} {}\n", m.name, owner));
            }
            let remaining = self.models.len().saturating_sub(max_models);
            if remaining > 0 {
                out.push_str(&format!("    (+{} more)\n", remaining));
            }
        }

        out
    }
}

/// A probe in the inventory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeEntry {
    /// Probe ID (e.g., "df", "free", "systemctl_status")
    pub id: String,
    /// Human-readable description
    pub description: String,
    /// Command to run
    pub command: String,
    /// Is this probe available (command exists)?
    pub available: bool,
}

/// Probe inventory.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProbeInventory {
    /// All probes by ID.
    pub probes: HashMap<String, ProbeEntry>,
}

impl ProbeInventory {
    /// Create empty inventory.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a probe.
    pub fn add(&mut self, id: &str, description: &str, command: &str, available: bool) {
        self.probes.insert(
            id.to_string(),
            ProbeEntry {
                id: id.to_string(),
                description: description.to_string(),
                command: command.to_string(),
                available,
            },
        );
    }

    /// Get probe by ID.
    pub fn get(&self, id: &str) -> Option<&ProbeEntry> {
        self.probes.get(id)
    }

    /// Get available probes.
    pub fn available(&self) -> Vec<&ProbeEntry> {
        self.probes.values().filter(|p| p.available).collect()
    }

    /// Get unavailable probes.
    pub fn unavailable(&self) -> Vec<&ProbeEntry> {
        self.probes.values().filter(|p| !p.available).collect()
    }

    /// Format for display in annactl debug probes.
    pub fn display(&self) -> String {
        let mut out = String::new();
        out.push_str("[probes]\n");

        let mut probes: Vec<_> = self.probes.values().collect();
        probes.sort_by(|a, b| a.id.cmp(&b.id));

        for p in probes {
            let status = if p.available { "✓" } else { "✗" };
            out.push_str(&format!(
                "  {} {} - {} ({})\n",
                status, p.id, p.description, p.command
            ));
        }

        let available = self.available().len();
        let total = self.probes.len();
        out.push_str(&format!("\n  {}/{} probes available\n", available, total));

        out
    }
}

/// Build default probe inventory with common probes.
pub fn default_probe_inventory() -> ProbeInventory {
    let mut inv = ProbeInventory::new();

    // System info probes
    inv.add("uname", "System information", "uname -a", true);
    inv.add("hostname", "Hostname", "hostname", true);
    inv.add("uptime", "System uptime", "uptime", true);

    // Disk probes
    inv.add("df", "Disk space usage", "df -h", true);
    inv.add("du_home", "Home directory size", "du -sh ~", true);
    inv.add("lsblk", "Block devices", "lsblk", true);

    // Memory probes
    inv.add("free", "Memory usage", "free -h", true);
    inv.add("meminfo", "Memory info", "cat /proc/meminfo", true);

    // Process probes
    inv.add("ps_aux", "Running processes", "ps aux", true);
    inv.add("top_snapshot", "Top processes", "top -bn1 | head -20", true);

    // Network probes
    inv.add("ip_addr", "IP addresses", "ip addr", true);
    inv.add("ss_listen", "Listening ports", "ss -tlnp", true);

    // Service probes
    inv.add(
        "systemctl_failed",
        "Failed services",
        "systemctl --failed --no-pager",
        true,
    );
    inv.add(
        "systemctl_list",
        "All services",
        "systemctl list-units --type=service --no-pager",
        true,
    );

    // Package probes
    inv.add("pacman_q", "Installed packages", "pacman -Q", true);
    inv.add("which", "Find command", "which", true);

    inv
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_inventory() {
        let mut inv = ModelInventory::new();
        inv.add_discovered("qwen2.5:7b", ModelOwner::User);
        inv.add_discovered("llama3.2:3b", ModelOwner::User);
        inv.add_anna_installed("gemma2:2b");

        assert_eq!(inv.discovered_count(), 3);
        assert_eq!(inv.anna_installed_count(), 1);
        assert_eq!(inv.user_installed_count(), 2);
    }

    #[test]
    fn test_model_normalization() {
        let mut inv = ModelInventory::new();
        inv.add_discovered("Qwen2.5:7b", ModelOwner::User);
        inv.add_discovered("qwen2.5:7b", ModelOwner::User); // Duplicate

        // Should deduplicate
        assert_eq!(inv.discovered_count(), 1);
    }

    #[test]
    fn test_configured_models() {
        let mut inv = ModelInventory::new();
        inv.add_discovered("qwen2.5:0.5b", ModelOwner::User);
        inv.add_discovered("qwen2.5:7b", ModelOwner::User);

        inv.set_configured(
            Some("qwen2.5:0.5b"),
            Some("qwen2.5:7b"),
            Some("qwen2.5:32b"), // Not present
        );

        let present = inv.configured_present();
        assert_eq!(present.len(), 2);

        let missing = inv.configured_missing();
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0].0, "senior");
    }

    #[test]
    fn test_probe_inventory() {
        let inv = default_probe_inventory();
        assert!(inv.probes.len() > 10);
        assert!(inv.get("df").is_some());
        assert!(inv.get("free").is_some());
    }

    #[test]
    fn test_model_display() {
        let mut inv = ModelInventory::new();
        inv.add_discovered("model1", ModelOwner::User);
        inv.add_discovered("model2", ModelOwner::Anna);
        inv.set_configured(Some("model1"), None, None);

        let display = inv.display(10);
        assert!(display.contains("model1"));
        assert!(display.contains("translator"));
    }
}
