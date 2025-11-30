//! UI Theme System v3.13.4
//!
//! Provides ASCII and Unicode modes for terminal output.
//! Default is ASCII - clean, readable, no emoji circus.
//!
//! Config:
//! ```toml
//! [ui]
//! mode = "ascii"  # or "unicode"
//! ```

use serde::{Deserialize, Serialize};

// ============================================================================
// UI MODE
// ============================================================================

/// UI rendering mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UiMode {
    /// ASCII-only output - clean terminal aesthetic
    Ascii,
    /// Unicode with some box drawing (no emoji)
    Unicode,
}

impl Default for UiMode {
    fn default() -> Self {
        // Default to ASCII - old school hacker style
        UiMode::Ascii
    }
}

impl UiMode {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "unicode" => UiMode::Unicode,
            _ => UiMode::Ascii,
        }
    }
}

// ============================================================================
// UI THEME
// ============================================================================

/// Theme glyphs for terminal output
#[derive(Debug, Clone)]
pub struct UiTheme {
    // Status indicators
    pub ok: &'static str,
    pub warn: &'static str,
    pub error: &'static str,
    pub info: &'static str,

    // List markers
    pub bullet: &'static str,
    pub arrow: &'static str,

    // Section dividers
    pub section_line: &'static str,
    pub banner_line: &'static str,

    // Agent labels
    pub anna: &'static str,
    pub junior: &'static str,
    pub senior: &'static str,
    pub brain: &'static str,

    // State indicators
    pub running: &'static str,
    pub stopped: &'static str,
    pub degraded: &'static str,
    pub stale: &'static str,
}

impl UiTheme {
    /// ASCII theme - clean terminal output
    pub fn ascii() -> Self {
        UiTheme {
            ok: "+",
            warn: "!",
            error: "X",
            info: "*",

            bullet: "-",
            arrow: "->",

            section_line: "------------------------------------------------------------",
            banner_line: "============================================================",

            anna: "ANNA",
            junior: "JR",
            senior: "SR",
            brain: "BRAIN",

            running: "RUNNING",
            stopped: "STOPPED",
            degraded: "DEGRADED",
            stale: "STALE",
        }
    }

    /// Unicode theme - box drawing, no emoji
    pub fn unicode() -> Self {
        UiTheme {
            ok: "+",
            warn: "!",
            error: "x",
            info: "*",

            bullet: "-",
            arrow: "->",

            section_line: "------------------------------------------------------------",
            banner_line: "============================================================",

            anna: "Anna",
            junior: "Junior",
            senior: "Senior",
            brain: "Brain",

            running: "running",
            stopped: "stopped",
            degraded: "degraded",
            stale: "stale",
        }
    }

    /// Get theme for mode
    pub fn for_mode(mode: UiMode) -> Self {
        match mode {
            UiMode::Ascii => Self::ascii(),
            UiMode::Unicode => Self::unicode(),
        }
    }
}

impl Default for UiTheme {
    fn default() -> Self {
        Self::ascii()
    }
}

// ============================================================================
// GLOBAL THEME STATE
// ============================================================================

use std::sync::OnceLock;

static CURRENT_MODE: OnceLock<UiMode> = OnceLock::new();

/// Set the global UI mode (call once at startup)
pub fn set_ui_mode(mode: UiMode) {
    let _ = CURRENT_MODE.set(mode);
}

/// Get the current UI mode
pub fn get_ui_mode() -> UiMode {
    *CURRENT_MODE.get().unwrap_or(&UiMode::Ascii)
}

/// Get the current theme
pub fn current_theme() -> UiTheme {
    UiTheme::for_mode(get_ui_mode())
}

// ============================================================================
// DAEMON STATE (honest reporting)
// ============================================================================

/// Daemon state - explicit and honest
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaemonState {
    /// Process running, API responding
    Running,
    /// Process not found
    NotRunning,
    /// Process running but API not responding
    ApiUnreachable,
    /// Process running but degraded
    Degraded,
    /// Unknown state
    Unknown,
}

impl DaemonState {
    /// Get display string for state
    pub fn display(&self, theme: &UiTheme) -> String {
        match self {
            DaemonState::Running => format!("{} annad {}", theme.ok, theme.running),
            DaemonState::NotRunning => format!("{} annad {}", theme.error, theme.stopped),
            DaemonState::ApiUnreachable => format!("{} annad PROCESS_RUNNING_API_UNREACHABLE", theme.warn),
            DaemonState::Degraded => format!("{} annad {}", theme.warn, theme.degraded),
            DaemonState::Unknown => format!("{} annad UNKNOWN", theme.warn),
        }
    }

    /// Get short status
    pub fn short(&self) -> &'static str {
        match self {
            DaemonState::Running => "OK",
            DaemonState::NotRunning => "STOPPED",
            DaemonState::ApiUnreachable => "API_UNREACHABLE",
            DaemonState::Degraded => "DEGRADED",
            DaemonState::Unknown => "UNKNOWN",
        }
    }
}

// ============================================================================
// DATA FRESHNESS
// ============================================================================

/// Data freshness indicator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataFreshness {
    /// Data is current (from live daemon)
    Live,
    /// Data is stale (from disk cache, daemon unreachable)
    Stale,
    /// No data available
    Unavailable,
}

impl DataFreshness {
    pub fn label(&self, theme: &UiTheme) -> &'static str {
        match self {
            DataFreshness::Live => "LIVE",
            DataFreshness::Stale => theme.stale,
            DataFreshness::Unavailable => "N/A",
        }
    }
}

// ============================================================================
// FORMATTING HELPERS
// ============================================================================

/// Print a section header
pub fn print_section(title: &str) {
    let theme = current_theme();
    println!("{}", theme.section_line);
    println!("{}", title);
    println!("{}", theme.section_line);
}

/// Print a banner (for major events like First Light)
pub fn print_banner(title: &str) {
    let theme = current_theme();
    println!("{}", theme.banner_line);
    println!("  {}", title);
    println!("{}", theme.banner_line);
}

/// Format a status line
pub fn format_status_line(indicator: &str, label: &str, value: &str) -> String {
    format!("  {}  {}: {}", indicator, label, value)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_theme() {
        let theme = UiTheme::ascii();
        assert_eq!(theme.ok, "+");
        assert_eq!(theme.warn, "!");
        assert_eq!(theme.junior, "JR");
    }

    #[test]
    fn test_mode_default() {
        assert_eq!(UiMode::default(), UiMode::Ascii);
    }

    #[test]
    fn test_mode_from_str() {
        assert_eq!(UiMode::from_str("ascii"), UiMode::Ascii);
        assert_eq!(UiMode::from_str("unicode"), UiMode::Unicode);
        assert_eq!(UiMode::from_str("garbage"), UiMode::Ascii);
    }

    #[test]
    fn test_daemon_state_display() {
        let theme = UiTheme::ascii();
        let running = DaemonState::Running.display(&theme);
        assert!(running.contains("RUNNING"));

        let unreachable = DaemonState::ApiUnreachable.display(&theme);
        assert!(unreachable.contains("API_UNREACHABLE"));
    }
}
