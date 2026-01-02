// v0.0.698: Settings Portfolio (Phase 274)
// Investment portfolio of settings assets - Tests

#[cfg(test)]
mod tests {
    use crate::settings_portfolio::*;

    #[test]
    fn test_portfolio_type_display() {
        assert_eq!(format!("{}", PortfolioType::Standard), "standard");
        assert_eq!(format!("{}", PortfolioType::Growth), "growth");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", PortfolioStatus::Active), "active");
        assert_eq!(format!("{}", PortfolioStatus::Rebalancing), "rebalancing");
    }

    #[test]
    fn test_config_new() {
        let c = PortfolioConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = PortfolioConfig::new("test")
            .portfolio_type(PortfolioType::Balanced)
            .max_assets(50);
        assert_eq!(c.portfolio_type, PortfolioType::Balanced);
        assert_eq!(c.max_assets, 50);
    }

    #[test]
    fn test_asset_new() {
        let a = PortfolioAsset::new("a1", "Asset 1", 100.0);
        assert_eq!(a.value, 100.0);
    }

    #[test]
    fn test_asset_builder() {
        let a = PortfolioAsset::new("a1", "Asset 1", 100.0)
            .weight(25.0)
            .category("config");
        assert_eq!(a.weight, 25.0);
        assert_eq!(a.category, "config");
    }

    #[test]
    fn test_holding_new() {
        let h = PortfolioHolding::new("key", "value", "a1");
        assert_eq!(h.asset_id, "a1");
    }

    #[test]
    fn test_holding_quantity() {
        let h = PortfolioHolding::new("key", "value", "a1").quantity(5);
        assert_eq!(h.quantity, 5);
    }

    #[test]
    fn test_stats_update() {
        let mut s = PortfolioStats::default();
        let assets = vec![PortfolioAsset::new("a1", "Asset", 100.0)];
        s.update(&assets);
        assert_eq!(s.total_assets, 1);
        assert_eq!(s.total_value, 100.0);
    }

    #[test]
    fn test_portfolio_new() {
        let p = SettingsPortfolio::new(PortfolioConfig::default());
        assert_eq!(p.asset_count(), 0);
    }

    #[test]
    fn test_portfolio_add_asset() {
        let mut p = SettingsPortfolio::new(PortfolioConfig::default());
        p.add_asset(PortfolioAsset::new("a1", "Asset 1", 100.0));
        assert_eq!(p.asset_count(), 1);
    }

    #[test]
    fn test_portfolio_rebalance() {
        let mut p = SettingsPortfolio::new(PortfolioConfig::default());
        p.add_asset(PortfolioAsset::new("a1", "Asset 1", 100.0));
        p.rebalance();
        assert_eq!(p.status(), PortfolioStatus::Active);
    }

    #[test]
    fn test_portfolio_pause() {
        let mut p = SettingsPortfolio::new(PortfolioConfig::default());
        p.pause();
        assert_eq!(p.status(), PortfolioStatus::Paused);
    }

    #[test]
    fn test_registry_new() {
        let r = PortfolioRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = PortfolioRegistry::new();
        r.register("p1", SettingsPortfolio::new(PortfolioConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_portfolio_query() {
        assert!(is_portfolio_query("settings portfolio"));
        assert!(!is_portfolio_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = portfolio_fun_fact();
        assert!(fact.contains("portfolio"));
    }
}
