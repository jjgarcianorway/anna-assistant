// v0.0.773: Settings Botanical Tests (Phase 349)
// Test suite for botanical garden functionality

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_botanical_type_display() {
        assert_eq!(format!("{}", BotanicalType::Display), "display");
        assert_eq!(format!("{}", BotanicalType::Research), "research");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", BotanicalStatus::Active), "active");
        assert_eq!(format!("{}", BotanicalStatus::Restoration), "restoration");
    }

    #[test]
    fn test_config_new() {
        let c = BotanicalConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = BotanicalConfig::new("test")
            .botanical_type(BotanicalType::Conservation)
            .status(BotanicalStatus::Expanding);
        assert_eq!(c.botanical_type, BotanicalType::Conservation);
        assert_eq!(c.status, BotanicalStatus::Expanding);
    }

    #[test]
    fn test_collection_new() {
        let c = BotanicalCollection::new("c1", "Title", "Content");
        assert_eq!(c.id, "c1");
    }

    #[test]
    fn test_collection_builder() {
        let c = BotanicalCollection::new("c1", "Title", "Content")
            .wing(1);
        assert_eq!(c.wing, 1);
    }

    #[test]
    fn test_collection_documented() {
        let mut c = BotanicalCollection::new("c1", "Title", "Content");
        c.make_undocumented();
        assert!(!c.documented);
        c.make_documented();
        assert!(c.documented);
    }

    #[test]
    fn test_botanist_new() {
        let b = BotanicalBotanist::new("key", "name", "c1");
        assert_eq!(b.collection_id, "c1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = BotanicalStats::default();
        let collection = BotanicalCollection::new("c1", "Title", "Content");
        s.update(&[collection], BotanicalType::Display);
        assert_eq!(s.total_collections, 1);
        assert_eq!(s.documented, 1);
    }

    #[test]
    fn test_botanical_new() {
        let b = SettingsBotanical::new(BotanicalConfig::default());
        assert_eq!(b.collection_count(), 0);
    }

    #[test]
    fn test_botanical_add_collection() {
        let mut b = SettingsBotanical::new(BotanicalConfig::default());
        b.add_collection(BotanicalCollection::new("c1", "Title", "Content"));
        assert_eq!(b.collection_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = BotanicalRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = BotanicalRegistry::new();
        r.register("b1", SettingsBotanical::new(BotanicalConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_botanical_query() {
        assert!(is_botanical_query("settings botanical"));
        assert!(!is_botanical_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = botanical_fun_fact();
        assert!(fact.contains("botanical"));
    }
}
