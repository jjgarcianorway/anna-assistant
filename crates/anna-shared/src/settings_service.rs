// v0.0.620: Settings Service (Phase 196)
// Service layer for settings operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Settings service state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SettingsServiceState {
    /// Starting
    Starting,
    /// Running
    #[default]
    Running,
    /// Stopping
    Stopping,
    /// Stopped
    Stopped,
    /// Error
    Error,
}

impl std::fmt::Display for SettingsServiceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Starting => write!(f, "starting"),
            Self::Running => write!(f, "running"),
            Self::Stopping => write!(f, "stopping"),
            Self::Stopped => write!(f, "stopped"),
            Self::Error => write!(f, "error"),
        }
    }
}

/// Service endpoint
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServiceEndpoint {
    /// Get endpoint
    Get,
    /// Set endpoint
    Set,
    /// Delete endpoint
    Delete,
    /// List endpoint
    List,
    /// Health endpoint
    Health,
    /// Stats endpoint
    Stats,
}

impl std::fmt::Display for ServiceEndpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Get => write!(f, "get"),
            Self::Set => write!(f, "set"),
            Self::Delete => write!(f, "delete"),
            Self::List => write!(f, "list"),
            Self::Health => write!(f, "health"),
            Self::Stats => write!(f, "stats"),
        }
    }
}

/// Service call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCall {
    /// Unique ID
    pub id: String,
    /// Endpoint
    pub endpoint: ServiceEndpoint,
    /// Category
    pub category: Option<SettingsCategory>,
    /// Key
    pub key: Option<String>,
    /// Value
    pub value: Option<String>,
    /// Timestamp
    pub timestamp: u64,
}

impl ServiceCall {
    /// Create new call
    pub fn new(id: impl Into<String>, endpoint: ServiceEndpoint) -> Self {
        Self {
            id: id.into(),
            endpoint,
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

/// Service response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceResponse {
    /// Call ID
    pub call_id: String,
    /// Success
    pub success: bool,
    /// Data
    pub data: Option<String>,
    /// Error
    pub error: Option<String>,
    /// Duration ms
    pub duration_ms: u64,
}

impl ServiceResponse {
    /// Create success response
    pub fn success(call_id: impl Into<String>) -> Self {
        Self {
            call_id: call_id.into(),
            success: true,
            data: None,
            error: None,
            duration_ms: 0,
        }
    }

    /// Create error response
    pub fn error(call_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            call_id: call_id.into(),
            success: false,
            data: None,
            error: Some(error.into()),
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
}

/// Service stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceStats {
    /// Total calls
    pub total_calls: usize,
    /// Successful calls
    pub successful_calls: usize,
    /// Failed calls
    pub failed_calls: usize,
    /// Average duration ms
    pub avg_duration_ms: u64,
}

impl ServiceStats {
    /// Record call
    pub fn record(&mut self, success: bool, duration_ms: u64) {
        self.total_calls += 1;
        if success {
            self.successful_calls += 1;
        } else {
            self.failed_calls += 1;
        }
        // Update rolling average
        if self.total_calls == 1 {
            self.avg_duration_ms = duration_ms;
        } else {
            self.avg_duration_ms = (self.avg_duration_ms + duration_ms) / 2;
        }
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_calls == 0 {
            1.0
        } else {
            self.successful_calls as f64 / self.total_calls as f64
        }
    }
}

/// Settings service
#[derive(Debug, Clone, Default)]
pub struct SettingsService {
    /// State
    state: SettingsServiceState,
    /// Stats by endpoint
    endpoint_stats: HashMap<ServiceEndpoint, ServiceStats>,
    /// Global stats
    global_stats: ServiceStats,
    /// Started at
    started_at: u64,
}

impl SettingsService {
    /// Create new service
    pub fn new() -> Self {
        Self::default()
    }

    /// Get state
    pub fn state(&self) -> SettingsServiceState {
        self.state
    }

    /// Start service
    pub fn start(&mut self, timestamp: u64) {
        self.state = SettingsServiceState::Running;
        self.started_at = timestamp;
    }

    /// Stop service
    pub fn stop(&mut self) {
        self.state = SettingsServiceState::Stopped;
    }

    /// Set error
    pub fn set_error(&mut self) {
        self.state = SettingsServiceState::Error;
    }

    /// Record call
    pub fn record_call(&mut self, endpoint: ServiceEndpoint, success: bool, duration_ms: u64) {
        self.global_stats.record(success, duration_ms);
        self.endpoint_stats
            .entry(endpoint)
            .or_default()
            .record(success, duration_ms);
    }

    /// Get endpoint stats
    pub fn endpoint_stats(&self, endpoint: ServiceEndpoint) -> Option<&ServiceStats> {
        self.endpoint_stats.get(&endpoint)
    }

    /// Get global stats
    pub fn global_stats(&self) -> &ServiceStats {
        &self.global_stats
    }

    /// Is running
    pub fn is_running(&self) -> bool {
        self.state == SettingsServiceState::Running
    }

    /// Uptime
    pub fn uptime(&self, now: u64) -> u64 {
        if self.started_at == 0 || now < self.started_at {
            0
        } else {
            now - self.started_at
        }
    }
}

/// Format service
pub fn format_service(service: &SettingsService) -> String {
    let mut output = String::new();
    output.push_str("Settings Service:\n");
    output.push_str(&format!("  State: {}\n", service.state()));
    output.push_str(&format!("  Total Calls: {}\n", service.global_stats().total_calls));
    output.push_str(&format!("  Success Rate: {:.1}%\n", service.global_stats().success_rate() * 100.0));
    output
}

/// Check if query is about service
pub fn is_service_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("service")
        || lower.contains("settings service")
        || lower.contains("api service")
}

/// Fun fact about service
pub fn service_fun_fact() -> &'static str {
    "Anna's settings service provides a clean API for all settings operations!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_display() {
        assert_eq!(format!("{}", SettingsServiceState::Running), "running");
        assert_eq!(format!("{}", SettingsServiceState::Stopped), "stopped");
    }

    #[test]
    fn test_endpoint_display() {
        assert_eq!(format!("{}", ServiceEndpoint::Get), "get");
        assert_eq!(format!("{}", ServiceEndpoint::Set), "set");
    }

    #[test]
    fn test_call_new() {
        let c = ServiceCall::new("c1", ServiceEndpoint::Get);
        assert!(c.key.is_none());
    }

    #[test]
    fn test_call_builder() {
        let c = ServiceCall::new("c1", ServiceEndpoint::Set)
            .key("test")
            .value("val");
        assert!(c.value.is_some());
    }

    #[test]
    fn test_response_success() {
        let r = ServiceResponse::success("c1");
        assert!(r.success);
    }

    #[test]
    fn test_response_error() {
        let r = ServiceResponse::error("c1", "failed");
        assert!(!r.success);
    }

    #[test]
    fn test_stats_record() {
        let mut s = ServiceStats::default();
        s.record(true, 100);
        s.record(false, 200);
        assert_eq!(s.total_calls, 2);
    }

    #[test]
    fn test_service_new() {
        let s = SettingsService::new();
        assert_eq!(s.state(), SettingsServiceState::Running);
    }

    #[test]
    fn test_service_lifecycle() {
        let mut s = SettingsService::new();
        s.start(100);
        assert!(s.is_running());
        s.stop();
        assert!(!s.is_running());
    }

    #[test]
    fn test_service_record_call() {
        let mut s = SettingsService::new();
        s.record_call(ServiceEndpoint::Get, true, 50);
        assert_eq!(s.global_stats().total_calls, 1);
    }

    #[test]
    fn test_is_service_query() {
        assert!(is_service_query("settings service"));
        assert!(!is_service_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = service_fun_fact();
        assert!(fact.contains("service"));
    }
}
