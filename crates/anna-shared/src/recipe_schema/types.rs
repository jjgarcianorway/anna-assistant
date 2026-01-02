//! Core type definitions for recipe schema.
//!
//! This module contains the primary data structures that define recipes,
//! including Recipe, RecipePattern, RecipeMatcher, and related types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{Precondition, PlanStep};

/// A learned recipe that can be executed without LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    /// Unique identifier (e.g., "vim_enable_syntax")
    pub id: String,
    /// Recipe version (incremented on updates)
    pub version: u32,
    /// Domain: desktop, storage, network, etc.
    pub domain: String,
    /// Canonical intent this recipe serves
    pub intent: String,
    /// Pattern describing what this recipe handles
    pub pattern: RecipePattern,
    /// Matcher configuration for runtime lookup
    pub matcher: RecipeMatcher,
    /// Preconditions that must be true before execution
    pub preconditions: Vec<Precondition>,
    /// Plan steps to execute
    pub plan: Vec<PlanStep>,
    /// Whether to require user confirmation
    pub confirmation_policy: ConfirmationPolicy,
    /// Success criteria and rollback behavior
    pub success_criteria: SuccessCriteria,
    /// Documentation citations (Arch Wiki, man pages)
    pub citations: Vec<String>,
    /// Usage metrics
    pub metrics: RecipeMetrics,
    /// Recipe status
    pub status: RecipeStatus,
    /// When this recipe was created
    pub created_at: DateTime<Utc>,
    /// When this recipe was last updated
    pub updated_at: DateTime<Utc>,
}

/// Pattern describing what user goal this recipe handles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipePattern {
    /// Human-readable description of user goal
    pub user_goal: String,
    /// Extracted slots/parameters (e.g., editor="vim", feature="syntax")
    #[serde(default)]
    pub slots: HashMap<String, String>,
}

/// Matcher configuration for finding applicable recipes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeMatcher {
    /// Keywords that MUST be present
    pub required_keywords: Vec<String>,
    /// Keywords that boost score if present
    #[serde(default)]
    pub optional_keywords: Vec<String>,
    /// Keywords that disqualify this recipe
    #[serde(default)]
    pub negative_keywords: Vec<String>,
    /// Minimum confidence to use this recipe (0.0-1.0)
    pub min_confidence: f32,
    /// Intent must match exactly (if set)
    #[serde(default)]
    pub exact_intent: Option<String>,
}

/// Confirmation policy for recipe execution.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConfirmationPolicy {
    /// Always require user confirmation
    Require,
    /// Ask for confirmation for mutating steps only
    MutatingOnly,
    /// Never ask (dangerous, only for safe read-only recipes)
    Never,
}

impl Default for ConfirmationPolicy {
    fn default() -> Self {
        Self::Require
    }
}

/// Success criteria for recipe execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessCriteria {
    /// Step types that must succeed
    #[serde(default)]
    pub must_succeed: Vec<String>,
    /// Whether to rollback on any failure
    #[serde(default = "default_true")]
    pub rollback_on_failure: bool,
    /// Optional verification command to run after plan
    #[serde(default)]
    pub post_verification: Option<String>,
}

fn default_true() -> bool {
    true
}

impl Default for SuccessCriteria {
    fn default() -> Self {
        Self {
            must_succeed: vec![],
            rollback_on_failure: true,
            post_verification: None,
        }
    }
}

/// Usage metrics for a recipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeMetrics {
    /// Times this recipe was used
    pub times_used: u32,
    /// Times this recipe failed
    pub times_failed: u32,
    /// Last time this recipe was used
    #[serde(default)]
    pub last_used_at: Option<DateTime<Utc>>,
    /// Average user rating (if collected)
    #[serde(default)]
    pub avg_user_rating: Option<f32>,
    /// Recent success rate (last N uses)
    #[serde(default)]
    pub recent_success_rate: Option<f32>,
}

impl Default for RecipeMetrics {
    fn default() -> Self {
        Self {
            times_used: 0,
            times_failed: 0,
            last_used_at: None,
            avg_user_rating: None,
            recent_success_rate: None,
        }
    }
}

/// Recipe status.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecipeStatus {
    /// Recipe is active and can be used
    Active,
    /// Recipe needs review (e.g., deprecated commands)
    NeedsReview,
    /// Recipe is disabled due to failures
    Disabled,
    /// Recipe is deprecated (superseded by another)
    Deprecated,
}

impl Default for RecipeStatus {
    fn default() -> Self {
        Self::Active
    }
}
