// v0.0.642: Settings Analyzer (Phase 218)
// Analyzer for settings patterns and anomalies

pub mod analyzer;
pub mod helpers;
pub mod registry;
pub mod types;

#[cfg(test)]
mod tests;

// Re-export types for public API
pub use analyzer::SettingsAnalyzer;
pub use helpers::{analyzer_fun_fact, format_analyzer_registry, is_analyzer_query};
pub use registry::SettingsAnalyzerRegistry;
pub use types::{
    AnalysisInsight, AnalysisResult, AnalysisScope, AnalysisType, AnalyzerConfig, AnalyzerStats,
};
