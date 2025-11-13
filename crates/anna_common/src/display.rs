//! Display Library - Anna's Voice
//!
//! This module provides consistent, beautiful output formatting for all Anna commands.
//! Every message Anna shows should go through this library to maintain consistency.
//!
//! Design principles:
//! - Clear visual hierarchy with boxes and sections
//! - Consistent use of colors and icons
//! - Plain English explanations
//! - Actionable recommendations
//! - Proper spacing and formatting

use owo_colors::OwoColorize;
use std::fmt;

/// Status level for messages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusLevel {
    /// Critical issue requiring immediate attention
    Critical,
    /// Warning that should be addressed
    Warning,
    /// Informational message
    Info,
    /// Success/all clear
    Success,
}

impl StatusLevel {
    /// Get the icon for this status level
    pub fn icon(&self, use_color: bool) -> String {
        if use_color {
            match self {
                StatusLevel::Critical => "🔴".red().to_string(),
                StatusLevel::Warning => "⚠️ ".yellow().to_string(),
                StatusLevel::Info => "ℹ️ ".blue().to_string(),
                StatusLevel::Success => "✅".green().to_string(),
            }
        } else {
            match self {
                StatusLevel::Critical => "CRITICAL".to_string(),
                StatusLevel::Warning => "WARNING".to_string(),
                StatusLevel::Info => "INFO".to_string(),
                StatusLevel::Success => "OK".to_string(),
            }
        }
    }

    /// Get the label for this status level
    pub fn label(&self) -> &'static str {
        match self {
            StatusLevel::Critical => "CRITICAL",
            StatusLevel::Warning => "WARNING",
            StatusLevel::Info => "INFO",
            StatusLevel::Success => "OK",
        }
    }
}

/// Builder for creating consistent output sections
pub struct Section {
    title: String,
    level: StatusLevel,
    content: Vec<String>,
    use_color: bool,
}

impl Section {
    /// Create a new section with a title and status level
    pub fn new(title: impl Into<String>, level: StatusLevel, use_color: bool) -> Self {
        Section {
            title: title.into(),
            level,
            content: Vec::new(),
            use_color,
        }
    }

    /// Add a line of content
    pub fn add_line(&mut self, line: impl Into<String>) {
        self.content.push(line.into());
    }

    /// Add a blank line
    pub fn add_blank(&mut self) {
        self.content.push(String::new());
    }

    /// Add a bulleted item
    pub fn add_bullet(&mut self, text: impl Into<String>) {
        self.content.push(format!("  • {}", text.into()));
    }

    /// Add a numbered item
    pub fn add_numbered(&mut self, num: usize, text: impl Into<String>) {
        self.content.push(format!("  {}. {}", num, text.into()));
    }

    /// Add an indented detail
    pub fn add_detail(&mut self, text: impl Into<String>) {
        self.content.push(format!("     {}", text.into()));
    }

    /// Render the section as a string
    pub fn render(&self) -> String {
        let mut output = String::new();

        // Top border
        output.push_str("╔══════════════════════════════════════════════════════════╗\n");

        // Title line with icon
        let icon = self.level.icon(self.use_color);
        let title = if self.use_color {
            match self.level {
                StatusLevel::Critical => format!("{} {}", icon, self.title).red().bold().to_string(),
                StatusLevel::Warning => format!("{} {}", icon, self.title).yellow().bold().to_string(),
                StatusLevel::Info => format!("{} {}", icon, self.title).blue().bold().to_string(),
                StatusLevel::Success => format!("{} {}", icon, self.title).green().bold().to_string(),
            }
        } else {
            format!("{} {}", icon, self.title)
        };
        output.push_str(&format!("║ {:<56} ║\n", title));

        // Bottom border
        output.push_str("╚══════════════════════════════════════════════════════════╝\n");

        // Content
        if !self.content.is_empty() {
            output.push('\n');
            for line in &self.content {
                output.push_str(line);
                output.push('\n');
            }
        }

        output
    }
}

/// Builder for recommendations
pub struct Recommendation {
    steps: Vec<RecommendationStep>,
    use_color: bool,
}

#[derive(Clone)]
pub struct RecommendationStep {
    pub number: usize,
    pub title: String,
    pub command: Option<String>,
    pub explanation: String,
    pub warning: Option<String>,
    pub wiki_link: Option<WikiLink>,
    pub estimated_impact: Option<String>,
}

#[derive(Clone)]
pub struct WikiLink {
    pub title: String,
    pub url: String,
    pub section: Option<String>,
}

impl Recommendation {
    /// Create a new recommendation builder
    pub fn new(use_color: bool) -> Self {
        Recommendation {
            steps: Vec::new(),
            use_color,
        }
    }

    /// Add a recommendation step
    pub fn add_step(&mut self, step: RecommendationStep) {
        self.steps.push(step);
    }

    /// Render recommendations
    pub fn render(&self) -> String {
        if self.steps.is_empty() {
            return String::new();
        }

        let mut output = String::new();

        // Header
        let header = if self.use_color {
            "🎯 What You Should Do".bold().to_string()
        } else {
            "RECOMMENDED ACTIONS".to_string()
        };
        output.push_str(&header);
        output.push_str("\n\n");

        // Each step
        for step in &self.steps {
            // Step number and title
            let step_header = if self.use_color {
                format!("{}. {}", step.number, step.title).bold().to_string()
            } else {
                format!("{}. {}", step.number, step.title)
            };
            output.push_str(&step_header);
            output.push('\n');

            // Command if provided
            if let Some(cmd) = &step.command {
                let formatted_cmd = if self.use_color {
                    format!("   $ {}", cmd).cyan().to_string()
                } else {
                    format!("   $ {}", cmd)
                };
                output.push_str(&formatted_cmd);
                output.push('\n');
            }

            // Explanation
            output.push_str(&format!("   📖 {}\n", step.explanation));

            // Warning if provided
            if let Some(warning) = &step.warning {
                let formatted_warning = if self.use_color {
                    format!("   ⚠️  {}", warning).yellow().to_string()
                } else {
                    format!("   WARNING: {}", warning)
                };
                output.push_str(&formatted_warning);
                output.push('\n');
            }

            // Estimated impact
            if let Some(impact) = &step.estimated_impact {
                let formatted_impact = if self.use_color {
                    format!("   💾 Impact: {}", impact).green().to_string()
                } else {
                    format!("   Impact: {}", impact)
                };
                output.push_str(&formatted_impact);
                output.push('\n');
            }

            // Wiki link if provided
            if let Some(wiki) = &step.wiki_link {
                let link_text = if let Some(section) = &wiki.section {
                    format!("   🔗 Arch Wiki: {} - {} ({})", wiki.title, section, wiki.url)
                } else {
                    format!("   🔗 Arch Wiki: {} ({})", wiki.title, wiki.url)
                };
                let formatted_link = if self.use_color {
                    link_text.blue().to_string()
                } else {
                    link_text
                };
                output.push_str(&formatted_link);
                output.push('\n');
            }

            output.push('\n');
        }

        output
    }
}

/// Display a summary box
pub fn summary_box(title: &str, items: &[(&str, &str)], use_color: bool) -> String {
    let mut output = String::new();

    // Top border
    output.push_str("┌────────────────────────────────────────────────────────┐\n");

    // Title
    let formatted_title = if use_color {
        format!("│ {}  │\n", title.bold())
    } else {
        format!("│ {}  │\n", title)
    };
    output.push_str(&formatted_title);

    // Separator
    output.push_str("├────────────────────────────────────────────────────────┤\n");

    // Items
    for (key, value) in items {
        output.push_str(&format!("│ {:<20} {:<33} │\n", key, value));
    }

    // Bottom border
    output.push_str("└────────────────────────────────────────────────────────┘\n");

    output
}

/// Display a progress indicator
pub fn progress(current: usize, total: usize, label: &str, use_color: bool) -> String {
    let percent = (current as f64 / total as f64 * 100.0) as usize;
    let bar_width = 40;
    let filled = (bar_width * current) / total;
    let empty = bar_width - filled;

    let bar = if use_color {
        format!(
            "[{}{}]",
            "█".repeat(filled).green(),
            "░".repeat(empty).dimmed()
        )
    } else {
        format!("[{}{}]", "=".repeat(filled), " ".repeat(empty))
    };

    format!("{} {}% - {}", bar, percent, label)
}

/// Check if color output should be used
pub fn should_use_color() -> bool {
    // Check NO_COLOR environment variable
    if std::env::var("NO_COLOR").is_ok() {
        return false;
    }

    // Check if output is a TTY
    atty::is(atty::Stream::Stdout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_section_creation() {
        let mut section = Section::new("Test Section", StatusLevel::Info, false);
        section.add_line("Line 1");
        section.add_bullet("Bullet 1");
        section.add_numbered(1, "Step 1");

        let output = section.render();
        assert!(output.contains("Test Section"));
        assert!(output.contains("Line 1"));
        assert!(output.contains("Bullet 1"));
        assert!(output.contains("Step 1"));
    }

    #[test]
    fn test_recommendation_builder() {
        let mut rec = Recommendation::new(false);
        rec.add_step(RecommendationStep {
            number: 1,
            title: "Test Step".to_string(),
            command: Some("test command".to_string()),
            explanation: "This is a test".to_string(),
            warning: None,
            wiki_link: None,
            estimated_impact: Some("High".to_string()),
        });

        let output = rec.render();
        assert!(output.contains("Test Step"));
        assert!(output.contains("test command"));
        assert!(output.contains("This is a test"));
    }
}
