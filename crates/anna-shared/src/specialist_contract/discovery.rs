//! Discovery types for proposing new probes and recipes.

use serde::{Deserialize, Serialize};

/// Discovery: how specialists propose new probes and recipes
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Discovery {
    #[serde(default)]
    pub new_probes: Vec<ProbeProposal>,
    #[serde(default)]
    pub new_recipes: Vec<RecipeProposal>,
}

/// A proposed new probe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeProposal {
    pub id: String,
    pub intent: String,
    pub domain: String,
    pub command: String,
    #[serde(default)]
    pub parse_hint: Option<String>,
    #[serde(default)]
    pub reusable_for: Vec<String>,
}

/// A proposed new recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeProposal {
    pub id: String,
    pub intent: String,
    pub domain: String,
    pub summary: String,
    #[serde(default)]
    pub prerequisites: Vec<String>,
    #[serde(default = "default_risk")]
    pub risk_level: RiskLevel,
    #[serde(default)]
    pub steps_high_level: Vec<String>,
    #[serde(default)]
    pub reusable_for: Vec<String>,
}

fn default_risk() -> RiskLevel {
    RiskLevel::Low
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    #[default]
    Low,
    Medium,
    High,
}
