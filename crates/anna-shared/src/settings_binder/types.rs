// v0.0.652: Settings Binder - Types
// Core types for binding system

use serde::{Deserialize, Serialize};

/// Binding type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BindingType {
    /// One-way binding
    #[default]
    OneWay,
    /// Two-way binding
    TwoWay,
    /// One-time binding
    OneTime,
    /// Lazy binding
    Lazy,
    /// Eager binding
    Eager,
}

impl std::fmt::Display for BindingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OneWay => write!(f, "one_way"),
            Self::TwoWay => write!(f, "two_way"),
            Self::OneTime => write!(f, "one_time"),
            Self::Lazy => write!(f, "lazy"),
            Self::Eager => write!(f, "eager"),
        }
    }
}

/// Binding state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BindingState {
    /// Unbound
    #[default]
    Unbound,
    /// Bound
    Bound,
    /// Pending
    Pending,
    /// Error
    Error,
}

impl std::fmt::Display for BindingState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unbound => write!(f, "unbound"),
            Self::Bound => write!(f, "bound"),
            Self::Pending => write!(f, "pending"),
            Self::Error => write!(f, "error"),
        }
    }
}
