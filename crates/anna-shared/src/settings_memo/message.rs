// v0.0.708: Settings Memo (Phase 284)
// Memo message and attachment

use serde::{Deserialize, Serialize};
use super::types::MemoStatus;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
