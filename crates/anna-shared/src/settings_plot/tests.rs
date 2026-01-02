// v0.0.758: Settings Plot (Phase 334)
// Tests

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_plot_type_display() {
        assert_eq!(format!("{}", PlotType::Garden), "garden");
        assert_eq!(format!("{}", PlotType::Building), "building");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", PlotStatus::Allocated), "allocated");
        assert_eq!(format!("{}", PlotStatus::Cultivated), "cultivated");
    }

    #[test]
    fn test_config_new() {
        let c = PlotConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = PlotConfig::new("test")
            .plot_type(PlotType::Cemetery)
            .status(PlotStatus::Fallow);
        assert_eq!(c.plot_type, PlotType::Cemetery);
        assert_eq!(c.status, PlotStatus::Fallow);
    }

    #[test]
    fn test_survey_new() {
        let s = PlotSurvey::new("s1", "Title", "Content");
        assert_eq!(s.id, "s1");
    }

    #[test]
    fn test_survey_builder() {
        let s = PlotSurvey::new("s1", "Title", "Content")
            .row(1);
        assert_eq!(s.row, 1);
    }

    #[test]
    fn test_survey_verified() {
        let mut s = PlotSurvey::new("s1", "Title", "Content");
        s.make_unverified();
        assert!(!s.verified);
        s.make_verified();
        assert!(s.verified);
    }

    #[test]
    fn test_steward_new() {
        let s = PlotSteward::new("key", "name", "s1");
        assert_eq!(s.survey_id, "s1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = PlotStats::default();
        let survey = PlotSurvey::new("s1", "Title", "Content");
        s.update(&[survey], PlotType::Garden);
        assert_eq!(s.total_surveys, 1);
        assert_eq!(s.verified, 1);
    }

    #[test]
    fn test_plot_new() {
        let p = SettingsPlot::new(PlotConfig::default());
        assert_eq!(p.survey_count(), 0);
    }

    #[test]
    fn test_plot_add_survey() {
        let mut p = SettingsPlot::new(PlotConfig::default());
        p.add_survey(PlotSurvey::new("s1", "Title", "Content"));
        assert_eq!(p.survey_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = PlotRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = PlotRegistry::new();
        r.register("p1", SettingsPlot::new(PlotConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_plot_query() {
        assert!(is_plot_query("settings plot"));
        assert!(!is_plot_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = plot_fun_fact();
        assert!(fact.contains("plot"));
    }
}
