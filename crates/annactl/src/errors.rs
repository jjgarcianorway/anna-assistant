//! User-friendly error presentation (v0.0.304).
//!
//! Provides consistent, helpful error messages with recovery suggestions.

use anna_shared::ui::colors;

/// Error categories for user-friendly presentation
pub enum ErrorKind {
    /// Daemon not running or socket missing
    DaemonNotRunning,
    /// Daemon crashed or connection refused
    DaemonCrashed,
    /// Request timed out
    #[allow(dead_code)]
    Timeout { seconds: u64 },
    /// LLM service unavailable
    LlmUnavailable,
    /// Network/connectivity issue
    NetworkError,
    /// Invalid user input
    InvalidInput { detail: String },
    /// Internal error (unexpected)
    Internal { detail: String },
}

impl ErrorKind {
    /// Detect error kind from error message
    pub fn from_error(e: &anyhow::Error) -> Self {
        let msg = e.to_string().to_lowercase();

        if msg.contains("socket") && msg.contains("not exist") {
            Self::DaemonNotRunning
        } else if msg.contains("cannot connect") || msg.contains("connection refused") {
            Self::DaemonCrashed
        } else if msg.contains("timed out") {
            // Extract timeout value if present
            let secs = extract_timeout_secs(&msg).unwrap_or(45);
            Self::Timeout { seconds: secs }
        } else if msg.contains("llm") || msg.contains("ollama") || msg.contains("model") {
            Self::LlmUnavailable
        } else if msg.contains("network") || msg.contains("dns") || msg.contains("resolve") {
            Self::NetworkError
        } else if msg.contains("invalid") || msg.contains("parse") {
            Self::InvalidInput {
                detail: e.to_string(),
            }
        } else {
            Self::Internal {
                detail: e.to_string(),
            }
        }
    }

    /// Get user-friendly title for this error
    pub fn title(&self) -> &'static str {
        match self {
            Self::DaemonNotRunning => "Anna daemon not running",
            Self::DaemonCrashed => "Cannot reach Anna daemon",
            Self::Timeout { .. } => "Request timed out",
            Self::LlmUnavailable => "AI models unavailable",
            Self::NetworkError => "Network issue",
            Self::InvalidInput { .. } => "Invalid input",
            Self::Internal { .. } => "Unexpected error",
        }
    }

    /// Get recovery suggestions
    pub fn suggestions(&self) -> Vec<&'static str> {
        match self {
            Self::DaemonNotRunning => vec![
                "Start the daemon: sudo systemctl start annad",
                "Check daemon status: sudo systemctl status annad",
                "Reinstall if needed: curl -sSL https://raw.githubusercontent.com/jjgarcianorway/anna-assistant/main/scripts/install.sh | bash",
            ],
            Self::DaemonCrashed => vec![
                "Restart the daemon: sudo systemctl restart annad",
                "Check logs: sudo journalctl -u annad -n 50",
                "If persistent, reinstall Anna",
            ],
            Self::Timeout { .. } => vec![
                "Try a simpler, more specific question",
                "Check if Ollama is running: systemctl status ollama",
                "The AI model may be loading - wait a moment and retry",
            ],
            Self::LlmUnavailable => vec![
                "Check Ollama status: systemctl status ollama",
                "Restart Ollama: sudo systemctl restart ollama",
                "Anna will auto-recover when Ollama is available",
            ],
            Self::NetworkError => vec![
                "Check your internet connection",
                "Verify DNS is working: ping 8.8.8.8",
                "Anna works offline for most queries once models are downloaded",
            ],
            Self::InvalidInput { .. } => vec![
                "Try rephrasing your question",
                "Use natural language like: \"what's my disk usage?\"",
            ],
            Self::Internal { .. } => vec![
                "Try running the command again",
                "Check daemon logs: sudo journalctl -u annad -n 20",
                "Report issues at: https://github.com/jjgarcianorway/anna-assistant/issues",
            ],
        }
    }
}

/// Print a user-friendly error with recovery suggestions
pub fn print_error(e: &anyhow::Error) {
    let kind = ErrorKind::from_error(e);

    println!();
    println!(
        "{}[error]{} {}",
        colors::ERR,
        colors::RESET,
        kind.title()
    );

    // Show technical detail for internal errors
    if let ErrorKind::Internal { detail } | ErrorKind::InvalidInput { detail } = &kind {
        println!("{}  {}{}", colors::DIM, detail, colors::RESET);
    }

    // Show recovery suggestions
    let suggestions = kind.suggestions();
    if !suggestions.is_empty() {
        println!();
        println!("{}To fix this:{}", colors::DIM, colors::RESET);
        for suggestion in suggestions {
            println!("  {} {}", bullet(), suggestion);
        }
    }
    println!();
}

/// Print a simple warning (not a full error)
pub fn print_warning(msg: &str) {
    println!(
        "{}[warn]{} {}",
        colors::WARN,
        colors::RESET,
        msg
    );
}

/// Print an info message
#[allow(dead_code)]
pub fn print_info(msg: &str) {
    println!(
        "{}[info]{} {}",
        colors::CYAN,
        colors::RESET,
        msg
    );
}

fn bullet() -> &'static str {
    "›"
}

fn extract_timeout_secs(msg: &str) -> Option<u64> {
    // Look for pattern like "timed out after 45s"
    if let Some(pos) = msg.find("after ") {
        let rest = &msg[pos + 6..];
        let num_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        num_str.parse().ok()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;

    #[test]
    fn test_detect_daemon_not_running() {
        let err = anyhow!("The socket at /run/anna.sock does not exist");
        let kind = ErrorKind::from_error(&err);
        assert!(matches!(kind, ErrorKind::DaemonNotRunning));
    }

    #[test]
    fn test_detect_timeout() {
        let err = anyhow!("Request timed out after 45s");
        let kind = ErrorKind::from_error(&err);
        assert!(matches!(kind, ErrorKind::Timeout { seconds: 45 }));
    }

    #[test]
    fn test_detect_llm_error() {
        let err = anyhow!("LLM request failed: model not loaded");
        let kind = ErrorKind::from_error(&err);
        assert!(matches!(kind, ErrorKind::LlmUnavailable));
    }
}
