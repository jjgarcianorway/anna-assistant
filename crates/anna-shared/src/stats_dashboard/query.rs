//! Query detection utilities for dashboard.

use super::types::DashboardSection;

/// Check if query is asking for dashboard
pub fn is_dashboard_query(query: &str) -> bool {
    let lower = query.to_lowercase();

    let patterns = [
        "dashboard",
        "all stats",
        "full stats",
        "overview",
        "summary stats",
        "stat summary",
        "show stats",
    ];

    patterns.iter().any(|p| lower.contains(p))
}

/// Check which section is being requested
pub fn detect_section(query: &str) -> Option<DashboardSection> {
    let lower = query.to_lowercase();

    if lower.contains("resolution") || lower.contains("time") {
        Some(DashboardSection::Resolutions)
    } else if lower.contains("interaction") || lower.contains("communication") {
        Some(DashboardSection::Interactions)
    } else if lower.contains("expert") || lower.contains("specialist") {
        Some(DashboardSection::Experts)
    } else if lower.contains("recipe") {
        Some(DashboardSection::Recipes)
    } else if lower.contains("response") || lower.contains("length") {
        Some(DashboardSection::Responses)
    } else if lower.contains("question") || lower.contains("repeated") {
        Some(DashboardSection::Questions)
    } else if lower.contains("status") || lower.contains("health") {
        Some(DashboardSection::Status)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_dashboard_query() {
        assert!(is_dashboard_query("show dashboard"));
        assert!(is_dashboard_query("all stats"));
        assert!(is_dashboard_query("overview"));

        assert!(!is_dashboard_query("install vim"));
    }

    #[test]
    fn test_detect_section() {
        assert_eq!(detect_section("resolution times"), Some(DashboardSection::Resolutions));
        assert_eq!(detect_section("expert stats"), Some(DashboardSection::Experts));
        assert_eq!(detect_section("recipe info"), Some(DashboardSection::Recipes));
        assert_eq!(detect_section("random query"), None);
    }
}
