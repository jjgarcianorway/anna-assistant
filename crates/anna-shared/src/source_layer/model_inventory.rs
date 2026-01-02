//! Model Inventory - v0.0.443.
//!
//! Clean model registry with no duplicates:
//! - Track installed_by (anna vs user)
//! - Track role (translator, junior, senior, disabled)
//! - Reconcile with ollama list

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::inventory_common::{normalize_model_name, InstalledBy};

/// Model role in Anna's pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelRole {
    /// Intent translation (fast, small).
    Translator,
    /// Junior specialist (medium).
    Junior,
    /// Senior specialist (large, slow).
    Senior,
    /// Not used by Anna.
    Disabled,
}

impl ModelRole {
    /// Get display label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Translator => "translator",
            Self::Junior => "junior",
            Self::Senior => "senior",
            Self::Disabled => "disabled",
        }
    }

    /// Is this role used in routing?
    pub fn is_active(&self) -> bool {
        !matches!(self, Self::Disabled)
    }
}

/// Model registry entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    /// Model name (e.g., "qwen2.5:7b-instruct").
    pub name: String,
    /// Provider (e.g., "ollama").
    pub provider: String,
    /// Whether installed.
    pub installed: bool,
    /// Who installed it.
    pub installed_by: InstalledBy,
    /// Installation timestamp (ISO 8601).
    pub installed_at: Option<String>,
    /// Size in GB.
    pub size_gb: f64,
    /// Last used timestamp.
    pub last_used_at: Option<String>,
    /// Role in Anna's pipeline.
    pub role: ModelRole,
    /// Whether enabled.
    pub enabled: bool,
}

impl ModelEntry {
    /// Create new entry.
    pub fn new(name: &str, provider: &str) -> Self {
        Self {
            name: name.to_string(),
            provider: provider.to_string(),
            installed: false,
            installed_by: InstalledBy::Unknown,
            installed_at: None,
            size_gb: 0.0,
            last_used_at: None,
            role: ModelRole::Disabled,
            enabled: false,
        }
    }

    /// Mark as installed by Anna.
    pub fn installed_by_anna(mut self) -> Self {
        self.installed = true;
        self.installed_by = InstalledBy::Anna;
        self.installed_at = Some(chrono::Utc::now().to_rfc3339());
        self
    }

    /// Mark as installed by user.
    pub fn installed_by_user(mut self) -> Self {
        self.installed = true;
        self.installed_by = InstalledBy::User;
        self
    }

    /// Set role.
    pub fn with_role(mut self, role: ModelRole) -> Self {
        self.role = role;
        self.enabled = role.is_active();
        self
    }

    /// Record usage.
    pub fn record_usage(&mut self) {
        self.last_used_at = Some(chrono::Utc::now().to_rfc3339());
    }
}

/// Model registry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelRegistry {
    /// Models by name.
    pub models: HashMap<String, ModelEntry>,
    /// Last reconciled timestamp.
    pub last_reconciled: Option<String>,
}

impl ModelRegistry {
    /// Registry file path.
    pub const FILE_PATH: &'static str = "/var/lib/anna/models/registry.json";

    /// Create empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add or update model.
    pub fn upsert(&mut self, entry: ModelEntry) {
        self.models.insert(entry.name.clone(), entry);
    }

    /// Get model by name.
    pub fn get(&self, name: &str) -> Option<&ModelEntry> {
        self.models.get(name)
    }

    /// Get mutable model.
    pub fn get_mut(&mut self, name: &str) -> Option<&mut ModelEntry> {
        self.models.get_mut(name)
    }

    /// Get models by role.
    pub fn by_role(&self, role: ModelRole) -> Vec<&ModelEntry> {
        self.models.values().filter(|m| m.role == role).collect()
    }

    /// Get enabled models.
    pub fn enabled(&self) -> Vec<&ModelEntry> {
        self.models.values().filter(|m| m.enabled).collect()
    }

    /// Get disabled models.
    pub fn disabled(&self) -> Vec<&ModelEntry> {
        self.models.values().filter(|m| !m.enabled).collect()
    }

    /// Get installed models.
    pub fn installed(&self) -> Vec<&ModelEntry> {
        self.models.values().filter(|m| m.installed).collect()
    }

    /// Get models installed by Anna.
    pub fn installed_by_anna(&self) -> Vec<&ModelEntry> {
        self.models
            .values()
            .filter(|m| m.installed_by == InstalledBy::Anna)
            .collect()
    }

    /// Remove duplicate entries (same name).
    pub fn deduplicate(&mut self) {
        // HashMap already handles uniqueness by key
        // But we should normalize names
        let mut normalized: HashMap<String, ModelEntry> = HashMap::new();

        for entry in self.models.values() {
            let norm_name = normalize_model_name(&entry.name);
            if let Some(existing) = normalized.get(&norm_name) {
                // Keep the one with more info
                if entry.installed && !existing.installed {
                    normalized.insert(norm_name, entry.clone());
                }
            } else {
                normalized.insert(norm_name, entry.clone());
            }
        }

        self.models = normalized;
    }

    /// Reconcile with ollama list output.
    pub fn reconcile_with_ollama(&mut self, ollama_models: &[OllamaModel]) {
        // Mark all as potentially not installed
        for entry in self.models.values_mut() {
            if entry.provider == "ollama" {
                entry.installed = false;
            }
        }

        // Update from ollama list
        for ollama in ollama_models {
            let norm_name = normalize_model_name(&ollama.name);

            if let Some(entry) = self.models.get_mut(&norm_name) {
                entry.installed = true;
                entry.size_gb = ollama.size_gb;
            } else {
                // New model not in registry (user installed)
                let entry = ModelEntry::new(&norm_name, "ollama")
                    .installed_by_user()
                    .with_role(ModelRole::Disabled);
                self.models.insert(norm_name, entry);
            }
        }

        self.last_reconciled = Some(chrono::Utc::now().to_rfc3339());
    }

    /// Get status summary.
    pub fn status_summary(&self) -> RegistrySummary {
        let enabled = self.enabled();
        let disabled = self.disabled();
        let by_anna = self.installed_by_anna();

        RegistrySummary {
            total: self.models.len(),
            enabled_count: enabled.len(),
            disabled_count: disabled.len(),
            installed_by_anna: by_anna.len(),
            translator: self
                .by_role(ModelRole::Translator)
                .first()
                .map(|m| m.name.clone()),
            junior: self
                .by_role(ModelRole::Junior)
                .first()
                .map(|m| m.name.clone()),
            senior: self
                .by_role(ModelRole::Senior)
                .first()
                .map(|m| m.name.clone()),
        }
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

/// Ollama model from `ollama list`.
#[derive(Debug, Clone)]
pub struct OllamaModel {
    /// Model name.
    pub name: String,
    /// Size in GB.
    pub size_gb: f64,
}

/// Registry summary for status display.
#[derive(Debug, Clone)]
pub struct RegistrySummary {
    /// Total models.
    pub total: usize,
    /// Enabled count.
    pub enabled_count: usize,
    /// Disabled count.
    pub disabled_count: usize,
    /// Installed by Anna.
    pub installed_by_anna: usize,
    /// Translator model name.
    pub translator: Option<String>,
    /// Junior model name.
    pub junior: Option<String>,
    /// Senior model name.
    pub senior: Option<String>,
}

impl RegistrySummary {
    /// Format for status display.
    pub fn display(&self) -> String {
        let mut output = String::new();

        output.push_str("Model Roles:\n");
        if let Some(ref t) = self.translator {
            output.push_str(&format!("  translator: {}\n", t));
        }
        if let Some(ref j) = self.junior {
            output.push_str(&format!("  junior: {}\n", j));
        }
        if let Some(ref s) = self.senior {
            output.push_str(&format!("  senior: {}\n", s));
        }

        output.push_str(&format!("\nEnabled models: {}\n", self.enabled_count));
        output.push_str(&format!("Disabled models: {}\n", self.disabled_count));
        output.push_str(&format!("Installed by Anna: {}\n", self.installed_by_anna));

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_entry() {
        let entry = ModelEntry::new("qwen2.5:7b", "ollama")
            .installed_by_anna()
            .with_role(ModelRole::Translator);

        assert!(entry.installed);
        assert_eq!(entry.installed_by, InstalledBy::Anna);
        assert_eq!(entry.role, ModelRole::Translator);
        assert!(entry.enabled);
    }

    #[test]
    fn test_model_registry() {
        let mut registry = ModelRegistry::new();

        registry.upsert(
            ModelEntry::new("qwen2.5:7b", "ollama")
                .installed_by_anna()
                .with_role(ModelRole::Translator),
        );
        registry.upsert(
            ModelEntry::new("llama3:8b", "ollama")
                .installed_by_user()
                .with_role(ModelRole::Junior),
        );

        assert_eq!(registry.enabled().len(), 2);
        assert_eq!(registry.installed_by_anna().len(), 1);
    }
}
