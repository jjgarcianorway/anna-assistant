//! Formatting functions for error summaries

use super::{ErrorSummary, ErrorSeverity};

/// Format a timestamp as a human-readable string
pub fn format_error_timestamp(ts: u64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    if ts > now {
        return "just now".to_string();
    }

    let diff = now - ts;
    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else {
        format!("{}d ago", diff / 86400)
    }
}

/// Format error summary as full display
pub fn format_error_summary(summary: &ErrorSummary) -> String {
    let mut lines = Vec::new();

    lines.push("=== Error Summary ===".to_string());
    lines.push(String::new());

    // Overview
    let unack = summary.unacknowledged_count();
    let critical = summary.critical_count();

    if critical > 0 {
        lines.push(format!("CRITICAL ERRORS: {}", critical));
    }
    if unack > 0 {
        lines.push(format!("Unacknowledged: {}", unack));
    }
    lines.push(format!("Total Recorded: {}", summary.total_recorded));
    lines.push(String::new());

    // Recent errors by severity
    let recent = summary.recent(15);

    if recent.is_empty() {
        lines.push("No errors recorded.".to_string());
    } else {
        lines.push("--- Recent Issues ---".to_string());

        for error in recent {
            let symbol = error.severity.symbol();
            let time = format_error_timestamp(error.timestamp);
            let ack = if error.acknowledged { " (ack)" } else { "" };
            let count = if error.occurrence_count > 1 {
                format!(" (x{})", error.occurrence_count)
            } else {
                String::new()
            };

            // Truncate message for display
            let msg = if error.message.len() > 60 {
                format!("{}...", &error.message[..57])
            } else {
                error.message.clone()
            };

            lines.push(format!("{} {}{}{}", symbol, msg, count, ack));
            lines.push(format!(
                "    {} | {} | {}",
                error.category.display(),
                error.severity.display(),
                time
            ));

            if let Some(ref action) = error.suggested_action {
                let action_display = if action.len() > 50 {
                    format!("{}...", &action[..47])
                } else {
                    action.clone()
                };
                lines.push(format!("    Suggestion: {}", action_display));
            }
        }
    }

    // Breakdown by severity
    if !summary.by_severity.is_empty() {
        lines.push(String::new());
        lines.push("--- By Severity ---".to_string());
        for (sev, count) in &summary.by_severity {
            lines.push(format!("  {}: {}", sev, count));
        }
    }

    // Breakdown by category
    if !summary.by_category.is_empty() {
        lines.push(String::new());
        lines.push("--- By Category ---".to_string());
        for (cat, count) in &summary.by_category {
            lines.push(format!("  {}: {}", cat, count));
        }
    }

    lines.join("\n")
}

/// Format error summary compact (for greetings/status)
pub fn format_error_summary_compact(summary: &ErrorSummary) -> String {
    let critical = summary.critical_count();
    let unack = summary.unacknowledged_count();

    if critical > 0 {
        return format!(
            "[!!] {} critical error{} need attention",
            critical,
            if critical == 1 { "" } else { "s" }
        );
    }

    if unack > 0 {
        return format!(
            "[!] {} issue{} pending review",
            unack,
            if unack == 1 { "" } else { "s" }
        );
    }

    "[OK] No issues detected".to_string()
}

/// Format error summary one-line
pub fn format_error_summary_oneline(summary: &ErrorSummary) -> String {
    let critical = summary.critical_count();
    let errors = summary.by_severity.get("ERROR").unwrap_or(&0);
    let warnings = summary.by_severity.get("WARNING").unwrap_or(&0);

    format!(
        "Issues: {} critical, {} errors, {} warnings",
        critical, errors, warnings
    )
}

/// Generate a health-related message based on error summary
pub fn error_health_message(summary: &ErrorSummary) -> String {
    let critical = summary.critical_count();
    let unack = summary.unacknowledged_count();

    if critical > 0 {
        return "System health: CRITICAL - Immediate attention required!".to_string();
    }

    if unack > 5 {
        return "System health: DEGRADED - Multiple issues detected".to_string();
    }

    if unack > 0 {
        return "System health: FAIR - Minor issues present".to_string();
    }

    "System health: GOOD - All systems operational".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error_summary_display::{ErrorCategory, ErrorEntry};

    #[test]
    fn test_format_error_timestamp() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        assert_eq!(format_error_timestamp(now), "just now");
        assert!(format_error_timestamp(now - 120).contains("m ago"));
        assert!(format_error_timestamp(now - 7200).contains("h ago"));
        assert!(format_error_timestamp(now - 172800).contains("d ago"));
    }

    #[test]
    fn test_format_error_summary() {
        let mut summary = ErrorSummary::new();
        let error = ErrorEntry::new(
            "ERR001",
            "Disk space low",
            ErrorSeverity::Warning,
            ErrorCategory::System,
            1000,
        );
        summary.add(error);

        let output = format_error_summary(&summary);
        assert!(output.contains("Error Summary"));
        assert!(output.contains("Disk space low"));
    }

    #[test]
    fn test_format_error_summary_compact() {
        let mut summary = ErrorSummary::new();

        // Empty summary
        assert!(format_error_summary_compact(&summary).contains("No issues"));

        // Add a warning
        let warning = ErrorEntry::new(
            "WARN001",
            "Warning",
            ErrorSeverity::Warning,
            ErrorCategory::Config,
            1000,
        );
        summary.add(warning);
        assert!(format_error_summary_compact(&summary).contains("pending review"));

        // Add a critical
        let critical = ErrorEntry::new(
            "CRIT001",
            "Critical",
            ErrorSeverity::Critical,
            ErrorCategory::System,
            2000,
        );
        summary.add(critical);
        assert!(format_error_summary_compact(&summary).contains("critical error"));
    }

    #[test]
    fn test_error_health_message() {
        let mut summary = ErrorSummary::new();

        // Good health
        assert!(error_health_message(&summary).contains("GOOD"));

        // Fair health
        let warning = ErrorEntry::new(
            "WARN001",
            "Warning",
            ErrorSeverity::Warning,
            ErrorCategory::Config,
            1000,
        );
        summary.add(warning);
        assert!(error_health_message(&summary).contains("FAIR"));

        // Critical health
        let critical = ErrorEntry::new(
            "CRIT001",
            "Critical",
            ErrorSeverity::Critical,
            ErrorCategory::System,
            2000,
        );
        summary.add(critical);
        assert!(error_health_message(&summary).contains("CRITICAL"));
    }
}
