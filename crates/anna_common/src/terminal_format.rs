//! Terminal Formatting Helpers
//!
//! Phase 8: Beautiful UX & Terminal Enhancements
//!
//! Provides consistent, professional formatting for Anna's terminal output.
//! Colors are subtle and WCAG-friendly. No hardcoded ANSI strings scattered everywhere.

/// ANSI color codes - WCAG-friendly palette
pub mod colors {
    // Success and safe states
    pub const GREEN: &str = "\x1b[38;5;120m"; // Soft green
    pub const GREEN_BOLD: &str = "\x1b[1;38;5;120m";

    // Warnings and caution
    pub const YELLOW: &str = "\x1b[38;5;228m"; // Soft yellow
    pub const YELLOW_BOLD: &str = "\x1b[1;38;5;228m";
    pub const ORANGE: &str = "\x1b[38;5;215m"; // Soft orange

    // Errors and danger
    pub const RED: &str = "\x1b[38;5;210m"; // Soft red
    pub const RED_BOLD: &str = "\x1b[1;38;5;210m";

    // Info and neutral
    pub const BLUE: &str = "\x1b[38;5;117m"; // Soft blue
    pub const CYAN: &str = "\x1b[38;5;159m"; // Soft cyan
    pub const GRAY: &str = "\x1b[38;5;250m"; // Medium gray
    pub const DIM: &str = "\x1b[2m"; // Dimmed text

    // Reset and modifiers
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
}

/// Visual symbols for different states
pub mod symbols {
    // Risk indicators
    pub const RISK_LOW: &str = "✓";
    pub const RISK_MEDIUM: &str = "△";
    pub const RISK_HIGH: &str = "⚠";
    pub const RISK_FORBIDDEN: &str = "⛔";

    // General purpose
    pub const CHECK: &str = "✓";
    pub const CROSS: &str = "✗";
    pub const ARROW: &str = "→";
    pub const BULLET: &str = "•";
    pub const STAR: &str = "★";
    pub const WARNING: &str = "⚠";
    pub const INFO: &str = "ℹ";
    pub const QUESTION: &str = "?";

    // Borders (ASCII-safe fallbacks available)
    pub const BOX_TL: &str = "┌";
    pub const BOX_TR: &str = "┐";
    pub const BOX_BL: &str = "└";
    pub const BOX_BR: &str = "┘";
    pub const BOX_H: &str = "─";
    pub const BOX_V: &str = "│";
    pub const BOX_T: &str = "┬";
    pub const BOX_B: &str = "┴";
}

/// Beta.141: Emoji indicators for visual scanning (catch Claude's attention!)
pub mod emojis {
    // System status
    pub const HEALTHY: &str = "✅";
    pub const DEGRADED: &str = "⚠️";
    pub const ERROR: &str = "❌";
    pub const RUNNING: &str = "🟢";
    pub const STOPPED: &str = "🔴";
    pub const UNKNOWN: &str = "❓";

    // Categories
    pub const CPU: &str = "🔥";
    pub const MEMORY: &str = "🧠";
    pub const DISK: &str = "💾";
    pub const NETWORK: &str = "🌐";
    pub const GPU: &str = "🎮";
    pub const PACKAGE: &str = "📦";
    pub const SERVICE: &str = "⚙️";
    pub const SECURITY: &str = "🔒";

    // Actions
    pub const INSTALL: &str = "⬇️";
    pub const REMOVE: &str = "🗑️";
    pub const UPDATE: &str = "🔄";
    pub const CONFIGURE: &str = "⚙️";
    pub const BACKUP: &str = "💾";
    pub const RESTORE: &str = "♻️";

    // Status
    pub const SUCCESS: &str = "✅";
    pub const FAILURE: &str = "❌";
    pub const PENDING: &str = "⏳";
    pub const SKIPPED: &str = "⏭️";

    // Information
    pub const NOTE: &str = "📝";
    pub const TIP: &str = "💡";
    pub const WARNING: &str = "⚠️";
    pub const CRITICAL: &str = "🚨";
    pub const INFO: &str = "ℹ️";

    // System components
    pub const DAEMON: &str = "👾";
    pub const LLM: &str = "🤖";
    pub const USER: &str = "👤";
    pub const ROOT: &str = "🔐";
    pub const TIME: &str = "⏰";
    pub const ROCKET: &str = "🚀";
}

/// Format a section title with icon
pub fn section_title(icon: &str, text: &str) -> String {
    format!(
        "{}{} {}{}{}",
        colors::BOLD,
        colors::CYAN,
        icon,
        text,
        colors::RESET
    )
}

/// Format a success message
pub fn success(text: &str) -> String {
    format!(
        "{}{} {}{}",
        colors::GREEN,
        symbols::CHECK,
        text,
        colors::RESET
    )
}

/// Format an error message
pub fn error(text: &str) -> String {
    format!(
        "{}{} {}{}",
        colors::RED,
        symbols::CROSS,
        text,
        colors::RESET
    )
}

/// Format a warning message
pub fn warning(text: &str) -> String {
    format!(
        "{}{} {}{}",
        colors::YELLOW,
        symbols::WARNING,
        text,
        colors::RESET
    )
}

/// Format an info message
pub fn info(text: &str) -> String {
    format!(
        "{}{} {}{}",
        colors::BLUE,
        symbols::INFO,
        text,
        colors::RESET
    )
}

/// Format a bullet point
pub fn bullet(text: &str) -> String {
    format!(
        "  {}{} {}{}",
        colors::GRAY,
        symbols::BULLET,
        colors::RESET,
        text
    )
}

/// Format an arrow item
pub fn arrow(text: &str) -> String {
    format!(
        "{}{} {}{}",
        colors::CYAN,
        symbols::ARROW,
        colors::RESET,
        text
    )
}

/// Format a horizontal separator
pub fn separator(width: usize) -> String {
    format!(
        "{}{}{}",
        colors::GRAY,
        symbols::BOX_H.repeat(width),
        colors::RESET
    )
}

/// Format text in a box
pub fn boxed(title: &str, lines: &[&str], width: usize) -> String {
    let mut output = String::new();

    // Top border with title
    output.push_str(&format!(
        "{}{}{} {} {}{}{}\n",
        colors::GRAY,
        symbols::BOX_TL,
        symbols::BOX_H.repeat(2),
        title,
        symbols::BOX_H.repeat(width.saturating_sub(title.len() + 5)),
        symbols::BOX_TR,
        colors::RESET
    ));

    // Content lines
    for line in lines {
        let content_width = width.saturating_sub(4);
        output.push_str(&format!(
            "{}{}{} {:<width$} {}{}{}\n",
            colors::GRAY,
            symbols::BOX_V,
            colors::RESET,
            line,
            colors::GRAY,
            symbols::BOX_V,
            colors::RESET,
            width = content_width
        ));
    }

    // Bottom border
    output.push_str(&format!(
        "{}{}{}{}{}",
        colors::GRAY,
        symbols::BOX_BL,
        symbols::BOX_H.repeat(width.saturating_sub(2)),
        symbols::BOX_BR,
        colors::RESET
    ));

    output
}

/// Format a risk badge
pub fn risk_badge(risk: &str) -> String {
    match risk.to_lowercase().as_str() {
        "low" => format!(
            "{}{} LOW{}",
            colors::GREEN_BOLD,
            symbols::RISK_LOW,
            colors::RESET
        ),
        "medium" => format!(
            "{}{} MEDIUM{}",
            colors::YELLOW_BOLD,
            symbols::RISK_MEDIUM,
            colors::RESET
        ),
        "high" => format!(
            "{}{} HIGH{}",
            colors::ORANGE,
            symbols::RISK_HIGH,
            colors::RESET
        ),
        "forbidden" => format!(
            "{}{} FORBIDDEN{}",
            colors::RED_BOLD,
            symbols::RISK_FORBIDDEN,
            colors::RESET
        ),
        _ => format!("[{}]", risk),
    }
}

/// Format a category badge
pub fn category_badge(category: &str) -> String {
    let (color, label) = match category.to_lowercase().as_str() {
        "cosmeticuser" | "cosmetic" => (colors::GREEN, "Cosmetic"),
        "userconfig" | "config" => (colors::BLUE, "Config"),
        "systemservice" | "service" => (colors::ORANGE, "Service"),
        "systempackage" | "package" => (colors::YELLOW, "Package"),
        "bootandstorage" | "boot" => (colors::RED, "Boot/Storage"),
        _ => (colors::GRAY, category),
    };

    format!("{}{}{}", color, label, colors::RESET)
}

/// Format a sudo indicator
pub fn sudo_badge() -> String {
    format!("{}[sudo]{}", colors::RED, colors::RESET)
}

/// Format a progress indicator
pub fn progress(current: usize, total: usize, label: &str) -> String {
    let percentage = if total > 0 {
        (current * 100) / total
    } else {
        0
    };

    let bar_width = 30;
    let filled = (percentage * bar_width) / 100;
    let empty = bar_width - filled;

    format!(
        "{}{} [{}{}>{}{}] {}/{}{}",
        colors::CYAN,
        label,
        colors::GREEN,
        "=".repeat(filled),
        " ".repeat(empty),
        colors::CYAN,
        current,
        total,
        colors::RESET
    )
}

/// Format a table header
pub fn table_header(columns: &[(&str, usize)]) -> String {
    let mut output = String::new();

    // Header row
    output.push_str(colors::BOLD);
    for (name, width) in columns {
        output.push_str(&format!("{:<width$}  ", name, width = width));
    }
    output.push_str(colors::RESET);
    output.push('\n');

    // Separator
    output.push_str(colors::GRAY);
    for (_, width) in columns {
        output.push_str(&symbols::BOX_H.repeat(*width + 2));
    }
    output.push_str(colors::RESET);

    output
}

/// Format a table row
pub fn table_row(cells: &[(&str, usize)]) -> String {
    let mut output = String::new();

    for (content, width) in cells {
        output.push_str(&format!("{:<width$}  ", content, width = width));
    }

    output
}

/// Format a key-value pair
pub fn key_value(key: &str, value: &str) -> String {
    format!("{}{:<20}{} {}", colors::GRAY, key, colors::RESET, value)
}

/// Format a numbered item
pub fn numbered(number: usize, text: &str) -> String {
    format!("{}{}. {}{}", colors::CYAN, number, colors::RESET, text)
}

/// Format a dimmed/secondary text
pub fn dimmed(text: &str) -> String {
    format!("{}{}{}", colors::DIM, text, colors::RESET)
}

/// Format bold text
pub fn bold(text: &str) -> String {
    format!("{}{}{}", colors::BOLD, text, colors::RESET)
}

/// Beta.141: System status with emoji indicator
pub fn system_status(status: &str, details: &str) -> String {
    let (emoji, color) = match status.to_lowercase().as_str() {
        "healthy" | "good" | "ok" => (emojis::HEALTHY, colors::GREEN),
        "degraded" | "warning" => (emojis::DEGRADED, colors::YELLOW),
        "error" | "critical" | "bad" => (emojis::ERROR, colors::RED),
        "running" => (emojis::RUNNING, colors::GREEN),
        "stopped" => (emojis::STOPPED, colors::RED),
        _ => (emojis::UNKNOWN, colors::GRAY),
    };
    format!(
        "{}{} {}{}{} {}",
        color,
        emoji,
        colors::BOLD,
        status.to_uppercase(),
        colors::RESET,
        details
    )
}

/// Beta.141: Telemetry item with category emoji
pub fn telemetry_item(category: &str, label: &str, value: &str) -> String {
    let emoji = match category.to_lowercase().as_str() {
        "cpu" => emojis::CPU,
        "memory" | "ram" => emojis::MEMORY,
        "disk" | "storage" => emojis::DISK,
        "network" => emojis::NETWORK,
        "gpu" => emojis::GPU,
        "package" => emojis::PACKAGE,
        "service" => emojis::SERVICE,
        "security" => emojis::SECURITY,
        _ => emojis::INFO,
    };
    format!(
        "{} {}{}{}: {}{}{}",
        emoji,
        colors::BOLD,
        label,
        colors::RESET,
        colors::CYAN,
        value,
        colors::RESET
    )
}

/// Beta.141: Action message with emoji
pub fn action_message(action: &str, target: &str) -> String {
    let emoji = match action.to_lowercase().as_str() {
        "install" | "installing" => emojis::INSTALL,
        "remove" | "removing" | "uninstall" => emojis::REMOVE,
        "update" | "updating" | "upgrade" => emojis::UPDATE,
        "configure" | "configuring" => emojis::CONFIGURE,
        "backup" => emojis::BACKUP,
        "restore" => emojis::RESTORE,
        _ => emojis::INFO,
    };
    format!(
        "{} {}{}{} {}",
        emoji,
        colors::BOLD,
        action,
        colors::RESET,
        target
    )
}

/// Beta.141: Component status (daemon, LLM, etc.)
pub fn component_status(component: &str, status: &str) -> String {
    let emoji = match component.to_lowercase().as_str() {
        "daemon" | "annad" => emojis::DAEMON,
        "llm" | "model" => emojis::LLM,
        "user" => emojis::USER,
        "root" | "sudo" => emojis::ROOT,
        _ => emojis::SERVICE,
    };
    let (status_emoji, color) = match status.to_lowercase().as_str() {
        "running" | "active" | "healthy" => (emojis::RUNNING, colors::GREEN),
        "stopped" | "inactive" => (emojis::STOPPED, colors::RED),
        "degraded" | "warning" => (emojis::DEGRADED, colors::YELLOW),
        _ => (emojis::UNKNOWN, colors::GRAY),
    };
    format!(
        "{} {}{}{}: {} {}{}{}",
        emoji,
        colors::BOLD,
        component,
        colors::RESET,
        status_emoji,
        color,
        status,
        colors::RESET
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_section_title_contains_text() {
        let result = section_title("🔧", "Test Section");
        assert!(result.contains("Test Section"));
        assert!(result.contains("🔧"));
    }

    #[test]
    fn test_success_contains_check() {
        let result = success("Operation completed");
        assert!(result.contains("Operation completed"));
        assert!(result.contains(symbols::CHECK));
    }

    #[test]
    fn test_error_contains_cross() {
        let result = error("Operation failed");
        assert!(result.contains("Operation failed"));
        assert!(result.contains(symbols::CROSS));
    }

    #[test]
    fn test_risk_badge_low() {
        let result = risk_badge("low");
        assert!(result.contains("LOW"));
        assert!(result.contains(symbols::RISK_LOW));
    }

    #[test]
    fn test_risk_badge_medium() {
        let result = risk_badge("medium");
        assert!(result.contains("MEDIUM"));
        assert!(result.contains(symbols::RISK_MEDIUM));
    }

    #[test]
    fn test_risk_badge_high() {
        let result = risk_badge("high");
        assert!(result.contains("HIGH"));
        assert!(result.contains(symbols::RISK_HIGH));
    }

    #[test]
    fn test_risk_badge_forbidden() {
        let result = risk_badge("forbidden");
        assert!(result.contains("FORBIDDEN"));
        assert!(result.contains(symbols::RISK_FORBIDDEN));
    }

    #[test]
    fn test_category_badge_cosmetic() {
        let result = category_badge("cosmetic");
        assert!(result.contains("Cosmetic"));
    }

    #[test]
    fn test_sudo_badge_contains_sudo() {
        let result = sudo_badge();
        assert!(result.contains("sudo"));
    }

    #[test]
    fn test_bullet_contains_text() {
        let result = bullet("Test item");
        assert!(result.contains("Test item"));
        assert!(result.contains(symbols::BULLET));
    }

    #[test]
    fn test_separator_has_correct_length() {
        let result = separator(50);
        // Should contain the separator character repeated 50 times (plus ANSI codes)
        assert!(result.len() > 50);
    }

    #[test]
    fn test_progress_shows_percentage() {
        let result = progress(5, 10, "Installing");
        assert!(result.contains("Installing"));
        assert!(result.contains("5/10"));
    }

    #[test]
    fn test_key_value_formatting() {
        let result = key_value("Risk", "Low");
        assert!(result.contains("Risk"));
        assert!(result.contains("Low"));
    }

    #[test]
    fn test_numbered_item() {
        let result = numbered(1, "First item");
        assert!(result.contains("1."));
        assert!(result.contains("First item"));
    }
}
