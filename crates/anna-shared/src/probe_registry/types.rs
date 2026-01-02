//! Probe type definitions.

use crate::evidence_engine::{EvidenceDomain, EvidenceIntent};
use serde::{Deserialize, Serialize};

/// Cost of running a probe
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProbeCost {
    /// Fast, no disk/network (< 100ms)
    Cheap,
    /// May take a moment (< 1s)
    Medium,
    /// Slow or resource-intensive (> 1s)
    Expensive,
}

/// A probe definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeDef {
    /// Unique identifier (e.g., "probe:df_root")
    pub id: String,
    /// Shell command to run
    pub command: String,
    /// Human description
    pub description: String,
    /// Applicable domains
    pub domains: Vec<EvidenceDomain>,
    /// Matching tags
    pub tags: Vec<String>,
    /// Cost classification
    pub cost: ProbeCost,
    /// Required intents (empty = any)
    pub intents: Vec<EvidenceIntent>,
    /// Output parser hint
    pub parse_hint: Option<String>,
}

impl ProbeDef {
    /// Check if this probe matches a request
    pub fn matches(&self, domain: EvidenceDomain, intent: EvidenceIntent, tags: &[String]) -> bool {
        // Domain must match (or be related)
        let domain_match = self.domains.contains(&domain);

        // Intent must match if specified
        let intent_match = self.intents.is_empty() || self.intents.contains(&intent);

        // At least one tag must match
        let tag_match = tags.iter().any(|t| {
            let t_lower = t.to_lowercase();
            self.tags.iter().any(|pt| pt.to_lowercase() == t_lower)
        });

        domain_match && intent_match && tag_match
    }

    /// Score relevance (higher = more relevant)
    pub fn relevance_score(&self, tags: &[String]) -> u32 {
        let mut score = 0u32;
        for tag in tags {
            let t_lower = tag.to_lowercase();
            if self.tags.iter().any(|pt| pt.to_lowercase() == t_lower) {
                score += 10;
            }
        }
        // Cheaper probes get slight boost
        match self.cost {
            ProbeCost::Cheap => score += 3,
            ProbeCost::Medium => score += 1,
            ProbeCost::Expensive => {}
        }
        score
    }
}
