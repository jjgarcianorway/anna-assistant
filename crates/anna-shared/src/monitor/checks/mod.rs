//! System monitoring checks.

mod basic;
mod changes;
mod hardware;
mod learning_checks;
mod performance;
mod security;

use crate::monitor::types::{MonitorResults, MonitorThresholds};

pub use changes::update_baseline;

/// Run all monitoring checks
/// v0.0.990: Added security, hardware, and behavioral checks
pub fn run_checks(thresholds: &MonitorThresholds) -> MonitorResults {
    let start = std::time::Instant::now();
    let mut issues = Vec::new();

    // Basic system checks
    issues.extend(basic::check_disk_space(thresholds));
    issues.extend(basic::check_memory(thresholds));
    issues.extend(basic::check_failed_services());
    issues.extend(basic::check_journal_errors());
    issues.extend(basic::check_updates(thresholds));

    // v0.0.990: Security checks
    issues.extend(security::check_ssh_security());
    issues.extend(security::check_firewall());
    issues.extend(security::check_suspicious_logins());
    issues.extend(security::check_open_ports());

    // v0.0.990: Hardware checks
    issues.extend(hardware::check_thermal());
    issues.extend(hardware::check_smart_health());

    // v0.0.990: Hardware and config change detection
    issues.extend(changes::check_hardware_changes());
    issues.extend(changes::check_config_changes());

    // v0.0.990: Learning-based checks
    issues.extend(learning_checks::check_learned_changes());

    // v0.0.990: Performance checks
    issues.extend(performance::check_boot_time());

    MonitorResults {
        issues,
        checked_at: chrono::Utc::now().to_rfc3339(),
        duration_ms: start.elapsed().as_millis() as u64,
    }
}
