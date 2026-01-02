// v0.0.715: Settings Communique - Core (Phase 291)
// Main communique system

use super::config::CommuniqueConfig;
use super::message::{CommuniqueMessage, CommuniqueAttachment};
use super::stats::CommuniqueStats;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
