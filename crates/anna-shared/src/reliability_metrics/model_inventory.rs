//! Accurate model and probe inventory (v0.0.444).
//!
//! Fixes:
//! - No more "and 22 more" with duplicates
//! - Track model ownership (user vs anna installed)
//! - Clean probe inventory with commands

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// Re-export types from submodules
pub use super::model_inventory_probes::{
    default_probe_inventory, ProbeEntry, ProbeInventory,
};
pub use super::model_inventory_types::{
    normalize_model_name, ConfiguredModels, ModelEntry, ModelOwner,
};

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
