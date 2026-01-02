// v0.0.714: Settings Dispatch Config (Phase 290)
// Configuration for dispatch operations

use serde::{Deserialize, Serialize};
use super::types::DispatchType;

/// Dispatch config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchConfig {
    /// Name
    pub name: String,
    /// Dispatch type
    pub dispatch_type: DispatchType,
    /// Retry count
    pub retry_count: usize,
    /// Max dispatches
    pub max_dispatches: usize,
}

impl DispatchConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            dispatch_type: DispatchType::Immediate,
            retry_count: 3,
            max_dispatches: 500,
        }
    }

    /// Set type
    pub fn dispatch_type(mut self, dt: DispatchType) -> Self {
        self.dispatch_type = dt;
        self
    }

    /// Set retry count
    pub fn retry_count(mut self, rc: usize) -> Self {
        self.retry_count = rc;
        self
    }

    /// Set max dispatches
    pub fn max_dispatches(mut self, max: usize) -> Self {
        self.max_dispatches = max;
        self
    }
}

impl Default for DispatchConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = DispatchConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = DispatchConfig::new("test")
            .dispatch_type(DispatchType::Scheduled)
            .retry_count(5);
        assert_eq!(c.dispatch_type, DispatchType::Scheduled);
        assert_eq!(c.retry_count, 5);
    }
}
