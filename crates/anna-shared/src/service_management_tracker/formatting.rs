//! Service tracker formatting functions - Phase 81

use super::tracker::ServiceTracker;

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
