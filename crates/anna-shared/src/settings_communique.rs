// v0.0.715: Settings Communique (Phase 291)
// Official communications about settings

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Communique type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CommuniqueType {
    /// Official communique
    #[default]
    Official,
    /// Informal communique
    Informal,
    /// Urgent communique
    Urgent,
    /// Diplomatic communique
    Diplomatic,
}

impl std::fmt::Display for CommuniqueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Official => write!(f, "official"),
            Self::Informal => write!(f, "informal"),
            Self::Urgent => write!(f, "urgent"),
            Self::Diplomatic => write!(f, "diplomatic"),
        }
    }
}

/// Communique classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CommuniqueClassification {
    /// Public
    #[default]
    Public,
    /// Internal
    Internal,
    /// Confidential
    Confidential,
    /// Restricted
    Restricted,
}

impl std::fmt::Display for CommuniqueClassification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Public => write!(f, "public"),
            Self::Internal => write!(f, "internal"),
            Self::Confidential => write!(f, "confidential"),
            Self::Restricted => write!(f, "restricted"),
        }
    }
}

/// Communique config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommuniqueConfig {
    /// Name
    pub name: String,
    /// Communique type
    pub communique_type: CommuniqueType,
    /// Classification
    pub classification: CommuniqueClassification,
    /// Max messages
    pub max_messages: usize,
}

impl CommuniqueConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            communique_type: CommuniqueType::Official,
            classification: CommuniqueClassification::Public,
            max_messages: 300,
        }
    }

    /// Set type
    pub fn communique_type(mut self, ct: CommuniqueType) -> Self {
        self.communique_type = ct;
        self
    }

    /// Set classification
    pub fn classification(mut self, c: CommuniqueClassification) -> Self {
        self.classification = c;
        self
    }

    /// Set max messages
    pub fn max_messages(mut self, max: usize) -> Self {
        self.max_messages = max;
        self
    }
}

impl Default for CommuniqueConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Communique message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommuniqueMessage {
    /// Message ID
    pub id: String,
    /// Subject
    pub subject: String,
    /// Body
    pub body: String,
    /// Sender
    pub sender: String,
    /// Recipients
    pub recipients: Vec<String>,
    /// Read
    pub read: bool,
}

impl CommuniqueMessage {
    /// Create new message
    pub fn new(id: impl Into<String>, subject: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            subject: subject.into(),
            body: body.into(),
            sender: String::new(),
            recipients: Vec::new(),
            read: false,
        }
    }

    /// Set sender
    pub fn sender(mut self, s: impl Into<String>) -> Self {
        self.sender = s.into();
        self
    }

    /// Add recipient
    pub fn recipient(mut self, r: impl Into<String>) -> Self {
        self.recipients.push(r.into());
        self
    }

    /// Mark read
    pub fn mark_read(&mut self) {
        self.read = true;
    }
}

/// Communique attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommuniqueAttachment {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Message ID
    pub message_id: String,
}

impl CommuniqueAttachment {
    /// Create new attachment
    pub fn new(key: impl Into<String>, value: impl Into<String>, message_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            message_id: message_id.into(),
        }
    }
}

/// Communique stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommuniqueStats {
    /// Total messages
    pub total_messages: usize,
    /// Read messages
    pub read_messages: usize,
    /// Urgent messages
    pub urgent_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl CommuniqueStats {
    /// Update from messages
    pub fn update(&mut self, messages: &[CommuniqueMessage], communique_type: CommuniqueType) {
        self.total_messages = messages.len();
        self.read_messages = messages.iter().filter(|m| m.read).count();
        if communique_type == CommuniqueType::Urgent {
            self.urgent_count = messages.len();
        }
        *self.by_type.entry(communique_type.to_string()).or_insert(0) += 1;
    }

    /// Read rate
    pub fn read_rate(&self) -> f64 {
        if self.total_messages == 0 { 0.0 } else { self.read_messages as f64 / self.total_messages as f64 * 100.0 }
    }
}

/// Settings communique
#[derive(Debug, Clone, Default)]
pub struct SettingsCommunique {
    /// Config
    config: CommuniqueConfig,
    /// Messages
    messages: Vec<CommuniqueMessage>,
    /// Attachments
    attachments: Vec<CommuniqueAttachment>,
    /// Stats
    stats: CommuniqueStats,
}

impl SettingsCommunique {
    /// Create new communique system
    pub fn new(config: CommuniqueConfig) -> Self {
        Self {
            config,
            messages: Vec::new(),
            attachments: Vec::new(),
            stats: CommuniqueStats::default(),
        }
    }

    /// Add message
    pub fn add_message(&mut self, message: CommuniqueMessage) -> bool {
        if self.messages.len() >= self.config.max_messages {
            return false;
        }
        self.messages.push(message);
        self.update_stats();
        true
    }

    /// Get message
    pub fn get_message(&self, id: &str) -> Option<&CommuniqueMessage> {
        self.messages.iter().find(|m| m.id == id)
    }

    /// Get message mut
    pub fn get_message_mut(&mut self, id: &str) -> Option<&mut CommuniqueMessage> {
        self.messages.iter_mut().find(|m| m.id == id)
    }

    /// Add attachment
    pub fn add_attachment(&mut self, attachment: CommuniqueAttachment) {
        self.attachments.push(attachment);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.messages, self.config.communique_type);
    }

    /// Get stats
    pub fn stats(&self) -> &CommuniqueStats {
        &self.stats
    }

    /// Message count
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }
}

/// Communique registry
#[derive(Debug, Clone, Default)]
pub struct CommuniqueRegistry {
    /// Communiques by ID
    communiques: HashMap<String, SettingsCommunique>,
}

impl CommuniqueRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register communique
    pub fn register(&mut self, id: impl Into<String>, communique: SettingsCommunique) {
        self.communiques.insert(id.into(), communique);
    }

    /// Unregister communique
    pub fn unregister(&mut self, id: &str) -> bool {
        self.communiques.remove(id).is_some()
    }

    /// Get communique
    pub fn get(&self, id: &str) -> Option<&SettingsCommunique> {
        self.communiques.get(id)
    }

    /// Get communique mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsCommunique> {
        self.communiques.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.communiques.len()
    }
}

/// Format communique registry
pub fn format_communique_registry(registry: &CommuniqueRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Communique Registry:\n");
    output.push_str(&format!("  Communiques: {}\n", registry.count()));
    output
}

/// Check if query is about communique
pub fn is_communique_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings communique") || lower.contains("communique settings") || lower.contains("official communication")
}

/// Fun fact about communique
pub fn communique_fun_fact() -> &'static str {
    "Anna's settings communique delivers official communications about configuration changes!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_communique_type_display() {
        assert_eq!(format!("{}", CommuniqueType::Official), "official");
        assert_eq!(format!("{}", CommuniqueType::Urgent), "urgent");
    }

    #[test]
    fn test_classification_display() {
        assert_eq!(format!("{}", CommuniqueClassification::Public), "public");
        assert_eq!(format!("{}", CommuniqueClassification::Confidential), "confidential");
    }

    #[test]
    fn test_config_new() {
        let c = CommuniqueConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = CommuniqueConfig::new("test")
            .communique_type(CommuniqueType::Diplomatic)
            .classification(CommuniqueClassification::Restricted);
        assert_eq!(c.communique_type, CommuniqueType::Diplomatic);
        assert_eq!(c.classification, CommuniqueClassification::Restricted);
    }

    #[test]
    fn test_message_new() {
        let m = CommuniqueMessage::new("m1", "Subject", "Body");
        assert_eq!(m.id, "m1");
    }

    #[test]
    fn test_message_builder() {
        let m = CommuniqueMessage::new("m1", "Subject", "Body")
            .sender("sender")
            .recipient("recipient");
        assert_eq!(m.sender, "sender");
        assert_eq!(m.recipients.len(), 1);
    }

    #[test]
    fn test_message_mark_read() {
        let mut m = CommuniqueMessage::new("m1", "Subject", "Body");
        m.mark_read();
        assert!(m.read);
    }

    #[test]
    fn test_attachment_new() {
        let a = CommuniqueAttachment::new("key", "value", "m1");
        assert_eq!(a.message_id, "m1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = CommuniqueStats::default();
        let mut msg = CommuniqueMessage::new("m1", "Subject", "Body");
        msg.mark_read();
        s.update(&[msg], CommuniqueType::Official);
        assert_eq!(s.total_messages, 1);
        assert_eq!(s.read_messages, 1);
    }

    #[test]
    fn test_communique_new() {
        let c = SettingsCommunique::new(CommuniqueConfig::default());
        assert_eq!(c.message_count(), 0);
    }

    #[test]
    fn test_communique_add_message() {
        let mut c = SettingsCommunique::new(CommuniqueConfig::default());
        c.add_message(CommuniqueMessage::new("m1", "Subject", "Body"));
        assert_eq!(c.message_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = CommuniqueRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = CommuniqueRegistry::new();
        r.register("c1", SettingsCommunique::new(CommuniqueConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_communique_query() {
        assert!(is_communique_query("settings communique"));
        assert!(!is_communique_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = communique_fun_fact();
        assert!(fact.contains("communique"));
    }
}
