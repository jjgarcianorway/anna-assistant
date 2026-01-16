//! Competence Record - Silent learning from resolutions.
//!
//! This record is NOT user-facing. It tracks:
//! - Issue types encountered
//! - Resolutions observed
//! - Who resolved them (Anna/User/Unknown)
//!
//! This data is used internally to:
//! - Reduce escalation for issues the user has resolved before
//! - Track patterns over time
//! - Eventually inform teaching (but not yet)
//!
//! NO GAMIFICATION. NO REWARDS. NO USER VISIBILITY.

use super::attribution::Actor;
use super::recognition::Resolution;
use crate::monitor::IssueType;
use crate::paths::paths;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// A single competence entry - one observed resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetenceEntry {
    /// The issue type that was resolved
    pub issue_type: IssueType,
    /// How it was resolved
    pub resolution_observed: Resolution,
    /// Who resolved it
    pub actor: Actor,
    /// When this was recorded
    pub timestamp: DateTime<Utc>,
}

/// Aggregated competence for a specific issue type.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IssueCompetence {
    /// Total times this issue type was encountered
    pub encounters: u32,
    /// Times resolved by Anna
    pub resolved_by_anna: u32,
    /// Times resolved by User
    pub resolved_by_user: u32,
    /// Times resolution actor unknown
    pub resolved_unknown: u32,
    /// Last encounter timestamp
    pub last_seen: Option<DateTime<Utc>>,
}

/// The complete competence record - internal, not user-facing.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompetenceRecord {
    /// Version for future migrations
    pub version: u32,
    /// Individual entries (recent history)
    pub entries: Vec<CompetenceEntry>,
    /// Aggregated competence by issue type
    pub by_issue_type: HashMap<String, IssueCompetence>,
    /// Last update time
    pub last_updated: Option<DateTime<Utc>>,
}

impl CompetenceRecord {
    /// Record a new competence entry.
    pub fn record(&mut self, entry: CompetenceEntry) {
        // Update aggregated stats
        let key = format!("{:?}", entry.issue_type);
        let competence = self.by_issue_type.entry(key).or_default();

        competence.encounters += 1;
        competence.last_seen = Some(entry.timestamp);

        match entry.actor {
            Actor::Anna => competence.resolved_by_anna += 1,
            Actor::User => competence.resolved_by_user += 1,
            Actor::Unknown => competence.resolved_unknown += 1,
        }

        // Keep recent entries (limit to 100)
        self.entries.push(entry);
        if self.entries.len() > 100 {
            self.entries.remove(0);
        }

        self.last_updated = Some(Utc::now());
    }

    /// Get competence level for an issue type.
    /// Returns a value from 0.0 (no history) to 1.0 (frequently resolved by user).
    ///
    /// This is used internally to potentially reduce escalation in the future.
    /// Higher values mean the user has demonstrated competence with this issue type.
    pub fn user_competence_for(&self, issue_type: &IssueType) -> f32 {
        let key = format!("{:?}", issue_type);

        if let Some(competence) = self.by_issue_type.get(&key) {
            if competence.encounters == 0 {
                return 0.0;
            }

            // Competence = ratio of user resolutions to total encounters
            // Weighted slightly toward recent activity (not implemented yet)
            let user_ratio = competence.resolved_by_user as f32 / competence.encounters as f32;

            // Cap at 1.0
            user_ratio.min(1.0)
        } else {
            0.0
        }
    }

    /// Check if user has demonstrated competence with an issue type.
    /// Returns true if user has resolved this type at least twice.
    pub fn user_has_competence(&self, issue_type: &IssueType) -> bool {
        let key = format!("{:?}", issue_type);

        if let Some(competence) = self.by_issue_type.get(&key) {
            competence.resolved_by_user >= 2
        } else {
            false
        }
    }
}

/// Path to competence record file.
fn competence_path() -> PathBuf {
    paths().data_dir.join("competence.json")
}

/// Load competence record from disk.
pub fn load_competence() -> Result<CompetenceRecord> {
    let path = competence_path();

    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        let record: CompetenceRecord = serde_json::from_str(&content)?;
        Ok(record)
    } else {
        Ok(CompetenceRecord {
            version: 1,
            ..Default::default()
        })
    }
}

/// Save competence record to disk.
pub fn save_competence(record: &CompetenceRecord) -> Result<()> {
    let path = competence_path();

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(record)?;
    std::fs::write(&path, content)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_competence() {
        let record = CompetenceRecord::default();
        assert_eq!(record.user_competence_for(&IssueType::ConfigChanged), 0.0);
        assert!(!record.user_has_competence(&IssueType::ConfigChanged));
    }

    #[test]
    fn test_record_entry() {
        let mut record = CompetenceRecord::default();

        record.record(CompetenceEntry {
            issue_type: IssueType::ConfigChanged,
            resolution_observed: Resolution::IssueCleared,
            actor: Actor::User,
            timestamp: Utc::now(),
        });

        let key = format!("{:?}", IssueType::ConfigChanged);
        let competence = record.by_issue_type.get(&key).unwrap();

        assert_eq!(competence.encounters, 1);
        assert_eq!(competence.resolved_by_user, 1);
        assert_eq!(competence.resolved_by_anna, 0);
    }

    #[test]
    fn test_user_competence_calculation() {
        let mut record = CompetenceRecord::default();

        // User resolves twice, Anna once
        record.record(CompetenceEntry {
            issue_type: IssueType::ServiceFailed,
            resolution_observed: Resolution::IssueCleared,
            actor: Actor::User,
            timestamp: Utc::now(),
        });
        record.record(CompetenceEntry {
            issue_type: IssueType::ServiceFailed,
            resolution_observed: Resolution::IssueCleared,
            actor: Actor::User,
            timestamp: Utc::now(),
        });
        record.record(CompetenceEntry {
            issue_type: IssueType::ServiceFailed,
            resolution_observed: Resolution::IssueCleared,
            actor: Actor::Anna,
            timestamp: Utc::now(),
        });

        // 2/3 = 0.666...
        let competence = record.user_competence_for(&IssueType::ServiceFailed);
        assert!(competence > 0.6 && competence < 0.7);

        // User has competence (2+ resolutions)
        assert!(record.user_has_competence(&IssueType::ServiceFailed));
    }

    #[test]
    fn test_entry_limit() {
        let mut record = CompetenceRecord::default();

        // Add 150 entries
        for _ in 0..150 {
            record.record(CompetenceEntry {
                issue_type: IssueType::ConfigChanged,
                resolution_observed: Resolution::IssueCleared,
                actor: Actor::Unknown,
                timestamp: Utc::now(),
            });
        }

        // Should be capped at 100
        assert_eq!(record.entries.len(), 100);
    }
}
