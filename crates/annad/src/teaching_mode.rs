//! Teaching Mode - Anna explains concepts and remembers what user knows.
//!
//! Philosophy: Teach once, remember knowledge level, adapt explanations.
//! NO HARDCODING: LLM generates explanations, remembers context.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, info};

/// User's knowledge database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBase {
    /// Topics the user has learned
    pub learned_topics: HashMap<String, LearnedTopic>,
    /// User's overall expertise level
    pub expertise_level: ExpertiseLevel,
    /// Teaching preferences
    pub preferences: TeachingPreferences,
}

/// A topic the user has learned.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedTopic {
    /// Topic name (normalized)
    pub topic: String,
    /// When it was taught
    pub taught_at: DateTime<Utc>,
    /// How many times user has asked about it
    pub reinforcement_count: u32,
    /// Mastery level (0.0-1.0)
    pub mastery: f32,
    /// Last time user asked about it
    pub last_referenced: DateTime<Utc>,
}

/// User's expertise level.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExpertiseLevel {
    Beginner,       // Needs detailed explanations
    Intermediate,   // Knows basics, needs context
    Advanced,       // Minimal explanation, focus on specifics
    Expert,         // Just the facts
}

/// Teaching preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeachingPreferences {
    /// Include examples
    pub include_examples: bool,
    /// Include analogies
    pub use_analogies: bool,
    /// Show commands (for learning)
    pub show_commands: bool,
    /// Depth level (1-5)
    pub depth_level: u8,
}

impl Default for TeachingPreferences {
    fn default() -> Self {
        Self {
            include_examples: true,
            use_analogies: true,
            show_commands: true,
            depth_level: 3,
        }
    }
}

impl KnowledgeBase {
    /// Load knowledge base from disk.
    pub fn load() -> Self {
        let path = Self::storage_path();

        if let Ok(contents) = std::fs::read_to_string(&path) {
            if let Ok(kb) = serde_json::from_str(&contents) {
                return kb;
            }
        }

        // Default
        Self {
            learned_topics: HashMap::new(),
            expertise_level: ExpertiseLevel::Intermediate,
            preferences: TeachingPreferences::default(),
        }
    }

    /// Save knowledge base to disk.
    pub fn save(&self) -> Result<()> {
        let path = Self::storage_path();

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;

        Ok(())
    }

    fn storage_path() -> PathBuf {
        PathBuf::from("/var/lib/anna/knowledge_base.json")
    }

    /// Record that a topic was taught.
    pub fn record_teaching(&mut self, topic: &str) {
        let normalized = normalize_topic(topic);

        if let Some(learned) = self.learned_topics.get_mut(&normalized) {
            learned.reinforcement_count += 1;
            learned.last_referenced = Utc::now();

            // Increase mastery slightly with each reinforcement
            learned.mastery = (learned.mastery + 0.1).min(1.0);
        } else {
            self.learned_topics.insert(
                normalized.clone(),
                LearnedTopic {
                    topic: normalized,
                    taught_at: Utc::now(),
                    reinforcement_count: 1,
                    mastery: 0.3, // Initial mastery
                    last_referenced: Utc::now(),
                },
            );
        }
    }

    /// Check if user already knows this topic.
    pub fn knows_topic(&self, topic: &str) -> Option<&LearnedTopic> {
        let normalized = normalize_topic(topic);
        self.learned_topics.get(&normalized)
    }

    /// Get teaching context for LLM.
    pub fn get_teaching_context(&self, topic: &str) -> TeachingContext {
        let normalized = normalize_topic(topic);
        let learned = self.learned_topics.get(&normalized);

        let already_knows = learned.is_some();
        let mastery = learned.map(|l| l.mastery).unwrap_or(0.0);

        TeachingContext {
            topic: topic.to_string(),
            expertise_level: self.expertise_level,
            already_taught: already_knows,
            mastery_level: mastery,
            preferences: self.preferences.clone(),
            reinforcement_count: learned.map(|l| l.reinforcement_count).unwrap_or(0),
        }
    }

    /// Update expertise level based on interactions.
    pub fn adjust_expertise(&mut self) {
        let total_topics = self.learned_topics.len();
        let mastered_topics = self.learned_topics.values()
            .filter(|t| t.mastery > 0.8)
            .count();

        let mastery_ratio = if total_topics > 0 {
            mastered_topics as f32 / total_topics as f32
        } else {
            0.0
        };

        // Adjust expertise level based on mastery
        self.expertise_level = if total_topics > 50 && mastery_ratio > 0.7 {
            ExpertiseLevel::Expert
        } else if total_topics > 20 && mastery_ratio > 0.5 {
            ExpertiseLevel::Advanced
        } else if total_topics > 5 {
            ExpertiseLevel::Intermediate
        } else {
            ExpertiseLevel::Beginner
        };
    }
}

/// Teaching context for LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeachingContext {
    pub topic: String,
    pub expertise_level: ExpertiseLevel,
    pub already_taught: bool,
    pub mastery_level: f32,
    pub preferences: TeachingPreferences,
    pub reinforcement_count: u32,
}

/// Normalize topic for comparison.
fn normalize_topic(topic: &str) -> String {
    topic
        .to_lowercase()
        .trim()
        .replace("what is ", "")
        .replace("how does ", "")
        .replace("what are ", "")
        .replace("?", "")
        .trim()
        .to_string()
}

/// Detect if question is a teaching request.
pub fn is_teaching_request(question: &str) -> bool {
    let q_lower = question.to_lowercase();

    q_lower.starts_with("what is ")
        || q_lower.starts_with("what are ")
        || q_lower.starts_with("how does ")
        || q_lower.starts_with("how do ")
        || q_lower.starts_with("explain ")
        || q_lower.starts_with("teach me ")
        || q_lower.starts_with("tell me about ")
        || q_lower.contains("how to ")
}

/// Extract topic from teaching question.
pub fn extract_topic(question: &str) -> Option<String> {
    let q_lower = question.to_lowercase();

    let topic = if q_lower.starts_with("what is ") {
        q_lower.strip_prefix("what is ")
    } else if q_lower.starts_with("what are ") {
        q_lower.strip_prefix("what are ")
    } else if q_lower.starts_with("how does ") {
        q_lower.strip_prefix("how does ")
    } else if q_lower.starts_with("how do ") {
        q_lower.strip_prefix("how do ")
    } else if q_lower.starts_with("explain ") {
        q_lower.strip_prefix("explain ")
    } else if q_lower.starts_with("teach me ") {
        q_lower.strip_prefix("teach me ")
    } else if q_lower.starts_with("tell me about ") {
        q_lower.strip_prefix("tell me about ")
    } else {
        None
    };

    topic.map(|t| {
        t.replace("?", "")
            .trim()
            .to_string()
    })
}

/// Format teaching prompt for LLM.
pub fn format_teaching_prompt(context: &TeachingContext, question: &str) -> String {
    let mut prompt = String::new();

    prompt.push_str("TEACHING MODE:\n\n");
    prompt.push_str(&format!("User Question: {}\n", question));
    prompt.push_str(&format!("User Expertise: {:?}\n", context.expertise_level));

    if context.already_taught {
        prompt.push_str(&format!(
            "Note: User was already taught about '{}' (mastery: {:.0}%, asked {} times)\n",
            context.topic,
            context.mastery_level * 100.0,
            context.reinforcement_count
        ));

        if context.mastery_level > 0.7 {
            prompt.push_str("Approach: Brief reminder or advanced details only.\n");
        } else {
            prompt.push_str("Approach: Reinforce previous teaching with new angle.\n");
        }
    } else {
        prompt.push_str("Note: First time teaching this topic.\n");
        prompt.push_str("Approach: Clear foundational explanation.\n");
    }

    prompt.push('\n');
    prompt.push_str("Teaching Guidelines:\n");

    match context.expertise_level {
        ExpertiseLevel::Beginner => {
            prompt.push_str("- Use simple language and avoid jargon\n");
            prompt.push_str("- Include step-by-step explanations\n");
            prompt.push_str("- Use analogies to familiar concepts\n");
        }
        ExpertiseLevel::Intermediate => {
            prompt.push_str("- Assume basic Linux knowledge\n");
            prompt.push_str("- Focus on how things work together\n");
            prompt.push_str("- Include practical examples\n");
        }
        ExpertiseLevel::Advanced => {
            prompt.push_str("- Skip basics, focus on details\n");
            prompt.push_str("- Explain edge cases and gotchas\n");
            prompt.push_str("- Reference internals when relevant\n");
        }
        ExpertiseLevel::Expert => {
            prompt.push_str("- Concise, technical explanations\n");
            prompt.push_str("- Focus on implementation details\n");
            prompt.push_str("- Skip common knowledge\n");
        }
    }

    if context.preferences.include_examples {
        prompt.push_str("- Include practical examples\n");
    }

    if context.preferences.use_analogies && context.expertise_level != ExpertiseLevel::Expert {
        prompt.push_str("- Use analogies where helpful\n");
    }

    if context.preferences.show_commands {
        prompt.push_str("- Show relevant commands (for learning)\n");
    }

    prompt.push_str(&format!("\nDepth: Level {} (1=brief, 5=comprehensive)\n", context.preferences.depth_level));

    prompt
}

/// Handle teaching question.
pub async fn handle_teaching_question(question: &str) -> Result<String> {
    info!("Teaching mode activated for: {}", question);

    let mut kb = KnowledgeBase::load();

    let topic = extract_topic(question).unwrap_or_else(|| question.to_string());
    let context = kb.get_teaching_context(&topic);

    // Generate teaching prompt
    let prompt = format_teaching_prompt(&context, question);

    // In real implementation, this would go to LLM
    // For now, return the prompt structure
    let response = format!(
        "Teaching Mode: {}\n\n{}\n\nThis would be sent to LLM for explanation generation.",
        topic, prompt
    );

    // Record that we taught this
    kb.record_teaching(&topic);
    kb.adjust_expertise();
    kb.save()?;

    Ok(response)
}

/// Get teaching statistics.
pub fn get_teaching_stats() -> TeachingStats {
    let kb = KnowledgeBase::load();

    let total_topics = kb.learned_topics.len();
    let mastered = kb.learned_topics.values()
        .filter(|t| t.mastery > 0.8)
        .count();
    let learning = kb.learned_topics.values()
        .filter(|t| t.mastery >= 0.4 && t.mastery <= 0.8)
        .count();
    let struggling = kb.learned_topics.values()
        .filter(|t| t.mastery < 0.4 && t.reinforcement_count > 2)
        .count();

    TeachingStats {
        total_topics_taught: total_topics,
        mastered_topics: mastered,
        learning_topics: learning,
        struggling_topics: struggling,
        expertise_level: kb.expertise_level,
    }
}

/// Teaching statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeachingStats {
    pub total_topics_taught: usize,
    pub mastered_topics: usize,
    pub learning_topics: usize,
    pub struggling_topics: usize,
    pub expertise_level: ExpertiseLevel,
}

/// Format teaching stats for display.
pub fn format_teaching_stats(stats: &TeachingStats) -> String {
    let mut response = String::new();

    response.push_str("Knowledge Base Summary:\n\n");
    response.push_str(&format!("Expertise Level: {:?}\n", stats.expertise_level));
    response.push_str(&format!("Total Topics Taught: {}\n", stats.total_topics_taught));
    response.push_str(&format!("  Mastered (>80%): {}\n", stats.mastered_topics));
    response.push_str(&format!("  Learning (40-80%): {}\n", stats.learning_topics));
    response.push_str(&format!("  Struggling (<40%, 3+ asks): {}\n", stats.struggling_topics));

    if stats.struggling_topics > 0 {
        response.push_str("\nConsider reviewing struggling topics for better understanding.\n");
    }

    response
}

/// Recommend topics to review.
pub fn recommend_review_topics() -> Vec<String> {
    let kb = KnowledgeBase::load();

    // Topics that need review:
    // 1. Low mastery with high reinforcement (struggling)
    // 2. High mastery but not referenced in >30 days (forgetting)
    let mut review_topics = Vec::new();

    for (_, topic) in kb.learned_topics.iter() {
        let struggling = topic.mastery < 0.4 && topic.reinforcement_count > 2;

        let days_since_reference = (Utc::now() - topic.last_referenced).num_days();
        let potentially_forgotten = topic.mastery > 0.7 && days_since_reference > 30;

        if struggling || potentially_forgotten {
            review_topics.push(topic.topic.clone());
        }
    }

    review_topics.sort();
    review_topics
}
