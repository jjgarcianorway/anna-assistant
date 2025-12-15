// v0.0.708: Settings Memo (Phase 284)
// Internal memos for settings communication

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Memo type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MemoType {
    /// Internal memo
    #[default]
    Internal,
    /// External memo
    External,
    /// Confidential memo
    Confidential,
    /// Broadcast memo
    Broadcast,
}

impl std::fmt::Display for MemoType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Internal => write!(f, "internal"),
            Self::External => write!(f, "external"),
            Self::Confidential => write!(f, "confidential"),
            Self::Broadcast => write!(f, "broadcast"),
        }
    }
}

/// Memo status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MemoStatus {
    /// Draft
    #[default]
    Draft,
    /// Sent
    Sent,
    /// Read
    Read,
    /// Archived
    Archived,
}

impl std::fmt::Display for MemoStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Sent => write!(f, "sent"),
            Self::Read => write!(f, "read"),
            Self::Archived => write!(f, "archived"),
        }
    }
}

/// Memo config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoConfig {
    /// Name
    pub name: String,
    /// Memo type
    pub memo_type: MemoType,
    /// Max memos
    pub max_memos: usize,
    /// Require acknowledgment
    pub require_ack: bool,
}

impl MemoConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            memo_type: MemoType::Internal,
            max_memos: 500,
            require_ack: false,
        }
    }

    /// Set type
    pub fn memo_type(mut self, mt: MemoType) -> Self {
        self.memo_type = mt;
        self
    }

    /// Set max memos
    pub fn max_memos(mut self, max: usize) -> Self {
        self.max_memos = max;
        self
    }

    /// Set require acknowledgment
    pub fn require_ack(mut self, req: bool) -> Self {
        self.require_ack = req;
        self
    }
}

impl Default for MemoConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Memo message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoMessage {
    /// Message ID
    pub id: String,
    /// Subject
    pub subject: String,
    /// Body
    pub body: String,
    /// From
    pub from: String,
    /// To
    pub to: Vec<String>,
    /// Status
    pub status: MemoStatus,
    /// Date
    pub date: String,
}

impl MemoMessage {
    /// Create new message
    pub fn new(id: impl Into<String>, subject: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            subject: subject.into(),
            body: body.into(),
            from: String::new(),
            to: Vec::new(),
            status: MemoStatus::Draft,
            date: String::new(),
        }
    }

    /// Set from
    pub fn from(mut self, f: impl Into<String>) -> Self {
        self.from = f.into();
        self
    }

    /// Add to
    pub fn to(mut self, t: impl Into<String>) -> Self {
        self.to.push(t.into());
        self
    }

    /// Set date
    pub fn date(mut self, d: impl Into<String>) -> Self {
        self.date = d.into();
        self
    }

    /// Send memo
    pub fn send(&mut self) {
        self.status = MemoStatus::Sent;
    }

    /// Mark read
    pub fn mark_read(&mut self) {
        self.status = MemoStatus::Read;
    }
}

/// Memo attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoAttachment {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Message ID
    pub message_id: String,
}

impl MemoAttachment {
    /// Create new attachment
    pub fn new(key: impl Into<String>, value: impl Into<String>, message_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            message_id: message_id.into(),
        }
    }
}

/// Memo stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoStats {
    /// Total memos
    pub total_memos: usize,
    /// Sent memos
    pub sent_memos: usize,
    /// Read memos
    pub read_memos: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl MemoStats {
    /// Update from memos
    pub fn update(&mut self, messages: &[MemoMessage], memo_type: MemoType) {
        self.total_memos = messages.len();
        self.sent_memos = messages.iter().filter(|m| matches!(m.status, MemoStatus::Sent | MemoStatus::Read)).count();
        self.read_memos = messages.iter().filter(|m| m.status == MemoStatus::Read).count();
        *self.by_type.entry(memo_type.to_string()).or_insert(0) += 1;
    }

    /// Read rate
    pub fn read_rate(&self) -> f64 {
        if self.sent_memos == 0 { 0.0 } else { self.read_memos as f64 / self.sent_memos as f64 * 100.0 }
    }
}

/// Settings memo
#[derive(Debug, Clone, Default)]
pub struct SettingsMemo {
    /// Config
    config: MemoConfig,
    /// Messages
    messages: Vec<MemoMessage>,
    /// Attachments
    attachments: Vec<MemoAttachment>,
    /// Stats
    stats: MemoStats,
}

impl SettingsMemo {
    /// Create new memo system
    pub fn new(config: MemoConfig) -> Self {
        Self {
            config,
            messages: Vec::new(),
            attachments: Vec::new(),
            stats: MemoStats::default(),
        }
    }

    /// Add message
    pub fn add_message(&mut self, message: MemoMessage) -> bool {
        if self.messages.len() >= self.config.max_memos {
            return false;
        }
        self.messages.push(message);
        self.update_stats();
        true
    }

    /// Get message
    pub fn get_message(&self, id: &str) -> Option<&MemoMessage> {
        self.messages.iter().find(|m| m.id == id)
    }

    /// Get message mut
    pub fn get_message_mut(&mut self, id: &str) -> Option<&mut MemoMessage> {
        self.messages.iter_mut().find(|m| m.id == id)
    }

    /// Add attachment
    pub fn add_attachment(&mut self, attachment: MemoAttachment) {
        self.attachments.push(attachment);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.messages, self.config.memo_type);
    }

    /// Get stats
    pub fn stats(&self) -> &MemoStats {
        &self.stats
    }

    /// Message count
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }
}

/// Memo registry
#[derive(Debug, Clone, Default)]
pub struct MemoRegistry {
    /// Memos by ID
    memos: HashMap<String, SettingsMemo>,
}

impl MemoRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register memo
    pub fn register(&mut self, id: impl Into<String>, memo: SettingsMemo) {
        self.memos.insert(id.into(), memo);
    }

    /// Unregister memo
    pub fn unregister(&mut self, id: &str) -> bool {
        self.memos.remove(id).is_some()
    }

    /// Get memo
    pub fn get(&self, id: &str) -> Option<&SettingsMemo> {
        self.memos.get(id)
    }

    /// Get memo mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsMemo> {
        self.memos.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.memos.len()
    }
}

/// Format memo registry
pub fn format_memo_registry(registry: &MemoRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Memo Registry:\n");
    output.push_str(&format!("  Memos: {}\n", registry.count()));
    output
}

/// Check if query is about memo
pub fn is_memo_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings memo") || lower.contains("memo settings") || lower.contains("internal memo")
}

/// Fun fact about memo
pub fn memo_fun_fact() -> &'static str {
    "Anna's settings memo system facilitates internal configuration communication!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memo_type_display() {
        assert_eq!(format!("{}", MemoType::Internal), "internal");
        assert_eq!(format!("{}", MemoType::Confidential), "confidential");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", MemoStatus::Draft), "draft");
        assert_eq!(format!("{}", MemoStatus::Sent), "sent");
    }

    #[test]
    fn test_config_new() {
        let c = MemoConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = MemoConfig::new("test")
            .memo_type(MemoType::Confidential)
            .require_ack(true);
        assert_eq!(c.memo_type, MemoType::Confidential);
        assert!(c.require_ack);
    }

    #[test]
    fn test_message_new() {
        let m = MemoMessage::new("m1", "Subject", "Body");
        assert_eq!(m.id, "m1");
    }

    #[test]
    fn test_message_builder() {
        let m = MemoMessage::new("m1", "Subject", "Body")
            .from("sender")
            .to("recipient");
        assert_eq!(m.from, "sender");
        assert_eq!(m.to.len(), 1);
    }

    #[test]
    fn test_message_send() {
        let mut m = MemoMessage::new("m1", "Subject", "Body");
        m.send();
        assert_eq!(m.status, MemoStatus::Sent);
    }

    #[test]
    fn test_attachment_new() {
        let a = MemoAttachment::new("key", "value", "m1");
        assert_eq!(a.message_id, "m1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = MemoStats::default();
        let mut msg = MemoMessage::new("m1", "Subject", "Body");
        msg.send();
        s.update(&[msg], MemoType::Internal);
        assert_eq!(s.total_memos, 1);
        assert_eq!(s.sent_memos, 1);
    }

    #[test]
    fn test_memo_new() {
        let m = SettingsMemo::new(MemoConfig::default());
        assert_eq!(m.message_count(), 0);
    }

    #[test]
    fn test_memo_add_message() {
        let mut m = SettingsMemo::new(MemoConfig::default());
        m.add_message(MemoMessage::new("m1", "Subject", "Body"));
        assert_eq!(m.message_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = MemoRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = MemoRegistry::new();
        r.register("m1", SettingsMemo::new(MemoConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_memo_query() {
        assert!(is_memo_query("settings memo"));
        assert!(!is_memo_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = memo_fun_fact();
        assert!(fact.contains("memo"));
    }
}
