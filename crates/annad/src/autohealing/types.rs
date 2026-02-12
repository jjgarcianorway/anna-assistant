//! Auto-healing data types.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const HEALING_LOG: &str = "/var/lib/anna/healing.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealingAction {
    pub timestamp: String,
    pub issue: String,
    pub action: String,
    pub result: HealingResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealingResult {
    Success(String),
    Failed(String),
    Skipped(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealingLog {
    pub actions: Vec<HealingAction>,
}

impl Default for HealingLog {
    fn default() -> Self {
        Self {
            actions: Vec::new(),
        }
    }
}

impl HealingLog {
    /// Load healing log
    pub fn load() -> Self {
        let path = PathBuf::from(HEALING_LOG);
        if !path.exists() {
            return Self::default();
        }

        match std::fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save healing log
    pub fn save(&self) -> Result<()> {
        let path = PathBuf::from(HEALING_LOG);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Record a healing action
    pub fn record(&mut self, issue: String, action: String, result: HealingResult) {
        self.actions.push(HealingAction {
            timestamp: chrono::Utc::now().to_rfc3339(),
            issue,
            action,
            result,
        });

        // Keep last 100 actions
        if self.actions.len() > 100 {
            self.actions.drain(0..self.actions.len() - 100);
        }
    }
}
