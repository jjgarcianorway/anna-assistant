//! Warm-up Learning v0.20.0
//!
//! Pre-populates the fact cache on startup by running essential probes.
//! Keeps facts fresh in the background for instant access.

use super::{Fact, FactStatus, KnowledgeStore};
use anyhow::Result;
use chrono::{Duration, Utc};
use std::collections::HashSet;

/// Configuration for warm-up behavior
#[derive(Debug, Clone)]
pub struct WarmupConfig {
    /// Probes to run on startup
    pub startup_probes: Vec<String>,
    /// How often to refresh facts (in minutes)
    pub refresh_interval_minutes: u64,
    /// Maximum age before a fact is considered stale (in hours)
    pub max_fact_age_hours: i64,
    /// Whether warm-up is enabled
    pub enabled: bool,
}

impl Default for WarmupConfig {
    fn default() -> Self {
        Self {
            startup_probes: vec![
                "cpu.info".to_string(),
                "mem.info".to_string(),
                "disk.info".to_string(),
                "net.info".to_string(),
                "fs.info".to_string(),
                "gpu.info".to_string(),
            ],
            refresh_interval_minutes: 30,
            max_fact_age_hours: 24,
            enabled: true,
        }
    }
}

/// Fact cache manager for warm-up and background refresh
pub struct FactCache {
    store: KnowledgeStore,
    config: WarmupConfig,
    /// Probes that have been warmed up
    warmed_probes: HashSet<String>,
}

impl FactCache {
    /// Create a new fact cache with default configuration
    pub fn new(store: KnowledgeStore) -> Self {
        Self {
            store,
            config: WarmupConfig::default(),
            warmed_probes: HashSet::new(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(store: KnowledgeStore, config: WarmupConfig) -> Self {
        Self {
            store,
            config,
            warmed_probes: HashSet::new(),
        }
    }

    /// Get facts that need refresh (stale or missing)
    pub fn needs_refresh(&self) -> Vec<String> {
        let mut needs_refresh = Vec::new();
        let cutoff = Utc::now() - Duration::hours(self.config.max_fact_age_hours);

        for probe_id in &self.config.startup_probes {
            // Check if we have fresh facts from this probe
            let entity = format!("probe:{}", probe_id);
            match self.store.get(&entity, "last_run") {
                Ok(Some(fact)) => {
                    if fact.last_seen < cutoff || fact.status == FactStatus::Stale {
                        needs_refresh.push(probe_id.clone());
                    }
                }
                _ => {
                    needs_refresh.push(probe_id.clone());
                }
            }
        }

        needs_refresh
    }

    /// Record that a probe was executed and store its facts
    pub fn record_probe_run(&mut self, probe_id: &str, facts: Vec<Fact>) -> Result<()> {
        // Store all facts from the probe
        for fact in facts {
            self.store.upsert(&fact)?;
        }

        // Record probe execution time
        let meta_fact = Fact::from_probe(
            format!("probe:{}", probe_id),
            "last_run".to_string(),
            Utc::now().to_rfc3339(),
            "warmup",
            1.0,
        );
        self.store.upsert(&meta_fact)?;

        self.warmed_probes.insert(probe_id.to_string());
        Ok(())
    }

    /// Check if a probe has been warmed up
    pub fn is_warmed(&self, probe_id: &str) -> bool {
        self.warmed_probes.contains(probe_id)
    }

    /// Get a fact from cache if fresh, None if stale or missing
    pub fn get_fresh(&self, entity: &str, attribute: &str) -> Option<Fact> {
        let cutoff = Utc::now() - Duration::hours(self.config.max_fact_age_hours);

        match self.store.get(entity, attribute) {
            Ok(Some(fact)) => {
                if fact.last_seen >= cutoff && fact.status == FactStatus::Active {
                    Some(fact)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Get all facts for an entity (regardless of freshness)
    pub fn get_entity_facts(&self, entity: &str) -> Vec<Fact> {
        use super::FactQuery;
        let query = FactQuery::new().entity(entity);
        self.store.query(&query).unwrap_or_default()
    }

    /// Get facts by entity prefix
    pub fn get_facts_by_prefix(&self, prefix: &str) -> Vec<Fact> {
        use super::FactQuery;
        let query = FactQuery::new().entity_prefix(prefix);
        self.store.query(&query).unwrap_or_default()
    }

    /// Get configuration
    pub fn config(&self) -> &WarmupConfig {
        &self.config
    }

    /// Get store reference
    pub fn store(&self) -> &KnowledgeStore {
        &self.store
    }

    /// Check if warm-up is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get warmed probe count
    pub fn warmed_count(&self) -> usize {
        self.warmed_probes.len()
    }
}

/// Result of warm-up phase
#[derive(Debug)]
pub struct WarmupResult {
    /// Probes that were run
    pub probes_run: Vec<String>,
    /// Facts stored
    pub facts_stored: usize,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Any errors encountered
    pub errors: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn test_store() -> (KnowledgeStore, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_warmup.db");
        let store = KnowledgeStore::open(&path).unwrap();
        (store, dir)
    }

    #[test]
    fn test_warmup_config_default() {
        let config = WarmupConfig::default();
        assert!(config.enabled);
        assert!(!config.startup_probes.is_empty());
        assert!(config.startup_probes.contains(&"cpu.info".to_string()));
    }

    #[test]
    fn test_fact_cache_creation() {
        let (store, _dir) = test_store();
        let cache = FactCache::new(store);
        assert!(cache.is_enabled());
        assert_eq!(cache.warmed_count(), 0);
    }

    #[test]
    fn test_needs_refresh_empty_store() {
        let (store, _dir) = test_store();
        let cache = FactCache::new(store);
        let needs = cache.needs_refresh();
        // All startup probes should need refresh
        assert!(!needs.is_empty());
    }

    #[test]
    fn test_record_probe_run() {
        let (store, _dir) = test_store();
        let mut cache = FactCache::new(store);

        let facts = vec![Fact::from_probe(
            "cpu:0".to_string(),
            "model".to_string(),
            "AMD Ryzen".to_string(),
            "cpu.info",
            0.95,
        )];

        cache.record_probe_run("cpu.info", facts).unwrap();
        assert!(cache.is_warmed("cpu.info"));
        assert_eq!(cache.warmed_count(), 1);
    }

    #[test]
    fn test_get_fresh() {
        let (store, _dir) = test_store();
        let cache = FactCache::new(store);

        let fact = Fact::from_probe(
            "test:entity".to_string(),
            "attr".to_string(),
            "value".to_string(),
            "test",
            0.9,
        );
        cache.store.upsert(&fact).unwrap();

        let retrieved = cache.get_fresh("test:entity", "attr");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().value, "value");
    }
}
