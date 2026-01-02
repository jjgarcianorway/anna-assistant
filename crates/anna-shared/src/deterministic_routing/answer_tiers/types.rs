//! Answer tier types and core structures.

use super::super::intent_schema::CanonicalIntent;

/// Answer tier levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AnswerTier {
    /// Tier 1: Raw facts from probes.
    Facts,
    /// Tier 2: Identified key items (top offenders, main issues).
    KeyItems,
    /// Tier 3: Specialist synthesis (interpretation, recommendations).
    Synthesis,
}

impl AnswerTier {
    /// Label for display.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Facts => "facts",
            Self::KeyItems => "key_items",
            Self::Synthesis => "synthesis",
        }
    }
}

/// Tiered answer for a specific intent.
#[derive(Debug, Clone)]
pub struct TieredAnswer {
    /// The intent being answered.
    pub intent: CanonicalIntent,
    /// Tier 1: Facts from probes.
    pub facts: Option<String>,
    /// Tier 2: Key items identified.
    pub key_items: Option<Vec<String>>,
    /// Tier 3: Synthesis (if specialist was called).
    pub synthesis: Option<String>,
    /// Current tier achieved.
    pub current_tier: AnswerTier,
}

impl TieredAnswer {
    /// Create a new tiered answer.
    pub fn new(intent: CanonicalIntent) -> Self {
        Self {
            intent,
            facts: None,
            key_items: None,
            synthesis: None,
            current_tier: AnswerTier::Facts,
        }
    }

    /// Set tier 1 facts.
    pub fn with_facts(mut self, facts: &str) -> Self {
        self.facts = Some(facts.to_string());
        self.current_tier = AnswerTier::Facts;
        self
    }

    /// Set tier 2 key items.
    pub fn with_key_items(mut self, items: Vec<String>) -> Self {
        self.key_items = Some(items);
        self.current_tier = AnswerTier::KeyItems;
        self
    }

    /// Set tier 3 synthesis.
    pub fn with_synthesis(mut self, synthesis: &str) -> Self {
        self.synthesis = Some(synthesis.to_string());
        self.current_tier = AnswerTier::Synthesis;
        self
    }

    /// Build the final answer string.
    pub fn build(&self) -> String {
        let mut parts = Vec::new();

        if let Some(facts) = &self.facts {
            parts.push(facts.clone());
        }

        if let Some(items) = &self.key_items {
            if !items.is_empty() {
                parts.push(items.join("\n"));
            }
        }

        if let Some(synthesis) = &self.synthesis {
            parts.push(synthesis.clone());
        }

        parts.join("\n\n")
    }

    /// Check if we have enough for a complete answer.
    pub fn is_complete(&self) -> bool {
        // For most intents, facts alone are sufficient
        self.facts.is_some()
    }
}
