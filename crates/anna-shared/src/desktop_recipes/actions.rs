//! Desktop configuration action types.

use serde::{Deserialize, Serialize};

/// Desktop configuration action types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DesktopAction {
    SetWallpaper { path: String },
    EnableDarkMode,
    DisableDarkMode,
    SetTheme { theme: String },
    SetIconTheme { theme: String },
    SetCursorTheme { theme: String },
}
