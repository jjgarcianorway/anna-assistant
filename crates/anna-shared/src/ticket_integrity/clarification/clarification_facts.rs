//! Known Facts Store - v0.0.442.
//!
//! User-provided facts and fact management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Known facts store (user-provided facts).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnownFacts {
    /// Facts by name.
    facts: HashMap<String, KnownFact>,
}

/// A known fact from user or previous clarification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnownFact {
    /// Fact name.
    pub name: String,
    /// Fact value.
    pub value: String,
    /// Source of fact.
    pub source: FactSource,
    /// Confidence (1.0 for user-provided).
    pub confidence: f64,
}

/// Source of a known fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactSource {
    /// User explicitly provided.
    User,
    /// Inferred from probe.
    Probe,
    /// Default assumption.
    Default,
}

impl KnownFacts {
    /// Create empty facts store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a fact.
    pub fn add(&mut self, name: &str, value: &str, source: FactSource) {
        let confidence = match source {
            FactSource::User => 1.0,
            FactSource::Probe => 0.9,
            FactSource::Default => 0.5,
        };
        self.facts.insert(
            name.to_string(),
            KnownFact {
                name: name.to_string(),
                value: value.to_string(),
                source,
                confidence,
            },
        );
    }

    /// Get a fact.
    pub fn get(&self, name: &str) -> Option<&KnownFact> {
        self.facts.get(name)
    }

    /// Check if fact is known.
    pub fn has(&self, name: &str) -> bool {
        self.facts.contains_key(name)
    }

    /// Check which facts are missing from a list.
    pub fn missing(&self, required: &[&str]) -> Vec<String> {
        required
            .iter()
            .filter(|f| !self.has(f))
            .map(|f| f.to_string())
            .collect()
    }

    /// Check if all required facts are known.
    pub fn has_all(&self, required: &[&str]) -> bool {
        required.iter().all(|f| self.has(f))
    }
}
