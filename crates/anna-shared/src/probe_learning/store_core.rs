//! Core storage operations for probe learning.
//! Handles file I/O, persistence, and basic store management.

use std::fs;
use std::path::PathBuf;

use super::store::ProbeLearningStore;

impl ProbeLearningStore {
    /// Load from disk or create new
    pub fn load() -> Self {
        let path = Self::store_path();
        if let Ok(content) = fs::read_to_string(&path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Save to disk
    pub fn save(&self) -> Result<(), String> {
        let path = Self::store_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&path, content).map_err(|e| e.to_string())
    }

    /// Store path
    pub fn store_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join(".anna")
            .join("probe_learning.json")
    }

    /// Reset all learning data
    pub fn reset() -> Result<(), String> {
        let path = Self::store_path();
        if path.exists() {
            fs::remove_file(&path).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// Apply decay if needed on load
    pub fn load_with_decay() -> Self {
        let mut store = Self::load();
        let result = store.apply_decay();
        if result.applied {
            let _ = store.save();
        }
        store
    }

    /// Get summary stats for display
    pub fn summary(&self) -> String {
        let total_categories = self.effectiveness.len();
        let total_probes: usize = self.effectiveness.values().map(|m| m.len()).sum();
        let total_uses: u32 = self
            .effectiveness
            .values()
            .flat_map(|m| m.values())
            .map(|e| e.uses)
            .sum();
        let negative_patterns = self.negative_patterns.len();

        format!(
            "{} categories, {} probes tracked, {} uses, {} negative patterns",
            total_categories, total_probes, total_uses, negative_patterns
        )
    }
}
