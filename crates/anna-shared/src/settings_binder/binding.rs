// v0.0.652: Settings Binder - Binding
// Binding definitions and results

use serde::{Deserialize, Serialize};
use super::types::{BindingType, BindingState};

/// Binding definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindingDef {
    /// Source key
    pub source: String,
    /// Target path
    pub target: String,
    /// Binding type
    pub binding_type: BindingType,
    /// Current state
    pub state: BindingState,
}

impl BindingDef {
    /// Create new binding
    pub fn new(source: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
            binding_type: BindingType::OneWay,
            state: BindingState::Unbound,
        }
    }

    /// Set binding type
    pub fn binding_type(mut self, binding_type: BindingType) -> Self {
        self.binding_type = binding_type;
        self
    }

    /// Is bound
    pub fn is_bound(&self) -> bool {
        self.state == BindingState::Bound
    }
}

/// Binding result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindingResult {
    /// Source key
    pub source: String,
    /// Target path
    pub target: String,
    /// Was successful
    pub success: bool,
    /// State after binding
    pub state: BindingState,
    /// Error message if failed
    pub error: Option<String>,
}

impl BindingResult {
    /// Create success result
    pub fn success(source: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
            success: true,
            state: BindingState::Bound,
            error: None,
        }
    }

    /// Create failure result
    pub fn failure(source: impl Into<String>, target: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
            success: false,
            state: BindingState::Error,
            error: Some(error.into()),
        }
    }
}
