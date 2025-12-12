//! UI Configuration - UX settings for Anna (v0.0.413).
//!
//! Controls how Anna renders output: cinematic vs debug mode,
//! internal comms visibility, spinner style, etc.

use crate::transcript_segment::TranscriptMode;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Display mode (cinematic or debug)
    #[serde(default)]
    pub mode: TranscriptMode,

    /// Show internal comms (IT department chatter)
    #[serde(default = "default_true")]
    pub show_internal_comms: bool,

    /// Show tips and status updates
    #[serde(default = "default_true")]
    pub show_tips: bool,

    /// Show probe details
    #[serde(default = "default_true")]
    pub show_probes: bool,

    /// Spinner style
    #[serde(default)]
    pub spinner: SpinnerStyle,

    /// Show timestamps in internal comms
    #[serde(default = "default_true")]
    pub show_timestamps: bool,

    /// Compact output (less whitespace)
    #[serde(default)]
    pub compact: bool,
}

fn default_true() -> bool {
    true
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            mode: TranscriptMode::Cinematic,
            show_internal_comms: true,
            show_tips: true,
            show_probes: true,
            spinner: SpinnerStyle::Simple,
            show_timestamps: true,
            compact: false,
        }
    }
}

/// Spinner animation style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SpinnerStyle {
    /// Simple ASCII spinner: | / - \
    #[default]
    Simple,
    /// Dots: . .. ...
    Dots,
    /// No spinner
    None,
}

impl std::fmt::Display for SpinnerStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpinnerStyle::Simple => write!(f, "simple"),
            SpinnerStyle::Dots => write!(f, "dots"),
            SpinnerStyle::None => write!(f, "none"),
        }
    }
}

impl UiConfig {
    /// Load from config file, falling back to defaults
    pub fn load() -> Self {
        // Try user config first, then system config
        let paths = [
            dirs::home_dir().map(|p| p.join(".anna").join("config.toml")),
            Some(PathBuf::from("/etc/anna/config.toml")),
        ];

        for path in paths.into_iter().flatten() {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(config) = Self::parse_from_toml(&content) {
                        return config;
                    }
                }
            }
        }

        Self::default()
    }

    /// Parse from TOML string (extracts [ui] section)
    fn parse_from_toml(content: &str) -> Result<Self, String> {
        // Try to parse as full config with [ui] section
        #[derive(Deserialize)]
        struct FullConfig {
            #[serde(default)]
            ui: UiConfig,
        }

        if let Ok(full) = toml::from_str::<FullConfig>(content) {
            return Ok(full.ui);
        }

        // Try to parse as just UiConfig
        toml::from_str(content).map_err(|e| e.to_string())
    }

    /// Apply command line overrides
    pub fn with_cli_overrides(
        mut self,
        mode: Option<TranscriptMode>,
        no_internal_comms: bool,
    ) -> Self {
        if let Some(m) = mode {
            self.mode = m;
        }
        if no_internal_comms {
            self.show_internal_comms = false;
        }
        self
    }

    /// Get spinner frames based on style
    pub fn spinner_frames(&self) -> &'static [&'static str] {
        match self.spinner {
            SpinnerStyle::Simple => &["|", "/", "-", "\\"],
            SpinnerStyle::Dots => &[".", "..", "...", ".."],
            SpinnerStyle::None => &[""],
        }
    }

    /// Summary for status display
    pub fn summary(&self) -> String {
        format!(
            "mode={}, internal_comms={}, tips={}, probes={}, spinner={}",
            self.mode, self.show_internal_comms, self.show_tips, self.show_probes, self.spinner
        )
    }
}

/// Runtime UI state (not persisted)
#[derive(Debug, Clone, Default)]
pub struct UiState {
    /// Current spinner frame index
    pub spinner_frame: usize,
    /// Last rendered line count (for clearing)
    pub last_line_count: usize,
    /// Whether we're in streaming mode
    pub streaming: bool,
    /// Terminal width
    pub term_width: usize,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            term_width: terminal_width(),
            ..Default::default()
        }
    }

    /// Advance spinner to next frame
    pub fn tick_spinner(&mut self, frames: &[&str]) -> String {
        let len = frames.len().max(1);
        let idx = self.spinner_frame % len;
        let frame = frames.get(idx).copied().unwrap_or("");
        self.spinner_frame = (self.spinner_frame + 1) % len;
        frame.to_string()
    }
}

/// Get terminal width (default 80 if detection fails)
pub fn terminal_width() -> usize {
    // Try to detect from environment or terminfo
    std::env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(80)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = UiConfig::default();
        assert_eq!(config.mode, TranscriptMode::Cinematic);
        assert!(config.show_internal_comms);
        assert!(config.show_tips);
    }

    #[test]
    fn test_parse_toml() {
        let toml = r#"
            [ui]
            mode = "debug"
            show_internal_comms = false
            spinner = "dots"
        "#;

        let config = UiConfig::parse_from_toml(toml).unwrap();
        assert_eq!(config.mode, TranscriptMode::Debug);
        assert!(!config.show_internal_comms);
        assert_eq!(config.spinner, SpinnerStyle::Dots);
    }

    #[test]
    fn test_spinner_frames() {
        let config = UiConfig::default();
        let frames = config.spinner_frames();
        assert_eq!(frames.len(), 4);
        assert_eq!(frames[0], "|");
    }
}
