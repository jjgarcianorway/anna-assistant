// v0.0.527: Skill Proficiency Tracker (Phase 103)
// Unit tests

#[cfg(test)]
mod tests {
    use super::super::tracker::SkillProficiencyTracker;
    use super::super::types::{ProficiencyLevel, SkillDomain, SkillRecord};
    use super::super::utils::is_skill_query;
    use super::super::utils::skill_fun_fact;

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
