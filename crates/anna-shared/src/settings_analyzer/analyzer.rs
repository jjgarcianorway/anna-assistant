// v0.0.642: Settings Analyzer (Phase 218)
// Settings analyzer implementation

use crate::settings_analyzer::types::{AnalyzerConfig, AnalyzerStats, AnalysisResult, AnalysisType};

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
