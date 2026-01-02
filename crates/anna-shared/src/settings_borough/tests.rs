// v0.0.751: Settings Borough Tests
// Test suite for borough system

#[cfg(test)]
mod tests {
    use crate::settings_borough::*;

    #[test]
    fn test_borough_type_display() {
        assert_eq!(format!("{}", BoroughType::Urban), "urban");
        assert_eq!(format!("{}", BoroughType::Metropolitan), "metropolitan");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", BoroughStatus::Established), "established");
        assert_eq!(format!("{}", BoroughStatus::Active), "active");
    }

    #[test]
    fn test_config_new() {
        let c = BoroughConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = BoroughConfig::new("test")
            .borough_type(BoroughType::London)
            .status(BoroughStatus::Reformed);
        assert_eq!(c.borough_type, BoroughType::London);
        assert_eq!(c.status, BoroughStatus::Reformed);
    }

    #[test]
    fn test_resolution_new() {
        let r = BoroughResolution::new("r1", "Title", "Content");
        assert_eq!(r.id, "r1");
    }

    #[test]
    fn test_resolution_builder() {
        let r = BoroughResolution::new("r1", "Title", "Content")
            .section(1);
        assert_eq!(r.section, 1);
    }

    #[test]
    fn test_resolution_adopted() {
        let mut r = BoroughResolution::new("r1", "Title", "Content");
        r.make_rescinded();
        assert!(!r.adopted);
        r.make_adopted();
        assert!(r.adopted);
    }

    #[test]
    fn test_representative_new() {
        let r = BoroughRepresentative::new("key", "name", "r1");
        assert_eq!(r.resolution_id, "r1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = BoroughStats::default();
        let resolution = BoroughResolution::new("r1", "Title", "Content");
        s.update(&[resolution], BoroughType::Urban);
        assert_eq!(s.total_resolutions, 1);
        assert_eq!(s.adopted, 1);
    }

    #[test]
    fn test_borough_new() {
        let b = SettingsBorough::new(BoroughConfig::default());
        assert_eq!(b.resolution_count(), 0);
    }

    #[test]
    fn test_borough_add_resolution() {
        let mut b = SettingsBorough::new(BoroughConfig::default());
        b.add_resolution(BoroughResolution::new("r1", "Title", "Content"));
        assert_eq!(b.resolution_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = BoroughRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = BoroughRegistry::new();
        r.register("b1", SettingsBorough::new(BoroughConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_borough_query() {
        assert!(is_borough_query("settings borough"));
        assert!(!is_borough_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = borough_fun_fact();
        assert!(fact.contains("borough"));
    }
}
