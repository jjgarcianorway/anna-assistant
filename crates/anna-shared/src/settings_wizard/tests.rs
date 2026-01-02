// v0.0.572: Settings Wizard Tests
// Comprehensive tests for wizard functionality

#[cfg(test)]
mod tests {
    use crate::settings_wizard::types::{WizardChoice, WizardState, WizardStep, WizardStepType};
    use crate::settings_wizard::manager::WizardManager;
    use crate::settings_wizard::wizard::SettingsWizard;
    use crate::settings_wizard::utils::{format_wizard, is_wizard_query, wizard_fun_fact};

    #[test]
    fn test_wizard_step_type_display() {
        assert_eq!(format!("{}", WizardStepType::Welcome), "Welcome");
        assert_eq!(format!("{}", WizardStepType::SingleChoice), "Single Choice");
    }

    #[test]
    fn test_wizard_choice_new() {
        let choice = WizardChoice::new("id", "Label", "value");
        assert_eq!(choice.id, "id");
        assert_eq!(choice.label, "Label");
    }

    #[test]
    fn test_wizard_step_new() {
        let step = WizardStep::new("id", WizardStepType::Welcome, "Title", "Desc");
        assert_eq!(step.id, "id");
        assert!(step.required);
    }

    #[test]
    fn test_wizard_state_display() {
        assert_eq!(format!("{}", WizardState::InProgress), "In Progress");
        assert_eq!(format!("{}", WizardState::Completed), "Completed");
    }

    #[test]
    fn test_settings_wizard_new() {
        let wizard = SettingsWizard::new("id", "Name", "Description");
        assert_eq!(wizard.id, "id");
        assert_eq!(wizard.state, WizardState::NotStarted);
    }

    #[test]
    fn test_settings_wizard_start() {
        let mut wizard = SettingsWizard::new("id", "Name", "Desc")
            .add_step(WizardStep::new("s1", WizardStepType::Welcome, "T", "D"));
        wizard.start();
        assert_eq!(wizard.state, WizardState::InProgress);
    }

    #[test]
    fn test_settings_wizard_progress() {
        let mut wizard = SettingsWizard::new("id", "Name", "Desc")
            .add_step(WizardStep::new("s1", WizardStepType::Welcome, "T", "D"))
            .add_step(WizardStep::new("s2", WizardStepType::Complete, "T", "D"));
        wizard.start();
        assert_eq!(wizard.progress(), 0.0);
        wizard.answer("test");
        wizard.next();
        assert_eq!(wizard.progress(), 0.5);
    }

    #[test]
    fn test_settings_wizard_navigation() {
        let mut wizard = SettingsWizard::new("id", "Name", "Desc")
            .add_step(WizardStep::new("s1", WizardStepType::Welcome, "T", "D"))
            .add_step(WizardStep::new("s2", WizardStepType::Complete, "T", "D"));
        wizard.start();
        wizard.answer("test");
        wizard.next();
        assert!(wizard.back());
        assert_eq!(wizard.current_step, 0);
    }

    #[test]
    fn test_wizard_manager_new() {
        let manager = WizardManager::new();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_wizard_manager_with_defaults() {
        let manager = WizardManager::with_defaults();
        assert!(manager.count() >= 2);
    }

    #[test]
    fn test_wizard_manager_start() {
        let mut manager = WizardManager::with_defaults();
        assert!(manager.start("quick_setup"));
        assert!(manager.active().is_some());
    }

    #[test]
    fn test_format_wizard() {
        let wizard = SettingsWizard::new("id", "Test Wizard", "Description")
            .add_step(WizardStep::new("s1", WizardStepType::Welcome, "Welcome", "Hi"));
        let output = format_wizard(&wizard);
        assert!(output.contains("Test Wizard"));
    }

    #[test]
    fn test_is_wizard_query() {
        assert!(is_wizard_query("setup wizard"));
        assert!(is_wizard_query("get started"));
        assert!(!is_wizard_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = wizard_fun_fact();
        assert!(fact.contains("wizard"));
    }
}
