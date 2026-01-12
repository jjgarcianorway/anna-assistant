//! RPG System - Anna's progression and gamification.
//! v0.0.999: Initial implementation
//!
//! Anna gains XP from resolving tickets. XP is non-linear like in RPG games.
//! Higher levels require exponentially more XP.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Anna's experience and progression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnaXP {
    /// Total XP earned
    pub total_xp: u64,
    /// Current level (1-100)
    pub level: u32,
    /// XP needed for next level
    pub xp_to_next: u64,
    /// Total tickets resolved
    pub tickets_resolved: u64,
    /// Tickets resolved by Anna alone (using recipes)
    pub resolved_by_anna: u64,
    /// Tickets resolved by specialists
    pub resolved_by_specialists: u64,
    /// Recipes learned
    pub recipes_learned: u64,
    /// Average reliability score (0.0-1.0)
    pub avg_reliability: f64,
    /// Longest resolution time (seconds)
    pub longest_resolution: u64,
    /// Shortest resolution time (seconds)
    pub shortest_resolution: u64,
    /// Questions by department
    pub questions_by_dept: std::collections::HashMap<String, u64>,
    /// Most consulted specialist
    pub most_consulted: Option<String>,
    /// Install timestamp
    pub installed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for AnnaXP {
    fn default() -> Self {
        Self {
            total_xp: 0,
            level: 1,
            xp_to_next: 100,
            tickets_resolved: 0,
            resolved_by_anna: 0,
            resolved_by_specialists: 0,
            recipes_learned: 0,
            avg_reliability: 0.0,
            longest_resolution: 0,
            shortest_resolution: u64::MAX,
            questions_by_dept: std::collections::HashMap::new(),
            most_consulted: None,
            installed_at: Some(chrono::Utc::now()),
        }
    }
}

impl AnnaXP {
    fn xp_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("anna/xp.json")
    }

    pub fn load() -> Self {
        let path = Self::xp_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(xp) = serde_json::from_str(&content) {
                    return xp;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::xp_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(path, content).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Calculate XP needed for a given level (non-linear, RPG style)
    pub fn xp_for_level(level: u32) -> u64 {
        // Formula: base * (level^1.5) + (level * 50)
        // Level 1: 100, Level 10: ~816, Level 50: ~6035, Level 100: ~15811
        let base = 100.0;
        let xp = base * (level as f64).powf(1.5) + (level as f64 * 50.0);
        xp as u64
    }

    /// Check and update level based on current XP
    fn update_level(&mut self) {
        while self.total_xp >= self.xp_to_next && self.level < 100 {
            self.level += 1;
            self.xp_to_next = Self::xp_for_level(self.level + 1);
        }
    }

    /// Add XP and update level
    pub fn add_xp(&mut self, amount: u64) {
        self.total_xp += amount;
        self.update_level();
    }

    /// Get current XP progress as percentage
    pub fn level_progress(&self) -> f64 {
        let prev_level_xp = if self.level == 1 { 0 } else { Self::xp_for_level(self.level) };
        let current = self.total_xp.saturating_sub(prev_level_xp);
        let needed = self.xp_to_next.saturating_sub(prev_level_xp);
        if needed == 0 { return 100.0; }
        (current as f64 / needed as f64 * 100.0).min(100.0)
    }
}

/// XP rewards for different actions
pub mod rewards {
    /// Ticket resolved by Anna using a recipe
    pub const RECIPE_USED: u64 = 5;
    /// Ticket resolved by junior specialist
    pub const JUNIOR_RESOLVED: u64 = 10;
    /// Ticket resolved by senior specialist
    pub const SENIOR_RESOLVED: u64 = 15;
    /// Escalation needed
    pub const ESCALATION_PENALTY: u64 = 2;
    /// New recipe learned
    pub const RECIPE_LEARNED: u64 = 25;
    /// Fast resolution (under 10 seconds)
    pub const FAST_RESOLUTION: u64 = 3;
    /// User expressed satisfaction
    pub const USER_HAPPY: u64 = 5;
}

/// Titles for different levels
pub fn get_title_for_level(level: u32) -> &'static str {
    match level {
        0..=5 => "Helpdesk Newbie",
        6..=10 => "Support Rookie",
        11..=15 => "Tech Apprentice",
        16..=20 => "Junior Analyst",
        21..=25 => "IT Assistant",
        26..=30 => "System Helper",
        31..=35 => "Tech Support Pro",
        36..=40 => "Senior Analyst",
        41..=45 => "IT Specialist",
        46..=50 => "System Expert",
        51..=55 => "Tech Guru",
        56..=60 => "IT Veteran",
        61..=65 => "System Master",
        66..=70 => "Tech Wizard",
        71..=75 => "IT Sage",
        76..=80 => "System Oracle",
        81..=85 => "Tech Legend",
        86..=90 => "IT Deity",
        91..=95 => "System Overlord",
        96..=99 => "Tech Transcendent",
        100 => "The One Who Knows All",
        _ => "Unknown Entity",
    }
}

/// Get a fun description for the current title
pub fn get_title_description(level: u32) -> &'static str {
    match level {
        0..=5 => "Just starting out, but eager to help!",
        6..=10 => "Learning the ropes, one ticket at a time.",
        11..=15 => "Getting the hang of this IT thing.",
        16..=20 => "Can handle most basic requests now.",
        21..=25 => "The go-to for everyday tech problems.",
        26..=30 => "Knows the system like the back of the hand.",
        31..=35 => "Rarely needs to escalate anymore.",
        36..=40 => "The specialists come to me for advice!",
        41..=45 => "One with the terminal, one with the code.",
        46..=50 => "Halfway to omniscience.",
        51..=55 => "The Arch Wiki fears me.",
        56..=60 => "Bugs flee at my presence.",
        61..=65 => "I dream in systemd unit files.",
        66..=70 => "The kernel sends me birthday cards.",
        71..=75 => "Linus would be proud.",
        76..=80 => "I see the Matrix now.",
        81..=85 => "Compiling wisdom since boot.",
        86..=90 => "The Arch Wiki quotes ME.",
        91..=95 => "I AM the documentation.",
        96..=99 => "One step from digital enlightenment.",
        100 => "I have achieved technical nirvana.",
        _ => "An enigma wrapped in a shell script.",
    }
}

// Global XP state
static XP: std::sync::RwLock<Option<AnnaXP>> = std::sync::RwLock::new(None);

fn get_xp_internal() -> AnnaXP {
    let guard = XP.read().unwrap();
    guard.clone().unwrap_or_else(|| {
        drop(guard);
        let xp = AnnaXP::load();
        let mut guard = XP.write().unwrap();
        *guard = Some(xp.clone());
        xp
    })
}

fn save_xp_internal(xp: &AnnaXP) {
    let mut guard = XP.write().unwrap();
    *guard = Some(xp.clone());
    let _ = xp.save();
}

/// Get Anna's current XP state
pub fn get_anna_xp() -> AnnaXP {
    get_xp_internal()
}

/// Award XP to Anna
pub fn award_xp(amount: u64, reason: &str) {
    let mut xp = get_xp_internal();
    let old_level = xp.level;
    xp.add_xp(amount);
    tracing::info!("Anna earned {} XP: {} (total: {}, level: {})", amount, reason, xp.total_xp, xp.level);
    if xp.level > old_level {
        tracing::info!("Anna leveled up! Now level {} - {}", xp.level, get_title_for_level(xp.level));
    }
    save_xp_internal(&xp);
}

/// Record a resolved ticket
pub fn record_ticket_resolved(by_anna: bool, by_senior: bool, resolution_time_secs: u64, department: &str) {
    let mut xp = get_xp_internal();

    xp.tickets_resolved += 1;

    if by_anna {
        xp.resolved_by_anna += 1;
        xp.add_xp(rewards::RECIPE_USED);
    } else if by_senior {
        xp.resolved_by_specialists += 1;
        xp.add_xp(rewards::SENIOR_RESOLVED);
    } else {
        xp.resolved_by_specialists += 1;
        xp.add_xp(rewards::JUNIOR_RESOLVED);
    }

    // Fast resolution bonus
    if resolution_time_secs < 10 {
        xp.add_xp(rewards::FAST_RESOLUTION);
    }

    // Track resolution times
    if resolution_time_secs > xp.longest_resolution {
        xp.longest_resolution = resolution_time_secs;
    }
    if resolution_time_secs < xp.shortest_resolution {
        xp.shortest_resolution = resolution_time_secs;
    }

    // Track department usage
    *xp.questions_by_dept.entry(department.to_string()).or_insert(0) += 1;

    save_xp_internal(&xp);
}

/// Record a new recipe learned
pub fn record_recipe_learned() {
    let mut xp = get_xp_internal();
    xp.recipes_learned += 1;
    xp.add_xp(rewards::RECIPE_LEARNED);
    save_xp_internal(&xp);
}

/// Get time since installation
pub fn time_since_install() -> Option<chrono::Duration> {
    let xp = get_xp_internal();
    xp.installed_at.map(|t| chrono::Utc::now() - t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xp_levels() {
        assert_eq!(AnnaXP::xp_for_level(1), 150); // 100 * 1 + 50
        assert!(AnnaXP::xp_for_level(10) > AnnaXP::xp_for_level(5));
        assert!(AnnaXP::xp_for_level(50) > AnnaXP::xp_for_level(10));
    }

    #[test]
    fn test_titles() {
        assert_eq!(get_title_for_level(1), "Helpdesk Newbie");
        assert_eq!(get_title_for_level(50), "System Expert");
        assert_eq!(get_title_for_level(100), "The One Who Knows All");
    }
}
