// v0.0.572: Settings Wizard Implementation
// Main wizard struct with navigation and state management

use serde::{Deserialize, Serialize};

use crate::unified_settings::UnifiedSettings;

use super::types::{WizardAnswer, WizardState, WizardStep};

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
