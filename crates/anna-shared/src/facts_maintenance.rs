//! Facts Maintenance (v0.0.471).
//!
//! Scheduled maintenance for the facts system:
//! - Automatic lifecycle transitions (Active -> Stale -> Archived)
//! - Pruning of archived facts
//! - Statistics and health reporting
//!
//! Per VISION.md: "Update or delete facts when not relevant, archive old facts"

use crate::facts::{FactKey, FactLifecycle, FactsStore};
use serde::{Deserialize, Serialize};

/// Result of a maintenance run
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MaintenanceResult {
    /// Facts transitioned to stale
    pub newly_stale: usize,
    /// Facts transitioned to archived
    pub newly_archived: usize,
    /// Facts pruned (removed)
    pub pruned: usize,
    /// Total active facts
    pub active_count: usize,
    /// Total stale facts
    pub stale_count: usize,
    /// Timestamp of maintenance
    pub timestamp: u64,
}

impl MaintenanceResult {
    /// Check if any work was done
    pub fn had_changes(&self) -> bool {
        self.newly_stale > 0 || self.newly_archived > 0 || self.pruned > 0
    }

    /// Format as summary string
    pub fn summary(&self) -> String {
        if !self.had_changes() {
            format!(
                "Facts healthy: {} active, {} stale",
                self.active_count, self.stale_count
            )
        } else {
            let mut parts = Vec::new();
            if self.newly_stale > 0 {
                parts.push(format!("{} marked stale", self.newly_stale));
            }
            if self.newly_archived > 0 {
                parts.push(format!("{} archived", self.newly_archived));
            }
            if self.pruned > 0 {
                parts.push(format!("{} pruned", self.pruned));
            }
            format!(
                "Maintenance: {} ({} active, {} stale)",
                parts.join(", "),
                self.active_count,
                self.stale_count
            )
        }
    }
}

/// Facts health statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FactsHealth {
    /// Total facts in store
    pub total: usize,
    /// Active (usable) facts
    pub active: usize,
    /// Stale facts needing re-verification
    pub stale: usize,
    /// Archived facts pending removal
    pub archived: usize,
    /// Oldest fact age in seconds
    pub oldest_age_secs: u64,
    /// Average fact age in seconds
    pub avg_age_secs: u64,
    /// Facts by category
    pub by_category: Vec<(String, usize)>,
}

impl FactsHealth {
    /// Check if store is healthy (not too many stale facts)
    pub fn is_healthy(&self) -> bool {
        if self.total == 0 {
            return true;
        }
        let stale_ratio = self.stale as f64 / self.total as f64;
        stale_ratio < 0.3 // Less than 30% stale is healthy
    }

    /// Format as display string
    pub fn display(&self) -> String {
        let mut lines = vec![
            format!("total             {}", self.total),
            format!("active            {}", self.active),
            format!("stale             {}", self.stale),
            format!("archived          {}", self.archived),
        ];

        if self.oldest_age_secs > 0 {
            lines.push(format!("oldest            {}", format_age(self.oldest_age_secs)));
        }
        if self.avg_age_secs > 0 {
            lines.push(format!("average_age       {}", format_age(self.avg_age_secs)));
        }

        if !self.by_category.is_empty() {
            lines.push(String::new());
            lines.push("by_category:".to_string());
            for (cat, count) in &self.by_category {
                lines.push(format!("  {}  {}", cat, count));
            }
        }

        lines.join("\n")
    }
}

/// Run maintenance on facts store
pub fn run_maintenance(store: &mut FactsStore, prune: bool) -> MaintenanceResult {
    let now = now_timestamp();

    // Count states before
    let stale_before: usize = store
        .verified_facts()
        .iter()
        .filter(|f| f.lifecycle == FactLifecycle::Stale)
        .count();
    let archived_before: usize = store
        .verified_facts()
        .iter()
        .filter(|f| f.lifecycle == FactLifecycle::Archived)
        .count();

    // Apply lifecycle transitions
    store.apply_lifecycle(now);

    // Count states after
    let stale_after: usize = store
        .verified_facts()
        .iter()
        .filter(|f| f.lifecycle == FactLifecycle::Stale)
        .count();
    let archived_after: usize = store
        .verified_facts()
        .iter()
        .filter(|f| f.lifecycle == FactLifecycle::Archived)
        .count();

    // Prune if requested
    let pruned = if prune {
        store.prune_archived()
    } else {
        0
    };

    let active_count = store.verified_count();

    MaintenanceResult {
        newly_stale: stale_after.saturating_sub(stale_before),
        newly_archived: archived_after.saturating_sub(archived_before),
        pruned,
        active_count,
        stale_count: store.stale_facts().len(),
        timestamp: now,
    }
}

/// Get health statistics for facts store
pub fn get_health(store: &FactsStore) -> FactsHealth {
    let now = now_timestamp();
    let facts: Vec<_> = store.verified_facts();

    let mut active = 0;
    let mut stale = 0;
    let mut archived = 0;
    let mut total_age: u64 = 0;
    let mut oldest_age: u64 = 0;
    let mut categories: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for fact in &facts {
        match fact.lifecycle {
            FactLifecycle::Active => active += 1,
            FactLifecycle::Stale => stale += 1,
            FactLifecycle::Archived => archived += 1,
        }

        let age = now.saturating_sub(fact.last_verified_at);
        total_age += age;
        if age > oldest_age {
            oldest_age = age;
        }

        // Categorize by key prefix
        let category = categorize_key(&fact.key);
        *categories.entry(category).or_insert(0) += 1;
    }

    let avg_age = if facts.is_empty() {
        0
    } else {
        total_age / facts.len() as u64
    };

    let mut by_category: Vec<_> = categories.into_iter().collect();
    by_category.sort_by(|a, b| b.1.cmp(&a.1));

    FactsHealth {
        total: facts.len(),
        active,
        stale,
        archived,
        oldest_age_secs: oldest_age,
        avg_age_secs: avg_age,
        by_category,
    }
}

/// Get facts that need re-verification
pub fn get_reverification_candidates(store: &FactsStore) -> Vec<&FactKey> {
    store.stale_facts().iter().map(|f| &f.key).collect()
}

/// Check if maintenance is needed (has stale or archived facts)
pub fn needs_maintenance(store: &FactsStore) -> bool {
    !store.stale_facts().is_empty() || store.verified_facts().iter().any(|f| f.lifecycle == FactLifecycle::Archived)
}

/// Categorize a fact key for statistics
fn categorize_key(key: &FactKey) -> String {
    match key {
        FactKey::PreferredEditor | FactKey::EditorInstalled(_) => "editor".to_string(),
        FactKey::NetworkPrimaryInterface | FactKey::NetworkPreference => "network".to_string(),
        FactKey::InstalledPackage(_) => "packages".to_string(),
        FactKey::BinaryAvailable(_) => "binaries".to_string(),
        FactKey::InitSystem | FactKey::PackageManager | FactKey::Hostname | FactKey::Kernel => {
            "system".to_string()
        }
        FactKey::Desktop | FactKey::GpuPresent => "hardware".to_string(),
        FactKey::BootTimeBaseline => "performance".to_string(),
        FactKey::UnitExists(_) | FactKey::MountExists(_) => "services".to_string(),
        _ => "other".to_string(),
    }
}

/// Format age in human-readable form
fn format_age(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else if secs < 86400 {
        format!("{}h", secs / 3600)
    } else {
        format!("{}d", secs / 86400)
    }
}

fn now_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maintenance_result_summary() {
        let result = MaintenanceResult {
            newly_stale: 2,
            newly_archived: 1,
            pruned: 0,
            active_count: 10,
            stale_count: 3,
            timestamp: 0,
        };
        let summary = result.summary();
        assert!(summary.contains("marked stale"));
        assert!(summary.contains("archived"));
    }

    #[test]
    fn test_maintenance_no_changes() {
        let result = MaintenanceResult::default();
        assert!(!result.had_changes());
    }

    #[test]
    fn test_facts_health_display() {
        let health = FactsHealth {
            total: 10,
            active: 8,
            stale: 2,
            archived: 0,
            oldest_age_secs: 86400 * 7,
            avg_age_secs: 86400 * 3,
            by_category: vec![("system".to_string(), 5), ("network".to_string(), 3)],
        };
        let output = health.display();
        assert!(output.contains("total"));
        assert!(output.contains("active"));
        assert!(output.contains("system"));
    }

    #[test]
    fn test_health_is_healthy() {
        let healthy = FactsHealth {
            total: 10,
            active: 8,
            stale: 2,
            ..Default::default()
        };
        assert!(healthy.is_healthy());

        let unhealthy = FactsHealth {
            total: 10,
            active: 3,
            stale: 7,
            ..Default::default()
        };
        assert!(!unhealthy.is_healthy());
    }

    #[test]
    fn test_format_age() {
        assert_eq!(format_age(30), "30s");
        assert_eq!(format_age(120), "2m");
        assert_eq!(format_age(7200), "2h");
        assert_eq!(format_age(172800), "2d");
    }

    #[test]
    fn test_empty_store_maintenance() {
        let mut store = FactsStore::new();
        let result = run_maintenance(&mut store, true);
        assert!(!result.had_changes());
        assert_eq!(result.active_count, 0);
    }
}
