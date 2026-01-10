//! Session Memory - Context awareness within a conversation.
//!
//! This module provides:
//! - Memory of what was discussed in the current session
//! - Context carry-over between questions
//! - Reference resolution ("it", "that service", "the error")
//! - Topic tracking for more relevant answers
//! - Persistence across daemon restarts

mod helpers;
mod patterns;
mod preferences;
mod store;
mod types;

pub use patterns::{CrossSessionPatterns, FrequentPattern, RecurringIssue, TopicFlow};
pub use preferences::{ContradictionPattern, ContradictionStore, ResponseStyle, UserPreferences};
pub use store::{sessions_path, SessionStore};
pub use types::{DetailLevel, Session, SessionContext, SessionEntities, Turn};

use crate::config::AnnaConfig;
use helpers::{detect_topic, extract_entities, truncate};

/// v0.0.893: Get max history from config
fn get_max_history() -> usize {
    AnnaConfig::load()
        .map(|c| c.performance.max_session_history)
        .unwrap_or(20)
}

impl Session {
    /// Create a new session
    pub fn new() -> Self {
        Session {
            id: uuid::Uuid::new_v4().to_string(),
            history: std::collections::VecDeque::new(),
            context: SessionContext::default(),
            entities: SessionEntities::default(),
            started_at: chrono::Utc::now().to_rfc3339(),
            last_activity: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Add a turn to the conversation
    pub fn add_turn(&mut self, question: &str, answer: &str, commands: Vec<String>) {
        let entities = extract_entities(question, answer);
        self.merge_entities(&entities);
        self.update_context(question, answer);

        let turn = Turn {
            question: question.to_string(),
            answer: answer.to_string(),
            commands,
            timestamp: chrono::Utc::now().to_rfc3339(),
            entities_mentioned: entities,
        };

        self.history.push_back(turn);

        let max_history = get_max_history();
        while self.history.len() > max_history {
            self.history.pop_front();
        }

        self.last_activity = chrono::Utc::now().to_rfc3339();
    }

    /// Merge new entities into session entities
    fn merge_entities(&mut self, new_entities: &[String]) {
        for entity in new_entities {
            if entity.ends_with(".service")
                || entity.ends_with(".socket")
                || entity.ends_with(".timer")
            {
                if !self.entities.services.contains(entity) {
                    self.entities.services.push(entity.clone());
                }
            } else if entity.starts_with('/')
                || entity.starts_with('~')
                || entity.contains('.') && entity.contains('/')
            {
                if !self.entities.files.contains(entity) {
                    self.entities.files.push(entity.clone());
                }
            } else if !entity.contains(' ') && entity.len() > 2 {
                if !self.entities.packages.contains(entity) {
                    self.entities.packages.push(entity.clone());
                }
            }
        }
    }

    /// Update context based on conversation
    fn update_context(&mut self, question: &str, _answer: &str) {
        let q_lower = question.to_lowercase();

        if let Some(topic) = detect_topic(&q_lower) {
            if self.context.current_topic.as_ref() != Some(&topic) {
                if let Some(old) = self.context.current_topic.take() {
                    if !self.context.explored_topics.contains(&old) {
                        self.context.explored_topics.push(old);
                    }
                }
                self.context.current_topic = Some(topic);
            }
        }

        // Detect apparent goal
        if q_lower.contains("install")
            || q_lower.contains("setup")
            || q_lower.contains("configure")
        {
            self.context.apparent_goal = Some("Configuration/Setup".to_string());
        } else if q_lower.contains("fix")
            || q_lower.contains("error")
            || q_lower.contains("not working")
        {
            self.context.apparent_goal = Some("Troubleshooting".to_string());
        } else if q_lower.contains("why")
            || q_lower.contains("how does")
            || q_lower.contains("explain")
        {
            self.context.apparent_goal = Some("Understanding".to_string());
        }

        // Detect detail preference
        if q_lower.contains("briefly") || q_lower.contains("quick") || q_lower.contains("just tell")
        {
            self.context.detail_preference = DetailLevel::Minimal;
        } else if q_lower.contains("explain")
            || q_lower.contains("detail")
            || q_lower.contains("why")
        {
            self.context.detail_preference = DetailLevel::Verbose;
        }
    }

    /// Resolve references like "it", "that", "the service"
    pub fn resolve_reference(&self, reference: &str) -> Option<String> {
        let ref_lower = reference.to_lowercase();

        if ref_lower == "it" || ref_lower == "that" || ref_lower == "this" {
            if let Some(last) = self.history.back() {
                if let Some(entity) = last.entities_mentioned.first() {
                    return Some(entity.clone());
                }
            }
        }

        if ref_lower.contains("service") || ref_lower.contains("unit") {
            return self.entities.services.last().cloned();
        }

        if ref_lower.contains("package") {
            return self.entities.packages.last().cloned();
        }

        if ref_lower.contains("file") || ref_lower.contains("config") {
            return self.entities.files.last().cloned();
        }

        if ref_lower.contains("error") {
            return self.entities.errors.last().cloned();
        }

        None
    }

    /// Get context for LLM prompts
    pub fn get_context_for_llm(&self) -> String {
        let mut context = String::new();

        if let Some(ref topic) = self.context.current_topic {
            context.push_str(&format!("Topic: {}\n", topic));
        }

        if !self.history.is_empty() {
            context.push_str("Previous:\n");
            for turn in self.history.iter().rev().take(2) {
                context.push_str(&format!("- {}\n", truncate(&turn.question, 60)));
            }
        }

        if !self.entities.services.is_empty() {
            let services = &self.entities.services;
            if services.len() <= 3 {
                context.push_str(&format!("Services: {}\n", services.join(", ")));
            } else {
                context.push_str(&format!(
                    "Services: {} discussed, recent: {}\n",
                    services.len(),
                    services
                        .iter()
                        .rev()
                        .take(2)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }

        if !self.entities.packages.is_empty() {
            let packages = &self.entities.packages;
            if packages.len() <= 5 {
                context.push_str(&format!("Packages: {}\n", packages.join(", ")));
            } else {
                context.push_str(&format!(
                    "Packages: {} discussed, recent: {}\n",
                    packages.len(),
                    packages
                        .iter()
                        .rev()
                        .take(3)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }

        if !self.entities.files.is_empty() {
            let files = &self.entities.files;
            if files.len() <= 3 {
                context.push_str(&format!("Files: {}\n", files.join(", ")));
            } else {
                context.push_str(&format!(
                    "Files: {} referenced, recent: {}\n",
                    files.len(),
                    files
                        .iter()
                        .rev()
                        .take(2)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }

        if let Some(ref goal) = self.context.apparent_goal {
            context.push_str(&format!("Goal: {}\n", truncate(goal, 50)));
        }

        context
    }

    /// Get a brief context summary for command selection
    pub fn get_brief_context(&self) -> String {
        let mut brief = String::new();
        if let Some(ref topic) = self.context.current_topic {
            brief.push_str(&format!("Topic: {}. ", truncate(topic, 30)));
        }
        if let Some(last) = self.history.back() {
            brief.push_str(&format!("Last Q: {}", truncate(&last.question, 40)));
        }
        brief
    }

    /// Expand a question with context
    pub fn expand_question(&self, question: &str) -> String {
        let mut expanded = question.to_string();

        let references = [
            "it",
            "that",
            "this",
            "the service",
            "the package",
            "the file",
            "the error",
        ];

        for reference in references {
            if expanded.to_lowercase().contains(reference) {
                if let Some(resolved) = self.resolve_reference(reference) {
                    expanded = expanded.replace(reference, &resolved);
                }
            }
        }

        expanded
    }
}
