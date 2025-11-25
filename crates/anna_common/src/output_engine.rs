//! Professional Output Engine v1 - v6.31.0
//!
//! Provides deterministic, consistent, beautiful formatting for all Anna output.
//! This replaces ad-hoc formatting across subsystems with unified rules.
//!
//! ## Design Principles
//! - Deterministic (no LLM, pure logic)
//! - Terminal-aware (color only when supported)
//! - Consistent spacing and alignment
//! - Professional visual hierarchy
//! - Zero markdown fences in output
//! - Command-first readability

use crate::insights_engine::{Insight, InsightSeverity};
use crate::predictive_diagnostics::PredictiveInsight;
use owo_colors::OwoColorize;

/// Terminal capability detection
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TerminalMode {
    Color,  // Full ANSI color support
    Basic,  // ASCII-only, no color
}

impl TerminalMode {
    /// Detect terminal capabilities
    pub fn detect() -> Self {
        if console::Term::stdout().features().colors_supported() {
            TerminalMode::Color
        } else {
            TerminalMode::Basic
        }
    }
}

/// Professional Output Engine
pub struct OutputEngine {
    mode: TerminalMode,
    use_emoji: bool,
}

impl OutputEngine {
    /// Create new output engine with detected terminal capabilities
    pub fn new() -> Self {
        Self {
            mode: TerminalMode::detect(),
            use_emoji: true,
        }
    }

    /// Create engine with explicit mode (for testing)
    pub fn with_mode(mode: TerminalMode) -> Self {
        Self {
            mode,
            use_emoji: mode == TerminalMode::Color,
        }
    }

    /// Format a top-level header
    pub fn format_header(&self, title: &str) -> String {
        let emoji = if self.use_emoji { "📋  " } else { "" };
        match self.mode {
            TerminalMode::Color => format!("{}{}", emoji, title.bold().bright_cyan()),
            TerminalMode::Basic => format!("{}{}", emoji, title),
        }
    }

    /// Format a section subheader
    pub fn format_subheader(&self, title: &str) -> String {
        let emoji = if self.use_emoji { "▪  " } else { "• " };
        match self.mode {
            TerminalMode::Color => format!("{}{}", emoji, title.bold()),
            TerminalMode::Basic => format!("{}{}", emoji, title),
        }
    }

    /// Format a section subheader with custom emoji (v6.32.0)
    pub fn format_subheader_with_emoji(&self, emoji: &str, title: &str) -> String {
        let display_emoji = if self.use_emoji {
            format!("{}  ", emoji)
        } else {
            String::new()
        };
        match self.mode {
            TerminalMode::Color => format!("{}{}", display_emoji, title.bold().bright_cyan()),
            TerminalMode::Basic => format!("{}{}", display_emoji, title),
        }
    }

    /// Format a complete section with title and body
    pub fn format_section(&self, title: &str, body: &str) -> String {
        let header = self.format_subheader(title);
        let body_clean = self.strip_markdown_fences(body);
        format!("{}\n{}\n", header, body_clean)
    }

    /// Format a list of commands with professional styling
    pub fn format_command_list(&self, commands: Vec<String>) -> String {
        let mut output = String::new();

        for cmd in commands {
            let formatted = self.format_command(&cmd);
            output.push_str(&formatted);
            output.push('\n');
        }

        output
    }

    /// Format a single command with [CMD] prefix
    pub fn format_command(&self, cmd: &str) -> String {
        match self.mode {
            TerminalMode::Color => {
                format!("  [{}] {}", "CMD".dimmed(), cmd.bright_white())
            }
            TerminalMode::Basic => {
                format!("  [CMD] {}", cmd)
            }
        }
    }

    /// Format an insight with color-coded severity
    pub fn format_insight(&self, insight: &Insight) -> String {
        let severity_marker = match insight.severity {
            InsightSeverity::Critical => {
                if self.mode == TerminalMode::Color {
                    "🔴".to_string()
                } else {
                    "[!]".to_string()
                }
            }
            InsightSeverity::Warning => {
                if self.mode == TerminalMode::Color {
                    "⚠️".to_string()
                } else {
                    "[*]".to_string()
                }
            }
            InsightSeverity::Info => {
                if self.mode == TerminalMode::Color {
                    "ℹ️".to_string()
                } else {
                    "[i]".to_string()
                }
            }
        };

        let title_formatted = match self.mode {
            TerminalMode::Color => match insight.severity {
                InsightSeverity::Critical => insight.title.bright_red().bold().to_string(),
                InsightSeverity::Warning => insight.title.bright_yellow().bold().to_string(),
                InsightSeverity::Info => insight.title.bright_blue().to_string(),
            },
            TerminalMode::Basic => insight.title.clone(),
        };

        let mut output = format!("{}  {}\n", severity_marker, title_formatted);
        output.push_str(&format!("   {}\n", insight.explanation));

        if let Some(ref suggestion) = insight.suggestion {
            let arrow = if self.mode == TerminalMode::Color { "→" } else { "->" };
            output.push_str(&format!("   {} {}\n", arrow, suggestion));
        }

        output
    }

    /// Format a predictive insight
    pub fn format_prediction(&self, prediction: &PredictiveInsight) -> String {
        let icon = if self.mode == TerminalMode::Color {
            "🔮  "
        } else {
            "[PRED] "
        };

        let title_formatted = match self.mode {
            TerminalMode::Color => prediction.title.bright_magenta().bold().to_string(),
            TerminalMode::Basic => prediction.title.clone(),
        };

        let mut output = format!("{}{}\n", icon, title_formatted);
        output.push_str(&format!("   Window: {}\n", prediction.prediction_window));

        if let Some(ref cause) = prediction.cause {
            output.push_str(&format!("   Cause: {}\n", cause));
        }

        if !prediction.recommended_actions.is_empty() {
            let arrow = if self.mode == TerminalMode::Color { "→" } else { "->" };
            for action in &prediction.recommended_actions {
                output.push_str(&format!("   {} {}\n", arrow, action));
            }
        }

        output
    }

    /// Format insight summary section
    pub fn format_summary(&self, summary: &str) -> String {
        self.strip_markdown_fences(summary)
    }

    /// Strip markdown code fences from text
    pub fn strip_markdown_fences(&self, text: &str) -> String {
        let mut result = String::new();
        let mut in_code_block = false;

        for line in text.lines() {
            if line.starts_with("```") {
                in_code_block = !in_code_block;
                continue;
            }

            if in_code_block {
                // Format as command
                result.push_str(&self.format_command(line.trim()));
                result.push('\n');
            } else {
                result.push_str(line);
                result.push('\n');
            }
        }

        result.trim_end().to_string()
    }

    /// Remove all color codes for basic terminals
    pub fn strip_color(text: &str) -> String {
        // Remove ANSI escape sequences
        let ansi_regex = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
        ansi_regex.replace_all(text, "").to_string()
    }

    /// Format a multi-section system report
    pub fn format_system_report(
        &self,
        summary: Option<String>,
        health: Option<String>,
        insights: Vec<Insight>,
        predictions: Vec<PredictiveInsight>,
        recommendations: Option<String>,
    ) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&self.format_header("System Report"));
        output.push_str("\n\n");

        // Summary section
        if let Some(summary_text) = summary {
            output.push_str(&self.format_section("Summary", &summary_text));
            output.push('\n');
        }

        // Health section
        if let Some(health_text) = health {
            output.push_str(&self.format_section("Health", &health_text));
            output.push('\n');
        }

        // Insights section
        if !insights.is_empty() {
            output.push_str(&self.format_subheader("Insights"));
            output.push('\n');
            for insight in insights {
                output.push_str(&self.format_insight(&insight));
            }
            output.push('\n');
        }

        // Predictions section
        if !predictions.is_empty() {
            output.push_str(&self.format_subheader("Predictions"));
            output.push('\n');
            for prediction in predictions {
                output.push_str(&self.format_prediction(&prediction));
            }
            output.push('\n');
        }

        // Recommendations section
        if let Some(rec_text) = recommendations {
            output.push_str(&self.format_section("Recommendations", &rec_text));
        }

        output.trim_end().to_string()
    }

    /// Format a simple key-value list
    pub fn format_key_value(&self, key: &str, value: &str) -> String {
        match self.mode {
            TerminalMode::Color => {
                format!("  {}  {}", key.bold(), value)
            }
            TerminalMode::Basic => {
                format!("  {}  {}", key, value)
            }
        }
    }

    /// Format a bulleted list item
    pub fn format_bullet(&self, text: &str) -> String {
        let bullet = if self.mode == TerminalMode::Color { "•" } else { "-" };
        format!("  {} {}", bullet, text)
    }

    /// Format a compact one-line answer with optional source
    /// v6.34.0: For simple capability checks and fact queries
    pub fn format_compact(&self, main_line: &str, source: Option<&str>) -> String {
        let mut output = main_line.to_string();
        if let Some(src) = source {
            output.push_str(&format!("\n\n{}", self.format_source_line(src)));
        }
        output
    }

    /// Format a source attribution line
    pub fn format_source_line(&self, source: &str) -> String {
        match self.mode {
            TerminalMode::Color => format!("Source: {}", source.dimmed()),
            TerminalMode::Basic => format!("Source: {}", source),
        }
    }

    /// Format a numbered list (for step-by-step instructions)
    /// v6.34.0: For wiki reasoning and procedural answers
    pub fn format_numbered_list(&self, items: Vec<String>) -> String {
        let mut output = String::new();
        for (idx, item) in items.iter().enumerate() {
            output.push_str(&format!("  {}. {}\n", idx + 1, item));
        }
        output.trim_end().to_string()
    }

    /// Validate that text contains no markdown fences
    /// v6.34.0: For testing and assertion
    pub fn validate_no_fences(text: &str) -> bool {
        !text.contains("```")
    }
}

/// Answer style guidelines (v6.34.0)
/// Maps intent types to consistent output formatting styles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnswerStyle {
    /// One-line or short block (CapabilityCheck, simple facts)
    Compact,
    /// Header + multiple sections (SystemStatus, SystemReport, Diagnostics)
    Sectioned,
    /// Numbered or bulleted steps (WikiReasoning, how-to questions)
    Stepwise,
    /// High-level summary (Insights, self-tuning reports)
    Summary,
}

impl AnswerStyle {
    /// Determine the appropriate style for a given intent
    pub fn from_intent(intent: &str) -> Self {
        match intent {
            "CapabilityCheck" | "SimpleFact" => AnswerStyle::Compact,
            "SystemStatus" | "SystemReport" | "Diagnostics" => AnswerStyle::Sectioned,
            "WikiReasoning" | "HowTo" | "DiskExplorer" => AnswerStyle::Stepwise,
            "InsightSummary" | "SelfTuning" => AnswerStyle::Summary,
            _ => AnswerStyle::Sectioned, // Safe default
        }
    }
}

impl Default for OutputEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::insights_engine::Insight;

    #[test]
    fn test_format_header_basic() {
        let engine = OutputEngine::with_mode(TerminalMode::Basic);
        let result = engine.format_header("Test Header");
        assert_eq!(result, "Test Header");
    }

    #[test]
    fn test_format_header_color() {
        let engine = OutputEngine::with_mode(TerminalMode::Color);
        let result = engine.format_header("Test Header");
        assert!(result.contains("Test Header"));
        assert!(result.contains("📋"));
    }

    #[test]
    fn test_format_subheader_basic() {
        let engine = OutputEngine::with_mode(TerminalMode::Basic);
        let result = engine.format_subheader("Subsection");
        assert_eq!(result, "• Subsection");
    }

    #[test]
    fn test_format_command_basic() {
        let engine = OutputEngine::with_mode(TerminalMode::Basic);
        let result = engine.format_command("df -h");
        assert_eq!(result, "  [CMD] df -h");
    }

    #[test]
    fn test_format_command_list() {
        let engine = OutputEngine::with_mode(TerminalMode::Basic);
        let commands = vec!["ls -la".to_string(), "df -h".to_string()];
        let result = engine.format_command_list(commands);
        assert!(result.contains("[CMD] ls -la"));
        assert!(result.contains("[CMD] df -h"));
    }

    #[test]
    fn test_strip_markdown_fences() {
        let engine = OutputEngine::with_mode(TerminalMode::Basic);
        let input = "Some text\n```bash\ndf -h\nls -la\n```\nMore text";
        let result = engine.strip_markdown_fences(input);
        assert!(!result.contains("```"));
        assert!(result.contains("[CMD] df -h"));
        assert!(result.contains("[CMD] ls -la"));
    }

    #[test]
    fn test_format_insight_critical() {
        let engine = OutputEngine::with_mode(TerminalMode::Basic);
        let insight = Insight::new(
            "test_detector",
            InsightSeverity::Critical,
            "Critical Issue",
            "This is critical"
        ).with_suggestion("Fix it now");

        let result = engine.format_insight(&insight);
        assert!(result.contains("[!]"));
        assert!(result.contains("Critical Issue"));
        assert!(result.contains("This is critical"));
        assert!(result.contains("-> Fix it now"));
    }

    #[test]
    fn test_format_key_value() {
        let engine = OutputEngine::with_mode(TerminalMode::Basic);
        let result = engine.format_key_value("Status", "Healthy");
        assert_eq!(result, "  Status  Healthy");
    }

    #[test]
    fn test_format_bullet() {
        let engine = OutputEngine::with_mode(TerminalMode::Basic);
        let result = engine.format_bullet("First item");
        assert_eq!(result, "  - First item");
    }

    #[test]
    fn test_strip_color() {
        let colored = "\x1b[31mRed text\x1b[0m";
        let result = OutputEngine::strip_color(colored);
        assert_eq!(result, "Red text");
    }
}
