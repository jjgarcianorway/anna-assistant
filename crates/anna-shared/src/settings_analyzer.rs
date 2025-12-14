// v0.0.642: Settings Analyzer (Phase 218)
// Analyzer for settings patterns and anomalies

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Analysis type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AnalysisType {
    /// Pattern analysis
    #[default]
    Pattern,
    /// Anomaly analysis
    Anomaly,
    /// Trend analysis
    Trend,
    /// Correlation analysis
    Correlation,
    /// Impact analysis
    Impact,
}

impl std::fmt::Display for AnalysisType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pattern => write!(f, "pattern"),
            Self::Anomaly => write!(f, "anomaly"),
            Self::Trend => write!(f, "trend"),
            Self::Correlation => write!(f, "correlation"),
            Self::Impact => write!(f, "impact"),
        }
    }
}

/// Analysis scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AnalysisScope {
    /// Single category
    Category,
    /// Cross-category
    #[default]
    CrossCategory,
    /// System-wide
    SystemWide,
    /// Historical
    Historical,
}

impl std::fmt::Display for AnalysisScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Category => write!(f, "category"),
            Self::CrossCategory => write!(f, "cross_category"),
            Self::SystemWide => write!(f, "system_wide"),
            Self::Historical => write!(f, "historical"),
        }
    }
}

/// Analyzer config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzerConfig {
    /// Analysis type
    pub analysis_type: AnalysisType,
    /// Scope
    pub scope: AnalysisScope,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Threshold
    pub threshold: f64,
    /// Enabled
    pub enabled: bool,
}

impl AnalyzerConfig {
    /// Create new config
    pub fn new(analysis_type: AnalysisType) -> Self {
        Self {
            analysis_type,
            scope: AnalysisScope::CrossCategory,
            category: None,
            threshold: 0.5,
            enabled: true,
        }
    }

    /// Set scope
    pub fn scope(mut self, scope: AnalysisScope) -> Self {
        self.scope = scope;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set threshold
    pub fn threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold;
        self
    }
}

/// Analysis insight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisInsight {
    /// ID
    pub id: String,
    /// Type
    pub insight_type: String,
    /// Category
    pub category: Option<SettingsCategory>,
    /// Description
    pub description: String,
    /// Confidence
    pub confidence: f64,
    /// Recommendation
    pub recommendation: Option<String>,
}

impl AnalysisInsight {
    /// Create new insight
    pub fn new(id: impl Into<String>, insight_type: impl Into<String>, confidence: f64) -> Self {
        Self {
            id: id.into(),
            insight_type: insight_type.into(),
            category: None,
            description: String::new(),
            confidence,
            recommendation: None,
        }
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set recommendation
    pub fn recommendation(mut self, rec: impl Into<String>) -> Self {
        self.recommendation = Some(rec.into());
        self
    }

    /// Is high confidence
    pub fn is_high_confidence(&self) -> bool {
        self.confidence >= 0.8
    }
}

/// Analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// ID
    pub id: String,
    /// Analysis type
    pub analysis_type: AnalysisType,
    /// Insights
    pub insights: Vec<AnalysisInsight>,
    /// Timestamp
    pub timestamp: u64,
    /// Duration ms
    pub duration_ms: u64,
}

impl AnalysisResult {
    /// Create new result
    pub fn new(id: impl Into<String>, analysis_type: AnalysisType) -> Self {
        Self {
            id: id.into(),
            analysis_type,
            insights: Vec::new(),
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

    /// Add insight
    pub fn add_insight(&mut self, insight: AnalysisInsight) {
        self.insights.push(insight);
    }

    /// Insight count
    pub fn insight_count(&self) -> usize {
        self.insights.len()
    }

    /// Has insights
    pub fn has_insights(&self) -> bool {
        !self.insights.is_empty()
    }

    /// High confidence insights
    pub fn high_confidence_insights(&self) -> Vec<&AnalysisInsight> {
        self.insights.iter().filter(|i| i.is_high_confidence()).collect()
    }
}

/// Analyzer stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalyzerStats {
    /// Total analyses
    pub total_analyses: usize,
    /// Total insights
    pub total_insights: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl AnalyzerStats {
    /// Record analysis
    pub fn record(&mut self, analysis_type: AnalysisType, insight_count: usize) {
        self.total_analyses += 1;
        self.total_insights += insight_count;
        *self.by_type.entry(analysis_type.to_string()).or_insert(0) += 1;
    }

    /// Average insights
    pub fn average_insights(&self) -> f64 {
        if self.total_analyses == 0 {
            0.0
        } else {
            self.total_insights as f64 / self.total_analyses as f64
        }
    }
}

/// Settings analyzer
#[derive(Debug, Clone, Default)]
pub struct SettingsAnalyzer {
    /// Config
    config: AnalyzerConfig,
    /// Results
    results: Vec<AnalysisResult>,
    /// Stats
    stats: AnalyzerStats,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self::new(AnalysisType::Pattern)
    }
}

impl SettingsAnalyzer {
    /// Create new analyzer
    pub fn new(config: AnalyzerConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
            stats: AnalyzerStats::default(),
        }
    }

    /// Analyze
    pub fn analyze(&mut self, id: impl Into<String>) -> AnalysisResult {
        let result = AnalysisResult::new(id, self.config.analysis_type);
        self.stats.record(self.config.analysis_type, 0);
        self.results.push(result.clone());
        result
    }

    /// Get results
    pub fn results(&self) -> &[AnalysisResult] {
        &self.results
    }

    /// Get stats
    pub fn stats(&self) -> &AnalyzerStats {
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

    /// Is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

/// Settings analyzer registry
#[derive(Debug, Clone, Default)]
pub struct SettingsAnalyzerRegistry {
    /// Analyzers by ID
    analyzers: HashMap<String, SettingsAnalyzer>,
}

impl SettingsAnalyzerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register analyzer
    pub fn register(&mut self, id: impl Into<String>, analyzer: SettingsAnalyzer) {
        self.analyzers.insert(id.into(), analyzer);
    }

    /// Unregister analyzer
    pub fn unregister(&mut self, id: &str) -> bool {
        self.analyzers.remove(id).is_some()
    }

    /// Get analyzer
    pub fn get(&self, id: &str) -> Option<&SettingsAnalyzer> {
        self.analyzers.get(id)
    }

    /// Get analyzer mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsAnalyzer> {
        self.analyzers.get_mut(id)
    }

    /// Analyzer count
    pub fn count(&self) -> usize {
        self.analyzers.len()
    }

    /// List enabled
    pub fn list_enabled(&self) -> Vec<&SettingsAnalyzer> {
        self.analyzers.values().filter(|a| a.is_enabled()).collect()
    }
}

/// Format analyzer registry
pub fn format_analyzer_registry(registry: &SettingsAnalyzerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Analyzer Registry:\n");
    output.push_str(&format!("  Analyzers: {}\n", registry.count()));
    output
}

/// Check if query is about analyzer
pub fn is_analyzer_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("analyzer") || lower.contains("analyze settings") || lower.contains("pattern")
}

/// Fun fact about analyzer
pub fn analyzer_fun_fact() -> &'static str {
    "Anna's settings analyzers detect patterns and anomalies!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analysis_type_display() {
        assert_eq!(format!("{}", AnalysisType::Pattern), "pattern");
        assert_eq!(format!("{}", AnalysisType::Anomaly), "anomaly");
    }

    #[test]
    fn test_scope_display() {
        assert_eq!(format!("{}", AnalysisScope::Category), "category");
        assert_eq!(format!("{}", AnalysisScope::SystemWide), "system_wide");
    }

    #[test]
    fn test_config_new() {
        let c = AnalyzerConfig::new(AnalysisType::Pattern);
        assert!(c.enabled);
    }

    #[test]
    fn test_config_builder() {
        let c = AnalyzerConfig::new(AnalysisType::Anomaly)
            .scope(AnalysisScope::SystemWide)
            .threshold(0.9);
        assert_eq!(c.scope, AnalysisScope::SystemWide);
        assert_eq!(c.threshold, 0.9);
    }

    #[test]
    fn test_insight_new() {
        let i = AnalysisInsight::new("i1", "pattern", 0.85);
        assert!(i.is_high_confidence());
    }

    #[test]
    fn test_insight_builder() {
        let i = AnalysisInsight::new("i1", "anomaly", 0.7)
            .description("Test")
            .recommendation("Fix it");
        assert!(i.recommendation.is_some());
    }

    #[test]
    fn test_result_new() {
        let r = AnalysisResult::new("r1", AnalysisType::Pattern);
        assert_eq!(r.insight_count(), 0);
    }

    #[test]
    fn test_result_insights() {
        let mut r = AnalysisResult::new("r1", AnalysisType::Pattern);
        r.add_insight(AnalysisInsight::new("i1", "pattern", 0.9));
        assert!(r.has_insights());
    }

    #[test]
    fn test_stats_record() {
        let mut s = AnalyzerStats::default();
        s.record(AnalysisType::Pattern, 5);
        assert_eq!(s.total_analyses, 1);
        assert_eq!(s.total_insights, 5);
    }

    #[test]
    fn test_analyzer_new() {
        let a = SettingsAnalyzer::new(AnalyzerConfig::new(AnalysisType::Pattern));
        assert_eq!(a.result_count(), 0);
    }

    #[test]
    fn test_analyzer_analyze() {
        let mut a = SettingsAnalyzer::new(AnalyzerConfig::new(AnalysisType::Pattern));
        a.analyze("a1");
        assert_eq!(a.result_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsAnalyzerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsAnalyzerRegistry::new();
        r.register("ana1", SettingsAnalyzer::new(AnalyzerConfig::new(AnalysisType::Pattern)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_analyzer_query() {
        assert!(is_analyzer_query("settings analyzer"));
        assert!(!is_analyzer_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = analyzer_fun_fact();
        assert!(fact.contains("analyzer"));
    }
}
