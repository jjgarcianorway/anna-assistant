//! Audit Log for Anna v0.14.0 "Orion III"
//!
//! Complete auditability of all autonomous and advisory actions

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: u64,                // When action occurred
    pub actor: String,                 // "advisor", "user", "auto", "scheduler"
    pub action: String,                // Action taken or recommendation made
    pub action_type: String,           // "execute", "recommend", "revert", "ignore"
    pub impact: Option<String>,        // Expected or actual impact
    pub rollback_cmd: Option<String>,  // Command to undo this action
    pub result: String,                // "success", "fail", "pending", "ignored"
    pub details: Option<String>,       // Additional context
    pub related_action_id: Option<String>, // Link to action if applicable
}

impl AuditEntry {
    /// Create new audit entry
    pub fn new(actor: &str, action: &str, action_type: &str) -> Self {
        Self {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            actor: actor.to_string(),
            action: action.to_string(),
            action_type: action_type.to_string(),
            impact: None,
            rollback_cmd: None,
            result: "pending".to_string(),
            details: None,
            related_action_id: None,
        }
    }

    /// Builder: with impact
    pub fn with_impact(mut self, impact: String) -> Self {
        self.impact = Some(impact);
        self
    }

    /// Builder: with rollback command
    pub fn with_rollback(mut self, rollback_cmd: String) -> Self {
        self.rollback_cmd = Some(rollback_cmd);
        self
    }

    /// Builder: with result
    pub fn with_result(mut self, result: String) -> Self {
        self.result = result;
        self
    }

    /// Builder: with details
    pub fn with_details(mut self, details: String) -> Self {
        self.details = Some(details);
        self
    }

    /// Builder: with related action ID
    pub fn with_action_id(mut self, action_id: String) -> Self {
        self.related_action_id = Some(action_id);
        self
    }

    /// Get emoji for actor
    pub fn actor_emoji(&self) -> &'static str {
        match self.actor.as_str() {
            "auto" => "🤖",
            "user" => "👤",
            "advisor" => "🧠",
            "scheduler" => "⏰",
            _ => "📋",
        }
    }

    /// Get emoji for result
    pub fn result_emoji(&self) -> &'static str {
        match self.result.as_str() {
            "success" => "✅",
            "fail" => "❌",
            "pending" => "⏳",
            "ignored" => "🔇",
            _ => "❓",
        }
    }
}

/// Audit log manager
pub struct AuditLog {
    audit_path: PathBuf,
}

impl AuditLog {
    /// Create new audit log
    pub fn new() -> Result<Self> {
        let state_dir = Self::get_state_dir()?;
        fs::create_dir_all(&state_dir)?;

        let audit_path = state_dir.join("audit.jsonl");

        Ok(Self { audit_path })
    }

    /// Get state directory
    fn get_state_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME not set")?;
        Ok(PathBuf::from(home).join(".local/state/anna"))
    }

    /// Log an entry
    pub fn log(&self, entry: AuditEntry) -> Result<()> {
        let json = serde_json::to_string(&entry)?;
        let mut content = String::new();

        // Load existing log
        if self.audit_path.exists() {
            content = fs::read_to_string(&self.audit_path)?;
        }

        // Append new entry
        content.push_str(&json);
        content.push('\n');

        fs::write(&self.audit_path, content)?;

        Ok(())
    }

    /// Load all entries
    pub fn load_all(&self) -> Result<Vec<AuditEntry>> {
        if !self.audit_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.audit_path)?;
        let mut entries = Vec::new();

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<AuditEntry>(line) {
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    eprintln!("Warning: Failed to parse audit entry: {}", e);
                    continue;
                }
            }
        }

        Ok(entries)
    }

    /// Get last N entries
    pub fn get_last(&self, n: usize) -> Result<Vec<AuditEntry>> {
        let mut entries = self.load_all()?;

        if entries.len() > n {
            entries.drain(0..(entries.len() - n));
        }

        Ok(entries)
    }

    /// Get entries by actor
    pub fn get_by_actor(&self, actor: &str) -> Result<Vec<AuditEntry>> {
        let entries = self.load_all()?;
        Ok(entries
            .into_iter()
            .filter(|e| e.actor == actor)
            .collect())
    }

    /// Get entries by result
    pub fn get_by_result(&self, result: &str) -> Result<Vec<AuditEntry>> {
        let entries = self.load_all()?;
        Ok(entries
            .into_iter()
            .filter(|e| e.result == result)
            .collect())
    }

    /// Get summary statistics
    pub fn get_summary(&self) -> Result<AuditSummary> {
        let entries = self.load_all()?;

        let total = entries.len();
        let by_actor = Self::count_by_field(&entries, |e| &e.actor);
        let by_result = Self::count_by_field(&entries, |e| &e.result);
        let by_type = Self::count_by_field(&entries, |e| &e.action_type);

        Ok(AuditSummary {
            total_entries: total,
            by_actor,
            by_result,
            by_action_type: by_type,
        })
    }

    /// Count entries by field
    fn count_by_field<F>(entries: &[AuditEntry], field_fn: F) -> std::collections::HashMap<String, usize>
    where
        F: Fn(&AuditEntry) -> &String,
    {
        use std::collections::HashMap;

        let mut counts = HashMap::new();
        for entry in entries {
            let field_value = field_fn(entry);
            *counts.entry(field_value.clone()).or_insert(0) += 1;
        }
        counts
    }

    /// Export to file
    pub fn export(&self, path: &PathBuf) -> Result<()> {
        let entries = self.load_all()?;
        let json = serde_json::to_string_pretty(&entries)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Clear audit log (for testing)
    #[allow(dead_code)]
    pub fn clear(&self) -> Result<()> {
        if self.audit_path.exists() {
            fs::remove_file(&self.audit_path)?;
        }
        Ok(())
    }
}

/// Audit summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSummary {
    pub total_entries: usize,
    pub by_actor: std::collections::HashMap<String, usize>,
    pub by_result: std::collections::HashMap<String, usize>,
    pub by_action_type: std::collections::HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_entry_creation() {
        let entry = AuditEntry::new("user", "Clean cache", "execute")
            .with_impact("+2 Disk".to_string())
            .with_result("success".to_string());

        assert_eq!(entry.actor, "user");
        assert_eq!(entry.action, "Clean cache");
        assert_eq!(entry.result, "success");
        assert_eq!(entry.impact, Some("+2 Disk".to_string()));
    }

    #[test]
    fn test_audit_entry_emojis() {
        let entry = AuditEntry::new("auto", "Test", "execute");
        assert_eq!(entry.actor_emoji(), "🤖");

        let user_entry = AuditEntry::new("user", "Test", "execute");
        assert_eq!(user_entry.actor_emoji(), "👤");

        let success_entry = AuditEntry::new("user", "Test", "execute")
            .with_result("success".to_string());
        assert_eq!(success_entry.result_emoji(), "✅");

        let fail_entry = AuditEntry::new("user", "Test", "execute")
            .with_result("fail".to_string());
        assert_eq!(fail_entry.result_emoji(), "❌");
    }

    #[test]
    fn test_audit_entry_serialization() {
        let entry = AuditEntry::new("advisor", "Update packages", "recommend")
            .with_impact("+3 Software".to_string())
            .with_rollback("none".to_string())
            .with_result("pending".to_string())
            .with_details("Security updates available".to_string());

        let json = serde_json::to_string(&entry).unwrap();
        let parsed: AuditEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.actor, "advisor");
        assert_eq!(parsed.action, "Update packages");
        assert_eq!(parsed.action_type, "recommend");
        assert_eq!(parsed.impact, Some("+3 Software".to_string()));
    }

    #[test]
    fn test_summary_statistics() {
        let entries = vec![
            AuditEntry::new("auto", "Clean cache", "execute")
                .with_result("success".to_string()),
            AuditEntry::new("user", "Update packages", "execute")
                .with_result("success".to_string()),
            AuditEntry::new("advisor", "Check backups", "recommend")
                .with_result("ignored".to_string()),
            AuditEntry::new("auto", "Rotate logs", "execute")
                .with_result("fail".to_string()),
        ];

        // Simulate summary calculation
        let auto_count = entries.iter().filter(|e| e.actor == "auto").count();
        let success_count = entries.iter().filter(|e| e.result == "success").count();

        assert_eq!(auto_count, 2);
        assert_eq!(success_count, 2);
    }

    #[test]
    fn test_state_dir_path() {
        if let Ok(dir) = AuditLog::get_state_dir() {
            assert!(dir.to_string_lossy().contains(".local/state/anna"));
        }
    }
}
