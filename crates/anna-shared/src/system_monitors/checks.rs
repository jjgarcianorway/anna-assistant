//! Individual system monitoring checks.

use super::platform::{
    count_failed_services, get_cpu_count, get_disk_space, get_load_average, get_memory_info,
    get_swap_info, is_service_failed,
};
use super::types::{CheckType, MonitorResult};

/// Check disk usage
pub fn check_disk_usage(path: Option<&str>) -> MonitorResult {
    let path = path.unwrap_or("/");

    // Try to get actual disk usage
    let (total, available) = get_disk_space(path);
    let used_percent = if total > 0 {
        ((total - available) * 100 / total) as u64
    } else {
        0
    };

    MonitorResult {
        check_type: CheckType::DiskUsage,
        current_value: used_percent,
        threshold: None,
        should_alert: false,
        message: format!("Disk usage on {}: {}%", path, used_percent),
    }
}

/// Check memory usage
pub fn check_memory_usage() -> MonitorResult {
    let (total, available) = get_memory_info();
    let used_percent = if total > 0 {
        ((total - available) * 100 / total) as u64
    } else {
        0
    };

    MonitorResult {
        check_type: CheckType::MemoryUsage,
        current_value: used_percent,
        threshold: None,
        should_alert: false,
        message: format!("Memory usage: {}%", used_percent),
    }
}

/// Check for failed systemd services
pub fn check_failed_services() -> MonitorResult {
    let count = count_failed_services();

    MonitorResult {
        check_type: CheckType::FailedServices,
        current_value: count,
        threshold: None,
        should_alert: count > 0,
        message: if count == 0 {
            "All services running".to_string()
        } else {
            format!("{} service(s) in failed state", count)
        },
    }
}

/// Check system load average
pub fn check_load_average() -> MonitorResult {
    let load = get_load_average();
    let cpu_count = get_cpu_count() as u64;
    let load_percent = if cpu_count > 0 {
        (load * 100 / cpu_count) as u64
    } else {
        load as u64
    };

    MonitorResult {
        check_type: CheckType::LoadAverage,
        current_value: load_percent,
        threshold: None,
        should_alert: false,
        message: format!("Load average: {} ({}% of {} cores)", load, load_percent, cpu_count),
    }
}

/// Check swap usage
pub fn check_swap_usage() -> MonitorResult {
    let (total, used) = get_swap_info();
    let used_percent = if total > 0 {
        (used * 100 / total) as u64
    } else {
        0
    };

    MonitorResult {
        check_type: CheckType::SwapUsage,
        current_value: used_percent,
        threshold: None,
        should_alert: false,
        message: format!("Swap usage: {}%", used_percent),
    }
}

/// Run all standard system checks
pub fn run_all_checks() -> Vec<MonitorResult> {
    vec![
        check_disk_usage(None),
        check_memory_usage(),
        check_failed_services(),
        check_load_average(),
        check_swap_usage(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_all_checks() {
        let results = run_all_checks();
        assert_eq!(results.len(), 5);
    }
}
