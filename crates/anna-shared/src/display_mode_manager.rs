// v0.0.536: Display Mode Manager (Phase 112)
// Manages debug vs fly-on-the-wall display modes per VISION.md

use serde::{Deserialize, Serialize};

/// Display mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum DisplayMode {
    #[default]
    FlyOnTheWall,
    Debug,
    Minimal,
    Verbose,
}

impl std::fmt::Display for DisplayMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FlyOnTheWall => write!(f, "Fly-on-the-Wall"),
            Self::Debug => write!(f, "Debug"),
            Self::Minimal => write!(f, "Minimal"),
            Self::Verbose => write!(f, "Verbose"),
        }
    }
}

impl DisplayMode {
    /// Should show internal communication?
    pub fn show_internal_comms(&self) -> bool {
        matches!(self, Self::FlyOnTheWall | Self::Verbose)
    }

    /// Should show JSON/technical details?
    pub fn show_technical(&self) -> bool {
        matches!(self, Self::Debug | Self::Verbose)
    }

    /// Should show spinner animations?
    pub fn show_spinner(&self) -> bool {
        !matches!(self, Self::Minimal)
    }

    /// Should stream word-by-word?
    pub fn stream_output(&self) -> bool {
        !matches!(self, Self::Minimal)
    }
}

/// Output section type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OutputSection {
    Greeting,
    InternalComms,
    SpecialistDialog,
    ProbeResult,
    Citation,
    Answer,
    Error,
    Warning,
    Progress,
    DebugInfo,
}

impl std::fmt::Display for OutputSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Greeting => write!(f, "Greeting"),
            Self::InternalComms => write!(f, "Internal Comms"),
            Self::SpecialistDialog => write!(f, "Specialist Dialog"),
            Self::ProbeResult => write!(f, "Probe Result"),
            Self::Citation => write!(f, "Citation"),
            Self::Answer => write!(f, "Answer"),
            Self::Error => write!(f, "Error"),
            Self::Warning => write!(f, "Warning"),
            Self::Progress => write!(f, "Progress"),
            Self::DebugInfo => write!(f, "Debug Info"),
        }
    }
}

/// Visibility rule for a section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisibilityRule {
    pub section: OutputSection,
    pub visible_in: Vec<DisplayMode>,
}

impl VisibilityRule {
    /// Create new rule
    pub fn new(section: OutputSection, modes: Vec<DisplayMode>) -> Self {
        Self {
            section,
            visible_in: modes,
        }
    }

    /// Is visible in mode?
    pub fn is_visible(&self, mode: &DisplayMode) -> bool {
        self.visible_in.contains(mode)
    }
}

/// Display mode manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayModeManager {
    current_mode: DisplayMode,
    rules: Vec<VisibilityRule>,
    use_true_color: bool,
    hollywood_style: bool,
}

impl Default for DisplayModeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DisplayModeManager {
    /// Create new manager with default rules
    pub fn new() -> Self {
        let mut manager = Self {
            current_mode: DisplayMode::FlyOnTheWall,
            rules: Vec::new(),
            use_true_color: true,
            hollywood_style: true,
        };
        manager.load_default_rules();
        manager
    }

    /// Load default visibility rules
    fn load_default_rules(&mut self) {
        use DisplayMode::*;
        use OutputSection::*;

        self.rules = vec![
            VisibilityRule::new(Greeting, vec![FlyOnTheWall, Debug, Verbose]),
            VisibilityRule::new(InternalComms, vec![FlyOnTheWall, Verbose]),
            VisibilityRule::new(SpecialistDialog, vec![FlyOnTheWall, Debug, Verbose]),
            VisibilityRule::new(ProbeResult, vec![Debug, Verbose]),
            VisibilityRule::new(Citation, vec![FlyOnTheWall, Debug, Verbose]),
            VisibilityRule::new(Answer, vec![FlyOnTheWall, Debug, Minimal, Verbose]),
            VisibilityRule::new(Error, vec![FlyOnTheWall, Debug, Minimal, Verbose]),
            VisibilityRule::new(Warning, vec![FlyOnTheWall, Debug, Verbose]),
            VisibilityRule::new(Progress, vec![FlyOnTheWall, Debug, Verbose]),
            VisibilityRule::new(OutputSection::DebugInfo, vec![Debug, Verbose]),
        ];
    }

    /// Set display mode
    pub fn set_mode(&mut self, mode: DisplayMode) {
        self.current_mode = mode;
    }

    /// Get current mode
    pub fn mode(&self) -> DisplayMode {
        self.current_mode
    }

    /// Toggle debug mode
    pub fn toggle_debug(&mut self) {
        self.current_mode = if self.current_mode == DisplayMode::Debug {
            DisplayMode::FlyOnTheWall
        } else {
            DisplayMode::Debug
        };
    }

    /// Is section visible?
    pub fn is_visible(&self, section: &OutputSection) -> bool {
        self.rules
            .iter()
            .find(|r| &r.section == section)
            .map(|r| r.is_visible(&self.current_mode))
            .unwrap_or(true)
    }

    /// Should show internal comms?
    pub fn show_internal_comms(&self) -> bool {
        self.current_mode.show_internal_comms()
    }

    /// Should show technical details?
    pub fn show_technical(&self) -> bool {
        self.current_mode.show_technical()
    }

    /// Should show spinner?
    pub fn show_spinner(&self) -> bool {
        self.current_mode.show_spinner()
    }

    /// Should stream output?
    pub fn stream_output(&self) -> bool {
        self.current_mode.stream_output()
    }

    /// Use true color?
    pub fn use_true_color(&self) -> bool {
        self.use_true_color
    }

    /// Set true color
    pub fn set_true_color(&mut self, enabled: bool) {
        self.use_true_color = enabled;
    }

    /// Hollywood style?
    pub fn hollywood_style(&self) -> bool {
        self.hollywood_style
    }

    /// Is debug mode?
    pub fn is_debug(&self) -> bool {
        self.current_mode == DisplayMode::Debug
    }

    /// Get mode description
    pub fn mode_description(&self) -> &'static str {
        match self.current_mode {
            DisplayMode::FlyOnTheWall => "Natural language dialog between Anna and specialists",
            DisplayMode::Debug => "Technical details, JSON, and internal workings",
            DisplayMode::Minimal => "Just the answer, nothing else",
            DisplayMode::Verbose => "Everything - dialog, debug, and more",
        }
    }
}

/// Format mode for display
pub fn format_display_mode(mode: &DisplayMode) -> String {
    format!(
        "{} - {}",
        mode,
        match mode {
            DisplayMode::FlyOnTheWall => "Watch IT team work like a fly on the wall",
            DisplayMode::Debug => "See what's really happening (troubleshooting)",
            DisplayMode::Minimal => "Just the essentials",
            DisplayMode::Verbose => "Everything, including debug info",
        }
    )
}

/// Format manager summary
pub fn format_manager_summary(manager: &DisplayModeManager) -> String {
    let mut output = String::new();
    output.push_str("=== Display Mode Manager ===\n\n");

    output.push_str(&format!("Current Mode: {}\n", manager.mode()));
    output.push_str(&format!("Description: {}\n\n", manager.mode_description()));

    output.push_str(&format!("True Color: {}\n", manager.use_true_color()));
    output.push_str(&format!("Hollywood Style: {}\n", manager.hollywood_style()));
    output.push_str(&format!("Show Spinner: {}\n", manager.show_spinner()));
    output.push_str(&format!("Stream Output: {}\n", manager.stream_output()));

    output
}

/// Check if query is display-related
pub fn is_display_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("display")
        || lower.contains("debug mode")
        || lower.contains("verbose")
        || lower.contains("minimal")
        || lower.contains("show more")
        || lower.contains("show less")
}

/// Fun fact about display modes
pub fn display_fun_fact() -> &'static str {
    "The 'fly-on-the-wall' mode lets you watch Anna's IT department work like you're observing a real team - Hollywood style, no icons, just clean professional output!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_mode_default() {
        let mode = DisplayMode::default();
        assert_eq!(mode, DisplayMode::FlyOnTheWall);
    }

    #[test]
    fn test_show_internal_comms() {
        assert!(DisplayMode::FlyOnTheWall.show_internal_comms());
        assert!(!DisplayMode::Debug.show_internal_comms());
        assert!(DisplayMode::Verbose.show_internal_comms());
    }

    #[test]
    fn test_show_technical() {
        assert!(!DisplayMode::FlyOnTheWall.show_technical());
        assert!(DisplayMode::Debug.show_technical());
        assert!(DisplayMode::Verbose.show_technical());
    }

    #[test]
    fn test_manager_creation() {
        let manager = DisplayModeManager::new();
        assert_eq!(manager.mode(), DisplayMode::FlyOnTheWall);
    }

    #[test]
    fn test_set_mode() {
        let mut manager = DisplayModeManager::new();
        manager.set_mode(DisplayMode::Debug);
        assert_eq!(manager.mode(), DisplayMode::Debug);
    }

    #[test]
    fn test_toggle_debug() {
        let mut manager = DisplayModeManager::new();
        assert!(!manager.is_debug());
        manager.toggle_debug();
        assert!(manager.is_debug());
        manager.toggle_debug();
        assert!(!manager.is_debug());
    }

    #[test]
    fn test_visibility_rules() {
        let manager = DisplayModeManager::new();
        assert!(manager.is_visible(&OutputSection::Answer));
        assert!(manager.is_visible(&OutputSection::InternalComms));
    }

    #[test]
    fn test_debug_visibility() {
        let mut manager = DisplayModeManager::new();
        manager.set_mode(DisplayMode::Debug);
        assert!(manager.is_visible(&OutputSection::DebugInfo));
        assert!(manager.is_visible(&OutputSection::ProbeResult));
    }

    #[test]
    fn test_is_display_query() {
        assert!(is_display_query("Enable debug mode"));
        assert!(is_display_query("Show more details"));
        assert!(is_display_query("Use verbose output"));
        assert!(!is_display_query("Install vim"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = display_fun_fact();
        assert!(fact.contains("fly-on-the-wall") || fact.contains("Hollywood"));
    }
}
