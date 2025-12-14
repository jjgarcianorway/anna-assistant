// v0.0.572: Settings Wizard (Phase 148)
// Guided interactive settings configuration

use serde::{Deserialize, Serialize};

use crate::unified_settings::{SettingsCategory, UnifiedSettings};

/// Wizard step type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WizardStepType {
    /// Welcome/introduction
    Welcome,
    /// Single choice question
    SingleChoice,
    /// Multiple choice question
    MultipleChoice,
    /// Text input
    TextInput,
    /// Numeric input
    NumericInput,
    /// Confirmation step
    Confirm,
    /// Summary/review step
    Summary,
    /// Completion step
    Complete,
}

impl std::fmt::Display for WizardStepType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Welcome => write!(f, "Welcome"),
            Self::SingleChoice => write!(f, "Single Choice"),
            Self::MultipleChoice => write!(f, "Multiple Choice"),
            Self::TextInput => write!(f, "Text Input"),
            Self::NumericInput => write!(f, "Numeric Input"),
            Self::Confirm => write!(f, "Confirm"),
            Self::Summary => write!(f, "Summary"),
            Self::Complete => write!(f, "Complete"),
        }
    }
}

/// A wizard choice option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WizardChoice {
    /// Choice ID
    pub id: String,
    /// Display label
    pub label: String,
    /// Description
    pub description: Option<String>,
    /// Value to apply
    pub value: String,
}

impl WizardChoice {
    /// Create new choice
    pub fn new(id: impl Into<String>, label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            description: None,
            value: value.into(),
        }
    }

    /// Add description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// A wizard step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WizardStep {
    /// Step ID
    pub id: String,
    /// Step type
    pub step_type: WizardStepType,
    /// Title
    pub title: String,
    /// Description/question
    pub description: String,
    /// Category affected
    pub category: Option<SettingsCategory>,
    /// Field affected
    pub field: Option<String>,
    /// Choices (for choice types)
    pub choices: Vec<WizardChoice>,
    /// Default value
    pub default: Option<String>,
    /// Is required
    pub required: bool,
    /// Skip condition (depends on previous answer)
    pub skip_if: Option<String>,
}

impl WizardStep {
    /// Create new step
    pub fn new(
        id: impl Into<String>,
        step_type: WizardStepType,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            step_type,
            title: title.into(),
            description: description.into(),
            category: None,
            field: None,
            choices: Vec::new(),
            default: None,
            required: true,
            skip_if: None,
        }
    }

    /// Set category
    pub fn with_category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set field
    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }

    /// Add choice
    pub fn add_choice(mut self, choice: WizardChoice) -> Self {
        self.choices.push(choice);
        self
    }

    /// Set default
    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self
    }

    /// Set as optional
    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }
}

/// Wizard answer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WizardAnswer {
    /// Step ID
    pub step_id: String,
    /// Answer value
    pub value: String,
    /// Selected choice IDs (for multi-choice)
    pub selected: Vec<String>,
}

/// Wizard state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WizardState {
    /// Not started
    #[default]
    NotStarted,
    /// In progress
    InProgress,
    /// Completed
    Completed,
    /// Cancelled
    Cancelled,
}

impl std::fmt::Display for WizardState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotStarted => write!(f, "Not Started"),
            Self::InProgress => write!(f, "In Progress"),
            Self::Completed => write!(f, "Completed"),
            Self::Cancelled => write!(f, "Cancelled"),
        }
    }
}

/// A settings wizard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsWizard {
    /// Wizard ID
    pub id: String,
    /// Name
    pub name: String,
    /// Description
    pub description: String,
    /// Steps
    pub steps: Vec<WizardStep>,
    /// Current step index
    pub current_step: usize,
    /// Collected answers
    pub answers: Vec<WizardAnswer>,
    /// State
    pub state: WizardState,
    /// Settings being built
    #[serde(skip)]
    pub settings: Option<UnifiedSettings>,
}

impl SettingsWizard {
    /// Create new wizard
    pub fn new(id: impl Into<String>, name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            steps: Vec::new(),
            current_step: 0,
            answers: Vec::new(),
            state: WizardState::NotStarted,
            settings: None,
        }
    }

    /// Add step
    pub fn add_step(mut self, step: WizardStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Start the wizard
    pub fn start(&mut self) {
        self.current_step = 0;
        self.answers.clear();
        self.state = WizardState::InProgress;
        self.settings = Some(UnifiedSettings::default());
    }

    /// Get current step
    pub fn current(&self) -> Option<&WizardStep> {
        self.steps.get(self.current_step)
    }

    /// Submit answer for current step
    pub fn answer(&mut self, value: impl Into<String>) -> bool {
        if self.state != WizardState::InProgress {
            return false;
        }

        if let Some(step) = self.steps.get(self.current_step) {
            self.answers.push(WizardAnswer {
                step_id: step.id.clone(),
                value: value.into(),
                selected: Vec::new(),
            });
            true
        } else {
            false
        }
    }

    /// Move to next step
    pub fn next(&mut self) -> bool {
        if self.state != WizardState::InProgress {
            return false;
        }

        if self.current_step + 1 < self.steps.len() {
            self.current_step += 1;
            true
        } else {
            self.state = WizardState::Completed;
            false
        }
    }

    /// Move to previous step
    pub fn back(&mut self) -> bool {
        if self.state != WizardState::InProgress || self.current_step == 0 {
            return false;
        }
        self.current_step -= 1;
        // Remove last answer
        if !self.answers.is_empty() {
            self.answers.pop();
        }
        true
    }

    /// Cancel wizard
    pub fn cancel(&mut self) {
        self.state = WizardState::Cancelled;
    }

    /// Get progress (0.0 - 1.0)
    pub fn progress(&self) -> f32 {
        if self.steps.is_empty() {
            return 0.0;
        }
        self.current_step as f32 / self.steps.len() as f32
    }

    /// Get answer for a step
    pub fn get_answer(&self, step_id: &str) -> Option<&WizardAnswer> {
        self.answers.iter().find(|a| a.step_id == step_id)
    }

    /// Is complete?
    pub fn is_complete(&self) -> bool {
        self.state == WizardState::Completed
    }

    /// Total steps
    pub fn total_steps(&self) -> usize {
        self.steps.len()
    }
}

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

/// Format wizard for display
pub fn format_wizard(wizard: &SettingsWizard) -> String {
    let mut output = String::new();

    output.push_str(&format!("=== {} ===\n\n", wizard.name));
    output.push_str(&format!("{}\n\n", wizard.description));
    output.push_str(&format!("Status: {}\n", wizard.state));
    output.push_str(&format!(
        "Progress: {}/{} ({:.0}%)\n\n",
        wizard.current_step + 1,
        wizard.total_steps(),
        wizard.progress() * 100.0
    ));

    if let Some(step) = wizard.current() {
        output.push_str(&format!("Current: {}\n", step.title));
        output.push_str(&format!("{}\n", step.description));
    }

    output
}

/// Check if query is about wizard
pub fn is_wizard_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("wizard")
        || lower.contains("setup")
        || lower.contains("configure anna")
        || lower.contains("get started")
}

/// Fun fact about wizards
pub fn wizard_fun_fact() -> &'static str {
    "The settings wizard helps you configure Anna step by step!"
}

#[cfg(test)]
mod tests {
    use super::*;

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
