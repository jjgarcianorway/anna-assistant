//! Specialist Learning System (v0.0.401)
//!
//! Captures knowledge when Anna receives help from specialists (Senior escalation,
//! LLM self-healing, or user feedback). This knowledge is used to create generic
//! recipes and improve future responses.
//!
//! Learning Threshold (Adaptive):
//! - High-confidence (80+): Learn immediately from first success
//! - Lower confidence: Require 2+ successes before creating generic recipe

use crate::probe_learning::QueryCategory;
use crate::revision::RevisionIssue;
use crate::rpc::SpecialistDomain;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// A lesson learned from specialist interaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialistLesson {
    pub id: String,                          // Unique lesson ID
    pub query_pattern: String,               // Normalized query pattern
    pub domain: SpecialistDomain,            // Domain this applies to
    pub category: QueryCategory,             // For probe learning
    pub issues_fixed: Vec<RevisionIssue>,    // What was wrong
    pub solution_type: SolutionType,         // How solution was obtained
    pub effective_probes: Vec<String>,       // Probes that worked
    pub answer_template: String,             // Successful answer
    pub confidence: u8,                      // Confidence 0-100
    pub success_count: u32,                  // Times this succeeded
    pub learned_at: u64,                     // First learning timestamp
    pub last_success_at: u64,                // Last success timestamp
    pub generic_pattern: Option<GenericPattern>, // Generic pattern if any
}

/// How the solution was obtained
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SolutionType {
    /// Senior staff provided guidance
    SeniorGuidance {
        /// The revision instruction that worked
        instruction_summary: String,
    },
    /// LLM self-healing corrected the answer
    LlmSelfHealing {
        /// What constraint/correction was applied
        correction_type: String,
    },
    /// User confirmed the answer was helpful
    UserFeedback {
        /// Whether user said helpful
        helpful: bool,
    },
}

/// A generic pattern extracted from a lesson
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericPattern {
    pub category: PatternCategory,
    pub variables: Vec<PatternVariable>,
    pub probe_templates: Vec<String>,
    pub answer_template: String,
}

/// Categories of generic patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PatternCategory {
    ConfigCheck,    // "check X config"
    ConfigEdit,     // "enable Y in X"
    ServiceAction,  // "start/stop/restart X"
    PackageQuery,   // "is X installed"
    DiskAnalysis,   // "what's using space"
    ProcessQuery,   // "what's using CPU/memory"
    Other,
}

/// A variable in a generic pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternVariable {
    pub name: String,
    pub detection_hint: String,
    pub example_values: Vec<String>,
}

/// Store for specialist lessons
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SpecialistLearningStore {
    /// All lessons indexed by ID
    pub lessons: HashMap<String, SpecialistLesson>,
    /// Index from query keywords to lesson IDs
    pub keyword_index: HashMap<String, Vec<String>>,
    /// Index from pattern category to lesson IDs
    pub category_index: HashMap<PatternCategory, Vec<String>>,
    /// Pending patterns waiting for more successes (confidence < 80)
    pub pending_patterns: HashMap<String, PendingPattern>,
}

/// A pattern waiting for more successes before becoming a lesson
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingPattern {
    pub query_pattern: String,
    pub domain: SpecialistDomain,
    pub success_count: u32,
    pub last_answer: String,
    pub last_probes: Vec<String>,
    pub confidence: u8,
}

impl SpecialistLearningStore {
    /// Load store from disk
    pub fn load() -> Self {
        let path = Self::store_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str(&content) {
                    Ok(store) => {
                        return store;
                    }
                    Err(_e) => { /* Parse error, use default */ }
                },
                Err(_e) => { /* Read error, use default */ }
            }
        }
        Self::default()
    }

    /// Save store to disk
    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Self::store_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    /// Get store path
    fn store_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home).join(".anna").join("specialist_lessons.json")
    }

    /// Record a new lesson from specialist interaction
    /// Returns true if a new lesson was created
    pub fn record_lesson(&mut self, lesson: SpecialistLesson) -> bool {
        let pattern_key = normalize_pattern(&lesson.query_pattern);

        // Adaptive threshold: high confidence (80+) learns immediately
        if lesson.confidence >= 80 {
            self.add_lesson(lesson);
            return true;
        }

        // Lower confidence: check pending patterns
        if let Some(pending) = self.pending_patterns.get_mut(&pattern_key) {
            pending.success_count += 1;
            pending.last_answer = lesson.answer_template.clone();
            pending.last_probes = lesson.effective_probes.clone();

            // Promote to lesson if we have 2+ successes
            if pending.success_count >= 2 {
                let promoted = SpecialistLesson {
                    success_count: pending.success_count,
                    ..lesson
                };
                self.pending_patterns.remove(&pattern_key);
                self.add_lesson(promoted);
                return true;
            }
        } else {
            // Add as pending
            self.pending_patterns.insert(pattern_key, PendingPattern {
                query_pattern: lesson.query_pattern.clone(),
                domain: lesson.domain,
                success_count: 1,
                last_answer: lesson.answer_template,
                last_probes: lesson.effective_probes,
                confidence: lesson.confidence,
            });
        }

        false
    }

    /// Add a lesson to the store
    fn add_lesson(&mut self, lesson: SpecialistLesson) {
        // Index by keywords
        for keyword in extract_keywords(&lesson.query_pattern) {
            self.keyword_index
                .entry(keyword)
                .or_default()
                .push(lesson.id.clone());
        }

        // Index by pattern category if generic
        if let Some(ref pattern) = lesson.generic_pattern {
            self.category_index
                .entry(pattern.category)
                .or_default()
                .push(lesson.id.clone());
        }

        self.lessons.insert(lesson.id.clone(), lesson);
    }

    /// Find relevant lessons for a query
    pub fn find_lessons(&self, query: &str) -> Vec<&SpecialistLesson> {
        let keywords = extract_keywords(query);
        let mut lesson_ids: HashMap<&str, u32> = HashMap::new();

        // Score by keyword matches
        for keyword in &keywords {
            if let Some(ids) = self.keyword_index.get(keyword) {
                for id in ids {
                    *lesson_ids.entry(id.as_str()).or_default() += 1;
                }
            }
        }

        // Sort by match count and return top lessons
        let mut sorted: Vec<_> = lesson_ids.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));

        sorted
            .into_iter()
            .take(5)
            .filter_map(|(id, _)| self.lessons.get(id))
            .collect()
    }

    /// Get a subtle hint about learning if we have relevant lessons
    pub fn get_learning_hint(&self, query: &str) -> Option<String> {
        let lessons = self.find_lessons(query);
        if lessons.is_empty() {
            return None;
        }

        let best = &lessons[0];
        // Only hint if we have high confidence and multiple successes
        if best.confidence >= 70 && best.success_count >= 2 {
            Some(format!("Based on similar cases..."))
        } else if best.success_count >= 3 {
            Some(format!("I've seen this pattern before..."))
        } else {
            None
        }
    }

    /// Get count of lessons
    pub fn lesson_count(&self) -> usize {
        self.lessons.len()
    }

    /// Get count of pending patterns
    pub fn pending_count(&self) -> usize {
        self.pending_patterns.len()
    }
}

/// Normalize a query pattern for consistent matching
fn normalize_pattern(query: &str) -> String {
    query.to_lowercase().trim().to_string()
}

/// Extract keywords from a query for indexing
fn extract_keywords(query: &str) -> Vec<String> {
    query
        .to_lowercase()
        .split_whitespace()
        .filter(|w| w.len() > 2)  // Skip short words
        .filter(|w| !STOP_WORDS.contains(w))
        .map(|w| w.to_string())
        .collect()
}

/// Common words to skip when indexing
const STOP_WORDS: &[&str] = &[
    "the", "and", "for", "are", "but", "not", "you", "all", "can", "had",
    "her", "was", "one", "our", "out", "has", "have", "been", "were", "being",
    "what", "when", "where", "which", "while", "who", "whom", "this", "that",
    "these", "those", "then", "than", "some", "such", "into", "from", "with",
    "how", "why", "does", "will", "would", "could", "should", "may", "might",
];

/// Detect if a query fits a generic pattern category
pub fn detect_pattern_category(query: &str) -> Option<PatternCategory> {
    let q = query.to_lowercase();

    // Config check patterns
    if (q.contains("config") || q.contains("configuration"))
        && (q.contains("check") || q.contains("show") || q.contains("view") || q.contains("see"))
    {
        return Some(PatternCategory::ConfigCheck);
    }

    // Config edit patterns - "enable X in Y" or "set X in Y" (even without "config" word)
    if q.contains("enable") || q.contains("disable") || q.contains("set") || q.contains("add") {
        // If it contains "in" with an app name, or mentions config/setting
        if q.contains(" in ") || q.contains("config") || q.contains("setting") {
            return Some(PatternCategory::ConfigEdit);
        }
    }

    // Service action patterns
    if q.contains("service") || q.contains("systemd") || q.contains("daemon")
        || q.contains("start") || q.contains("stop") || q.contains("restart")
    {
        return Some(PatternCategory::ServiceAction);
    }

    // Package query patterns
    if q.contains("installed") || q.contains("package") || q.contains("install")
    {
        return Some(PatternCategory::PackageQuery);
    }

    // Disk analysis patterns
    if q.contains("disk") || q.contains("space") || q.contains("storage")
        || q.contains("folder") || q.contains("directory")
    {
        return Some(PatternCategory::DiskAnalysis);
    }

    // Process query patterns
    if q.contains("cpu") || q.contains("memory") || q.contains("process")
        || q.contains("using") || q.contains("consuming")
    {
        return Some(PatternCategory::ProcessQuery);
    }

    None
}

/// Extract the target variable from a query (e.g., "hyprland" from "check hyprland config")
pub fn extract_target(query: &str) -> Option<String> {
    let q = query.to_lowercase();
    let words: Vec<&str> = q.split_whitespace().collect();

    // Priority 1: Look for known app/service names first
    for word in &words {
        if is_known_target(word) {
            return Some(word.to_string());
        }
    }

    // Priority 2: Look for word before "config", "service", "package"
    for (i, word) in words.iter().enumerate() {
        if *word == "config" || *word == "configuration" || *word == "service" || *word == "package" {
            if i > 0 {
                return Some(words[i - 1].to_string());
            }
        }
    }

    None
}

/// Known apps, services, editors, etc.
const KNOWN_TARGETS: &[&str] = &[
    "vim", "nvim", "neovim", "nano", "emacs", "helix", "hx", "code", "vscode",
    "hyprland", "sway", "i3", "bspwm", "awesome", "dwm", "qtile",
    "bash", "zsh", "fish", "tmux", "alacritty", "kitty", "wezterm",
    "nginx", "apache", "postgres", "mysql", "redis", "docker", "podman",
    "ssh", "sshd", "cups", "bluetooth", "networkmanager", "pipewire", "pulseaudio",
];

/// Check if word is a known target
fn is_known_target(word: &str) -> bool {
    KNOWN_TARGETS.contains(&word)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_pattern_category() {
        assert_eq!(detect_pattern_category("check hyprland config"), Some(PatternCategory::ConfigCheck));
        assert_eq!(detect_pattern_category("enable syntax highlighting in vim"), Some(PatternCategory::ConfigEdit));
        assert_eq!(detect_pattern_category("restart nginx service"), Some(PatternCategory::ServiceAction));
        assert_eq!(detect_pattern_category("is docker installed"), Some(PatternCategory::PackageQuery));
        assert_eq!(detect_pattern_category("what folders are taking space"), Some(PatternCategory::DiskAnalysis));
    }

    #[test]
    fn test_extract_target() {
        assert_eq!(extract_target("check hyprland config"), Some("hyprland".to_string()));
        assert_eq!(extract_target("restart nginx service"), Some("nginx".to_string()));
        assert_eq!(extract_target("is vim installed"), Some("vim".to_string()));
    }

    #[test]
    fn test_extract_keywords() {
        let kw = extract_keywords("what folders are taking the most space");
        assert!(kw.contains(&"folders".to_string()) && kw.contains(&"space".to_string()));
        assert!(!kw.contains(&"the".to_string()) && !kw.contains(&"are".to_string()));
    }
}
