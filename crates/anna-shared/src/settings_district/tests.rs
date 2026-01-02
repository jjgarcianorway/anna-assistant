// v0.0.748: Settings District Tests (Phase 324)
// Test suite for settings district module

#[cfg(test)]
mod tests {
    use crate::settings_district::*;

    #[test]
    fn test_district_type_display() {
        assert_eq!(format!("{}", DistrictType::Urban), "urban");
        assert_eq!(format!("{}", DistrictType::Rural), "rural");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", DistrictStatus::Planned), "planned");
        assert_eq!(format!("{}", DistrictStatus::Operational), "operational");
    }

    #[test]
    fn test_config_new() {
        let c = DistrictConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = DistrictConfig::new("test")
            .district_type(DistrictType::Industrial)
            .status(DistrictStatus::Developing);
        assert_eq!(c.district_type, DistrictType::Industrial);
        assert_eq!(c.status, DistrictStatus::Developing);
    }

    #[test]
    fn test_bylaw_new() {
        let b = DistrictBylaw::new("b1", "Title", "Content");
        assert_eq!(b.id, "b1");
    }

    #[test]
    fn test_bylaw_builder() {
        let b = DistrictBylaw::new("b1", "Title", "Content")
            .ward(1);
        assert_eq!(b.ward, 1);
    }

    #[test]
    fn test_bylaw_active() {
        let mut b = DistrictBylaw::new("b1", "Title", "Content");
        b.make_inactive();
        assert!(!b.active);
        b.make_active();
        assert!(b.active);
    }

    #[test]
    fn test_official_new() {
        let o = DistrictOfficial::new("key", "name", "b1");
        assert_eq!(o.bylaw_id, "b1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = DistrictStats::default();
        let bylaw = DistrictBylaw::new("b1", "Title", "Content");
        s.update(&[bylaw], DistrictType::Urban);
        assert_eq!(s.total_bylaws, 1);
        assert_eq!(s.active, 1);
    }

    #[test]
    fn test_district_new() {
        let d = SettingsDistrict::new(DistrictConfig::default());
        assert_eq!(d.bylaw_count(), 0);
    }

    #[test]
    fn test_district_add_bylaw() {
        let mut d = SettingsDistrict::new(DistrictConfig::default());
        d.add_bylaw(DistrictBylaw::new("b1", "Title", "Content"));
        assert_eq!(d.bylaw_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = DistrictRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = DistrictRegistry::new();
        r.register("d1", SettingsDistrict::new(DistrictConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_district_query() {
        assert!(is_district_query("settings district"));
        assert!(!is_district_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = district_fun_fact();
        assert!(fact.contains("district"));
    }
}
