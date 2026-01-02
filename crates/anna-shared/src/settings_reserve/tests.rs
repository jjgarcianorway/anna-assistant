// v0.0.782: Settings Reserve - Tests
// Unit tests for settings reserve

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_reserve_type_display() {
        assert_eq!(format!("{}", ReserveType::Nature), "nature");
        assert_eq!(format!("{}", ReserveType::Game), "game");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", ReserveStatus::Protected), "protected");
        assert_eq!(format!("{}", ReserveStatus::Conserved), "conserved");
    }

    #[test]
    fn test_config_new() {
        let c = ReserveConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = ReserveConfig::new("test")
            .reserve_type(ReserveType::Game)
            .status(ReserveStatus::Managed);
        assert_eq!(c.reserve_type, ReserveType::Game);
        assert_eq!(c.status, ReserveStatus::Managed);
    }

    #[test]
    fn test_species_new() {
        let s = ReserveSpecies::new("s1", "Title", "Content");
        assert_eq!(s.id, "s1");
    }

    #[test]
    fn test_species_builder() {
        let s = ReserveSpecies::new("s1", "Title", "Content")
            .territory(1);
        assert_eq!(s.territory, 1);
    }

    #[test]
    fn test_species_thriving() {
        let mut s = ReserveSpecies::new("s1", "Title", "Content");
        s.make_endangered();
        assert!(!s.thriving);
        s.make_thriving();
        assert!(s.thriving);
    }

    #[test]
    fn test_ranger_new() {
        let r = ReserveRanger::new("key", "name", "s1");
        assert_eq!(r.species_id, "s1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = ReserveStats::default();
        let species = ReserveSpecies::new("s1", "Title", "Content");
        s.update(&[species], ReserveType::Nature);
        assert_eq!(s.total_species, 1);
        assert_eq!(s.thriving, 1);
    }

    #[test]
    fn test_reserve_new() {
        let r = SettingsReserve::new(ReserveConfig::default());
        assert_eq!(r.species_count(), 0);
    }

    #[test]
    fn test_reserve_add_species() {
        let mut r = SettingsReserve::new(ReserveConfig::default());
        r.add_species(ReserveSpecies::new("s1", "Title", "Content"));
        assert_eq!(r.species_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = ReserveRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ReserveRegistry::new();
        r.register("r1", SettingsReserve::new(ReserveConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_reserve_query() {
        assert!(is_reserve_query("settings reserve"));
        assert!(!is_reserve_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = reserve_fun_fact();
        assert!(fact.contains("reserve"));
    }
}
