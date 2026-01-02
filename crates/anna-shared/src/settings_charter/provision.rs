// v0.0.724: Settings Charter - Provision module
// Charter provisions and amendments

use serde::{Deserialize, Serialize};

/// Charter provision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharterProvision {
    /// Provision ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Section number
    pub section: String,
    /// Active
    pub active: bool,
}

impl CharterProvision {
    /// Create new provision
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            section: String::new(),
            active: true,
        }
    }

    /// Set section
    pub fn section(mut self, s: impl Into<String>) -> Self {
        self.section = s.into();
        self
    }

    /// Activate provision
    pub fn activate(&mut self) {
        self.active = true;
    }

    /// Deactivate provision
    pub fn deactivate(&mut self) {
        self.active = false;
    }
}

/// Charter amendment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharterAmendment {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Provision ID
    pub provision_id: String,
}

impl CharterAmendment {
    /// Create new amendment
    pub fn new(key: impl Into<String>, value: impl Into<String>, provision_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            provision_id: provision_id.into(),
        }
    }
}
