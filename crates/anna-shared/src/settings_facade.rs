// v0.0.627: Settings Facade (Phase 203)
// Simplified facade for complex settings subsystem

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;

/// Facade operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FacadeOperation {
    /// Get setting
    #[default]
    Get,
    /// Set setting
    Set,
    /// Reset setting
    Reset,
    /// List settings
    List,
    /// Validate settings
    Validate,
}

impl std::fmt::Display for FacadeOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Get => write!(f, "get"),
            Self::Set => write!(f, "set"),
            Self::Reset => write!(f, "reset"),
            Self::List => write!(f, "list"),
            Self::Validate => write!(f, "validate"),
        }
    }
}

/// Facade result
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FacadeResult {
    /// Success
    #[default]
    Success,
    /// Not found
    NotFound,
    /// Invalid value
    InvalidValue,
    /// Permission denied
    PermissionDenied,
    /// System error
    SystemError,
}

impl std::fmt::Display for FacadeResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "success"),
            Self::NotFound => write!(f, "not_found"),
            Self::InvalidValue => write!(f, "invalid_value"),
            Self::PermissionDenied => write!(f, "permission_denied"),
            Self::SystemError => write!(f, "system_error"),
        }
    }
}

/// Facade request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacadeRequest {
    /// Operation
    pub operation: FacadeOperation,
    /// Key
    pub key: Option<String>,
    /// Value
    pub value: Option<String>,
    /// Category
    pub category: Option<SettingsCategory>,
}

impl FacadeRequest {
    /// Create get request
    pub fn get(key: impl Into<String>) -> Self {
        Self {
            operation: FacadeOperation::Get,
            key: Some(key.into()),
            value: None,
            category: None,
        }
    }

    /// Create set request
    pub fn set(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            operation: FacadeOperation::Set,
            key: Some(key.into()),
            value: Some(value.into()),
            category: None,
        }
    }

    /// Create reset request
    pub fn reset(key: impl Into<String>) -> Self {
        Self {
            operation: FacadeOperation::Reset,
            key: Some(key.into()),
            value: None,
            category: None,
        }
    }

    /// Create list request
    pub fn list(category: SettingsCategory) -> Self {
        Self {
            operation: FacadeOperation::List,
            key: None,
            value: None,
            category: Some(category),
        }
    }

    /// Create validate request
    pub fn validate() -> Self {
        Self {
            operation: FacadeOperation::Validate,
            key: None,
            value: None,
            category: None,
        }
    }
}

/// Facade response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacadeResponse {
    /// Result
    pub result: FacadeResult,
    /// Value (for get operations)
    pub value: Option<String>,
    /// Message
    pub message: Option<String>,
    /// Count (for list operations)
    pub count: usize,
}

impl FacadeResponse {
    /// Create success response
    pub fn success() -> Self {
        Self {
            result: FacadeResult::Success,
            value: None,
            message: None,
            count: 0,
        }
    }

    /// Create success with value
    pub fn with_value(value: impl Into<String>) -> Self {
        Self {
            result: FacadeResult::Success,
            value: Some(value.into()),
            message: None,
            count: 0,
        }
    }

    /// Create error response
    pub fn error(result: FacadeResult, message: impl Into<String>) -> Self {
        Self {
            result,
            value: None,
            message: Some(message.into()),
            count: 0,
        }
    }

    /// Set count
    pub fn with_count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }

    /// Is success
    pub fn is_success(&self) -> bool {
        self.result == FacadeResult::Success
    }
}

/// Facade usage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FacadeUsage {
    /// Total requests
    pub total_requests: usize,
    /// Successful requests
    pub successful_requests: usize,
    /// Failed requests
    pub failed_requests: usize,
    /// Get operations
    pub get_count: usize,
    /// Set operations
    pub set_count: usize,
}

impl FacadeUsage {
    /// Record operation
    pub fn record(&mut self, operation: FacadeOperation, success: bool) {
        self.total_requests += 1;
        if success {
            self.successful_requests += 1;
        } else {
            self.failed_requests += 1;
        }
        match operation {
            FacadeOperation::Get => self.get_count += 1,
            FacadeOperation::Set => self.set_count += 1,
            _ => {}
        }
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            1.0
        } else {
            self.successful_requests as f64 / self.total_requests as f64
        }
    }
}

/// Settings facade
#[derive(Debug, Clone, Default)]
pub struct SettingsFacade {
    /// Usage statistics
    usage: FacadeUsage,
    /// Enabled
    enabled: bool,
}

impl SettingsFacade {
    /// Create new facade
    pub fn new() -> Self {
        Self {
            enabled: true,
            ..Default::default()
        }
    }

    /// Process request
    pub fn process(&mut self, request: &FacadeRequest) -> FacadeResponse {
        if !self.enabled {
            return FacadeResponse::error(FacadeResult::SystemError, "Facade disabled");
        }

        // Simulate processing
        let response = match request.operation {
            FacadeOperation::Get => {
                if request.key.is_some() {
                    FacadeResponse::with_value("default_value")
                } else {
                    FacadeResponse::error(FacadeResult::NotFound, "Key required")
                }
            }
            FacadeOperation::Set => {
                if request.key.is_some() && request.value.is_some() {
                    FacadeResponse::success()
                } else {
                    FacadeResponse::error(FacadeResult::InvalidValue, "Key and value required")
                }
            }
            FacadeOperation::Reset => FacadeResponse::success(),
            FacadeOperation::List => FacadeResponse::success().with_count(10),
            FacadeOperation::Validate => FacadeResponse::success(),
        };

        self.usage.record(request.operation, response.is_success());
        response
    }

    /// Enable facade
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable facade
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get usage
    pub fn usage(&self) -> &FacadeUsage {
        &self.usage
    }
}

/// Format facade
pub fn format_facade(facade: &SettingsFacade) -> String {
    let mut output = String::new();
    output.push_str("Settings Facade:\n");
    output.push_str(&format!("  Enabled: {}\n", facade.is_enabled()));
    output.push_str(&format!("  Requests: {}\n", facade.usage().total_requests));
    output.push_str(&format!("  Success Rate: {:.1}%\n", facade.usage().success_rate() * 100.0));
    output
}

/// Check if query is about facade
pub fn is_facade_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("facade")
        || lower.contains("settings facade")
        || lower.contains("simple settings")
}

/// Fun fact about facade
pub fn facade_fun_fact() -> &'static str {
    "Anna's settings facade provides a simple interface to the complex settings system!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_display() {
        assert_eq!(format!("{}", FacadeOperation::Get), "get");
        assert_eq!(format!("{}", FacadeOperation::Set), "set");
    }

    #[test]
    fn test_result_display() {
        assert_eq!(format!("{}", FacadeResult::Success), "success");
        assert_eq!(format!("{}", FacadeResult::NotFound), "not_found");
    }

    #[test]
    fn test_request_get() {
        let r = FacadeRequest::get("key");
        assert_eq!(r.operation, FacadeOperation::Get);
    }

    #[test]
    fn test_request_set() {
        let r = FacadeRequest::set("key", "value");
        assert!(r.value.is_some());
    }

    #[test]
    fn test_response_success() {
        let r = FacadeResponse::success();
        assert!(r.is_success());
    }

    #[test]
    fn test_response_error() {
        let r = FacadeResponse::error(FacadeResult::NotFound, "msg");
        assert!(!r.is_success());
    }

    #[test]
    fn test_usage_record() {
        let mut u = FacadeUsage::default();
        u.record(FacadeOperation::Get, true);
        assert_eq!(u.get_count, 1);
    }

    #[test]
    fn test_facade_new() {
        let f = SettingsFacade::new();
        assert!(f.is_enabled());
    }

    #[test]
    fn test_facade_process_get() {
        let mut f = SettingsFacade::new();
        let req = FacadeRequest::get("key");
        let resp = f.process(&req);
        assert!(resp.is_success());
    }

    #[test]
    fn test_facade_disable() {
        let mut f = SettingsFacade::new();
        f.disable();
        assert!(!f.is_enabled());
    }

    #[test]
    fn test_is_facade_query() {
        assert!(is_facade_query("settings facade"));
        assert!(!is_facade_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = facade_fun_fact();
        assert!(fact.contains("facade"));
    }
}
