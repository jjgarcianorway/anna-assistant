// v0.0.723: Settings Statute Module (Phase 299)
// Written laws for settings governance

mod types;
mod statute;
mod registry;
mod helpers;

// Re-export all public types and functions
pub use types::{
    StatuteType,
    StatuteScope,
    StatuteConfig,
    StatuteArticle,
    StatuteSubsection,
    StatuteStats,
};

pub use statute::SettingsStatute;
pub use registry::StatuteRegistry;
pub use helpers::{
    format_statute_registry,
    is_statute_query,
    statute_fun_fact,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statute_type_display() {
        assert_eq!(format!("{}", StatuteType::General), "general");
        assert_eq!(format!("{}", StatuteType::Administrative), "administrative");
    }

    #[test]
    fn test_scope_display() {
        assert_eq!(format!("{}", StatuteScope::Federal), "federal");
        assert_eq!(format!("{}", StatuteScope::International), "international");
    }

    #[test]
    fn test_config_new() {
        let c = StatuteConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = StatuteConfig::new("test")
            .statute_type(StatuteType::Civil)
            .scope(StatuteScope::State);
        assert_eq!(c.statute_type, StatuteType::Civil);
        assert_eq!(c.scope, StatuteScope::State);
    }

    #[test]
    fn test_article_new() {
        let a = StatuteArticle::new("a1", "Title", "Content");
        assert_eq!(a.id, "a1");
    }

    #[test]
    fn test_article_builder() {
        let a = StatuteArticle::new("a1", "Title", "Content")
            .number("Article 1");
        assert_eq!(a.number, "Article 1");
    }

    #[test]
    fn test_article_enact_repeal() {
        let mut a = StatuteArticle::new("a1", "Title", "Content");
        a.enact();
        assert!(a.enacted);
        a.repeal();
        assert!(!a.enacted);
    }

    #[test]
    fn test_subsection_new() {
        let s = StatuteSubsection::new("key", "value", "a1");
        assert_eq!(s.article_id, "a1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = StatuteStats::default();
        let mut article = StatuteArticle::new("a1", "Title", "Content");
        article.enact();
        s.update(&[article], StatuteType::General);
        assert_eq!(s.total_statutes, 1);
        assert_eq!(s.enacted, 1);
    }

    #[test]
    fn test_statute_new() {
        let s = SettingsStatute::new(StatuteConfig::default());
        assert_eq!(s.article_count(), 0);
    }

    #[test]
    fn test_statute_add_article() {
        let mut s = SettingsStatute::new(StatuteConfig::default());
        s.add_article(StatuteArticle::new("a1", "Title", "Content"));
        assert_eq!(s.article_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = StatuteRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = StatuteRegistry::new();
        r.register("s1", SettingsStatute::new(StatuteConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_statute_query() {
        assert!(is_statute_query("settings statute"));
        assert!(!is_statute_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = statute_fun_fact();
        assert!(fact.contains("statute"));
    }
}
