// v0.0.636: Settings Listener (Phase 212)
// Main listener implementation

use serde::{Deserialize, Serialize};

use super::config::ListenerConfig;
use super::event::ReceivedEvent;
use super::types::ListenerState;

/// Settings listener
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsListener {
    /// ID
    pub id: String,
    /// Name
    pub name: String,
    /// Config
    pub config: ListenerConfig,
    /// State
    pub state: ListenerState,
    /// Created timestamp
    pub created_at: u64,
    /// Event buffer
    pub buffer: Vec<ReceivedEvent>,
}

impl SettingsListener {
    /// Create new listener
    pub fn new(id: impl Into<String>, name: impl Into<String>, config: ListenerConfig) -> Self {
        let auto_start = config.auto_start;
        Self {
            id: id.into(),
            name: name.into(),
            config,
            state: if auto_start {
                ListenerState::Listening
            } else {
                ListenerState::Idle
            },
            created_at: 0,
            buffer: Vec::new(),
        }
    }

    /// Set created timestamp
    pub fn created_at(mut self, ts: u64) -> Self {
        self.created_at = ts;
        self
    }

    /// Start listening
    pub fn start(&mut self) {
        if self.state != ListenerState::Stopped {
            self.state = ListenerState::Listening;
        }
    }

    /// Stop listening
    pub fn stop(&mut self) {
        self.state = ListenerState::Stopped;
    }

    /// Pause listening
    pub fn pause(&mut self) {
        if self.state == ListenerState::Listening {
            self.state = ListenerState::Paused;
        }
    }

    /// Resume listening
    pub fn resume(&mut self) {
        if self.state == ListenerState::Paused {
            self.state = ListenerState::Listening;
        }
    }

    /// Is listening
    pub fn is_listening(&self) -> bool {
        self.state == ListenerState::Listening
    }

    /// Receive event
    pub fn receive(&mut self, event: ReceivedEvent) -> bool {
        if self.buffer.len() < self.config.buffer_size && self.is_listening() {
            self.buffer.push(event);
            true
        } else {
            false
        }
    }

    /// Get next event
    pub fn next(&mut self) -> Option<ReceivedEvent> {
        if self.buffer.is_empty() {
            None
        } else {
            Some(self.buffer.remove(0))
        }
    }

    /// Buffer count
    pub fn buffer_count(&self) -> usize {
        self.buffer.len()
    }

    /// Clear buffer
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}
