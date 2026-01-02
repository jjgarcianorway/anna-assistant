//! Store for specialist lessons

use super::types::{PatternCategory, PendingPattern, SpecialistLesson};
use super::utils::{extract_keywords, normalize_pattern};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

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
        PathBuf::from(home)
            .join(".anna")
            .join("specialist_lessons.json")
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
            self.pending_patterns.insert(
                pattern_key,
                PendingPattern {
                    query_pattern: lesson.query_pattern.clone(),
                    domain: lesson.domain,
                    success_count: 1,
                    last_answer: lesson.answer_template,
                    last_probes: lesson.effective_probes,
                    confidence: lesson.confidence,
                },
            );
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
