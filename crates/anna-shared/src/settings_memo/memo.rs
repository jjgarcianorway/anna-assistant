// v0.0.708: Settings Memo (Phase 284)
// Settings memo main implementation

use super::config::MemoConfig;
use super::message::{MemoMessage, MemoAttachment};
use super::stats::MemoStats;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
