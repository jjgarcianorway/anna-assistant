//! Belief Decay - Knowledge decays with system drift.
//!
//! Every belief carries a decay function. Confidence decreases with:
//! - Time since observation
//! - System changes (kernel updates, package changes, config churn)
//! - Lack of reinforcement

use serde::{Deserialize, Serialize};

/// A belief with confidence that decays over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecayingBelief {
    /// The belief content
    pub content: String,
    /// Initial confidence when established
    pub initial_confidence: f32,
    /// Current confidence (may be stale)
    pub current_confidence: f32,
    /// When the belief was established
    pub established_at: String,
    /// Last time it was reinforced (confirmed true)
    pub last_reinforced: Option<String>,
    /// Evidence count supporting this belief
    pub evidence_count: u32,
    /// Decay rate per day (0.0-1.0)
    pub decay_rate: f32,
    /// System events that would invalidate this
    pub invalidators: Vec<String>,
    /// Whether this belief has been explicitly falsified
    pub falsified: bool,
    /// Falsification evidence
    pub falsification: Option<String>,
}

impl DecayingBelief {
    /// Create a new belief
    pub fn new(content: &str, confidence: f32) -> Self {
        Self {
            content: content.to_string(),
            initial_confidence: confidence,
            current_confidence: confidence,
            established_at: chrono::Utc::now().to_rfc3339(),
            last_reinforced: None,
            evidence_count: 1,
            decay_rate: 0.05, // 5% decay per day by default
            invalidators: Vec::new(),
            falsified: false,
            falsification: None,
        }
    }

    /// Get current confidence after applying decay
    pub fn confidence(&self) -> f32 {
        if self.falsified {
            return 0.0;
        }

        let days_since = self.days_since_last_activity();
        let decayed = calculate_decay(self.current_confidence, self.decay_rate, days_since);

        // Boost for evidence
        let evidence_boost = (self.evidence_count as f32 * 0.05).min(0.3);

        (decayed + evidence_boost).min(1.0)
    }

    /// Days since last reinforcement or establishment
    fn days_since_last_activity(&self) -> f32 {
        let reference = self
            .last_reinforced
            .as_ref()
            .unwrap_or(&self.established_at);

        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(reference) {
            let now = chrono::Utc::now();
            let duration = now - dt.with_timezone(&chrono::Utc);
            duration.num_hours() as f32 / 24.0
        } else {
            0.0
        }
    }

    /// Reinforce the belief (it was confirmed true)
    pub fn reinforce(&mut self) {
        self.last_reinforced = Some(chrono::Utc::now().to_rfc3339());
        self.evidence_count += 1;
        self.current_confidence = (self.current_confidence + 0.1).min(1.0);
    }

    /// Falsify the belief
    pub fn falsify(&mut self, evidence: &str) {
        self.falsified = true;
        self.falsification = Some(evidence.to_string());
        self.current_confidence = 0.0;
    }

    /// Check if belief should be considered stale
    pub fn is_stale(&self) -> bool {
        self.confidence() < 0.3
    }

    /// Add an invalidator
    pub fn add_invalidator(&mut self, event: &str) {
        if !self.invalidators.contains(&event.to_string()) {
            self.invalidators.push(event.to_string());
        }
    }

    /// Check if a system event invalidates this belief
    pub fn is_invalidated_by(&self, event: &str) -> bool {
        self.invalidators.iter().any(|i| event.contains(i) || i.contains(event))
    }
}

/// Belief strength categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BeliefStrength {
    /// Very confident (>0.8)
    Strong,
    /// Moderately confident (0.5-0.8)
    Moderate,
    /// Low confidence (0.3-0.5)
    Weak,
    /// Should not be relied upon (<0.3)
    Stale,
    /// Explicitly disproven
    Falsified,
}

impl BeliefStrength {
    pub fn from_confidence(confidence: f32, falsified: bool) -> Self {
        if falsified {
            BeliefStrength::Falsified
        } else if confidence > 0.8 {
            BeliefStrength::Strong
        } else if confidence > 0.5 {
            BeliefStrength::Moderate
        } else if confidence > 0.3 {
            BeliefStrength::Weak
        } else {
            BeliefStrength::Stale
        }
    }
}

/// Calculate exponential decay
pub fn calculate_decay(initial: f32, rate: f32, days: f32) -> f32 {
    initial * (-rate * days).exp()
}

/// Suggested decay rates for different types of knowledge
pub mod decay_rates {
    /// Hardware info (rarely changes)
    pub const HARDWARE: f32 = 0.01;
    /// Installed packages (change occasionally)
    pub const PACKAGES: f32 = 0.03;
    /// Service states (can change frequently)
    pub const SERVICES: f32 = 0.1;
    /// Network state (highly volatile)
    pub const NETWORK: f32 = 0.2;
    /// Process state (very volatile)
    pub const PROCESSES: f32 = 0.5;
    /// File contents (depends on file)
    pub const FILES: f32 = 0.05;
    /// User preferences (stable)
    pub const PREFERENCES: f32 = 0.01;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decay_calculation() {
        // After 0 days, no decay
        assert!((calculate_decay(1.0, 0.1, 0.0) - 1.0).abs() < 0.01);

        // After some time, should decay
        let decayed = calculate_decay(1.0, 0.1, 10.0);
        assert!(decayed < 1.0);
        assert!(decayed > 0.0);
    }

    #[test]
    fn test_belief_reinforcement() {
        let mut belief = DecayingBelief::new("test", 0.5);
        assert_eq!(belief.evidence_count, 1);

        belief.reinforce();
        assert_eq!(belief.evidence_count, 2);
        assert!(belief.current_confidence > 0.5);
    }

    #[test]
    fn test_belief_falsification() {
        let mut belief = DecayingBelief::new("test", 0.9);
        assert!(belief.confidence() > 0.5);

        belief.falsify("Proven wrong by experiment");
        assert_eq!(belief.confidence(), 0.0);
        assert!(belief.falsified);
    }

    #[test]
    fn test_belief_strength() {
        assert_eq!(
            BeliefStrength::from_confidence(0.9, false),
            BeliefStrength::Strong
        );
        assert_eq!(
            BeliefStrength::from_confidence(0.6, false),
            BeliefStrength::Moderate
        );
        assert_eq!(
            BeliefStrength::from_confidence(0.2, false),
            BeliefStrength::Stale
        );
        assert_eq!(
            BeliefStrength::from_confidence(0.9, true),
            BeliefStrength::Falsified
        );
    }
}
