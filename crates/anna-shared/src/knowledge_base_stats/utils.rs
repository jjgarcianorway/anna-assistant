//! Utility functions for knowledge base analysis

use super::stats::KnowledgeBaseStats;

/// Generate a knowledge insight
pub fn knowledge_insight(stats: &KnowledgeBaseStats) -> Option<String> {
    if stats.total_entries == 0 {
        return None;
    }

    // Check for interesting patterns
    if stats.learned_count() > 50 {
        return Some(format!(
            "Impressive! Anna has learned {} things beyond the initial knowledge.",
            stats.learned_count()
        ));
    }

    if stats.user_taught_count() > 10 {
        return Some(format!(
            "You've taught Anna {} things directly. Great collaboration!",
            stats.user_taught_count()
        ));
    }

    if stats.avg_uses() > 5.0 {
        return Some(format!(
            "Knowledge is being put to good use! Average {:.1} uses per entry.",
            stats.avg_uses()
        ));
    }

    if let Some((topic, count)) = stats.top_topic() {
        return Some(format!(
            "Most knowledge is about {} ({} entries).",
            topic, count
        ));
    }

    Some(format!(
        "Anna's knowledge base contains {} entries.",
        stats.total_entries
    ))
}

/// Check if query is asking about knowledge base
pub fn is_knowledge_stats_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "knowledge base",
        "how much do you know",
        "what do you know",
        "knowledge stats",
        "recipe count",
        "how many recipes",
        "learned knowledge",
        "knowledge size",
        "your knowledge",
    ];

    keywords.iter().any(|kw| q.contains(kw))
}

/// Knowledge health assessment
pub fn knowledge_health(stats: &KnowledgeBaseStats) -> &'static str {
    if stats.total_entries == 0 {
        return "Empty - No knowledge yet";
    }

    if stats.stale_percent() > 50.0 {
        return "Needs refresh - Many stale entries";
    }

    if stats.learned_count() > 100 {
        return "Thriving - Rich learned knowledge";
    }

    if stats.avg_uses() > 3.0 {
        return "Active - Knowledge is well-used";
    }

    "Growing - Building knowledge"
}
