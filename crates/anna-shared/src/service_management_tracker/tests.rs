//! Tests for service management tracker - Phase 81

#[cfg(test)]
mod tests {
    use crate::service_management_tracker::{
        formatting::{format_service_tracker, format_service_tracker_compact, format_service_tracker_oneline},
        queries::{is_service_tracker_query, service_fun_fact},
        tracker::ServiceTracker,
        types::{OperationResult, ServiceOperation, ServiceRecord},
    };

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
