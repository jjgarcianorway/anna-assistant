//! Session Memory - Context awareness within a conversation.
//!
//! This module provides:
//! - Memory of what was discussed in the current session
//! - Context carry-over between questions
//! - Reference resolution ("it", "that service", "the error")
//! - Topic tracking for more relevant answers
//! - Persistence across daemon restarts

use crate::config::anna_data_dir;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;

/// Maximum number of turns to remember
const MAX_HISTORY: usize = 20;

/// A session maintains conversational context
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Session {
    /// Session ID
    pub id: String,

    /// Conversation history
    pub history: VecDeque<Turn>,

    /// Current topic/context
    pub context: SessionContext,

    /// Entities mentioned in conversation (packages, services, files, etc.)
    pub entities: SessionEntities,

    /// When this session started
    pub started_at: String,

    /// Last activity timestamp
    pub last_activity: String,
}

/// A single turn in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Turn {
    /// User's question
    pub question: String,

    /// Anna's answer
    pub answer: String,

    /// Commands that were run
    pub commands: Vec<String>,

    /// When this turn occurred
    pub timestamp: String,

    /// Entities extracted from this turn
    pub entities_mentioned: Vec<String>,
}

/// Current session context
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionContext {
    /// Main topic being discussed
    pub current_topic: Option<String>,

    /// Sub-topics explored
    pub explored_topics: Vec<String>,

    /// What the user seems to be trying to accomplish
    pub apparent_goal: Option<String>,

    /// Level of detail the user prefers
    pub detail_preference: DetailLevel,
}

/// How much detail the user wants
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub enum DetailLevel {
    /// Just the facts
    Minimal,
    /// Normal explanations
    #[default]
    Normal,
    /// Detailed with context
    Verbose,
}

/// Entities mentioned in the session
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionEntities {
    /// Packages mentioned
    pub packages: Vec<String>,

    /// Services mentioned
    pub services: Vec<String>,

    /// Files/paths mentioned
    pub files: Vec<String>,

    /// Users mentioned
    pub users: Vec<String>,

    /// Commands that were run
    pub commands_run: Vec<String>,

    /// Errors encountered
    pub errors: Vec<String>,
}

impl Session {
    /// Create a new session
    pub fn new() -> Self {
        Session {
            id: uuid::Uuid::new_v4().to_string(),
            history: VecDeque::new(),
            context: SessionContext::default(),
            entities: SessionEntities::default(),
            started_at: chrono::Utc::now().to_rfc3339(),
            last_activity: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Add a turn to the conversation
    pub fn add_turn(&mut self, question: &str, answer: &str, commands: Vec<String>) {
        // Extract entities from the question and answer
        let entities = extract_entities(question, answer);

        // Update session entities
        self.merge_entities(&entities);

        // Update context based on new turn
        self.update_context(question, answer);

        // Add the turn
        let turn = Turn {
            question: question.to_string(),
            answer: answer.to_string(),
            commands,
            timestamp: chrono::Utc::now().to_rfc3339(),
            entities_mentioned: entities,
        };

        self.history.push_back(turn);

        // Keep history bounded
        while self.history.len() > MAX_HISTORY {
            self.history.pop_front();
        }

        self.last_activity = chrono::Utc::now().to_rfc3339();
    }

    /// Merge new entities into session entities
    fn merge_entities(&mut self, new_entities: &[String]) {
        for entity in new_entities {
            // Categorize and add to appropriate list
            if entity.ends_with(".service") || entity.ends_with(".socket") || entity.ends_with(".timer") {
                if !self.entities.services.contains(entity) {
                    self.entities.services.push(entity.clone());
                }
            } else if entity.starts_with('/') || entity.starts_with('~') || entity.contains('.') && entity.contains('/') {
                if !self.entities.files.contains(entity) {
                    self.entities.files.push(entity.clone());
                }
            } else if !entity.contains(' ') && entity.len() > 2 {
                // Likely a package name
                if !self.entities.packages.contains(entity) {
                    self.entities.packages.push(entity.clone());
                }
            }
        }
    }

    /// Update context based on conversation
    fn update_context(&mut self, question: &str, _answer: &str) {
        let q_lower = question.to_lowercase();

        // Detect topic changes
        let new_topic = detect_topic(&q_lower);
        if let Some(topic) = new_topic {
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
        if q_lower.contains("install") || q_lower.contains("setup") || q_lower.contains("configure") {
            self.context.apparent_goal = Some("Configuration/Setup".to_string());
        } else if q_lower.contains("fix") || q_lower.contains("error") || q_lower.contains("not working") {
            self.context.apparent_goal = Some("Troubleshooting".to_string());
        } else if q_lower.contains("why") || q_lower.contains("how does") || q_lower.contains("explain") {
            self.context.apparent_goal = Some("Understanding".to_string());
        }

        // Detect detail preference
        if q_lower.contains("briefly") || q_lower.contains("quick") || q_lower.contains("just tell") {
            self.context.detail_preference = DetailLevel::Minimal;
        } else if q_lower.contains("explain") || q_lower.contains("detail") || q_lower.contains("why") {
            self.context.detail_preference = DetailLevel::Verbose;
        }
    }

    /// Resolve references like "it", "that", "the service"
    pub fn resolve_reference(&self, reference: &str) -> Option<String> {
        let ref_lower = reference.to_lowercase();

        // "it", "that", "this" - refer to most recent entity
        if ref_lower == "it" || ref_lower == "that" || ref_lower == "this" {
            // Check last turn for entities
            if let Some(last) = self.history.back() {
                if let Some(entity) = last.entities_mentioned.first() {
                    return Some(entity.clone());
                }
            }
        }

        // "the service" - most recent service
        if ref_lower.contains("service") || ref_lower.contains("unit") {
            return self.entities.services.last().cloned();
        }

        // "the package"
        if ref_lower.contains("package") {
            return self.entities.packages.last().cloned();
        }

        // "the file" / "that config"
        if ref_lower.contains("file") || ref_lower.contains("config") {
            return self.entities.files.last().cloned();
        }

        // "the error"
        if ref_lower.contains("error") {
            return self.entities.errors.last().cloned();
        }

        None
    }

    /// Get context for LLM prompts (optimized for smaller token usage)
    pub fn get_context_for_llm(&self) -> String {
        let mut context = String::new();

        // Current topic (most important)
        if let Some(ref topic) = self.context.current_topic {
            context.push_str(&format!("Topic: {}\n", topic));
        }

        // Recent history - only include last 2 questions (save tokens)
        if !self.history.is_empty() {
            context.push_str("Previous:\n");
            for turn in self.history.iter().rev().take(2) {
                context.push_str(&format!("- {}\n", truncate(&turn.question, 60)));
            }
        }

        // Entities - compress if too many (save tokens)
        if !self.entities.services.is_empty() {
            let services = &self.entities.services;
            if services.len() <= 3 {
                context.push_str(&format!("Services: {}\n", services.join(", ")));
            } else {
                // Show count + most recent
                context.push_str(&format!(
                    "Services: {} discussed, recent: {}\n",
                    services.len(),
                    services.iter().rev().take(2).cloned().collect::<Vec<_>>().join(", ")
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
                    packages.iter().rev().take(3).cloned().collect::<Vec<_>>().join(", ")
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
                    files.iter().rev().take(2).cloned().collect::<Vec<_>>().join(", ")
                ));
            }
        }

        // Goal (if relevant)
        if let Some(ref goal) = self.context.apparent_goal {
            context.push_str(&format!("Goal: {}\n", truncate(goal, 50)));
        }

        context
    }

    /// Get a brief context summary for command selection (minimal tokens)
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

        // Resolve references
        let references = ["it", "that", "this", "the service", "the package", "the file", "the error"];

        for reference in references {
            if expanded.to_lowercase().contains(reference) {
                if let Some(resolved) = self.resolve_reference(reference) {
                    // Only replace if it makes sense
                    expanded = expanded.replace(reference, &resolved);
                }
            }
        }

        expanded
    }
}

/// Extract entities from text
fn extract_entities(question: &str, answer: &str) -> Vec<String> {
    let mut entities = Vec::new();
    let combined = format!("{} {}", question, answer);

    // Extract service names (*.service)
    for word in combined.split_whitespace() {
        let clean = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '.' && c != '-' && c != '_' && c != '/');

        if clean.ends_with(".service") || clean.ends_with(".socket") || clean.ends_with(".timer") {
            entities.push(clean.to_string());
        }

        // File paths
        if clean.starts_with('/') && clean.len() > 3 {
            entities.push(clean.to_string());
        }

        // Package names (usually lowercase with dashes)
        if clean.chars().all(|c| c.is_lowercase() || c == '-' || c.is_numeric())
            && clean.len() > 2
            && !clean.starts_with('-')
        {
            // Avoid common words
            let common = ["the", "and", "for", "with", "from", "that", "this", "have"];
            if !common.contains(&clean) {
                entities.push(clean.to_string());
            }
        }
    }

    entities.sort();
    entities.dedup();
    entities
}

/// Detect the main topic from a question
fn detect_topic(question: &str) -> Option<String> {
    let topics = [
        ("network", &["network", "wifi", "ethernet", "ip", "dns", "connection"][..]),
        ("audio", &["audio", "sound", "speaker", "microphone", "pulseaudio", "pipewire"]),
        ("display", &["display", "screen", "monitor", "resolution", "wayland", "x11", "xorg"]),
        ("boot", &["boot", "grub", "systemd-boot", "kernel", "initramfs"]),
        ("storage", &["disk", "partition", "mount", "filesystem", "btrfs", "ext4", "storage"]),
        ("packages", &["package", "install", "pacman", "yay", "aur", "update"]),
        ("services", &["service", "systemd", "daemon", "unit"]),
        ("security", &["security", "firewall", "permission", "sudo", "password"]),
        ("performance", &["slow", "performance", "cpu", "memory", "ram"]),
    ];

    for (topic, keywords) in topics {
        if keywords.iter().any(|k| question.contains(k)) {
            return Some(topic.to_string());
        }
    }

    None
}

/// Truncate string with ellipsis
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}

/// Persistent storage for sessions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionStore {
    /// Sessions by ID
    pub sessions: HashMap<String, Session>,
    /// Last save timestamp
    pub last_saved: Option<String>,
}

impl SessionStore {
    /// Create a new session store
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            last_saved: None,
        }
    }

    /// Load sessions from disk
    pub fn load() -> Result<Self> {
        let path = sessions_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let mut store: SessionStore = serde_json::from_str(&content)?;
            // Clean up old sessions on load
            store.cleanup_old_sessions();
            Ok(store)
        } else {
            Ok(Self::new())
        }
    }

    /// Save sessions to disk
    pub fn save(&mut self) -> Result<()> {
        let path = sessions_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        self.last_saved = Some(chrono::Utc::now().to_rfc3339());
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Get or create a session
    pub fn get_or_create(&mut self, session_id: &str) -> &mut Session {
        self.sessions.entry(session_id.to_string()).or_insert_with(|| {
            let mut session = Session::new();
            session.id = session_id.to_string();
            session
        })
    }

    /// Remove sessions older than 24 hours
    pub fn cleanup_old_sessions(&mut self) {
        let now = chrono::Utc::now();
        self.sessions.retain(|_, session| {
            if let Ok(last_activity) = chrono::DateTime::parse_from_rfc3339(&session.last_activity) {
                let duration = now.signed_duration_since(last_activity);
                duration.num_hours() < 24
            } else {
                false // Remove if timestamp is unparseable
            }
        });
    }

    /// Get number of active sessions
    pub fn active_count(&self) -> usize {
        self.sessions.len()
    }
}

/// Get sessions file path
pub fn sessions_path() -> PathBuf {
    anna_data_dir().join("sessions.json")
}
