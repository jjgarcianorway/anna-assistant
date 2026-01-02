//! File I/O operations for recipe library and evidence cache.

use super::RecipeLibrary;
use crate::learning_engine::EvidenceCache;
use std::path::PathBuf;

impl RecipeLibrary {
    /// Load library from file
    pub fn load(path: &PathBuf) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::new());
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read recipe library: {}", e))?;

        let mut library: Self = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse recipe library: {}", e))?;

        library.rebuild_indexes();
        Ok(library)
    }

    /// Save library to file
    pub fn save(&self, path: &PathBuf) -> Result<(), String> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize library: {}", e))?;

        std::fs::write(path, content).map_err(|e| format!("Failed to write library: {}", e))?;

        Ok(())
    }

    /// Get default library path
    pub fn default_path() -> PathBuf {
        let state_dir =
            std::env::var("ANNA_STATE_DIR").unwrap_or_else(|_| "/var/lib/anna".to_string());
        PathBuf::from(state_dir).join("recipes.json")
    }

    /// Get default evidence cache path
    pub fn evidence_cache_path() -> PathBuf {
        let state_dir =
            std::env::var("ANNA_STATE_DIR").unwrap_or_else(|_| "/var/lib/anna".to_string());
        PathBuf::from(state_dir).join("evidence_cache.json")
    }

    /// Load evidence cache from file
    pub fn load_evidence_cache(&mut self) -> Result<(), String> {
        let path = Self::evidence_cache_path();
        if !path.exists() {
            self.set_evidence_cache(EvidenceCache::default());
            return Ok(());
        }

        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read evidence cache: {}", e))?;

        let cache: EvidenceCache = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse evidence cache: {}", e))?;

        self.set_evidence_cache(cache);
        Ok(())
    }

    /// Save evidence cache to file
    pub fn save_evidence_cache(&self) -> Result<(), String> {
        let Some(cache) = self.evidence_cache_ref() else {
            return Ok(());
        };

        let path = Self::evidence_cache_path();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        let content = serde_json::to_string_pretty(cache)
            .map_err(|e| format!("Failed to serialize evidence cache: {}", e))?;

        std::fs::write(path, content)
            .map_err(|e| format!("Failed to write evidence cache: {}", e))?;

        Ok(())
    }
}
