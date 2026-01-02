// v0.0.572: Settings Wizard Manager
// Manages multiple wizards and default wizard definitions

use crate::unified_settings::SettingsCategory;

use super::types::{WizardChoice, WizardStep, WizardStepType};
use super::wizard::SettingsWizard;

/// Wizard manager
#[derive(Debug, Clone, Default)]
pub struct WizardManager {
    /// Available wizards
    wizards: Vec<SettingsWizard>,
    /// Active wizard ID
    active: Option<String>,
}

impl WizardManager {
    /// Create new manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with default wizards
    pub fn with_defaults() -> Self {
        let mut mgr = Self::new();
        mgr.add_default_wizards();
        mgr
    }

    /// Add default wizards
    fn add_default_wizards(&mut self) {
        // Quick setup wizard
        let quick_setup = SettingsWizard::new(
            "quick_setup",
            "Quick Setup",
            "Configure basic settings quickly",
        )
        .add_step(
            WizardStep::new("welcome", WizardStepType::Welcome, "Welcome to Anna",
                "Let's configure Anna to work best for you.")
        )
        .add_step(
            WizardStep::new("experience", WizardStepType::SingleChoice, "Experience Level",
                "How familiar are you with Linux?")
                .with_category(SettingsCategory::Learning)
                .add_choice(WizardChoice::new("beginner", "Beginner", "basic"))
                .add_choice(WizardChoice::new("intermediate", "Intermediate", "intermediate"))
                .add_choice(WizardChoice::new("advanced", "Advanced", "advanced"))
        )
        .add_step(
            WizardStep::new("risk", WizardStepType::SingleChoice, "Risk Tolerance",
                "How should Anna handle potentially risky operations?")
                .with_category(SettingsCategory::Risk)
                .add_choice(WizardChoice::new("cautious", "Ask for everything", "none"))
                .add_choice(WizardChoice::new("balanced", "Ask for risky operations", "low"))
                .add_choice(WizardChoice::new("trusting", "Only ask for dangerous operations", "medium"))
        )
        .add_step(
            WizardStep::new("complete", WizardStepType::Complete, "Setup Complete",
                "Anna is configured and ready to help!")
        );

        self.wizards.push(quick_setup);

        // Privacy wizard
        let privacy_wizard = SettingsWizard::new(
            "privacy",
            "Privacy Setup",
            "Configure privacy and data handling",
        )
        .add_step(
            WizardStep::new("intro", WizardStepType::Welcome, "Privacy Settings",
                "Configure how Anna handles your data.")
        )
        .add_step(
            WizardStep::new("history", WizardStepType::SingleChoice, "Command History",
                "How long should Anna remember your commands?")
                .with_category(SettingsCategory::Privacy)
                .add_choice(WizardChoice::new("none", "Don't remember", "0"))
                .add_choice(WizardChoice::new("session", "This session only", "session"))
                .add_choice(WizardChoice::new("week", "One week", "week"))
                .add_choice(WizardChoice::new("forever", "Forever", "forever"))
        )
        .add_step(
            WizardStep::new("complete", WizardStepType::Complete, "Privacy Configured",
                "Your privacy settings have been saved.")
        );

        self.wizards.push(privacy_wizard);
    }

    /// Add a wizard
    pub fn add(&mut self, wizard: SettingsWizard) {
        self.wizards.push(wizard);
    }

    /// Get wizard by ID
    pub fn get(&self, id: &str) -> Option<&SettingsWizard> {
        self.wizards.iter().find(|w| w.id == id)
    }

    /// Get mutable wizard
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsWizard> {
        self.wizards.iter_mut().find(|w| w.id == id)
    }

    /// List all wizards
    pub fn list(&self) -> &[SettingsWizard] {
        &self.wizards
    }

    /// Start a wizard
    pub fn start(&mut self, id: &str) -> bool {
        if let Some(wizard) = self.wizards.iter_mut().find(|w| w.id == id) {
            wizard.start();
            self.active = Some(id.to_string());
            true
        } else {
            false
        }
    }

    /// Get active wizard
    pub fn active(&self) -> Option<&SettingsWizard> {
        self.active.as_ref().and_then(|id| self.get(id))
    }

    /// Get active wizard mut
    pub fn active_mut(&mut self) -> Option<&mut SettingsWizard> {
        let id = self.active.clone()?;
        self.get_mut(&id)
    }

    /// Count wizards
    pub fn count(&self) -> usize {
        self.wizards.len()
    }
}
