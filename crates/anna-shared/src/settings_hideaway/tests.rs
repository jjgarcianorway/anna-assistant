// v0.0.786: Settings Hideaway (Phase 362)
// Tests

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_hideaway_type_display() {
        assert_eq!(format!("{}", HideawayType::Secret), "secret");
        assert_eq!(format!("{}", HideawayType::Private), "private");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", HideawayStatus::Secluded), "secluded");
        assert_eq!(format!("{}", HideawayStatus::Isolated), "isolated");
    }

    #[test]
    fn test_config_new() {
        let c = HideawayConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = HideawayConfig::new("test")
            .hideaway_type(HideawayType::Private)
            .status(HideawayStatus::Concealed);
        assert_eq!(c.hideaway_type, HideawayType::Private);
        assert_eq!(c.status, HideawayStatus::Concealed);
    }

    #[test]
    fn test_occupant_new() {
        let o = HideawayOccupant::new("o1", "Title", "Content");
        assert_eq!(o.id, "o1");
    }

    #[test]
    fn test_occupant_builder() {
        let o = HideawayOccupant::new("o1", "Title", "Content")
            .nook(1);
        assert_eq!(o.nook, 1);
    }

    #[test]
    fn test_occupant_visibility() {
        let mut o = HideawayOccupant::new("o1", "Title", "Content");
        o.make_visible();
        assert!(!o.hidden);
        o.make_hidden();
        assert!(o.hidden);
    }

    #[test]
    fn test_guardian_new() {
        let g = HideawayGuardian::new("key", "name", "o1");
        assert_eq!(g.occupant_id, "o1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = HideawayStats::default();
        let occupant = HideawayOccupant::new("o1", "Title", "Content");
        s.update(&[occupant], HideawayType::Secret);
        assert_eq!(s.total_occupants, 1);
        assert_eq!(s.hidden, 1);
    }

    #[test]
    fn test_hideaway_new() {
        let h = SettingsHideaway::new(HideawayConfig::default());
        assert_eq!(h.occupant_count(), 0);
    }

    #[test]
    fn test_hideaway_add_occupant() {
        let mut h = SettingsHideaway::new(HideawayConfig::default());
        h.add_occupant(HideawayOccupant::new("o1", "Title", "Content"));
        assert_eq!(h.occupant_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = HideawayRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = HideawayRegistry::new();
        r.register("h1", SettingsHideaway::new(HideawayConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_hideaway_query() {
        assert!(is_hideaway_query("settings hideaway"));
        assert!(!is_hideaway_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = hideaway_fun_fact();
        assert!(fact.contains("hideaway"));
    }
}
