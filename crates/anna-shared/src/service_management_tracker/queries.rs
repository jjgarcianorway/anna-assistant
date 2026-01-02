//! Service tracker query utilities - Phase 81

use super::tracker::ServiceTracker;

/// Check if query is about service management
pub fn is_service_tracker_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "service history",
        "service operations",
        "managed services",
        "services managed",
        "service tracker",
        "restarted services",
        "stopped services",
        "services have you",
        "service management",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Generate fun fact about service management
pub fn service_fun_fact(tracker: &ServiceTracker) -> String {
    if tracker.records.is_empty() {
        return "No service operations yet!".to_string();
    }

    let facts = [
        format!(
            "Anna has performed {} service operations.",
            tracker.total_count()
        ),
        format!(
            "{} unique services have been managed.",
            tracker.unique_services()
        ),
        {
            if let Some((service, count)) = tracker.most_managed() {
                format!("{} is the most managed service ({} operations).", service, count)
            } else {
                "No service stats yet.".to_string()
            }
        },
        format!(
            "Service management success rate: {:.1}%.",
            tracker.success_rate()
        ),
    ];

    facts[tracker.total_count() % facts.len()].clone()
}
