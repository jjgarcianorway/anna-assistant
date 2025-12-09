//! Render types (v0.0.203).

/// Render policy determines what gets shown
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderPolicy {
    /// Debug OFF: Clean movie-terminal output
    Narrative,
    /// Debug ON: Full developer trace (existing behavior)
    Debug,
}

impl RenderPolicy {
    pub fn from_debug_mode(debug: bool) -> Self {
        if debug {
            Self::Debug
        } else {
            Self::Narrative
        }
    }
}

/// UI verbosity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Verbosity {
    Low,
    #[default]
    Normal,
    High,
}

impl Verbosity {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "low" => Self::Low,
            "high" => Self::High,
            _ => Self::Normal,
        }
    }
}

/// UI configuration
#[derive(Debug, Clone)]
pub struct UiConfig {
    pub verbosity: Verbosity,
    pub streaming: bool,
    pub narrative: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            verbosity: Verbosity::Normal,
            streaming: true,
            narrative: true,
        }
    }
}

/// Risk level for actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Low,    // Read-only operations
    Medium, // Config edits
    High,   // Package installs, system changes
}

impl RiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}
