//! Memory system types - Anna remembers conversations and learns from experience.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// A single interaction Anna had with the user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    pub timestamp: String,
    pub user_query: String,
    pub anna_response: String,
    pub context: InteractionContext,
    pub outcome: Option<InteractionOutcome>,
}

/// Context about the interaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionContext {
    pub session_id: String,
    pub commands_executed: Vec<String>,
    pub services_affected: Vec<String>,
    pub files_modified: Vec<String>,
}

/// Outcome of the interaction (did it solve the problem?)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractionOutcome {
    Success { user_feedback: Option<String> },
    Failure { reason: String },
    Partial { what_worked: String, what_didnt: String },
    Unknown, // User didn't indicate success/failure
}

/// Anna's episodic memory - remembers past interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicMemory {
    pub interactions: VecDeque<Interaction>,
    pub max_size: usize,
}

impl Default for EpisodicMemory {
    fn default() -> Self {
        Self {
            interactions: VecDeque::new(),
            max_size: 1000, // Keep last 1000 interactions
        }
    }
}

impl EpisodicMemory {
    /// Add a new interaction
    pub fn record(&mut self, interaction: Interaction) {
        self.interactions.push_back(interaction);

        // Trim if exceeds max size
        while self.interactions.len() > self.max_size {
            self.interactions.pop_front();
        }
    }

    /// Find similar past interactions
    pub fn find_similar(&self, query: &str) -> Vec<&Interaction> {
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();

        let mut matches: Vec<(&Interaction, usize)> = self.interactions
            .iter()
            .filter_map(|interaction| {
                let interaction_text = format!("{} {}",
                    interaction.user_query.to_lowercase(),
                    interaction.anna_response.to_lowercase()
                );

                // Count matching words
                let match_count = query_words.iter()
                    .filter(|word| interaction_text.contains(*word))
                    .count();

                if match_count > query_words.len() / 2 {
                    Some((interaction, match_count))
                } else {
                    None
                }
            })
            .collect();

        // Sort by match count
        matches.sort_by(|a, b| b.1.cmp(&a.1));

        matches.into_iter()
            .take(5)
            .map(|(interaction, _)| interaction)
            .collect()
    }

    /// Get successful solutions for a problem
    pub fn get_successful_solutions(&self, problem_keywords: &[&str]) -> Vec<&Interaction> {
        self.interactions
            .iter()
            .filter(|interaction| {
                // Check if any keyword matches
                let has_keyword = problem_keywords.iter().any(|keyword| {
                    interaction.user_query.to_lowercase().contains(keyword)
                });

                // Check if outcome was success
                let was_successful = matches!(
                    interaction.outcome,
                    Some(InteractionOutcome::Success { .. })
                );

                has_keyword && was_successful
            })
            .take(10)
            .collect()
    }
}

/// Semantic memory - facts and knowledge Anna has learned
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticMemory {
    pub facts: Vec<LearnedFact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedFact {
    pub category: FactCategory,
    pub statement: String,
    pub confidence: f32, // 0.0 to 1.0
    pub learned_from: String, // Source (e.g., "user interaction", "system observation")
    pub learned_at: String,
    pub validated_count: u32, // How many times this has been confirmed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FactCategory {
    ServiceDependency,  // "Service X depends on Y"
    UserPreference,     // "User prefers minimal notifications"
    SystemCharacteristic, // "This system is a development workstation"
    TroubleshootingRule, // "When X fails, check Y first"
    TimingPattern,      // "User typically updates on weekends"
}

impl Default for SemanticMemory {
    fn default() -> Self {
        Self {
            facts: Vec::new(),
        }
    }
}

impl SemanticMemory {
    /// Add or update a fact
    pub fn learn(&mut self, fact: LearnedFact) {
        let fact_cat_disc = std::mem::discriminant(&fact.category);
        let fact_statement = fact.statement.to_lowercase();

        // Check if similar fact exists
        if let Some(existing) = self.facts.iter_mut().find(|f| {
            std::mem::discriminant(&f.category) == fact_cat_disc && f.statement.to_lowercase() == fact_statement
        }) {
            // Update confidence and validation count
            existing.validated_count += 1;
            existing.confidence = (existing.confidence + fact.confidence) / 2.0;
        } else {
            self.facts.push(fact);
        }

        // Keep only high-confidence facts if list gets too large
        if self.facts.len() > 500 {
            self.facts.retain(|f| f.confidence > 0.3);
        }
    }

    /// Get facts by category
    pub fn get_facts(&self, category: &FactCategory) -> Vec<&LearnedFact> {
        let cat_disc = std::mem::discriminant(category);
        self.facts
            .iter()
            .filter(|f| std::mem::discriminant(&f.category) == cat_disc)
            .filter(|f| f.confidence > 0.5) // Only high-confidence facts
            .collect()
    }
}
