// v0.0.527: Skill Proficiency Tracker (Phase 103)
// Tracks Anna's learned skills and proficiency levels for continuous improvement

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Skill domain categories
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillDomain {
    SystemAdmin,
    Networking,
    Security,
    Storage,
    Audio,
    Video,
    Desktop,
    Scripting,
    Troubleshooting,
    UserSupport,
}

impl Default for SkillDomain {
    fn default() -> Self {
        Self::SystemAdmin
    }
}

impl std::fmt::Display for SkillDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SystemAdmin => write!(f, "System Admin"),
            Self::Networking => write!(f, "Networking"),
            Self::Security => write!(f, "Security"),
            Self::Storage => write!(f, "Storage"),
            Self::Audio => write!(f, "Audio"),
            Self::Video => write!(f, "Video"),
            Self::Desktop => write!(f, "Desktop"),
            Self::Scripting => write!(f, "Scripting"),
            Self::Troubleshooting => write!(f, "Troubleshooting"),
            Self::UserSupport => write!(f, "User Support"),
        }
    }
}

/// Proficiency level for a skill
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ProficiencyLevel {
    Novice,
    Beginner,
    Intermediate,
    Advanced,
    Expert,
    Master,
}

impl Default for ProficiencyLevel {
    fn default() -> Self {
        Self::Novice
    }
}

impl std::fmt::Display for ProficiencyLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Novice => write!(f, "Novice"),
            Self::Beginner => write!(f, "Beginner"),
            Self::Intermediate => write!(f, "Intermediate"),
            Self::Advanced => write!(f, "Advanced"),
            Self::Expert => write!(f, "Expert"),
            Self::Master => write!(f, "Master"),
        }
    }
}

impl ProficiencyLevel {
    /// Get XP threshold for this level
    pub fn xp_threshold(&self) -> u32 {
        match self {
            Self::Novice => 0,
            Self::Beginner => 100,
            Self::Intermediate => 500,
            Self::Advanced => 1500,
            Self::Expert => 4000,
            Self::Master => 10000,
        }
    }

    /// Get level from XP
    pub fn from_xp(xp: u32) -> Self {
        if xp >= 10000 {
            Self::Master
        } else if xp >= 4000 {
            Self::Expert
        } else if xp >= 1500 {
            Self::Advanced
        } else if xp >= 500 {
            Self::Intermediate
        } else if xp >= 100 {
            Self::Beginner
        } else {
            Self::Novice
        }
    }
}

/// Individual skill record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRecord {
    pub name: String,
    pub domain: SkillDomain,
    pub xp: u32,
    pub times_used: u32,
    pub successes: u32,
    pub failures: u32,
    pub last_used: Option<String>,
    pub learned_at: String,
}

impl SkillRecord {
    /// Create a new skill record
    pub fn new(name: &str, domain: SkillDomain, learned_at: &str) -> Self {
        Self {
            name: name.to_string(),
            domain,
            xp: 0,
            times_used: 0,
            successes: 0,
            failures: 0,
            last_used: None,
            learned_at: learned_at.to_string(),
        }
    }

    /// Get current proficiency level
    pub fn level(&self) -> ProficiencyLevel {
        ProficiencyLevel::from_xp(self.xp)
    }

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        if self.times_used == 0 {
            0.0
        } else {
            (self.successes as f64 / self.times_used as f64) * 100.0
        }
    }

    /// Record skill usage
    pub fn record_use(&mut self, success: bool, timestamp: &str) {
        self.times_used += 1;
        self.last_used = Some(timestamp.to_string());

        if success {
            self.successes += 1;
            // XP gain scales with level
            let base_xp = 10u32;
            let bonus = match self.level() {
                ProficiencyLevel::Novice => 5,
                ProficiencyLevel::Beginner => 4,
                ProficiencyLevel::Intermediate => 3,
                ProficiencyLevel::Advanced => 2,
                ProficiencyLevel::Expert => 1,
                ProficiencyLevel::Master => 0,
            };
            self.xp = self.xp.saturating_add(base_xp + bonus);
        } else {
            self.failures += 1;
            // Small XP loss on failure (learn from mistakes)
            self.xp = self.xp.saturating_sub(2);
        }
    }

    /// XP to next level
    pub fn xp_to_next_level(&self) -> Option<u32> {
        let current = self.level();
        let next = match current {
            ProficiencyLevel::Novice => Some(ProficiencyLevel::Beginner),
            ProficiencyLevel::Beginner => Some(ProficiencyLevel::Intermediate),
            ProficiencyLevel::Intermediate => Some(ProficiencyLevel::Advanced),
            ProficiencyLevel::Advanced => Some(ProficiencyLevel::Expert),
            ProficiencyLevel::Expert => Some(ProficiencyLevel::Master),
            ProficiencyLevel::Master => None,
        };

        next.map(|n| n.xp_threshold().saturating_sub(self.xp))
    }
}

/// Skill proficiency tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillProficiencyTracker {
    skills: HashMap<String, SkillRecord>,
}

impl SkillProficiencyTracker {
    /// Create a new tracker
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    /// Learn a new skill
    pub fn learn(&mut self, name: &str, domain: SkillDomain, timestamp: &str) {
        if !self.skills.contains_key(name) {
            self.skills
                .insert(name.to_string(), SkillRecord::new(name, domain, timestamp));
        }
    }

    /// Use a skill
    pub fn use_skill(&mut self, name: &str, success: bool, timestamp: &str) {
        if let Some(skill) = self.skills.get_mut(name) {
            skill.record_use(success, timestamp);
        }
    }

    /// Get a skill
    pub fn get(&self, name: &str) -> Option<&SkillRecord> {
        self.skills.get(name)
    }

    /// Get skills by domain
    pub fn by_domain(&self, domain: &SkillDomain) -> Vec<&SkillRecord> {
        self.skills
            .values()
            .filter(|s| &s.domain == domain)
            .collect()
    }

    /// Get skills by proficiency level
    pub fn by_level(&self, level: ProficiencyLevel) -> Vec<&SkillRecord> {
        self.skills.values().filter(|s| s.level() == level).collect()
    }

    /// Get top skills by XP
    pub fn top_skills(&self, n: usize) -> Vec<&SkillRecord> {
        let mut skills: Vec<_> = self.skills.values().collect();
        skills.sort_by(|a, b| b.xp.cmp(&a.xp));
        skills.into_iter().take(n).collect()
    }

    /// Get skills needing practice (low success rate)
    pub fn needs_practice(&self, threshold: f64) -> Vec<&SkillRecord> {
        self.skills
            .values()
            .filter(|s| s.times_used >= 5 && s.success_rate() < threshold)
            .collect()
    }

    /// Total skills learned
    pub fn total_skills(&self) -> usize {
        self.skills.len()
    }

    /// Total XP across all skills
    pub fn total_xp(&self) -> u32 {
        self.skills.values().map(|s| s.xp).sum()
    }

    /// Average proficiency level
    pub fn average_level(&self) -> Option<ProficiencyLevel> {
        if self.skills.is_empty() {
            return None;
        }
        let avg_xp = self.total_xp() / self.skills.len() as u32;
        Some(ProficiencyLevel::from_xp(avg_xp))
    }

    /// Stats by domain
    pub fn domain_stats(&self) -> HashMap<SkillDomain, (usize, u32)> {
        let mut stats = HashMap::new();
        for skill in self.skills.values() {
            let entry = stats.entry(skill.domain.clone()).or_insert((0, 0));
            entry.0 += 1;
            entry.1 += skill.xp;
        }
        stats
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proficiency_from_xp() {
        assert_eq!(ProficiencyLevel::from_xp(0), ProficiencyLevel::Novice);
        assert_eq!(ProficiencyLevel::from_xp(99), ProficiencyLevel::Novice);
        assert_eq!(ProficiencyLevel::from_xp(100), ProficiencyLevel::Beginner);
        assert_eq!(ProficiencyLevel::from_xp(500), ProficiencyLevel::Intermediate);
        assert_eq!(ProficiencyLevel::from_xp(1500), ProficiencyLevel::Advanced);
        assert_eq!(ProficiencyLevel::from_xp(4000), ProficiencyLevel::Expert);
        assert_eq!(ProficiencyLevel::from_xp(10000), ProficiencyLevel::Master);
        assert_eq!(ProficiencyLevel::from_xp(99999), ProficiencyLevel::Master);
    }

    #[test]
    fn test_skill_record_creation() {
        let skill = SkillRecord::new("vim_config", SkillDomain::Desktop, "2024-01-01");
        assert_eq!(skill.name, "vim_config");
        assert_eq!(skill.xp, 0);
        assert_eq!(skill.level(), ProficiencyLevel::Novice);
    }

    #[test]
    fn test_skill_use_success() {
        let mut skill = SkillRecord::new("test", SkillDomain::SystemAdmin, "2024-01-01");
        skill.record_use(true, "2024-01-02");
        assert_eq!(skill.times_used, 1);
        assert_eq!(skill.successes, 1);
        assert!(skill.xp > 0);
    }

    #[test]
    fn test_skill_use_failure() {
        let mut skill = SkillRecord::new("test", SkillDomain::SystemAdmin, "2024-01-01");
        skill.xp = 10;
        skill.record_use(false, "2024-01-02");
        assert_eq!(skill.times_used, 1);
        assert_eq!(skill.failures, 1);
        assert!(skill.xp < 10);
    }

    #[test]
    fn test_success_rate() {
        let mut skill = SkillRecord::new("test", SkillDomain::Networking, "2024-01-01");
        skill.successes = 7;
        skill.failures = 3;
        skill.times_used = 10;
        assert!((skill.success_rate() - 70.0).abs() < 0.1);
    }

    #[test]
    fn test_tracker_learn() {
        let mut tracker = SkillProficiencyTracker::new();
        tracker.learn("pacman_update", SkillDomain::SystemAdmin, "2024-01-01");
        assert_eq!(tracker.total_skills(), 1);
        assert!(tracker.get("pacman_update").is_some());
    }

    #[test]
    fn test_tracker_use_skill() {
        let mut tracker = SkillProficiencyTracker::new();
        tracker.learn("test_skill", SkillDomain::Security, "2024-01-01");
        tracker.use_skill("test_skill", true, "2024-01-02");
        let skill = tracker.get("test_skill").unwrap();
        assert_eq!(skill.times_used, 1);
    }

    #[test]
    fn test_by_domain() {
        let mut tracker = SkillProficiencyTracker::new();
        tracker.learn("skill1", SkillDomain::Networking, "2024-01-01");
        tracker.learn("skill2", SkillDomain::Networking, "2024-01-01");
        tracker.learn("skill3", SkillDomain::Security, "2024-01-01");
        assert_eq!(tracker.by_domain(&SkillDomain::Networking).len(), 2);
    }

    #[test]
    fn test_top_skills() {
        let mut tracker = SkillProficiencyTracker::new();
        tracker.learn("low", SkillDomain::Audio, "2024-01-01");
        tracker.learn("high", SkillDomain::Video, "2024-01-01");
        for _ in 0..20 {
            tracker.use_skill("high", true, "2024-01-02");
        }
        let top = tracker.top_skills(1);
        assert_eq!(top[0].name, "high");
    }

    #[test]
    fn test_is_skill_query() {
        assert!(is_skill_query("What skills have I learned?"));
        assert!(is_skill_query("Show my proficiency level"));
        assert!(is_skill_query("How much XP do I have?"));
        assert!(!is_skill_query("Install vim"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = skill_fun_fact();
        assert!(fact.contains("10,000"));
    }
}
