//! Desktop environment configuration recipes (v0.0.257).
//!
//! Provides recipes for common desktop configuration tasks like:
//! - Setting wallpaper
//! - Enabling/disabling dark mode
//! - Changing themes
//! - Display settings
//!
//! Detects the desktop environment and generates appropriate commands.

mod actions;
mod detection;
mod environment;
mod recipe;
mod utils;

// Re-export public API
pub use actions::DesktopAction;
pub use detection::{detect_desktop_intent, is_desktop_config_request};
pub use environment::DesktopEnvironment;
pub use recipe::DesktopRecipe;
