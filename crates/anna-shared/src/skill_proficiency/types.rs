// v0.0.527: Skill Proficiency Tracker (Phase 103)
// Tracks Anna's learned skills and proficiency levels for continuous improvement

use serde::{Deserialize, Serialize};

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
