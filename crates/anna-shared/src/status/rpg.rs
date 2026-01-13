//! RPG-style statistics for gamification.

use serde::{Deserialize, Serialize};

/// v0.2.7: RPG-style statistics for gamification
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RpgStats {
    /// Experience points (0-100, non-linear scaling)
    pub xp: u32,
    /// Current title based on XP
    pub title: String,
    /// Total questions asked
    pub total_questions: u64,
    /// Questions answered without LLM (fast-path/instant)
    pub instant_answers: u64,
    /// Questions answered from memory/recipes
    pub memory_answers: u64,
    /// Questions that needed full LLM processing
    pub llm_answers: u64,
    /// Average response time in milliseconds
    pub avg_response_ms: u64,
    /// Fastest response time in milliseconds
    pub fastest_response_ms: u64,
    /// Slowest response time in milliseconds
    pub slowest_response_ms: u64,
    /// Number of recipes learned
    pub recipes_learned: u32,
    /// Reliability score (0.0-1.0)
    pub reliability: f32,
    /// When Anna was first installed
    pub installed_at: Option<String>,
    /// Total uptime since installation (seconds)
    pub total_uptime_secs: u64,
}

impl RpgStats {
    /// Calculate XP from stats (non-linear, 0-100)
    pub fn calculate_xp(&mut self) {
        // XP formula: weighted combination of activity metrics
        // - Questions answered: logarithmic scaling (each doubling adds ~10 XP)
        // - Memory efficiency: bonus for not needing LLM
        // - Recipes learned: linear bonus
        // - Reliability: multiplier

        let questions_xp = if self.total_questions > 0 {
            (self.total_questions as f64).log2() * 10.0
        } else {
            0.0
        };

        let efficiency = if self.total_questions > 0 {
            (self.instant_answers + self.memory_answers) as f64 / self.total_questions as f64
        } else {
            0.0
        };
        let efficiency_bonus = efficiency * 20.0; // Up to 20 XP for 100% efficiency

        let recipe_bonus = (self.recipes_learned as f64).min(20.0); // Max 20 XP from recipes

        let reliability_mult = 0.5 + (self.reliability as f64 * 0.5); // 0.5 - 1.0 multiplier

        let raw_xp = (questions_xp + efficiency_bonus + recipe_bonus) * reliability_mult;
        self.xp = (raw_xp as u32).min(100);
        self.title = Self::get_title(self.xp);
    }

    /// Get title based on XP level (fun RPG-style progression)
    pub fn get_title(xp: u32) -> String {
        match xp {
            0..=4 => "Novice Apprentice".to_string(),
            5..=9 => "Eager Learner".to_string(),
            10..=19 => "Junior Technician".to_string(),
            20..=29 => "Curious Explorer".to_string(),
            30..=39 => "Competent Assistant".to_string(),
            40..=49 => "Skilled Operator".to_string(),
            50..=59 => "Senior Specialist".to_string(),
            60..=69 => "Expert Analyst".to_string(),
            70..=79 => "Master Troubleshooter".to_string(),
            80..=89 => "IT Sage".to_string(),
            90..=94 => "System Whisperer".to_string(),
            95..=99 => "Arch Wizard".to_string(),
            100 => "Omniscient Oracle".to_string(),
            _ => "Unknown".to_string(),
        }
    }

    /// Get XP bar visualization (ASCII only)
    pub fn xp_bar(&self) -> String {
        let filled = (self.xp as usize) / 5; // 20 character bar
        let empty = 20 - filled;
        format!(
            "[{}{}] {}%",
            "=".repeat(filled),
            "-".repeat(empty),
            self.xp
        )
    }
}
