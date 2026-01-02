// v0.0.737: Settings Federation (Phase 313)
// Federal union for settings governance - Tests

#[cfg(test)]
mod tests {
    use super::super::core::SettingsFederation;
    use super::super::registry::FederationRegistry;
    use super::super::types::{FederationArticle, FederationConfig, FederationState, FederationStats, FederationStatus, FederationType};
    use super::super::utils::{is_federation_query, federation_fun_fact};

    #[test]
    fn test_federation_type_display() {
        assert_eq!(format!("{}", FederationType::Symmetric), "symmetric");
        assert_eq!(format!("{}", FederationType::Asymmetric), "asymmetric");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", FederationStatus::Constituting), "constituting");
        assert_eq!(format!("{}", FederationStatus::Established), "established");
    }

    #[test]
    fn test_config_new() {
        let c = FederationConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = FederationConfig::new("test")
            .federation_type(FederationType::Asymmetric)
            .status(FederationStatus::Established);
        assert_eq!(c.federation_type, FederationType::Asymmetric);
        assert_eq!(c.status, FederationStatus::Established);
    }

    #[test]
    fn test_article_new() {
        let a = FederationArticle::new("a1", "Title", "Content");
        assert_eq!(a.id, "a1");
    }

    #[test]
    fn test_article_builder() {
        let a = FederationArticle::new("a1", "Title", "Content")
            .section(1);
        assert_eq!(a.section, 1);
    }

    #[test]
    fn test_article_constitutional() {
        let mut a = FederationArticle::new("a1", "Title", "Content");
        a.make_constitutional();
        assert!(a.constitutional);
        a.make_statutory();
        assert!(!a.constitutional);
    }

    #[test]
    fn test_state_new() {
        let s = FederationState::new("key", "name", "a1");
        assert_eq!(s.article_id, "a1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = FederationStats::default();
        let mut article = FederationArticle::new("a1", "Title", "Content");
        article.make_constitutional();
        s.update(&[article], FederationType::Symmetric);
        assert_eq!(s.total_articles, 1);
        assert_eq!(s.constitutional, 1);
    }

    #[test]
    fn test_federation_new() {
        let f = SettingsFederation::new(FederationConfig::default());
        assert_eq!(f.article_count(), 0);
    }

    #[test]
    fn test_federation_add_article() {
        let mut f = SettingsFederation::new(FederationConfig::default());
        f.add_article(FederationArticle::new("a1", "Title", "Content"));
        assert_eq!(f.article_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = FederationRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = FederationRegistry::new();
        r.register("f1", SettingsFederation::new(FederationConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_federation_query() {
        assert!(is_federation_query("settings federation"));
        assert!(!is_federation_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = federation_fun_fact();
        assert!(fact.contains("federation"));
    }
}
