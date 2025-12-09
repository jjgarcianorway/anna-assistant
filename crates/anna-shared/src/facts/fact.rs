//! Fact struct and implementation (v0.0.181).

use serde::{Deserialize, Serialize};

use crate::facts_types::{FactSource, FactValue};

use super::key::FactKey;
use super::policy::{default_policy, FactLifecycle, StalenessPolicy};

fn now_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// A fact with lifecycle metadata (v0.0.32, enhanced v0.0.41)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    pub key: FactKey,
    /// v0.0.41: Legacy string value, use typed_value for new facts
    pub value: String,
    /// v0.0.41: Typed value (optional for backwards compat)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub typed_value: Option<FactValue>,
    pub verified: bool,
    /// v0.0.41: Legacy string source, use fact_source for new facts
    pub source: String,
    /// v0.0.41: Typed source (optional for backwards compat)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fact_source: Option<FactSource>,
    /// v0.0.41: Confidence 0-100 (0 = unverified, 100 = probe-confirmed)
    #[serde(default)]
    pub confidence: u8,
    #[serde(default)]
    pub lifecycle: FactLifecycle,
    #[serde(default)]
    pub policy: StalenessPolicy,
    #[serde(default)]
    pub created_at: u64,
    #[serde(default)]
    pub last_verified_at: u64,
    #[serde(default, rename = "timestamp")]
    timestamp_compat: u64, // backwards compat
}

impl Fact {
    pub fn verified(key: FactKey, value: String, source: String) -> Self {
        let now = now_epoch();
        let policy = default_policy(&key);
        Self {
            key,
            value,
            typed_value: None,
            verified: true,
            source,
            fact_source: None,
            confidence: 80, // Verified but legacy source
            lifecycle: FactLifecycle::Active,
            policy,
            created_at: now,
            last_verified_at: now,
            timestamp_compat: now,
        }
    }

    pub fn unverified(key: FactKey, value: String, source: String) -> Self {
        let now = now_epoch();
        let policy = default_policy(&key);
        Self {
            key,
            value,
            typed_value: None,
            verified: false,
            source,
            fact_source: None,
            confidence: 0, // Unverified
            lifecycle: FactLifecycle::Active,
            policy,
            created_at: now,
            last_verified_at: 0,
            timestamp_compat: now,
        }
    }

    /// Create a verified fact with typed source (v0.0.41)
    pub fn verified_with_source(
        key: FactKey,
        value: FactValue,
        source: FactSource,
        confidence: u8,
    ) -> Self {
        let now = now_epoch();
        let policy = default_policy(&key);
        Self {
            key,
            value: value.to_string_value(),
            typed_value: Some(value),
            verified: true,
            source: "typed".to_string(),
            fact_source: Some(source),
            confidence,
            lifecycle: FactLifecycle::Active,
            policy,
            created_at: now,
            last_verified_at: now,
            timestamp_compat: now,
        }
    }

    /// Create from probe observation (v0.0.41)
    pub fn from_probe(key: FactKey, value: FactValue, probe_id: &str, output_hash: &str) -> Self {
        Self::verified_with_source(
            key,
            value,
            FactSource::ObservedProbe {
                probe_id: probe_id.to_string(),
                output_hash: output_hash.to_string(),
            },
            100, // Probe-confirmed = high confidence
        )
    }

    /// Create from user confirmation (v0.0.41)
    pub fn from_user(key: FactKey, value: FactValue, transcript_id: &str) -> Self {
        Self::verified_with_source(
            key,
            value,
            FactSource::UserConfirmed {
                transcript_id: transcript_id.to_string(),
            },
            90, // User-confirmed = high confidence
        )
    }

    /// Check if this fact is stale based on current time
    pub fn is_stale(&self, now: u64) -> bool {
        match self.policy {
            StalenessPolicy::Never => false,
            StalenessPolicy::SessionOnly => true, // Always stale for persistence purposes
            StalenessPolicy::TTLSeconds(ttl) => {
                if self.last_verified_at == 0 {
                    return !self.verified;
                }
                now.saturating_sub(self.last_verified_at) > ttl
            }
        }
    }

    /// Check if should be archived (stale for > 2x TTL)
    pub fn should_archive(&self, now: u64) -> bool {
        match self.policy {
            StalenessPolicy::TTLSeconds(ttl) => {
                if self.last_verified_at == 0 {
                    return false;
                }
                now.saturating_sub(self.last_verified_at) > ttl * 2
            }
            _ => false,
        }
    }

    /// Re-verify this fact, resetting staleness
    pub fn reverify(&mut self, source: String) {
        self.verified = true;
        self.source = source;
        self.lifecycle = FactLifecycle::Active;
        self.last_verified_at = now_epoch();
    }

    /// Mark as stale (failed re-verification)
    pub fn mark_stale(&mut self) {
        self.lifecycle = FactLifecycle::Stale;
    }

    /// Archive this fact
    pub fn archive(&mut self) {
        self.lifecycle = FactLifecycle::Archived;
    }

    /// Check if usable for decisions (verified and active)
    pub fn is_usable(&self) -> bool {
        self.verified && self.lifecycle == FactLifecycle::Active
    }
}
