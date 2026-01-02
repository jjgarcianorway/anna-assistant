//! Monitor storage (v0.0.430).
//!
//! Persistence layer for monitors.

use super::types::Monitor;
use std::fs;
use std::path::PathBuf;

/// Monitor storage
pub struct MonitorStorage {
    path: PathBuf,
}

impl MonitorStorage {
    pub fn new(base_path: &str) -> Self {
        Self {
            path: PathBuf::from(base_path),
        }
    }

    fn monitors_file(&self) -> PathBuf {
        self.path.join(crate::background_worker::MONITORS_FILE)
    }

    pub fn load_monitors(&self) -> Result<Vec<Monitor>, String> {
        let file = self.monitors_file();
        if !file.exists() {
            return Ok(Vec::new());
        }
        let content = fs::read_to_string(&file).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    }

    pub fn save_monitors(&self, monitors: &[Monitor]) -> Result<(), String> {
        fs::create_dir_all(&self.path).map_err(|e| e.to_string())?;
        let content = serde_json::to_string_pretty(monitors).map_err(|e| e.to_string())?;
        fs::write(self.monitors_file(), content).map_err(|e| e.to_string())
    }
}
