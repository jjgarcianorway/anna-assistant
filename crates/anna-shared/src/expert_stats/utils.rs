//! Utility functions for expert statistics.

use super::tracker::ExpertStatsTracker;

/// Generate fun fact about expert stats
pub fn expert_stats_fun_fact(tracker: &ExpertStatsTracker) -> Option<String> {
    if tracker.total_tickets() < 5 {
        return None;
    }

    let facts = vec![
        format!(
            "Anna handles {:.0}% of requests independently - {}!",
            tracker.anna_share(),
            if tracker.anna_share() > 50.0 {
                "she's becoming an expert"
            } else {
                "teamwork at its finest"
            }
        ),
        format!(
            "Juniors resolved {} tickets, seniors handled {} - great balance!",
            tracker.junior_total, tracker.senior_total
        ),
        tracker
            .most_reliable()
            .map(|(id, conf)| {
                let name = tracker
                    .experts
                    .get(id)
                    .map(|e| e.name.clone())
                    .unwrap_or_else(|| id.to_string());
                format!("{} has {:.0}% confidence - most reliable expert!", name, conf * 100.0)
            })
            .unwrap_or_else(|| "The team maintains high standards!".to_string()),
        format!(
            "{} experts working together across {} tickets!",
            tracker.experts.len(),
            tracker.total_tickets()
        ),
    ];

    let index = (tracker.total_tickets() as usize) % facts.len();
    Some(facts[index].clone())
}

/// Check if query is asking about expert stats
pub fn is_expert_stats_query(query: &str) -> bool {
    let lower = query.to_lowercase();

    let patterns = [
        "expert stats",
        "who closed",
        "tickets per expert",
        "top performer",
        "junior stats",
        "senior stats",
        "expert performance",
    ];

    patterns.iter().any(|p| lower.contains(p))
}
