// v0.0.718: Settings Directive Orders (Phase 294)
// Directive orders and supplements

use serde::{Deserialize, Serialize};
use super::types::DirectiveAuthority;

/// Directive order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectiveOrder {
    /// Order ID
    pub id: String,
    /// Title
    pub title: String,
    /// Instructions
    pub instructions: String,
    /// Authority
    pub authority: DirectiveAuthority,
    /// Enforced
    pub enforced: bool,
}

impl DirectiveOrder {
    /// Create new order
    pub fn new(id: impl Into<String>, title: impl Into<String>, instructions: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            instructions: instructions.into(),
            authority: DirectiveAuthority::System,
            enforced: false,
        }
    }

    /// Set authority
    pub fn authority(mut self, a: DirectiveAuthority) -> Self {
        self.authority = a;
        self
    }

    /// Enforce directive
    pub fn enforce(&mut self) {
        self.enforced = true;
    }
}

/// Directive supplement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectiveSupplement {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Order ID
    pub order_id: String,
}

impl DirectiveSupplement {
    /// Create new supplement
    pub fn new(key: impl Into<String>, value: impl Into<String>, order_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            order_id: order_id.into(),
        }
    }
}
