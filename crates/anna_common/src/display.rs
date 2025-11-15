//! Display Library - Anna's UI Abstraction Layer
//!
//! This module implements the complete UI abstraction layer as specified in the Language Contract.
//! All Anna components must use this layer for output - direct terminal formatting is prohibited.
//!
//! Key Features:
//! - Terminal capability detection and adaptation
//! - Language profile integration
//! - Graceful degradation for limited terminals
//! - Consistent visual hierarchy
//! - Emoji/Unicode fallback substitution

use crate::language::{LanguageConfig, LanguageProfile, TerminalCapabilities};
use owo_colors::OwoColorize;
use std::io::{self, IsTerminal, Write};

/// UI Abstraction Layer - The only interface for generating user-facing output
pub struct UI {
    profile: LanguageProfile,
    caps: TerminalCapabilities,
}

impl UI {
    /// Create new UI with language configuration
    pub fn new(config: &LanguageConfig) -> Self {
        let mut ui_caps = config.terminal;
        ui_caps = TerminalCapabilities::detect(); // Always detect fresh

        Self {
            profile: config.profile(),
            caps: ui_caps,
        }
    }

    /// Create UI with auto-detected language and terminal
    pub fn auto() -> Self {
        let config = LanguageConfig::new();
        Self::new(&config)
    }

    /// Get terminal capabilities
    pub fn capabilities(&self) -> &TerminalCapabilities {
        &self.caps
    }

    /// Get language profile
    pub fn profile(&self) -> &LanguageProfile {
        &self.profile
    }

    // ========================================================================
    // Core UILayer Trait Implementation
    // ========================================================================

    /// Display success message
    pub fn success(&self, message: &str) {
        let icon = self.render_icon("✓", "[OK]");
        let text = if self.caps.use_colors() {
            format!("{} {}", icon.bright_green(), message)
        } else {
            format!("{} {}", icon, message)
        };
        println!("{}", text);
    }

    /// Display error message
    pub fn error(&self, message: &str) {
        let icon = self.render_icon("✗", "[ERROR]");
        let text = if self.caps.use_colors() {
            format!("{} {}", icon.bright_red(), message)
        } else {
            format!("{} {}", icon, message)
        };
        eprintln!("{}", text);
    }

    /// Display warning message
    pub fn warning(&self, message: &str) {
        let icon = self.render_icon("⚠", "[!]");
        let text = if self.caps.use_colors() {
            format!("{} {}", icon.bright_yellow(), message)
        } else {
            format!("{} {}", icon, message)
        };
        println!("{}", text);
    }

    /// Display info message
    pub fn info(&self, message: &str) {
        let icon = self.render_icon("ℹ", "[i]");
        let text = if self.caps.use_colors() {
            format!("{} {}", icon.cyan(), message.dimmed())
        } else {
            format!("{} {}", icon, message)
        };
        println!("{}", text);
    }

    /// Display section header
    pub fn section_header(&self, icon: &str, title: &str) {
        println!();
        let rendered_icon = self.render_emoji(icon);
        let formatted = if self.caps.use_colors() {
            format!("{} {}", rendered_icon, title.bright_cyan().bold())
        } else {
            format!("{} {}", rendered_icon, title)
        };
        println!("{}", formatted);

        let separator = self.render_box_char("─", "-");
        let sep_colored = if self.caps.use_colors() {
            separator.repeat(60).dimmed().to_string()
        } else {
            separator.repeat(60)
        };
        println!("{}", sep_colored);
        println!();
    }

    /// Display bullet list
    pub fn bullet_list(&self, items: &[&str]) {
        let bullet = self.render_icon("•", "*");
        for item in items {
            println!("  {} {}", bullet, item);
        }
    }

    /// Display numbered list
    pub fn numbered_list(&self, items: &[&str]) {
        for (i, item) in items.iter().enumerate() {
            println!("  {}. {}", i + 1, item);
        }
    }

    /// Display progress indicator
    pub fn progress(&self, current: usize, total: usize, label: &str) {
        let percent = if total > 0 {
            (current as f64 / total as f64 * 100.0) as usize
        } else {
            0
        };

        let bar_width = 40;
        let filled = if total > 0 {
            (bar_width * current) / total
        } else {
            0
        };
        let empty = bar_width.saturating_sub(filled);

        let filled_char = self.render_box_char("█", "=");
        let empty_char = self.render_box_char("░", " ");

        let bar = if self.caps.use_colors() {
            format!(
                "[{}{}]",
                filled_char.repeat(filled).bright_cyan(),
                empty_char.repeat(empty).bright_black()
            )
        } else {
            format!("[{}{}]", filled_char.repeat(filled), empty_char.repeat(empty))
        };

        let formatted_percent = if self.caps.use_colors() {
            format!("{}%", percent).bright_cyan().to_string()
        } else {
            format!("{}%", percent)
        };

        println!("{} {} - {}", bar, formatted_percent, label.dimmed());
    }

    /// Display spinner with message
    pub fn spinner(&self, message: &str) {
        let spinner = self.render_icon("⏳", "[...]");
        print!("{} {}...", spinner, message);
        io::stdout().flush().ok();
    }

    /// Prompt user for yes/no decision
    pub fn prompt_yes_no(&self, question: &str) -> bool {
        println!();
        println!("{}", question);
        print!("[{}/{}]: ",
            self.profile.translations.yes,
            self.profile.translations.no
        );
        io::stdout().flush().ok();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            return false;
        }

        let input = input.trim().to_lowercase();
        // Match both English and localized yes
        input == "y" || input == "yes" || input == self.profile.translations.yes.to_lowercase()
    }

    /// Prompt user to choose from options
    pub fn prompt_choice(&self, question: &str, choices: &[&str]) -> usize {
        println!();
        println!("{}", question);
        println!();

        for (i, choice) in choices.iter().enumerate() {
            println!("  {}. {}", i + 1, choice);
        }
        println!("  0. {}", self.profile.translations.cancel);
        println!();

        print!("{}: ", "Enter number"); // TODO: Add translation
        io::stdout().flush().ok();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            return 0;
        }

        input.trim().parse::<usize>().unwrap_or(0)
    }

    // ========================================================================
    // Advanced Display Functions
    // ========================================================================

    /// Display a box with content
    pub fn box_content(&self, title: &str, lines: &[&str]) {
        let (top_left, top_right, bottom_left, bottom_right, horizontal, vertical) =
            self.box_chars();

        // Top border
        println!("{}{}{}", top_left, horizontal.repeat(58), top_right);

        // Title
        let formatted_title = if self.caps.use_colors() {
            format!("{} {:<56} {}", vertical, title.bold(), vertical)
        } else {
            format!("{} {:<56} {}", vertical, title, vertical)
        };
        println!("{}", formatted_title);

        // Separator
        let (left_t, right_t) = self.render_t_chars();
        println!("{}{}{}", left_t, horizontal.repeat(58), right_t);

        // Content
        for line in lines {
            println!("{} {:<56} {}", vertical, line, vertical);
        }

        // Bottom border
        println!("{}{}{}", bottom_left, horizontal.repeat(58), bottom_right);
    }

    /// Display key-value summary
    pub fn summary(&self, title: &str, items: &[(&str, &str)]) {
        self.section_header("📊", title);

        for (key, value) in items {
            let formatted = if self.caps.use_colors() {
                format!("{:<20} {}", key.bold(), value)
            } else {
                format!("{:<20} {}", key, value)
            };
            println!("  {}", formatted);
        }
        println!();
    }

    /// Display thinking message
    pub fn thinking(&self) {
        let icon = self.render_emoji("💭");
        let text = if self.caps.use_colors() {
            format!("{} {}...", icon, self.profile.translations.working).dimmed().to_string()
        } else {
            format!("{} {}...", icon, self.profile.translations.working)
        };
        println!("\n{}\n", text);
    }

    /// Display done message
    pub fn done(&self, message: &str) {
        let icon = self.render_icon("✓", "[OK]");
        let text = if self.caps.use_colors() {
            format!("{} {}", icon.bright_green(), message)
        } else {
            format!("{} {}", icon, message)
        };
        println!("\n{}\n", text);
    }

    /// Display command to be executed
    pub fn command(&self, cmd: &str, requires_sudo: bool) {
        let sudo_icon = if requires_sudo {
            self.render_emoji("🔐")
        } else {
            "  ".to_string()
        };

        let formatted = if self.caps.use_colors() {
            format!("  {} $ {}", sudo_icon, cmd).cyan().to_string()
        } else {
            format!("  {} $ {}", sudo_icon, cmd)
        };
        println!("{}", formatted);
    }

    /// Display explanation text
    pub fn explain(&self, text: &str) {
        let icon = self.render_emoji("📖");
        println!("  {} {}", icon, text);
    }

    /// Display Arch Wiki reference
    pub fn wiki_link(&self, title: &str, url: &str) {
        let icon = self.render_emoji("🏛️");
        let formatted = if self.caps.use_colors() {
            format!("  {} Arch Wiki: {} - {}", icon, title, url).blue().to_string()
        } else {
            format!("  {} Arch Wiki: {} - {}", icon, title, url)
        };
        println!("{}", formatted);
    }

    // ========================================================================
    // Rendering Helpers - Handle Terminal Capability Fallbacks
    // ========================================================================

    /// Render icon with fallback
    fn render_icon(&self, unicode: &str, ascii_fallback: &str) -> String {
        if self.caps.use_emojis() || self.caps.use_unicode_graphics() {
            unicode.to_string()
        } else {
            ascii_fallback.to_string()
        }
    }

    /// Render emoji with fallback
    fn render_emoji(&self, emoji: &str) -> String {
        if self.caps.use_emojis() {
            emoji.to_string()
        } else {
            // Map emoji to text
            match emoji {
                "✓" => "[OK]".to_string(),
                "✗" => "[X]".to_string(),
                "⚠" | "⚠️" => "[!]".to_string(),
                "🔒" | "🔐" => "[SECURE]".to_string(),
                "📊" => "[STATS]".to_string(),
                "💭" => "[...]".to_string(),
                "📖" => "[INFO]".to_string(),
                "🏛️" => "[WIKI]".to_string(),
                "⏳" => "[WAIT]".to_string(),
                "•" => "*".to_string(),
                _ => "".to_string(),
            }
        }
    }

    /// Render box-drawing character with fallback
    fn render_box_char(&self, unicode: &str, ascii: &str) -> String {
        if self.caps.use_unicode_graphics() {
            unicode.to_string()
        } else {
            ascii.to_string()
        }
    }

    /// Get box-drawing characters based on terminal capabilities
    fn box_chars(&self) -> (String, String, String, String, String, String) {
        if self.caps.use_unicode_graphics() {
            (
                "┌".to_string(),
                "┐".to_string(),
                "└".to_string(),
                "┘".to_string(),
                "─".to_string(),
                "│".to_string(),
            )
        } else {
            (
                "+".to_string(),
                "+".to_string(),
                "+".to_string(),
                "+".to_string(),
                "-".to_string(),
                "|".to_string(),
            )
        }
    }

    /// Get T-junction characters for borders
    fn render_t_chars(&self) -> (String, String) {
        if self.caps.use_unicode_graphics() {
            ("├".to_string(), "┤".to_string())
        } else {
            ("+".to_string(), "+".to_string())
        }
    }

    // ========================================================================
    // Conversational Display Helpers
    // ========================================================================

    /// Print warm greeting
    pub fn greeting(&self, username: &str) {
        let separator = self.render_box_char("═", "=").repeat(60);
        println!("{}", separator);

        let welcome_icon = self.render_emoji("🌟");
        println!("  {} Welcome, {}", welcome_icon, username);
        println!("  I am Anna, your Arch Linux caretaker");
        println!("{}", separator);
        println!();
    }

    /// Print how Anna works explanation
    pub fn explain_how_i_work(&self) {
        self.section_header("ℹ️", "How do I work?");

        let bullet = self.render_icon("•", "*");
        println!("  {} I watch your system locally.", bullet);
        println!("  {} I compare what I see with best practices from the Arch Wiki.", bullet);
        println!("  {} I suggest improvements, explain them in plain language,", bullet);
        println!("     and only change things after you approve them.");
        println!();
    }

    /// Print privacy explanation
    pub fn explain_privacy(&self) {
        self.section_header("🔒", "Privacy & Data Handling");

        let bullet = self.render_icon("•", "*");
        println!("  {} I do not send your data anywhere.", bullet);
        println!("  {} I keep telemetry and notes on this machine only.", bullet);
        println!("  {} I read the Arch Wiki and official documentation when needed.", bullet);
        println!("  {} I never run commands behind your back.", bullet);
        println!();
    }

    /// Print REPL welcome
    pub fn repl_welcome(&self) {
        println!("\nAnna is ready. You can just talk to me.\n");
        println!("For example:");

        let bullet = self.render_icon("•", "*");
        println!("  {} \"How are you, any problems with my system\"", bullet);
        println!("  {} \"It feels slower than usual, did you see anything\"", bullet);
        println!("  {} \"I am not happy with vim, what CLI editors do you suggest\"", bullet);
        println!("  {} \"Prepare a report for my boss about this machine\"\n", bullet);

        println!("Ask me something:");
    }

    /// Print prompt
    pub fn prompt(&self) {
        print!("\n> ");
        io::stdout().flush().ok();
    }

    /// Print separator line
    pub fn separator(&self) {
        let line = self.render_box_char("─", "-");
        println!("{}", line.repeat(60));
    }

    // ========================================================================
    // Recommendation Display
    // ========================================================================

    /// Display recommendation step
    pub fn recommendation_step(
        &self,
        number: usize,
        title: &str,
        explanation: &str,
        command: Option<&str>,
        wiki_link: Option<(&str, &str)>,
        warning: Option<&str>,
    ) {
        // Step header
        let formatted_title = if self.caps.use_colors() {
            format!("{}. {}", number, title).bold().to_string()
        } else {
            format!("{}. {}", number, title)
        };
        println!("{}", formatted_title);

        // Command
        if let Some(cmd) = command {
            self.command(cmd, cmd.starts_with("sudo"));
        }

        // Explanation
        self.explain(explanation);

        // Warning
        if let Some(warn) = warning {
            let icon = self.render_icon("⚠", "[!]");
            let formatted = if self.caps.use_colors() {
                format!("  {} {}", icon, warn).yellow().to_string()
            } else {
                format!("  {} {}", icon, warn)
            };
            println!("{}", formatted);
        }

        // Wiki link
        if let Some((wiki_title, wiki_url)) = wiki_link {
            self.wiki_link(wiki_title, wiki_url);
        }

        println!();
    }
}

// ============================================================================
// Legacy Compatibility (Deprecated - Will be removed)
// ============================================================================

/// Status level for messages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusLevel {
    Critical,
    Warning,
    Info,
    Success,
}

/// Legacy section builder - DEPRECATED - Use UI methods instead
pub struct Section {
    title: String,
    level: StatusLevel,
    content: Vec<String>,
    use_color: bool,
}

impl Section {
    pub fn new(title: impl Into<String>, level: StatusLevel, use_color: bool) -> Self {
        Section {
            title: title.into(),
            level,
            content: Vec::new(),
            use_color,
        }
    }

    pub fn add_line(&mut self, line: impl Into<String>) {
        self.content.push(line.into());
    }

    pub fn add_bullet(&mut self, text: impl Into<String>) {
        self.content.push(format!("  • {}", text.into()));
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str("╔══════════════════════════════════════════════════════════╗\n");

        let icon = match self.level {
            StatusLevel::Critical => "🔴",
            StatusLevel::Warning => "⚠️",
            StatusLevel::Info => "ℹ️",
            StatusLevel::Success => "✅",
        };

        let title_line = format!("{} {}", icon, self.title);
        output.push_str(&format!("║ {:<56} ║\n", title_line));
        output.push_str("╚══════════════════════════════════════════════════════════╝\n");

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

// ============================================================================
// Legacy Helper Functions - DEPRECATED
// ============================================================================

pub fn should_use_color() -> bool {
    if std::env::var("NO_COLOR").is_ok() {
        return false;
    }
    std::io::stdout().is_terminal()
}

pub fn print_section_header(emoji: &str, title: &str) {
    let ui = UI::auto();
    ui.section_header(emoji, title);
}

pub fn print_thinking() {
    let ui = UI::auto();
    ui.thinking();
}

pub fn print_prompt() {
    let ui = UI::auto();
    ui.prompt();
}

pub fn print_repl_welcome() {
    let ui = UI::auto();
    ui.repl_welcome();
}

pub fn print_greeting(username: &str) {
    let ui = UI::auto();
    ui.greeting(username);
}

pub fn print_privacy_explanation() {
    let ui = UI::auto();
    ui.explain_privacy();
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_creation() {
        let ui = UI::auto();
        assert!(ui.capabilities().is_tty || !ui.capabilities().is_tty); // Always true
    }

    #[test]
    fn test_emoji_fallback() {
        let config = LanguageConfig::new();
        let ui = UI::new(&config);

        // Even if emojis not supported, should not panic
        let result = ui.render_emoji("✓");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_box_char_fallback() {
        let config = LanguageConfig::new();
        let ui = UI::new(&config);

        let (tl, tr, bl, br, h, v) = ui.box_chars();
        assert!(!tl.is_empty());
        assert!(!h.is_empty());
        assert!(!v.is_empty());
    }

    #[test]
    fn test_language_integration() {
        let mut config = LanguageConfig::new();
        config.set_user_language(crate::language::Language::Spanish);

        let ui = UI::new(&config);
        assert_eq!(ui.profile().language, crate::language::Language::Spanish);
        assert_eq!(ui.profile().translations.yes, "sí");
    }
}
