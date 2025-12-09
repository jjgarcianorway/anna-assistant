//! Tests for cron_recipes module (v0.0.234).

#[cfg(test)]
mod tests {
    use crate::cron_recipes::{detect_feature, match_query, CronFeature, CronPreset};

    #[test]
    fn test_preset_expression() {
        assert_eq!(CronPreset::EveryMinute.expression(), "* * * * *");
        assert_eq!(CronPreset::Hourly.expression(), "0 * * * *");
        assert_eq!(CronPreset::Daily.expression(), "0 0 * * *");
        assert_eq!(CronPreset::Weekly.expression(), "0 0 * * 0");
        assert_eq!(CronPreset::Monthly.expression(), "0 0 1 * *");
    }

    #[test]
    fn test_match_add_job() {
        let recipe = match_query("how do I add a cron job");
        assert!(recipe.is_some());
        assert_eq!(recipe.unwrap().feature, CronFeature::AddJob);
    }

    #[test]
    fn test_match_list_jobs() {
        let recipe = match_query("list my cron jobs");
        assert!(recipe.is_some());
        assert_eq!(recipe.unwrap().feature, CronFeature::ListJobs);
    }

    #[test]
    fn test_match_syntax() {
        let recipe = match_query("cron syntax help");
        assert!(recipe.is_some());
        assert_eq!(recipe.unwrap().feature, CronFeature::SyntaxHelp);
    }

    #[test]
    fn test_match_debug() {
        let recipe = match_query("debug cron job not running");
        assert!(recipe.is_some());
        assert_eq!(recipe.unwrap().feature, CronFeature::DebugJob);
    }

    #[test]
    fn test_match_environment() {
        let recipe = match_query("cron environment variables");
        assert!(recipe.is_some());
        assert_eq!(recipe.unwrap().feature, CronFeature::Environment);
    }

    #[test]
    fn test_no_match_unrelated() {
        let recipe = match_query("what is the weather");
        assert!(recipe.is_none());
    }

    #[test]
    fn test_detect_feature_logs() {
        assert_eq!(detect_feature("cron logs"), Some(CronFeature::ViewLogs));
    }

    #[test]
    fn test_detect_feature_remove() {
        assert_eq!(
            detect_feature("remove cron job"),
            Some(CronFeature::RemoveJob)
        );
    }
}
