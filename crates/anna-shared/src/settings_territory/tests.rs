// v0.0.745: Settings Territory - Tests
// Unit tests for settings territory module

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_territory_type_display() {
        assert_eq!(format!("{}", TerritoryType::Sovereign), "sovereign");
        assert_eq!(format!("{}", TerritoryType::Occupied), "occupied");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", TerritoryStatus::Administered), "administered");
        assert_eq!(format!("{}", TerritoryStatus::Autonomous), "autonomous");
    }

    #[test]
    fn test_config_new() {
        let c = TerritoryConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = TerritoryConfig::new("test")
            .territory_type(TerritoryType::Trust)
            .status(TerritoryStatus::Autonomous);
        assert_eq!(c.territory_type, TerritoryType::Trust);
        assert_eq!(c.status, TerritoryStatus::Autonomous);
    }

    #[test]
    fn test_ordinance_new() {
        let o = TerritoryOrdinance::new("o1", "Title", "Content");
        assert_eq!(o.id, "o1");
    }

    #[test]
    fn test_ordinance_builder() {
        let o = TerritoryOrdinance::new("o1", "Title", "Content")
            .district(1);
        assert_eq!(o.district, 1);
    }

    #[test]
    fn test_ordinance_enforced() {
        let mut o = TerritoryOrdinance::new("o1", "Title", "Content");
        o.make_suspended();
        assert!(!o.enforced);
        o.make_enforced();
        assert!(o.enforced);
    }

    #[test]
    fn test_administrator_new() {
        let a = TerritoryAdministrator::new("key", "name", "o1");
        assert_eq!(a.ordinance_id, "o1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = TerritoryStats::default();
        let ordinance = TerritoryOrdinance::new("o1", "Title", "Content");
        s.update(&[ordinance], TerritoryType::Sovereign);
        assert_eq!(s.total_ordinances, 1);
        assert_eq!(s.enforced, 1);
    }

    #[test]
    fn test_territory_new() {
        let t = SettingsTerritory::new(TerritoryConfig::default());
        assert_eq!(t.ordinance_count(), 0);
    }

    #[test]
    fn test_territory_add_ordinance() {
        let mut t = SettingsTerritory::new(TerritoryConfig::default());
        t.add_ordinance(TerritoryOrdinance::new("o1", "Title", "Content"));
        assert_eq!(t.ordinance_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = TerritoryRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = TerritoryRegistry::new();
        r.register("t1", SettingsTerritory::new(TerritoryConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_territory_query() {
        assert!(is_territory_query("settings territory"));
        assert!(!is_territory_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = territory_fun_fact();
        assert!(fact.contains("territory"));
    }
}
