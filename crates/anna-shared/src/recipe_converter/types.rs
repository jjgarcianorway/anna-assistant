//! Types for recipe conversion.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Minimum confidence for recipe creation
pub const MIN_CONFIDENCE: u8 = 80;
/// Maximum steps for a recipe (keep simple)
pub const MAX_STEPS: usize = 5;

/// Recipe candidate proposed by specialist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialistRecipeCandidate {
    /// Human-friendly name
    pub name: String,
    /// Domain (services, storage, etc.)
    pub domain: String,
    /// Intent pattern description
    pub intent_pattern: String,
    /// Tags for matching
    pub tags: Vec<String>,
    /// Evidence requirements
    pub required_evidence: Vec<String>,
    /// Steps to execute
    pub steps: Vec<SpecialistStepCandidate>,
    /// Documentation sources
    pub doc_sources: Vec<String>,
    /// Recipe IDs this supersedes (for updates)
    #[serde(default)]
    pub supersedes_recipe_ids: Vec<String>,
}

/// Step candidate from specialist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialistStepCandidate {
    /// Step type
    pub kind: String,
    /// Description
    pub description: String,
    /// Parameters (command, probe_id, template, etc.)
    pub params: HashMap<String, String>,
}

/// Result of validation
#[derive(Debug)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}
