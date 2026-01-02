// v0.0.715: Settings Communique - Message (Phase 291)
// Communique messages and attachments

use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
