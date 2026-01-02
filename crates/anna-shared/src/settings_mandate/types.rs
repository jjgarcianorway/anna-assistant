// v0.0.721: Settings Mandate Types (Phase 297)
// Core types for mandate system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Mandate type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MandateType {
    /// Legal mandate
    #[default]
    Legal,
    /// Regulatory mandate
    Regulatory,
    /// Corporate mandate
    Corporate,
    /// Technical mandate
    Technical,
}

impl std::fmt::Display for MandateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Legal => write!(f, "legal"),
            Self::Regulatory => write!(f, "regulatory"),
            Self::Corporate => write!(f, "corporate"),
            Self::Technical => write!(f, "technical"),
        }
    }
}

/// Mandate compliance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MandateCompliance {
    /// Required compliance
    #[default]
    Required,
    /// Recommended compliance
    Recommended,
    /// Optional compliance
    Optional,
    /// Exempt compliance
    Exempt,
}

impl std::fmt::Display for MandateCompliance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Required => write!(f, "required"),
            Self::Recommended => write!(f, "recommended"),
            Self::Optional => write!(f, "optional"),
            Self::Exempt => write!(f, "exempt"),
        }
    }
}

/// Mandate config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MandateConfig {
    /// Name
    pub name: String,
    /// Mandate type
    pub mandate_type: MandateType,
    /// Default compliance
    pub default_compliance: MandateCompliance,
    /// Max mandates
    pub max_mandates: usize,
}

impl MandateConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            mandate_type: MandateType::Legal,
            default_compliance: MandateCompliance::Required,
            max_mandates: 100,
        }
    }

    /// Set type
    pub fn mandate_type(mut self, mt: MandateType) -> Self {
        self.mandate_type = mt;
        self
    }

    /// Set default compliance
    pub fn default_compliance(mut self, dc: MandateCompliance) -> Self {
        self.default_compliance = dc;
        self
    }

    /// Set max mandates
    pub fn max_mandates(mut self, max: usize) -> Self {
        self.max_mandates = max;
        self
    }
}

impl Default for MandateConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Mandate requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MandateRequirement {
    /// Requirement ID
    pub id: String,
    /// Title
    pub title: String,
    /// Description
    pub description: String,
    /// Compliance
    pub compliance: MandateCompliance,
    /// Fulfilled
    pub fulfilled: bool,
}

impl MandateRequirement {
    /// Create new requirement
    pub fn new(id: impl Into<String>, title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: description.into(),
            compliance: MandateCompliance::Required,
            fulfilled: false,
        }
    }

    /// Set compliance
    pub fn compliance(mut self, c: MandateCompliance) -> Self {
        self.compliance = c;
        self
    }

    /// Fulfill requirement
    pub fn fulfill(&mut self) {
        self.fulfilled = true;
    }

    /// Unfulfill requirement
    pub fn unfulfill(&mut self) {
        self.fulfilled = false;
    }
}

/// Mandate evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MandateEvidence {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Requirement ID
    pub requirement_id: String,
}

impl MandateEvidence {
    /// Create new evidence
    pub fn new(key: impl Into<String>, value: impl Into<String>, requirement_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            requirement_id: requirement_id.into(),
        }
    }
}

/// Mandate stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MandateStats {
    /// Total mandates
    pub total_mandates: usize,
    /// Fulfilled mandates
    pub fulfilled: usize,
    /// Required count
    pub required_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl MandateStats {
    /// Update from requirements
    pub fn update(&mut self, requirements: &[MandateRequirement], mandate_type: MandateType) {
        self.total_mandates = requirements.len();
        self.fulfilled = requirements.iter().filter(|r| r.fulfilled).count();
        self.required_count = requirements.iter().filter(|r| r.compliance == MandateCompliance::Required).count();
        *self.by_type.entry(mandate_type.to_string()).or_insert(0) += 1;
    }

    /// Fulfillment rate
    pub fn fulfillment_rate(&self) -> f64 {
        if self.total_mandates == 0 { 0.0 } else { self.fulfilled as f64 / self.total_mandates as f64 * 100.0 }
    }
}
