// v0.0.752: Settings Ward Motion (Phase 328)
// Ward motion and delegate types

use serde::{Deserialize, Serialize};

/// Ward motion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WardMotion {
    /// Motion ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Precinct number
    pub precinct: u32,
    /// Passed
    pub passed: bool,
}

impl WardMotion {
    /// Create new motion
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            precinct: 0,
            passed: true,
        }
    }

    /// Set precinct
    pub fn precinct(mut self, p: u32) -> Self {
        self.precinct = p;
        self
    }

    /// Make passed
    pub fn make_passed(&mut self) {
        self.passed = true;
    }

    /// Make failed
    pub fn make_failed(&mut self) {
        self.passed = false;
    }
}

/// Ward delegate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WardDelegate {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Motion ID
    pub motion_id: String,
}

impl WardDelegate {
    /// Create new delegate
    pub fn new(key: impl Into<String>, name: impl Into<String>, motion_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            motion_id: motion_id.into(),
        }
    }
}
