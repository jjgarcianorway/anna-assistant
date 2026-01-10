//! Memory persistence - load, save, and recovery operations.

use anyhow::Result;
use std::path::PathBuf;

use super::types::{Memory, MemoryLoadResult};
use crate::config::anna_data_dir;

impl Memory {
    /// Load memory from disk
    pub fn load() -> Result<Self> {
        let path = memory_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let memory: Memory = serde_json::from_str(&content)?;
            Ok(memory)
        } else {
            Ok(Memory::default())
        }
    }

    /// Load memory with recovery on failure (v0.0.890)
    pub fn load_with_recovery() -> MemoryLoadResult {
        let path = memory_path();

        if !path.exists() {
            return MemoryLoadResult {
                memory: Memory::default(),
                was_recovered: false,
                error: None,
            };
        }

        match std::fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str::<Memory>(&content) {
                Ok(memory) => MemoryLoadResult {
                    memory,
                    was_recovered: false,
                    error: None,
                },
                Err(e) => {
                    let error_msg = format!("Memory corruption detected: {}", e);
                    let backup_path = memory_path().with_extension("json.corrupted");
                    let _ = std::fs::rename(&path, &backup_path);

                    let mut memory = Memory::default();
                    memory.stats.load_failures += 1;
                    memory.stats.last_error = Some(error_msg.clone());
                    memory.stats.recoveries += 1;

                    MemoryLoadResult {
                        memory,
                        was_recovered: true,
                        error: Some(error_msg),
                    }
                }
            },
            Err(e) => {
                let error_msg = format!("Memory file read error: {}", e);
                let mut memory = Memory::default();
                memory.stats.load_failures += 1;
                memory.stats.last_error = Some(error_msg.clone());

                MemoryLoadResult {
                    memory,
                    was_recovered: true,
                    error: Some(error_msg),
                }
            }
        }
    }

    /// Check memory health (v0.0.890)
    pub fn health_check(&self) -> Vec<String> {
        let mut issues = Vec::new();

        if self.stats.load_failures > 0 {
            issues.push(format!(
                "Memory has had {} load failures (last: {})",
                self.stats.load_failures,
                self.stats.last_error.as_deref().unwrap_or("unknown")
            ));
        }

        if self.experiences.len() > 800 {
            issues.push(format!(
                "Memory approaching capacity ({}/1000 experiences)",
                self.experiences.len()
            ));
        }

        if self.clusters.len() > 100 {
            issues.push(format!(
                "High cluster count ({}) may slow recall",
                self.clusters.len()
            ));
        }

        issues
    }

    /// Save memory to disk
    pub fn save(&self) -> Result<()> {
        let path = memory_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

/// Get memory storage path
pub fn memory_path() -> PathBuf {
    anna_data_dir().join("memory.json")
}
