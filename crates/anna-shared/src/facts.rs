//! Facts store for verified user/system facts.
//!
//! Persists validated facts so Anna stops re-asking and can validate user clarifications.
//! Facts are only stored after verification against system reality.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Keys for facts that Anna can learn and remember
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum FactKey {
    /// User's preferred text editor (verified to exist)
    PreferredEditor,
    /// Whether a specific editor is installed: EditorInstalled("vim") -> true
    EditorInstalled(String),
    /// Path to a binary: BinaryAvailable("vim") -> "/usr/bin/vim"
    BinaryAvailable(String),
    /// Primary network interface: "wlan0" or "enp0s3"
    NetworkPrimaryInterface,
    /// Interface type preference: "wifi" or "ethernet"
    NetworkPreference,
    /// Default shell: "bash", "zsh", etc.
    PreferredShell,
    /// Init system: "systemd", "openrc", etc.
    InitSystem,
    /// Package manager: "pacman", "apt", "dnf"
    PackageManager,
    /// Whether a systemd unit exists: UnitExists("nginx.service") -> true
    UnitExists(String),
    /// Whether a mount point exists: MountExists("/var") -> true
    MountExists(String),
    /// Custom fact with string key
    Custom(String),
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
            Self::Custom(k) => write!(f, "custom:{}", k),
        }
    }
}

/// A verified fact about the user or system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    /// The fact key
    pub key: FactKey,
    /// The verified value
    pub value: String,
    /// Whether this fact has been verified against system reality
    pub verified: bool,
    /// How this fact was verified: "probe:which vim", "user+verify", "system_detect"
    pub source: String,
    /// Unix timestamp when fact was stored
    pub timestamp: u64,
}

impl Fact {
    /// Create a new verified fact
    pub fn verified(key: FactKey, value: String, source: String) -> Self {
        Self {
            key,
            value,
            verified: true,
            source,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }

    /// Create an unverified fact (pending verification)
    pub fn unverified(key: FactKey, value: String, source: String) -> Self {
        Self {
            key,
            value,
            verified: false,
            source,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }
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

    /// Get a verified fact value by key
    pub fn get_verified(&self, key: &FactKey) -> Option<&str> {
        self.facts.get(key)
            .filter(|f| f.verified)
            .map(|f| f.value.as_str())
    }

    /// Check if a fact exists and is verified
    pub fn has_verified(&self, key: &FactKey) -> bool {
        self.facts.get(key).map(|f| f.verified).unwrap_or(false)
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
            fact.timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            true
        } else {
            false
        }
    }

    /// Remove a fact
    pub fn remove(&mut self, key: &FactKey) -> Option<Fact> {
        self.facts.remove(key)
    }

    /// Get all verified facts
    pub fn verified_facts(&self) -> Vec<&Fact> {
        self.facts.values().filter(|f| f.verified).collect()
    }

    /// Get count of verified facts
    pub fn verified_count(&self) -> usize {
        self.facts.values().filter(|f| f.verified).count()
    }

    /// Clear all facts
    pub fn clear(&mut self) {
        self.facts.clear();
    }
}

/// Result of checking if a fact is known
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FactStatus {
    /// Fact is known and verified
    Known(String),
    /// Fact exists but is unverified
    Unverified(String),
    /// Fact is unknown
    Unknown,
}

impl FactsStore {
    /// Check the status of a fact
    pub fn status(&self, key: &FactKey) -> FactStatus {
        match self.facts.get(key) {
            Some(f) if f.verified => FactStatus::Known(f.value.clone()),
            Some(f) => FactStatus::Unverified(f.value.clone()),
            None => FactStatus::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_fact_key_display() {
        assert_eq!(FactKey::PreferredEditor.to_string(), "preferred_editor");
        assert_eq!(FactKey::EditorInstalled("vim".to_string()).to_string(), "editor_installed:vim");
        assert_eq!(FactKey::BinaryAvailable("nvim".to_string()).to_string(), "binary_available:nvim");
    }

    #[test]
    fn test_fact_creation() {
        let fact = Fact::verified(
            FactKey::PreferredEditor,
            "vim".to_string(),
            "probe:which vim".to_string(),
        );
        assert!(fact.verified);
        assert_eq!(fact.value, "vim");
        assert_eq!(fact.source, "probe:which vim");
    }

    #[test]
    fn test_facts_store_set_get() {
        let mut store = FactsStore::new();

        store.set_verified(
            FactKey::PreferredEditor,
            "vim".to_string(),
            "user+verify".to_string(),
        );

        assert!(store.has_verified(&FactKey::PreferredEditor));
        assert_eq!(store.get_verified(&FactKey::PreferredEditor), Some("vim"));
    }

    #[test]
    fn test_facts_store_unverified_not_returned_as_verified() {
        let mut store = FactsStore::new();

        store.set_unverified(
            FactKey::PreferredEditor,
            "vim".to_string(),
            "user_claim".to_string(),
        );

        assert!(!store.has_verified(&FactKey::PreferredEditor));
        assert_eq!(store.get_verified(&FactKey::PreferredEditor), None);

        // But we can get the unverified fact
        assert!(store.get(&FactKey::PreferredEditor).is_some());
    }

    #[test]
    fn test_facts_store_verify() {
        let mut store = FactsStore::new();

        store.set_unverified(
            FactKey::PreferredEditor,
            "vim".to_string(),
            "user_claim".to_string(),
        );

        assert!(!store.has_verified(&FactKey::PreferredEditor));

        store.verify(&FactKey::PreferredEditor, "probe:which vim".to_string());

        assert!(store.has_verified(&FactKey::PreferredEditor));
        assert_eq!(store.get_verified(&FactKey::PreferredEditor), Some("vim"));
    }

    #[test]
    fn test_facts_store_save_load_deterministic() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("facts.json");

        let mut store = FactsStore::new();
        store.set_verified(FactKey::PreferredEditor, "vim".to_string(), "test".to_string());
        store.set_verified(FactKey::PreferredShell, "zsh".to_string(), "test".to_string());
        store.set_verified(
            FactKey::BinaryAvailable("vim".to_string()),
            "/usr/bin/vim".to_string(),
            "test".to_string(),
        );

        store.save_to_path(&path).unwrap();
        let content1 = fs::read_to_string(&path).unwrap();

        // Save again
        store.save_to_path(&path).unwrap();
        let content2 = fs::read_to_string(&path).unwrap();

        // Deterministic output
        assert_eq!(content1, content2);

        // Load and verify
        let loaded = FactsStore::load_from_path(&path);
        assert_eq!(loaded.get_verified(&FactKey::PreferredEditor), Some("vim"));
        assert_eq!(loaded.get_verified(&FactKey::PreferredShell), Some("zsh"));
    }

    #[test]
    fn test_facts_store_only_saves_verified() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("facts.json");

        let mut store = FactsStore::new();
        store.set_verified(FactKey::PreferredEditor, "vim".to_string(), "verified".to_string());
        store.set_unverified(FactKey::PreferredShell, "zsh".to_string(), "unverified".to_string());

        store.save_to_path(&path).unwrap();

        let loaded = FactsStore::load_from_path(&path);
        assert!(loaded.has_verified(&FactKey::PreferredEditor));
        assert!(!loaded.has_verified(&FactKey::PreferredShell));
        assert!(loaded.get(&FactKey::PreferredShell).is_none()); // Not saved at all
    }

    #[test]
    fn test_facts_store_overwrite() {
        let mut store = FactsStore::new();

        store.set_verified(FactKey::PreferredEditor, "vim".to_string(), "test1".to_string());
        assert_eq!(store.get_verified(&FactKey::PreferredEditor), Some("vim"));

        store.set_verified(FactKey::PreferredEditor, "nvim".to_string(), "test2".to_string());
        assert_eq!(store.get_verified(&FactKey::PreferredEditor), Some("nvim"));
    }

    #[test]
    fn test_fact_status() {
        let mut store = FactsStore::new();

        assert_eq!(store.status(&FactKey::PreferredEditor), FactStatus::Unknown);

        store.set_unverified(FactKey::PreferredEditor, "vim".to_string(), "claim".to_string());
        assert_eq!(store.status(&FactKey::PreferredEditor), FactStatus::Unverified("vim".to_string()));

        store.verify(&FactKey::PreferredEditor, "probe".to_string());
        assert_eq!(store.status(&FactKey::PreferredEditor), FactStatus::Known("vim".to_string()));
    }

    #[test]
    fn test_verified_facts_list() {
        let mut store = FactsStore::new();
        store.set_verified(FactKey::PreferredEditor, "vim".to_string(), "test".to_string());
        store.set_verified(FactKey::PreferredShell, "zsh".to_string(), "test".to_string());
        store.set_unverified(FactKey::InitSystem, "systemd".to_string(), "claim".to_string());

        assert_eq!(store.verified_count(), 2);
        assert_eq!(store.verified_facts().len(), 2);
    }
}
