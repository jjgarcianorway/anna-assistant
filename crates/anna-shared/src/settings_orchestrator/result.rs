// v0.0.574: Operation Result Types
// Result types for orchestrator operations

/// Operation result
#[derive(Debug, Clone)]
pub struct OperationResult {
    /// Was successful
    pub success: bool,
    /// Message
    pub message: String,
    /// Warnings
    pub warnings: Vec<String>,
    /// Errors
    pub errors: Vec<String>,
}

impl OperationResult {
    /// Create success result
    pub fn ok(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Create error result
    pub fn err(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Add warning
    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }

    /// Add error
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.errors.push(error.into());
        self.success = false;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_result_ok() {
        let result = OperationResult::ok("Success");
        assert!(result.success);
    }

    #[test]
    fn test_operation_result_err() {
        let result = OperationResult::err("Failed");
        assert!(!result.success);
    }

    #[test]
    fn test_operation_result_with_warning() {
        let result = OperationResult::ok("Done").with_warning("Check this");
        assert!(result.success);
        assert_eq!(result.warnings.len(), 1);
    }
}
