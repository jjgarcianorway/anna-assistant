// v0.0.718: Settings Directive Config (Phase 294)
// Configuration for directive systems

use serde::{Deserialize, Serialize};
use super::types::{DirectiveType, DirectiveAuthority};

/// Directive config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectiveConfig {
    /// Name
    pub name: String,
    /// Directive type
    pub directive_type: DirectiveType,
    /// Authority
    pub authority: DirectiveAuthority,
    /// Max directives
    pub max_directives: usize,
}

impl DirectiveConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            directive_type: DirectiveType::Mandatory,
            authority: DirectiveAuthority::System,
            max_directives: 150,
        }
    }

    /// Set type
    pub fn directive_type(mut self, dt: DirectiveType) -> Self {
        self.directive_type = dt;
        self
    }

    /// Set authority
    pub fn authority(mut self, a: DirectiveAuthority) -> Self {
        self.authority = a;
        self
    }

    /// Set max directives
    pub fn max_directives(mut self, max: usize) -> Self {
        self.max_directives = max;
        self
    }
}

impl Default for DirectiveConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
