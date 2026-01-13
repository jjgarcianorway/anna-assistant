//! Hardware and config change detection.

use crate::monitor::baseline::SystemBaseline;
use crate::monitor::types::{Issue, IssueType, Severity};

/// Check for hardware changes since baseline
pub fn check_hardware_changes() -> Vec<Issue> {
    let mut issues = Vec::new();

    // Load or create baseline
    let baseline = match SystemBaseline::load() {
        Some(b) => b,
        None => {
            // First run - capture baseline
            let baseline = SystemBaseline::capture();
            let _ = baseline.save();
            return issues; // No changes to report on first run
        }
    };

    let changes = baseline.compare();

    // Report new USB devices
    for device in &changes.usb_added {
        // Skip hubs and common internal devices
        if device.description.to_lowercase().contains("hub") {
            continue;
        }
        issues.push(Issue {
            issue_type: IssueType::HardwareAdded,
            severity: Severity::Info,
            summary: format!("New USB: {}", device.description),
            details: format!(
                "USB device added: {} ({}:{})",
                device.description, device.vendor_id, device.product_id
            ),
            suggested_fix: Some("Run: lsusb to see all USB devices".to_string()),
            detected_at: chrono::Utc::now().to_rfc3339(),
            notified: false,
            acknowledged: false,
        });
    }

    // Report removed USB devices (could indicate hardware failure or theft)
    for device in &changes.usb_removed {
        if device.description.to_lowercase().contains("hub") {
            continue;
        }
        issues.push(Issue {
            issue_type: IssueType::HardwareRemoved,
            severity: Severity::Warning,
            summary: format!("USB removed: {}", device.description),
            details: format!(
                "USB device removed: {} ({}:{})",
                device.description, device.vendor_id, device.product_id
            ),
            suggested_fix: Some("Check if device was intentionally removed".to_string()),
            detected_at: chrono::Utc::now().to_rfc3339(),
            notified: false,
            acknowledged: false,
        });
    }

    // Report new PCI devices (unusual - might indicate hardware change or driver issue)
    for device in &changes.pci_added {
        issues.push(Issue {
            issue_type: IssueType::HardwareAdded,
            severity: Severity::Info,
            summary: format!("New PCI: {} {}", device.vendor, device.class),
            details: format!(
                "PCI device appeared: {} {} (slot {})",
                device.vendor, device.device, device.slot
            ),
            suggested_fix: Some("Run: lspci to see all PCI devices".to_string()),
            detected_at: chrono::Utc::now().to_rfc3339(),
            notified: false,
            acknowledged: false,
        });
    }

    // Report removed PCI devices (concerning - hardware failure?)
    for device in &changes.pci_removed {
        issues.push(Issue {
            issue_type: IssueType::HardwareRemoved,
            severity: Severity::Warning,
            summary: format!("PCI missing: {} {}", device.vendor, device.class),
            details: format!(
                "PCI device disappeared: {} {} (slot {}). Possible hardware failure.",
                device.vendor, device.device, device.slot
            ),
            suggested_fix: Some("Check: dmesg | grep -i error".to_string()),
            detected_at: chrono::Utc::now().to_rfc3339(),
            notified: false,
            acknowledged: false,
        });
    }

    issues
}

/// Check for config file changes since baseline
pub fn check_config_changes() -> Vec<Issue> {
    let mut issues = Vec::new();

    let baseline = match SystemBaseline::load() {
        Some(b) => b,
        None => return issues, // No baseline yet
    };

    let changes = baseline.compare();

    // Security-critical files that warrant warnings
    let critical_files = [
        "/etc/ssh/sshd_config",
        "/etc/sudoers",
        "/etc/passwd",
        "/etc/shadow",
        "/etc/group",
        "/etc/pam.d/system-auth",
    ];

    // Report changed config files
    for path in &changes.config_changed {
        let is_critical = critical_files.iter().any(|c| path.contains(c));
        let severity = if is_critical {
            Severity::Warning
        } else {
            Severity::Info
        };

        let summary = if path.contains("cron") {
            format!("Cron modified: {}", path.split('/').last().unwrap_or(path))
        } else {
            format!("Config changed: {}", path.split('/').last().unwrap_or(path))
        };

        issues.push(Issue {
            issue_type: IssueType::ConfigChanged,
            severity,
            summary,
            details: format!("File {} has been modified since baseline.", path),
            suggested_fix: Some(format!("Review changes: diff {} {}.bak", path, path)),
            detected_at: chrono::Utc::now().to_rfc3339(),
            notified: false,
            acknowledged: false,
        });
    }

    // Report new cron jobs (potential persistence mechanism)
    for path in &changes.config_added {
        if path.contains("cron") || path.contains("spool") {
            issues.push(Issue {
                issue_type: IssueType::CronAdded,
                severity: Severity::Warning,
                summary: format!("New cron: {}", path.split('/').last().unwrap_or(path)),
                details: format!("New cron job detected: {}", path),
                suggested_fix: Some(format!("Review: cat {}", path)),
                detected_at: chrono::Utc::now().to_rfc3339(),
                notified: false,
                acknowledged: false,
            });
        }
    }

    issues
}

/// Update the baseline with current system state
/// Call this after user acknowledges changes
pub fn update_baseline() -> anyhow::Result<()> {
    let baseline = SystemBaseline::capture();
    baseline.save()
}
