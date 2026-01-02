// v0.0.603: Settings Router Tests (Phase 179)
// Tests for routing logic

#[cfg(test)]
mod tests {
    use super::super::types::{RouteAction, RouteDef, RouteMatch, RouteType};
    use super::super::routing::{RouteTable, SettingsRouter};
    use super::super::utils::{is_router_query, router_fun_fact};
    use crate::unified_settings::SettingsCategory;

    #[test]
    fn test_route_type_display() {
        assert_eq!(format!("{}", RouteType::Direct), "direct");
        assert_eq!(format!("{}", RouteType::Cached), "cached");
    }

    #[test]
    fn test_route_action_display() {
        assert_eq!(format!("{}", RouteAction::Get), "get");
        assert_eq!(format!("{}", RouteAction::Set), "set");
    }

    #[test]
    fn test_route_def_new() {
        let r = RouteDef::new("r1", RouteType::Direct);
        assert_eq!(r.id, "r1");
        assert!(r.enabled);
    }

    #[test]
    fn test_route_def_builder() {
        let r = RouteDef::new("r1", RouteType::Middleware)
            .name("Main")
            .action(RouteAction::Get)
            .category(SettingsCategory::Personality)
            .priority(50);
        assert!(r.supports(RouteAction::Get));
    }

    #[test]
    fn test_route_match_new() {
        let m = RouteMatch::new("r1", RouteType::Direct, 100);
        assert_eq!(m.score, 100);
    }

    #[test]
    fn test_table_new() {
        let t = RouteTable::new();
        assert_eq!(t.count(), 0);
    }

    #[test]
    fn test_table_add_remove() {
        let mut t = RouteTable::new();
        t.add(RouteDef::new("r1", RouteType::Direct));
        assert_eq!(t.count(), 1);
        t.remove("r1");
        assert_eq!(t.count(), 0);
    }

    #[test]
    fn test_table_find() {
        let mut t = RouteTable::new();
        t.add(RouteDef::new("r1", RouteType::Direct).action(RouteAction::Get));
        let matches = t.find(RouteAction::Get, SettingsCategory::Personality);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_router_new() {
        let r = SettingsRouter::new();
        assert_eq!(r.table_count(), 0);
    }

    #[test]
    fn test_router_add_table() {
        let mut r = SettingsRouter::new();
        r.add_table("test", RouteTable::new());
        assert_eq!(r.table_count(), 1);
    }

    #[test]
    fn test_is_router_query() {
        assert!(is_router_query("settings routing"));
        assert!(!is_router_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = router_fun_fact();
        assert!(fact.contains("routing"));
    }
}
