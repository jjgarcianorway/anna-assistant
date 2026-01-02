// v0.0.531: LLM Model Registry Tests
// Test cases for model registry functionality

#[cfg(test)]
mod tests {
    use super::super::types::{InstalledBy, ModelCapability, ModelRecord, ModelStatus};
    use super::super::registry::LlmModelRegistry;
    use super::super::utils::{is_model_query, model_fun_fact};

    #[test]
    fn test_model_creation() {
        let model = ModelRecord::new("qwen2.5:3b", ModelCapability::Light, 2.0, 3.0);
        assert_eq!(model.name, "qwen2.5:3b");
        assert_eq!(model.capability, ModelCapability::Light);
        assert_eq!(model.status, ModelStatus::Available);
    }

    #[test]
    fn test_model_install() {
        let mut model = ModelRecord::new("test", ModelCapability::Standard, 5.0, 8.0);
        model.install(InstalledBy::Anna, "2024-01-01");
        assert!(model.is_ready());
        assert_eq!(model.installed_by, InstalledBy::Anna);
    }

    #[test]
    fn test_model_assign() {
        let mut model = ModelRecord::new("test", ModelCapability::Heavy, 14.0, 16.0);
        model.assign("senior-1");
        model.assign("senior-2");
        assert_eq!(model.assigned_specialists.len(), 2);
    }

    #[test]
    fn test_record_use() {
        let mut model = ModelRecord::new("test", ModelCapability::Light, 2.0, 3.0);
        model.install(InstalledBy::User, "ts");
        model.record_use(1000);
        model.record_use(3000);
        assert_eq!(model.usage_count, 2);
        assert_eq!(model.avg_response_ms, 2000);
    }

    #[test]
    fn test_registry_register() {
        let mut registry = LlmModelRegistry::new();
        let model = ModelRecord::new("qwen", ModelCapability::Light, 2.0, 3.0);
        registry.register(model);
        assert_eq!(registry.total(), 1);
    }

    #[test]
    fn test_ready_filter() {
        let mut registry = LlmModelRegistry::new();
        let mut m1 = ModelRecord::new("m1", ModelCapability::Light, 2.0, 3.0);
        m1.install(InstalledBy::User, "ts");
        let m2 = ModelRecord::new("m2", ModelCapability::Light, 2.0, 3.0);
        registry.register(m1);
        registry.register(m2);
        assert_eq!(registry.ready_count(), 1);
    }

    #[test]
    fn test_by_capability() {
        let mut registry = LlmModelRegistry::new();
        let mut m1 = ModelRecord::new("light", ModelCapability::Light, 2.0, 3.0);
        m1.install(InstalledBy::User, "ts");
        let mut m2 = ModelRecord::new("heavy", ModelCapability::Heavy, 14.0, 16.0);
        m2.install(InstalledBy::User, "ts");
        registry.register(m1);
        registry.register(m2);
        assert_eq!(registry.by_capability(ModelCapability::Light).len(), 1);
    }

    #[test]
    fn test_installed_by_anna() {
        let mut registry = LlmModelRegistry::new();
        let mut m1 = ModelRecord::new("m1", ModelCapability::Light, 2.0, 3.0);
        m1.install(InstalledBy::Anna, "ts");
        let mut m2 = ModelRecord::new("m2", ModelCapability::Light, 2.0, 3.0);
        m2.install(InstalledBy::User, "ts");
        registry.register(m1);
        registry.register(m2);
        assert_eq!(registry.installed_by_anna().len(), 1);
    }

    #[test]
    fn test_is_model_query() {
        assert!(is_model_query("Which models are installed?"));
        assert!(is_model_query("Show LLM status"));
        assert!(is_model_query("Check Ollama models"));
        assert!(!is_model_query("Install vim"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = model_fun_fact();
        assert!(fact.contains("billion") || fact.contains("trillion"));
    }
}
