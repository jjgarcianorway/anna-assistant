//! Facts store with lifecycle management (v0.0.181).
//!
//! Persists validated facts with staleness policies and automatic expiration.
//! Facts transition: Active -> Stale -> Archived based on TTL and verification.
//!
//! v0.0.41: Added FactSource, FactValue, confidence, and pinned TTL rules.
//! v0.0.181: Modularized into domain-focused submodules.

mod fact;
mod key;
mod policy;
mod status;
mod store;

// Re-export types from facts_types
pub use crate::facts_types::{FactSource, FactValue};

// Re-export main types
pub use fact::Fact;
pub use key::FactKey;
pub use policy::{default_policy, ttl, FactLifecycle, StalenessPolicy};
pub use status::FactStatus;
pub use store::FactsStore;
