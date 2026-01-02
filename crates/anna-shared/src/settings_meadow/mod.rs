// v0.0.763: Settings Meadow (Phase 339)
// Grassland meadow for settings grazing

mod types;
mod config;
mod data;
mod stats;
mod meadow;
mod utils;

// Re-export all public items to maintain API compatibility
pub use types::{MeadowType, MeadowStatus};
pub use config::MeadowConfig;
pub use data::{MeadowGrass, MeadowKeeper};
pub use stats::MeadowStats;
pub use meadow::{SettingsMeadow, MeadowRegistry};
pub use utils::{format_meadow_registry, is_meadow_query, meadow_fun_fact};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meadow_type_display() {
        assert_eq!(format!("{}", MeadowType::Hay), "hay");
        assert_eq!(format!("{}", MeadowType::Alpine), "alpine");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", MeadowStatus::Resting), "resting");
        assert_eq!(format!("{}", MeadowStatus::Grazing), "grazing");
    }

    #[test]
    fn test_config_new() {
        let c = MeadowConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = MeadowConfig::new("test")
            .meadow_type(MeadowType::Wildflower)
            .status(MeadowStatus::Mowing);
        assert_eq!(c.meadow_type, MeadowType::Wildflower);
        assert_eq!(c.status, MeadowStatus::Mowing);
    }

    #[test]
    fn test_grass_new() {
        let g = MeadowGrass::new("g1", "Title", "Content");
        assert_eq!(g.id, "g1");
    }

    #[test]
    fn test_grass_builder() {
        let g = MeadowGrass::new("g1", "Title", "Content")
            .sward(1);
        assert_eq!(g.sward, 1);
    }

    #[test]
    fn test_grass_lush() {
        let mut g = MeadowGrass::new("g1", "Title", "Content");
        g.make_sparse();
        assert!(!g.lush);
        g.make_lush();
        assert!(g.lush);
    }

    #[test]
    fn test_keeper_new() {
        let k = MeadowKeeper::new("key", "name", "g1");
        assert_eq!(k.grass_id, "g1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = MeadowStats::default();
        let grass = MeadowGrass::new("g1", "Title", "Content");
        s.update(&[grass], MeadowType::Hay);
        assert_eq!(s.total_grasses, 1);
        assert_eq!(s.lush, 1);
    }

    #[test]
    fn test_meadow_new() {
        let m = SettingsMeadow::new(MeadowConfig::default());
        assert_eq!(m.grass_count(), 0);
    }

    #[test]
    fn test_meadow_add_grass() {
        let mut m = SettingsMeadow::new(MeadowConfig::default());
        m.add_grass(MeadowGrass::new("g1", "Title", "Content"));
        assert_eq!(m.grass_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = MeadowRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = MeadowRegistry::new();
        r.register("m1", SettingsMeadow::new(MeadowConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_meadow_query() {
        assert!(is_meadow_query("settings meadow"));
        assert!(!is_meadow_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = meadow_fun_fact();
        assert!(fact.contains("meadow"));
    }
}
