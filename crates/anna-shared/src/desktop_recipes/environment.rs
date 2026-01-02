//! Desktop environment detection and types.

use serde::{Deserialize, Serialize};

/// Supported desktop environments
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DesktopEnvironment {
    Gnome,
    Kde,
    Xfce,
    Cinnamon,
    Mate,
    Unknown,
}

impl DesktopEnvironment {
    /// Detect current desktop environment from environment variables
    pub fn detect() -> Self {
        // Check XDG_CURRENT_DESKTOP first (most reliable)
        if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
            let desktop_lower = desktop.to_lowercase();
            if desktop_lower.contains("gnome") {
                return Self::Gnome;
            }
            if desktop_lower.contains("kde") || desktop_lower.contains("plasma") {
                return Self::Kde;
            }
            if desktop_lower.contains("xfce") {
                return Self::Xfce;
            }
            if desktop_lower.contains("cinnamon") {
                return Self::Cinnamon;
            }
            if desktop_lower.contains("mate") {
                return Self::Mate;
            }
        }

        // Fallback to DESKTOP_SESSION
        if let Ok(session) = std::env::var("DESKTOP_SESSION") {
            let session_lower = session.to_lowercase();
            if session_lower.contains("gnome") {
                return Self::Gnome;
            }
            if session_lower.contains("plasma") || session_lower.contains("kde") {
                return Self::Kde;
            }
            if session_lower.contains("xfce") {
                return Self::Xfce;
            }
        }

        Self::Unknown
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Gnome => "GNOME",
            Self::Kde => "KDE Plasma",
            Self::Xfce => "Xfce",
            Self::Cinnamon => "Cinnamon",
            Self::Mate => "MATE",
            Self::Unknown => "Unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_desktop_environment() {
        // Just test that detection doesn't panic
        let _de = DesktopEnvironment::detect();
    }

    #[test]
    fn test_desktop_environment_display_name() {
        assert_eq!(DesktopEnvironment::Gnome.display_name(), "GNOME");
        assert_eq!(DesktopEnvironment::Kde.display_name(), "KDE Plasma");
    }
}
