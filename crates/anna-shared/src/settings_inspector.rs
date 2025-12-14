// v0.0.641: Settings Inspector (Phase 217)
// Inspector for settings structure and values

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Inspection type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum InspectionType {
    /// Structure inspection
    #[default]
    Structure,
    /// Value inspection
    Value,
    /// Type inspection
    Type,
    /// Dependency inspection
    Dependency,
    /// Full inspection
    Full,
}

impl std::fmt::Display for InspectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Structure => write!(f, "structure"),
            Self::Value => write!(f, "value"),
            Self::Type => write!(f, "type"),
            Self::Dependency => write!(f, "dependency"),
            Self::Full => write!(f, "full"),
        }
    }
}

/// Inspection depth
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum InspectionDepth {
    /// Shallow
    Shallow,
    /// Normal
    #[default]
    Normal,
    /// Deep
    Deep,
    /// Complete
    Complete,
}

impl std::fmt::Display for InspectionDepth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Shallow => write!(f, "shallow"),
            Self::Normal => write!(f, "normal"),
            Self::Deep => write!(f, "deep"),
            Self::Complete => write!(f, "complete"),
        }
    }
}

/// Inspector config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectorConfig {
    /// Inspection type
    pub inspection_type: InspectionType,
    /// Depth
    pub depth: InspectionDepth,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Include metadata
    pub include_metadata: bool,
    /// Include defaults
    pub include_defaults: bool,
}

impl InspectorConfig {
    /// Create new config
    pub fn new(inspection_type: InspectionType) -> Self {
        Self {
            inspection_type,
            depth: InspectionDepth::Normal,
            category: None,
            include_metadata: true,
            include_defaults: false,
        }
    }

    /// Set depth
    pub fn depth(mut self, depth: InspectionDepth) -> Self {
        self.depth = depth;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set include metadata
    pub fn include_metadata(mut self, include: bool) -> Self {
        self.include_metadata = include;
        self
    }

    /// Set include defaults
    pub fn include_defaults(mut self, include: bool) -> Self {
        self.include_defaults = include;
        self
    }
}

/// Inspection finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionFinding {
    /// Finding ID
    pub id: String,
    /// Category
    pub category: SettingsCategory,
    /// Key
    pub key: String,
    /// Finding type
    pub finding_type: String,
    /// Description
    pub description: String,
    /// Severity
    pub severity: String,
}

impl InspectionFinding {
    /// Create new finding
    pub fn new(
        id: impl Into<String>,
        category: SettingsCategory,
        key: impl Into<String>,
        finding_type: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            category,
            key: key.into(),
            finding_type: finding_type.into(),
            description: String::new(),
            severity: "info".to_string(),
        }
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set severity
    pub fn severity(mut self, sev: impl Into<String>) -> Self {
        self.severity = sev.into();
        self
    }
}

/// Inspection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionResult {
    /// ID
    pub id: String,
    /// Inspection type
    pub inspection_type: InspectionType,
    /// Findings
    pub findings: Vec<InspectionFinding>,
    /// Timestamp
    pub timestamp: u64,
    /// Duration ms
    pub duration_ms: u64,
}

impl InspectionResult {
    /// Create new result
    pub fn new(id: impl Into<String>, inspection_type: InspectionType) -> Self {
        Self {
            id: id.into(),
            inspection_type,
            findings: Vec::new(),
            timestamp: 0,
            duration_ms: 0,
        }
    }

    /// Set timestamp
    pub fn timestamp(mut self, ts: u64) -> Self {
        self.timestamp = ts;
        self
    }

    /// Set duration
    pub fn duration_ms(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }

    /// Add finding
    pub fn add_finding(&mut self, finding: InspectionFinding) {
        self.findings.push(finding);
    }

    /// Finding count
    pub fn finding_count(&self) -> usize {
        self.findings.len()
    }

    /// Has findings
    pub fn has_findings(&self) -> bool {
        !self.findings.is_empty()
    }
}

/// Inspector stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InspectorStats {
    /// Total inspections
    pub total_inspections: usize,
    /// Total findings
    pub total_findings: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl InspectorStats {
    /// Record inspection
    pub fn record(&mut self, inspection_type: InspectionType, finding_count: usize) {
        self.total_inspections += 1;
        self.total_findings += finding_count;
        *self.by_type.entry(inspection_type.to_string()).or_insert(0) += 1;
    }

    /// Average findings
    pub fn average_findings(&self) -> f64 {
        if self.total_inspections == 0 {
            0.0
        } else {
            self.total_findings as f64 / self.total_inspections as f64
        }
    }
}

/// Settings inspector
#[derive(Debug, Clone, Default)]
pub struct SettingsInspector {
    /// Config
    config: InspectorConfig,
    /// Results
    results: Vec<InspectionResult>,
    /// Stats
    stats: InspectorStats,
}

impl Default for InspectorConfig {
    fn default() -> Self {
        Self::new(InspectionType::Structure)
    }
}

impl SettingsInspector {
    /// Create new inspector
    pub fn new(config: InspectorConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
            stats: InspectorStats::default(),
        }
    }

    /// Inspect
    pub fn inspect(&mut self, id: impl Into<String>) -> InspectionResult {
        let result = InspectionResult::new(id, self.config.inspection_type);
        self.stats.record(self.config.inspection_type, 0);
        self.results.push(result.clone());
        result
    }

    /// Get results
    pub fn results(&self) -> &[InspectionResult] {
        &self.results
    }

    /// Get stats
    pub fn stats(&self) -> &InspectorStats {
        &self.stats
    }

    /// Result count
    pub fn result_count(&self) -> usize {
        self.results.len()
    }

    /// Clear results
    pub fn clear(&mut self) {
        self.results.clear();
    }
}

/// Settings inspector registry
#[derive(Debug, Clone, Default)]
pub struct SettingsInspectorRegistry {
    /// Inspectors by ID
    inspectors: HashMap<String, SettingsInspector>,
}

impl SettingsInspectorRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register inspector
    pub fn register(&mut self, id: impl Into<String>, inspector: SettingsInspector) {
        self.inspectors.insert(id.into(), inspector);
    }

    /// Unregister inspector
    pub fn unregister(&mut self, id: &str) -> bool {
        self.inspectors.remove(id).is_some()
    }

    /// Get inspector
    pub fn get(&self, id: &str) -> Option<&SettingsInspector> {
        self.inspectors.get(id)
    }

    /// Get inspector mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsInspector> {
        self.inspectors.get_mut(id)
    }

    /// Inspector count
    pub fn count(&self) -> usize {
        self.inspectors.len()
    }
}

/// Format inspector registry
pub fn format_inspector_registry(registry: &SettingsInspectorRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Inspector Registry:\n");
    output.push_str(&format!("  Inspectors: {}\n", registry.count()));
    output
}

/// Check if query is about inspector
pub fn is_inspector_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("inspector") || lower.contains("inspect settings") || lower.contains("examine")
}

/// Fun fact about inspector
pub fn inspector_fun_fact() -> &'static str {
    "Anna's settings inspectors analyze structure and values!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspection_type_display() {
        assert_eq!(format!("{}", InspectionType::Structure), "structure");
        assert_eq!(format!("{}", InspectionType::Value), "value");
    }

    #[test]
    fn test_depth_display() {
        assert_eq!(format!("{}", InspectionDepth::Normal), "normal");
        assert_eq!(format!("{}", InspectionDepth::Deep), "deep");
    }

    #[test]
    fn test_config_new() {
        let c = InspectorConfig::new(InspectionType::Structure);
        assert!(c.include_metadata);
    }

    #[test]
    fn test_config_builder() {
        let c = InspectorConfig::new(InspectionType::Full)
            .depth(InspectionDepth::Deep)
            .include_defaults(true);
        assert_eq!(c.depth, InspectionDepth::Deep);
        assert!(c.include_defaults);
    }

    #[test]
    fn test_finding_new() {
        let f = InspectionFinding::new("f1", SettingsCategory::Privacy, "key", "missing");
        assert_eq!(f.severity, "info");
    }

    #[test]
    fn test_finding_builder() {
        let f = InspectionFinding::new("f1", SettingsCategory::Privacy, "key", "missing")
            .severity("warning");
        assert_eq!(f.severity, "warning");
    }

    #[test]
    fn test_result_new() {
        let r = InspectionResult::new("r1", InspectionType::Structure);
        assert_eq!(r.finding_count(), 0);
    }

    #[test]
    fn test_result_findings() {
        let mut r = InspectionResult::new("r1", InspectionType::Structure);
        r.add_finding(InspectionFinding::new("f1", SettingsCategory::Privacy, "key", "missing"));
        assert!(r.has_findings());
    }

    #[test]
    fn test_stats_record() {
        let mut s = InspectorStats::default();
        s.record(InspectionType::Structure, 5);
        assert_eq!(s.total_inspections, 1);
        assert_eq!(s.total_findings, 5);
    }

    #[test]
    fn test_inspector_new() {
        let i = SettingsInspector::new(InspectorConfig::new(InspectionType::Structure));
        assert_eq!(i.result_count(), 0);
    }

    #[test]
    fn test_inspector_inspect() {
        let mut i = SettingsInspector::new(InspectorConfig::new(InspectionType::Structure));
        i.inspect("i1");
        assert_eq!(i.result_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsInspectorRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsInspectorRegistry::new();
        r.register("insp1", SettingsInspector::new(InspectorConfig::new(InspectionType::Structure)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_inspector_query() {
        assert!(is_inspector_query("settings inspector"));
        assert!(!is_inspector_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = inspector_fun_fact();
        assert!(fact.contains("inspector"));
    }
}
