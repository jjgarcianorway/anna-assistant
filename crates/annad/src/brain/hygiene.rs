//! Knowledge Hygiene v0.11.0
//!
//! Detects stale, conflicting, or outdated facts and schedules cleanup.

use anna_common::{Fact, FactQuery, FactStatus, KnowledgeStore};
use chrono::{Duration, Utc};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Knowledge hygiene manager
pub struct KnowledgeHygiene {
    store: Arc<RwLock<KnowledgeStore>>,
}

/// Hygiene report
#[derive(Debug, Default)]
pub struct HygieneReport {
    /// Number of stale facts marked
    pub stale_count: usize,
    /// Number of conflicts detected
    pub conflict_count: usize,
    /// Number of facts revalidated
    pub revalidated_count: usize,
    /// Number of facts cleaned up
    pub cleanup_count: usize,
}

impl KnowledgeHygiene {
    pub fn new(store: Arc<RwLock<KnowledgeStore>>) -> Self {
        Self { store }
    }

    /// Run full hygiene check
    pub async fn run(&self) -> anyhow::Result<HygieneReport> {
        info!("Running knowledge hygiene check");
        let mut report = HygieneReport::default();

        // Mark stale facts
        report.stale_count = self.mark_stale_facts().await?;

        // Detect conflicts
        report.conflict_count = self.detect_conflicts().await?;

        // Clean up deprecated facts
        report.cleanup_count = self.cleanup_deprecated().await?;

        info!(
            "Hygiene complete: {} stale, {} conflicts, {} cleaned",
            report.stale_count, report.conflict_count, report.cleanup_count
        );

        Ok(report)
    }

    /// Mark facts as stale based on age
    async fn mark_stale_facts(&self) -> anyhow::Result<usize> {
        let store = self.store.write().await;

        // Different staleness thresholds by entity type
        // Hardware: 24 hours (should be stable)
        // Network: 1 hour (can change)
        // Packages: 6 hours
        // Services: 2 hours

        let count = store.mark_stale_by_age(24)?;
        debug!("Marked {} facts as stale (default 24h threshold)", count);

        Ok(count)
    }

    /// Detect conflicting facts
    async fn detect_conflicts(&self) -> anyhow::Result<usize> {
        let store = self.store.read().await;

        // Get all active facts
        let query = FactQuery::new().active_only();
        let facts = store.query(&query)?;

        // Group by entity+attribute to find conflicts
        let mut groups: std::collections::HashMap<String, Vec<&Fact>> = std::collections::HashMap::new();

        for fact in &facts {
            let key = format!("{}:{}", fact.entity, fact.attribute);
            groups.entry(key).or_default().push(fact);
        }

        let mut conflict_count = 0;

        for (key, group) in groups {
            if group.len() > 1 {
                warn!(
                    "Conflict detected for {}: {} values",
                    key,
                    group.len()
                );
                conflict_count += 1;

                // In a full implementation, we would resolve conflicts here
                // by comparing timestamps, confidence, and source reliability
            }
        }

        Ok(conflict_count)
    }

    /// Clean up old deprecated facts
    async fn cleanup_deprecated(&self) -> anyhow::Result<usize> {
        let store = self.store.read().await;

        // Get deprecated facts older than 7 days
        let cutoff = Utc::now() - Duration::days(7);
        let query = FactQuery::new()
            .status(vec![FactStatus::Deprecated])
            .seen_after(Some(cutoff));

        let deprecated = store.query(&query)?;

        // In a full implementation, we might archive or delete very old deprecated facts
        // For now, just count them
        let count = deprecated.len();

        if count > 0 {
            debug!("Found {} deprecated facts older than 7 days", count);
        }

        Ok(0) // Not actually removing anything for safety
    }

    /// Get facts that need revalidation based on user activity
    pub async fn get_revalidation_candidates(&self, topics: &[String]) -> anyhow::Result<Vec<Fact>> {
        let store = self.store.read().await;

        let mut candidates = Vec::new();

        for topic in topics {
            // Map topic to entity prefix
            let prefix = match topic.as_str() {
                "cpu" => "cpu:",
                "memory" => "system:memory",
                "storage" => "disk:",
                "network" => "net:",
                "dns" => "net:dns",
                "packages" => "pkg:",
                "desktop" => "desktop:",
                _ => continue,
            };

            let query = FactQuery::new()
                .entity_prefix(prefix)
                .status(vec![FactStatus::Stale])
                .limit(10);

            let stale = store.query(&query)?;
            candidates.extend(stale);
        }

        Ok(candidates)
    }

    /// Schedule revalidation for high-priority facts
    pub async fn schedule_revalidation(&self, topic: &str) -> anyhow::Result<Vec<String>> {
        let candidates = self.get_revalidation_candidates(&[topic.to_string()]).await?;

        let fact_ids: Vec<String> = candidates.iter().map(|f| f.id.clone()).collect();

        if !fact_ids.is_empty() {
            info!(
                "Scheduling revalidation for {} {} facts",
                fact_ids.len(),
                topic
            );
        }

        Ok(fact_ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn test_hygiene() -> (KnowledgeHygiene, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_knowledge.db");
        let store = KnowledgeStore::open(&path).unwrap();
        let hygiene = KnowledgeHygiene::new(Arc::new(RwLock::new(store)));
        (hygiene, dir)
    }

    #[tokio::test]
    async fn test_hygiene_run() {
        let (hygiene, _dir) = test_hygiene().await;
        let report = hygiene.run().await.unwrap();

        // Empty store should have no issues
        assert_eq!(report.stale_count, 0);
        assert_eq!(report.conflict_count, 0);
    }
}
