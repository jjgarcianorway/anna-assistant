// v0.0.565: Settings Profiles (Phase 141)
// Tests

#[cfg(test)]
mod tests {
    use crate::unified_settings::UnifiedSettings;
    use super::super::*;

    fn sample_settings() -> UnifiedSettings {
        UnifiedSettings::default()
    }

    #[test]
    fn test_profile_meta_new() {
        let meta = ProfileMeta::new("Test", "A test profile");
        assert_eq!(meta.name, "Test");
        assert_eq!(meta.description, "A test profile");
        assert!(!meta.is_active);
    }

    #[test]
    fn test_profile_meta_with_tag() {
        let meta = ProfileMeta::new("Test", "desc")
            .with_tag("work")
            .with_tag("dev");
        assert_eq!(meta.tags.len(), 2);
    }

    #[test]
    fn test_settings_profile_new() {
        let settings = sample_settings();
        let profile = SettingsProfile::new("test", "Test", "desc", settings);
        assert_eq!(profile.id, "test");
        assert_eq!(profile.meta.name, "Test");
    }

    #[test]
    fn test_profile_manager_new() {
        let manager = ProfileManager::new();
        assert_eq!(manager.count(), 0);
        assert!(manager.active().is_none());
    }

    #[test]
    fn test_profile_manager_with_default() {
        let settings = sample_settings();
        let manager = ProfileManager::with_default(&settings);
        assert_eq!(manager.count(), 1);
        assert!(manager.active().is_some());
    }

    #[test]
    fn test_profile_manager_add() {
        let mut manager = ProfileManager::new();
        let profile = SettingsProfile::new("test", "Test", "desc", sample_settings());
        manager.add(profile);
        assert_eq!(manager.count(), 1);
    }

    #[test]
    fn test_profile_manager_get() {
        let mut manager = ProfileManager::new();
        let profile = SettingsProfile::new("test", "Test", "desc", sample_settings());
        manager.add(profile);
        assert!(manager.get("test").is_some());
        assert!(manager.get("none").is_none());
    }

    #[test]
    fn test_profile_manager_switch() {
        let mut manager = ProfileManager::new();
        manager.add(SettingsProfile::new("p1", "Profile 1", "desc", sample_settings()));
        manager.add(SettingsProfile::new("p2", "Profile 2", "desc", sample_settings()));

        assert!(manager.switch_to("p2").is_ok());
        assert_eq!(manager.active_id(), Some("p2"));
    }

    #[test]
    fn test_profile_manager_remove_active() {
        let mut manager = ProfileManager::new();
        manager.add(SettingsProfile::new("p1", "Profile 1", "desc", sample_settings()));

        // Can't remove active profile
        assert!(manager.remove("p1").is_none());
    }

    #[test]
    fn test_profile_manager_find_by_name() {
        let mut manager = ProfileManager::new();
        manager.add(SettingsProfile::new("work", "Work Profile", "desc", sample_settings()));
        manager.add(SettingsProfile::new("home", "Home Setup", "desc", sample_settings()));

        let found = manager.find_by_name("work");
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn test_profile_manager_duplicate() {
        let mut manager = ProfileManager::new();
        manager.add(SettingsProfile::new("orig", "Original", "desc", sample_settings()));

        assert!(manager.duplicate("orig", "copy", "Copy").is_ok());
        assert_eq!(manager.count(), 2);
    }

    #[test]
    fn test_format_profiles_list() {
        let manager = ProfileManager::with_default(&sample_settings());
        let output = format_profiles_list(&manager);
        assert!(output.contains("Settings Profiles"));
        assert!(output.contains("Default"));
    }

    #[test]
    fn test_is_profile_query() {
        assert!(is_profile_query("show my profiles"));
        assert!(is_profile_query("create profile"));
        assert!(is_profile_query("switch settings"));
        assert!(!is_profile_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = profiles_fun_fact();
        assert!(fact.contains("profile"));
    }
}
