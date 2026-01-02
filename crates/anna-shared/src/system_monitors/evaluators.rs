//! Condition evaluation and alarm checking.

use super::checks::{check_disk_usage, check_failed_services, check_memory_usage};
use super::platform::is_service_failed;
use super::types::{CheckType, MonitorResult};
use crate::user_alarms::{AlarmCondition, AlarmStore, UserAlarm};

/// Evaluate an alarm condition and return result
pub fn evaluate_condition(condition: &AlarmCondition) -> MonitorResult {
    match condition {
        AlarmCondition::DiskAbove { threshold_percent, path } => {
            let mut result = check_disk_usage(path.as_deref());
            result.threshold = Some(*threshold_percent as u64);
            result.should_alert = result.current_value >= *threshold_percent as u64;
            if result.should_alert {
                result.message = format!(
                    "Disk usage ({}%) exceeds threshold ({}%)",
                    result.current_value, threshold_percent
                );
            }
            result
        }
        AlarmCondition::MemoryAbove { threshold_percent } => {
            let mut result = check_memory_usage();
            result.threshold = Some(*threshold_percent as u64);
            result.should_alert = result.current_value >= *threshold_percent as u64;
            if result.should_alert {
                result.message = format!(
                    "Memory usage ({}%) exceeds threshold ({}%)",
                    result.current_value, threshold_percent
                );
            }
            result
        }
        AlarmCondition::ServiceFailed { service } => {
            let is_failed = is_service_failed(service);
            MonitorResult {
                check_type: CheckType::FailedServices,
                current_value: if is_failed { 1 } else { 0 },
                threshold: Some(1),
                should_alert: is_failed,
                message: if is_failed {
                    format!("Service '{}' is in failed state", service)
                } else {
                    format!("Service '{}' is running", service)
                },
            }
        }
        AlarmCondition::AnyServiceFailed => check_failed_services(),
        AlarmCondition::ProbeMatches { probe, pattern } => {
            // This would run a probe and match output - simplified here
            MonitorResult {
                check_type: CheckType::FailedServices,
                current_value: 0,
                threshold: None,
                should_alert: false,
                message: format!("Probe '{}' with pattern '{}'", probe, pattern),
            }
        }
    }
}

/// Check all conditional alarms and return those that should trigger
pub fn check_conditional_alarms(store: &AlarmStore) -> Vec<(&UserAlarm, MonitorResult)> {
    let mut triggered = Vec::new();

    for alarm in store.list() {
        if !alarm.enabled {
            continue;
        }

        if let crate::user_alarms::AlarmSchedule::Conditional { condition } = &alarm.schedule {
            let result = evaluate_condition(condition);
            if result.should_alert {
                triggered.push((alarm, result));
            }
        }
    }

    triggered
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_disk_condition() {
        let condition = AlarmCondition::DiskAbove {
            threshold_percent: 90,
            path: None,
        };
        let result = evaluate_condition(&condition);
        assert_eq!(result.check_type, CheckType::DiskUsage);
        assert_eq!(result.threshold, Some(90));
    }

    #[test]
    fn test_evaluate_memory_condition() {
        let condition = AlarmCondition::MemoryAbove {
            threshold_percent: 80,
        };
        let result = evaluate_condition(&condition);
        assert_eq!(result.check_type, CheckType::MemoryUsage);
        assert_eq!(result.threshold, Some(80));
    }
}
