//! Fact status enum and status checking (v0.0.181).

use super::key::FactKey;
use super::policy::FactLifecycle;
use super::store::FactsStore;

/// Result of checking if a fact is known (v0.0.32: includes Stale)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FactStatus {
    Known(String),
    Unverified(String),
    Stale(String),
    Unknown,
}

impl FactsStore {
    /// Check the status of a fact (considers lifecycle)
    pub fn status(&self, key: &FactKey) -> FactStatus {
        match self.get(key) {
            Some(f) if f.is_usable() => FactStatus::Known(f.value.clone()),
            Some(f) if f.lifecycle == FactLifecycle::Stale => FactStatus::Stale(f.value.clone()),
            Some(f) if !f.verified => FactStatus::Unverified(f.value.clone()),
            Some(f) => FactStatus::Stale(f.value.clone()), // Archived treated as stale
            None => FactStatus::Unknown,
        }
    }
}
