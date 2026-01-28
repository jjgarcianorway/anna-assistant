//! LLM Core - Prompts, investigation, and evidence gathering.

pub mod commands;
pub mod evidence;
pub mod investigate;
pub mod prompts;
pub mod system_context;
pub mod types;

pub use system_context::system_context;
pub use types::{Finding, InvestigationState, NextStep, Understanding, VerificationResult};
