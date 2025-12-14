// v0.0.594: Settings Encryption (Phase 170)
// Encryption for sensitive settings

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::unified_settings::SettingsCategory;

/// Encryption algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    /// AES-256-GCM
    #[default]
    Aes256Gcm,
    /// ChaCha20-Poly1305
    ChaCha20,
    /// XChaCha20-Poly1305
    XChaCha20,
}

impl std::fmt::Display for EncryptionAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Aes256Gcm => write!(f, "AES-256-GCM"),
            Self::ChaCha20 => write!(f, "ChaCha20-Poly1305"),
            Self::XChaCha20 => write!(f, "XChaCha20-Poly1305"),
        }
    }
}

/// Encryption status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum EncryptionStatus {
    /// Not encrypted
    #[default]
    Plain,
    /// Encrypted
    Encrypted,
    /// Decrypted (in memory)
    Decrypted,
    /// Error state
    Error,
}

impl std::fmt::Display for EncryptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Plain => write!(f, "plain"),
            Self::Encrypted => write!(f, "encrypted"),
            Self::Decrypted => write!(f, "decrypted"),
            Self::Error => write!(f, "error"),
        }
    }
}

/// Encrypted value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedValue {
    /// Category
    pub category: SettingsCategory,
    /// Key path
    pub key: String,
    /// Encrypted data (base64)
    pub ciphertext: String,
    /// Algorithm used
    pub algorithm: EncryptionAlgorithm,
    /// Status
    pub status: EncryptionStatus,
    /// Encrypted at
    pub encrypted_at: chrono::DateTime<chrono::Utc>,
    /// Nonce (base64)
    pub nonce: String,
}

impl EncryptedValue {
    /// Create new encrypted value
    pub fn new(category: SettingsCategory, key: impl Into<String>, ciphertext: impl Into<String>) -> Self {
        Self {
            category,
            key: key.into(),
            ciphertext: ciphertext.into(),
            algorithm: EncryptionAlgorithm::default(),
            status: EncryptionStatus::Encrypted,
            encrypted_at: chrono::Utc::now(),
            nonce: uuid::Uuid::new_v4().to_string(),
        }
    }

    /// Set algorithm
    pub fn algorithm(mut self, algo: EncryptionAlgorithm) -> Self {
        self.algorithm = algo;
        self
    }

    /// Check if encrypted
    pub fn is_encrypted(&self) -> bool {
        self.status == EncryptionStatus::Encrypted
    }
}

/// Encryption key info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyInfo {
    /// Key ID
    pub id: String,
    /// Key name
    pub name: String,
    /// Algorithm
    pub algorithm: EncryptionAlgorithm,
    /// Created at
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Expires at
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Active
    pub active: bool,
}

impl KeyInfo {
    /// Create new key info
    pub fn new(name: impl Into<String>, algorithm: EncryptionAlgorithm) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            algorithm,
            created_at: chrono::Utc::now(),
            expires_at: None,
            active: true,
        }
    }

    /// Set expiration
    pub fn expires_in(mut self, days: i64) -> Self {
        self.expires_at = Some(chrono::Utc::now() + chrono::Duration::days(days));
        self
    }

    /// Check if expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires_at {
            chrono::Utc::now() > expires
        } else {
            false
        }
    }
}

/// Encryption manager
#[derive(Debug, Clone, Default)]
pub struct EncryptionManager {
    /// Encrypted values
    values: Vec<EncryptedValue>,
    /// Available keys
    keys: Vec<KeyInfo>,
    /// Active key ID
    active_key: Option<String>,
    /// Categories requiring encryption
    required: HashSet<SettingsCategory>,
}

impl EncryptionManager {
    /// Create new manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Add encryption key
    pub fn add_key(&mut self, key: KeyInfo) {
        if self.active_key.is_none() {
            self.active_key = Some(key.id.clone());
        }
        self.keys.push(key);
    }

    /// Set active key
    pub fn set_active_key(&mut self, id: &str) -> bool {
        if self.keys.iter().any(|k| k.id == id && k.active) {
            self.active_key = Some(id.to_string());
            return true;
        }
        false
    }

    /// Get active key
    pub fn active_key(&self) -> Option<&KeyInfo> {
        self.active_key.as_ref().and_then(|id| self.keys.iter().find(|k| k.id == *id))
    }

    /// Store encrypted value
    pub fn store(&mut self, value: EncryptedValue) {
        self.values.push(value);
    }

    /// Get encrypted value
    pub fn get(&self, category: SettingsCategory, key: &str) -> Option<&EncryptedValue> {
        self.values.iter().find(|v| v.category == category && v.key == key)
    }

    /// Remove encrypted value
    pub fn remove(&mut self, category: SettingsCategory, key: &str) -> bool {
        let len = self.values.len();
        self.values.retain(|v| !(v.category == category && v.key == key));
        self.values.len() < len
    }

    /// Require encryption for category
    pub fn require(&mut self, category: SettingsCategory) {
        self.required.insert(category);
    }

    /// Check if encryption required
    pub fn is_required(&self, category: SettingsCategory) -> bool {
        self.required.contains(&category)
    }

    /// Check if value is encrypted
    pub fn is_encrypted(&self, category: SettingsCategory, key: &str) -> bool {
        self.values.iter().any(|v| v.category == category && v.key == key && v.is_encrypted())
    }

    /// List encrypted values
    pub fn all(&self) -> &[EncryptedValue] {
        &self.values
    }

    /// List keys
    pub fn keys(&self) -> &[KeyInfo] {
        &self.keys
    }

    /// Encrypted count
    pub fn count(&self) -> usize {
        self.values.len()
    }

    /// Key count
    pub fn key_count(&self) -> usize {
        self.keys.len()
    }

    /// Clear all
    pub fn clear(&mut self) {
        self.values.clear();
    }
}

/// Format encryption status
pub fn format_encryption(manager: &EncryptionManager) -> String {
    let mut output = String::new();

    output.push_str("=== Settings Encryption ===\n\n");
    output.push_str(&format!("Keys: {} | Encrypted values: {}\n", manager.key_count(), manager.count()));

    if let Some(key) = manager.active_key() {
        output.push_str(&format!("Active key: {} ({})\n", key.name, key.algorithm));
    }

    output.push_str(&format!("\nRequired categories: {}\n", manager.required.len()));

    output
}

/// Check if query is about encryption
pub fn is_encryption_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("encrypt")
        || lower.contains("decrypt")
        || lower.contains("secure")
        || lower.contains("sensitive")
}

/// Fun fact about encryption
pub fn settings_encryption_fun_fact() -> &'static str {
    "Anna can encrypt sensitive settings to protect your private data!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_algorithm_display() {
        assert_eq!(format!("{}", EncryptionAlgorithm::Aes256Gcm), "AES-256-GCM");
        assert_eq!(format!("{}", EncryptionAlgorithm::ChaCha20), "ChaCha20-Poly1305");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", EncryptionStatus::Encrypted), "encrypted");
        assert_eq!(format!("{}", EncryptionStatus::Plain), "plain");
    }

    #[test]
    fn test_encrypted_value_new() {
        let val = EncryptedValue::new(SettingsCategory::Privacy, "api_key", "encrypted_data");
        assert!(val.is_encrypted());
    }

    #[test]
    fn test_key_info_new() {
        let key = KeyInfo::new("main", EncryptionAlgorithm::Aes256Gcm);
        assert!(key.active);
        assert!(!key.is_expired());
    }

    #[test]
    fn test_manager_new() {
        let manager = EncryptionManager::new();
        assert_eq!(manager.count(), 0);
        assert_eq!(manager.key_count(), 0);
    }

    #[test]
    fn test_manager_add_key() {
        let mut manager = EncryptionManager::new();
        manager.add_key(KeyInfo::new("main", EncryptionAlgorithm::Aes256Gcm));
        assert_eq!(manager.key_count(), 1);
        assert!(manager.active_key().is_some());
    }

    #[test]
    fn test_manager_store() {
        let mut manager = EncryptionManager::new();
        manager.store(EncryptedValue::new(SettingsCategory::Privacy, "key", "data"));
        assert_eq!(manager.count(), 1);
    }

    #[test]
    fn test_manager_get() {
        let mut manager = EncryptionManager::new();
        manager.store(EncryptedValue::new(SettingsCategory::Privacy, "key", "data"));
        assert!(manager.get(SettingsCategory::Privacy, "key").is_some());
    }

    #[test]
    fn test_manager_require() {
        let mut manager = EncryptionManager::new();
        manager.require(SettingsCategory::Privacy);
        assert!(manager.is_required(SettingsCategory::Privacy));
    }

    #[test]
    fn test_manager_is_encrypted() {
        let mut manager = EncryptionManager::new();
        manager.store(EncryptedValue::new(SettingsCategory::Privacy, "key", "data"));
        assert!(manager.is_encrypted(SettingsCategory::Privacy, "key"));
    }

    #[test]
    fn test_format_encryption() {
        let manager = EncryptionManager::new();
        let output = format_encryption(&manager);
        assert!(output.contains("Encryption"));
    }

    #[test]
    fn test_is_encryption_query() {
        assert!(is_encryption_query("encrypt settings"));
        assert!(is_encryption_query("secure data"));
        assert!(!is_encryption_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_encryption_fun_fact();
        assert!(fact.contains("encrypt"));
    }
}
