//! Protocol v0.23.0: System Brain, User Brain & Idle Learning
//!
//! This module implements:
//! - Two-tier knowledge system (system vs user scope)
//! - Idle learning engine with resource limits
//! - Safe file scanning within allowed paths
//! - LLM-driven learning missions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Protocol version identifier
pub const PROTOCOL_VERSION_V23: &str = "0.23.0";

// ============================================================================
// KNOWLEDGE SCOPE
// ============================================================================

/// Knowledge scope - system-wide vs per-user
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeScope {
    /// Machine-wide facts shared by all users
    /// Keys: system.hardware.*, system.pkg.*, system.fs.*, etc.
    System,
    /// Per-user facts specific to $USER
    /// Keys: user.editor.*, user.terminal.*, user.dewm.*, etc.
    User,
}

impl KnowledgeScope {
    /// Get the key prefix for this scope
    pub fn prefix(&self) -> &'static str {
        match self {
            KnowledgeScope::System => "system",
            KnowledgeScope::User => "user",
        }
    }

    /// Determine scope from a fact key
    pub fn from_key(key: &str) -> Option<Self> {
        if key.starts_with("system.") {
            Some(KnowledgeScope::System)
        } else if key.starts_with("user.") {
            Some(KnowledgeScope::User)
        } else {
            None
        }
    }

    /// Check if a key belongs to this scope
    pub fn owns_key(&self, key: &str) -> bool {
        key.starts_with(self.prefix())
    }
}

// ============================================================================
// SYSTEM KNOWLEDGE STORE
// ============================================================================

/// Identity for a system knowledge store
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SystemIdentity {
    /// Hostname of the machine
    pub hostname: String,
    /// Machine ID (from /etc/machine-id or similar)
    pub machine_id: String,
}

impl SystemIdentity {
    /// Create a new system identity
    pub fn new(hostname: String, machine_id: String) -> Self {
        Self { hostname, machine_id }
    }

    /// Generate a storage key for this identity
    pub fn storage_key(&self) -> String {
        format!("{}_{}", self.hostname, &self.machine_id[..8.min(self.machine_id.len())])
    }
}

/// A fact stored in the system knowledge store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemFact {
    /// Fact key (e.g., "system.hardware.cpu.model")
    pub key: String,
    /// Fact value
    pub value: String,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Source of this fact
    pub source: FactSourceV23,
    /// When this fact was recorded (Unix timestamp)
    pub recorded_at: i64,
    /// When this fact expires (Unix timestamp, 0 = never)
    pub expires_at: i64,
}

/// A fact stored in the user knowledge store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFact {
    /// Fact key (e.g., "user.editor.primary")
    pub key: String,
    /// Fact value
    pub value: String,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Source of this fact
    pub source: FactSourceV23,
    /// When this fact was recorded (Unix timestamp)
    pub recorded_at: i64,
    /// When this fact expires (Unix timestamp, 0 = never)
    pub expires_at: i64,
    /// Username this fact belongs to
    pub username: String,
}

/// Source of a fact in v0.23.0
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactSourceV23 {
    /// From a probe execution
    Probe { probe_id: String },
    /// From file scanning
    FileScan { path: String },
    /// User explicitly stated
    UserAsserted,
    /// Inferred from other facts
    Inferred { from_keys: Vec<String> },
    /// From idle learning
    IdleLearning { mission_id: String },
}

// ============================================================================
// USER KNOWLEDGE STORE
// ============================================================================

/// Identity for a user knowledge store
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserIdentity {
    /// Username ($USER)
    pub username: String,
    /// Hostname (to avoid confusion across nodes)
    pub hostname: String,
}

impl UserIdentity {
    /// Create a new user identity
    pub fn new(username: String, hostname: String) -> Self {
        Self { username, hostname }
    }

    /// Generate a storage key for this identity
    pub fn storage_key(&self) -> String {
        format!("{}@{}", self.username, self.hostname)
    }
}

// ============================================================================
// DUAL KNOWLEDGE BRAIN
// ============================================================================

/// The dual knowledge brain managing both system and user facts
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DualBrain {
    /// System-wide facts (machine scope)
    pub system_facts: HashMap<String, SystemFact>,
    /// User-specific facts (per-user scope)
    pub user_facts: HashMap<String, UserFact>,
    /// System identity
    pub system_id: Option<SystemIdentity>,
    /// Current user identity
    pub user_id: Option<UserIdentity>,
}

impl DualBrain {
    /// Create a new dual brain
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize with identities
    pub fn with_identities(system_id: SystemIdentity, user_id: UserIdentity) -> Self {
        Self {
            system_facts: HashMap::new(),
            user_facts: HashMap::new(),
            system_id: Some(system_id),
            user_id: Some(user_id),
        }
    }

    /// Get a fact by key (checks scope automatically)
    pub fn get_fact(&self, key: &str) -> Option<FactValue> {
        match KnowledgeScope::from_key(key) {
            Some(KnowledgeScope::System) => {
                self.system_facts.get(key).map(|f| FactValue {
                    value: f.value.clone(),
                    confidence: f.confidence,
                    scope: KnowledgeScope::System,
                })
            }
            Some(KnowledgeScope::User) => {
                self.user_facts.get(key).map(|f| FactValue {
                    value: f.value.clone(),
                    confidence: f.confidence,
                    scope: KnowledgeScope::User,
                })
            }
            None => None,
        }
    }

    /// Check if a fact exists
    pub fn has_fact(&self, key: &str) -> bool {
        match KnowledgeScope::from_key(key) {
            Some(KnowledgeScope::System) => self.system_facts.contains_key(key),
            Some(KnowledgeScope::User) => self.user_facts.contains_key(key),
            None => false,
        }
    }

    /// Get all facts matching a prefix
    pub fn get_facts_by_prefix(&self, prefix: &str) -> Vec<FactValue> {
        let mut results = Vec::new();

        for (key, fact) in &self.system_facts {
            if key.starts_with(prefix) {
                results.push(FactValue {
                    value: fact.value.clone(),
                    confidence: fact.confidence,
                    scope: KnowledgeScope::System,
                });
            }
        }

        for (key, fact) in &self.user_facts {
            if key.starts_with(prefix) {
                results.push(FactValue {
                    value: fact.value.clone(),
                    confidence: fact.confidence,
                    scope: KnowledgeScope::User,
                });
            }
        }

        results
    }

    /// Count facts by scope
    pub fn count_facts(&self) -> (usize, usize) {
        (self.system_facts.len(), self.user_facts.len())
    }
}

/// A fact value with scope information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactValue {
    pub value: String,
    pub confidence: f32,
    pub scope: KnowledgeScope,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_scope_from_key() {
        assert_eq!(
            KnowledgeScope::from_key("system.hardware.cpu.model"),
            Some(KnowledgeScope::System)
        );
        assert_eq!(
            KnowledgeScope::from_key("user.editor.primary"),
            Some(KnowledgeScope::User)
        );
        assert_eq!(KnowledgeScope::from_key("invalid.key"), None);
    }

    #[test]
    fn test_system_identity() {
        let id = SystemIdentity::new("myhost".to_string(), "abc123def456".to_string());
        assert_eq!(id.storage_key(), "myhost_abc123de");
    }

    #[test]
    fn test_user_identity() {
        let id = UserIdentity::new("juanjo".to_string(), "myhost".to_string());
        assert_eq!(id.storage_key(), "juanjo@myhost");
    }

    #[test]
    fn test_dual_brain_get_fact() {
        let mut brain = DualBrain::new();

        brain.system_facts.insert(
            "system.hardware.cpu.cores".to_string(),
            SystemFact {
                key: "system.hardware.cpu.cores".to_string(),
                value: "8".to_string(),
                confidence: 1.0,
                source: FactSourceV23::Probe { probe_id: "lscpu".to_string() },
                recorded_at: 1234567890,
                expires_at: 0,
            },
        );

        let fact = brain.get_fact("system.hardware.cpu.cores");
        assert!(fact.is_some());
        assert_eq!(fact.unwrap().value, "8");

        assert!(brain.get_fact("user.editor.primary").is_none());
    }

    #[test]
    fn test_scope_prefix() {
        assert_eq!(KnowledgeScope::System.prefix(), "system");
        assert_eq!(KnowledgeScope::User.prefix(), "user");
    }
}
