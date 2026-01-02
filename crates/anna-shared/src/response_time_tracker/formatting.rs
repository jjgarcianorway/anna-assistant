// v0.0.538: Response Time Tracker - Formatting (Phase 114)
// Formatting utilities and helper functions

use super::record::ResponseTimeRecord;
use super::tracker::ResponseTimeTracker;

/// Format response time record
pub fn format_response_time(record: &ResponseTimeRecord) -> String {
    let mut output = format!(
        "Response {} [{}]\n  Time: {}ms | Words: {}\n  Type: {} | Complexity: {}\n",
        record.id, record.timestamp.format("%Y-%m-%d %H:%M"),
        record.time_ms, record.word_count,
        record.response_type, record.complexity
    );

    if let Some(tokens) = record.token_count {
        output.push_str(&format!("  Tokens: {}\n", tokens));
    }

    output.push_str(&format!("  Speed: {:.1} words/sec\n", record.words_per_second()));
    output
}

/// Format tracker summary
pub fn format_tracker_summary(tracker: &ResponseTimeTracker) -> String {
    let mut output = String::new();
    output.push_str("=== Response Time Stats ===\n\n");

    output.push_str(&format!("Total Responses: {}\n", tracker.total()));

    if let Some(avg) = tracker.average_time_ms() {
        output.push_str(&format!("Average Time: {}ms\n", avg));
    }

    if let Some(avg) = tracker.average_word_count() {
        output.push_str(&format!("Average Words: {}\n", avg));
    }

    if let Some(shortest) = tracker.shortest_reply() {
        output.push_str(&format!("Shortest Reply: {} words ({})\n",
            shortest.word_count, shortest.id));
    }

    if let Some(longest) = tracker.longest_reply() {
        output.push_str(&format!("Longest Reply: {} words ({})\n",
            longest.word_count, longest.id));
    }

    if let Some(p95) = tracker.percentile_time(95) {
        output.push_str(&format!("95th Percentile: {}ms\n", p95));
    }

    output
}

/// Check if query is about response times
pub fn is_response_time_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("response time")
        || lower.contains("how long")
        || lower.contains("fastest")
        || lower.contains("slowest")
        || lower.contains("shortest reply")
        || lower.contains("longest reply")
}

/// Fun fact about response times
pub fn response_time_fun_fact() -> &'static str {
    "Anna tracks every response time! The 'shortest reply' might be a quick yes/no, while the 'longest reply' could be a full troubleshooting guide."
}
