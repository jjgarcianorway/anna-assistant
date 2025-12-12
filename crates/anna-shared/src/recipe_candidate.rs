//! Recipe candidate storage for grounded learning (v0.0.408).
//!
//! Stores proposed recipes from successful ticket resolutions.
//! Candidates must be grounded in evidence (probes + docs).
//! Promotion to full recipe requires multiple confirmations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// Minimum confirmations before auto-promotion
const PROMOTION_THRESHOLD: u32 = 3;

/// A candidate recipe learned from a successful ticket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeCandidate {
    /// Unique ID (hash of pattern)
    pub id: String,
    /// Ticket ID that created this candidate
    pub from_ticket_id: String,
    /// Creation timestamp (Unix millis)
    pub created_at: u64,
    /// Pattern keywords extracted from query
    pub pattern_keywords: Vec<String>,
    /// Domain (services, packages, etc.)
    pub domain: String,
    /// Intent (diagnose, configure, etc.)
    pub intent: String,
    /// Probes that were used
    pub required_probes: Vec<String>,
    /// Knowledge tags used (for doc search)
    pub required_docs: Vec<String>,
    /// Actions taken (grounded in evidence)
    pub actions: Vec<CandidateAction>,
    /// Evidence item IDs used
    pub evidence_used: Vec<String>,
    /// How many similar tickets confirmed this pattern
    pub confirmations: u32,
    /// Last confirmation timestamp
    pub last_confirmed_at: u64,
}

/// An action in a candidate recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateAction {
    /// Action type
    pub action_type: ActionType,
    /// Command to run (if applicable)
    pub command: Option<String>,
    /// Evidence ID backing this action
    pub evidence_id: String,
    /// Short description
    pub description: String,
}

/// Type of action
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    /// Run a probe/command for diagnosis
    Probe,
    /// Check a configuration file
    ConfigCheck,
    /// Suggest user action
    Suggest,
    /// Explain something
    Explain,
}

impl RecipeCandidate {
    /// Create a new candidate from ticket data
    pub fn new(ticket_id: &str, domain: &str, intent: &str, pattern_keywords: Vec<String>) -> Self {
        let id = compute_candidate_id(domain, intent, &pattern_keywords);
        let now = current_millis();

        Self {
            id,
            from_ticket_id: ticket_id.to_string(),
            created_at: now,
            pattern_keywords,
            domain: domain.to_string(),
            intent: intent.to_string(),
            required_probes: vec![],
            required_docs: vec![],
            actions: vec![],
            evidence_used: vec![],
            confirmations: 1,
            last_confirmed_at: now,
        }
    }

    /// Add required probes
    pub fn with_probes(mut self, probes: Vec<String>) -> Self {
        self.required_probes = probes;
        self
    }

    /// Add required doc tags
    pub fn with_docs(mut self, docs: Vec<String>) -> Self {
        self.required_docs = docs;
        self
    }

    /// Add actions
    pub fn with_actions(mut self, actions: Vec<CandidateAction>) -> Self {
        self.actions = actions;
        self
    }

    /// Add evidence IDs
    pub fn with_evidence(mut self, evidence: Vec<String>) -> Self {
        self.evidence_used = evidence;
        self
    }

    /// Increment confirmation count
    pub fn confirm(&mut self) {
        self.confirmations += 1;
        self.last_confirmed_at = current_millis();
    }

    /// Check if ready for promotion
    pub fn ready_for_promotion(&self) -> bool {
        self.confirmations >= PROMOTION_THRESHOLD
    }

    /// Check if this candidate matches a query pattern
    pub fn matches(&self, domain: &str, intent: &str, keywords: &[String]) -> bool {
        if self.domain != domain || self.intent != intent {
            return false;
        }
        // Check keyword overlap
        let overlap = keywords
            .iter()
            .filter(|k| self.pattern_keywords.contains(k))
            .count();
        overlap >= 1 && overlap >= keywords.len() / 2
    }
}

/// Persistent store for recipe candidates
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecipeCandidateStore {
    pub candidates: HashMap<String, RecipeCandidate>,
}

impl RecipeCandidateStore {
    /// Load from disk or create empty
    pub fn load() -> Self {
        let path = store_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(json) => match serde_json::from_str(&json) {
                    Ok(store) => {
                        debug!("Loaded {} recipe candidates", store_count(&store));
                        return store;
                    }
                    Err(e) => warn!("Failed to parse recipe candidates: {}", e),
                },
                Err(e) => warn!("Failed to read recipe candidates: {}", e),
            }
        }
        Self::default()
    }

    /// Save to disk
    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = store_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(&path, json)?;
        debug!("Saved {} recipe candidates", self.candidates.len());
        Ok(())
    }

    /// Add a new candidate or confirm existing
    pub fn add_or_confirm(&mut self, candidate: RecipeCandidate) {
        if let Some(existing) = self.candidates.get_mut(&candidate.id) {
            existing.confirm();
            info!(
                "Confirmed recipe candidate {} (now {} confirmations)",
                candidate.id, existing.confirmations
            );
        } else {
            info!(
                "New recipe candidate: {} ({})",
                candidate.id,
                candidate.pattern_keywords.join(", ")
            );
            self.candidates.insert(candidate.id.clone(), candidate);
        }
    }

    /// Find similar candidates
    pub fn find_similar(
        &self,
        domain: &str,
        intent: &str,
        keywords: &[String],
    ) -> Vec<&RecipeCandidate> {
        self.candidates
            .values()
            .filter(|c| c.matches(domain, intent, keywords))
            .collect()
    }

    /// Get candidates ready for promotion
    pub fn promotable(&self) -> Vec<&RecipeCandidate> {
        self.candidates
            .values()
            .filter(|c| c.ready_for_promotion())
            .collect()
    }

    /// Remove a candidate (after promotion or rejection)
    pub fn remove(&mut self, id: &str) {
        self.candidates.remove(id);
    }

    /// Count candidates
    pub fn len(&self) -> usize {
        self.candidates.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.candidates.is_empty()
    }
}

/// Check if a ticket should generate a candidate
pub fn should_create_candidate(
    state: &str,
    reliability: u8,
    has_evidence: bool,
    used_llm: bool,
) -> bool {
    // Only from successful LLM-handled tickets with evidence
    state == "success" && reliability >= 70 && has_evidence && used_llm
}

/// Extract pattern keywords from a query
pub fn extract_pattern_keywords(query: &str) -> Vec<String> {
    let query_lower = query.to_lowercase();
    let mut keywords = vec![];

    // Remove common stop words
    let stop_words = [
        "the", "a", "an", "is", "are", "was", "were", "be", "been", "what", "why", "how", "when",
        "where", "which", "who", "my", "your", "i", "me", "you", "it", "this", "that", "do",
        "does", "did", "can", "could", "would", "should", "to", "of", "in", "on", "at", "for",
        "with", "by",
    ];

    for word in query_lower.split_whitespace() {
        let clean: String = word.chars().filter(|c| c.is_alphanumeric()).collect();

        if clean.len() >= 3 && !stop_words.contains(&clean.as_str()) {
            keywords.push(clean);
        }
    }

    // Deduplicate
    keywords.sort();
    keywords.dedup();
    keywords
}

/// Compute deterministic candidate ID
fn compute_candidate_id(domain: &str, intent: &str, keywords: &[String]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    domain.hash(&mut hasher);
    intent.hash(&mut hasher);
    for kw in keywords {
        kw.hash(&mut hasher);
    }
    format!("cand_{:016x}", hasher.finish())
}

/// Get store file path
fn store_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".anna")
        .join("recipe_candidates.json")
}

fn current_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn store_count(store: &RecipeCandidateStore) -> usize {
    store.candidates.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_candidate_creation() {
        let candidate = RecipeCandidate::new(
            "TEST-001",
            "services",
            "diagnose",
            vec!["sshd".to_string(), "failed".to_string()],
        );

        assert!(candidate.id.starts_with("cand_"));
        assert_eq!(candidate.confirmations, 1);
    }

    #[test]
    fn test_candidate_matching() {
        let candidate = RecipeCandidate::new(
            "TEST-001",
            "services",
            "diagnose",
            vec!["sshd".to_string(), "failed".to_string()],
        );

        // Same domain/intent/keywords should match
        assert!(candidate.matches("services", "diagnose", &["sshd".to_string()]));

        // Different domain should not match
        assert!(!candidate.matches("packages", "diagnose", &["sshd".to_string()]));
    }

    #[test]
    fn test_extract_pattern_keywords() {
        let keywords = extract_pattern_keywords("Why is my sshd service failing to start?");

        assert!(keywords.contains(&"sshd".to_string()));
        assert!(keywords.contains(&"service".to_string()));
        assert!(keywords.contains(&"failing".to_string()));
        // Stop words should be filtered
        assert!(!keywords.contains(&"why".to_string()));
        assert!(!keywords.contains(&"is".to_string()));
    }

    #[test]
    fn test_confirmation() {
        let mut candidate =
            RecipeCandidate::new("TEST-001", "services", "diagnose", vec!["test".to_string()]);

        assert_eq!(candidate.confirmations, 1);
        assert!(!candidate.ready_for_promotion());

        candidate.confirm();
        candidate.confirm();

        assert_eq!(candidate.confirmations, 3);
        assert!(candidate.ready_for_promotion());
    }

    #[test]
    fn test_should_create_candidate() {
        // Should create: success, high reliability, has evidence, used LLM
        assert!(should_create_candidate("success", 85, true, true));

        // Should not create: failed
        assert!(!should_create_candidate("failed", 85, true, true));

        // Should not create: low reliability
        assert!(!should_create_candidate("success", 50, true, true));

        // Should not create: no evidence
        assert!(!should_create_candidate("success", 85, false, true));

        // Should not create: didn't use LLM (already a recipe)
        assert!(!should_create_candidate("success", 85, true, false));
    }
}
