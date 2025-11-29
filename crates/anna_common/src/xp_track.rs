//! XP Tracking System for v0.95.0
//!
//! Unified XP model for Anna, Junior, and Senior agents.
//! Tracks levels, XP, trust, and streaks for reinforcement learning.
//! v0.95.0: Expanded RPG title bands, reliability-based XP scaling.

use crate::rpg_display::{get_rpg_title, ReliabilityScale, TrustLevel};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// XP tracking directory
pub const XP_DIR: &str = "/var/lib/anna/xp";

// ============================================================================
// Title and Level System (v0.95.0: Expanded)
// ============================================================================

/// Title bands based on level (v0.95.0 expanded)
/// Level 1-4: Intern
/// Level 5-9: Junior Specialist
/// Level 10-19: Specialist
/// Level 20-34: Senior Specialist
/// Level 35-49: Lead
/// Level 50-69: Principal
/// Level 70-89: Archon
/// Level 90-99: Mythic
pub const TITLE_BANDS: &[(u8, u8, &str)] = &[
    (1, 4, "Intern"),
    (5, 9, "Junior Specialist"),
    (10, 19, "Specialist"),
    (20, 34, "Senior Specialist"),
    (35, 49, "Lead"),
    (50, 69, "Principal"),
    (70, 89, "Archon"),
    (90, 99, "Mythic"),
];

/// Get title for a level (uses rpg_display module)
pub fn get_title(level: u8) -> &'static str {
    get_rpg_title(level)
}

/// Calculate XP needed for next level
pub fn xp_for_level(level: u8) -> u64 {
    100 * level as u64
}

// ============================================================================
// XP Track for a Single Agent
// ============================================================================

/// XP tracking for an agent (Anna, Junior, or Senior)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XpTrack {
    /// Agent name
    pub name: String,
    /// Current level (1-99)
    pub level: u8,
    /// Total XP accumulated
    pub xp: u64,
    /// XP needed to reach next level
    pub xp_to_next: u64,
    /// Consecutive good events
    pub streak_good: u32,
    /// Consecutive bad events
    pub streak_bad: u32,
    /// Trust score (0.0-1.0)
    pub trust: f32,
    /// Total good events
    pub total_good: u64,
    /// Total bad events
    pub total_bad: u64,
    /// Last update timestamp
    pub last_update: u64,
}

impl XpTrack {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            level: 1,
            xp: 0,
            xp_to_next: xp_for_level(1),
            streak_good: 0,
            streak_bad: 0,
            trust: 0.5, // Start neutral
            total_good: 0,
            total_bad: 0,
            last_update: Self::now(),
        }
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Get current title
    pub fn title(&self) -> &'static str {
        get_title(self.level)
    }

    /// Get XP progress within current level (0.0-1.0)
    pub fn level_progress(&self) -> f64 {
        let level_start = if self.level > 1 {
            (1..self.level).map(|l| xp_for_level(l)).sum()
        } else {
            0
        };
        let xp_in_level = self.xp.saturating_sub(level_start);
        xp_in_level as f64 / self.xp_to_next as f64
    }

    /// Record a positive event
    pub fn record_good(&mut self, xp_gain: u64, trust_delta: f32) {
        // XP only increases
        self.xp += xp_gain;

        // Trust increases (clamped to 1.0)
        self.trust = (self.trust + trust_delta).min(1.0).max(0.0);

        // Update streaks
        self.streak_good += 1;
        self.streak_bad = self.decay_streak(self.streak_bad);

        self.total_good += 1;
        self.last_update = Self::now();

        // Check for level up
        self.check_level_up();
    }

    /// Record a negative event (XP doesn't decrease, but trust and streaks change)
    pub fn record_bad(&mut self, trust_delta: f32) {
        // Trust decreases (clamped to 0.0)
        self.trust = (self.trust - trust_delta.abs()).max(0.0);

        // Update streaks
        self.streak_bad += 1;
        self.streak_good = self.decay_streak(self.streak_good);

        self.total_bad += 1;
        self.last_update = Self::now();
    }

    /// Decay a streak to prevent unbounded growth
    fn decay_streak(&self, streak: u32) -> u32 {
        if streak > 10 {
            streak / 2
        } else if streak > 5 {
            streak.saturating_sub(2)
        } else if streak > 0 {
            streak.saturating_sub(1)
        } else {
            0
        }
    }

    /// Check if we leveled up
    fn check_level_up(&mut self) {
        while self.level < 99 {
            // Total XP needed to reach next level
            let total_xp_for_next: u64 = (1..=self.level).map(|l| xp_for_level(l)).sum();
            if self.xp >= total_xp_for_next {
                self.level += 1;
                self.xp_to_next = xp_for_level(self.level);
            } else {
                break;
            }
        }
    }

    /// Get trust as percentage string
    pub fn trust_pct(&self) -> String {
        format!("{:.0}%", self.trust * 100.0)
    }

    /// Is trust considered low?
    pub fn is_low_trust(&self) -> bool {
        self.trust < 0.4
    }

    /// Is trust considered high?
    pub fn is_high_trust(&self) -> bool {
        self.trust >= 0.7
    }

    /// Get trust level classification (v0.95.0)
    pub fn trust_level(&self) -> TrustLevel {
        TrustLevel::from_trust(self.trust)
    }

    /// Get trust label (low/normal/high)
    pub fn trust_label(&self) -> &'static str {
        self.trust_level().label()
    }

    /// Record a good event with reliability scaling (v0.95.0)
    /// XP and trust are scaled based on answer reliability
    pub fn record_good_with_reliability(&mut self, base_xp: u64, base_trust: f32, reliability: f64) -> u64 {
        let scale = ReliabilityScale::from_reliability(reliability);
        let actual_xp = scale.scale_xp(base_xp);
        let actual_trust = scale.scale_trust(base_trust);

        self.xp += actual_xp;
        self.trust = (self.trust + actual_trust).min(1.0).max(0.0);
        self.streak_good += 1;
        self.streak_bad = self.decay_streak(self.streak_bad);
        self.total_good += 1;
        self.last_update = Self::now();
        self.check_level_up();

        actual_xp
    }

    /// Format as summary line
    pub fn format_summary(&self) -> String {
        format!(
            "{}: Level {} - {} (trust {})",
            self.name, self.level, self.title(), self.trust_pct()
        )
    }

    /// Format as XP debug line
    pub fn format_xp_line(&self, event: &str, xp_change: i32) -> String {
        let sign = if xp_change >= 0 { "+" } else { "" };
        format!(
            "[XP] {}: {}{} XP ({})   Level {} ({})  trust={:.2}",
            self.name, sign, xp_change, event, self.level, self.title(), self.trust
        )
    }
}

// ============================================================================
// Agent Stats for Display
// ============================================================================

/// Statistics for Junior agent
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JuniorStats {
    pub good_plans: u64,
    pub bad_plans: u64,
    pub timeouts: u64,
    pub needs_fix: u64,
    pub overcomplicated: u64,
}

/// Statistics for Senior agent
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SeniorStats {
    pub approvals: u64,
    pub fix_and_accept: u64,
    pub rubber_stamps_blocked: u64,
    pub refusals: u64,
    pub timeouts: u64,
}

/// Statistics for Anna (global)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnnaStats {
    pub self_solves: u64,
    pub brain_assists: u64,
    pub llm_answers: u64,
    pub refusals: u64,
    pub timeouts: u64,
    pub total_questions: u64,
    pub avg_reliability: f64,
    pub avg_latency_ms: u64,
}

// ============================================================================
// Combined XP Store
// ============================================================================

/// Combined XP store for all agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XpStore {
    pub anna: XpTrack,
    pub junior: XpTrack,
    pub senior: XpTrack,
    pub anna_stats: AnnaStats,
    pub junior_stats: JuniorStats,
    pub senior_stats: SeniorStats,
}

impl Default for XpStore {
    fn default() -> Self {
        Self::new()
    }
}

impl XpStore {
    pub fn new() -> Self {
        Self {
            anna: XpTrack::new("Anna"),
            junior: XpTrack::new("Junior"),
            senior: XpTrack::new("Senior"),
            anna_stats: AnnaStats::default(),
            junior_stats: JuniorStats::default(),
            senior_stats: SeniorStats::default(),
        }
    }

    /// Load from disk
    pub fn load() -> Self {
        let path = PathBuf::from(XP_DIR).join("xp_store.json");
        if let Ok(data) = fs::read_to_string(&path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Self::new()
        }
    }

    /// Save to disk
    pub fn save(&self) -> std::io::Result<()> {
        let path = PathBuf::from(XP_DIR).join("xp_store.json");
        fs::create_dir_all(XP_DIR)?;
        let data = serde_json::to_string_pretty(self)?;
        fs::write(path, data)
    }

    // ========== Anna Events ==========

    /// Anna self-solved without LLM
    pub fn anna_self_solve(&mut self, question: &str) -> String {
        self.anna.record_good(5, 0.02);
        self.anna_stats.self_solves += 1;
        self.anna_stats.total_questions += 1;
        let _ = self.save();
        self.anna.format_xp_line("self_solve_ok", 5)
    }

    /// Anna self-solve failed
    pub fn anna_self_solve_fail(&mut self) -> String {
        self.anna.record_bad(0.03);
        let _ = self.save();
        self.anna.format_xp_line("self_solve_fail", 0)
    }

    /// Brain helped LLM
    pub fn anna_brain_helped(&mut self) -> String {
        self.anna.record_good(3, 0.01);
        self.anna_stats.brain_assists += 1;
        let _ = self.save();
        self.anna.format_xp_line("brain_helped_llm", 3)
    }

    /// Timeout fallback
    pub fn anna_timeout(&mut self) -> String {
        self.anna.record_bad(0.05);
        self.anna_stats.timeouts += 1;
        let _ = self.save();
        self.anna.format_xp_line("timeout_fallback", 0)
    }

    /// Correct refusal
    pub fn anna_refusal_correct(&mut self) -> String {
        self.anna.record_good(2, 0.01);
        self.anna_stats.refusals += 1;
        let _ = self.save();
        self.anna.format_xp_line("refusal_correct", 2)
    }

    /// Wrong refusal (refused when evidence existed)
    pub fn anna_refusal_wrong(&mut self) -> String {
        self.anna.record_bad(0.04);
        let _ = self.save();
        self.anna.format_xp_line("refusal_wrong", 0)
    }

    // ========== Junior Events ==========

    /// Junior plan was good
    pub fn junior_plan_good(&mut self, command: &str) -> String {
        self.junior.record_good(3, 0.02);
        self.junior_stats.good_plans += 1;
        let _ = self.save();
        self.junior.format_xp_line("plan_good", 3)
    }

    /// Junior plan was bad
    pub fn junior_plan_bad(&mut self) -> String {
        self.junior.record_bad(0.05);
        self.junior_stats.bad_plans += 1;
        let _ = self.save();
        self.junior.format_xp_line("plan_bad", 0)
    }

    /// Junior needed fix from Senior
    pub fn junior_needs_fix(&mut self) -> String {
        self.junior.record_good(1, 0.0); // Small XP for trying
        self.junior.record_bad(0.02);     // But trust penalty
        self.junior_stats.needs_fix += 1;
        let _ = self.save();
        self.junior.format_xp_line("needs_fix", 1)
    }

    /// Junior timed out
    pub fn junior_timeout(&mut self) -> String {
        self.junior.record_bad(0.05);
        self.junior_stats.timeouts += 1;
        let _ = self.save();
        self.junior.format_xp_line("timeout", 0)
    }

    /// Junior overcomplicated (too many probes)
    pub fn junior_overcomplicates(&mut self, probes_requested: u32, minimal_needed: u32) -> String {
        self.junior.record_bad(0.03);
        self.junior_stats.overcomplicated += 1;
        let _ = self.save();
        format!(
            "[JUNIOR_COST] probes_requested={} minimal_needed={} penalty=overcomplicated_plan\n{}",
            probes_requested,
            minimal_needed,
            self.junior.format_xp_line("overcomplicates", 0)
        )
    }

    // ========== Senior Events ==========

    /// Senior approved correctly
    pub fn senior_approve_correct(&mut self, score: f64) -> String {
        self.senior.record_good(3, 0.02);
        self.senior_stats.approvals += 1;
        let _ = self.save();
        self.senior.format_xp_line(&format!("approve_correct ({:.0}%)", score * 100.0), 3)
    }

    /// Senior fix and accept was good
    pub fn senior_fix_accept_good(&mut self) -> String {
        self.senior.record_good(4, 0.01);
        self.senior_stats.fix_and_accept += 1;
        let _ = self.save();
        self.senior.format_xp_line("fix_and_accept_good", 4)
    }

    /// Senior fix and accept was wrong
    pub fn senior_fix_accept_wrong(&mut self) -> String {
        self.senior.record_bad(0.08);
        let _ = self.save();
        self.senior.format_xp_line("fix_and_accept_wrong", 0)
    }

    /// Senior timed out
    pub fn senior_timeout(&mut self) -> String {
        self.senior.record_bad(0.05);
        self.senior_stats.timeouts += 1;
        let _ = self.save();
        self.senior.format_xp_line("timeout", 0)
    }

    /// Senior rubber-stamped (blocked)
    pub fn senior_rubber_stamp(&mut self) -> String {
        self.senior.record_bad(0.10);
        self.senior_stats.rubber_stamps_blocked += 1;
        let _ = self.save();
        self.senior.format_xp_line("rubber_stamp_blocked", 0)
    }

    /// Senior refused
    pub fn senior_refusal(&mut self) -> String {
        self.senior_stats.refusals += 1;
        let _ = self.save();
        self.senior.format_xp_line("refusal", 0)
    }

    // ========== Update Helpers ==========

    /// Update Anna stats after a question
    pub fn update_anna_question_stats(&mut self, reliability: f64, latency_ms: u64) {
        self.anna_stats.total_questions += 1;

        // Running average for reliability
        let n = self.anna_stats.total_questions as f64;
        self.anna_stats.avg_reliability =
            (self.anna_stats.avg_reliability * (n - 1.0) + reliability) / n;

        // Running average for latency
        self.anna_stats.avg_latency_ms =
            ((self.anna_stats.avg_latency_ms as f64 * (n - 1.0) + latency_ms as f64) / n) as u64;

        let _ = self.save();
    }

    /// Record LLM answer
    pub fn anna_llm_answer(&mut self) {
        self.anna_stats.llm_answers += 1;
        let _ = self.save();
    }

    // ========== v0.95.0: Reliability-Scaled XP Events ==========

    /// Anna LLM orchestration completed with reliability-scaled XP
    /// Uses ReliabilityScale to adjust XP and trust based on answer quality
    pub fn anna_llm_orchestration_ok(&mut self, reliability: f64) -> String {
        let base_xp = 4u64;
        let base_trust = 0.015f32;

        let actual_xp = self.anna.record_good_with_reliability(base_xp, base_trust, reliability);
        self.anna_stats.llm_answers += 1;
        self.anna_stats.total_questions += 1;
        let _ = self.save();

        format!(
            "[XP] Anna: +{} XP ({} @ {}%)   Level {} ({})  trust={:.2}",
            actual_xp, "llm_ok", (reliability * 100.0).round() as u32,
            self.anna.level, self.anna.title(), self.anna.trust
        )
    }

    /// Anna answered with partial evidence (low reliability but honest)
    pub fn anna_partial_answer(&mut self, reliability: f64) -> String {
        let base_xp = 1u64;
        let base_trust = 0.005f32;

        let actual_xp = self.anna.record_good_with_reliability(base_xp, base_trust, reliability);
        self.anna_stats.total_questions += 1;
        let _ = self.save();

        format!(
            "[XP] Anna: +{} XP ({} @ {}%)   Level {} ({})  trust={:.2}",
            actual_xp, "partial", (reliability * 100.0).round() as u32,
            self.anna.level, self.anna.title(), self.anna.trust
        )
    }

    // ========== Trust-Based Behaviour Hints ==========

    /// Should Anna use Brain more aggressively?
    pub fn should_anna_use_brain(&self) -> bool {
        self.anna.is_high_trust() && self.anna.level >= 5
    }

    /// Should Anna lean on Senior more?
    pub fn should_anna_lean_on_senior(&self) -> bool {
        self.anna.is_low_trust()
    }

    /// Should Junior be simpler?
    pub fn should_junior_be_simple(&self) -> bool {
        self.junior.is_low_trust()
    }

    /// Should Senior be stricter?
    pub fn should_senior_be_strict(&self) -> bool {
        self.senior.is_low_trust()
    }

    // ========== Display Helpers ==========

    /// Format LLM agents section for status
    pub fn format_llm_agents(&self) -> String {
        let mut lines = Vec::new();
        lines.push("LLM AGENTS".to_string());
        lines.push("-".repeat(60));

        // Junior
        lines.push(format!(
            "  * Junior: Level {} - {} (trust {})",
            self.junior.level, self.junior.title(), self.junior.trust_pct()
        ));
        lines.push(format!(
            "      - Good plans: {}   Bad plans: {}   Timeouts: {}",
            self.junior_stats.good_plans, self.junior_stats.bad_plans, self.junior_stats.timeouts
        ));

        // Senior
        lines.push(format!(
            "  * Senior: Level {} - {} (trust {})",
            self.senior.level, self.senior.title(), self.senior.trust_pct()
        ));
        lines.push(format!(
            "      - Approvals: {}    Fix&Accept: {}  Rubber-stamps blocked: {}",
            self.senior_stats.approvals, self.senior_stats.fix_and_accept,
            self.senior_stats.rubber_stamps_blocked
        ));

        lines.push("-".repeat(60));
        lines.join("\n")
    }

    /// Check if any agent has low trust and needs warning
    pub fn low_trust_warning(&self) -> Option<String> {
        let mut warnings = Vec::new();

        if self.anna.trust < 0.3 {
            warnings.push("Anna trust is low; favour Brain path and stricter validation.");
        }
        if self.junior.trust < 0.3 {
            warnings.push("Junior trust is low; favour simpler prompts and stricter Senior review.");
        }
        if self.senior.trust < 0.3 {
            warnings.push("Senior trust is low; consider more conservative approvals.");
        }

        if warnings.is_empty() {
            None
        } else {
            Some(warnings.join("\n"))
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_title_bands() {
        // v0.95.0: Updated title bands
        assert_eq!(get_title(1), "Intern");
        assert_eq!(get_title(4), "Intern");
        assert_eq!(get_title(5), "Junior Specialist");
        assert_eq!(get_title(10), "Specialist");
        assert_eq!(get_title(20), "Senior Specialist");
        assert_eq!(get_title(35), "Lead");
        assert_eq!(get_title(50), "Principal");
        assert_eq!(get_title(70), "Archon");
        assert_eq!(get_title(90), "Mythic");
    }

    #[test]
    fn test_xp_for_level() {
        assert_eq!(xp_for_level(1), 100);
        assert_eq!(xp_for_level(10), 1000);
        assert_eq!(xp_for_level(50), 5000);
    }

    #[test]
    fn test_xp_track_new() {
        let track = XpTrack::new("Anna");
        assert_eq!(track.level, 1);
        assert_eq!(track.xp, 0);
        assert_eq!(track.trust, 0.5);
        assert_eq!(track.title(), "Intern");
    }

    #[test]
    fn test_record_good() {
        let mut track = XpTrack::new("Anna");
        track.record_good(50, 0.1);
        assert_eq!(track.xp, 50);
        assert_eq!(track.trust, 0.6);
        assert_eq!(track.streak_good, 1);
        assert_eq!(track.total_good, 1);
    }

    #[test]
    fn test_record_bad() {
        let mut track = XpTrack::new("Anna");
        track.record_bad(0.2);
        assert_eq!(track.xp, 0); // XP never decreases
        assert_eq!(track.trust, 0.3);
        assert_eq!(track.streak_bad, 1);
        assert_eq!(track.total_bad, 1);
    }

    #[test]
    fn test_level_up() {
        let mut track = XpTrack::new("Anna");
        track.record_good(150, 0.1); // More than 100 XP for level 1
        assert_eq!(track.level, 2);
    }

    #[test]
    fn test_trust_clamp() {
        let mut track = XpTrack::new("Anna");
        track.record_good(10, 0.7);
        assert!(track.trust <= 1.0);

        track.trust = 0.1;
        track.record_bad(0.5);
        assert!(track.trust >= 0.0);
    }

    #[test]
    fn test_xp_store_events() {
        let mut store = XpStore::new();

        let line = store.anna_self_solve("test");
        assert!(line.contains("+5 XP"));
        assert_eq!(store.anna_stats.self_solves, 1);

        let line = store.junior_plan_good("lscpu");
        assert!(line.contains("+3 XP"));
        assert_eq!(store.junior_stats.good_plans, 1);

        let line = store.senior_approve_correct(0.95);
        assert!(line.contains("+3 XP"));
        assert_eq!(store.senior_stats.approvals, 1);
    }

    #[test]
    fn test_trust_behaviour_hints() {
        let mut store = XpStore::new();

        // Low trust
        store.junior.trust = 0.3;
        assert!(store.should_junior_be_simple());

        // High trust
        store.anna.trust = 0.8;
        store.anna.level = 10;
        assert!(store.should_anna_use_brain());
    }

    #[test]
    fn test_format_llm_agents() {
        let store = XpStore::new();
        let output = store.format_llm_agents();
        assert!(output.contains("Junior"));
        assert!(output.contains("Senior"));
        assert!(output.contains("LLM AGENTS"));
    }
}
