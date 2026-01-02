// v0.0.530: Knowledge Citation Tracker - Formatting
// Display and formatting functions for citations and tracker summaries

use super::record::CitationRecord;
use super::tracker::KnowledgeCitationTracker;

/// Format citation for display
pub fn format_citation(cit: &CitationRecord) -> String {
    format!(
        "{} [{}]\n  Source: {} | Reliability: {}\n  Location: {}\n  Used: {} times\n  Snippet: {}...",
        cit.id,
        cit.title,
        cit.source,
        cit.reliability,
        cit.location,
        cit.used_count,
        if cit.snippet.len() > 50 {
            &cit.snippet[..50]
        } else {
            &cit.snippet
        }
    )
}

/// Format citation compact
pub fn format_citation_compact(cit: &CitationRecord) -> String {
    format!(
        "{}: {} [{}] (used {} times)",
        cit.id, cit.title, cit.source, cit.used_count
    )
}

/// Format citation oneline
pub fn format_citation_oneline(cit: &CitationRecord) -> String {
    format!("[{}] {}", cit.source, cit.title)
}

/// Format tracker summary
pub fn format_tracker_summary(tracker: &KnowledgeCitationTracker) -> String {
    let mut output = String::new();
    output.push_str("=== Knowledge Citations ===\n\n");

    output.push_str(&format!("Total Citations: {}\n\n", tracker.total()));

    output.push_str("--- By Source ---\n");
    for (source, count) in tracker.source_stats() {
        output.push_str(&format!("  {}: {}\n", source, count));
    }

    output.push_str("\n--- Most Used ---\n");
    for cit in tracker.most_used(5) {
        output.push_str(&format!("  {}\n", format_citation_compact(cit)));
    }

    output
}
