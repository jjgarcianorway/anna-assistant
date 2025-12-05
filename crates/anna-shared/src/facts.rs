//! Facts store with lifecycle management (v0.0.32, enhanced v0.0.41).
//!
//! Persists validated facts with staleness policies and automatic expiration.
//! Facts transition: Active -> Stale -> Archived based on TTL and verification.
//!
//! v0.0.41: Added FactSource, FactValue, confidence, and pinned TTL rules.
//! Types extracted to facts_types.rs to keep under 400 lines.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// Re-export types from facts_types
pub use crate::facts_types::{FactSource, FactValue};

/// Keys for facts that Anna can learn and remember
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum FactKey {
    PreferredEditor,
    EditorInstalled(String),
    BinaryAvailable(String),
    NetworkPrimaryInterface,
    NetworkPreference,
    PreferredShell,
    InitSystem,
    PackageManager,
    UnitExists(String),
    MountExists(String),
    // v0.0.41 additions
    WallpaperFolder,
    BootTimeBaseline,
    InstalledPackage(String),
    Desktop,
    GpuPresent,
    Hostname,
    Kernel,
    Custom(String),
}

/// Staleness policy for facts (v0.0.32)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StalenessPolicy {
    Never,
    TTLSeconds(u64),
    SessionOnly,
}

impl Default for StalenessPolicy {
    fn default() -> Self { Self::TTLSeconds(30 * 24 * 3600) } // 30 days
}

/// Pinned TTL constants for v0.0.41
pub mod ttl {
    /// Installed packages: 7 days (invalidated on pacman hooks later)
    pub const INSTALLED_PACKAGE_SECS: u64 = 7 * 24 * 3600;
    /// Preferred editor: 90 days
    pub const PREFERRED_EDITOR_SECS: u64 = 90 * 24 * 3600;
    /// Boot time baseline: 30 days (keep 14 samples in history)
    pub const BOOT_TIME_SECS: u64 = 30 * 24 * 3600;
    /// Network facts: 1 day
    pub const NETWORK_SECS: u64 = 24 * 3600;
    /// Binary available: 7 days
    pub const BINARY_AVAILABLE_SECS: u64 = 7 * 24 * 3600;
    /// Desktop environment: 30 days
    pub const DESKTOP_SECS: u64 = 30 * 24 * 3600;
}

/// Get default staleness policy for a fact key (v0.0.41 pinned TTLs)
pub fn default_policy(key: &FactKey) -> StalenessPolicy {
    match key {
        FactKey::PreferredEditor => StalenessPolicy::TTLSeconds(ttl::PREFERRED_EDITOR_SECS),
        FactKey::BinaryAvailable(_) => StalenessPolicy::TTLSeconds(ttl::BINARY_AVAILABLE_SECS),
        FactKey::EditorInstalled(_) => StalenessPolicy::TTLSeconds(ttl::BINARY_AVAILABLE_SECS),
        FactKey::NetworkPrimaryInterface => StalenessPolicy::TTLSeconds(ttl::NETWORK_SECS),
        FactKey::NetworkPreference => StalenessPolicy::TTLSeconds(ttl::NETWORK_SECS),
        FactKey::InstalledPackage(_) => StalenessPolicy::TTLSeconds(ttl::INSTALLED_PACKAGE_SECS),
        FactKey::BootTimeBaseline => StalenessPolicy::TTLSeconds(ttl::BOOT_TIME_SECS),
        FactKey::Desktop => StalenessPolicy::TTLSeconds(ttl::DESKTOP_SECS),
        FactKey::InitSystem | FactKey::PackageManager | FactKey::Hostname | FactKey::Kernel => {
            StalenessPolicy::Never // System constants rarely change
        }
        FactKey::GpuPresent => StalenessPolicy::Never, // Hardware doesn't change
        FactKey::UnitExists(_) | FactKey::MountExists(_) => {
            StalenessPolicy::TTLSeconds(ttl::BINARY_AVAILABLE_SECS)
        }
        _ => StalenessPolicy::TTLSeconds(30 * 24 * 3600),
    }
}

/// Lifecycle status for facts (v0.0.32)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FactLifecycle {
    #[default]
    Active,
    Stale,
    Archived,
}

impl std::fmt::Display for FactKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PreferredEditor => write!(f, "preferred_editor"),
            Self::EditorInstalled(e) => write!(f, "editor_installed:{}", e),
            Self::BinaryAvailable(b) => write!(f, "binary_available:{}", b),
            Self::NetworkPrimaryInterface => write!(f, "network_primary_interface"),
            Self::NetworkPreference => write!(f, "network_preference"),
            Self::PreferredShell => write!(f, "preferred_shell"),
            Self::InitSystem => write!(f, "init_system"),
            Self::PackageManager => write!(f, "package_manager"),
            Self::UnitExists(u) => write!(f, "unit_exists:{}", u),
            Self::MountExists(m) => write!(f, "mount_exists:{}", m),
            // v0.0.41 additions
            Self::WallpaperFolder => write!(f, "wallpaper_folder"),
            Self::BootTimeBaseline => write!(f, "boot_time_baseline"),
            Self::InstalledPackage(p) => write!(f, "installed_package:{}", p),
            Self::Desktop => write!(f, "desktop"),
            Self::GpuPresent => write!(f, "gpu_present"),
            Self::Hostname => write!(f, "hostname"),
            Self::Kernel => write!(f, "kernel"),
            Self::Custom(k) => write!(f, "custom:{}", k),
        }
    }
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

fn now_epoch() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

impl Fact {
    pub fn verified(key: FactKey, value: String, source: String) -> Self {
        let now = now_epoch();
        let policy = default_policy(&key);
        Self {
            key, value, typed_value: None, verified: true, source,
            fact_source: None, confidence: 80, // Verified but legacy source
            lifecycle: FactLifecycle::Active, policy,
            created_at: now, last_verified_at: now, timestamp_compat: now
        }
    }

    pub fn unverified(key: FactKey, value: String, source: String) -> Self {
        let now = now_epoch();
        let policy = default_policy(&key);
        Self {
            key, value, typed_value: None, verified: false, source,
            fact_source: None, confidence: 0, // Unverified
            lifecycle: FactLifecycle::Active, policy,
            created_at: now, last_verified_at: 0, timestamp_compat: now
        }
    }

    /// Create a verified fact with typed source (v0.0.41)
    pub fn verified_with_source(key: FactKey, value: FactValue, source: FactSource, confidence: u8) -> Self {
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
            FactSource::UserConfirmed { transcript_id: transcript_id.to_string() },
            90, // User-confirmed = high confidence
        )
    }

    /// Check if this fact is stale based on current time
    pub fn is_stale(&self, now: u64) -> bool {
        match self.policy {
            StalenessPolicy::Never => false,
            StalenessPolicy::SessionOnly => true, // Always stale for persistence purposes
            StalenessPolicy::TTLSeconds(ttl) => {
                if self.last_verified_at == 0 { return !self.verified; }
                now.saturating_sub(self.last_verified_at) > ttl
            }
        }
    }

    /// Check if should be archived (stale for > 2x TTL)
    pub fn should_archive(&self, now: u64) -> bool {
        match self.policy {
            StalenessPolicy::TTLSeconds(ttl) => {
                if self.last_verified_at == 0 { return false; }
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
    pub fn mark_stale(&mut self) { self.lifecycle = FactLifecycle::Stale; }

    /// Archive this fact
    pub fn archive(&mut self) { self.lifecycle = FactLifecycle::Archived; }

    /// Check if usable for decisions (verified and active)
    pub fn is_usable(&self) -> bool { self.verified && self.lifecycle == FactLifecycle::Active }
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
        Ok(Self { facts, version: wire.version })
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
        let mut verified: Vec<Fact> = self.facts.values()
            .filter(|f| f.verified)
            .cloned()
            .collect();
        verified.sort_by(|a, b| a.key.to_string().cmp(&b.key.to_string()));

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let wire = FactsStoreWire { facts: verified, version: self.version };
        let json = serde_json::to_string_pretty(&wire)?;
        fs::write(path, json)
    }

    /// Get a fact by key
    pub fn get(&self, key: &FactKey) -> Option<&Fact> {
        self.facts.get(key)
    }

    /// Get a verified fact value by key (must be usable: verified + active)
    pub fn get_verified(&self, key: &FactKey) -> Option<&str> {
        self.facts.get(key)
            .filter(|f| f.is_usable())
            .map(|f| f.value.as_str())
    }

    /// Get a fresh fact (v0.0.41) - returns None if stale
    /// Use this for decisions that require current data
    pub fn get_fresh(&self, key: &FactKey, now: u64) -> Option<&Fact> {
        self.facts.get(key).filter(|f| f.is_usable() && !f.is_stale(now))
    }

    /// Upsert verified fact (v0.0.41) - updates last_verified on successful verification
    pub fn upsert_verified(&mut self, key: FactKey, value: FactValue, source: FactSource, confidence: u8) {
        let fact = Fact::verified_with_source(key.clone(), value, source, confidence);
        self.facts.insert(key, fact);
    }

    /// Check if a fact exists and is usable (verified + active lifecycle)
    pub fn has_verified(&self, key: &FactKey) -> bool {
        self.facts.get(key).map(|f| f.is_usable()).unwrap_or(false)
    }

    /// Check if fact is fresh (not stale) at given time (v0.0.41)
    pub fn is_fresh(&self, key: &FactKey, now: u64) -> bool {
        self.facts.get(key)
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
    pub fn clear(&mut self) { self.facts.clear(); }

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
        } else { false }
    }

    /// Get stale facts that need re-verification
    pub fn stale_facts(&self) -> Vec<&Fact> {
        self.facts.values().filter(|f| f.lifecycle == FactLifecycle::Stale).collect()
    }

    /// Remove archived facts
    pub fn prune_archived(&mut self) -> usize {
        let before = self.facts.len();
        self.facts.retain(|_, f| f.lifecycle != FactLifecycle::Archived);
        before - self.facts.len()
    }

    /// Get mutable access to facts (for testing)
    pub fn facts_mut(&mut self) -> &mut HashMap<FactKey, Fact> {
        &mut self.facts
    }
}

// Tests moved to tests/facts_tests.rs

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
        match self.facts.get(key) {
            Some(f) if f.is_usable() => FactStatus::Known(f.value.clone()),
            Some(f) if f.lifecycle == FactLifecycle::Stale => FactStatus::Stale(f.value.clone()),
            Some(f) if !f.verified => FactStatus::Unverified(f.value.clone()),
            Some(f) => FactStatus::Stale(f.value.clone()), // Archived treated as stale
            None => FactStatus::Unknown,
        }
    }
}

// Tests in tests/facts_tests.rs
