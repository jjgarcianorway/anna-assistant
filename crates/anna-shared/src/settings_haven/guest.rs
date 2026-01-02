// v0.0.784: Settings Haven (Phase 360)
// Safe haven for settings protection - Guest module

use serde::{Deserialize, Serialize};

/// Haven guest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HavenGuest {
    /// Guest ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Room number
    pub room: u32,
    /// Comfortable
    pub comfortable: bool,
}

impl HavenGuest {
    /// Create new guest
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            room: 0,
            comfortable: true,
        }
    }

    /// Set room
    pub fn room(mut self, r: u32) -> Self {
        self.room = r;
        self
    }

    /// Make comfortable
    pub fn make_comfortable(&mut self) {
        self.comfortable = true;
    }

    /// Make restless
    pub fn make_restless(&mut self) {
        self.comfortable = false;
    }
}
