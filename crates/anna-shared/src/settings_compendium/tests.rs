// v0.0.700: Settings Compendium (Phase 276) - Milestone!
// Tests

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_compendium_type_display() {
        assert_eq!(format!("{}", CompendiumType::Reference), "reference");
        assert_eq!(format!("{}", CompendiumType::Encyclopedia), "encyclopedia");
    }

    #[test]
    fn test_edition_display() {
        assert_eq!(format!("{}", CompendiumEdition::First), "first");
        assert_eq!(format!("{}", CompendiumEdition::Final), "final");
    }

    #[test]
    fn test_config_new() {
        let c = CompendiumConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = CompendiumConfig::new("test")
            .compendium_type(CompendiumType::Encyclopedia)
            .edition(CompendiumEdition::Revised);
        assert_eq!(c.compendium_type, CompendiumType::Encyclopedia);
        assert_eq!(c.edition, CompendiumEdition::Revised);
    }

    #[test]
    fn test_volume_new() {
        let v = CompendiumVolume::new(1, "Volume 1");
        assert_eq!(v.number, 1);
    }

    #[test]
    fn test_volume_add() {
        let mut v = CompendiumVolume::new(1, "Volume 1");
        v.add(CompendiumArticle::new("a1", "Article 1", "Content"));
        assert_eq!(v.article_count(), 1);
    }

    #[test]
    fn test_article_new() {
        let a = CompendiumArticle::new("a1", "Article 1", "Content");
        assert_eq!(a.id, "a1");
    }

    #[test]
    fn test_article_keywords() {
        let a = CompendiumArticle::new("a1", "Article", "Content")
            .keyword("config")
            .keyword("settings");
        assert_eq!(a.keywords.len(), 2);
    }

    #[test]
    fn test_entry_new() {
        let e = CompendiumEntry::new("key", "value", "a1");
        assert_eq!(e.article_id, "a1");
    }

    #[test]
    fn test_entry_definition() {
        let e = CompendiumEntry::new("key", "value", "a1").definition("A configuration key");
        assert!(e.definition.is_some());
    }

    #[test]
    fn test_stats_update() {
        let mut s = CompendiumStats::default();
        let volumes = vec![CompendiumVolume::new(1, "Volume")];
        s.update(&volumes);
        assert_eq!(s.total_volumes, 1);
    }

    #[test]
    fn test_compendium_new() {
        let c = SettingsCompendium::new(CompendiumConfig::default());
        assert_eq!(c.volume_count(), 0);
    }

    #[test]
    fn test_compendium_add_volume() {
        let mut c = SettingsCompendium::new(CompendiumConfig::default());
        c.add_volume(CompendiumVolume::new(1, "Volume 1"));
        assert_eq!(c.volume_count(), 1);
    }

    #[test]
    fn test_compendium_add_entry() {
        let mut c = SettingsCompendium::new(CompendiumConfig::default());
        c.add_entry(CompendiumEntry::new("key", "value", "a1"));
        assert_eq!(c.entry_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = CompendiumRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = CompendiumRegistry::new();
        r.register("c1", SettingsCompendium::new(CompendiumConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_compendium_query() {
        assert!(is_compendium_query("settings compendium"));
        assert!(!is_compendium_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = compendium_fun_fact();
        assert!(fact.contains("compendium"));
    }
}
