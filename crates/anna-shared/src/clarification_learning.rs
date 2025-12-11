//! Learning from user clarifications (v0.0.401).
//!
//! When users answer clarification questions, we capture:
//! - Preferences (e.g., "I prefer vim over nano")
//! - System facts (e.g., "my editor is vim")
//! - Query patterns (e.g., "when I say X I mean Y")
//!
//! This makes Anna smarter over time.

use crate::clarify_v2::ClarifyRequest;
use crate::context_memory::ContextMemory;
use crate::facts::{FactKey, FactsStore};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// A lesson learned from a clarification exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarificationLesson {
    /// What question was asked
    pub question_pattern: String,
    /// What the user chose/answered
    pub user_answer: String,
    /// Derived fact key (if any)
    pub fact_key: Option<String>,
    /// Derived preference (if any)
    pub preference: Option<String>,
    /// Confidence in this lesson (0-100)
    pub confidence: u8,
    /// When learned
    pub learned_at: DateTime<Utc>,
    /// How many times confirmed
    pub confirmations: u32,
}

impl ClarificationLesson {
    pub fn new(question: &str, answer: &str) -> Self {
        Self {
            question_pattern: normalize_question(question),
            user_answer: answer.to_string(),
            fact_key: None,
            preference: None,
            confidence: 60,
            learned_at: Utc::now(),
            confirmations: 1,
        }
    }

    pub fn with_fact(mut self, key: &str) -> Self {
        self.fact_key = Some(key.to_string());
        self.confidence = 80;
        self
    }

    pub fn with_preference(mut self, pref: &str) -> Self {
        self.preference = Some(pref.to_string());
        self
    }

    /// Reinforce this lesson (user confirmed same answer again)
    pub fn reinforce(&mut self) {
        self.confirmations += 1;
        self.confidence = (self.confidence + 10).min(100);
        self.learned_at = Utc::now();
    }

    /// Is this a high-confidence lesson?
    pub fn is_trusted(&self) -> bool {
        self.confidence >= 80 || self.confirmations >= 2
    }
}

/// Store of lessons learned from clarifications
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClarificationLearningStore {
    /// Lessons indexed by normalized question pattern
    pub lessons: HashMap<String, ClarificationLesson>,
    /// Quick lookup: question pattern -> answer
    pub quick_answers: HashMap<String, String>,
}

impl ClarificationLearningStore {
    pub fn load() -> Self {
        Self::path()
            .and_then(|p| fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> std::io::Result<()> {
        if let Some(path) = Self::path() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let data = serde_json::to_string_pretty(self)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            fs::write(path, data)?;
        }
        Ok(())
    }

    fn path() -> Option<PathBuf> {
        dirs::data_dir().map(|d| d.join("anna").join("clarification_lessons.json"))
    }

    /// Learn from a clarification response
    pub fn learn_from_response(
        &mut self,
        request: &ClarifyRequest,
        selected_value: &str,
        fact_key: Option<&str>,
    ) {
        let pattern = normalize_question(&request.question);

        if let Some(existing) = self.lessons.get_mut(&pattern) {
            if existing.user_answer == selected_value {
                existing.reinforce();
            } else {
                // User changed their preference - update
                existing.user_answer = selected_value.to_string();
                existing.confidence = 70;
                existing.learned_at = Utc::now();
            }
        } else {
            let mut lesson = ClarificationLesson::new(&request.question, selected_value);
            if let Some(key) = fact_key {
                lesson = lesson.with_fact(key);
            }
            self.lessons.insert(pattern.clone(), lesson);
        }

        // Update quick answers for trusted lessons
        self.rebuild_quick_answers();
    }

    /// Can we auto-answer this clarification from learning?
    pub fn can_auto_answer(&self, request: &ClarifyRequest) -> Option<&str> {
        let pattern = normalize_question(&request.question);
        if let Some(lesson) = self.lessons.get(&pattern) {
            if lesson.is_trusted() {
                // Verify the option is still valid
                if request.options.iter().any(|o| o.value == lesson.user_answer) {
                    return Some(&lesson.user_answer);
                }
            }
        }
        // Also check quick answers
        self.quick_answers.get(&pattern).map(|s| s.as_str())
    }

    /// Get all high-confidence lessons
    pub fn trusted_lessons(&self) -> Vec<&ClarificationLesson> {
        self.lessons.values().filter(|l| l.is_trusted()).collect()
    }

    fn rebuild_quick_answers(&mut self) {
        self.quick_answers.clear();
        for (pattern, lesson) in &self.lessons {
            if lesson.is_trusted() {
                self.quick_answers
                    .insert(pattern.clone(), lesson.user_answer.clone());
            }
        }
    }
}

/// Normalize a question for pattern matching
fn normalize_question(question: &str) -> String {
    question
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != ' ', "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Extract learnable facts from a clarification exchange
pub fn extract_facts(
    request: &ClarifyRequest,
    answer: &str,
    facts_store: &mut FactsStore,
) -> Option<String> {
    let q = request.question.to_lowercase();

    // Editor preference questions
    if q.contains("editor") || q.contains("text editor") {
        if is_known_editor(answer) {
            facts_store.set_verified(
                FactKey::PreferredEditor,
                answer.to_string(),
                "clarification".to_string(),
            );
            return Some("preferred_editor".to_string());
        }
    }

    // Shell preference questions
    if q.contains("shell") || q.contains("terminal") {
        if is_known_shell(answer) {
            facts_store.set_verified(
                FactKey::PreferredShell,
                answer.to_string(),
                "clarification".to_string(),
            );
            return Some("preferred_shell".to_string());
        }
    }

    None
}

/// Update context memory from clarification
pub fn update_context_memory(
    request: &ClarifyRequest,
    answer: &str,
    memory: &mut ContextMemory,
) {
    let q = request.question.to_lowercase();

    // Editor preference
    if q.contains("editor") && is_known_editor(answer) {
        memory.detect_editor(answer);
    }

    // Shell preference
    if q.contains("shell") && is_known_shell(answer) {
        memory.preferred_shell = Some(answer.to_string());
    }

    // Learn as preference
    let pref = format!("prefers {} for {}", answer, extract_topic(&q));
    memory.learn_preference(&pref, "clarification response", 0.8);
}

fn is_known_editor(name: &str) -> bool {
    let known = [
        "vim", "nvim", "neovim", "nano", "emacs", "code", "vscode", "kate", "gedit", "sublime",
        "helix", "micro",
    ];
    known.iter().any(|e| name.to_lowercase().contains(e))
}

fn is_known_shell(name: &str) -> bool {
    let known = ["bash", "zsh", "fish", "sh", "dash", "nu", "nushell"];
    known.iter().any(|s| name.to_lowercase().contains(s))
}

fn extract_topic(question: &str) -> String {
    // Extract the main topic from a question
    let q = question.to_lowercase();
    if q.contains("editor") {
        return "editing".to_string();
    }
    if q.contains("shell") {
        return "shell".to_string();
    }
    if q.contains("browser") {
        return "browsing".to_string();
    }
    "this task".to_string()
}

/// Integrate learning: call after successful clarification
pub fn record_clarification_learning(
    request: &ClarifyRequest,
    answer: &str,
) {
    let mut store = ClarificationLearningStore::load();
    let mut facts = FactsStore::load();
    let mut memory = ContextMemory::load();

    // Learn fact if applicable
    let fact_key = extract_facts(request, answer, &mut facts);

    // Update context memory
    update_context_memory(request, answer, &mut memory);

    // Record the lesson
    store.learn_from_response(request, answer, fact_key.as_deref());

    // Save all stores
    let _ = store.save();
    let _ = facts.save();
    let _ = memory.save();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clarify_v2::ClarifyOption;

    #[test]
    fn test_normalize_question() {
        assert_eq!(
            normalize_question("Which editor do you prefer?"),
            "which editor do you prefer"
        );
        assert_eq!(
            normalize_question("What's your   favorite  editor??"),
            "whats your favorite editor"
        );
    }

    #[test]
    fn test_lesson_reinforcement() {
        let mut lesson = ClarificationLesson::new("which editor?", "vim");
        assert_eq!(lesson.confidence, 60);
        assert!(!lesson.is_trusted());

        lesson.reinforce();
        assert_eq!(lesson.confidence, 70);
        assert_eq!(lesson.confirmations, 2);
        assert!(lesson.is_trusted());
    }

    #[test]
    fn test_can_auto_answer() {
        let mut store = ClarificationLearningStore::default();

        let request = ClarifyRequest::new("test", "Which editor do you prefer?")
            .add_option(ClarifyOption::new(1, "vim", "vim"))
            .add_option(ClarifyOption::new(2, "nano", "nano"));

        // First time - no auto answer
        assert!(store.can_auto_answer(&request).is_none());

        // Learn from response
        store.learn_from_response(&request, "vim", None);

        // Still not trusted (only 1 confirmation, confidence 60)
        assert!(store.can_auto_answer(&request).is_none());

        // Reinforce
        store.learn_from_response(&request, "vim", None);

        // Now trusted
        assert_eq!(store.can_auto_answer(&request), Some("vim"));
    }

    #[test]
    fn test_is_known_editor() {
        assert!(is_known_editor("vim"));
        assert!(is_known_editor("nvim"));
        assert!(is_known_editor("VSCode"));
        assert!(!is_known_editor("random"));
    }
}
