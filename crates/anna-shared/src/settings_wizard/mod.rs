// v0.0.572: Settings Wizard (Phase 148)
// Guided interactive settings configuration
//
// This module is organized into logical submodules:
// - types: Core types (WizardStepType, WizardChoice, WizardStep, WizardAnswer, WizardState)
// - wizard: Main SettingsWizard implementation with navigation and state management
// - manager: WizardManager for managing multiple wizards and defaults
// - utils: Utility functions for formatting and query detection
// - tests: Comprehensive test coverage

mod types;
mod wizard;
mod manager;
mod utils;

#[cfg(test)]
mod tests;

// Re-export public API
pub use types::{WizardAnswer, WizardChoice, WizardState, WizardStep, WizardStepType};
pub use wizard::SettingsWizard;
pub use manager::WizardManager;
pub use utils::{format_wizard, is_wizard_query, wizard_fun_fact};
