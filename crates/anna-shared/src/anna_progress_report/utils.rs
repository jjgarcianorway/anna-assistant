//! Utility functions for progress reporting
//!
//! Includes trend calculation, change percentage, milestone generation,
//! and query detection utilities.

use super::types::{Milestone, Trend};

/// Calculate trend from two values
pub fn calculate_trend(current: f64, previous: f64) -> Trend {
    let change = (current - previous).abs();
    let threshold = previous.abs() * 0.05; // 5% threshold

    if current > previous + threshold {
        Trend::Up
    } else if current < previous - threshold {
        Trend::Down
    } else {
        Trend::Stable
    }
}

/// Calculate percentage change
pub fn calculate_change_percent(current: f64, previous: f64) -> f64 {
    if previous == 0.0 {
        if current > 0.0 {
            return 100.0;
        }
        return 0.0;
    }
    ((current - previous) / previous) * 100.0
}

/// Generate default milestones for Anna
pub fn default_milestones() -> Vec<Milestone> {
    vec![
        Milestone::new("First Ticket", "Handle your first support ticket", 1),
        Milestone::new("Getting Started", "Handle 10 tickets", 10),
        Milestone::new("Finding Rhythm", "Handle 50 tickets", 50),
        Milestone::new("Century", "Handle 100 tickets", 100),
        Milestone::new("Seasoned Pro", "Handle 500 tickets", 500),
        Milestone::new("First Recipe", "Learn your first recipe", 1),
        Milestone::new("Recipe Collection", "Learn 10 recipes", 10),
        Milestone::new("Recipe Master", "Learn 50 recipes", 50),
        Milestone::new("Independence Day", "Solve 10 tickets without help", 10),
        Milestone::new("Solo Artist", "Solve 50 tickets without help", 50),
    ]
}

/// Check if query is asking about progress
pub fn is_progress_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "progress report",
        "my progress",
        "anna progress",
        "how am i doing",
        "how are we doing",
        "learning progress",
        "show progress",
        "what have you learned",
        "improvements",
        "milestones",
        "achievements",
    ];

    keywords.iter().any(|kw| q.contains(kw))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_trend() {
        assert_eq!(calculate_trend(100.0, 80.0), Trend::Up);
        assert_eq!(calculate_trend(80.0, 100.0), Trend::Down);
        assert_eq!(calculate_trend(100.0, 99.0), Trend::Stable);
    }

    #[test]
    fn test_calculate_change_percent() {
        assert!((calculate_change_percent(150.0, 100.0) - 50.0).abs() < 0.1);
        assert!((calculate_change_percent(50.0, 100.0) - (-50.0)).abs() < 0.1);
        assert_eq!(calculate_change_percent(100.0, 0.0), 100.0);
    }

    #[test]
    fn test_default_milestones() {
        let milestones = default_milestones();
        assert!(!milestones.is_empty());
        assert!(milestones.iter().any(|m| m.name == "First Ticket"));
    }

    #[test]
    fn test_is_progress_query() {
        assert!(is_progress_query("show me my progress report"));
        assert!(is_progress_query("how am i doing?"));
        assert!(is_progress_query("what have you learned so far?"));
        assert!(is_progress_query("show milestones"));
        assert!(!is_progress_query("how do I install vim?"));
    }
}
