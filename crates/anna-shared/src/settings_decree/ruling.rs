// v0.0.720: Settings Decree - Ruling (Phase 296)
// Decree rulings and clauses

use serde::{Deserialize, Serialize};
use super::types::DecreeBinding;

/// Decree ruling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecreeRuling {
    /// Ruling ID
    pub id: String,
    /// Title
    pub title: String,
    /// Text
    pub text: String,
    /// Binding
    pub binding: DecreeBinding,
    /// In force
    pub in_force: bool,
}

impl DecreeRuling {
    /// Create new ruling
    pub fn new(id: impl Into<String>, title: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            text: text.into(),
            binding: DecreeBinding::Mandatory,
            in_force: false,
        }
    }

    /// Set binding
    pub fn binding(mut self, b: DecreeBinding) -> Self {
        self.binding = b;
        self
    }

    /// Put in force
    pub fn enact(&mut self) {
        self.in_force = true;
    }

    /// Remove from force
    pub fn repeal(&mut self) {
        self.in_force = false;
    }
}

/// Decree clause
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecreeClause {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Ruling ID
    pub ruling_id: String,
}

impl DecreeClause {
    /// Create new clause
    pub fn new(key: impl Into<String>, value: impl Into<String>, ruling_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            ruling_id: ruling_id.into(),
        }
    }
}
