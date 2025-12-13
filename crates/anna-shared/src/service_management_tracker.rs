//! Service Management Tracker - Phase 81
//!
//! Tracks service operations (start, stop, restart, enable, disable) by Anna.
//! VISION.md mentions Anna being able to restart services and manage daemons.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Service operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServiceOperation {
    Start,
    Stop,
    Restart,
    Reload,
    Enable,
    Disable,
    Status,
}

impl ServiceOperation {
    pub fn symbol(&self) -> &'static str {
        match self {
            ServiceOperation::Start => ">",
            ServiceOperation::Stop => "x",
            ServiceOperation::Restart => "~",
            ServiceOperation::Reload => "r",
            ServiceOperation::Enable => "+",
            ServiceOperation::Disable => "-",
            ServiceOperation::Status => "?",
        }
    }

    pub fn verb(&self) -> &'static str {
        match self {
            ServiceOperation::Start => "started",
            ServiceOperation::Stop => "stopped",
            ServiceOperation::Restart => "restarted",
            ServiceOperation::Reload => "reloaded",
            ServiceOperation::Enable => "enabled",
            ServiceOperation::Disable => "disabled",
            ServiceOperation::Status => "checked",
        }
    }
}

/// Result of service operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationResult {
    Success,
    Failed,
    Skipped,
    Pending,
}

impl OperationResult {
    pub fn symbol(&self) -> &'static str {
        match self {
            OperationResult::Success => "+",
            OperationResult::Failed => "x",
            OperationResult::Skipped => "-",
            OperationResult::Pending => "?",
        }
    }
}

/// A single service operation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRecord {
    /// Service name
    pub service_name: String,
    /// Operation performed
    pub operation: ServiceOperation,
    /// Result
    pub result: OperationResult,
    /// Timestamp
    pub timestamp: u64,
    /// Associated ticket ID
    pub ticket_id: Option<String>,
    /// Reason for operation
    pub reason: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
    /// Whether user confirmed
    pub user_confirmed: bool,
}

/// Service management tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceTracker {
    /// All operation records
    pub records: Vec<ServiceRecord>,
    /// Count by operation type
    pub by_operation: HashMap<String, u64>,
    /// Count by service
    pub by_service: HashMap<String, u64>,
    /// Success count
    pub success_count: u64,
    /// Failure count
    pub failure_count: u64,
}

impl ServiceTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a service operation
    pub fn record(&mut self, record: ServiceRecord) {
        let op_key = format!("{:?}", record.operation);
        *self.by_operation.entry(op_key).or_insert(0) += 1;
        *self.by_service.entry(record.service_name.clone()).or_insert(0) += 1;

        match record.result {
            OperationResult::Success => self.success_count += 1,
            OperationResult::Failed => self.failure_count += 1,
            _ => {}
        }

        self.records.push(record);
    }

    /// Get recent operations
    pub fn recent(&self, limit: usize) -> Vec<&ServiceRecord> {
        self.records.iter().rev().take(limit).collect()
    }

    /// Get operations by type
    pub fn by_operation_type(&self, op: ServiceOperation) -> Vec<&ServiceRecord> {
        self.records.iter().filter(|r| r.operation == op).collect()
    }

    /// Get operations for a service
    pub fn for_service(&self, name: &str) -> Vec<&ServiceRecord> {
        self.records.iter().filter(|r| r.service_name == name).collect()
    }

    /// Get failed operations
    pub fn failed(&self) -> Vec<&ServiceRecord> {
        self.records
            .iter()
            .filter(|r| r.result == OperationResult::Failed)
            .collect()
    }

    /// Get successful operations
    pub fn successful(&self) -> Vec<&ServiceRecord> {
        self.records
            .iter()
            .filter(|r| r.result == OperationResult::Success)
            .collect()
    }

    /// Total operation count
    pub fn total_count(&self) -> usize {
        self.records.len()
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        let total = self.success_count + self.failure_count;
        if total == 0 {
            return 100.0;
        }
        (self.success_count as f64 / total as f64) * 100.0
    }

    /// Most managed service
    pub fn most_managed(&self) -> Option<(&str, u64)> {
        self.by_service
            .iter()
            .max_by_key(|(_, v)| *v)
            .map(|(k, v)| (k.as_str(), *v))
    }

    /// Most common operation
    pub fn most_common_op(&self) -> Option<(&str, u64)> {
        self.by_operation
            .iter()
            .max_by_key(|(_, v)| *v)
            .map(|(k, v)| (k.as_str(), *v))
    }

    /// Unique services managed
    pub fn unique_services(&self) -> usize {
        self.by_service.len()
    }
}

/// Format service tracker for display
pub fn format_service_tracker(tracker: &ServiceTracker) -> String {
    let mut lines = vec!["=== Service Management History ===".to_string()];
    lines.push(String::new());

    if tracker.records.is_empty() {
        lines.push("No service operations yet.".to_string());
        return lines.join("\n");
    }

    // Summary
    lines.push(format!("Total operations: {}", tracker.total_count()));
    lines.push(format!("Success rate: {:.1}%", tracker.success_rate()));
    lines.push(format!("Unique services: {}", tracker.unique_services()));

    // By operation
    if !tracker.by_operation.is_empty() {
        lines.push(String::new());
        lines.push("By operation type:".to_string());
        for (op, count) in &tracker.by_operation {
            lines.push(format!("  {}: {}", op, count));
        }
    }

    // Most managed
    if let Some((service, count)) = tracker.most_managed() {
        lines.push(String::new());
        lines.push(format!("Most managed: {} ({} ops)", service, count));
    }

    // Recent operations
    let recent = tracker.recent(5);
    if !recent.is_empty() {
        lines.push(String::new());
        lines.push("Recent operations:".to_string());
        for rec in recent {
            let result = rec.result.symbol();
            let op = rec.operation.verb();
            lines.push(format!("  [{}] {} {}", result, rec.service_name, op));
        }
    }

    // Failed operations
    let failed = tracker.failed();
    if !failed.is_empty() {
        lines.push(String::new());
        lines.push(format!("Failed operations: {}", failed.len()));
        for rec in failed.iter().take(3) {
            let error = rec.error.as_deref().unwrap_or("unknown error");
            lines.push(format!("  {} - {}", rec.service_name, error));
        }
    }

    lines.join("\n")
}

/// Format service tracker compact
pub fn format_service_tracker_compact(tracker: &ServiceTracker) -> String {
    format!(
        "Services: {} ops ({:.1}% ok) | {} unique | {} failed",
        tracker.total_count(),
        tracker.success_rate(),
        tracker.unique_services(),
        tracker.failure_count
    )
}

/// Format service tracker one-line
pub fn format_service_tracker_oneline(tracker: &ServiceTracker) -> String {
    format!(
        "{} service ops ({} services)",
        tracker.total_count(),
        tracker.unique_services()
    )
}

/// Check if query is about service management
pub fn is_service_tracker_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "service history",
        "service operations",
        "managed services",
        "services managed",
        "service tracker",
        "restarted services",
        "stopped services",
        "services have you",
        "service management",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Generate fun fact about service management
pub fn service_fun_fact(tracker: &ServiceTracker) -> String {
    if tracker.records.is_empty() {
        return "No service operations yet!".to_string();
    }

    let facts = [
        format!(
            "Anna has performed {} service operations.",
            tracker.total_count()
        ),
        format!(
            "{} unique services have been managed.",
            tracker.unique_services()
        ),
        {
            if let Some((service, count)) = tracker.most_managed() {
                format!("{} is the most managed service ({} operations).", service, count)
            } else {
                "No service stats yet.".to_string()
            }
        },
        format!(
            "Service management success rate: {:.1}%.",
            tracker.success_rate()
        ),
    ];

    facts[tracker.total_count() % facts.len()].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(service: &str, op: ServiceOperation, result: OperationResult) -> ServiceRecord {
        ServiceRecord {
            service_name: service.to_string(),
            operation: op,
            result,
            timestamp: 1234567890,
            ticket_id: None,
            reason: Some("test".to_string()),
            error: None,
            user_confirmed: true,
        }
    }

    #[test]
    fn test_service_operation() {
        assert_eq!(ServiceOperation::Start.symbol(), ">");
        assert_eq!(ServiceOperation::Restart.verb(), "restarted");
    }

    #[test]
    fn test_operation_result() {
        assert_eq!(OperationResult::Success.symbol(), "+");
        assert_eq!(OperationResult::Failed.symbol(), "x");
    }

    #[test]
    fn test_service_tracker_record() {
        let mut tracker = ServiceTracker::new();
        tracker.record(make_record("nginx", ServiceOperation::Restart, OperationResult::Success));

        assert_eq!(tracker.total_count(), 1);
        assert_eq!(tracker.success_count, 1);
    }

    #[test]
    fn test_success_rate() {
        let mut tracker = ServiceTracker::new();
        tracker.record(make_record("nginx", ServiceOperation::Restart, OperationResult::Success));
        tracker.record(make_record("docker", ServiceOperation::Start, OperationResult::Failed));

        assert_eq!(tracker.success_rate(), 50.0);
    }

    #[test]
    fn test_for_service() {
        let mut tracker = ServiceTracker::new();
        tracker.record(make_record("nginx", ServiceOperation::Restart, OperationResult::Success));
        tracker.record(make_record("nginx", ServiceOperation::Stop, OperationResult::Success));
        tracker.record(make_record("docker", ServiceOperation::Start, OperationResult::Success));

        assert_eq!(tracker.for_service("nginx").len(), 2);
        assert_eq!(tracker.for_service("docker").len(), 1);
    }

    #[test]
    fn test_by_operation_type() {
        let mut tracker = ServiceTracker::new();
        tracker.record(make_record("nginx", ServiceOperation::Restart, OperationResult::Success));
        tracker.record(make_record("docker", ServiceOperation::Restart, OperationResult::Success));

        assert_eq!(tracker.by_operation_type(ServiceOperation::Restart).len(), 2);
    }

    #[test]
    fn test_most_managed() {
        let mut tracker = ServiceTracker::new();
        tracker.record(make_record("nginx", ServiceOperation::Restart, OperationResult::Success));
        tracker.record(make_record("nginx", ServiceOperation::Reload, OperationResult::Success));
        tracker.record(make_record("docker", ServiceOperation::Start, OperationResult::Success));

        let (service, count) = tracker.most_managed().unwrap();
        assert_eq!(service, "nginx");
        assert_eq!(count, 2);
    }

    #[test]
    fn test_format_service_tracker() {
        let mut tracker = ServiceTracker::new();
        tracker.record(make_record("nginx", ServiceOperation::Restart, OperationResult::Success));

        let output = format_service_tracker(&tracker);
        assert!(output.contains("Service Management History"));
        assert!(output.contains("Total operations: 1"));
    }

    #[test]
    fn test_is_service_tracker_query() {
        assert!(is_service_tracker_query("show service history"));
        assert!(is_service_tracker_query("what services have you managed?"));
        assert!(is_service_tracker_query("restarted services"));
        assert!(!is_service_tracker_query("what is my disk space?"));
    }

    #[test]
    fn test_service_fun_fact() {
        let mut tracker = ServiceTracker::new();
        tracker.record(make_record("nginx", ServiceOperation::Restart, OperationResult::Success));

        let fact = service_fun_fact(&tracker);
        assert!(!fact.is_empty());
    }

    #[test]
    fn test_format_compact_oneline() {
        let mut tracker = ServiceTracker::new();
        tracker.record(make_record("nginx", ServiceOperation::Restart, OperationResult::Success));

        let compact = format_service_tracker_compact(&tracker);
        assert!(compact.contains("Services: 1 ops"));

        let oneline = format_service_tracker_oneline(&tracker);
        assert!(oneline.contains("1 service ops"));
    }
}
