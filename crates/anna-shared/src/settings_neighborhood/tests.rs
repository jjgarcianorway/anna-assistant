// v0.0.754: Settings Neighborhood (Phase 330)
// Tests

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_neighborhood_type_display() {
        assert_eq!(format!("{}", NeighborhoodType::Residential), "residential");
        assert_eq!(format!("{}", NeighborhoodType::Commercial), "commercial");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", NeighborhoodStatus::Planned), "planned");
        assert_eq!(format!("{}", NeighborhoodStatus::Established), "established");
    }

    #[test]
    fn test_config_new() {
        let c = NeighborhoodConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = NeighborhoodConfig::new("test")
            .neighborhood_type(NeighborhoodType::Commercial)
            .status(NeighborhoodStatus::Developing);
        assert_eq!(c.neighborhood_type, NeighborhoodType::Commercial);
        assert_eq!(c.status, NeighborhoodStatus::Developing);
    }

    #[test]
    fn test_initiative_new() {
        let i = NeighborhoodInitiative::new("i1", "Title", "Content");
        assert_eq!(i.id, "i1");
    }

    #[test]
    fn test_initiative_builder() {
        let i = NeighborhoodInitiative::new("i1", "Title", "Content")
            .block(1);
        assert_eq!(i.block, 1);
    }

    #[test]
    fn test_initiative_approved() {
        let mut i = NeighborhoodInitiative::new("i1", "Title", "Content");
        i.make_rejected();
        assert!(!i.approved);
        i.make_approved();
        assert!(i.approved);
    }

    #[test]
    fn test_organizer_new() {
        let o = NeighborhoodOrganizer::new("key", "name", "i1");
        assert_eq!(o.initiative_id, "i1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = NeighborhoodStats::default();
        let initiative = NeighborhoodInitiative::new("i1", "Title", "Content");
        s.update(&[initiative], NeighborhoodType::Residential);
        assert_eq!(s.total_initiatives, 1);
        assert_eq!(s.approved, 1);
    }

    #[test]
    fn test_neighborhood_new() {
        let n = SettingsNeighborhood::new(NeighborhoodConfig::default());
        assert_eq!(n.initiative_count(), 0);
    }

    #[test]
    fn test_neighborhood_add_initiative() {
        let mut n = SettingsNeighborhood::new(NeighborhoodConfig::default());
        n.add_initiative(NeighborhoodInitiative::new("i1", "Title", "Content"));
        assert_eq!(n.initiative_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = NeighborhoodRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = NeighborhoodRegistry::new();
        r.register("n1", SettingsNeighborhood::new(NeighborhoodConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_neighborhood_query() {
        assert!(is_neighborhood_query("settings neighborhood"));
        assert!(!is_neighborhood_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = neighborhood_fun_fact();
        assert!(fact.contains("neighborhood"));
    }
}
