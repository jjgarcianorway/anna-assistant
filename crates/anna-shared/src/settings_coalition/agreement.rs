// v0.0.736: Settings Coalition - Agreement (Phase 312)
// Coalition agreement

use serde::{Deserialize, Serialize};

/// Coalition agreement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoalitionAgreement {
    /// Agreement ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Priority
    pub priority: u32,
    /// Consensus
    pub consensus: bool,
}

impl CoalitionAgreement {
    /// Create new agreement
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            priority: 0,
            consensus: false,
        }
    }

    /// Set priority
    pub fn priority(mut self, p: u32) -> Self {
        self.priority = p;
        self
    }

    /// Reach consensus
    pub fn reach_consensus(&mut self) {
        self.consensus = true;
    }

    /// Break consensus
    pub fn break_consensus(&mut self) {
        self.consensus = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agreement_new() {
        let a = CoalitionAgreement::new("a1", "Title", "Content");
        assert_eq!(a.id, "a1");
    }

    #[test]
    fn test_agreement_builder() {
        let a = CoalitionAgreement::new("a1", "Title", "Content")
            .priority(1);
        assert_eq!(a.priority, 1);
    }

    #[test]
    fn test_agreement_consensus() {
        let mut a = CoalitionAgreement::new("a1", "Title", "Content");
        a.reach_consensus();
        assert!(a.consensus);
        a.break_consensus();
        assert!(!a.consensus);
    }
}
