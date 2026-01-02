// v0.0.748: Settings District Bylaw (Phase 324)
// District bylaw management

use serde::{Deserialize, Serialize};

/// District bylaw
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistrictBylaw {
    /// Bylaw ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Ward number
    pub ward: u32,
    /// Active
    pub active: bool,
}

impl DistrictBylaw {
    /// Create new bylaw
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            ward: 0,
            active: true,
        }
    }

    /// Set ward
    pub fn ward(mut self, w: u32) -> Self {
        self.ward = w;
        self
    }

    /// Make active
    pub fn make_active(&mut self) {
        self.active = true;
    }

    /// Make inactive
    pub fn make_inactive(&mut self) {
        self.active = false;
    }
}
