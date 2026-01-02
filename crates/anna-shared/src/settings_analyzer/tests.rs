// v0.0.642: Settings Analyzer (Phase 218)
// Tests for settings analyzer

#[cfg(test)]
mod tests {
    use crate::settings_analyzer::{
        analyzer::SettingsAnalyzer,
        helpers::{analyzer_fun_fact, is_analyzer_query},
        registry::SettingsAnalyzerRegistry,
        types::{AnalysisInsight, AnalysisResult, AnalysisScope, AnalysisType, AnalyzerConfig, AnalyzerStats},
    };

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
