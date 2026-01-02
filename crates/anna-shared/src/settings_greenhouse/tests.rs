// v0.0.770: Settings Greenhouse - Tests Module
// Unit tests for all greenhouse modules

#[cfg(test)]
mod tests {
    use super::super::types::{GreenhouseType, GreenhouseStatus};
    use super::super::config::GreenhouseConfig;
    use super::super::crop::{GreenhouseCrop, GreenhouseGrower};
    use super::super::stats::GreenhouseStats;
    use super::super::greenhouse::SettingsGreenhouse;
    use super::super::registry::GreenhouseRegistry;
    use super::super::utils::is_greenhouse_query;
    use super::super::utils::greenhouse_fun_fact;

    #[test]
    fn test_greenhouse_type_display() {
        assert_eq!(format!("{}", GreenhouseType::Commercial), "commercial");
        assert_eq!(format!("{}", GreenhouseType::Hobby), "hobby");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", GreenhouseStatus::Active), "active");
        assert_eq!(format!("{}", GreenhouseStatus::Maintenance), "maintenance");
    }

    #[test]
    fn test_config_new() {
        let c = GreenhouseConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = GreenhouseConfig::new("test")
            .greenhouse_type(GreenhouseType::Research)
            .status(GreenhouseStatus::Heating);
        assert_eq!(c.greenhouse_type, GreenhouseType::Research);
        assert_eq!(c.status, GreenhouseStatus::Heating);
    }

    #[test]
    fn test_crop_new() {
        let c = GreenhouseCrop::new("c1", "Title", "Content");
        assert_eq!(c.id, "c1");
    }

    #[test]
    fn test_crop_builder() {
        let c = GreenhouseCrop::new("c1", "Title", "Content")
            .zone(1);
        assert_eq!(c.zone, 1);
    }

    #[test]
    fn test_crop_flourishing() {
        let mut c = GreenhouseCrop::new("c1", "Title", "Content");
        c.make_struggling();
        assert!(!c.flourishing);
        c.make_flourishing();
        assert!(c.flourishing);
    }

    #[test]
    fn test_grower_new() {
        let g = GreenhouseGrower::new("key", "name", "c1");
        assert_eq!(g.crop_id, "c1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = GreenhouseStats::default();
        let crop = GreenhouseCrop::new("c1", "Title", "Content");
        s.update(&[crop], GreenhouseType::Commercial);
        assert_eq!(s.total_crops, 1);
        assert_eq!(s.flourishing, 1);
    }

    #[test]
    fn test_greenhouse_new() {
        let g = SettingsGreenhouse::new(GreenhouseConfig::default());
        assert_eq!(g.crop_count(), 0);
    }

    #[test]
    fn test_greenhouse_add_crop() {
        let mut g = SettingsGreenhouse::new(GreenhouseConfig::default());
        g.add_crop(GreenhouseCrop::new("c1", "Title", "Content"));
        assert_eq!(g.crop_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = GreenhouseRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = GreenhouseRegistry::new();
        r.register("g1", SettingsGreenhouse::new(GreenhouseConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_greenhouse_query() {
        assert!(is_greenhouse_query("settings greenhouse"));
        assert!(!is_greenhouse_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = greenhouse_fun_fact();
        assert!(fact.contains("greenhouse"));
    }
}
