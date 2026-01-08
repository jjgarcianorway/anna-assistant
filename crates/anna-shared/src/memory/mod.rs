//! Learning Memory System - Anna learns from every interaction.
//!
//! This is NOT a hardcoded recipe system. Instead:
//! - Every successful Q&A is stored with semantic embeddings
//! - Similar questions retrieve relevant past experiences
//! - Patterns emerge organically from successful interactions
//! - Anna learns what commands work for what types of questions
//!
//! The system learns:
//! - Question patterns → effective commands
//! - System context → relevant approaches
//! - Error patterns → successful fixes

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::config::anna_data_dir;

/// A learned experience from a past interaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experience {
    /// Unique ID
    pub id: String,

    /// The question asked (normalized)
    pub question: String,

    /// Keywords extracted from the question
    pub keywords: Vec<String>,

    /// Commands that successfully answered this question
    pub successful_commands: Vec<String>,

    /// The answer that was generated
    pub answer: String,

    /// System context at the time (relevant profile fields)
    pub context: ExperienceContext,

    /// How many times this experience has been useful
    pub usefulness_score: u32,

    /// When this experience was created
    pub created_at: String,

    /// When this experience was last used
    pub last_used: Option<String>,

    /// Embedding vector for semantic search (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
}

/// Context captured with an experience
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExperienceContext {
    /// Was this about a specific package?
    pub package: Option<String>,

    /// Was this about a specific service?
    pub service: Option<String>,

    /// Was this about a specific file/path?
    pub path: Option<String>,

    /// What topic category does this fall into?
    pub topic: Option<String>,

    /// System-specific context (e.g., "wayland", "nvidia", "btrfs")
    pub system_tags: Vec<String>,
}

/// The memory store
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Memory {
    /// All learned experiences
    pub experiences: Vec<Experience>,

    /// Learned patterns: keyword -> common commands
    pub patterns: Vec<LearnedPattern>,

    /// Statistics
    pub stats: MemoryStats,
}

/// A pattern learned from multiple experiences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedPattern {
    /// Keywords that trigger this pattern
    pub keywords: Vec<String>,

    /// Commands that commonly work for these keywords
    pub common_commands: Vec<CommandPattern>,

    /// How many experiences support this pattern
    pub evidence_count: u32,
}

/// A command pattern with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPattern {
    /// The command template (may include placeholders like {package})
    pub command: String,

    /// How often this command succeeded
    pub success_count: u32,

    /// What type of information this command retrieves
    pub retrieves: Option<String>,
}

/// Memory statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Total experiences stored
    pub total_experiences: u32,

    /// Total patterns learned
    pub total_patterns: u32,

    /// Questions answered from memory (without full LLM)
    pub memory_hits: u32,

    /// Questions that needed full LLM processing
    pub memory_misses: u32,
}

impl Memory {
    /// Load memory from disk
    pub fn load() -> Result<Self> {
        let path = memory_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let memory: Memory = serde_json::from_str(&content)?;
            Ok(memory)
        } else {
            Ok(Memory::default())
        }
    }

    /// Save memory to disk
    pub fn save(&self) -> Result<()> {
        let path = memory_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Learn from a successful interaction
    pub fn learn(&mut self, question: &str, commands: Vec<String>, answer: &str, context: ExperienceContext) {
        let keywords = extract_keywords(question);

        // Create new experience
        let experience = Experience {
            id: uuid::Uuid::new_v4().to_string(),
            question: question.to_lowercase(),
            keywords: keywords.clone(),
            successful_commands: commands.clone(),
            answer: answer.to_string(),
            context,
            usefulness_score: 1,
            created_at: chrono::Utc::now().to_rfc3339(),
            last_used: None,
            embedding: None,
        };

        self.experiences.push(experience);
        self.stats.total_experiences += 1;

        // Update patterns
        self.update_patterns(&keywords, &commands);
    }

    /// Update patterns based on new experience
    fn update_patterns(&mut self, keywords: &[String], commands: &[String]) {
        // Find or create pattern for these keywords
        for keyword in keywords {
            if let Some(pattern) = self.patterns.iter_mut().find(|p| p.keywords.contains(keyword)) {
                // Update existing pattern
                for cmd in commands {
                    if let Some(cp) = pattern.common_commands.iter_mut().find(|c| &c.command == cmd) {
                        cp.success_count += 1;
                    } else {
                        pattern.common_commands.push(CommandPattern {
                            command: cmd.clone(),
                            success_count: 1,
                            retrieves: None,
                        });
                    }
                }
                pattern.evidence_count += 1;
            } else if !keyword.is_empty() && keyword.len() > 2 {
                // Create new pattern
                let pattern = LearnedPattern {
                    keywords: vec![keyword.clone()],
                    common_commands: commands
                        .iter()
                        .map(|c| CommandPattern {
                            command: c.clone(),
                            success_count: 1,
                            retrieves: None,
                        })
                        .collect(),
                    evidence_count: 1,
                };
                self.patterns.push(pattern);
                self.stats.total_patterns += 1;
            }
        }
    }

    /// Find relevant experiences for a question
    pub fn recall(&self, question: &str, limit: usize) -> Vec<&Experience> {
        let keywords = extract_keywords(question);
        let question_lower = question.to_lowercase();

        // Score experiences by relevance
        let mut scored: Vec<(&Experience, f32)> = self
            .experiences
            .iter()
            .filter_map(|exp| {
                let score = calculate_relevance(exp, &question_lower, &keywords);
                if score > 0.2 {
                    Some((exp, score))
                } else {
                    None
                }
            })
            .collect();

        // Sort by score (highest first) then by usefulness
        scored.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.0.usefulness_score.cmp(&a.0.usefulness_score))
        });

        scored.into_iter().take(limit).map(|(e, _)| e).collect()
    }

    /// Get suggested commands based on learned patterns
    pub fn suggest_commands(&self, question: &str) -> Vec<String> {
        let keywords = extract_keywords(question);
        let mut suggestions: Vec<(String, u32)> = Vec::new();

        for keyword in &keywords {
            for pattern in &self.patterns {
                if pattern.keywords.iter().any(|k| k == keyword || keyword.contains(k)) {
                    for cmd in &pattern.common_commands {
                        if let Some((_, count)) = suggestions.iter_mut().find(|(c, _)| c == &cmd.command) {
                            *count += cmd.success_count;
                        } else {
                            suggestions.push((cmd.command.clone(), cmd.success_count));
                        }
                    }
                }
            }
        }

        // Sort by success count
        suggestions.sort_by(|a, b| b.1.cmp(&a.1));
        suggestions.into_iter().map(|(c, _)| c).collect()
    }

    /// Mark an experience as useful (was retrieved and helped)
    pub fn mark_useful(&mut self, experience_id: &str) {
        if let Some(exp) = self.experiences.iter_mut().find(|e| e.id == experience_id) {
            exp.usefulness_score += 1;
            exp.last_used = Some(chrono::Utc::now().to_rfc3339());
            self.stats.memory_hits += 1;
        }
    }

    /// Record a memory miss (had to use full LLM)
    pub fn record_miss(&mut self) {
        self.stats.memory_misses += 1;
    }

    /// Get memory statistics
    pub fn get_stats(&self) -> &MemoryStats {
        &self.stats
    }

    /// Compact memory by removing low-value experiences
    pub fn compact(&mut self, max_experiences: usize) {
        if self.experiences.len() <= max_experiences {
            return;
        }

        // Sort by usefulness and recency
        self.experiences.sort_by(|a, b| {
            // Prefer higher usefulness
            let usefulness_cmp = b.usefulness_score.cmp(&a.usefulness_score);
            if usefulness_cmp != std::cmp::Ordering::Equal {
                return usefulness_cmp;
            }
            // Then prefer more recent
            b.created_at.cmp(&a.created_at)
        });

        // Keep only the most valuable
        self.experiences.truncate(max_experiences);
        self.stats.total_experiences = self.experiences.len() as u32;
    }
}

/// Extract keywords from a question
fn extract_keywords(question: &str) -> Vec<String> {
    let stop_words = [
        "a", "an", "the", "is", "are", "was", "were", "be", "been", "being",
        "have", "has", "had", "do", "does", "did", "will", "would", "could",
        "should", "may", "might", "must", "shall", "can", "need", "dare",
        "ought", "used", "to", "of", "in", "for", "on", "with", "at", "by",
        "from", "as", "into", "through", "during", "before", "after", "above",
        "below", "between", "under", "again", "further", "then", "once", "here",
        "there", "when", "where", "why", "how", "all", "each", "every", "both",
        "few", "more", "most", "other", "some", "such", "no", "nor", "not",
        "only", "own", "same", "so", "than", "too", "very", "just", "and",
        "but", "if", "or", "because", "until", "while", "what", "which", "who",
        "whom", "this", "that", "these", "those", "am", "i", "my", "me", "you",
        "your", "it", "its", "he", "she", "they", "we", "them", "his", "her",
        "their", "our", "much", "many", "any", "about", "get", "tell", "show",
    ];

    question
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
        .filter(|w| w.len() > 2 && !stop_words.contains(w))
        .map(String::from)
        .collect()
}

/// Calculate relevance score between experience and question
fn calculate_relevance(experience: &Experience, question: &str, keywords: &[String]) -> f32 {
    let mut score = 0.0;

    // Exact substring match
    if experience.question.contains(question) || question.contains(&experience.question) {
        score += 0.5;
    }

    // Keyword overlap
    let keyword_matches = keywords
        .iter()
        .filter(|k| experience.keywords.contains(k))
        .count();

    if !keywords.is_empty() {
        score += (keyword_matches as f32) / (keywords.len() as f32) * 0.4;
    }

    // Boost by usefulness
    score += (experience.usefulness_score as f32).min(10.0) / 100.0;

    score
}

/// Get memory storage path
pub fn memory_path() -> PathBuf {
    anna_data_dir().join("memory.json")
}
