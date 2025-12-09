//! Facts store implementation (v0.0.181).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::facts_types::{FactSource, FactValue};

use super::fact::Fact;
use super::key::FactKey;
use super::policy::FactLifecycle;

fn now_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Persistent store for verified facts (serializes as Vec for JSON compatibility)
#[derive(Debug, Clone, Default)]
pub struct FactsStore {
    /// Map of fact keys to facts (only verified facts should be persisted)
    facts: HashMap<FactKey, Fact>,
    /// Version for forward compatibility
    version: u32,
}

/// Wire format for FactsStore serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FactsStoreWire {
    facts: Vec<Fact>,
    #[serde(default)]
    version: u32,
}

impl Serialize for FactsStore {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let wire = FactsStoreWire {
            facts: self.facts.values().cloned().collect(),
            version: self.version,
        };
        wire.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for FactsStore {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = FactsStoreWire::deserialize(deserializer)?;
        let facts = wire.facts.into_iter().map(|f| (f.key.clone(), f)).collect();
        Ok(Self {
            facts,
            version: wire.version,
        })
    }
}

impl FactsStore {
    /// Default path for facts store
    pub fn default_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".anna")
            .join("facts.json")
    }

    /// Create a new empty facts store
    pub fn new() -> Self {
        Self {
            facts: HashMap::new(),
            version: 1,
        }
    }

    /// Load facts store from default path
    pub fn load() -> Self {
        Self::load_from_path(&Self::default_path())
    }

    /// Load facts store from specific path
    pub fn load_from_path(path: &PathBuf) -> Self {
        match fs::read_to_string(path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Self::new(),
        }
    }

    /// Save facts store to default path
    pub fn save(&self) -> Result<(), std::io::Error> {
        self.save_to_path(&Self::default_path())
    }

    /// Save facts store to specific path
    pub fn save_to_path(&self, path: &PathBuf) -> Result<(), std::io::Error> {
        // Only save verified facts, sorted for deterministic output
        let mut verified: Vec<Fact> = self
            .facts
            .values()
            .filter(|f| f.verified)
            .cloned()
            .collect();
        verified.sort_by(|a, b| a.key.to_string().cmp(&b.key.to_string()));

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let wire = FactsStoreWire {
            facts: verified,
            version: self.version,
        };
        let json = serde_json::to_string_pretty(&wire)?;
        fs::write(path, json)
    }

    /// Get a fact by key
    pub fn get(&self, key: &FactKey) -> Option<&Fact> {
        self.facts.get(key)
    }

    /// Get a verified fact value by key (must be usable: verified + active)
    pub fn get_verified(&self, key: &FactKey) -> Option<&str> {
        self.facts
            .get(key)
            .filter(|f| f.is_usable())
            .map(|f| f.value.as_str())
    }

    /// Get a fresh fact (v0.0.41) - returns None if stale
    /// Use this for decisions that require current data
    pub fn get_fresh(&self, key: &FactKey, now: u64) -> Option<&Fact> {
        self.facts
            .get(key)
            .filter(|f| f.is_usable() && !f.is_stale(now))
    }

    /// Upsert verified fact (v0.0.41) - updates last_verified on successful verification
    pub fn upsert_verified(
        &mut self,
        key: FactKey,
        value: FactValue,
        source: FactSource,
        confidence: u8,
    ) {
        let fact = Fact::verified_with_source(key.clone(), value, source, confidence);
        self.facts.insert(key, fact);
    }

    /// Check if a fact exists and is usable (verified + active lifecycle)
    pub fn has_verified(&self, key: &FactKey) -> bool {
        self.facts.get(key).map(|f| f.is_usable()).unwrap_or(false)
    }

    /// Check if fact is fresh (not stale) at given time (v0.0.41)
    pub fn is_fresh(&self, key: &FactKey, now: u64) -> bool {
        self.facts
            .get(key)
            .map(|f| f.is_usable() && !f.is_stale(now))
            .unwrap_or(false)
    }

    /// Set a verified fact (overwrites any existing)
    pub fn set_verified(&mut self, key: FactKey, value: String, source: String) {
        let fact = Fact::verified(key.clone(), value, source);
        self.facts.insert(key, fact);
    }

    /// Set an unverified fact (pending verification, not persisted)
    pub fn set_unverified(&mut self, key: FactKey, value: String, source: String) {
        let fact = Fact::unverified(key.clone(), value, source);
        self.facts.insert(key, fact);
    }

    /// Mark an existing unverified fact as verified
    pub fn verify(&mut self, key: &FactKey, source: String) -> bool {
        if let Some(fact) = self.facts.get_mut(key) {
            fact.verified = true;
            fact.source = source;
            fact.lifecycle = FactLifecycle::Active;
            fact.last_verified_at = now_epoch();
            true
        } else {
            false
        }
    }

    /// Remove a fact
    pub fn remove(&mut self, key: &FactKey) -> Option<Fact> {
        self.facts.remove(key)
    }

    /// Get all verified and active facts
    pub fn verified_facts(&self) -> Vec<&Fact> {
        self.facts.values().filter(|f| f.is_usable()).collect()
    }

    /// Get count of usable facts
    pub fn verified_count(&self) -> usize {
        self.facts.values().filter(|f| f.is_usable()).count()
    }

    /// Clear all facts
    pub fn clear(&mut self) {
        self.facts.clear();
    }

    // === Lifecycle management (v0.0.32) ===

    /// Apply lifecycle transitions based on current time
    pub fn apply_lifecycle(&mut self, now: u64) {
        for fact in self.facts.values_mut() {
            if fact.lifecycle == FactLifecycle::Active && fact.is_stale(now) {
                fact.lifecycle = FactLifecycle::Stale;
            }
            if fact.lifecycle == FactLifecycle::Stale && fact.should_archive(now) {
                fact.lifecycle = FactLifecycle::Archived;
            }
        }
    }

    /// Mark a fact as stale (failed verification)
    pub fn invalidate(&mut self, key: &FactKey) {
        if let Some(fact) = self.facts.get_mut(key) {
            fact.mark_stale();
        }
    }

    /// Re-verify a fact, making it active again
    pub fn reverify(&mut self, key: &FactKey, source: String) -> bool {
        if let Some(fact) = self.facts.get_mut(key) {
            fact.reverify(source);
            true
        } else {
            false
        }
    }

    /// Get stale facts that need re-verification
    pub fn stale_facts(&self) -> Vec<&Fact> {
        self.facts
            .values()
            .filter(|f| f.lifecycle == FactLifecycle::Stale)
            .collect()
    }

    /// Remove archived facts
    pub fn prune_archived(&mut self) -> usize {
        let before = self.facts.len();
        self.facts
            .retain(|_, f| f.lifecycle != FactLifecycle::Archived);
        before - self.facts.len()
    }

    /// Get mutable access to facts (for testing)
    pub fn facts_mut(&mut self) -> &mut HashMap<FactKey, Fact> {
        &mut self.facts
    }
}
