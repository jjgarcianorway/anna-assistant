// v0.0.756: Settings Lot Tests (Phase 332)
// Unit tests for settings lot

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_lot_type_display() {
        assert_eq!(format!("{}", LotType::Residential), "residential");
        assert_eq!(format!("{}", LotType::Commercial), "commercial");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", LotStatus::Vacant), "vacant");
        assert_eq!(format!("{}", LotStatus::Improved), "improved");
    }

    #[test]
    fn test_config_new() {
        let c = LotConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = LotConfig::new("test")
            .lot_type(LotType::Commercial)
            .status(LotStatus::Subdivided);
        assert_eq!(c.lot_type, LotType::Commercial);
        assert_eq!(c.status, LotStatus::Subdivided);
    }

    #[test]
    fn test_deed_new() {
        let d = LotDeed::new("d1", "Title", "Content");
        assert_eq!(d.id, "d1");
    }

    #[test]
    fn test_deed_builder() {
        let d = LotDeed::new("d1", "Title", "Content")
            .parcel(1);
        assert_eq!(d.parcel, 1);
    }

    #[test]
    fn test_deed_registered() {
        let mut d = LotDeed::new("d1", "Title", "Content");
        d.make_unregistered();
        assert!(!d.registered);
        d.make_registered();
        assert!(d.registered);
    }

    #[test]
    fn test_assessor_new() {
        let a = LotAssessor::new("key", "name", "d1");
        assert_eq!(a.deed_id, "d1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = LotStats::default();
        let deed = LotDeed::new("d1", "Title", "Content");
        s.update(&[deed], LotType::Residential);
        assert_eq!(s.total_deeds, 1);
        assert_eq!(s.registered, 1);
    }

    #[test]
    fn test_lot_new() {
        let l = SettingsLot::new(LotConfig::default());
        assert_eq!(l.deed_count(), 0);
    }

    #[test]
    fn test_lot_add_deed() {
        let mut l = SettingsLot::new(LotConfig::default());
        l.add_deed(LotDeed::new("d1", "Title", "Content"));
        assert_eq!(l.deed_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = LotRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = LotRegistry::new();
        r.register("l1", SettingsLot::new(LotConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_lot_query() {
        assert!(is_lot_query("settings lot"));
        assert!(!is_lot_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = lot_fun_fact();
        assert!(fact.contains("lot"));
    }
}
