//! Reflection Helper - Builds reflection summary locally (6.7.0)
//!
//! Temporary implementation that builds reflection on the client side.
//! TODO: Migrate to RPC-based approach once daemon integration is complete.

use anna_common::ipc::{ReflectionItemData, ReflectionSeverity, ReflectionSummaryData};
use chrono::Utc;

/// Build a simple reflection summary
///
/// This is a simplified client-side implementation for 6.7.0.
/// It performs basic checks without full Historian/Proactive access.
pub fn build_local_reflection() -> ReflectionSummaryData {
    let mut items = Vec::new();

    // Check for version changes (compare with last known version)
    if let Some(prev_version) = get_previous_version() {
        let current_version = env!("CARGO_PKG_VERSION");
        if prev_version != current_version {
            items.push(ReflectionItemData {
                severity: ReflectionSeverity::Notice,
                category: "upgrade".to_string(),
                title: format!("Anna updated to v{}", current_version),
                details: format!(
                    "Previously v{}, now v{}. Diagnostics and planner behavior may have changed.",
                    prev_version, current_version
                ),
                since_timestamp: Some(Utc::now()),
            });
        }
    }

    // Extract journal errors
    extract_journal_errors(&mut items);

    // TODO: Add disk usage checks from df
    // TODO: Add service failure checks from systemctl

    ReflectionSummaryData {
        items,
        generated_at: Utc::now(),
    }
}

/// Format reflection summary for display
pub fn format_reflection(summary: &ReflectionSummaryData, use_colors: bool) -> String {
    if summary.items.is_empty() {
        return "Anna reflection: no significant changes, degradations, or recent errors detected.\n".to_string();
    }

    let mut output = String::from("Anna reflection (recent changes and events):\n\n");

    for item in &summary.items {
        output.push_str(&format_reflection_item(item, use_colors));
        output.push_str("\n\n");
    }

    output
}

/// Format a single reflection item
fn format_reflection_item(item: &ReflectionItemData, use_colors: bool) -> String {
    let reset = if use_colors { "\x1b[0m" } else { "" };
    let label_color = if use_colors {
        severity_color(item.severity)
    } else {
        ""
    };
    let label = severity_label(item.severity);

    let timestamp_str = item
        .since_timestamp
        .map(|ts| format!(" ({})", ts.format("%Y-%m-%d %H:%M:%S UTC")))
        .unwrap_or_default();

    format!(
        "[{}{}{}] {}: {}{}\n     {}",
        label_color, label, reset, item.category, item.title, timestamp_str, item.details
    )
}

fn severity_label(severity: ReflectionSeverity) -> &'static str {
    match severity {
        ReflectionSeverity::Info => "INFO",
        ReflectionSeverity::Notice => "NOTICE",
        ReflectionSeverity::Warning => "WARNING",
        ReflectionSeverity::Critical => "CRITICAL",
    }
}

fn severity_color(severity: ReflectionSeverity) -> &'static str {
    match severity {
        ReflectionSeverity::Info => "\x1b[36m",       // Cyan
        ReflectionSeverity::Notice => "\x1b[34m",     // Blue
        ReflectionSeverity::Warning => "\x1b[33m",    // Yellow
        ReflectionSeverity::Critical => "\x1b[31m",   // Red
    }
}

/// Extract recent journal errors
fn extract_journal_errors(items: &mut Vec<ReflectionItemData>) {
    use std::process::Command;

    // Get recent errors from journalctl (last 1 hour, priority error and critical)
    let output = Command::new("journalctl")
        .args(&[
            "--priority=err",
            "--since",
            "1 hour ago",
            "--no-pager",
            "-n",
            "10", // Last 10 error entries
        ])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let log_text = String::from_utf8_lossy(&output.stdout);
            let mut seen_errors = std::collections::HashSet::new();

            for line in log_text.lines() {
                // Parse journalctl output format
                if let Some(error_info) = parse_journal_line(line) {
                    // 6.8.x: Filter out kernel/hardware noise
                    if !is_actionable_error(&error_info) {
                        continue; // Skip this error, it's noise
                    }

                    // Deduplicate by creating a key from service + message prefix
                    let key = format!("{}:{}", error_info.service, &error_info.message[..error_info.message.len().min(50)]);
                    if seen_errors.insert(key) && items.len() < 3 {
                        // Limit to 3 distinct errors
                        items.push(ReflectionItemData {
                            severity: ReflectionSeverity::Warning,
                            category: "logs".to_string(),
                            title: format!("Error: {}", error_info.service),
                            details: format!(
                                "{} ({})",
                                error_info.message,
                                error_info.timestamp
                            ),
                            since_timestamp: Some(error_info.parsed_timestamp),
                        });
                    }
                }
            }
        }
    }
}

/// Parsed journal error information
struct JournalError {
    service: String,
    message: String,
    timestamp: String,
    parsed_timestamp: chrono::DateTime<Utc>,
}

/// Parse a journalctl line
fn parse_journal_line(line: &str) -> Option<JournalError> {
    // Example format: "Nov 24 10:32:11 hostname service[1234]: error message"
    let parts: Vec<&str> = line.splitn(4, ' ').collect();
    if parts.len() < 4 {
        return None;
    }

    // Extract timestamp (first 3 parts: "Nov 24 10:32:11")
    let timestamp_str = parts[..3].join(" ");

    // Parse timestamp
    let parsed_timestamp = chrono::NaiveDateTime::parse_from_str(
        &format!("{} {}", chrono::Utc::now().format("%Y"), timestamp_str),
        "%Y %b %d %H:%M:%S"
    ).ok()?;
    let parsed_timestamp = chrono::DateTime::<Utc>::from_naive_utc_and_offset(parsed_timestamp, Utc);

    // Rest is hostname and message
    let rest = parts[3];
    let message_parts: Vec<&str> = rest.splitn(2, ' ').collect();
    if message_parts.len() < 2 {
        return None;
    }

    // Extract service name from message
    let full_message = message_parts[1];
    let service = if let Some(colon_pos) = full_message.find(':') {
        full_message[..colon_pos].split('[').next().unwrap_or("unknown")
    } else {
        "system"
    };

    // Extract actual error message
    let message = if let Some(colon_pos) = full_message.find(':') {
        &full_message[colon_pos + 1..]
    } else {
        full_message
    };

    Some(JournalError {
        service: service.trim().to_string(),
        message: message.trim().to_string(),
        timestamp: timestamp_str,
        parsed_timestamp,
    })
}

/// 6.8.x: Filter out known kernel/hardware noise
///
/// Returns true if the error is actionable and should be shown to the user.
/// Returns false if it's known noise (driver spam, transient hardware messages).
fn is_actionable_error(error: &JournalError) -> bool {
    let service_lower = error.service.to_lowercase();
    let message_lower = error.message.to_lowercase();

    // Filter kernel/driver noise
    if service_lower == "kernel" {
        // Known noisy patterns in kernel messages
        let noise_patterns = [
            "crc",
            "dualsense",
            "playstation",
            "iwlwifi",
            "microcode",
            "bluetooth",  // Kernel Bluetooth spam, not service failures
            "usb disconnect",
            "usb connect",
            "pcie",
            "acpi",
        ];

        for pattern in &noise_patterns {
            if message_lower.contains(pattern) {
                return false; // Known noise, filter out
            }
        }
    }

    // Filter non-interactive sudo failures (these are from scripts/automation)
    if service_lower == "sudo" {
        if message_lower.contains("conversation failed")
            || message_lower.contains("auth could not identify password") {
            return false; // Non-interactive sudo attempt, not user error
        }
    }

    // Filter transient systemd-logind messages
    if service_lower.contains("systemd-logind") {
        if message_lower.contains("watching system buttons") {
            return false; // Transient message, not an error
        }
    }

    // Keep everything else - it's likely actionable
    true
}

/// Get previous version from a cache file
fn get_previous_version() -> Option<String> {
    use std::fs;
    let cache_path = "/tmp/anna_last_version.txt";
    let current_version = env!("CARGO_PKG_VERSION");

    // Try to read previous version
    let prev_version = fs::read_to_string(cache_path).ok();

    // Always save current version for next time
    let _ = fs::write(cache_path, current_version);

    // Return previous version if it existed and was different
    prev_version.map(|v| v.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_reflection_format() {
        let summary = ReflectionSummaryData {
            items: vec![],
            generated_at: Utc::now(),
        };

        let formatted = format_reflection(&summary, false);
        assert!(formatted.contains("no significant changes"));
    }

    #[test]
    fn test_reflection_with_items() {
        let summary = ReflectionSummaryData {
            items: vec![ReflectionItemData {
                severity: ReflectionSeverity::Notice,
                category: "upgrade".to_string(),
                title: "Anna updated to v6.7.0".to_string(),
                details: "Previously v6.5.0".to_string(),
                since_timestamp: None,
            }],
            generated_at: Utc::now(),
        };

        let formatted = format_reflection(&summary, false);
        assert!(formatted.contains("Anna reflection"));
        assert!(formatted.contains("[NOTICE]"));
        assert!(formatted.contains("upgrade"));
        assert!(formatted.contains("6.7.0"));
    }

    #[test]
    fn test_severity_labels() {
        assert_eq!(severity_label(ReflectionSeverity::Info), "INFO");
        assert_eq!(severity_label(ReflectionSeverity::Notice), "NOTICE");
        assert_eq!(severity_label(ReflectionSeverity::Warning), "WARNING");
        assert_eq!(severity_label(ReflectionSeverity::Critical), "CRITICAL");
    }
}
