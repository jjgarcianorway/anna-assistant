// v0.0.721: Settings Mandate (Phase 297)
// Authoritative mandates for settings compliance

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

/// Settings mandate
#[derive(Debug, Clone, Default)]
pub struct SettingsMandate {
    /// Config
    config: MandateConfig,
    /// Requirements
    requirements: Vec<MandateRequirement>,
    /// Evidence
    evidence: Vec<MandateEvidence>,
    /// Stats
    stats: MandateStats,
}

impl SettingsMandate {
    /// Create new mandate system
    pub fn new(config: MandateConfig) -> Self {
        Self {
            config,
            requirements: Vec::new(),
            evidence: Vec::new(),
            stats: MandateStats::default(),
        }
    }

    /// Add requirement
    pub fn add_requirement(&mut self, requirement: MandateRequirement) -> bool {
        if self.requirements.len() >= self.config.max_mandates {
            return false;
        }
        self.requirements.push(requirement);
        self.update_stats();
        true
    }

    /// Get requirement
    pub fn get_requirement(&self, id: &str) -> Option<&MandateRequirement> {
        self.requirements.iter().find(|r| r.id == id)
    }

    /// Get requirement mut
    pub fn get_requirement_mut(&mut self, id: &str) -> Option<&mut MandateRequirement> {
        self.requirements.iter_mut().find(|r| r.id == id)
    }

    /// Add evidence
    pub fn add_evidence(&mut self, ev: MandateEvidence) {
        self.evidence.push(ev);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.requirements, self.config.mandate_type);
    }

    /// Get stats
    pub fn stats(&self) -> &MandateStats {
        &self.stats
    }

    /// Requirement count
    pub fn requirement_count(&self) -> usize {
        self.requirements.len()
    }
}

/// Mandate registry
#[derive(Debug, Clone, Default)]
pub struct MandateRegistry {
    /// Mandates by ID
    mandates: HashMap<String, SettingsMandate>,
}

impl MandateRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register mandate
    pub fn register(&mut self, id: impl Into<String>, mandate: SettingsMandate) {
        self.mandates.insert(id.into(), mandate);
    }

    /// Unregister mandate
    pub fn unregister(&mut self, id: &str) -> bool {
        self.mandates.remove(id).is_some()
    }

    /// Get mandate
    pub fn get(&self, id: &str) -> Option<&SettingsMandate> {
        self.mandates.get(id)
    }

    /// Get mandate mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsMandate> {
        self.mandates.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.mandates.len()
    }
}

/// Format mandate registry
pub fn format_mandate_registry(registry: &MandateRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Mandate Registry:\n");
    output.push_str(&format!("  Mandates: {}\n", registry.count()));
    output
}

/// Check if query is about mandate
pub fn is_mandate_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings mandate") || lower.contains("mandate settings") || lower.contains("compliance mandate")
}

/// Fun fact about mandate
pub fn mandate_fun_fact() -> &'static str {
    "Anna's settings mandate ensures configuration compliance with authoritative requirements!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mandate_type_display() {
        assert_eq!(format!("{}", MandateType::Legal), "legal");
        assert_eq!(format!("{}", MandateType::Regulatory), "regulatory");
    }

    #[test]
    fn test_compliance_display() {
        assert_eq!(format!("{}", MandateCompliance::Required), "required");
        assert_eq!(format!("{}", MandateCompliance::Exempt), "exempt");
    }

    #[test]
    fn test_config_new() {
        let c = MandateConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = MandateConfig::new("test")
            .mandate_type(MandateType::Corporate)
            .default_compliance(MandateCompliance::Recommended);
        assert_eq!(c.mandate_type, MandateType::Corporate);
        assert_eq!(c.default_compliance, MandateCompliance::Recommended);
    }

    #[test]
    fn test_requirement_new() {
        let r = MandateRequirement::new("r1", "Title", "Description");
        assert_eq!(r.id, "r1");
    }

    #[test]
    fn test_requirement_builder() {
        let r = MandateRequirement::new("r1", "Title", "Description")
            .compliance(MandateCompliance::Optional);
        assert_eq!(r.compliance, MandateCompliance::Optional);
    }

    #[test]
    fn test_requirement_fulfill() {
        let mut r = MandateRequirement::new("r1", "Title", "Description");
        r.fulfill();
        assert!(r.fulfilled);
        r.unfulfill();
        assert!(!r.fulfilled);
    }

    #[test]
    fn test_evidence_new() {
        let e = MandateEvidence::new("key", "value", "r1");
        assert_eq!(e.requirement_id, "r1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = MandateStats::default();
        let mut req = MandateRequirement::new("r1", "Title", "Description");
        req.fulfill();
        s.update(&[req], MandateType::Legal);
        assert_eq!(s.total_mandates, 1);
        assert_eq!(s.fulfilled, 1);
        assert_eq!(s.required_count, 1);
    }

    #[test]
    fn test_mandate_new() {
        let m = SettingsMandate::new(MandateConfig::default());
        assert_eq!(m.requirement_count(), 0);
    }

    #[test]
    fn test_mandate_add_requirement() {
        let mut m = SettingsMandate::new(MandateConfig::default());
        m.add_requirement(MandateRequirement::new("r1", "Title", "Description"));
        assert_eq!(m.requirement_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = MandateRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = MandateRegistry::new();
        r.register("m1", SettingsMandate::new(MandateConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_mandate_query() {
        assert!(is_mandate_query("settings mandate"));
        assert!(!is_mandate_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = mandate_fun_fact();
        assert!(fact.contains("mandate"));
    }
}
