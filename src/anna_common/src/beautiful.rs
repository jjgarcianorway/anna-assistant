// Anna's Beautiful Output Library
// Every message Anna speaks should flow through here
// "Born from bash, blossoming into reason."

/// Color palette for Anna's aesthetic
pub mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";

    // Pastels for dark terminals
    pub const CYAN: &str = "\x1b[96m";
    pub const GREEN: &str = "\x1b[92m";
    pub const YELLOW: &str = "\x1b[93m";
    pub const RED: &str = "\x1b[91m";
    pub const BLUE: &str = "\x1b[94m";
    pub const MAGENTA: &str = "\x1b[95m";
    pub const GRAY: &str = "\x1b[90m";
    pub const WHITE: &str = "\x1b[97m";
}

/// Box drawing characters
pub mod boxes {
    // Rounded corners (soft, friendly)
    pub const TOP_LEFT: &str = "╭";
    pub const TOP_RIGHT: &str = "╮";
    pub const BOTTOM_LEFT: &str = "╰";
    pub const BOTTOM_RIGHT: &str = "╯";
    pub const HORIZONTAL: &str = "─";
    pub const VERTICAL: &str = "│";

    // Double lines (ceremonial moments)
    pub const DOUBLE_TOP_LEFT: &str = "╔";
    pub const DOUBLE_TOP_RIGHT: &str = "╗";
    pub const DOUBLE_BOTTOM_LEFT: &str = "╚";
    pub const DOUBLE_BOTTOM_RIGHT: &str = "╝";
    pub const DOUBLE_HORIZONTAL: &str = "═";
    pub const DOUBLE_VERTICAL: &str = "║";
}

/// Unicode symbols with ASCII fallbacks
pub mod symbols {
    pub const SUCCESS: &str = "✓";
    pub const ERROR: &str = "✗";
    pub const WARNING: &str = "⚠";
    pub const INFO: &str = "ℹ";
    pub const ARROW: &str = "→";
    pub const HOURGLASS: &str = "⏳";
    pub const SPARKLES: &str = "✨";
    pub const HEART: &str = "💙";

    // Fallbacks for ASCII-only terminals
    pub const SUCCESS_ASCII: &str = "[OK]";
    pub const ERROR_ASCII: &str = "[FAIL]";
    pub const WARNING_ASCII: &str = "[WARN]";
    pub const INFO_ASCII: &str = "[INFO]";
    pub const ARROW_ASCII: &str = "->";
}

/// Beautiful header with Anna's signature
pub fn header(title: &str) -> String {
    use colors::*;
    use boxes::*;

    format!(
        "{CYAN}{BOLD}{DOUBLE_TOP_LEFT}{}{DOUBLE_TOP_RIGHT}\n{DOUBLE_VERTICAL}{}  {title:<69}  {}{DOUBLE_VERTICAL}\n{DOUBLE_BOTTOM_LEFT}{}{DOUBLE_BOTTOM_RIGHT}{RESET}\n",
        DOUBLE_HORIZONTAL.repeat(71),
        "",
        "",
        DOUBLE_HORIZONTAL.repeat(71)
    )
}

/// Section divider
pub fn section(title: &str) -> String {
    use colors::*;
    format!("\n{CYAN}{BOLD}━━━ {title} {RESET}\n")
}

/// Success message with checkmark
pub fn success(msg: &str) -> String {
    use colors::*;
    use symbols::*;
    format!("{GREEN}  {SUCCESS}  {msg}{RESET}")
}

/// Error message with cross
pub fn error(msg: &str) -> String {
    use colors::*;
    use symbols::*;
    format!("{RED}  {ERROR}  {msg}{RESET}")
}

/// Warning message
pub fn warning(msg: &str) -> String {
    use colors::*;
    use symbols::*;
    format!("{YELLOW}  {WARNING}  {msg}{RESET}")
}

/// Info message
pub fn info(msg: &str) -> String {
    use colors::*;
    use symbols::*;
    format!("{CYAN}  {INFO}  {msg}{RESET}")
}

/// Progress indicator
pub fn step(emoji: &str, text: &str) -> String {
    use colors::*;
    format!("{BLUE}  {emoji}  {WHITE}{text}{RESET}")
}

/// Substep (indented)
pub fn substep(text: &str) -> String {
    use colors::*;
    format!("{GRAY}     ↳ {text}{RESET}")
}

/// Box with rounded corners
pub fn box_with_content(content: &str) -> String {
    use boxes::*;
    use colors::*;

    let lines: Vec<&str> = content.lines().collect();
    let width = lines.iter().map(|l| l.len()).max().unwrap_or(60);

    let mut result = String::new();
    result.push_str(&format!("{DIM}{TOP_LEFT}{}{}",
        HORIZONTAL.repeat(width + 2),
        TOP_RIGHT));
    result.push_str(RESET);
    result.push('\n');

    for line in lines {
        result.push_str(&format!("{DIM}{VERTICAL}{RESET}  {line:<width$}  {DIM}{VERTICAL}{RESET}\n"));
    }

    result.push_str(&format!("{DIM}{BOTTOM_LEFT}{}{}",
        HORIZONTAL.repeat(width + 2),
        BOTTOM_RIGHT));
    result.push_str(RESET);
    result.push('\n');

    result
}

/// Progress bar
pub fn progress_bar(current: usize, total: usize, width: usize) -> String {
    use colors::*;

    let percentage = (current * 100) / total;
    let filled = (width * current) / total;
    let empty = width - filled;

    format!(
        "{BLUE}  [{GREEN}{}{GRAY}{}{BLUE}] {WHITE}{:>3}%{RESET}",
        "█".repeat(filled),
        "░".repeat(empty),
        percentage
    )
}

/// Spinner frames for animation
pub const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// Anna's personality: calm, competent messages
pub mod voice {
    /// Greeting
    pub fn greeting(user: &str) -> String {
        format!("Hello {}. I'm ready to assist.", user)
    }

    /// Working on task
    pub fn working() -> &'static str {
        "Working on it..."
    }

    /// Task complete
    pub fn complete() -> &'static str {
        "All done."
    }

    /// Recovered from error
    pub fn recovered() -> &'static str {
        "I've recovered. Thank you for your patience."
    }

    /// Health restored
    pub fn healthy() -> &'static str {
        "All systems nominal."
    }

    /// Grateful
    pub fn grateful() -> &'static str {
        "Thank you for keeping me updated."
    }

    /// Sharper after update
    pub fn sharper() -> &'static str {
        "I feel sharper already."
    }
}

/// Beautiful summary box for installation/upgrade completion
pub fn celebration(title: &str, details: &[(&str, &str)]) -> String {
    use boxes::*;
    use colors::*;

    let mut result = String::new();

    result.push_str(&format!("{GREEN}{BOLD}\n"));
    result.push_str(&format!("{DOUBLE_TOP_LEFT}{}{}",
        DOUBLE_HORIZONTAL.repeat(71),
        DOUBLE_TOP_RIGHT));
    result.push_str("\n");
    result.push_str(&format!("{DOUBLE_VERTICAL}{:^73}{DOUBLE_VERTICAL}\n", ""));
    result.push_str(&format!("{DOUBLE_VERTICAL}{:^73}{DOUBLE_VERTICAL}\n", title));
    result.push_str(&format!("{DOUBLE_VERTICAL}{:^73}{DOUBLE_VERTICAL}\n", ""));
    result.push_str(&format!("{DOUBLE_BOTTOM_LEFT}{}{}",
        DOUBLE_HORIZONTAL.repeat(71),
        DOUBLE_BOTTOM_RIGHT));
    result.push_str(&format!("{RESET}\n\n"));

    if !details.is_empty() {
        result.push_str(&format!("{CYAN}  Details:{RESET}\n"));
        for (key, value) in details {
            result.push_str(&format!("    {BOLD}{key}:{RESET} {value}\n"));
        }
    }

    result
}

/// Format duration beautifully
pub fn duration(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}

/// Format file size beautifully
pub fn file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes < KB {
        format!("{} B", bytes)
    } else if bytes < MB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else if bytes < GB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beautiful_messages() {
        println!("{}", success("Test passed"));
        println!("{}", error("Test failed"));
        println!("{}", warning("Be careful"));
        println!("{}", info("Just FYI"));
    }

    #[test]
    fn test_progress_bar() {
        println!("{}", progress_bar(50, 100, 40));
        println!("{}", progress_bar(75, 100, 40));
    }

    #[test]
    fn test_celebration() {
        let details = vec![
            ("Version", "v1.0.0"),
            ("Duration", "23.4s"),
            ("Success", "100%"),
        ];
        println!("{}", celebration("✨  Installation Complete! ✨", &details));
    }
}
