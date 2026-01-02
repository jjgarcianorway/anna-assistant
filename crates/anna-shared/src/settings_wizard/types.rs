// v0.0.572: Settings Wizard Core Types
// Wizard step types, choices, answers, and state enums

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;

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
