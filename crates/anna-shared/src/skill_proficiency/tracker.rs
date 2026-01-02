// v0.0.527: Skill Proficiency Tracker (Phase 103)
// Main tracker implementation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::{ProficiencyLevel, SkillDomain, SkillRecord};

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
