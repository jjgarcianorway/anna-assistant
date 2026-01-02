// v0.0.642: Settings Analyzer (Phase 218)
// Types for settings analysis

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

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self::new(AnalysisType::Pattern)
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
