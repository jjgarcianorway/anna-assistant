// v0.0.549: Output Style Config (Phase 125)
// Configurable output style per VISION.md - Hollywood IT aesthetic

use serde::{Deserialize, Serialize};

/// Color scheme
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum ColorScheme {
    #[default]
    TrueColor,
    Ansi256,
    Ansi16,
    NoColor,
}

impl std::fmt::Display for ColorScheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TrueColor => write!(f, "True Color (24-bit)"),
            Self::Ansi256 => write!(f, "ANSI 256"),
            Self::Ansi16 => write!(f, "ANSI 16"),
            Self::NoColor => write!(f, "No Color"),
        }
    }
}

/// Theme style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum ThemeStyle {
    #[default]
    Hollywood,
    Minimal,
    Classic,
    Hacker,
    Professional,
}

impl std::fmt::Display for ThemeStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hollywood => write!(f, "Hollywood IT"),
            Self::Minimal => write!(f, "Minimal"),
            Self::Classic => write!(f, "Classic"),
            Self::Hacker => write!(f, "Hacker"),
            Self::Professional => write!(f, "Professional"),
        }
    }
}

/// Animation style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum AnimationStyle {
    #[default]
    Spinner,
    Progress,
    Dots,
    None,
}

impl std::fmt::Display for AnimationStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Spinner => write!(f, "Spinner"),
            Self::Progress => write!(f, "Progress Bar"),
            Self::Dots => write!(f, "Dots"),
            Self::None => write!(f, "None"),
        }
    }
}

/// Border style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum BorderStyle {
    #[default]
    Rounded,
    Sharp,
    Double,
    Ascii,
    None,
}

impl std::fmt::Display for BorderStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rounded => write!(f, "Rounded"),
            Self::Sharp => write!(f, "Sharp"),
            Self::Double => write!(f, "Double"),
            Self::Ascii => write!(f, "ASCII"),
            Self::None => write!(f, "None"),
        }
    }
}

/// Output style configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputStyleConfig {
    pub color_scheme: ColorScheme,
    pub theme: ThemeStyle,
    pub animation: AnimationStyle,
    pub border: BorderStyle,
    pub use_bold: bool,
    pub use_italic: bool,
    pub use_underline: bool,
    pub highlight_commands: bool,
    pub highlight_paths: bool,
    pub max_width: u32,
    pub compact_mode: bool,
}

impl Default for OutputStyleConfig {
    fn default() -> Self {
        Self {
            color_scheme: ColorScheme::TrueColor,
            theme: ThemeStyle::Hollywood,
            animation: AnimationStyle::Spinner,
            border: BorderStyle::Rounded,
            use_bold: true,
            use_italic: false,
            use_underline: false,
            highlight_commands: true,
            highlight_paths: true,
            max_width: 120,
            compact_mode: false,
        }
    }
}

impl OutputStyleConfig {
    /// Create new config
    pub fn new() -> Self {
        Self::default()
    }

    /// Minimal style preset
    pub fn minimal() -> Self {
        Self {
            color_scheme: ColorScheme::Ansi16,
            theme: ThemeStyle::Minimal,
            animation: AnimationStyle::None,
            border: BorderStyle::None,
            use_bold: false,
            use_italic: false,
            use_underline: false,
            highlight_commands: false,
            highlight_paths: false,
            max_width: 80,
            compact_mode: true,
        }
    }

    /// Hacker style preset
    pub fn hacker() -> Self {
        Self {
            color_scheme: ColorScheme::TrueColor,
            theme: ThemeStyle::Hacker,
            animation: AnimationStyle::Dots,
            border: BorderStyle::Ascii,
            use_bold: true,
            use_italic: false,
            use_underline: false,
            highlight_commands: true,
            highlight_paths: true,
            max_width: 0, // unlimited
            compact_mode: false,
        }
    }

    /// Professional style preset
    pub fn professional() -> Self {
        Self {
            color_scheme: ColorScheme::Ansi256,
            theme: ThemeStyle::Professional,
            animation: AnimationStyle::Progress,
            border: BorderStyle::Sharp,
            use_bold: true,
            use_italic: false,
            use_underline: true,
            highlight_commands: true,
            highlight_paths: true,
            max_width: 100,
            compact_mode: false,
        }
    }

    /// No color mode
    pub fn no_color() -> Self {
        Self {
            color_scheme: ColorScheme::NoColor,
            theme: ThemeStyle::Classic,
            animation: AnimationStyle::None,
            border: BorderStyle::Ascii,
            use_bold: false,
            use_italic: false,
            use_underline: false,
            highlight_commands: false,
            highlight_paths: false,
            max_width: 80,
            compact_mode: true,
        }
    }

    /// Is color enabled?
    pub fn has_color(&self) -> bool {
        self.color_scheme != ColorScheme::NoColor
    }

    /// Is animation enabled?
    pub fn has_animation(&self) -> bool {
        self.animation != AnimationStyle::None
    }

    /// Is compact mode?
    pub fn is_compact(&self) -> bool {
        self.compact_mode
    }

    /// Get effective max width (0 means terminal width)
    pub fn effective_width(&self, terminal_width: u32) -> u32 {
        if self.max_width == 0 || self.max_width > terminal_width {
            terminal_width
        } else {
            self.max_width
        }
    }

    /// Apply natural language change
    pub fn apply_change(&mut self, request: &str) -> Option<String> {
        let lower = request.to_lowercase();

        // Theme changes
        if lower.contains("hollywood") || lower.contains("movie style") {
            *self = Self::default();
            return Some("Hollywood IT style applied - movie-like aesthetics.".to_string());
        }
        if lower.contains("minimal") || lower.contains("simple style") {
            *self = Self::minimal();
            return Some("Minimal style applied - clean and simple.".to_string());
        }
        if lower.contains("hacker") || lower.contains("matrix") {
            *self = Self::hacker();
            return Some("Hacker style applied - terminal aesthetics.".to_string());
        }
        if lower.contains("professional") || lower.contains("business") {
            *self = Self::professional();
            return Some("Professional style applied.".to_string());
        }
        if lower.contains("no color") || lower.contains("monochrome") || lower.contains("plain") {
            *self = Self::no_color();
            return Some("Plain text mode - no colors or formatting.".to_string());
        }

        // Individual toggles
        if lower.contains("enable color") || lower.contains("use color") {
            self.color_scheme = ColorScheme::TrueColor;
            return Some("True color enabled.".to_string());
        }
        if lower.contains("disable color") || lower.contains("turn off color") {
            self.color_scheme = ColorScheme::NoColor;
            return Some("Colors disabled.".to_string());
        }
        if lower.contains("enable animation") || lower.contains("show spinner") {
            self.animation = AnimationStyle::Spinner;
            return Some("Animations enabled.".to_string());
        }
        if lower.contains("disable animation") || lower.contains("no spinner") {
            self.animation = AnimationStyle::None;
            return Some("Animations disabled.".to_string());
        }
        if lower.contains("compact") || lower.contains("dense") {
            self.compact_mode = true;
            return Some("Compact mode enabled.".to_string());
        }
        if lower.contains("spacious") || lower.contains("expanded") {
            self.compact_mode = false;
            return Some("Spacious mode enabled.".to_string());
        }

        None
    }
}

/// Format output style config
pub fn format_output_style(config: &OutputStyleConfig) -> String {
    let mut output = String::new();
    output.push_str("=== Output Style Configuration ===\n\n");

    output.push_str(&format!("Color Scheme: {}\n", config.color_scheme));
    output.push_str(&format!("Theme: {}\n", config.theme));
    output.push_str(&format!("Animation: {}\n", config.animation));
    output.push_str(&format!("Border: {}\n", config.border));
    output.push_str(&format!("Bold: {}\n", config.use_bold));
    output.push_str(&format!("Italic: {}\n", config.use_italic));
    output.push_str(&format!("Underline: {}\n", config.use_underline));
    output.push_str(&format!("Highlight Commands: {}\n", config.highlight_commands));
    output.push_str(&format!("Highlight Paths: {}\n", config.highlight_paths));
    output.push_str(&format!("Max Width: {}\n", config.max_width));
    output.push_str(&format!("Compact Mode: {}\n", config.compact_mode));

    output
}

/// Check if query is output style related
pub fn is_output_style_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("output style")
        || lower.contains("theme")
        || lower.contains("color scheme")
        || lower.contains("appearance")
}

/// Fun fact about output style
pub fn output_style_fun_fact() -> &'static str {
    "The 'Hollywood hacker' aesthetic in movies was popularized by the 1995 film Hackers!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_scheme_display() {
        assert_eq!(format!("{}", ColorScheme::TrueColor), "True Color (24-bit)");
        assert_eq!(format!("{}", ColorScheme::NoColor), "No Color");
    }

    #[test]
    fn test_default_config() {
        let config = OutputStyleConfig::default();
        assert_eq!(config.color_scheme, ColorScheme::TrueColor);
        assert_eq!(config.theme, ThemeStyle::Hollywood);
    }

    #[test]
    fn test_minimal_preset() {
        let config = OutputStyleConfig::minimal();
        assert_eq!(config.theme, ThemeStyle::Minimal);
        assert!(config.compact_mode);
    }

    #[test]
    fn test_hacker_preset() {
        let config = OutputStyleConfig::hacker();
        assert_eq!(config.theme, ThemeStyle::Hacker);
        assert_eq!(config.animation, AnimationStyle::Dots);
    }

    #[test]
    fn test_no_color_preset() {
        let config = OutputStyleConfig::no_color();
        assert!(!config.has_color());
        assert!(!config.has_animation());
    }

    #[test]
    fn test_has_color() {
        let config = OutputStyleConfig::default();
        assert!(config.has_color());
        let no_color = OutputStyleConfig::no_color();
        assert!(!no_color.has_color());
    }

    #[test]
    fn test_effective_width() {
        let config = OutputStyleConfig::default();
        assert_eq!(config.effective_width(80), 80);
        assert_eq!(config.effective_width(200), 120);
    }

    #[test]
    fn test_apply_hacker() {
        let mut config = OutputStyleConfig::default();
        let result = config.apply_change("use hacker style");
        assert!(result.is_some());
        assert_eq!(config.theme, ThemeStyle::Hacker);
    }

    #[test]
    fn test_apply_compact() {
        let mut config = OutputStyleConfig::default();
        config.apply_change("enable compact mode");
        assert!(config.is_compact());
    }

    #[test]
    fn test_is_output_style_query() {
        assert!(is_output_style_query("Change theme"));
        assert!(is_output_style_query("What's the color scheme?"));
        assert!(!is_output_style_query("Install vim"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = output_style_fun_fact();
        assert!(fact.contains("1995"));
    }
}
