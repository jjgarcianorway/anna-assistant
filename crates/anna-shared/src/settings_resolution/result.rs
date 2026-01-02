// v0.0.664: Settings Resolution Result
// Result and request types

use serde::{Deserialize, Serialize};
use super::types::{ResolutionStatus, ResolutionStrategy};

/// Resolution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionResult {
    /// Key resolved
    pub key: String,
    /// Resolved value
    pub value: Option<String>,
    /// Status
    pub status: ResolutionStatus,
    /// Strategy used
    pub strategy: ResolutionStrategy,
    /// Depth of resolution
    pub depth: usize,
    /// Error message
    pub error: Option<String>,
}

impl ResolutionResult {
    /// Create success result
    pub fn success(key: impl Into<String>, value: impl Into<String>, strategy: ResolutionStrategy) -> Self {
        Self {
            key: key.into(),
            value: Some(value.into()),
            status: ResolutionStatus::Resolved,
            strategy,
            depth: 0,
            error: None,
        }
    }

    /// Create failure result
    pub fn failure(key: impl Into<String>, status: ResolutionStatus, error: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: None,
            status,
            strategy: ResolutionStrategy::Direct,
            depth: 0,
            error: Some(error.into()),
        }
    }

    /// With depth
    pub fn with_depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }

    /// Is resolved
    pub fn is_resolved(&self) -> bool {
        self.status == ResolutionStatus::Resolved
    }

    /// Is failed
    pub fn is_failed(&self) -> bool {
        matches!(self.status, ResolutionStatus::Failed | ResolutionStatus::Circular | ResolutionStatus::NotFound)
    }
}

/// Resolution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionRequest {
    /// Key to resolve
    pub key: String,
    /// Strategy to use
    pub strategy: Option<ResolutionStrategy>,
    /// Default value
    pub default_value: Option<String>,
}

impl ResolutionRequest {
    /// Create new request
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            strategy: None,
            default_value: None,
        }
    }

    /// With strategy
    pub fn with_strategy(mut self, strategy: ResolutionStrategy) -> Self {
        self.strategy = Some(strategy);
        self
    }

    /// With default
    pub fn with_default(mut self, value: impl Into<String>) -> Self {
        self.default_value = Some(value.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_success() {
        let r = ResolutionResult::success("key", "value", ResolutionStrategy::Direct);
        assert!(r.is_resolved());
        assert!(!r.is_failed());
    }

    #[test]
    fn test_result_failure() {
        let r = ResolutionResult::failure("key", ResolutionStatus::NotFound, "not found");
        assert!(!r.is_resolved());
        assert!(r.is_failed());
    }

    #[test]
    fn test_request_new() {
        let r = ResolutionRequest::new("key");
        assert_eq!(r.key, "key");
    }

    #[test]
    fn test_request_with_default() {
        let r = ResolutionRequest::new("key").with_default("default");
        assert_eq!(r.default_value, Some("default".to_string()));
    }
}
