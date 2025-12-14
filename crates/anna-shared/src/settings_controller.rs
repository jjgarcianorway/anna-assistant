// v0.0.619: Settings Controller (Phase 195)
// Control settings operations with actions and responses

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Controller action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControllerAction {
    /// Get action
    Get,
    /// Set action
    Set,
    /// List action
    List,
    /// Reset action
    Reset,
    /// Apply action
    Apply,
    /// Rollback action
    Rollback,
}

impl std::fmt::Display for ControllerAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Get => write!(f, "get"),
            Self::Set => write!(f, "set"),
            Self::List => write!(f, "list"),
            Self::Reset => write!(f, "reset"),
            Self::Apply => write!(f, "apply"),
            Self::Rollback => write!(f, "rollback"),
        }
    }
}

/// Response status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ResponseStatus {
    /// Success
    #[default]
    Success,
    /// Error
    Error,
    /// NotFound
    NotFound,
    /// Forbidden
    Forbidden,
    /// Invalid
    Invalid,
}

impl std::fmt::Display for ResponseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "success"),
            Self::Error => write!(f, "error"),
            Self::NotFound => write!(f, "not_found"),
            Self::Forbidden => write!(f, "forbidden"),
            Self::Invalid => write!(f, "invalid"),
        }
    }
}

/// Control request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlRequest {
    /// Unique ID
    pub id: String,
    /// Action
    pub action: ControllerAction,
    /// Category
    pub category: Option<SettingsCategory>,
    /// Key
    pub key: Option<String>,
    /// Value
    pub value: Option<String>,
    /// Timestamp
    pub timestamp: u64,
}

impl ControlRequest {
    /// Create new request
    pub fn new(id: impl Into<String>, action: ControllerAction) -> Self {
        Self {
            id: id.into(),
            action,
            category: None,
            key: None,
            value: None,
            timestamp: 0,
        }
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set key
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Set value
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Set timestamp
    pub fn timestamp(mut self, ts: u64) -> Self {
        self.timestamp = ts;
        self
    }
}

/// Control response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlResponse {
    /// Request ID
    pub request_id: String,
    /// Status
    pub status: ResponseStatus,
    /// Data
    pub data: Option<String>,
    /// Error message
    pub error: Option<String>,
    /// Duration ms
    pub duration_ms: u64,
}

impl ControlResponse {
    /// Create success response
    pub fn success(request_id: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
            status: ResponseStatus::Success,
            data: None,
            error: None,
            duration_ms: 0,
        }
    }

    /// Create error response
    pub fn error(request_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
            status: ResponseStatus::Error,
            data: None,
            error: Some(error.into()),
            duration_ms: 0,
        }
    }

    /// Create not found response
    pub fn not_found(request_id: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
            status: ResponseStatus::NotFound,
            data: None,
            error: None,
            duration_ms: 0,
        }
    }

    /// Set data
    pub fn with_data(mut self, data: impl Into<String>) -> Self {
        self.data = Some(data.into());
        self
    }

    /// Set duration
    pub fn with_duration(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }

    /// Is success
    pub fn is_success(&self) -> bool {
        self.status == ResponseStatus::Success
    }
}

/// Settings controller
#[derive(Debug, Clone, Default)]
pub struct SettingsController {
    /// Pending requests
    pending: HashMap<String, ControlRequest>,
    /// Responses
    responses: Vec<ControlResponse>,
    /// Max responses
    max_responses: usize,
    /// Total requests
    total_requests: usize,
    /// Successful requests
    successful: usize,
}

impl SettingsController {
    /// Create new controller
    pub fn new() -> Self {
        Self {
            max_responses: 200,
            ..Default::default()
        }
    }

    /// Submit request
    pub fn submit(&mut self, request: ControlRequest) {
        self.pending.insert(request.id.clone(), request);
        self.total_requests += 1;
    }

    /// Get pending
    pub fn get_pending(&self, id: &str) -> Option<&ControlRequest> {
        self.pending.get(id)
    }

    /// Respond
    pub fn respond(&mut self, response: ControlResponse) {
        self.pending.remove(&response.request_id);
        if response.is_success() {
            self.successful += 1;
        }
        self.responses.push(response);
        while self.responses.len() > self.max_responses {
            self.responses.remove(0);
        }
    }

    /// Pending count
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Total requests
    pub fn total_requests(&self) -> usize {
        self.total_requests
    }

    /// Successful count
    pub fn successful_count(&self) -> usize {
        self.successful
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            1.0
        } else {
            self.successful as f64 / self.total_requests as f64
        }
    }

    /// Get responses
    pub fn responses(&self) -> &[ControlResponse] {
        &self.responses
    }
}

/// Format controller
pub fn format_controller(controller: &SettingsController) -> String {
    let mut output = String::new();
    output.push_str("Settings Controller:\n");
    output.push_str(&format!("  Pending: {}\n", controller.pending_count()));
    output.push_str(&format!("  Total: {}\n", controller.total_requests()));
    output.push_str(&format!("  Successful: {}\n", controller.successful_count()));
    output.push_str(&format!("  Success Rate: {:.1}%\n", controller.success_rate() * 100.0));
    output
}

/// Check if query is about controller
pub fn is_controller_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("controller")
        || lower.contains("control settings")
        || lower.contains("settings control")
}

/// Fun fact about controller
pub fn controller_fun_fact() -> &'static str {
    "Anna's controller handles all settings commands with proper request/response handling!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_display() {
        assert_eq!(format!("{}", ControllerAction::Get), "get");
        assert_eq!(format!("{}", ControllerAction::Set), "set");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", ResponseStatus::Success), "success");
        assert_eq!(format!("{}", ResponseStatus::Error), "error");
    }

    #[test]
    fn test_request_new() {
        let r = ControlRequest::new("r1", ControllerAction::Get);
        assert!(r.key.is_none());
    }

    #[test]
    fn test_request_builder() {
        let r = ControlRequest::new("r1", ControllerAction::Set)
            .key("test_key")
            .value("test_value");
        assert!(r.value.is_some());
    }

    #[test]
    fn test_response_success() {
        let r = ControlResponse::success("r1");
        assert!(r.is_success());
    }

    #[test]
    fn test_response_error() {
        let r = ControlResponse::error("r1", "failed");
        assert!(!r.is_success());
    }

    #[test]
    fn test_response_not_found() {
        let r = ControlResponse::not_found("r1");
        assert_eq!(r.status, ResponseStatus::NotFound);
    }

    #[test]
    fn test_controller_new() {
        let c = SettingsController::new();
        assert_eq!(c.pending_count(), 0);
    }

    #[test]
    fn test_controller_submit() {
        let mut c = SettingsController::new();
        c.submit(ControlRequest::new("r1", ControllerAction::Get));
        assert_eq!(c.pending_count(), 1);
    }

    #[test]
    fn test_controller_respond() {
        let mut c = SettingsController::new();
        c.submit(ControlRequest::new("r1", ControllerAction::Get));
        c.respond(ControlResponse::success("r1"));
        assert_eq!(c.pending_count(), 0);
        assert_eq!(c.successful_count(), 1);
    }

    #[test]
    fn test_is_controller_query() {
        assert!(is_controller_query("settings controller"));
        assert!(!is_controller_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = controller_fun_fact();
        assert!(fact.contains("controller"));
    }
}
