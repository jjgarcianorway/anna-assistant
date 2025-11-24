//! Professional Output Styling (6.17.0)
//!
//! Central formatting library with capability detection and configuration support.
//! Ensures Anna's output looks professional for seasoned Linux engineers.
//!
//! Features:
//! - Terminal capability detection (TTY, color depth, emoji support)
//! - Configuration-driven emoji/color control
//! - Consistent formatting across all commands
//! - ASCII fallbacks for all output
//! - ANSI-strippable for testing

use std::env;
use std::io::IsTerminal;

// ============================================================================
// Capability Detection
// ============================================================================

/// Terminal capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalCapabilities {
    pub is_tty: bool,
    pub color_depth: ColorDepth,
    pub supports_emoji: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorDepth {
    None,
    Basic,      // 8 colors
    Extended,   // 256 colors
    TrueColor,  // 24-bit
}

impl TerminalCapabilities {
    /// Detect terminal capabilities from environment
    pub fn detect() -> Self {
        let is_tty = std::io::stdout().is_terminal();

        let color_depth = if !is_tty {
            ColorDepth::None
        } else if let Ok(colorterm) = env::var("COLORTERM") {
            if colorterm.contains("truecolor") || colorterm.contains("24bit") {
                ColorDepth::TrueColor
            } else {
                ColorDepth::Extended
            }
        } else if env::var("TERM").ok().as_deref() == Some("dumb") {
            ColorDepth::None
        } else {
            // Check tput colors if available
            ColorDepth::Extended // Safe default for modern terminals
        };

        let supports_emoji = is_tty && detect_emoji_support();

        Self {
            is_tty,
            color_depth,
            supports_emoji,
        }
    }
}

fn detect_emoji_support() -> bool {
    // Check if LANG/LC_ALL contains UTF-8
    if let Ok(lang) = env::var("LANG") {
        if lang.to_lowercase().contains("utf") {
            return true;
        }
    }
    if let Ok(lc_all) = env::var("LC_ALL") {
        if lc_all.to_lowercase().contains("utf") {
            return true;
        }
    }
    // Default to false if uncertain
    false
}

// ============================================================================
// Output Configuration
// ============================================================================

/// User preferences for output styling
#[derive(Debug, Clone)]
pub struct OutputConfig {
    pub emoji_mode: EmojiMode,
    pub color_mode: ColorMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmojiMode {
    Auto,
    Enabled,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Auto,
    Basic,
    None,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            emoji_mode: EmojiMode::Auto,
            color_mode: ColorMode::Auto,
        }
    }
}

// ============================================================================
// Severity Enum
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub enum Severity {
    Ok,
    Info,
    Warning,
    Critical,
}

// ============================================================================
// Output Formatter
// ============================================================================

/// Professional output formatter with capability awareness
pub struct OutputFormatter {
    caps: TerminalCapabilities,
    config: OutputConfig,
    use_emoji: bool,
    use_color: bool,
}

impl OutputFormatter {
    /// Create formatter with detected capabilities and config
    pub fn new(config: OutputConfig) -> Self {
        let caps = TerminalCapabilities::detect();

        let use_emoji = match config.emoji_mode {
            EmojiMode::Auto => caps.supports_emoji,
            EmojiMode::Enabled => true,
            EmojiMode::Disabled => false,
        };

        let use_color = match config.color_mode {
            ColorMode::Auto => caps.color_depth != ColorDepth::None,
            ColorMode::Basic => true,
            ColorMode::None => false,
        };

        Self {
            caps,
            config,
            use_emoji,
            use_color,
        }
    }

    /// Create with default config
    pub fn default() -> Self {
        Self::new(OutputConfig::default())
    }

    // ========================================================================
    // Severity Levels
    // ========================================================================

    /// Format a heading
    pub fn heading(&self, text: &str) -> String {
        if self.use_color {
            format!("\x1b[1;36m{}\x1b[0m", text) // Bold cyan
        } else {
            text.to_string()
        }
    }

    /// Format a subheading with optional emoji
    pub fn subheading(&self, emoji: &str, text: &str) -> String {
        let icon = if self.use_emoji {
            format!("{}  ", emoji) // Two spaces after emoji
        } else {
            String::new()
        };

        if self.use_color {
            format!("\x1b[1m{}{}\x1b[0m", icon, text) // Bold
        } else {
            format!("{}{}", icon, text)
        }
    }

    /// Format an OK bullet (✓ or [OK])
    pub fn bullet_ok(&self, text: &str) -> String {
        let symbol = if self.use_emoji { "✓" } else { "[OK]" };

        if self.use_color {
            format!("\x1b[38;5;120m{}\x1b[0m {}", symbol, text) // Green
        } else {
            format!("{} {}", symbol, text)
        }
    }

    /// Format a warning bullet (⚠ or [WARN])
    pub fn bullet_warn(&self, text: &str) -> String {
        let symbol = if self.use_emoji { "⚠" } else { "[WARN]" };

        if self.use_color {
            format!("\x1b[38;5;228m{}\x1b[0m {}", symbol, text) // Yellow
        } else {
            format!("{} {}", symbol, text)
        }
    }

    /// Format a critical bullet (✗ or [CRIT])
    pub fn bullet_crit(&self, text: &str) -> String {
        let symbol = if self.use_emoji { "✗" } else { "[CRIT]" };

        if self.use_color {
            format!("\x1b[38;5;210m{}\x1b[0m {}", symbol, text) // Red
        } else {
            format!("{} {}", symbol, text)
        }
    }

    /// Format a key-value pair
    pub fn format_key_value(&self, key: &str, value: &str) -> String {
        if self.use_color {
            format!("\x1b[2m{:<20}\x1b[0m {}", key, value) // Dim key
        } else {
            format!("{:<20} {}", key, value)
        }
    }

    // ========================================================================
    // Diagnostic Blocks
    // ========================================================================

    /// Format a diagnostic block with title, body, severity, and hints
    pub fn diagnostic_block(&self, title: &str, body: &str, severity: Severity, hints: &[&str]) -> String {
        let mut output = String::new();

        // Title with severity indicator
        let (symbol, color) = match severity {
            Severity::Ok => (if self.use_emoji { "✓" } else { "[OK]" }, "\x1b[38;5;120m"),
            Severity::Info => (if self.use_emoji { "ℹ" } else { "[INFO]" }, "\x1b[38;5;117m"),
            Severity::Warning => (if self.use_emoji { "⚠" } else { "[WARN]" }, "\x1b[38;5;228m"),
            Severity::Critical => (if self.use_emoji { "✗" } else { "[CRIT]" }, "\x1b[38;5;210m"),
        };

        if self.use_color {
            output.push_str(&format!("{}{} \x1b[1m{}\x1b[0m\n", color, symbol, title));
        } else {
            output.push_str(&format!("{} {}\n", symbol, title));
        }

        // Body (indented)
        for line in body.lines() {
            output.push_str(&format!("  {}\n", line));
        }

        // Hints (indented with arrow)
        if !hints.is_empty() {
            for hint in hints {
                let arrow = if self.use_emoji { "→" } else { "->" };
                if self.use_color {
                    output.push_str(&format!("  \x1b[2m{} {}\x1b[0m\n", arrow, hint));
                } else {
                    output.push_str(&format!("  {} {}\n", arrow, hint));
                }
            }
        }

        output
    }

    // ========================================================================
    // Specialized Formatters
    // ========================================================================

    /// Format overall status line
    pub fn overall_status(&self, status: &str, details: &str) -> String {
        let (emoji, color) = match status.to_lowercase().as_str() {
            "healthy" | "ok" => (if self.use_emoji { "✅" } else { "[OK]" }, "\x1b[38;5;120m"),
            "degraded" => (if self.use_emoji { "⚠️" } else { "[WARN]" }, "\x1b[38;5;228m"),
            "critical" | "broken" => (if self.use_emoji { "🚨" } else { "[CRIT]" }, "\x1b[38;5;210m"),
            _ => (if self.use_emoji { "❓" } else { "[?]" }, "\x1b[2m"),
        };

        if self.use_color {
            format!("{}{} \x1b[1m{}\x1b[0m – {}", color, emoji, status.to_uppercase(), details)
        } else {
            format!("{} {} – {}", emoji, status.to_uppercase(), details)
        }
    }

    /// Format section title with emoji
    pub fn section(&self, emoji: &str, title: &str) -> String {
        let icon = if self.use_emoji {
            format!("{}  ", emoji) // Two spaces
        } else {
            format!("[{}] ", title.to_uppercase().chars().next().unwrap_or('S'))
        };

        if self.use_color {
            format!("\x1b[1;36m{}{}\x1b[0m", icon, title)
        } else {
            format!("{}{}", icon, title)
        }
    }

    /// Format a command suggestion
    pub fn command(&self, cmd: &str) -> String {
        if self.use_color {
            format!("\x1b[38;5;159m$ {}\x1b[0m", cmd) // Cyan
        } else {
            format!("$ {}", cmd)
        }
    }

    /// Strip ANSI codes for testing
    pub fn strip_ansi(text: &str) -> String {
        let re = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
        re.replace_all(text, "").to_string()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emoji_enabled_has_emoji() {
        let config = OutputConfig {
            emoji_mode: EmojiMode::Enabled,
            color_mode: ColorMode::None,
        };
        let fmt = OutputFormatter::new(config);

        let result = fmt.subheading("⚙️", "Core Health");
        let stripped = OutputFormatter::strip_ansi(&result);

        assert!(stripped.contains("⚙️"));
        assert!(stripped.contains("  ")); // Two spaces after emoji
    }

    #[test]
    fn test_emoji_disabled_has_no_emoji() {
        let config = OutputConfig {
            emoji_mode: EmojiMode::Disabled,
            color_mode: ColorMode::None,
        };
        let fmt = OutputFormatter::new(config);

        let result = fmt.subheading("⚙️", "Core Health");
        let stripped = OutputFormatter::strip_ansi(&result);

        assert!(!stripped.contains("⚙️"));
        assert!(stripped.contains("Core Health"));
    }

    #[test]
    fn test_bullet_ok_format() {
        let config = OutputConfig {
            emoji_mode: EmojiMode::Enabled,
            color_mode: ColorMode::None,
        };
        let fmt = OutputFormatter::new(config);

        let result = fmt.bullet_ok("Service running");
        let stripped = OutputFormatter::strip_ansi(&result);

        assert!(stripped.contains("✓"));
        assert!(stripped.contains("Service running"));
    }

    #[test]
    fn test_bullet_crit_format() {
        let config = OutputConfig {
            emoji_mode: EmojiMode::Enabled,
            color_mode: ColorMode::None,
        };
        let fmt = OutputFormatter::new(config);

        let result = fmt.bullet_crit("Service failed");
        let stripped = OutputFormatter::strip_ansi(&result);

        assert!(stripped.contains("✗"));
        assert!(stripped.contains("Service failed"));
    }

    #[test]
    fn test_diagnostic_block_with_hints() {
        let config = OutputConfig {
            emoji_mode: EmojiMode::Disabled,
            color_mode: ColorMode::None,
        };
        let fmt = OutputFormatter::new(config);

        let result = fmt.diagnostic_block(
            "Disk Full",
            "Root filesystem at 97% capacity",
            Severity::Critical,
            &["sudo du -xh / | sort -h | tail -20"]
        );

        let stripped = OutputFormatter::strip_ansi(&result);
        assert!(stripped.contains("[CRIT]"));
        assert!(stripped.contains("Disk Full"));
        assert!(stripped.contains("Root filesystem"));
        assert!(stripped.contains("sudo du"));
    }

    #[test]
    fn test_overall_status_degraded() {
        let config = OutputConfig {
            emoji_mode: EmojiMode::Disabled,
            color_mode: ColorMode::None,
        };
        let fmt = OutputFormatter::new(config);

        let result = fmt.overall_status("degraded", "2 critical issues, 1 warning");
        let stripped = OutputFormatter::strip_ansi(&result);

        assert!(stripped.contains("DEGRADED"));
        assert!(stripped.contains("2 critical issues"));
    }

    #[test]
    fn test_color_disabled_no_ansi() {
        let config = OutputConfig {
            emoji_mode: EmojiMode::Disabled,
            color_mode: ColorMode::None,
        };
        let fmt = OutputFormatter::new(config);

        let result = fmt.heading("Test Heading");
        assert!(!result.contains("\x1b["));
    }

    #[test]
    fn test_strip_ansi() {
        let colored = "\x1b[1;36mTest\x1b[0m Text";
        let stripped = OutputFormatter::strip_ansi(colored);
        assert_eq!(stripped, "Test Text");
    }
}
