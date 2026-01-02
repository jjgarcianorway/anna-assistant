// v0.0.660: Settings Versioner - Tests
// All tests for the versioner module

#[cfg(test)]
mod tests {
    use super::super::version_types::{VersionScheme, BumpType};
    use super::super::version_config::VersionerConfig;
    use super::super::version_core::{SettingsVersion, VersionResult, VersionerStats};
    use super::super::versioner::SettingsVersioner;
    use super::super::registry::SettingsVersionerRegistry;
    use super::super::utils::{is_versioner_query, versioner_fun_fact};

    #[test]
    fn test_version_scheme_display() {
        assert_eq!(format!("{}", VersionScheme::Semantic), "semantic");
        assert_eq!(format!("{}", VersionScheme::Sequential), "sequential");
    }

    #[test]
    fn test_bump_type_display() {
        assert_eq!(format!("{}", BumpType::Major), "major");
        assert_eq!(format!("{}", BumpType::Minor), "minor");
    }

    #[test]
    fn test_config_new() {
        let c = VersionerConfig::new(VersionScheme::Semantic);
        assert!(c.track_history);
    }

    #[test]
    fn test_config_builder() {
        let c = VersionerConfig::new(VersionScheme::DateBased)
            .default_bump(BumpType::Patch)
            .max_history(50);
        assert_eq!(c.default_bump, BumpType::Patch);
        assert_eq!(c.max_history, 50);
    }

    #[test]
    fn test_version_new() {
        let v = SettingsVersion::new(1, 2, 3);
        assert_eq!(v.version, "1.2.3");
    }

    #[test]
    fn test_version_from_string() {
        let v = SettingsVersion::from_string("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_version_bump() {
        let v = SettingsVersion::new(1, 0, 0);
        let bumped = v.bump(BumpType::Minor);
        assert_eq!(bumped.version, "1.1.0");
    }

    #[test]
    fn test_result_new() {
        let r = VersionResult::new(SettingsVersion::new(1, 0, 0), BumpType::Minor);
        assert!(!r.was_bumped());
    }

    #[test]
    fn test_result_with_previous() {
        let r = VersionResult::new(SettingsVersion::new(1, 1, 0), BumpType::Minor)
            .with_previous(SettingsVersion::new(1, 0, 0));
        assert!(r.was_bumped());
    }

    #[test]
    fn test_stats_record() {
        let mut s = VersionerStats::default();
        s.record(BumpType::Minor, "1.1.0");
        assert_eq!(s.total_bumps, 1);
        assert_eq!(s.current_version, Some("1.1.0".to_string()));
    }

    #[test]
    fn test_versioner_new() {
        let v = SettingsVersioner::new(VersionerConfig::new(VersionScheme::Semantic));
        assert_eq!(v.current().version, "0.0.1");
    }

    #[test]
    fn test_versioner_bump() {
        let mut v = SettingsVersioner::new(VersionerConfig::new(VersionScheme::Semantic));
        let r = v.bump(BumpType::Minor);
        assert_eq!(r.current.version, "0.1.0");
        assert!(r.was_bumped());
    }

    #[test]
    fn test_versioner_history() {
        let mut v = SettingsVersioner::new(VersionerConfig::new(VersionScheme::Semantic));
        v.bump(BumpType::Minor);
        v.bump(BumpType::Minor);
        assert_eq!(v.history_count(), 2);
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsVersionerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsVersionerRegistry::new();
        r.register("v1", SettingsVersioner::new(VersionerConfig::new(VersionScheme::Semantic)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_versioner_query() {
        assert!(is_versioner_query("settings versioner"));
        assert!(!is_versioner_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = versioner_fun_fact();
        assert!(fact.contains("versioner"));
    }
}
