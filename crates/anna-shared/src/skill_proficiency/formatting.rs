// v0.0.527: Skill Proficiency Tracker (Phase 103)
// Display formatting functions

use super::tracker::SkillProficiencyTracker;
use super::types::SkillRecord;

/// Format skill for display
pub fn format_skill(skill: &SkillRecord) -> String {
    format!(
        "{} [{}] - {} ({} XP)\n  Uses: {} | Success: {:.1}% | Domain: {}",
        skill.name,
        skill.level(),
        if let Some(next) = skill.xp_to_next_level() {
            format!("{} XP to next level", next)
        } else {
            "Max level!".to_string()
        },
        skill.xp,
        skill.times_used,
        skill.success_rate(),
        skill.domain
    )
}

/// Format skill compact
pub fn format_skill_compact(skill: &SkillRecord) -> String {
    format!(
        "{}: {} ({} XP, {:.0}% success)",
        skill.name,
        skill.level(),
        skill.xp,
        skill.success_rate()
    )
}

/// Format skill oneline
pub fn format_skill_oneline(skill: &SkillRecord) -> String {
    format!("{} [{}]", skill.name, skill.level())
}

/// Format tracker summary
pub fn format_tracker_summary(tracker: &SkillProficiencyTracker) -> String {
    let mut output = String::new();
    output.push_str("=== Skill Proficiency Summary ===\n\n");

    output.push_str(&format!("Total Skills: {}\n", tracker.total_skills()));
    output.push_str(&format!("Total XP: {}\n", tracker.total_xp()));

    if let Some(avg) = tracker.average_level() {
        output.push_str(&format!("Average Level: {}\n", avg));
    }

    output.push_str("\n--- Top Skills ---\n");
    for skill in tracker.top_skills(5) {
        output.push_str(&format!("  {}\n", format_skill_compact(skill)));
    }

    let needs_practice = tracker.needs_practice(70.0);
    if !needs_practice.is_empty() {
        output.push_str("\n--- Needs Practice ---\n");
        for skill in needs_practice.iter().take(3) {
            output.push_str(&format!(
                "  {} ({:.0}% success)\n",
                skill.name,
                skill.success_rate()
            ));
        }
    }

    output
}
