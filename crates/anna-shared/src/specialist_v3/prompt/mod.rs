//! Specialist prompt contract (v0.0.425).
//!
//! Strict prompts that enforce JSON-only responses.
//! No tutorials, no explanations - just structured data.

pub mod builder;
pub mod domain;
pub mod examples;
pub mod helpers;
pub mod system;

// Re-export commonly used items
pub use builder::build_specialist_prompt;
pub use domain::DomainPrompt;
pub use examples::{example_no_data_response, example_success_response};
pub use helpers::{risk_for_command, severity_for_finding};
pub use system::{confidence_guidelines, SPECIALIST_SYSTEM_PROMPT};
