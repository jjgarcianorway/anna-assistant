//! Type definitions for learned recipes.

use crate::canonical_intents::CanonicalIntent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A learned recipe - deterministic, replayable solution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedRecipe {
    /// Stable ID
    pub id: String,
    /// Name
    pub name: String,
    /// Version (increments on update)
    pub version: u32,
    /// Canonical intent this recipe handles
    pub intent: CanonicalIntent,
    /// Domain (storage, services, etc.)
    pub domain: String,
    /// Required probes (must all succeed)
    pub required_probes: Vec<String>,
    /// Optional probes (nice to have)
    pub optional_probes: Vec<String>,
    /// Computation steps
    pub steps: Vec<RecipeComputeStep>,
    /// Answer template (ok case)
    pub answer_ok: AnswerTemplate,
    /// Answer template (critical/warning case)
    pub answer_critical: Option<AnswerTemplate>,
    /// Answer template (partial case)
    pub answer_partial: Option<AnswerTemplate>,
    /// Knowledge topics for enrichment
    pub knowledge_topics: Vec<String>,
    /// Tickets that contributed to this recipe
    pub source_tickets: Vec<String>,
    /// Stats
    pub stats: RecipeStats,
    /// Created timestamp
    pub created_at: u64,
    /// Last used timestamp
    pub last_used_at: u64,
    /// Deprecated flag
    pub deprecated: bool,
}

/// Computation step - deterministic logic
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum RecipeComputeStep {
    /// Extract a value from probe output
    Extract {
        probe: String,
        pattern: String,
        variable: String,
    },
    /// Compare a value against threshold
    Compare {
        variable: String,
        operator: CompareOp,
        threshold: f64,
        result_var: String,
    },
    /// Count items matching pattern
    Count {
        probe: String,
        pattern: String,
        variable: String,
    },
    /// Check if probe output is empty
    IsEmpty { probe: String, variable: String },
    /// Parse a numeric value
    ParseNumber {
        source_var: String,
        target_var: String,
    },
}

/// Comparison operator
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompareOp {
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
}

impl CompareOp {
    pub fn eval(&self, a: f64, b: f64) -> bool {
        match self {
            Self::Lt => a < b,
            Self::Le => a <= b,
            Self::Gt => a > b,
            Self::Ge => a >= b,
            Self::Eq => (a - b).abs() < f64::EPSILON,
            Self::Ne => (a - b).abs() >= f64::EPSILON,
        }
    }
}

/// Answer template with placeholders
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerTemplate {
    /// Summary with {placeholders}
    pub summary: String,
    /// Detail lines with {placeholders}
    pub details: Vec<String>,
    /// Evidence probes to cite
    pub evidence: Vec<String>,
}

impl AnswerTemplate {
    /// Render template with values
    pub fn render(&self, values: &HashMap<String, String>) -> RenderedAnswer {
        RenderedAnswer {
            summary: substitute(&self.summary, values),
            details: self.details.iter().map(|d| substitute(d, values)).collect(),
            evidence: self.evidence.clone(),
        }
    }
}

/// Rendered answer
#[derive(Debug, Clone)]
pub struct RenderedAnswer {
    pub summary: String,
    pub details: Vec<String>,
    pub evidence: Vec<String>,
}

/// Substitute {placeholders} in template
fn substitute(template: &str, values: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in values {
        result = result.replace(&format!("{{{}}}", key), value);
    }
    result
}

/// Recipe statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecipeStats {
    /// Total uses
    pub uses: u32,
    /// Successful uses
    pub successes: u32,
    /// Failed uses
    pub failures: u32,
    /// Average confidence
    pub avg_confidence: f32,
}

impl RecipeStats {
    pub fn success_rate(&self) -> f32 {
        if self.uses == 0 {
            1.0
        } else {
            self.successes as f32 / self.uses as f32
        }
    }

    pub fn record_success(&mut self, confidence: f32) {
        self.uses += 1;
        self.successes += 1;
        self.avg_confidence =
            (self.avg_confidence * (self.uses - 1) as f32 + confidence) / self.uses as f32;
    }

    pub fn record_failure(&mut self) {
        self.uses += 1;
        self.failures += 1;
    }
}

/// Recipe store summary stats
#[derive(Debug, Clone)]
pub struct RecipeStoreSummary {
    pub total: usize,
    pub active: usize,
    pub deprecated: usize,
    pub total_uses: u32,
    pub success_rate: f32,
}

/// Recipe execution context
#[derive(Debug, Clone)]
pub struct RecipeContext {
    /// Probe outputs
    pub probe_outputs: HashMap<String, String>,
    /// Computed variables
    pub variables: HashMap<String, String>,
}

impl RecipeContext {
    pub fn new() -> Self {
        Self {
            probe_outputs: HashMap::new(),
            variables: HashMap::new(),
        }
    }

    pub fn with_probes(probes: HashMap<String, String>) -> Self {
        Self {
            probe_outputs: probes,
            variables: HashMap::new(),
        }
    }
}

impl Default for RecipeContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Recipe execution result
#[derive(Debug, Clone)]
pub enum RecipeResult {
    /// Recipe succeeded with answer
    Success {
        answer: RenderedAnswer,
        confidence: f32,
    },
    /// Recipe partially succeeded
    Partial {
        answer: RenderedAnswer,
        confidence: f32,
        missing: Vec<String>,
    },
    /// Recipe failed (fall back to specialist)
    Failed { reason: String },
}
