//! Statistics Engine v0.42.0
//!
//! Tracks Anna's performance metrics for RPG progression and self-improvement.
//!
//! ## Statistics Tracked
//!
//! - Global: total questions, success rate, avg reliability, latency, iterations
//! - Per-question patterns: improvement tracking for repeated questions
//! - Skill usage: integrated with SkillStore
//!
//! v0.42.0: Added strike_count and difficulty_score for pain-driven learning.

use super::levels::AnnaProgression;
use super::xp::{XpCalculator, XpGain, XpInput};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Default stats directory
pub const STATS_DIR: &str = "/var/lib/anna/knowledge/stats";

/// Global statistics for Anna
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalStats {
    /// Total questions answered
    pub total_questions: u64,
    /// Total successful answers (reliability >= 0.70)
    pub total_successful: u64,
    /// Running average reliability (0.0 - 1.0)
    pub avg_reliability: f64,
    /// Running average latency in milliseconds
    pub avg_latency_ms: f64,
    /// Running average iterations per answer
    pub avg_iterations: f64,
    /// Timestamp of last answered question
    pub last_question_time: Option<DateTime<Utc>>,
    /// Number of distinct question patterns seen
    pub distinct_patterns: u64,
    /// Number of patterns that improved over time
    pub patterns_improved: u64,
}

impl Default for GlobalStats {
    fn default() -> Self {
        Self {
            total_questions: 0,
            total_successful: 0,
            avg_reliability: 0.0,
            avg_latency_ms: 0.0,
            avg_iterations: 0.0,
            last_question_time: None,
            distinct_patterns: 0,
            patterns_improved: 0,
        }
    }
}

impl GlobalStats {
    /// Success rate as percentage (0-100)
    pub fn success_rate(&self) -> f64 {
        if self.total_questions == 0 {
            return 0.0;
        }
        (self.total_successful as f64 / self.total_questions as f64) * 100.0
    }

    /// Update running averages with new answer
    pub fn record_answer(&mut self, reliability: f64, latency_ms: u64, iterations: u32) {
        let n = self.total_questions as f64;
        let new_n = n + 1.0;

        // Incremental average update
        self.avg_reliability = (self.avg_reliability * n + reliability) / new_n;
        self.avg_latency_ms = (self.avg_latency_ms * n + latency_ms as f64) / new_n;
        self.avg_iterations = (self.avg_iterations * n + iterations as f64) / new_n;

        self.total_questions += 1;
        if reliability >= 0.70 {
            self.total_successful += 1;
        }
        self.last_question_time = Some(Utc::now());
    }
}

/// Stats for a specific question pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternStats {
    /// Hash of normalized question pattern
    pub pattern_hash: String,
    /// Number of times this pattern was seen
    pub times_seen: u32,
    /// Most recent reliability score
    pub last_reliability: f64,
    /// Best reliability achieved
    pub best_reliability: f64,
    /// Most recent latency in ms
    pub last_latency_ms: u64,
    /// Best (lowest) latency achieved
    pub best_latency_ms: u64,
    /// First time seen
    pub first_seen: DateTime<Utc>,
    /// Last time seen
    pub last_seen: DateTime<Utc>,
    /// Whether this pattern has improved over time
    pub has_improved: bool,
    /// v0.42.0: Strike count (consecutive low reliability answers)
    /// Increases when R < 0.70, resets on R >= 0.70
    #[serde(default)]
    pub strike_count: u32,
    /// v0.42.0: Difficulty score [0.0, 1.0]
    /// Higher = harder question, based on failure history
    #[serde(default)]
    pub difficulty_score: f64,
}

impl PatternStats {
    /// Strike threshold - below this reliability, add a strike
    pub const STRIKE_THRESHOLD: f64 = 0.70;
    /// Max difficulty score
    pub const MAX_DIFFICULTY: f64 = 1.0;
    /// Difficulty increase per strike
    pub const DIFFICULTY_PER_STRIKE: f64 = 0.1;

    /// Create new pattern stats
    pub fn new(pattern_hash: &str, reliability: f64, latency_ms: u64) -> Self {
        let now = Utc::now();
        let initial_strike = if reliability < Self::STRIKE_THRESHOLD { 1 } else { 0 };
        let initial_difficulty = if reliability < Self::STRIKE_THRESHOLD {
            Self::DIFFICULTY_PER_STRIKE
        } else {
            0.0
        };
        Self {
            pattern_hash: pattern_hash.to_string(),
            times_seen: 1,
            last_reliability: reliability,
            best_reliability: reliability,
            last_latency_ms: latency_ms,
            best_latency_ms: latency_ms,
            first_seen: now,
            last_seen: now,
            has_improved: false,
            strike_count: initial_strike,
            difficulty_score: initial_difficulty,
        }
    }

    /// Update with new occurrence
    pub fn record(&mut self, reliability: f64, latency_ms: u64) {
        self.times_seen += 1;

        // Track improvement
        if reliability > self.best_reliability || latency_ms < self.best_latency_ms {
            self.has_improved = true;
        }

        // Update bests
        if reliability > self.best_reliability {
            self.best_reliability = reliability;
        }
        if latency_ms < self.best_latency_ms {
            self.best_latency_ms = latency_ms;
        }

        // v0.42.0: Update strike count and difficulty
        if reliability < Self::STRIKE_THRESHOLD {
            self.strike_count += 1;
            self.difficulty_score = (self.difficulty_score + Self::DIFFICULTY_PER_STRIKE)
                .min(Self::MAX_DIFFICULTY);
        } else {
            // Reset strike count on successful answer
            self.strike_count = 0;
            // Slowly reduce difficulty on success
            self.difficulty_score = (self.difficulty_score - 0.05).max(0.0);
        }

        // Update currents
        self.last_reliability = reliability;
        self.last_latency_ms = latency_ms;
        self.last_seen = Utc::now();
    }

    /// Check if pattern is "difficult" (difficulty_score >= 0.5)
    pub fn is_difficult(&self) -> bool {
        self.difficulty_score >= 0.5
    }

    /// Check if pattern needs remediation (3+ strikes)
    pub fn needs_remediation(&self) -> bool {
        self.strike_count >= 3
    }
}

/// Normalized question pattern for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionPattern {
    /// Original question text (for display)
    pub original: String,
    /// Normalized form (for hashing)
    pub normalized: String,
    /// Hash of normalized form
    pub hash: String,
}

impl QuestionPattern {
    /// Create from question text
    pub fn from_question(question: &str) -> Self {
        let normalized = Self::normalize(question);
        let hash = Self::hash(&normalized);
        Self {
            original: question.to_string(),
            normalized,
            hash,
        }
    }

    /// Normalize question text for pattern matching
    fn normalize(question: &str) -> String {
        question
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Generate hash of normalized question
    fn hash(normalized: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        normalized.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }
}

/// Performance snapshot for status display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    /// Anna's progression (level, XP, title)
    pub progression: AnnaProgression,
    /// Global stats
    pub global: GlobalStats,
    /// Top patterns by usage
    pub top_patterns: Vec<PatternStats>,
    /// Patterns that improved
    pub improved_count: u64,
    /// Snapshot timestamp
    pub timestamp: DateTime<Utc>,
}

impl PerformanceSnapshot {
    /// Create snapshot from engine
    pub fn from_engine(engine: &StatsEngine) -> Self {
        let mut patterns: Vec<_> = engine.patterns.values().cloned().collect();
        patterns.sort_by(|a, b| b.times_seen.cmp(&a.times_seen));
        let top_patterns = patterns.into_iter().take(5).collect();

        Self {
            progression: engine.progression.clone(),
            global: engine.global.clone(),
            top_patterns,
            improved_count: engine.global.patterns_improved,
            timestamp: Utc::now(),
        }
    }
}

/// Statistics engine - tracks all performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsEngine {
    /// Anna's progression (level, XP, title)
    pub progression: AnnaProgression,
    /// Global statistics
    pub global: GlobalStats,
    /// Per-pattern statistics
    pub patterns: HashMap<String, PatternStats>,
    /// XP calculator
    #[serde(skip)]
    xp_calculator: Option<XpCalculator>,
}

impl Default for StatsEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl StatsEngine {
    /// Create new stats engine
    pub fn new() -> Self {
        Self {
            progression: AnnaProgression::new(),
            global: GlobalStats::default(),
            patterns: HashMap::new(),
            xp_calculator: Some(XpCalculator::new()),
        }
    }

    /// Get XP calculator
    fn calculator(&self) -> XpCalculator {
        self.xp_calculator.clone().unwrap_or_default()
    }

    /// Load from default location
    pub fn load_default() -> Result<Self> {
        Self::load(&Self::default_path())
    }

    /// Load from file
    pub fn load(path: &PathBuf) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(path)?;
        let mut engine: StatsEngine = serde_json::from_str(&content)?;
        engine.xp_calculator = Some(XpCalculator::new());
        Ok(engine)
    }

    /// Save to default location
    pub fn save_default(&self) -> Result<()> {
        self.save(&Self::default_path())
    }

    /// Save to file
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Default path for stats file
    pub fn default_path() -> PathBuf {
        PathBuf::from(STATS_DIR).join("anna_stats.json")
    }

    /// Record an answered question
    pub fn record_answer(
        &mut self,
        question: &str,
        reliability: f64,
        latency_ms: u64,
        iterations: u32,
        skills_used: u32,
        was_decomposed: bool,
        answer_success: bool,
    ) -> XpGain {
        // Update global stats
        self.global.record_answer(reliability, latency_ms, iterations);

        // Update pattern stats
        let pattern = QuestionPattern::from_question(question);
        let pattern_improved = if let Some(stats) = self.patterns.get_mut(&pattern.hash) {
            let was_improved = stats.has_improved;
            stats.record(reliability, latency_ms);
            !was_improved && stats.has_improved
        } else {
            let stats = PatternStats::new(&pattern.hash, reliability, latency_ms);
            self.patterns.insert(pattern.hash.clone(), stats);
            self.global.distinct_patterns += 1;
            false
        };

        if pattern_improved {
            self.global.patterns_improved += 1;
        }

        // Calculate and award XP
        let input = XpInput::new(
            reliability,
            skills_used,
            iterations,
            was_decomposed,
            answer_success,
        );
        let xp_gain = self.calculator().calculate(&input);

        if xp_gain.total > 0 {
            self.progression.add_xp(xp_gain.total);
        }

        xp_gain
    }

    /// Get current performance snapshot
    pub fn snapshot(&self) -> PerformanceSnapshot {
        PerformanceSnapshot::from_engine(self)
    }

    /// Get level
    pub fn level(&self) -> u8 {
        self.progression.level.value()
    }

    /// Get title
    pub fn title(&self) -> String {
        self.progression.title.to_string()
    }

    /// Get total XP
    pub fn total_xp(&self) -> u64 {
        self.progression.total_xp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_stats_default() {
        let stats = GlobalStats::default();
        assert_eq!(stats.total_questions, 0);
        assert_eq!(stats.success_rate(), 0.0);
    }

    #[test]
    fn test_global_stats_record() {
        let mut stats = GlobalStats::default();

        stats.record_answer(0.80, 1000, 2);
        assert_eq!(stats.total_questions, 1);
        assert_eq!(stats.total_successful, 1);
        assert!((stats.avg_reliability - 0.80).abs() < 0.01);

        stats.record_answer(0.60, 2000, 3);
        assert_eq!(stats.total_questions, 2);
        assert_eq!(stats.total_successful, 1); // 0.60 < 0.70
        assert!((stats.avg_reliability - 0.70).abs() < 0.01);
    }

    #[test]
    fn test_success_rate() {
        let mut stats = GlobalStats::default();

        stats.record_answer(0.90, 500, 1);
        stats.record_answer(0.80, 500, 1);
        stats.record_answer(0.50, 500, 1);
        stats.record_answer(0.40, 500, 1);

        // 2 out of 4 successful (>= 0.70)
        assert!((stats.success_rate() - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_question_pattern_normalization() {
        let p1 = QuestionPattern::from_question("What is my CPU?");
        let p2 = QuestionPattern::from_question("what is my cpu?");
        let p3 = QuestionPattern::from_question("WHAT IS MY CPU?");
        let p4 = QuestionPattern::from_question("What   is  my   CPU?");

        // All should have same hash
        assert_eq!(p1.hash, p2.hash);
        assert_eq!(p2.hash, p3.hash);
        assert_eq!(p3.hash, p4.hash);
    }

    #[test]
    fn test_pattern_stats_improvement() {
        let mut stats = PatternStats::new("test", 0.60, 2000);
        assert!(!stats.has_improved);

        // Same or worse - no improvement
        stats.record(0.55, 2500);
        assert!(!stats.has_improved);

        // Better reliability - improved!
        stats.record(0.75, 2000);
        assert!(stats.has_improved);
        assert!((stats.best_reliability - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_pattern_stats_latency_improvement() {
        let mut stats = PatternStats::new("test", 0.70, 2000);
        assert!(!stats.has_improved);

        // Better latency - improved!
        stats.record(0.70, 1500);
        assert!(stats.has_improved);
        assert_eq!(stats.best_latency_ms, 1500);
    }

    #[test]
    fn test_stats_engine_record_answer() {
        let mut engine = StatsEngine::new();

        // First answer - high reliability
        let xp1 = engine.record_answer(
            "What is my CPU?",
            0.90,
            1000,
            1,
            2,
            false,
            true,
        );
        assert!(xp1.total > 0);
        assert_eq!(engine.global.total_questions, 1);
        assert!(engine.total_xp() > 0);

        // Second answer - same question, better
        let xp2 = engine.record_answer(
            "What is my CPU?",
            0.95,
            800,
            1,
            2,
            false,
            true,
        );
        assert!(xp2.total > 0);
        assert_eq!(engine.global.total_questions, 2);
        assert_eq!(engine.global.distinct_patterns, 1); // Same pattern
    }

    #[test]
    fn test_stats_engine_level_progression() {
        let mut engine = StatsEngine::new();

        // Start at level 0
        assert_eq!(engine.level(), 0);
        assert_eq!(engine.title(), "Intern");

        // Simulate many successful answers
        for _ in 0..20 {
            engine.record_answer("test question", 0.95, 500, 1, 1, false, true);
        }

        // Should have gained some levels
        assert!(engine.level() > 0);
        assert!(engine.total_xp() > 0);
    }

    #[test]
    fn test_stats_engine_no_xp_for_low_reliability() {
        let mut engine = StatsEngine::new();

        let xp = engine.record_answer("bad answer", 0.30, 5000, 5, 0, false, true);
        assert_eq!(xp.total, 0);
        assert_eq!(engine.total_xp(), 0);
    }

    #[test]
    fn test_performance_snapshot() {
        let mut engine = StatsEngine::new();

        engine.record_answer("q1", 0.90, 1000, 1, 1, false, true);
        engine.record_answer("q2", 0.85, 1200, 2, 2, false, true);
        engine.record_answer("q1", 0.95, 800, 1, 1, false, true);

        let snapshot = engine.snapshot();
        assert_eq!(snapshot.global.total_questions, 3);
        assert_eq!(snapshot.global.distinct_patterns, 2);
        assert!(snapshot.improved_count >= 1);
    }

    #[test]
    fn test_stats_engine_serialization() {
        let mut engine = StatsEngine::new();
        engine.record_answer("test", 0.90, 1000, 1, 1, false, true);

        let json = serde_json::to_string(&engine).unwrap();
        let loaded: StatsEngine = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.global.total_questions, 1);
        assert_eq!(loaded.progression.total_xp, engine.progression.total_xp);
    }
}
