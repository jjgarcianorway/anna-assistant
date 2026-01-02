// v0.0.768: Settings Garden (Phase 344)
// Tests for settings garden

#[cfg(test)]
mod tests {
    use crate::settings_garden::*;

    #[test]
    fn test_garden_type_display() {
        assert_eq!(format!("{}", GardenType::Flower), "flower");
        assert_eq!(format!("{}", GardenType::Vegetable), "vegetable");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", GardenStatus::Planned), "planned");
        assert_eq!(format!("{}", GardenStatus::Blooming), "blooming");
    }

    #[test]
    fn test_config_new() {
        let c = GardenConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = GardenConfig::new("test")
            .garden_type(GardenType::Herb)
            .status(GardenStatus::Growing);
        assert_eq!(c.garden_type, GardenType::Herb);
        assert_eq!(c.status, GardenStatus::Growing);
    }

    #[test]
    fn test_plant_new() {
        let p = GardenPlant::new("p1", "Title", "Content");
        assert_eq!(p.id, "p1");
    }

    #[test]
    fn test_plant_builder() {
        let p = GardenPlant::new("p1", "Title", "Content")
            .bed(1);
        assert_eq!(p.bed, 1);
    }

    #[test]
    fn test_plant_thriving() {
        let mut p = GardenPlant::new("p1", "Title", "Content");
        p.make_wilting();
        assert!(!p.thriving);
        p.make_thriving();
        assert!(p.thriving);
    }

    #[test]
    fn test_gardener_new() {
        let g = GardenGardener::new("key", "name", "p1");
        assert_eq!(g.plant_id, "p1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = GardenStats::default();
        let plant = GardenPlant::new("p1", "Title", "Content");
        s.update(&[plant], GardenType::Flower);
        assert_eq!(s.total_plants, 1);
        assert_eq!(s.thriving, 1);
    }

    #[test]
    fn test_garden_new() {
        let g = SettingsGarden::new(GardenConfig::default());
        assert_eq!(g.plant_count(), 0);
    }

    #[test]
    fn test_garden_add_plant() {
        let mut g = SettingsGarden::new(GardenConfig::default());
        g.add_plant(GardenPlant::new("p1", "Title", "Content"));
        assert_eq!(g.plant_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = GardenRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = GardenRegistry::new();
        r.register("g1", SettingsGarden::new(GardenConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_garden_query() {
        assert!(is_garden_query("settings garden"));
        assert!(!is_garden_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = garden_fun_fact();
        assert!(fact.contains("garden"));
    }
}
