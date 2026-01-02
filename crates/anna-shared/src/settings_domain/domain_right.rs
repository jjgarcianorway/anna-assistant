// v0.0.743: Settings Domain - Domain Rights (Phase 319)
// Rights and holders for domain management

use serde::{Deserialize, Serialize};

/// Domain right
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainRight {
    /// Right ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Priority level
    pub priority: u32,
    /// Exclusive
    pub exclusive: bool,
}

impl DomainRight {
    /// Create new right
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            priority: 0,
            exclusive: true,
        }
    }

    /// Set priority
    pub fn priority(mut self, p: u32) -> Self {
        self.priority = p;
        self
    }

    /// Make exclusive
    pub fn make_exclusive(&mut self) {
        self.exclusive = true;
    }

    /// Make shared
    pub fn make_shared(&mut self) {
        self.exclusive = false;
    }
}

/// Domain holder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainHolder {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Right ID
    pub right_id: String,
}

impl DomainHolder {
    /// Create new holder
    pub fn new(key: impl Into<String>, name: impl Into<String>, right_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            right_id: right_id.into(),
        }
    }
}
