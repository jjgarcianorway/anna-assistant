//! Pre-Router for Anna v0.0.82 - "Stop the Nonsense"
//!
//! Fast, deterministic routing BEFORE the translator.
//! Handles common intents without LLM for reliable UX.
//!
//! If pre-router matches, skip translator entirely and go straight
//! to the correct probe pipeline + answer generation.

use serde::{Deserialize, Serialize};

/// Pre-router result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreRouterResult {
    /// Whether a match was found
    pub matched: bool,
    /// The matched intent
    pub intent: PreRouterIntent,
    /// Confidence (always 100 for deterministic matches)
    pub confidence: u8,
    /// Reason for match (for debug mode)
    pub match_reason: String,
    /// Extracted targets if any
    pub targets: Vec<String>,
}

/// Pre-router intent types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreRouterIntent {
    /// RPG stats query (xp, level, titles)
    StatsQuery,
    /// Updates/upgrade check
    UpdatesQuery,
    /// Memory/RAM query
    MemoryQuery,
    /// Disk/storage space query
    DiskQuery,
    /// Editor detection query
    EditorQuery,
    /// Debug mode toggle request
    DebugToggle,
    /// Capabilities/help query
    CapabilitiesQuery,
    /// CPU info query
    CpuQuery,
    /// Kernel/system info query
    KernelQuery,
    /// Network status query
    NetworkQuery,
    /// Service status query
    ServiceQuery,
    /// No match - fall through to translator
    NoMatch,
}

impl PreRouterIntent {
    /// Human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            PreRouterIntent::StatsQuery => "RPG stats and history",
            PreRouterIntent::UpdatesQuery => "system updates check",
            PreRouterIntent::MemoryQuery => "memory/RAM info",
            PreRouterIntent::DiskQuery => "disk/storage space",
            PreRouterIntent::EditorQuery => "editor detection",
            PreRouterIntent::DebugToggle => "debug mode toggle",
            PreRouterIntent::CapabilitiesQuery => "capabilities/help",
            PreRouterIntent::CpuQuery => "CPU information",
            PreRouterIntent::KernelQuery => "kernel/system info",
            PreRouterIntent::NetworkQuery => "network status",
            PreRouterIntent::ServiceQuery => "service status",
            PreRouterIntent::NoMatch => "unknown",
        }
    }

    /// Is this a read-only intent?
    pub fn is_read_only(&self) -> bool {
        !matches!(self, PreRouterIntent::DebugToggle)
    }
}

/// Run the pre-router on a request
pub fn pre_route(request: &str) -> PreRouterResult {
    let lower = request.to_lowercase();
    let words: Vec<&str> = lower.split_whitespace().collect();

    // 1. Stats query - RPG stats, XP, level, titles
    if is_stats_query(&lower, &words) {
        return PreRouterResult {
            matched: true,
            intent: PreRouterIntent::StatsQuery,
            confidence: 100,
            match_reason: "matched stats/rpg keywords".to_string(),
            targets: vec![],
        };
    }

    // 2. Debug toggle - enable/disable debug mode
    if is_debug_toggle(&lower) {
        return PreRouterResult {
            matched: true,
            intent: PreRouterIntent::DebugToggle,
            confidence: 100,
            match_reason: "matched debug toggle pattern".to_string(),
            targets: vec![],
        };
    }

    // 3. Updates query - check for system updates
    if is_updates_query(&lower, &words) {
        return PreRouterResult {
            matched: true,
            intent: PreRouterIntent::UpdatesQuery,
            confidence: 100,
            match_reason: "matched updates/upgrade keywords".to_string(),
            targets: vec![],
        };
    }

    // 4. Memory query - RAM, free memory
    if is_memory_query(&lower, &words) {
        return PreRouterResult {
            matched: true,
            intent: PreRouterIntent::MemoryQuery,
            confidence: 100,
            match_reason: "matched memory/ram keywords".to_string(),
            targets: vec![],
        };
    }

    // 5. Disk query - storage, disk space
    if is_disk_query(&lower, &words) {
        return PreRouterResult {
            matched: true,
            intent: PreRouterIntent::DiskQuery,
            confidence: 100,
            match_reason: "matched disk/storage keywords".to_string(),
            targets: vec![],
        };
    }

    // 6. Editor query - vim, nvim, emacs, code, etc.
    if is_editor_query(&lower, &words) {
        return PreRouterResult {
            matched: true,
            intent: PreRouterIntent::EditorQuery,
            confidence: 100,
            match_reason: "matched editor keywords".to_string(),
            targets: extract_editor_targets(&lower),
        };
    }

    // 7. Capabilities query - what can you do, help
    if is_capabilities_query(&lower) {
        return PreRouterResult {
            matched: true,
            intent: PreRouterIntent::CapabilitiesQuery,
            confidence: 100,
            match_reason: "matched capabilities/help pattern".to_string(),
            targets: vec![],
        };
    }

    // 8. CPU query
    if is_cpu_query(&lower, &words) {
        return PreRouterResult {
            matched: true,
            intent: PreRouterIntent::CpuQuery,
            confidence: 100,
            match_reason: "matched cpu keywords".to_string(),
            targets: vec![],
        };
    }

    // 9. Kernel query
    if is_kernel_query(&lower, &words) {
        return PreRouterResult {
            matched: true,
            intent: PreRouterIntent::KernelQuery,
            confidence: 100,
            match_reason: "matched kernel keywords".to_string(),
            targets: vec![],
        };
    }

    // 10. Network query
    if is_network_query(&lower, &words) {
        return PreRouterResult {
            matched: true,
            intent: PreRouterIntent::NetworkQuery,
            confidence: 100,
            match_reason: "matched network keywords".to_string(),
            targets: vec![],
        };
    }

    // No match - fall through to translator
    PreRouterResult {
        matched: false,
        intent: PreRouterIntent::NoMatch,
        confidence: 0,
        match_reason: "no deterministic match".to_string(),
        targets: vec![],
    }
}

// =============================================================================
// Intent Detection Functions
// =============================================================================

fn is_stats_query(lower: &str, words: &[&str]) -> bool {
    // Exact "stats" command
    if words == ["stats"] || lower == "annactl stats" {
        return true;
    }

    // RPG-related keywords
    let rpg_keywords = ["rpg", "xp", "level", "titles", "experience", "my stats"];
    for kw in rpg_keywords {
        if lower.contains(kw) {
            return true;
        }
    }

    // "show stats", "my stats", "anna stats"
    if (lower.contains("show") || lower.contains("my") || lower.contains("anna"))
        && lower.contains("stats")
    {
        return true;
    }

    false
}

fn is_debug_toggle(lower: &str) -> bool {
    let patterns = [
        "enable debug",
        "disable debug",
        "debug on",
        "debug off",
        "turn debug on",
        "turn debug off",
        "toggle debug",
        "debug mode on",
        "debug mode off",
        "enable debug mode",
        "disable debug mode",
    ];

    for pattern in patterns {
        if lower.contains(pattern) {
            return true;
        }
    }

    false
}

fn is_updates_query(lower: &str, words: &[&str]) -> bool {
    // Direct update-related queries
    let update_keywords = [
        "update",
        "updates",
        "upgrade",
        "upgrades",
        "pacman",
        "checkupdates",
    ];

    for kw in update_keywords {
        if lower.contains(kw) {
            // Exclude action requests like "install updates"
            if lower.contains("install") || lower.contains("run") || lower.contains("do") {
                // Still match "check" variants
                if lower.contains("check") || lower.contains("any") || lower.contains("pending") {
                    return true;
                }
                return false;
            }
            return true;
        }
    }

    // "check for updates", "any updates", "pending updates"
    if lower.contains("check") && (lower.contains("update") || lower.contains("upgrade")) {
        return true;
    }

    false
}

fn is_memory_query(lower: &str, _words: &[&str]) -> bool {
    let memory_keywords = ["ram", "memory", "mem ", " mem", "free ram", "available memory"];

    for kw in memory_keywords {
        if lower.contains(kw) {
            return true;
        }
    }

    // Typo tolerance: "rum" in memory context
    if lower.contains("rum") && (lower.contains("free") || lower.contains("how much")) {
        return true;
    }

    false
}

fn is_disk_query(lower: &str, _words: &[&str]) -> bool {
    let disk_keywords = [
        "disk",
        "storage",
        "space",
        "filesystem",
        "mount",
        "partition",
        "df ",
        " df",
    ];

    for kw in disk_keywords {
        if lower.contains(kw) {
            return true;
        }
    }

    // "free space", "disk space", "storage space"
    if lower.contains("space") {
        return true;
    }

    false
}

fn is_editor_query(lower: &str, _words: &[&str]) -> bool {
    let editor_keywords = [
        "editor",
        "vim",
        "nvim",
        "neovim",
        "emacs",
        "nano",
        "code",
        "vscode",
        "visual studio",
        "helix",
        "kate",
        "gedit",
        "sublime",
    ];

    for kw in editor_keywords {
        if lower.contains(kw) {
            // Must be a query, not an action
            if lower.contains("install")
                || lower.contains("remove")
                || lower.contains("open")
                || lower.contains("start")
            {
                return false;
            }
            return true;
        }
    }

    // "what editor", "which editor", "my editor"
    if (lower.contains("what") || lower.contains("which") || lower.contains("my"))
        && lower.contains("editor")
    {
        return true;
    }

    false
}

fn extract_editor_targets(lower: &str) -> Vec<String> {
    let editors = [
        "vim", "nvim", "neovim", "emacs", "nano", "code", "vscode", "helix", "kate", "gedit",
        "sublime",
    ];

    let mut found = Vec::new();
    for ed in editors {
        if lower.contains(ed) {
            found.push(ed.to_string());
        }
    }
    found
}

fn is_capabilities_query(lower: &str) -> bool {
    let patterns = [
        "what can you do",
        "what can anna do",
        "your capabilities",
        "help me",
        "how can you help",
        "what are you capable",
        "what do you do",
        "tell me about yourself",
        "introduce yourself",
    ];

    for pattern in patterns {
        if lower.contains(pattern) {
            return true;
        }
    }

    false
}

fn is_cpu_query(lower: &str, _words: &[&str]) -> bool {
    let cpu_keywords = ["cpu", "processor", "cores", "threads", "ghz", "mhz"];

    for kw in cpu_keywords {
        if lower.contains(kw) {
            return true;
        }
    }

    false
}

fn is_kernel_query(lower: &str, _words: &[&str]) -> bool {
    let kernel_keywords = [
        "kernel",
        "linux version",
        "uname",
        "system version",
        "os version",
        "arch version",
    ];

    for kw in kernel_keywords {
        if lower.contains(kw) {
            return true;
        }
    }

    false
}

fn is_network_query(lower: &str, _words: &[&str]) -> bool {
    let network_keywords = [
        "network",
        "wifi",
        "ethernet",
        "ip address",
        "connected",
        "internet",
        "connection",
    ];

    for kw in network_keywords {
        if lower.contains(kw) {
            // Exclude diagnostic requests
            if lower.contains("not working")
                || lower.contains("broken")
                || lower.contains("fix")
                || lower.contains("problem")
            {
                return false;
            }
            return true;
        }
    }

    false
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_query() {
        assert!(pre_route("stats").matched);
        assert_eq!(pre_route("stats").intent, PreRouterIntent::StatsQuery);

        assert!(pre_route("show my stats").matched);
        assert!(pre_route("what's my xp").matched);
        assert!(pre_route("my level").matched);
    }

    #[test]
    fn test_debug_toggle() {
        assert!(pre_route("enable debug mode").matched);
        assert_eq!(
            pre_route("enable debug mode").intent,
            PreRouterIntent::DebugToggle
        );

        assert!(pre_route("turn debug off").matched);
        assert!(pre_route("disable debug").matched);
    }

    #[test]
    fn test_updates_query() {
        assert!(pre_route("check for updates").matched);
        assert_eq!(
            pre_route("check for updates").intent,
            PreRouterIntent::UpdatesQuery
        );

        assert!(pre_route("any updates?").matched);
        assert!(pre_route("pending updates").matched);
        assert!(pre_route("can you check updates").matched);
    }

    #[test]
    fn test_memory_query() {
        assert!(pre_route("how much ram").matched);
        assert_eq!(pre_route("how much ram").intent, PreRouterIntent::MemoryQuery);

        assert!(pre_route("free memory").matched);
        assert!(pre_route("available ram").matched);

        // Typo tolerance
        assert!(pre_route("how much free rum").matched);
    }

    #[test]
    fn test_editor_query() {
        assert!(pre_route("what editor am i using").matched);
        assert_eq!(
            pre_route("what editor am i using").intent,
            PreRouterIntent::EditorQuery
        );

        assert!(pre_route("which editor").matched);
        assert!(pre_route("my editor").matched);

        // Should not match install requests
        assert!(!pre_route("install vim").matched);
    }

    #[test]
    fn test_disk_query() {
        assert!(pre_route("disk space").matched);
        assert_eq!(pre_route("disk space").intent, PreRouterIntent::DiskQuery);

        assert!(pre_route("free space").matched);
        assert!(pre_route("storage").matched);
    }

    #[test]
    fn test_capabilities_query() {
        assert!(pre_route("what can you do").matched);
        assert_eq!(
            pre_route("what can you do").intent,
            PreRouterIntent::CapabilitiesQuery
        );

        assert!(pre_route("help me").matched);
    }

    #[test]
    fn test_no_match() {
        let result = pre_route("restart nginx");
        assert!(!result.matched);
        assert_eq!(result.intent, PreRouterIntent::NoMatch);
    }

    #[test]
    fn test_cpu_query() {
        assert!(pre_route("what cpu do i have").matched);
        assert_eq!(pre_route("what cpu").intent, PreRouterIntent::CpuQuery);
    }

    #[test]
    fn test_kernel_query() {
        assert!(pre_route("kernel version").matched);
        assert_eq!(
            pre_route("kernel version").intent,
            PreRouterIntent::KernelQuery
        );
    }
}
