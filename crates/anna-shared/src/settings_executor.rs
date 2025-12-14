// v0.0.613: Settings Executor (Phase 189)
// Execute settings operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ExecutionStatus {
    /// Pending
    #[default]
    Pending,
    /// Running
    Running,
    /// Success
    Success,
    /// Failed
    Failed,
    /// Cancelled
    Cancelled,
    /// Timeout
    Timeout,
}

impl std::fmt::Display for ExecutionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Running => write!(f, "running"),
            Self::Success => write!(f, "success"),
            Self::Failed => write!(f, "failed"),
            Self::Cancelled => write!(f, "cancelled"),
            Self::Timeout => write!(f, "timeout"),
        }
    }
}

/// Execution type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionType {
    /// Get setting
    Get,
    /// Set setting
    Set,
    /// Delete setting
    Delete,
    /// Reset setting
    Reset,
    /// Batch operation
    Batch,
    /// Transaction
    Transaction,
}

impl std::fmt::Display for ExecutionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Get => write!(f, "get"),
            Self::Set => write!(f, "set"),
            Self::Delete => write!(f, "delete"),
            Self::Reset => write!(f, "reset"),
            Self::Batch => write!(f, "batch"),
            Self::Transaction => write!(f, "transaction"),
        }
    }
}

/// Execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    /// Unique ID
    pub id: String,
    /// Type
    pub exec_type: ExecutionType,
    /// Category
    pub category: Option<SettingsCategory>,
    /// Key
    pub key: Option<String>,
    /// Value
    pub value: Option<String>,
    /// Timeout ms
    pub timeout_ms: u64,
}

impl ExecutionRequest {
    /// Create new request
    pub fn new(id: impl Into<String>, exec_type: ExecutionType) -> Self {
        Self {
            id: id.into(),
            exec_type,
            category: None,
            key: None,
            value: None,
            timeout_ms: 30000,
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

    /// Set timeout
    pub fn timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }
}

/// Execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Request ID
    pub request_id: String,
    /// Status
    pub status: ExecutionStatus,
    /// Value
    pub value: Option<String>,
    /// Error
    pub error: Option<String>,
    /// Duration ms
    pub duration_ms: u64,
}

impl ExecutionResult {
    /// Create success result
    pub fn success(request_id: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
            status: ExecutionStatus::Success,
            value: None,
            error: None,
            duration_ms: 0,
        }
    }

    /// Create failure result
    pub fn failure(request_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
            status: ExecutionStatus::Failed,
            value: None,
            error: Some(error.into()),
            duration_ms: 0,
        }
    }

    /// Set value
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Set duration
    pub fn with_duration(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }

    /// Is success
    pub fn is_success(&self) -> bool {
        self.status == ExecutionStatus::Success
    }
}

/// Settings executor
#[derive(Debug, Clone, Default)]
pub struct SettingsExecutor {
    /// Pending requests
    pending: HashMap<String, ExecutionRequest>,
    /// Results
    results: Vec<ExecutionResult>,
    /// Max results
    max_results: usize,
    /// Total executed
    total_executed: usize,
    /// Total failed
    total_failed: usize,
}

impl SettingsExecutor {
    /// Create new executor
    pub fn new() -> Self {
        Self {
            max_results: 200,
            ..Default::default()
        }
    }

    /// Submit request
    pub fn submit(&mut self, request: ExecutionRequest) {
        self.pending.insert(request.id.clone(), request);
    }

    /// Get pending
    pub fn pending(&self) -> Vec<&ExecutionRequest> {
        self.pending.values().collect()
    }

    /// Get pending request
    pub fn get_pending(&self, id: &str) -> Option<&ExecutionRequest> {
        self.pending.get(id)
    }

    /// Complete request
    pub fn complete(&mut self, result: ExecutionResult) {
        self.pending.remove(&result.request_id);
        if result.is_success() {
            self.total_executed += 1;
        } else {
            self.total_failed += 1;
        }
        self.results.push(result);
        while self.results.len() > self.max_results {
            self.results.remove(0);
        }
    }

    /// Get results
    pub fn results(&self) -> &[ExecutionResult] {
        &self.results
    }

    /// Pending count
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Total executed
    pub fn total_executed(&self) -> usize {
        self.total_executed
    }

    /// Total failed
    pub fn total_failed(&self) -> usize {
        self.total_failed
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        let total = self.total_executed + self.total_failed;
        if total == 0 {
            1.0
        } else {
            self.total_executed as f64 / total as f64
        }
    }
}

/// Format executor
pub fn format_executor(executor: &SettingsExecutor) -> String {
    let mut output = String::new();
    output.push_str("Settings Executor:\n");
    output.push_str(&format!("  Pending: {}\n", executor.pending_count()));
    output.push_str(&format!("  Executed: {}\n", executor.total_executed()));
    output.push_str(&format!("  Failed: {}\n", executor.total_failed()));
    output.push_str(&format!("  Success Rate: {:.1}%\n", executor.success_rate() * 100.0));
    output
}

/// Check if query is about executor
pub fn is_executor_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("executor")
        || lower.contains("execute")
        || lower.contains("run operation")
}

/// Fun fact about executor
pub fn executor_fun_fact() -> &'static str {
    "Anna's executor handles all settings operations with timeout protection!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", ExecutionStatus::Success), "success");
        assert_eq!(format!("{}", ExecutionStatus::Failed), "failed");
    }

    #[test]
    fn test_type_display() {
        assert_eq!(format!("{}", ExecutionType::Get), "get");
        assert_eq!(format!("{}", ExecutionType::Set), "set");
    }

    #[test]
    fn test_request_new() {
        let r = ExecutionRequest::new("r1", ExecutionType::Get);
        assert_eq!(r.timeout_ms, 30000);
    }

    #[test]
    fn test_request_builder() {
        let r = ExecutionRequest::new("r1", ExecutionType::Set)
            .key("test_key")
            .value("test_value")
            .timeout(60000);
        assert_eq!(r.timeout_ms, 60000);
    }

    #[test]
    fn test_result_success() {
        let r = ExecutionResult::success("r1");
        assert!(r.is_success());
    }

    #[test]
    fn test_result_failure() {
        let r = ExecutionResult::failure("r1", "error message");
        assert!(!r.is_success());
    }

    #[test]
    fn test_executor_new() {
        let e = SettingsExecutor::new();
        assert_eq!(e.pending_count(), 0);
    }

    #[test]
    fn test_executor_submit() {
        let mut e = SettingsExecutor::new();
        e.submit(ExecutionRequest::new("r1", ExecutionType::Get));
        assert_eq!(e.pending_count(), 1);
    }

    #[test]
    fn test_executor_complete() {
        let mut e = SettingsExecutor::new();
        e.submit(ExecutionRequest::new("r1", ExecutionType::Get));
        e.complete(ExecutionResult::success("r1"));
        assert_eq!(e.pending_count(), 0);
        assert_eq!(e.total_executed(), 1);
    }

    #[test]
    fn test_is_executor_query() {
        assert!(is_executor_query("execute operation"));
        assert!(!is_executor_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = executor_fun_fact();
        assert!(fact.contains("executor"));
    }
}
