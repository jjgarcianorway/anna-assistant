// v0.0.527: Skill Proficiency Tracker (Phase 103)
// Utility helper functions

/// Check if query is skill-related
pub fn is_skill_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("skill")
        || lower.contains("proficiency")
        || lower.contains("level")
        || lower.contains("expertise")
        || lower.contains("learn")
        || lower.contains("master")
        || lower.contains("xp")
        || lower.contains("experience")
}

/// Fun fact about skill learning
pub fn skill_fun_fact() -> &'static str {
    "It takes approximately 10,000 hours of deliberate practice to achieve mastery in a complex skill - Anna is getting there one ticket at a time!"
}
