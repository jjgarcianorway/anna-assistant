// v0.0.569: Settings Templates - Tests (Phase 145)
// Test suite for settings templates functionality

#[cfg(test)]
mod tests {
    use crate::settings_templates::*;
    use crate::unified_settings::{SettingsCategory, UnifiedSettings};

    #[test]
    fn test_template_scope_display() {
        assert_eq!(format!("{}", TemplateScope::Full), "Full");
        assert_eq!(format!("{}", TemplateScope::Partial), "Partial");
    }

    #[test]
    fn test_template_use_case_display() {
        assert_eq!(format!("{}", TemplateUseCase::Development), "Development");
        assert_eq!(format!("{}", TemplateUseCase::Production), "Production");
    }

    #[test]
    fn test_template_meta_new() {
        let meta = TemplateMeta::new("Test", "Test template");
        assert_eq!(meta.name, "Test");
        assert_eq!(meta.use_case, TemplateUseCase::Custom);
    }

    #[test]
    fn test_template_meta_builder() {
        let meta = TemplateMeta::new("Test", "Test")
            .with_use_case(TemplateUseCase::Development)
            .with_tag("dev");
        assert_eq!(meta.use_case, TemplateUseCase::Development);
        assert!(meta.tags.contains(&"dev".to_string()));
    }

    #[test]
    fn test_settings_template_new() {
        let meta = TemplateMeta::new("Test", "Test");
        let template = SettingsTemplate::new(1, meta, UnifiedSettings::default());
        assert_eq!(template.id, 1);
        assert!(!template.builtin);
    }

    #[test]
    fn test_settings_template_builtin() {
        let meta = TemplateMeta::new("Test", "Test");
        let template = SettingsTemplate::new(1, meta, UnifiedSettings::default()).builtin();
        assert!(template.builtin);
    }

    #[test]
    fn test_settings_template_includes_category() {
        let meta = TemplateMeta::new("Test", "Test").with_scope(TemplateScope::Full);
        let template = SettingsTemplate::new(1, meta, UnifiedSettings::default());
        assert!(template.includes_category(SettingsCategory::Personality));
    }

    #[test]
    fn test_template_manager_new() {
        let manager = TemplateManager::new();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_template_manager_with_defaults() {
        let manager = TemplateManager::with_defaults();
        assert!(manager.count() >= 4);
    }

    #[test]
    fn test_template_manager_add() {
        let mut manager = TemplateManager::new();
        let meta = TemplateMeta::new("Test", "Test");
        let id = manager.add(meta, UnifiedSettings::default());
        assert_eq!(manager.count(), 1);
        assert!(manager.get(id).is_some());
    }

    #[test]
    fn test_template_manager_remove() {
        let mut manager = TemplateManager::new();
        let meta = TemplateMeta::new("Test", "Test");
        let id = manager.add(meta, UnifiedSettings::default());
        assert!(manager.remove(id).is_some());
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_template_manager_find_by_name() {
        let manager = TemplateManager::with_defaults();
        let found = manager.find_by_name("dev");
        assert!(!found.is_empty());
    }

    #[test]
    fn test_template_manager_find_by_use_case() {
        let manager = TemplateManager::with_defaults();
        let found = manager.find_by_use_case(TemplateUseCase::Development);
        assert!(!found.is_empty());
    }

    #[test]
    fn test_format_templates() {
        let manager = TemplateManager::with_defaults();
        let output = format_templates(&manager);
        assert!(output.contains("Templates"));
    }

    #[test]
    fn test_is_template_query() {
        assert!(is_template_query("create a template"));
        assert!(is_template_query("apply template"));
        assert!(!is_template_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = template_fun_fact();
        assert!(fact.contains("template"));
    }
}
