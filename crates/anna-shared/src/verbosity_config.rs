// v0.0.546: Verbosity Config (Phase 122)
// Configurable verbosity and detail level per VISION.md

use serde::{Deserialize, Serialize};

/// Verbosity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum VerbosityLevel {
    Minimal,
    #[default]
    Normal,
    Detailed,
    Verbose,
    Debug,
}

impl std::fmt::Display for VerbosityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Minimal => write!(f, "Minimal"),
            Self::Normal => write!(f, "Normal"),
            Self::Detailed => write!(f, "Detailed"),
            Self::Verbose => write!(f, "Verbose"),
            Self::Debug => write!(f, "Debug"),
        }
    }
}

/// Detail level for different output types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum DetailLevel {
    Summary,
    #[default]
    Standard,
    Full,
    Exhaustive,
}

impl std::fmt::Display for DetailLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Summary => write!(f, "Summary"),
            Self::Standard => write!(f, "Standard"),
            Self::Full => write!(f, "Full"),
            Self::Exhaustive => write!(f, "Exhaustive"),
        }
    }
}

/// Output context type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OutputContext {
    Greeting,
    Answer,
    Status,
    Stats,
    Error,
    InternalComms,
    Progress,
}

impl std::fmt::Display for OutputContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Greeting => write!(f, "Greeting"),
            Self::Answer => write!(f, "Answer"),
            Self::Status => write!(f, "Status"),
            Self::Stats => write!(f, "Stats"),
            Self::Error => write!(f, "Error"),
            Self::InternalComms => write!(f, "Internal Comms"),
            Self::Progress => write!(f, "Progress"),
        }
    }
}

/// Verbosity configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerbosityConfig {
    pub level: VerbosityLevel,
    pub answer_detail: DetailLevel,
    pub status_detail: DetailLevel,
    pub stats_detail: DetailLevel,
    pub show_citations: bool,
    pub show_timestamps: bool,
    pub show_confidence: bool,
    pub show_internal_comms: bool,
    pub show_progress: bool,
    pub max_lines_per_section: u32,
}

impl Default for VerbosityConfig {
    fn default() -> Self {
        Self {
            level: VerbosityLevel::Normal,
            answer_detail: DetailLevel::Standard,
            status_detail: DetailLevel::Standard,
            stats_detail: DetailLevel::Standard,
            show_citations: true,
            show_timestamps: false,
            show_confidence: false,
            show_internal_comms: true,
            show_progress: true,
            max_lines_per_section: 50,
        }
    }
}

impl VerbosityConfig {
    /// Create new config
    pub fn new() -> Self {
        Self::default()
    }

    /// Minimal verbosity preset
    pub fn minimal() -> Self {
        Self {
            level: VerbosityLevel::Minimal,
            answer_detail: DetailLevel::Summary,
            status_detail: DetailLevel::Summary,
            stats_detail: DetailLevel::Summary,
            show_citations: false,
            show_timestamps: false,
            show_confidence: false,
            show_internal_comms: false,
            show_progress: false,
            max_lines_per_section: 10,
        }
    }

    /// Verbose preset - lots of detail
    pub fn verbose() -> Self {
        Self {
            level: VerbosityLevel::Verbose,
            answer_detail: DetailLevel::Full,
            status_detail: DetailLevel::Full,
            stats_detail: DetailLevel::Full,
            show_citations: true,
            show_timestamps: true,
            show_confidence: true,
            show_internal_comms: true,
            show_progress: true,
            max_lines_per_section: 200,
        }
    }

    /// Debug preset - maximum detail
    pub fn debug() -> Self {
        Self {
            level: VerbosityLevel::Debug,
            answer_detail: DetailLevel::Exhaustive,
            status_detail: DetailLevel::Exhaustive,
            stats_detail: DetailLevel::Exhaustive,
            show_citations: true,
            show_timestamps: true,
            show_confidence: true,
            show_internal_comms: true,
            show_progress: true,
            max_lines_per_section: 500,
        }
    }

    /// Is minimal mode?
    pub fn is_minimal(&self) -> bool {
        self.level == VerbosityLevel::Minimal
    }

    /// Is verbose or debug?
    pub fn is_verbose(&self) -> bool {
        matches!(self.level, VerbosityLevel::Verbose | VerbosityLevel::Debug)
    }

    /// Is debug mode?
    pub fn is_debug(&self) -> bool {
        self.level == VerbosityLevel::Debug
    }

    /// Get detail level for context
    pub fn detail_for(&self, context: OutputContext) -> DetailLevel {
        match context {
            OutputContext::Answer => self.answer_detail,
            OutputContext::Status => self.status_detail,
            OutputContext::Stats => self.stats_detail,
            _ => match self.level {
                VerbosityLevel::Minimal => DetailLevel::Summary,
                VerbosityLevel::Normal => DetailLevel::Standard,
                VerbosityLevel::Detailed => DetailLevel::Standard,
                VerbosityLevel::Verbose => DetailLevel::Full,
                VerbosityLevel::Debug => DetailLevel::Exhaustive,
            },
        }
    }

    /// Should show section?
    pub fn should_show(&self, context: OutputContext) -> bool {
        match context {
            OutputContext::InternalComms => self.show_internal_comms,
            OutputContext::Progress => self.show_progress,
            _ => true,
        }
    }

    /// Get max lines for section
    pub fn max_lines(&self) -> u32 {
        self.max_lines_per_section
    }

    /// Apply natural language change
    pub fn apply_change(&mut self, request: &str) -> Option<String> {
        let lower = request.to_lowercase();

        // Verbosity level changes
        if lower.contains("minimal") || lower.contains("brief") || lower.contains("short") {
            *self = Self::minimal();
            return Some("Verbosity set to minimal - just the essentials.".to_string());
        }
        if lower.contains("normal verbosity") || lower.contains("default verbosity") {
            *self = Self::default();
            return Some("Verbosity set to normal.".to_string());
        }
        if lower.contains("verbose") || lower.contains("detailed") || lower.contains("more detail") {
            *self = Self::verbose();
            return Some("Verbosity set to verbose - showing all details.".to_string());
        }
        if lower.contains("debug") || lower.contains("maximum detail") {
            *self = Self::debug();
            return Some("Debug verbosity enabled - maximum detail.".to_string());
        }

        // Individual toggles
        if lower.contains("show citation") || lower.contains("include citation") {
            self.show_citations = true;
            return Some("Citations will be shown.".to_string());
        }
        if lower.contains("hide citation") || lower.contains("no citation") {
            self.show_citations = false;
            return Some("Citations will be hidden.".to_string());
        }
        if lower.contains("show timestamp") || lower.contains("include time") {
            self.show_timestamps = true;
            return Some("Timestamps will be shown.".to_string());
        }
        if lower.contains("hide timestamp") || lower.contains("no timestamp") {
            self.show_timestamps = false;
            return Some("Timestamps will be hidden.".to_string());
        }
        if lower.contains("show confidence") {
            self.show_confidence = true;
            return Some("Confidence levels will be shown.".to_string());
        }
        if lower.contains("hide confidence") {
            self.show_confidence = false;
            return Some("Confidence levels will be hidden.".to_string());
        }
        if lower.contains("show progress") || lower.contains("show internal") {
            self.show_progress = true;
            self.show_internal_comms = true;
            return Some("Progress and internal communication will be shown.".to_string());
        }
        if lower.contains("hide progress") || lower.contains("quiet") {
            self.show_progress = false;
            return Some("Progress updates will be hidden.".to_string());
        }

        None
    }
}

/// Format verbosity config
pub fn format_verbosity_config(config: &VerbosityConfig) -> String {
    let mut output = String::new();
    output.push_str("=== Verbosity Configuration ===\n\n");

    output.push_str(&format!("Level: {}\n", config.level));
    output.push_str(&format!("Answer Detail: {}\n", config.answer_detail));
    output.push_str(&format!("Status Detail: {}\n", config.status_detail));
    output.push_str(&format!("Stats Detail: {}\n", config.stats_detail));
    output.push_str(&format!("Show Citations: {}\n", config.show_citations));
    output.push_str(&format!("Show Timestamps: {}\n", config.show_timestamps));
    output.push_str(&format!("Show Confidence: {}\n", config.show_confidence));
    output.push_str(&format!("Show Internal Comms: {}\n", config.show_internal_comms));
    output.push_str(&format!("Show Progress: {}\n", config.show_progress));
    output.push_str(&format!("Max Lines/Section: {}\n", config.max_lines_per_section));

    output
}

/// Check if query is verbosity-related
pub fn is_verbosity_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("verbosity")
        || lower.contains("detail level")
        || lower.contains("output level")
        || lower.contains("how much detail")
}

/// Fun fact about verbosity
pub fn verbosity_fun_fact() -> &'static str {
    "The right amount of detail is key - too little leaves you guessing, too much drowns the signal in noise!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verbosity_level_display() {
        assert_eq!(format!("{}", VerbosityLevel::Minimal), "Minimal");
        assert_eq!(format!("{}", VerbosityLevel::Verbose), "Verbose");
    }

    #[test]
    fn test_default_config() {
        let config = VerbosityConfig::default();
        assert_eq!(config.level, VerbosityLevel::Normal);
        assert!(config.show_citations);
        assert!(!config.show_timestamps);
    }

    #[test]
    fn test_minimal_preset() {
        let config = VerbosityConfig::minimal();
        assert_eq!(config.level, VerbosityLevel::Minimal);
        assert!(!config.show_citations);
        assert!(!config.show_internal_comms);
    }

    #[test]
    fn test_verbose_preset() {
        let config = VerbosityConfig::verbose();
        assert_eq!(config.level, VerbosityLevel::Verbose);
        assert!(config.show_timestamps);
        assert!(config.show_confidence);
    }

    #[test]
    fn test_is_verbose() {
        let config = VerbosityConfig::verbose();
        assert!(config.is_verbose());
        let normal = VerbosityConfig::default();
        assert!(!normal.is_verbose());
    }

    #[test]
    fn test_detail_for_context() {
        let config = VerbosityConfig::verbose();
        assert_eq!(config.detail_for(OutputContext::Answer), DetailLevel::Full);
        assert_eq!(config.detail_for(OutputContext::Greeting), DetailLevel::Full);
    }

    #[test]
    fn test_should_show() {
        let mut config = VerbosityConfig::default();
        assert!(config.should_show(OutputContext::InternalComms));
        config.show_internal_comms = false;
        assert!(!config.should_show(OutputContext::InternalComms));
    }

    #[test]
    fn test_apply_minimal() {
        let mut config = VerbosityConfig::default();
        let result = config.apply_change("use minimal output");
        assert!(result.is_some());
        assert_eq!(config.level, VerbosityLevel::Minimal);
    }

    #[test]
    fn test_apply_show_citations() {
        let mut config = VerbosityConfig::minimal();
        config.apply_change("show citations please");
        assert!(config.show_citations);
    }

    #[test]
    fn test_is_verbosity_query() {
        assert!(is_verbosity_query("Show verbosity settings"));
        assert!(is_verbosity_query("What's the detail level?"));
        assert!(!is_verbosity_query("Install vim"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = verbosity_fun_fact();
        assert!(fact.contains("detail"));
    }
}
