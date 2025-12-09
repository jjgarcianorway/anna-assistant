//! Transcript container struct (v0.0.178).

use serde::{Deserialize, Serialize};

use crate::resource_limits::{ResourceDiagnostic, MAX_TRANSCRIPT_EVENTS};

use super::TranscriptEvent;

/// Full transcript for a request
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Transcript {
    /// All events in chronological order
    pub events: Vec<TranscriptEvent>,
    /// Number of events dropped due to cap (not serialized for wire compat)
    #[serde(skip)]
    dropped_events: usize,
}

impl Transcript {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            dropped_events: 0,
        }
    }

    /// Push event, enforcing cap. Returns true if event was added.
    /// COST: Never silently truncate - track dropped count for diagnostic.
    pub fn push(&mut self, event: TranscriptEvent) -> bool {
        if self.events.len() >= MAX_TRANSCRIPT_EVENTS {
            self.dropped_events += 1;
            false
        } else {
            self.events.push(event);
            true
        }
    }

    /// Check if transcript was capped (events were dropped)
    pub fn was_capped(&self) -> bool {
        self.dropped_events > 0
    }

    /// Get number of dropped events
    pub fn dropped_count(&self) -> usize {
        self.dropped_events
    }

    /// Get resource diagnostic if capped
    pub fn diagnostic(&self) -> Option<ResourceDiagnostic> {
        if self.dropped_events > 0 {
            Some(ResourceDiagnostic::transcript_capped(self.dropped_events))
        } else {
            None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }
}
