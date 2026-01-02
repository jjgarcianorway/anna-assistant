//! Persistent storage for user alarms.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use super::types::UserAlarm;

/// Storage for user alarms
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AlarmStore {
    pub alarms: HashMap<String, UserAlarm>,
}

impl AlarmStore {
    /// Load from disk
    pub fn load() -> Self {
        let path = Self::store_path();
        if !path.exists() {
            return Self::default();
        }

        match fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
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

    /// Add an alarm
    pub fn add(&mut self, alarm: UserAlarm) {
        self.alarms.insert(alarm.id.clone(), alarm);
    }

    /// Remove an alarm
    pub fn remove(&mut self, id: &str) -> Option<UserAlarm> {
        self.alarms.remove(id)
    }

    /// Get alarm by ID
    pub fn get(&self, id: &str) -> Option<&UserAlarm> {
        self.alarms.get(id)
    }

    /// Get mutable alarm by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut UserAlarm> {
        self.alarms.get_mut(id)
    }

    /// List all alarms
    pub fn list(&self) -> Vec<&UserAlarm> {
        self.alarms.values().collect()
    }

    /// Get alarms that should trigger now
    pub fn due_alarms(&self, now_ts: u64) -> Vec<&UserAlarm> {
        self.alarms
            .values()
            .filter(|a| a.should_trigger(now_ts))
            .collect()
    }

    fn store_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".anna").join("alarms.json")
    }
}
