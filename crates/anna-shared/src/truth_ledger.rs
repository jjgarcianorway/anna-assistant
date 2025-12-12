use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Claim {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Source {
    Url(Url),
    File(String),
    User(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Veracity {
    Verified,
    Disputed,
    Unverified,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TrustScore {
    Unknown,
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SourceMetadata {
    pub source: Source,
    pub trust_score: TrustScore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub claim: Claim,
    pub source_metadata: SourceMetadata,
    pub veracity: Veracity,
    pub timestamp: DateTime<Utc>,
    pub confidence_score: f32,
    pub feedback: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TruthLedger {
    pub entries: Vec<LedgerEntry>,
}

impl TruthLedger {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add_claim(
        &mut self,
        claim: Claim,
        source: Source,
        source_trust_score: TrustScore,
        confidence_score: f32,
        initial_veracity: Option<Veracity>,
    ) {
        let entry = LedgerEntry {
            claim,
            source_metadata: SourceMetadata {
                source,
                trust_score: source_trust_score,
            },
            veracity: initial_veracity.unwrap_or(Veracity::Unverified),
            timestamp: Utc::now(),
            confidence_score,
            feedback: None,
        };
        self.entries.push(entry);
    }

    pub fn find_claim(&self, claim_text: &str) -> Option<&LedgerEntry> {
        self.entries
            .iter()
            .find(|entry| entry.claim.text == claim_text)
    }

    pub fn verify_claim(&mut self, claim_text: &str) -> bool {
        if let Some(entry) = self
            .entries
            .iter_mut()
            .find(|entry| entry.claim.text == claim_text)
        {
            entry.veracity = Veracity::Verified;
            true
        } else {
            false
        }
    }

    pub fn dispute_claim(&mut self, claim_text: &str) -> bool {
        if let Some(entry) = self
            .entries
            .iter_mut()
            .find(|entry| entry.claim.text == claim_text)
        {
            entry.veracity = Veracity::Disputed;
            true
        } else {
            false
        }
    }

    pub fn add_feedback(&mut self, claim_text: &str, positive_feedback: bool) -> bool {
        if let Some(entry) = self
            .entries
            .iter_mut()
            .find(|entry| entry.claim.text == claim_text)
        {
            entry.feedback = Some(positive_feedback);
            true
        } else {
            false
        }
    }

    /// Get all claims in the ledger.
    pub fn get_all_claims(&self) -> Vec<&LedgerEntry> {
        self.entries.iter().collect()
    }

    pub fn check_truthfulness(&self, claim_text: &str) -> Option<(Veracity, bool)> {
        if let Some(entry) = self
            .entries
            .iter()
            .find(|entry| entry.claim.text == claim_text)
        {
            let mut effective_confidence = entry.confidence_score;
            let mut effective_trust_score = entry.source_metadata.trust_score.clone();
            let mut web_search_recommended = false;

            // Adjust confidence based on direct user feedback
            if let Some(feedback) = entry.feedback {
                if feedback {
                    effective_confidence = (effective_confidence + 0.2).min(1.0); // Boost confidence
                    if effective_trust_score == TrustScore::Unknown {
                        effective_trust_score = TrustScore::Medium; // Promote unknown to medium
                    }
                } else {
                    effective_confidence = (effective_confidence - 0.3).max(0.0); // Penalize confidence
                    if effective_trust_score != TrustScore::Low {
                        effective_trust_score = TrustScore::Low; // Demote to low unless already low
                    }
                }
            }

            // Adjust confidence based on recency (e.g., within last 24 hours gets a slight boost)
            let twenty_four_hours_ago = Utc::now() - chrono::Duration::hours(24);
            if entry.timestamp > twenty_four_hours_ago {
                effective_confidence = (effective_confidence + 0.1).min(1.0); // Small boost for recency
            }

            let current_veracity =
                if effective_confidence > 0.8 && effective_trust_score == TrustScore::High {
                    Veracity::Verified
                } else if effective_confidence < 0.2 || effective_trust_score == TrustScore::Low {
                    Veracity::Disputed
                } else if entry.feedback == Some(true) {
                    Veracity::Verified
                } else if entry.feedback == Some(false) {
                    Veracity::Disputed
                } else {
                    entry.veracity.clone()
                };

            // If still unverified and no strong signals, recommend web search
            if current_veracity == Veracity::Unverified
                && effective_confidence < 0.7
                && effective_trust_score == TrustScore::Unknown
            {
                web_search_recommended = true;
            }

            Some((current_veracity, web_search_recommended))
        } else {
            // Claim not found, potentially recommend web search if it's a new claim to verify
            Some((Veracity::Unverified, true))
        }
    }

    pub fn save(&self, path: &str) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(self).unwrap();
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn load(path: &str) -> Result<Self, std::io::Error> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let ledger: Self = serde_json::from_str(&contents).unwrap();
        Ok(ledger)
    }
}
