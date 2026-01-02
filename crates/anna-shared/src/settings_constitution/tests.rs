// v0.0.725: Constitution Tests (Phase 301)

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_constitution_type_display() {
        assert_eq!(format!("{}", ConstitutionType::Written), "written");
        assert_eq!(format!("{}", ConstitutionType::Codified), "codified");
    }

    #[test]
    fn test_branch_display() {
        assert_eq!(format!("{}", ConstitutionBranch::Executive), "executive");
        assert_eq!(format!("{}", ConstitutionBranch::Judicial), "judicial");
    }

    #[test]
    fn test_config_new() {
        let c = ConstitutionConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = ConstitutionConfig::new("test")
            .constitution_type(ConstitutionType::Codified)
            .branch(ConstitutionBranch::Legislative);
        assert_eq!(c.constitution_type, ConstitutionType::Codified);
        assert_eq!(c.branch, ConstitutionBranch::Legislative);
    }

    #[test]
    fn test_article_new() {
        let a = ConstitutionArticle::new("a1", "Title", "Content");
        assert_eq!(a.id, "a1");
    }

    #[test]
    fn test_article_builder() {
        let a = ConstitutionArticle::new("a1", "Title", "Content")
            .number(1);
        assert_eq!(a.number, 1);
    }

    #[test]
    fn test_article_ratify_repeal() {
        let mut a = ConstitutionArticle::new("a1", "Title", "Content");
        a.ratify();
        assert!(a.ratified);
        a.repeal();
        assert!(!a.ratified);
    }

    #[test]
    fn test_clause_new() {
        let c = ConstitutionClause::new("key", "value", "a1");
        assert_eq!(c.article_id, "a1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = ConstitutionStats::default();
        let mut article = ConstitutionArticle::new("a1", "Title", "Content");
        article.ratify();
        s.update(&[article], ConstitutionType::Written);
        assert_eq!(s.total_articles, 1);
        assert_eq!(s.ratified, 1);
    }

    #[test]
    fn test_constitution_new() {
        let c = SettingsConstitution::new(ConstitutionConfig::default());
        assert_eq!(c.article_count(), 0);
    }

    #[test]
    fn test_constitution_add_article() {
        let mut c = SettingsConstitution::new(ConstitutionConfig::default());
        c.add_article(ConstitutionArticle::new("a1", "Title", "Content"));
        assert_eq!(c.article_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = ConstitutionRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ConstitutionRegistry::new();
        r.register("c1", SettingsConstitution::new(ConstitutionConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_constitution_query() {
        assert!(is_constitution_query("settings constitution"));
        assert!(!is_constitution_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = constitution_fun_fact();
        assert!(fact.contains("constitution"));
    }
}
