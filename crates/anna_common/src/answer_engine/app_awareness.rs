//! App Awareness Module - v0.24.0
//!
//! Detects desktop environment, window manager, and running applications.
//! All knowledge is probe-derived - no hardcoded mappings.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Protocol version for v0.24.0
pub const PROTOCOL_VERSION_V24: &str = "0.24.0";

// ============================================================================
// Desktop Environment Detection
// ============================================================================

/// Known desktop environment indicators (discovered via probes)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DesktopEnvironment {
    /// GNOME desktop
    Gnome,
    /// KDE Plasma
    Kde,
    /// XFCE
    Xfce,
    /// Cinnamon
    Cinnamon,
    /// MATE
    Mate,
    /// LXQt
    Lxqt,
    /// Budgie
    Budgie,
    /// Deepin
    Deepin,
    /// Pantheon (elementary OS)
    Pantheon,
    /// Unknown or standalone WM
    Unknown(String),
}

impl DesktopEnvironment {
    /// Parse from XDG_CURRENT_DESKTOP or similar env var
    pub fn from_xdg_string(s: &str) -> Self {
        let lower = s.to_lowercase();
        if lower.contains("gnome") {
            Self::Gnome
        } else if lower.contains("kde") || lower.contains("plasma") {
            Self::Kde
        } else if lower.contains("xfce") {
            Self::Xfce
        } else if lower.contains("cinnamon") {
            Self::Cinnamon
        } else if lower.contains("mate") {
            Self::Mate
        } else if lower.contains("lxqt") {
            Self::Lxqt
        } else if lower.contains("budgie") {
            Self::Budgie
        } else if lower.contains("deepin") {
            Self::Deepin
        } else if lower.contains("pantheon") {
            Self::Pantheon
        } else {
            Self::Unknown(s.to_string())
        }
    }
}

// ============================================================================
// Window Manager Detection
// ============================================================================

/// Window manager type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WindowManagerType {
    /// Stacking/floating window manager
    Stacking,
    /// Tiling window manager
    Tiling,
    /// Compositing window manager
    Compositing,
    /// Wayland compositor
    WaylandCompositor,
}

/// Window manager information (probe-derived)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowManagerInfo {
    /// WM name (e.g., "i3", "sway", "mutter", "kwin")
    pub name: String,
    /// WM type classification
    pub wm_type: WindowManagerType,
    /// Display protocol
    pub display_protocol: DisplayProtocol,
    /// Is this WM running?
    pub is_active: bool,
    /// Process ID if running
    pub pid: Option<u32>,
    /// Version string if available
    pub version: Option<String>,
}

/// Display protocol
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DisplayProtocol {
    X11,
    Wayland,
    Tty,
    Unknown,
}

impl DisplayProtocol {
    /// Detect from environment variables
    pub fn detect_from_env(env_vars: &HashMap<String, String>) -> Self {
        if env_vars.contains_key("WAYLAND_DISPLAY") {
            Self::Wayland
        } else if env_vars.contains_key("DISPLAY") {
            Self::X11
        } else {
            // Check for TTY
            if env_vars.get("TERM").map(|t| t.contains("linux")).unwrap_or(false) {
                Self::Tty
            } else {
                Self::Unknown
            }
        }
    }
}

// ============================================================================
// Running Application Detection
// ============================================================================

/// Information about a running application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunningApp {
    /// Process name
    pub name: String,
    /// Process ID
    pub pid: u32,
    /// Command line arguments
    pub cmdline: Vec<String>,
    /// Application category (if determinable)
    pub category: Option<AppCategory>,
    /// Desktop file path (if found)
    pub desktop_file: Option<String>,
    /// Window title (if X11/Wayland app with window)
    pub window_title: Option<String>,
    /// Memory usage in bytes
    pub memory_bytes: Option<u64>,
    /// CPU percentage
    pub cpu_percent: Option<f32>,
}

/// Application category (discovered, not hardcoded)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AppCategory {
    Browser,
    Editor,
    Terminal,
    FileManager,
    MediaPlayer,
    ImageViewer,
    DocumentViewer,
    Communication,
    Development,
    System,
    Game,
    Office,
    Graphics,
    Audio,
    Video,
    Network,
    Security,
    Utility,
    Other,
}

// ============================================================================
// Desktop Session State
// ============================================================================

/// Complete desktop session state (all probe-derived)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DesktopSession {
    /// Detected desktop environment
    pub desktop_environment: Option<DesktopEnvironment>,
    /// Window manager info
    pub window_manager: Option<WindowManagerInfo>,
    /// Display protocol in use
    pub display_protocol: DisplayProtocol,
    /// List of running GUI applications
    pub running_apps: Vec<RunningApp>,
    /// Session type (graphical, console)
    pub session_type: SessionType,
    /// Current user session
    pub user_session: Option<UserSession>,
    /// When this state was captured
    pub captured_at: i64,
}

impl Default for DisplayProtocol {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Session type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum SessionType {
    /// Graphical session (X11 or Wayland)
    Graphical,
    /// TTY/console session
    Console,
    /// SSH session
    Ssh,
    /// Unknown
    #[default]
    Unknown,
}

/// User session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    /// Username
    pub user: String,
    /// Session ID (from loginctl)
    pub session_id: String,
    /// Seat (if applicable)
    pub seat: Option<String>,
    /// TTY or display
    pub tty: Option<String>,
    /// Session state (active, online)
    pub state: String,
}

// ============================================================================
// App Detection Probes
// ============================================================================

/// Probe definition for app detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppDetectionProbe {
    /// Probe identifier
    pub id: String,
    /// What this probe detects
    pub detects: AppDetectionTarget,
    /// Command to run
    pub command: String,
    /// How to parse the output
    pub parser: OutputParser,
    /// Fact key to store result
    pub fact_key: String,
}

/// What the probe is trying to detect
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AppDetectionTarget {
    DesktopEnvironment,
    WindowManager,
    RunningApps,
    DisplayProtocol,
    SessionType,
}

/// Output parser type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OutputParser {
    /// Parse as JSON
    Json,
    /// Parse line by line
    Lines,
    /// Parse as key=value pairs
    KeyValue,
    /// Single value (trim whitespace)
    SingleValue,
    /// Custom regex extraction
    Regex(String),
}

// ============================================================================
// WM-Aware Suggestions
// ============================================================================

/// Context-aware suggestion based on WM/DE
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WmAwareSuggestion {
    /// The suggestion text
    pub suggestion: String,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    /// Which WM/DE this applies to
    pub applicable_to: Vec<String>,
    /// Source of this suggestion
    pub source: SuggestionSource,
}

/// Where the suggestion came from
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SuggestionSource {
    /// From probed config files
    ConfigFile(String),
    /// From fact store
    FactStore,
    /// From LLM reasoning
    LlmDerived,
    /// From previous user interaction
    UserHistory,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_desktop_environment_parsing() {
        assert_eq!(
            DesktopEnvironment::from_xdg_string("GNOME"),
            DesktopEnvironment::Gnome
        );
        assert_eq!(
            DesktopEnvironment::from_xdg_string("KDE"),
            DesktopEnvironment::Kde
        );
        assert_eq!(
            DesktopEnvironment::from_xdg_string("plasma"),
            DesktopEnvironment::Kde
        );
        assert_eq!(
            DesktopEnvironment::from_xdg_string("XFCE"),
            DesktopEnvironment::Xfce
        );
        assert!(matches!(
            DesktopEnvironment::from_xdg_string("custom-wm"),
            DesktopEnvironment::Unknown(_)
        ));
    }

    #[test]
    fn test_display_protocol_detection() {
        let mut env = HashMap::new();
        env.insert("WAYLAND_DISPLAY".to_string(), "wayland-0".to_string());
        assert_eq!(DisplayProtocol::detect_from_env(&env), DisplayProtocol::Wayland);

        let mut env2 = HashMap::new();
        env2.insert("DISPLAY".to_string(), ":0".to_string());
        assert_eq!(DisplayProtocol::detect_from_env(&env2), DisplayProtocol::X11);

        let empty: HashMap<String, String> = HashMap::new();
        assert_eq!(DisplayProtocol::detect_from_env(&empty), DisplayProtocol::Unknown);
    }

    #[test]
    fn test_desktop_session_default() {
        let session = DesktopSession::default();
        assert!(session.desktop_environment.is_none());
        assert!(session.window_manager.is_none());
        assert_eq!(session.display_protocol, DisplayProtocol::Unknown);
        assert!(session.running_apps.is_empty());
    }

    #[test]
    fn test_app_category_serialization() {
        let category = AppCategory::Browser;
        let json = serde_json::to_string(&category).unwrap();
        assert_eq!(json, "\"Browser\"");

        let parsed: AppCategory = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, AppCategory::Browser);
    }

    #[test]
    fn test_running_app_creation() {
        let app = RunningApp {
            name: "firefox".to_string(),
            pid: 1234,
            cmdline: vec!["firefox".to_string(), "--new-window".to_string()],
            category: Some(AppCategory::Browser),
            desktop_file: Some("/usr/share/applications/firefox.desktop".to_string()),
            window_title: Some("Mozilla Firefox".to_string()),
            memory_bytes: Some(512 * 1024 * 1024),
            cpu_percent: Some(5.2),
        };
        assert_eq!(app.name, "firefox");
        assert_eq!(app.category, Some(AppCategory::Browser));
    }
}
