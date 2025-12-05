//! Team-tagged recipes for learning from successful patterns.
//!
//! Recipes capture successful ticket resolutions for future reference.
//! Only persisted when: Ticket status = Verified, reliability score >= 80.
//!
//! Storage: ~/.anna/recipes/{recipe_id}.json

use crate::teams::Team;
use crate::ticket::RiskLevel;
use crate::trace::EvidenceKind;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

/// Signature that uniquely identifies a query pattern.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecipeSignature {
    /// Domain (e.g., "system", "network", "storage")
    pub domain: String,
    /// Intent (e.g., "question", "investigate", "request")
    pub intent: String,
    /// Route class from classifier
    pub route_class: String,
    /// Normalized query pattern (lowercase, trimmed)
    pub query_pattern: String,
}

impl RecipeSignature {
    /// Create a new recipe signature
    pub fn new(
        domain: impl Into<String>,
        intent: impl Into<String>,
        route_class: impl Into<String>,
        query: &str,
    ) -> Self {
        Self {
            domain: domain.into(),
            intent: intent.into(),
            route_class: route_class.into(),
            query_pattern: normalize_query(query),
        }
    }

    /// Compute a deterministic hash of this signature
    pub fn hash_id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

/// Normalize query for pattern matching
fn normalize_query(query: &str) -> String {
    query.to_lowercase().trim().to_string()
}

/// A learned recipe from a successful ticket resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    /// Unique recipe ID (hash of signature + team)
    pub id: String,
    /// Query signature this recipe applies to
    pub signature: RecipeSignature,
    /// Team that handled this successfully
    pub team: Team,
    /// Risk level of the original request
    pub risk_level: RiskLevel,
    /// Evidence kinds required for this pattern
    pub required_evidence_kinds: Vec<EvidenceKind>,
    /// Probe sequence that worked
    pub probe_sequence: Vec<String>,
    /// Answer template (with placeholders for evidence)
    #[serde(default)]
    pub answer_template: String,
    /// Unix timestamp when created
    pub created_at: u64,
    /// Number of successful uses
    #[serde(default)]
    pub success_count: u32,
    /// Reliability score when created
    pub reliability_score: u8,
}

impl Recipe {
    /// Create a new recipe from verified ticket
    pub fn new(
        signature: RecipeSignature,
        team: Team,
        risk_level: RiskLevel,
        required_evidence_kinds: Vec<EvidenceKind>,
        probe_sequence: Vec<String>,
        answer_template: String,
        reliability_score: u8,
    ) -> Self {
        let id = compute_recipe_id(&signature, team);
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            id,
            signature,
            team,
            risk_level,
            required_evidence_kinds,
            probe_sequence,
            answer_template,
            created_at,
            success_count: 1,
            reliability_score,
        }
    }

    /// Increment success count
    pub fn record_success(&mut self) {
        self.success_count = self.success_count.saturating_add(1);
    }

    /// Check if recipe is mature (used successfully multiple times)
    pub fn is_mature(&self) -> bool {
        self.success_count >= 3
    }

    /// Get filesystem path for this recipe
    pub fn file_path(&self) -> PathBuf {
        recipe_dir().join(format!("{}.json", self.id))
    }
}

/// Compute deterministic recipe ID from signature and team
pub fn compute_recipe_id(signature: &RecipeSignature, team: Team) -> String {
    let mut hasher = DefaultHasher::new();
    signature.hash(&mut hasher);
    team.to_string().hash(&mut hasher);
    let hash = hasher.finish();
    format!("{:016x}", hash)
}

/// Get the recipes directory path
pub fn recipe_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".anna")
        .join("recipes")
}

/// Get path for a specific recipe file
pub fn recipe_filename(recipe_id: &str) -> PathBuf {
    recipe_dir().join(format!("{}.json", recipe_id))
}

/// Check if a recipe should be persisted
/// Only persist when: Verified status AND reliability >= 80
pub fn should_persist_recipe(verified: bool, score: u8) -> bool {
    verified && score >= 80
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_signature_creation() {
        let sig = RecipeSignature::new("system", "question", "MemoryUsage", "How much RAM?");

        assert_eq!(sig.domain, "system");
        assert_eq!(sig.query_pattern, "how much ram?");
    }

    #[test]
    fn test_recipe_signature_hash_deterministic() {
        let sig1 = RecipeSignature::new("system", "question", "MemoryUsage", "How much RAM?");
        let sig2 = RecipeSignature::new("system", "question", "MemoryUsage", "How much RAM?");

        assert_eq!(sig1.hash_id(), sig2.hash_id());
    }

    #[test]
    fn test_recipe_id_deterministic() {
        let sig = RecipeSignature::new("storage", "investigate", "DiskUsage", "check disk");

        let id1 = compute_recipe_id(&sig, Team::Storage);
        let id2 = compute_recipe_id(&sig, Team::Storage);

        assert_eq!(id1, id2);
        assert_eq!(id1.len(), 16); // 16 hex chars
    }

    #[test]
    fn test_recipe_id_differs_by_team() {
        let sig = RecipeSignature::new("storage", "investigate", "DiskUsage", "check disk");

        let id_storage = compute_recipe_id(&sig, Team::Storage);
        let id_general = compute_recipe_id(&sig, Team::General);

        assert_ne!(id_storage, id_general);
    }

    #[test]
    fn test_recipe_creation() {
        let sig = RecipeSignature::new("network", "question", "NetworkInfo", "show ip");
        let recipe = Recipe::new(
            sig,
            Team::Network,
            RiskLevel::ReadOnly,
            vec![],
            vec!["ip addr show".to_string()],
            "Your IP is {ip}".to_string(),
            85,
        );

        assert_eq!(recipe.team, Team::Network);
        assert_eq!(recipe.success_count, 1);
        assert_eq!(recipe.reliability_score, 85);
        assert!(!recipe.is_mature());
    }

    #[test]
    fn test_recipe_maturity() {
        let sig = RecipeSignature::new("system", "question", "CpuInfo", "cpu info");
        let mut recipe = Recipe::new(
            sig,
            Team::Hardware,
            RiskLevel::ReadOnly,
            vec![EvidenceKind::Cpu],
            vec!["lscpu".to_string()],
            String::new(),
            90,
        );

        assert!(!recipe.is_mature());

        recipe.record_success();
        assert!(!recipe.is_mature());

        recipe.record_success();
        assert!(recipe.is_mature()); // 3 successes
    }

    #[test]
    fn test_should_persist_recipe() {
        assert!(should_persist_recipe(true, 80));
        assert!(should_persist_recipe(true, 100));
        assert!(!should_persist_recipe(true, 79));
        assert!(!should_persist_recipe(false, 100));
        assert!(!should_persist_recipe(false, 50));
    }

    #[test]
    fn test_recipe_serialization() {
        let sig = RecipeSignature::new("storage", "question", "DiskUsage", "disk space");
        let recipe = Recipe::new(
            sig,
            Team::Storage,
            RiskLevel::ReadOnly,
            vec![EvidenceKind::Disk, EvidenceKind::BlockDevices],
            vec!["df -h".to_string(), "lsblk".to_string()],
            "Disk usage: {usage}".to_string(),
            88,
        );

        let json = serde_json::to_string(&recipe).unwrap();
        let parsed: Recipe = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.id, recipe.id);
        assert_eq!(parsed.team, Team::Storage);
        assert_eq!(parsed.probe_sequence.len(), 2);
    }

    #[test]
    fn test_recipe_filename() {
        let path = recipe_filename("abc123def456");
        assert!(path.to_string_lossy().contains("abc123def456.json"));
        assert!(path.to_string_lossy().contains("recipes"));
    }
}
