// v0.0.537: Query History Tracker Formatting (Phase 113)
// Formatting functions for displaying query history

use super::tracker::QueryHistoryTracker;
use super::types::QueryRecord;

/// Format query record
pub fn format_query(record: &QueryRecord) -> String {
    let mut output = format!(
        "Query {} [{}]\n  \"{}\"\n  Category: {} | Outcome: {}\n",
        record.id, record.timestamp.format("%Y-%m-%d %H:%M"),
        record.query_text, record.category, record.outcome
    );

    if let Some(time) = record.response_time_ms {
        output.push_str(&format!("  Response: {}ms\n", time));
    }

    if record.similar_count > 0 {
        output.push_str(&format!("  Similar queries: {}\n", record.similar_count));
    }

    output
}

/// Format tracker summary
pub fn format_tracker_summary(tracker: &QueryHistoryTracker) -> String {
    let mut output = String::new();
    output.push_str("=== Query History ===\n\n");

    output.push_str(&format!("Total Queries: {}\n", tracker.total()));

    if let Some(avg) = tracker.average_response_time_ms() {
        output.push_str(&format!("Avg Response Time: {}ms\n", avg));
    }

    output.push_str("\nBy Category:\n");
    for (cat, count) in tracker.category_stats().iter().take(5) {
        output.push_str(&format!("  {}: {}\n", cat, count));
    }

    output.push_str("\nBy Outcome:\n");
    for (outcome, count) in tracker.resolution_stats() {
        output.push_str(&format!("  {}: {}\n", outcome, count));
    }

    let repeated = tracker.repeated_queries();
    if !repeated.is_empty() {
        output.push_str(&format!("\nRepeated Queries: {}\n", repeated.len()));
    }

    output
}
