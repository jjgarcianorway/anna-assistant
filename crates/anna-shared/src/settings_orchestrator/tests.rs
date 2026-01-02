// v0.0.574: Orchestrator Tests
// Integration tests for the settings orchestrator

#[cfg(test)]
mod tests {
    use crate::settings_orchestrator::*;
    use crate::unified_settings::SettingsCategory;

    #[test]
    fn test_settings_orchestrator_new() {
        let orchestrator = SettingsOrchestrator::new();
        assert_eq!(orchestrator.state, OrchestratorState::Uninitialized);
    }

    #[test]
    fn test_settings_orchestrator_with_defaults() {
        let orchestrator = SettingsOrchestrator::with_defaults();
        assert_eq!(orchestrator.state, OrchestratorState::Ready);
        assert!(orchestrator.templates.count() > 0);
    }

    #[test]
    fn test_settings_orchestrator_initialize() {
        let mut orchestrator = SettingsOrchestrator::new();
        let result = orchestrator.initialize();
        assert!(result.success);
        assert_eq!(orchestrator.state, OrchestratorState::Ready);
    }

    #[test]
    fn test_settings_orchestrator_set_session() {
        let mut orchestrator = SettingsOrchestrator::with_defaults();
        orchestrator.set_session("test-session");
        assert!(orchestrator.session_id().is_some());
    }

    #[test]
    fn test_settings_orchestrator_change_setting() {
        let mut orchestrator = SettingsOrchestrator::with_defaults();
        let result = orchestrator.change_setting(SettingsCategory::Risk, "level", "high");
        assert!(result.success);
    }

    #[test]
    fn test_settings_orchestrator_status_summary() {
        let orchestrator = SettingsOrchestrator::with_defaults();
        let status = orchestrator.status_summary();
        assert_eq!(status.state, OrchestratorState::Ready);
    }
}
