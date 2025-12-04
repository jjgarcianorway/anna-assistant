//! Transcript Configuration for Anna v0.0.81
//!
//! Unified configuration for transcript rendering modes:
//! - Human mode: Professional IT department dialogue without raw internals
//! - Debug mode: Full tool names, evidence IDs, timing, retries, fallbacks
//!
//! Mode switching via (priority order):
//! 1. ANNA_DEBUG_TRANSCRIPT=1 env var
//! 2. ANNA_UI_TRANSCRIPT_MODE=debug|human env var
//! 3. Config file: /etc/anna/config.toml [ui] transcript_mode
//! 4. Default: human

use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

/// Global transcript mode cache
static TRANSCRIPT_MODE: OnceLock<TranscriptMode> = OnceLock::new();

/// Transcript rendering mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TranscriptMode {
    /// Professional IT department dialogue
    Human,
    /// Full internal details
    Debug,
}

impl Default for TranscriptMode {
    fn default() -> Self {
        TranscriptMode::Human
    }
}

impl TranscriptMode {
    /// Parse from string (case-insensitive)
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "debug" | "d" | "1" | "true" => TranscriptMode::Debug,
            _ => TranscriptMode::Human,
        }
    }

    /// Should show internal tool names?
    pub fn show_tool_names(&self) -> bool {
        matches!(self, TranscriptMode::Debug)
    }

    /// Should show evidence IDs?
    pub fn show_evidence_ids(&self) -> bool {
        matches!(self, TranscriptMode::Debug)
    }

    /// Should show timing info?
    pub fn show_timing(&self) -> bool {
        matches!(self, TranscriptMode::Debug)
    }

    /// Should show retries and fallbacks?
    pub fn show_retries(&self) -> bool {
        matches!(self, TranscriptMode::Debug)
    }

    /// Should show parse warnings?
    pub fn show_parse_warnings(&self) -> bool {
        matches!(self, TranscriptMode::Debug)
    }
}

/// Get the current transcript mode (cached)
pub fn get_transcript_mode() -> TranscriptMode {
    *TRANSCRIPT_MODE.get_or_init(detect_transcript_mode)
}

/// Check if currently in debug mode
pub fn is_debug_mode() -> bool {
    matches!(get_transcript_mode(), TranscriptMode::Debug)
}

/// Detect transcript mode from environment and config
fn detect_transcript_mode() -> TranscriptMode {
    // Priority 1: ANNA_DEBUG_TRANSCRIPT=1
    if std::env::var("ANNA_DEBUG_TRANSCRIPT")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        return TranscriptMode::Debug;
    }

    // Priority 2: ANNA_UI_TRANSCRIPT_MODE=debug
    if let Ok(mode) = std::env::var("ANNA_UI_TRANSCRIPT_MODE") {
        if mode.eq_ignore_ascii_case("debug") {
            return TranscriptMode::Debug;
        }
    }

    // Priority 3: Config file /etc/anna/config.toml
    if let Ok(contents) = std::fs::read_to_string("/etc/anna/config.toml") {
        if contents.contains("transcript_mode = \"debug\"")
            || contents.contains("transcript_mode = 'debug'")
        {
            return TranscriptMode::Debug;
        }
    }

    // Priority 4: Default is human mode
    TranscriptMode::Human
}

// =============================================================================
// Humanized Equivalents (v0.0.81)
// =============================================================================

/// Humanize a deterministic fallback message
pub fn humanize_fallback() -> &'static str {
    "Translator struggled to classify this; we used house rules."
}

/// Humanize a tool name for human mode
pub fn humanize_tool_name(tool_name: &str) -> String {
    match tool_name {
        "hw_snapshot_summary" => "Hardware inventory".to_string(),
        "hw_snapshot_cpu" => "CPU snapshot".to_string(),
        "hw_snapshot_memory" => "Memory status".to_string(),
        "hw_snapshot_disk" => "Storage snapshot".to_string(),
        "hw_snapshot_gpu" => "Graphics info".to_string(),
        "sw_snapshot_kernel" => "Kernel info".to_string(),
        "sw_snapshot_packages" => "Package list".to_string(),
        "sw_snapshot_services" => "Service status".to_string(),
        "net_snapshot" => "Network snapshot".to_string(),
        "audio_snapshot" => "Audio status".to_string(),
        "boot_snapshot" => "Boot timeline".to_string(),
        "journal_errors" => "Error journal".to_string(),
        "service_status" => "Service check".to_string(),
        _ => {
            // Convert snake_case to "Title Case" and clean up
            tool_name
                .split('_')
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        Some(first) => first.to_uppercase().chain(chars).collect(),
                        None => String::new(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        }
    }
}

/// Humanize evidence reference
pub fn humanize_evidence_ref(evidence_id: &str, tool_name: &str) -> String {
    let humanized_source = humanize_tool_name(tool_name);
    format!("Pulled data from the latest {}", humanized_source.to_lowercase())
}

/// Strip evidence IDs from a string for human mode
pub fn strip_evidence_ids(text: &str) -> String {
    // Remove [E1], [E2], etc.
    let re = regex::Regex::new(r"\[E\d+\]").unwrap();
    re.replace_all(text, "").to_string()
}

/// Humanize a parse retry message
pub fn humanize_parse_retry(attempt: u8) -> &'static str {
    match attempt {
        1 => "Rephrasing my query...",
        2 => "Trying a different approach...",
        _ => "Working on it...",
    }
}

/// Humanize LLM timeout or error
pub fn humanize_llm_error(error_kind: &str) -> &'static str {
    match error_kind {
        "timeout" => "Taking a moment to think...",
        "connection" => "Connection hiccup, retrying...",
        _ => "Brief delay, continuing...",
    }
}

// =============================================================================
// Mode-Aware Rendering Helpers
// =============================================================================

/// Render a message conditionally based on mode
pub fn render_if_debug(debug_msg: &str) -> Option<String> {
    if is_debug_mode() {
        Some(debug_msg.to_string())
    } else {
        None
    }
}

/// Render with mode-appropriate version
pub fn render_mode_aware(human_msg: &str, debug_msg: &str) -> String {
    if is_debug_mode() {
        debug_msg.to_string()
    } else {
        human_msg.to_string()
    }
}

/// Format timing for display
pub fn format_timing(ms: u64) -> String {
    if is_debug_mode() {
        format!("{}ms", ms)
    } else if ms > 5000 {
        format!("({:.1}s)", ms as f64 / 1000.0)
    } else {
        String::new() // Don't show sub-5s timing in human mode
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_parsing() {
        assert_eq!(TranscriptMode::from_str("debug"), TranscriptMode::Debug);
        assert_eq!(TranscriptMode::from_str("DEBUG"), TranscriptMode::Debug);
        assert_eq!(TranscriptMode::from_str("1"), TranscriptMode::Debug);
        assert_eq!(TranscriptMode::from_str("human"), TranscriptMode::Human);
        assert_eq!(TranscriptMode::from_str("anything"), TranscriptMode::Human);
    }

    #[test]
    fn test_humanize_tool_names() {
        assert_eq!(humanize_tool_name("hw_snapshot_summary"), "Hardware inventory");
        assert_eq!(humanize_tool_name("journal_errors"), "Error journal");
        assert_eq!(humanize_tool_name("some_custom_tool"), "Some Custom Tool");
    }

    #[test]
    fn test_strip_evidence_ids() {
        let input = "CPU info from [E1] shows 8 cores, memory from [E2] shows 16GB";
        let expected = "CPU info from  shows 8 cores, memory from  shows 16GB";
        assert_eq!(strip_evidence_ids(input), expected);
    }

    #[test]
    fn test_mode_properties() {
        assert!(TranscriptMode::Debug.show_tool_names());
        assert!(!TranscriptMode::Human.show_tool_names());

        assert!(TranscriptMode::Debug.show_evidence_ids());
        assert!(!TranscriptMode::Human.show_evidence_ids());
    }
}
