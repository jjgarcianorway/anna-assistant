//! Unified messaging layer for Anna Assistant
//!
//! This module provides the `anna_say` function - the single source of truth
//! for all user-facing output. It handles:
//! - Color and emoji detection
//! - Terminal capability detection
//! - Consistent formatting and alignment
//! - Timestamp formatting
//! - Message categorization (info, ok, warn, error, narrative)

use chrono::Local;
use once_cell::sync::Lazy;
use std::io::{self, IsTerminal, Write};
use std::sync::Mutex;

/// Message types determine formatting and emoji selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    /// Informational message (blue, ⚙️)
    Info,
    /// Success message (green, ✅)
    Ok,
    /// Warning message (yellow, 🟡)
    Warn,
    /// Error message (red, ❌)
    Error,
    /// Narrative message - Anna speaking naturally (cyan, 🤖)
    Narrative,
}

/// Terminal capabilities detected at runtime
#[derive(Debug, Clone)]
pub struct TerminalCapabilities {
    pub is_tty: bool,
    pub supports_color: bool,
    pub supports_unicode: bool,
    pub width: usize,
}

/// Global terminal capabilities
pub static TERM_CAPS: Lazy<TerminalCapabilities> = Lazy::new(detect_terminal_capabilities);

/// Global configuration state
static CONFIG_OVERRIDE: Lazy<Mutex<Option<crate::config::AnnaConfig>>> =
    Lazy::new(|| Mutex::new(None));

/// Set global configuration override
pub fn set_config(config: crate::config::AnnaConfig) {
    *CONFIG_OVERRIDE.lock().unwrap() = Some(config);
}

/// Get current configuration (override or default)
fn get_config() -> crate::config::AnnaConfig {
    CONFIG_OVERRIDE
        .lock()
        .unwrap()
        .clone()
        .unwrap_or_else(crate::config::default_config)
}

/// Detect terminal capabilities at startup
fn detect_terminal_capabilities() -> TerminalCapabilities {
    let is_tty = io::stdout().is_terminal();

    // Check for NO_COLOR environment variable
    let no_color = std::env::var("NO_COLOR").is_ok();

    // Check color support
    let supports_color = if no_color {
        false
    } else if let Ok(term) = std::env::var("TERM") {
        is_tty && !term.contains("dumb") && term != "unknown"
    } else {
        false
    };

    // Check Unicode support
    let supports_unicode = std::env::var("LANG")
        .or_else(|_| std::env::var("LC_ALL"))
        .map(|s| s.to_uppercase().contains("UTF-8"))
        .unwrap_or(false);

    // Get terminal width
    let width = if is_tty {
        terminal_size::terminal_size()
            .map(|(terminal_size::Width(w), _)| w as usize)
            .unwrap_or(80)
    } else {
        80
    };

    TerminalCapabilities {
        is_tty,
        supports_color,
        supports_unicode,
        width,
    }
}

/// ANSI color codes (pastel palette for dark terminals)
mod colors {
    pub const CYAN: &str = "\x1b[38;5;87m"; // Headers, narrative
    pub const GREEN: &str = "\x1b[38;5;120m"; // Success
    pub const YELLOW: &str = "\x1b[38;5;228m"; // Warnings
    pub const RED: &str = "\x1b[38;5;210m"; // Errors
    pub const BLUE: &str = "\x1b[38;5;111m"; // Info
    pub const GRAY: &str = "\x1b[38;5;245m"; // Secondary
    pub const BOLD: &str = "\x1b[1m"; // Bold
    pub const RESET: &str = "\x1b[0m"; // Reset
}

/// Emoji for each message type (with ASCII fallbacks)
impl MessageType {
    fn emoji(&self, unicode_supported: bool) -> &'static str {
        if unicode_supported {
            match self {
                MessageType::Info => "⚙️ ",
                MessageType::Ok => "✅",
                MessageType::Warn => "🟡",
                MessageType::Error => "❌",
                MessageType::Narrative => "🤖",
            }
        } else {
            match self {
                MessageType::Info => "[i]",
                MessageType::Ok => "[✓]",
                MessageType::Warn => "[!]",
                MessageType::Error => "[X]",
                MessageType::Narrative => "[Anna]",
            }
        }
    }

    fn color(&self) -> &'static str {
        match self {
            MessageType::Info => colors::BLUE,
            MessageType::Ok => colors::GREEN,
            MessageType::Warn => colors::YELLOW,
            MessageType::Error => colors::RED,
            MessageType::Narrative => colors::CYAN,
        }
    }
}

/// Structured message for anna_say
#[derive(Debug, Clone)]
pub struct AnnaMessage {
    pub msg_type: MessageType,
    pub text: String,
    pub show_timestamp: bool,
    pub show_prefix: bool,
}

impl AnnaMessage {
    /// Create a new message
    pub fn new(msg_type: MessageType, text: impl Into<String>) -> Self {
        Self {
            msg_type,
            text: text.into(),
            show_timestamp: true,
            show_prefix: true,
        }
    }

    /// Disable timestamp for this message
    pub fn without_timestamp(mut self) -> Self {
        self.show_timestamp = false;
        self
    }

    /// Disable "Anna:" prefix for this message
    pub fn without_prefix(mut self) -> Self {
        self.show_prefix = false;
        self
    }
}

/// Main output function - Anna's voice
///
/// This is the unified interface for all output in Anna Assistant.
/// Use this instead of println!, eprintln!, or echo in scripts.
///
/// # Examples
///
/// ```rust
/// use anna_common::{anna_say, AnnaMessage, MessageType};
///
/// // Simple info message
/// anna_say(AnnaMessage::new(MessageType::Info, "Checking dependencies..."));
///
/// // Success without timestamp
/// anna_say(
///     AnnaMessage::new(MessageType::Ok, "Installation complete!")
///         .without_timestamp()
/// );
///
/// // Narrative (Anna speaking naturally)
/// anna_say(AnnaMessage::new(
///     MessageType::Narrative,
///     "Let me see if everything you need is already installed."
/// ));
/// ```
pub fn anna_say(message: AnnaMessage) {
    let config = get_config();
    let caps = &*TERM_CAPS;

    // Respect user configuration
    let use_color = config.colors && caps.supports_color;
    let use_emoji = config.emojis && caps.supports_unicode;

    // Build the message parts
    let mut parts = Vec::new();

    // Timestamp
    if message.show_timestamp && config.verbose {
        let timestamp = crate::locale::format_timestamp(&Local::now());
        if use_color {
            parts.push(format!("{}[{}]{}", colors::GRAY, timestamp, colors::RESET));
        } else {
            parts.push(format!("[{}]", timestamp));
        }
    }

    // Emoji/icon
    let emoji = message.msg_type.emoji(use_emoji);
    if use_color {
        parts.push(format!(
            "{}{}{}",
            message.msg_type.color(),
            emoji,
            colors::RESET
        ));
    } else {
        parts.push(emoji.to_string());
    }

    // "Anna:" prefix for narrative messages
    if message.show_prefix && message.msg_type == MessageType::Narrative {
        if use_color {
            parts.push(format!("{}Anna:{}", colors::BOLD, colors::RESET));
        } else {
            parts.push("Anna:".to_string());
        }
    }

    // The actual message text
    if use_color {
        parts.push(format!(
            "{}{}{}",
            message.msg_type.color(),
            message.text,
            colors::RESET
        ));
    } else {
        parts.push(message.text.clone());
    }

    // Print to appropriate stream
    let output = parts.join(" ");
    match message.msg_type {
        MessageType::Error => {
            let mut stderr = io::stderr().lock();
            let _ = writeln!(stderr, "{}", output);
        }
        _ => {
            let mut stdout = io::stdout().lock();
            let _ = writeln!(stdout, "{}", output);
        }
    }
}

/// Convenience functions for common message types
pub fn anna_info(text: impl Into<String>) {
    anna_say(AnnaMessage::new(MessageType::Info, text));
}

pub fn anna_ok(text: impl Into<String>) {
    anna_say(AnnaMessage::new(MessageType::Ok, text));
}

pub fn anna_warn(text: impl Into<String>) {
    anna_say(AnnaMessage::new(MessageType::Warn, text));
}

pub fn anna_error(text: impl Into<String>) {
    anna_say(AnnaMessage::new(MessageType::Error, text));
}

pub fn anna_narrative(text: impl Into<String>) {
    anna_say(AnnaMessage::new(MessageType::Narrative, text));
}

/// Print a decorative box (for installer ceremonies)
pub fn anna_box(lines: &[&str], box_type: MessageType) {
    let caps = &*TERM_CAPS;
    let config = get_config();
    let use_unicode = config.emojis && caps.supports_unicode;
    let use_color = config.colors && caps.supports_color;

    // Calculate box width (max line length + padding)
    let max_len = lines.iter().map(|l| l.len()).max().unwrap_or(0);
    let box_width = max_len + 4; // 2 chars padding on each side

    // Box drawing characters
    let (tl, tr, bl, br, h, v) = if use_unicode {
        ('╭', '╮', '╰', '╯', '─', '│')
    } else {
        ('+', '+', '+', '+', '-', '|')
    };

    let color = if use_color { box_type.color() } else { "" };
    let reset = if use_color { colors::RESET } else { "" };

    // Top border
    println!(
        "{}{}{}{}{}",
        color,
        tl,
        h.to_string().repeat(box_width - 2),
        tr,
        reset
    );

    // Content lines (centered)
    for line in lines {
        let padding = (box_width - 2 - line.len()) / 2;
        let padding_right = box_width - 2 - line.len() - padding;
        println!(
            "{}{}{}{}{}{}{}",
            color,
            v,
            reset,
            " ".repeat(padding),
            line,
            " ".repeat(padding_right),
            format!("{}{}{}", color, v, reset)
        );
    }

    // Bottom border
    println!(
        "{}{}{}{}{}",
        color,
        bl,
        h.to_string().repeat(box_width - 2),
        br,
        reset
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_detection() {
        let caps = detect_terminal_capabilities();
        // Basic sanity check - should not panic
        assert!(caps.width > 0);
    }

    #[test]
    fn test_message_creation() {
        let msg = AnnaMessage::new(MessageType::Info, "test");
        assert_eq!(msg.msg_type, MessageType::Info);
        assert_eq!(msg.text, "test");
        assert!(msg.show_timestamp);
        assert!(msg.show_prefix);

        let msg = msg.without_timestamp().without_prefix();
        assert!(!msg.show_timestamp);
        assert!(!msg.show_prefix);
    }
}
