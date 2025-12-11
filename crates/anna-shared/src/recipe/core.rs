//! Recipe struct and implementation (v0.0.419).
//! v0.0.295: Added negative_match_patterns for learning from "not helpful" feedback.
//! v0.0.419: Added citation_ids for provenance tracking.

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use crate::specialist_contract::KnowledgeCitation;
use crate::teams::Team;
use crate::ticket::RiskLevel;
use crate::trace::EvidenceKind;

use super::{
    ClarifyPrereq, RecipeAction, RecipeKind, RecipeSignature, RecipeSlot, RecipeTarget,
    RollbackInfo,
};

/// A learned recipe from a successful ticket resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub id: String,
    pub signature: RecipeSignature,
    pub team: Team,
    pub risk_level: RiskLevel,
    pub required_evidence_kinds: Vec<EvidenceKind>,
    pub probe_sequence: Vec<String>,
    #[serde(default)]
    pub answer_template: String,
    pub created_at: u64,
    #[serde(default)]
    pub success_count: u32,
    pub reliability_score: u8,
    #[serde(default)]
    pub kind: RecipeKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<RecipeTarget>,
    #[serde(default)]
    pub action: RecipeAction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rollback: Option<RollbackInfo>,
    // v0.0.31 clarification template fields
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clarification_slots: Vec<RecipeSlot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_question_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub populates_facts: Vec<String>,
    // v0.0.41: Deterministic retrieval keys for RAG-lite
    /// Intent tags for matching (e.g., ["enable", "syntax", "highlight"])
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub intent_tags: Vec<String>,
    /// Target identifiers for boosted matching (e.g., ["vim", "vimrc"])
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub targets: Vec<String>,
    /// Preconditions required (e.g., ["vim_installed"])
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preconditions: Vec<String>,
    /// v0.45.5: Clarification prerequisites - facts that must be known before execution
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clarify_prereqs: Vec<ClarifyPrereq>,
    /// v0.0.295: Query patterns that received "not helpful" feedback
    /// These patterns are excluded from future semantic matching
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub negative_match_patterns: Vec<String>,
    /// v0.0.419: Citations from knowledge sources that back this recipe
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub citations: Vec<KnowledgeCitation>,
}

/// Compute deterministic recipe ID from signature and team
pub fn compute_recipe_id(signature: &RecipeSignature, team: Team) -> String {
    let mut hasher = DefaultHasher::new();
    signature.hash(&mut hasher);
    team.to_string().hash(&mut hasher);
    let hash = hasher.finish();
    format!("{:016x}", hash)
}

impl Recipe {
    /// Create a new read-only query recipe from verified ticket
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
            kind: RecipeKind::Query,
            target: None,
            action: RecipeAction::None,
            rollback: None,
            clarification_slots: Vec::new(),
            default_question_id: None,
            populates_facts: Vec::new(),
            intent_tags: Vec::new(),
            targets: Vec::new(),
            preconditions: Vec::new(),
            clarify_prereqs: Vec::new(),
            negative_match_patterns: Vec::new(),
            citations: Vec::new(),
        }
    }

    /// Create a config edit recipe (v0.0.27)
    pub fn config_edit(
        signature: RecipeSignature,
        team: Team,
        target: RecipeTarget,
        action: RecipeAction,
        reliability_score: u8,
    ) -> Self {
        let id = compute_recipe_id(&signature, team);
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let kind = match &action {
            RecipeAction::EnsureLine { .. } => RecipeKind::ConfigEnsureLine,
            RecipeAction::AppendLine { .. } => RecipeKind::ConfigEditLineAppend,
            RecipeAction::None => RecipeKind::Query,
        };

        Self {
            id,
            signature,
            team,
            risk_level: RiskLevel::LowRiskChange,
            required_evidence_kinds: vec![],
            probe_sequence: vec![],
            answer_template: String::new(),
            created_at,
            success_count: 1,
            reliability_score,
            kind,
            target: Some(target),
            action,
            rollback: None,
            clarification_slots: Vec::new(),
            default_question_id: None,
            populates_facts: Vec::new(),
            intent_tags: Vec::new(),
            targets: Vec::new(),
            preconditions: Vec::new(),
            clarify_prereqs: Vec::new(),
            negative_match_patterns: Vec::new(),
            citations: Vec::new(),
        }
    }

    /// Create a clarification template recipe (v0.0.31)
    pub fn clarification_template(
        signature: RecipeSignature,
        team: Team,
        slots: Vec<RecipeSlot>,
        default_question: Option<String>,
        populates: Vec<String>,
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
            risk_level: RiskLevel::ReadOnly,
            required_evidence_kinds: vec![],
            probe_sequence: vec![],
            answer_template: String::new(),
            created_at,
            success_count: 1,
            reliability_score,
            kind: RecipeKind::ClarificationTemplate,
            target: None,
            action: RecipeAction::None,
            rollback: None,
            clarification_slots: slots,
            default_question_id: default_question,
            populates_facts: populates,
            intent_tags: Vec::new(),
            targets: Vec::new(),
            preconditions: Vec::new(),
            clarify_prereqs: Vec::new(),
            negative_match_patterns: Vec::new(),
            citations: Vec::new(),
        }
    }

    /// Set rollback info
    pub fn with_rollback(mut self, rollback: RollbackInfo) -> Self {
        self.rollback = Some(rollback);
        self
    }

    /// Set intent tags for RAG-lite retrieval (v0.0.41)
    pub fn with_intent_tags(mut self, tags: Vec<String>) -> Self {
        self.intent_tags = tags;
        self
    }

    /// Set targets for boosted matching (v0.0.41)
    pub fn with_targets(mut self, targets: Vec<String>) -> Self {
        self.targets = targets;
        self
    }

    /// Set preconditions (v0.0.41)
    pub fn with_preconditions(mut self, preconditions: Vec<String>) -> Self {
        self.preconditions = preconditions;
        self
    }

    /// Set clarification prerequisites (v0.45.5)
    pub fn with_clarify_prereqs(mut self, prereqs: Vec<ClarifyPrereq>) -> Self {
        self.clarify_prereqs = prereqs;
        self
    }

    /// v0.0.419: Set citations from knowledge sources
    pub fn with_citations(mut self, citations: Vec<KnowledgeCitation>) -> Self {
        self.citations = citations;
        self
    }

    /// v0.0.419: Add a citation
    pub fn add_citation(&mut self, citation: KnowledgeCitation) {
        if !self.citations.iter().any(|c| c.citation_id == citation.citation_id) {
            self.citations.push(citation);
        }
    }

    /// Check if recipe requires clarification before execution (v0.45.5)
    pub fn needs_clarification(&self) -> bool {
        !self.clarify_prereqs.is_empty()
    }

    /// Get clarification prerequisites
    pub fn get_clarify_prereqs(&self) -> &[ClarifyPrereq] {
        &self.clarify_prereqs
    }

    /// Increment success count
    pub fn record_success(&mut self) {
        self.success_count = self.success_count.saturating_add(1);
    }

    /// Check if recipe is mature (used successfully multiple times)
    pub fn is_mature(&self) -> bool {
        self.success_count >= 3
    }

    /// Check if this is a config edit recipe
    pub fn is_config_edit(&self) -> bool {
        matches!(
            self.kind,
            RecipeKind::ConfigEnsureLine | RecipeKind::ConfigEditLineAppend
        )
    }

    /// Check if this is a clarification template recipe (v0.0.31)
    pub fn is_clarification_template(&self) -> bool {
        matches!(self.kind, RecipeKind::ClarificationTemplate)
    }

    /// Get clarification slots if this is a template
    pub fn get_clarification_slots(&self) -> &[RecipeSlot] {
        &self.clarification_slots
    }

    /// Get filesystem path for this recipe
    pub fn file_path(&self) -> PathBuf {
        super::recipe_dir().join(format!("{}.json", self.id))
    }

    /// Save recipe to disk
    pub fn save(&self) -> std::io::Result<()> {
        let dir = super::recipe_dir();
        std::fs::create_dir_all(&dir)?;
        let path = self.file_path();
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, json)
    }

    /// Load recipe from disk
    pub fn load(recipe_id: &str) -> std::io::Result<Self> {
        let path = super::recipe_filename(recipe_id);
        let json = std::fs::read_to_string(path)?;
        serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// v0.0.295: Check if a query is in the negative match list
    /// Returns true if this query pattern was previously marked "not helpful"
    pub fn is_negative_match(&self, query: &str) -> bool {
        let q_lower = query.to_lowercase();
        self.negative_match_patterns
            .iter()
            .any(|p| p.to_lowercase() == q_lower)
    }

    /// v0.0.295: Add a query pattern to negative matches
    /// Called when user gives "not helpful" feedback on a semantic match
    pub fn add_negative_match(&mut self, query: &str) {
        let q_lower = query.to_lowercase();
        if !self.negative_match_patterns.iter().any(|p| p.to_lowercase() == q_lower) {
            self.negative_match_patterns.push(query.to_string());
        }
    }
}
