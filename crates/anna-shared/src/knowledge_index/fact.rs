//! Learned facts - simple key-value knowledge.

use serde::{Deserialize, Serialize};

use super::utils::current_millis;

/// A learned fact (simple key-value)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedFact {
    /// Fact key (e.g., "swap_enabled", "gpu_vendor")
    pub key: String,
    /// Fact value
    pub value: String,
    /// Domain this fact belongs to
    pub domain: String,
    /// Confidence (0-100)
    pub confidence: u8,
    /// Last verified timestamp
    pub last_verified: u64,
    /// How many times this fact was confirmed
    pub confirmations: u32,
}

impl LearnedFact {
    pub fn new(key: &str, value: &str, domain: &str) -> Self {
        Self {
            key: key.to_string(),
            value: value.to_string(),
            domain: domain.to_string(),
            confidence: 70,
            last_verified: current_millis(),
            confirmations: 1,
        }
    }

    /// Boost confidence on reconfirmation
    pub fn confirm(&mut self, new_value: &str) {
        if self.value == new_value {
            self.confirmations += 1;
            self.confidence = (self.confidence + 10).min(100);
        } else {
            // Value changed - lower confidence
            self.value = new_value.to_string();
            self.confidence = 60;
        }
        self.last_verified = current_millis();
    }

    /// Check if fact is stale (older than 1 hour)
    pub fn is_stale(&self) -> bool {
        current_millis().saturating_sub(self.last_verified) > 3600_000
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_learned_fact() {
        let mut fact = LearnedFact::new("swap_enabled", "true", "system");
        assert_eq!(fact.confirmations, 1);

        fact.confirm("true");
        assert_eq!(fact.confirmations, 2);
        assert!(fact.confidence > 70);
    }
}
