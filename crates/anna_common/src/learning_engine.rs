//! Learning Engine v4.4.0 - Functional Learning for Anna
//!
//! This module implements REAL learning:
//! - Question classification into semantic classes
//! - Paraphrase recognition (same class = same fast path)
//! - Pattern caching (probes used, evidence, reliability)
//! - Debug logging for learning events
//!
//! ## What "Learning" Means
//!
//! 1. Fast-path caching: Same question answered instantly without LLM
//! 2. Probe specialization: Know which probes matter, skip useless ones
//! 3. Paraphrase recognition: Variants of same question_class are fast path
//! 4. Full history sync: Brain, LLM, stats all reflect same truth
//!
//! ## Question Classes
//!
//! ```text
//! "what CPU do I have?" -> cpu.info.model
//! "tell me my CPU model" -> cpu.info.model
//! "how many cores?" -> cpu.info.cores
//! "how much RAM?" -> ram.info.total
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info};

// ============================================================================
// Configuration
// ============================================================================

/// Pattern store location
pub const PATTERN_STORE_PATH: &str = "/var/lib/anna/learning/patterns.json";

/// Minimum reliability to learn a pattern
pub const MIN_LEARN_RELIABILITY: f64 = 0.85;

/// Pattern cache TTL in seconds (5 minutes)
pub const PATTERN_CACHE_TTL_SECS: u64 = 300;

/// Maximum patterns per class
pub const MAX_PATTERNS_PER_CLASS: usize = 10;

// ============================================================================
// Question Classification
// ============================================================================

/// Semantic question class - groups related questions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QuestionClass {
    /// Domain (cpu, ram, disk, network, health, etc.)
    pub domain: String,
    /// Topic within domain (info, usage, status, etc.)
    pub topic: String,
    /// Specific aspect (model, cores, total, free, etc.)
    pub aspect: String,
}

impl QuestionClass {
    pub fn new(domain: &str, topic: &str, aspect: &str) -> Self {
        Self {
            domain: domain.to_lowercase(),
            topic: topic.to_lowercase(),
            aspect: aspect.to_lowercase(),
        }
    }

    /// Format as canonical string (e.g., "cpu.info.model")
    pub fn canonical(&self) -> String {
        format!("{}.{}.{}", self.domain, self.topic, self.aspect)
    }

    /// Parse from canonical string
    pub fn from_canonical(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() >= 3 {
            Some(Self::new(parts[0], parts[1], parts[2]))
        } else {
            None
        }
    }
}

/// Classify a question into a semantic class
pub fn classify_question(question: &str) -> QuestionClass {
    let q = question.to_lowercase();

    // =====================
    // CPU Questions
    // =====================
    if is_cpu_question(&q) {
        if is_cores_question(&q) {
            return QuestionClass::new("cpu", "info", "cores");
        }
        if is_threads_question(&q) {
            return QuestionClass::new("cpu", "info", "threads");
        }
        // Default: asking about CPU model
        return QuestionClass::new("cpu", "info", "model");
    }

    // =====================
    // RAM Questions
    // =====================
    if is_ram_question(&q) {
        if q.contains("free") || q.contains("available") {
            return QuestionClass::new("ram", "info", "available");
        }
        return QuestionClass::new("ram", "info", "total");
    }

    // =====================
    // Disk Questions
    // =====================
    if is_disk_question(&q) {
        if q.contains("free") || q.contains("available") || q.contains("left") {
            return QuestionClass::new("disk", "info", "free");
        }
        if q.contains("used") || q.contains("usage") {
            return QuestionClass::new("disk", "info", "used");
        }
        return QuestionClass::new("disk", "info", "total");
    }

    // =====================
    // OS/Kernel Questions
    // =====================
    if is_os_question(&q) {
        if q.contains("kernel") {
            return QuestionClass::new("system", "info", "kernel");
        }
        if q.contains("distro") || q.contains("distribution") {
            return QuestionClass::new("system", "info", "distro");
        }
        return QuestionClass::new("system", "info", "os");
    }

    // =====================
    // Health Questions
    // =====================
    if is_health_question(&q) {
        return QuestionClass::new("anna", "health", "status");
    }

    // =====================
    // GPU Questions
    // =====================
    if is_gpu_question(&q) {
        return QuestionClass::new("gpu", "info", "model");
    }

    // =====================
    // Network Questions
    // =====================
    if is_network_question(&q) {
        if q.contains("ip") {
            return QuestionClass::new("network", "info", "ip");
        }
        return QuestionClass::new("network", "info", "interfaces");
    }

    // =====================
    // Uptime Questions
    // =====================
    if is_uptime_question(&q) {
        return QuestionClass::new("system", "info", "uptime");
    }

    // Default: unknown class
    QuestionClass::new("general", "query", "unknown")
}

// Helper functions for classification
fn is_cpu_question(q: &str) -> bool {
    q.contains("cpu") || q.contains("processor") || q.contains("chip")
}

fn is_cores_question(q: &str) -> bool {
    q.contains("core") && (q.contains("how many") || q.contains("number") || q.contains("count"))
}

fn is_threads_question(q: &str) -> bool {
    q.contains("thread") && (q.contains("how many") || q.contains("number") || q.contains("count"))
}

fn is_ram_question(q: &str) -> bool {
    (q.contains("ram") || q.contains("memory"))
        && !q.contains("remember")
        && (q.contains("how much") || q.contains("total") || q.contains("have") ||
            q.contains("my") || q.contains("free") || q.contains("available") ||
            q == "ram" || q == "ram?" || q == "memory" || q == "memory?")
}

fn is_disk_question(q: &str) -> bool {
    q.contains("disk") || q.contains("storage") || q.contains("space") ||
    q.contains("filesystem") || q == "df"
}

fn is_os_question(q: &str) -> bool {
    (q.contains("os") || q.contains("operating system") || q.contains("distro") ||
     q.contains("distribution") || q.contains("kernel") || q.contains("linux"))
        && (q.contains("what") || q.contains("which") || q.contains("version") ||
            q.contains("running") || q == "os?" || q == "kernel?" || q == "distro?")
}

fn is_health_question(q: &str) -> bool {
    (q.contains("health") || q.contains("status") || q.contains("diagnose"))
        && (q.contains("your") || q.contains("anna") || q.contains("yourself"))
}

fn is_gpu_question(q: &str) -> bool {
    q.contains("gpu") || q.contains("graphics card") || q.contains("video card")
}

fn is_network_question(q: &str) -> bool {
    q.contains("network") || q.contains("interface") ||
    (q.contains("ip") && (q.contains("address") || q.contains("my")))
}

fn is_uptime_question(q: &str) -> bool {
    q.contains("uptime") || (q.contains("how long") && (q.contains("running") || q.contains("up")))
}

// ============================================================================
// Learned Pattern
// ============================================================================

/// A learned pattern - how we successfully answered a question class
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedPattern {
    /// Question class this pattern applies to
    pub class: String,

    /// Original question that created this pattern
    pub original_question: String,

    /// Example paraphrases that matched this pattern
    #[serde(default)]
    pub paraphrases: Vec<String>,

    /// Probes that were used to answer
    pub probes_used: Vec<String>,

    /// Answer origin (Brain, Recipe, Junior, Senior)
    pub origin: String,

    /// Cached answer text (for instant replay)
    pub cached_answer: String,

    /// Reliability score when learned
    pub reliability: f64,

    /// Latency when learned (ms)
    pub latency_ms: u64,

    /// Times this pattern was used
    pub hit_count: u32,

    /// Creation timestamp (unix secs)
    pub created_at: u64,

    /// Last used timestamp (unix secs)
    pub last_used: u64,
}

impl LearnedPattern {
    /// Create a new learned pattern
    pub fn new(
        class: &QuestionClass,
        question: &str,
        probes_used: Vec<String>,
        origin: &str,
        answer: &str,
        reliability: f64,
        latency_ms: u64,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            class: class.canonical(),
            original_question: question.to_string(),
            paraphrases: vec![],
            probes_used,
            origin: origin.to_string(),
            cached_answer: answer.to_string(),
            reliability,
            latency_ms,
            hit_count: 0,
            created_at: now,
            last_used: now,
        }
    }

    /// Check if pattern is still fresh (within TTL)
    pub fn is_fresh(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now - self.last_used < PATTERN_CACHE_TTL_SECS
    }

    /// Record a cache hit
    pub fn record_hit(&mut self, question: &str) {
        self.hit_count += 1;
        self.last_used = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Track paraphrases (up to 5)
        let q_lower = question.to_lowercase();
        if q_lower != self.original_question.to_lowercase()
            && !self.paraphrases.iter().any(|p| p.to_lowercase() == q_lower)
            && self.paraphrases.len() < 5
        {
            self.paraphrases.push(question.to_string());
        }
    }

    /// Get age in seconds
    pub fn age_secs(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now.saturating_sub(self.created_at)
    }
}

// ============================================================================
// Pattern Store
// ============================================================================

/// Persistent store for learned patterns
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PatternStore {
    /// Patterns indexed by question class
    #[serde(default)]
    pub patterns: HashMap<String, LearnedPattern>,

    /// Total pattern cache hits
    #[serde(default)]
    pub total_hits: u64,

    /// Total patterns learned
    #[serde(default)]
    pub total_learned: u64,

    /// Questions where learning was triggered
    #[serde(default)]
    pub learning_events: u64,
}

impl PatternStore {
    /// Load from disk or create new
    pub fn load() -> Self {
        if let Ok(content) = fs::read_to_string(PATTERN_STORE_PATH) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Save to disk
    pub fn save(&self) -> std::io::Result<()> {
        if let Some(parent) = Path::new(PATTERN_STORE_PATH).parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(PATTERN_STORE_PATH, json)
    }

    /// Clear all patterns (for reset)
    pub fn clear(&mut self) {
        self.patterns.clear();
        self.total_hits = 0;
        self.total_learned = 0;
        self.learning_events = 0;
        let _ = self.save();
    }

    /// Verify patterns are cleared (for reset verification)
    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    /// Try to get a cached pattern for a question
    /// Returns (pattern, is_fresh) - stale patterns can be used but should be refreshed
    pub fn get(&mut self, question: &str) -> Option<(LearnedPattern, bool)> {
        let class = classify_question(question);
        let key = class.canonical();

        if let Some(pattern) = self.patterns.get_mut(&key) {
            let fresh = pattern.is_fresh();
            pattern.record_hit(question);
            self.total_hits += 1;

            // Log the cache hit
            debug!(
                "LEARNING: CACHE HIT class={} hits={} fresh={} ({}ms original)",
                key, pattern.hit_count, fresh, pattern.latency_ms
            );

            // Clone before saving to avoid borrow issues
            let result = (pattern.clone(), fresh);
            let _ = self.save();
            Some(result)
        } else {
            None
        }
    }

    /// Learn a pattern from a successful answer
    pub fn learn(
        &mut self,
        question: &str,
        probes_used: Vec<String>,
        origin: &str,
        answer: &str,
        reliability: f64,
        latency_ms: u64,
    ) -> bool {
        // Only learn from high-reliability answers
        if reliability < MIN_LEARN_RELIABILITY {
            debug!(
                "LEARNING: SKIP (reliability {:.2} < {:.2})",
                reliability, MIN_LEARN_RELIABILITY
            );
            return false;
        }

        let class = classify_question(question);
        let key = class.canonical();

        // Check if we already have a better pattern
        if let Some(existing) = self.patterns.get(&key) {
            if existing.reliability >= reliability && existing.is_fresh() {
                debug!(
                    "LEARNING: SKIP (existing pattern {} is better/fresher)",
                    key
                );
                return false;
            }
        }

        // Create and store new pattern
        let pattern = LearnedPattern::new(
            &class,
            question,
            probes_used.clone(),
            origin,
            answer,
            reliability,
            latency_ms,
        );

        info!(
            "LEARNING: NEW PATTERN class={} origin={} reliability={:.2} probes={:?}",
            key, origin, reliability, probes_used
        );

        self.patterns.insert(key.clone(), pattern);
        self.total_learned += 1;
        self.learning_events += 1;
        let _ = self.save();

        true
    }

    /// Get statistics for display
    pub fn stats(&self) -> PatternStats {
        let mut classes_by_domain: HashMap<String, usize> = HashMap::new();
        let mut total_hit_count: u32 = 0;
        let mut fresh_count: usize = 0;

        for pattern in self.patterns.values() {
            let domain = pattern.class.split('.').next().unwrap_or("unknown");
            *classes_by_domain.entry(domain.to_string()).or_insert(0) += 1;
            total_hit_count += pattern.hit_count;
            if pattern.is_fresh() {
                fresh_count += 1;
            }
        }

        PatternStats {
            total_patterns: self.patterns.len(),
            fresh_patterns: fresh_count,
            total_hits: self.total_hits,
            total_hit_count,
            total_learned: self.total_learned,
            learning_events: self.learning_events,
            classes_by_domain,
        }
    }

    /// Format debug output showing learning evolution
    pub fn format_evolution(&self) -> String {
        let mut output = String::new();
        output.push_str("=== LEARNING ENGINE STATUS ===\n\n");

        let stats = self.stats();
        output.push_str(&format!("Total patterns: {}\n", stats.total_patterns));
        output.push_str(&format!("Fresh patterns: {}\n", stats.fresh_patterns));
        output.push_str(&format!("Cache hits: {}\n", stats.total_hits));
        output.push_str(&format!("Learning events: {}\n\n", stats.learning_events));

        output.push_str("Patterns by domain:\n");
        for (domain, count) in &stats.classes_by_domain {
            output.push_str(&format!("  {}: {}\n", domain, count));
        }

        if !self.patterns.is_empty() {
            output.push_str("\nMost used patterns:\n");
            let mut sorted: Vec<_> = self.patterns.values().collect();
            sorted.sort_by(|a, b| b.hit_count.cmp(&a.hit_count));
            for (i, pattern) in sorted.iter().take(5).enumerate() {
                output.push_str(&format!(
                    "  {}. {} ({} hits, {:.0}% reliability)\n",
                    i + 1,
                    pattern.class,
                    pattern.hit_count,
                    pattern.reliability * 100.0
                ));
            }
        }

        output
    }
}

/// Pattern statistics for display
#[derive(Debug, Clone)]
pub struct PatternStats {
    pub total_patterns: usize,
    pub fresh_patterns: usize,
    pub total_hits: u64,
    pub total_hit_count: u32,
    pub total_learned: u64,
    pub learning_events: u64,
    pub classes_by_domain: HashMap<String, usize>,
}

impl PatternStats {
    /// Format for status display
    pub fn format_status(&self) -> String {
        format!(
            "LEARNING ENGINE\n──────────────────────────────────────────\n\
             Patterns: {} ({} fresh)\n\
             Cache hits: {}\n\
             Learning events: {}",
            self.total_patterns, self.fresh_patterns, self.total_hits, self.learning_events
        )
    }
}

// ============================================================================
// Learning Event Log (Debug)
// ============================================================================

/// Log entry for learning event (for debug output)
#[derive(Debug, Clone)]
pub struct LearningLogEntry {
    pub event_type: LearningLogType,
    pub class: String,
    pub message: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LearningLogType {
    /// New pattern learned
    PatternLearned,
    /// Cache hit on existing pattern
    CacheHit,
    /// Cache miss, will need LLM
    CacheMiss,
    /// Pattern refreshed (updated with new answer)
    PatternRefreshed,
    /// Paraphrase recognized for existing class
    ParaphraseRecognized,
    /// Probes reduced based on learning
    ProbesReduced,
}

impl LearningLogEntry {
    pub fn new(event_type: LearningLogType, class: &str, message: &str) -> Self {
        Self {
            event_type,
            class: class.to_string(),
            message: message.to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Format as debug log line
    pub fn format_log(&self) -> String {
        let prefix = match self.event_type {
            LearningLogType::PatternLearned => "LEARNING: NEW PATTERN",
            LearningLogType::CacheHit => "LEARNING: CACHE HIT",
            LearningLogType::CacheMiss => "LEARNING: CACHE MISS",
            LearningLogType::PatternRefreshed => "LEARNING: PATTERN REFRESHED",
            LearningLogType::ParaphraseRecognized => "LEARNING: PARAPHRASE RECOGNIZED",
            LearningLogType::ProbesReduced => "LEARNING: PROBES REDUCED",
        };
        format!("{} class={} {}", prefix, self.class, self.message)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_cpu_model() {
        assert_eq!(
            classify_question("what cpu do I have?").canonical(),
            "cpu.info.model"
        );
        assert_eq!(
            classify_question("tell me my CPU model").canonical(),
            "cpu.info.model"
        );
        assert_eq!(
            classify_question("which processor?").canonical(),
            "cpu.info.model"
        );
        assert_eq!(
            classify_question("what chip is this?").canonical(),
            "cpu.info.model"
        );
    }

    #[test]
    fn test_classify_cpu_cores() {
        assert_eq!(
            classify_question("how many cores does my processor have?").canonical(),
            "cpu.info.cores"
        );
        assert_eq!(
            classify_question("number of cpu cores?").canonical(),
            "cpu.info.cores"
        );
    }

    #[test]
    fn test_classify_ram() {
        assert_eq!(
            classify_question("how much ram do I have?").canonical(),
            "ram.info.total"
        );
        assert_eq!(
            classify_question("total memory?").canonical(),
            "ram.info.total"
        );
        assert_eq!(
            classify_question("free memory available?").canonical(),
            "ram.info.available"
        );
    }

    #[test]
    fn test_classify_disk() {
        assert_eq!(
            classify_question("how much disk space?").canonical(),
            "disk.info.total"
        );
        assert_eq!(
            classify_question("free space on disk?").canonical(),
            "disk.info.free"
        );
    }

    #[test]
    fn test_paraphrase_same_class() {
        // All these should be the same class
        let q1 = "what CPU do I have?";
        let q2 = "tell me my CPU model";
        let q3 = "which processor is installed?";

        let c1 = classify_question(q1);
        let c2 = classify_question(q2);
        let c3 = classify_question(q3);

        assert_eq!(c1.canonical(), c2.canonical());
        assert_eq!(c2.canonical(), c3.canonical());
    }

    #[test]
    fn test_pattern_learning() {
        let mut store = PatternStore::default();

        // Learn a pattern
        let learned = store.learn(
            "what CPU do I have?",
            vec!["lscpu".to_string()],
            "Brain",
            "Your CPU is AMD Ryzen 9 5900X",
            0.99,
            15,
        );
        assert!(learned);

        // Should be able to retrieve it
        let result = store.get("what CPU do I have?");
        assert!(result.is_some());
        let (pattern, fresh) = result.unwrap();
        assert!(fresh);
        assert_eq!(pattern.origin, "Brain");
        assert_eq!(pattern.hit_count, 1);
    }

    #[test]
    fn test_paraphrase_cache_hit() {
        let mut store = PatternStore::default();

        // Learn from original question
        store.learn(
            "what CPU do I have?",
            vec!["lscpu".to_string()],
            "Brain",
            "Your CPU is AMD Ryzen 9 5900X",
            0.99,
            15,
        );

        // Paraphrase should hit the same pattern
        let result = store.get("tell me my CPU model");
        assert!(result.is_some());
        let (pattern, _) = result.unwrap();
        assert_eq!(pattern.class, "cpu.info.model");
    }

    #[test]
    fn test_low_reliability_not_learned() {
        let mut store = PatternStore::default();

        // Low reliability answer should not be learned
        let learned = store.learn(
            "what CPU?",
            vec!["lscpu".to_string()],
            "Junior",
            "Maybe AMD?",
            0.5,
            5000,
        );
        assert!(!learned);
        assert!(store.patterns.is_empty());
    }

    #[test]
    fn test_pattern_stats() {
        let mut store = PatternStore::default();

        store.learn("what cpu?", vec![], "Brain", "AMD", 0.99, 10);
        store.learn("how much ram?", vec![], "Brain", "32GB", 0.99, 10);

        let stats = store.stats();
        assert_eq!(stats.total_patterns, 2);
        assert_eq!(stats.total_learned, 2);
        assert!(stats.classes_by_domain.contains_key("cpu"));
        assert!(stats.classes_by_domain.contains_key("ram"));
    }

    #[test]
    fn test_clear_patterns() {
        let mut store = PatternStore::default();
        store.learn("what cpu?", vec![], "Brain", "AMD", 0.99, 10);
        assert!(!store.is_empty());

        store.clear();
        assert!(store.is_empty());
        assert_eq!(store.total_hits, 0);
    }
}
