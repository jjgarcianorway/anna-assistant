//! Pain Module v0.42.0
//!
//! Tracks negative feedback events for self-improvement learning.
//! Pain events trigger remediation and influence future behavior.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Pain log directory
pub const PAIN_DIR: &str = "/var/lib/anna/knowledge/pain";

/// Type of pain event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PainType {
    /// XP penalty from low reliability
    XpPenalty,
    /// Skill trust decreased
    SkillTrustDrop,
    /// Pattern difficulty increased
    PatternStrike,
    /// User flagged answer as wrong
    UserFlagged,
    /// Remediation triggered
    RemediationTriggered,
    /// Junior plan rejected by Senior
    PlanRejected,
}

impl PainType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::XpPenalty => "XP Penalty",
            Self::SkillTrustDrop => "Skill Trust Drop",
            Self::PatternStrike => "Pattern Strike",
            Self::UserFlagged => "User Flagged",
            Self::RemediationTriggered => "Remediation",
            Self::PlanRejected => "Plan Rejected",
        }
    }
}

/// A single pain event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PainEvent {
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Type of pain
    pub pain_type: PainType,
    /// Severity (0.0 - 1.0)
    pub severity: f64,
    /// Reliability score that caused this
    pub reliability: f64,
    /// Related skill ID (if applicable)
    pub skill_id: Option<String>,
    /// Related pattern hash (if applicable)
    pub pattern_hash: Option<String>,
    /// Description of what happened
    pub description: String,
    /// XP lost (if applicable)
    pub xp_lost: u64,
}

impl PainEvent {
    /// Create XP penalty event
    pub fn xp_penalty(reliability: f64, xp_lost: u64) -> Self {
        Self {
            timestamp: Utc::now(),
            pain_type: PainType::XpPenalty,
            severity: (0.40 - reliability).max(0.0) / 0.40,
            reliability,
            skill_id: None,
            pattern_hash: None,
            description: format!("R={:.2} below 0.40 threshold", reliability),
            xp_lost,
        }
    }

    /// Create skill trust drop event
    pub fn skill_trust_drop(skill_id: &str, reliability: f64, old_trust: u8, new_trust: u8) -> Self {
        Self {
            timestamp: Utc::now(),
            pain_type: PainType::SkillTrustDrop,
            severity: (old_trust - new_trust) as f64 / 100.0,
            reliability,
            skill_id: Some(skill_id.to_string()),
            pattern_hash: None,
            description: format!("Trust: {} -> {}", old_trust, new_trust),
            xp_lost: 0,
        }
    }

    /// Create pattern strike event
    pub fn pattern_strike(pattern_hash: &str, reliability: f64, strike_count: u32) -> Self {
        Self {
            timestamp: Utc::now(),
            pain_type: PainType::PatternStrike,
            severity: (strike_count as f64 / 10.0).min(1.0),
            reliability,
            skill_id: None,
            pattern_hash: Some(pattern_hash.to_string()),
            description: format!("Strike #{} (R={:.2})", strike_count, reliability),
            xp_lost: 0,
        }
    }

    /// Create user flagged event
    pub fn user_flagged(pattern_hash: &str, xp_lost: u64) -> Self {
        Self {
            timestamp: Utc::now(),
            pain_type: PainType::UserFlagged,
            severity: 1.0,
            reliability: 0.0,
            skill_id: None,
            pattern_hash: Some(pattern_hash.to_string()),
            description: "User flagged answer as wrong".to_string(),
            xp_lost,
        }
    }

    /// Create plan rejected event
    pub fn plan_rejected(reason: &str) -> Self {
        Self {
            timestamp: Utc::now(),
            pain_type: PainType::PlanRejected,
            severity: 0.3,
            reliability: 0.0,
            skill_id: None,
            pattern_hash: None,
            description: reason.to_string(),
            xp_lost: 0,
        }
    }
}

/// Pain log - append-only storage for pain events
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PainLog {
    /// All pain events (append-only)
    pub events: Vec<PainEvent>,
    /// Total XP lost
    pub total_xp_lost: u64,
    /// Total events count
    pub event_count: u64,
}

impl PainLog {
    /// Create new pain log
    pub fn new() -> Self {
        Self::default()
    }

    /// Load from default location
    pub fn load_default() -> anyhow::Result<Self> {
        Self::load(&Self::default_path())
    }

    /// Load from file
    pub fn load(path: &PathBuf) -> anyhow::Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let content = fs::read_to_string(path)?;
        let log: PainLog = serde_json::from_str(&content)?;
        Ok(log)
    }

    /// Save to default location
    pub fn save_default(&self) -> anyhow::Result<()> {
        self.save(&Self::default_path())
    }

    /// Save to file
    pub fn save(&self, path: &PathBuf) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Default path for pain log
    pub fn default_path() -> PathBuf {
        PathBuf::from(PAIN_DIR).join("pain_log.json")
    }

    /// Record a pain event
    pub fn record(&mut self, event: PainEvent) {
        self.total_xp_lost += event.xp_lost;
        self.event_count += 1;
        self.events.push(event);
    }

    /// Get last N events
    pub fn last_events(&self, n: usize) -> Vec<&PainEvent> {
        self.events.iter().rev().take(n).collect()
    }

    /// Get events by type
    pub fn events_by_type(&self, pain_type: PainType) -> Vec<&PainEvent> {
        self.events
            .iter()
            .filter(|e| e.pain_type == pain_type)
            .collect()
    }

    /// Count events in last N hours
    pub fn events_in_hours(&self, hours: i64) -> usize {
        let cutoff = Utc::now() - chrono::Duration::hours(hours);
        self.events
            .iter()
            .filter(|e| e.timestamp > cutoff)
            .count()
    }

    /// Get skill IDs with most pain
    pub fn most_painful_skills(&self, limit: usize) -> Vec<(String, usize)> {
        use std::collections::HashMap;
        let mut counts: HashMap<String, usize> = HashMap::new();
        for event in &self.events {
            if let Some(ref skill_id) = event.skill_id {
                *counts.entry(skill_id.clone()).or_default() += 1;
            }
        }
        let mut sorted: Vec<_> = counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.into_iter().take(limit).collect()
    }

    /// Get pattern hashes with most pain
    pub fn most_painful_patterns(&self, limit: usize) -> Vec<(String, usize)> {
        use std::collections::HashMap;
        let mut counts: HashMap<String, usize> = HashMap::new();
        for event in &self.events {
            if let Some(ref hash) = event.pattern_hash {
                *counts.entry(hash.clone()).or_default() += 1;
            }
        }
        let mut sorted: Vec<_> = counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.into_iter().take(limit).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pain_event_creation() {
        let event = PainEvent::xp_penalty(0.30, 25);
        assert_eq!(event.pain_type, PainType::XpPenalty);
        assert_eq!(event.xp_lost, 25);
        assert!(event.severity > 0.0);
    }

    #[test]
    fn test_skill_trust_drop() {
        let event = PainEvent::skill_trust_drop("test.skill", 0.45, 60, 50);
        assert_eq!(event.pain_type, PainType::SkillTrustDrop);
        assert_eq!(event.skill_id, Some("test.skill".to_string()));
        assert!((event.severity - 0.10).abs() < 0.01);
    }

    #[test]
    fn test_pattern_strike() {
        let event = PainEvent::pattern_strike("abc123", 0.35, 3);
        assert_eq!(event.pain_type, PainType::PatternStrike);
        assert_eq!(event.pattern_hash, Some("abc123".to_string()));
    }

    #[test]
    fn test_pain_log_record() {
        let mut log = PainLog::new();
        assert_eq!(log.event_count, 0);

        log.record(PainEvent::xp_penalty(0.30, 25));
        log.record(PainEvent::xp_penalty(0.20, 50));

        assert_eq!(log.event_count, 2);
        assert_eq!(log.total_xp_lost, 75);
    }

    #[test]
    fn test_last_events() {
        let mut log = PainLog::new();
        log.record(PainEvent::xp_penalty(0.30, 10));
        log.record(PainEvent::xp_penalty(0.25, 20));
        log.record(PainEvent::xp_penalty(0.20, 30));

        let last = log.last_events(2);
        assert_eq!(last.len(), 2);
        assert_eq!(last[0].xp_lost, 30); // Most recent first
        assert_eq!(last[1].xp_lost, 20);
    }

    #[test]
    fn test_events_by_type() {
        let mut log = PainLog::new();
        log.record(PainEvent::xp_penalty(0.30, 10));
        log.record(PainEvent::pattern_strike("abc", 0.35, 1));
        log.record(PainEvent::xp_penalty(0.25, 20));

        let xp_events = log.events_by_type(PainType::XpPenalty);
        assert_eq!(xp_events.len(), 2);
    }

    #[test]
    fn test_most_painful_skills() {
        let mut log = PainLog::new();
        log.record(PainEvent::skill_trust_drop("skill.a", 0.4, 60, 50));
        log.record(PainEvent::skill_trust_drop("skill.b", 0.4, 60, 50));
        log.record(PainEvent::skill_trust_drop("skill.a", 0.3, 50, 40));
        log.record(PainEvent::skill_trust_drop("skill.a", 0.2, 40, 30));

        let painful = log.most_painful_skills(2);
        assert_eq!(painful[0].0, "skill.a");
        assert_eq!(painful[0].1, 3);
        assert_eq!(painful[1].0, "skill.b");
        assert_eq!(painful[1].1, 1);
    }

    #[test]
    fn test_serialization() {
        let mut log = PainLog::new();
        log.record(PainEvent::xp_penalty(0.30, 25));

        let json = serde_json::to_string(&log).unwrap();
        let loaded: PainLog = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.event_count, 1);
        assert_eq!(loaded.total_xp_lost, 25);
    }
}
