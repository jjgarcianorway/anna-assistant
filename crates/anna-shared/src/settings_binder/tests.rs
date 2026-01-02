// v0.0.652: Settings Binder - Tests
// Test suite for binder

#[cfg(test)]
mod tests {
    use crate::settings_binder::*;

    #[test]
    fn test_binding_type_display() {
        assert_eq!(format!("{}", BindingType::OneWay), "one_way");
        assert_eq!(format!("{}", BindingType::TwoWay), "two_way");
    }

    #[test]
    fn test_binding_state_display() {
        assert_eq!(format!("{}", BindingState::Bound), "bound");
        assert_eq!(format!("{}", BindingState::Unbound), "unbound");
    }

    #[test]
    fn test_binding_def_new() {
        let b = BindingDef::new("src", "dst");
        assert!(!b.is_bound());
    }

    #[test]
    fn test_config_new() {
        let c = BinderConfig::new();
        assert!(c.validate_on_bind);
    }

    #[test]
    fn test_config_builder() {
        let c = BinderConfig::new()
            .default_type(BindingType::TwoWay)
            .auto_bind(true);
        assert_eq!(c.default_type, BindingType::TwoWay);
        assert!(c.auto_bind);
    }

    #[test]
    fn test_result_success() {
        let r = BindingResult::success("src", "dst");
        assert!(r.success);
        assert_eq!(r.state, BindingState::Bound);
    }

    #[test]
    fn test_result_failure() {
        let r = BindingResult::failure("src", "dst", "error");
        assert!(!r.success);
        assert!(r.error.is_some());
    }

    #[test]
    fn test_stats_record() {
        let mut s = BinderStats::default();
        s.record(BindingType::OneWay, true);
        s.record(BindingType::TwoWay, false);
        assert_eq!(s.total_binds, 2);
        assert_eq!(s.successful, 1);
    }

    #[test]
    fn test_binder_new() {
        let b = SettingsBinder::new(BinderConfig::new());
        assert_eq!(b.binding_count(), 0);
    }

    #[test]
    fn test_binder_add_binding() {
        let mut b = SettingsBinder::new(BinderConfig::new());
        b.add_binding(BindingDef::new("src", "dst"));
        assert_eq!(b.binding_count(), 1);
    }

    #[test]
    fn test_binder_bind_all() {
        let mut b = SettingsBinder::new(BinderConfig::new());
        b.add_binding(BindingDef::new("src", "dst"));
        let results = b.bind_all();
        assert_eq!(results.len(), 1);
        assert!(results[0].success);
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsBinderRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsBinderRegistry::new();
        r.register("binder1", SettingsBinder::new(BinderConfig::new()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_binder_query() {
        assert!(is_binder_query("settings binder"));
        assert!(!is_binder_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = binder_fun_fact();
        assert!(fact.contains("binder"));
    }
}
