// v0.0.755: Settings Block (Phase 331)
// Tests

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_block_type_display() {
        assert_eq!(format!("{}", BlockType::Residential), "residential");
        assert_eq!(format!("{}", BlockType::Commercial), "commercial");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", BlockStatus::Surveyed), "surveyed");
        assert_eq!(format!("{}", BlockStatus::Developed), "developed");
    }

    #[test]
    fn test_config_new() {
        let c = BlockConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = BlockConfig::new("test")
            .block_type(BlockType::Commercial)
            .status(BlockStatus::Subdivided);
        assert_eq!(c.block_type, BlockType::Commercial);
        assert_eq!(c.status, BlockStatus::Subdivided);
    }

    #[test]
    fn test_plat_new() {
        let p = BlockPlat::new("p1", "Title", "Content");
        assert_eq!(p.id, "p1");
    }

    #[test]
    fn test_plat_builder() {
        let p = BlockPlat::new("p1", "Title", "Content")
            .lot(1);
        assert_eq!(p.lot, 1);
    }

    #[test]
    fn test_plat_recorded() {
        let mut p = BlockPlat::new("p1", "Title", "Content");
        p.make_pending();
        assert!(!p.recorded);
        p.make_recorded();
        assert!(p.recorded);
    }

    #[test]
    fn test_surveyor_new() {
        let s = BlockSurveyor::new("key", "name", "p1");
        assert_eq!(s.plat_id, "p1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = BlockStats::default();
        let plat = BlockPlat::new("p1", "Title", "Content");
        s.update(&[plat], BlockType::Residential);
        assert_eq!(s.total_plats, 1);
        assert_eq!(s.recorded, 1);
    }

    #[test]
    fn test_block_new() {
        let b = SettingsBlock::new(BlockConfig::default());
        assert_eq!(b.plat_count(), 0);
    }

    #[test]
    fn test_block_add_plat() {
        let mut b = SettingsBlock::new(BlockConfig::default());
        b.add_plat(BlockPlat::new("p1", "Title", "Content"));
        assert_eq!(b.plat_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = BlockRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = BlockRegistry::new();
        r.register("b1", SettingsBlock::new(BlockConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_block_query() {
        assert!(is_block_query("settings block"));
        assert!(!is_block_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = block_fun_fact();
        assert!(fact.contains("block"));
    }
}
