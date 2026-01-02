//! Status and stats display (v0.0.434).
//!
//! Honest reflection of hardware, models, and helpers in annactl status/stats.

mod core;
mod helper_usage;
mod helpers;
mod llm;
mod model_usage;
mod system_profile;
mod tests;

pub use core::HardwareStatus;
pub use helper_usage::{HelperUsage, HelperUsageStats};
pub use helpers::{HelperStatusEntry, HelperStatusSection};
pub use llm::{LlmSection, ModelStatusEntry};
pub use model_usage::{ModelError, ModelUsage, ModelUsageStats};
pub use system_profile::SystemProfileSection;
