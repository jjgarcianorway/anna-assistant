//! Pipeline v0.0.82 - Pre-Router Integration + Direct Handlers
//!
//! This module integrates the pre-router with direct handlers for:
//! - Stats query (RPG stats, XP, level)
//! - Editor detection ($VISUAL, $EDITOR, installed editors)
//! - Updates check (pacman updates)
//! - Memory/RAM queries (with typo tolerance for "rum")
//! - Disk space queries
//! - Debug toggle
//! - Capabilities/help
//!
//! If pre-router matches, skip translator entirely and handle directly.

use crate::debug_toggle::{generate_toggle_response, toggle_debug_session};
use crate::pre_router::{pre_route, PreRouterIntent, PreRouterResult};
use std::env;
use std::process::Command;

/// Result of direct handler execution
#[derive(Debug, Clone)]
pub struct DirectHandlerResult {
    /// Whether a direct handler was invoked
    pub handled: bool,
    /// The response message
    pub response: String,
    /// Reliability score (100 for deterministic)
    pub reliability: u8,
    /// Evidence ID if any
    pub evidence_id: Option<String>,
    /// Intent type for case file
    pub intent: String,
}

impl DirectHandlerResult {
    fn not_handled() -> Self {
        Self {
            handled: false,
            response: String::new(),
            reliability: 0,
            evidence_id: None,
            intent: "unknown".to_string(),
        }
    }
}

/// Try to handle request directly via pre-router
/// Returns Some(result) if handled, None if should go to translator
pub fn try_direct_handle(request: &str) -> DirectHandlerResult {
    let pre_result = pre_route(request);

    if !pre_result.matched {
        return DirectHandlerResult::not_handled();
    }

    match pre_result.intent {
        PreRouterIntent::StatsQuery => handle_stats_query(),
        PreRouterIntent::DebugToggle => handle_debug_toggle(request),
        PreRouterIntent::UpdatesQuery => handle_updates_query(),
        PreRouterIntent::MemoryQuery => handle_memory_query(),
        PreRouterIntent::DiskQuery => handle_disk_query(),
        PreRouterIntent::EditorQuery => handle_editor_query(&pre_result),
        PreRouterIntent::CapabilitiesQuery => handle_capabilities_query(),
        PreRouterIntent::CpuQuery => handle_cpu_query(),
        PreRouterIntent::KernelQuery => handle_kernel_query(),
        PreRouterIntent::NetworkQuery => handle_network_query(),
        PreRouterIntent::ServiceQuery => DirectHandlerResult::not_handled(), // Needs LLM
        PreRouterIntent::NoMatch => DirectHandlerResult::not_handled(),
    }
}

// =============================================================================
// Stats Query Handler
// =============================================================================

fn handle_stats_query() -> DirectHandlerResult {
    // Try to load RPG stats
    let stats_response = match load_rpg_stats() {
        Some(stats) => stats,
        None => "No stats yet. Keep using Anna to earn XP!".to_string(),
    };

    DirectHandlerResult {
        handled: true,
        response: stats_response,
        reliability: 100,
        evidence_id: Some("rpg_stats".to_string()),
        intent: "stats_query".to_string(),
    }
}

fn load_rpg_stats() -> Option<String> {
    use crate::rpg_stats::RpgStatsManager;

    let stats = RpgStatsManager::get_stats();

    // Use the built-in format_status_block if stats are available
    if stats.is_available() {
        Some(stats.format_status_block())
    } else {
        None
    }
}

// =============================================================================
// Debug Toggle Handler
// =============================================================================

fn handle_debug_toggle(request: &str) -> DirectHandlerResult {
    let lower = request.to_lowercase();

    // Determine enable/disable
    let enable = lower.contains("enable")
        || lower.contains(" on")
        || lower.contains("turn on")
        || lower.contains("start");

    let result = toggle_debug_session(enable);
    let response = generate_toggle_response(&result);

    DirectHandlerResult {
        handled: true,
        response,
        reliability: 100,
        evidence_id: None,
        intent: "debug_toggle".to_string(),
    }
}

// =============================================================================
// Updates Query Handler
// =============================================================================

fn handle_updates_query() -> DirectHandlerResult {
    let (response, evidence_id) = check_system_updates();

    DirectHandlerResult {
        handled: true,
        response,
        reliability: 100,
        evidence_id: Some(evidence_id),
        intent: "updates_query".to_string(),
    }
}

fn check_system_updates() -> (String, String) {
    // Try checkupdates first (safer)
    let output = Command::new("checkupdates").output();

    match output {
        Ok(result) if result.status.success() => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let lines: Vec<&str> = stdout.lines().collect();

            if lines.is_empty() {
                (
                    "System is up to date. No pending updates.".to_string(),
                    "checkupdates".to_string(),
                )
            } else {
                let mut response = format!("{} updates available:\n\n", lines.len());

                // Show first 10 updates
                for line in lines.iter().take(10) {
                    response.push_str(&format!("  {}\n", line));
                }

                if lines.len() > 10 {
                    response.push_str(&format!("\n  ... and {} more\n", lines.len() - 10));
                }

                response.push_str("\nRun 'sudo pacman -Syu' to install updates.");
                (response, "checkupdates".to_string())
            }
        }
        Ok(result) if result.status.code() == Some(2) => {
            // Exit code 2 = no updates available
            (
                "System is up to date. No pending updates.".to_string(),
                "checkupdates".to_string(),
            )
        }
        _ => {
            // checkupdates not available or failed, try pacman -Qu
            match Command::new("pacman").args(["-Qu"]).output() {
                Ok(result) if result.status.success() => {
                    let stdout = String::from_utf8_lossy(&result.stdout);
                    let lines: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();

                    if lines.is_empty() {
                        (
                            "System is up to date. No pending updates.".to_string(),
                            "pacman_qu".to_string(),
                        )
                    } else {
                        let mut response = format!("{} updates available:\n\n", lines.len());
                        for line in lines.iter().take(10) {
                            response.push_str(&format!("  {}\n", line));
                        }
                        if lines.len() > 10 {
                            response.push_str(&format!("\n  ... and {} more\n", lines.len() - 10));
                        }
                        response.push_str("\nRun 'sudo pacman -Syu' to install updates.");
                        (response, "pacman_qu".to_string())
                    }
                }
                _ => (
                    "Cannot check updates. Pacman database may need sync (pacman -Sy).".to_string(),
                    "error".to_string(),
                ),
            }
        }
    }
}

// =============================================================================
// Memory Query Handler (with typo tolerance for "rum")
// =============================================================================

fn handle_memory_query() -> DirectHandlerResult {
    let (response, evidence_id) = get_memory_info();

    DirectHandlerResult {
        handled: true,
        response,
        reliability: 100,
        evidence_id: Some(evidence_id),
        intent: "memory_query".to_string(),
    }
}

fn get_memory_info() -> (String, String) {
    // Read /proc/meminfo
    match std::fs::read_to_string("/proc/meminfo") {
        Ok(content) => {
            let mut total_kb = 0u64;
            let mut available_kb = 0u64;
            let mut free_kb = 0u64;
            let mut buffers_kb = 0u64;
            let mut cached_kb = 0u64;

            for line in content.lines() {
                if let Some((key, value)) = line.split_once(':') {
                    let value = value.trim().trim_end_matches(" kB").trim();
                    if let Ok(v) = value.parse::<u64>() {
                        match key {
                            "MemTotal" => total_kb = v,
                            "MemAvailable" => available_kb = v,
                            "MemFree" => free_kb = v,
                            "Buffers" => buffers_kb = v,
                            "Cached" => cached_kb = v,
                            _ => {}
                        }
                    }
                }
            }

            let total_gb = total_kb as f64 / 1024.0 / 1024.0;
            let available_gb = available_kb as f64 / 1024.0 / 1024.0;
            let used_gb = total_gb - available_gb;
            let used_percent = (used_gb / total_gb) * 100.0;

            let response = format!(
                "Memory Usage:\n\n\
                 Total:     {:.1} GB\n\
                 Used:      {:.1} GB ({:.0}%)\n\
                 Available: {:.1} GB\n\n\
                 Breakdown:\n\
                   Free:    {:.1} GB\n\
                   Buffers: {:.1} GB\n\
                   Cached:  {:.1} GB",
                total_gb,
                used_gb,
                used_percent,
                available_gb,
                free_kb as f64 / 1024.0 / 1024.0,
                buffers_kb as f64 / 1024.0 / 1024.0,
                cached_kb as f64 / 1024.0 / 1024.0
            );

            (response, "proc_meminfo".to_string())
        }
        Err(_) => (
            "Cannot read memory info from /proc/meminfo".to_string(),
            "error".to_string(),
        ),
    }
}

// =============================================================================
// Disk Query Handler
// =============================================================================

fn handle_disk_query() -> DirectHandlerResult {
    let (response, evidence_id) = get_disk_info();

    DirectHandlerResult {
        handled: true,
        response,
        reliability: 100,
        evidence_id: Some(evidence_id),
        intent: "disk_query".to_string(),
    }
}

fn get_disk_info() -> (String, String) {
    match Command::new("df")
        .args(["-h", "--output=target,size,used,avail,pcent", "-x", "tmpfs", "-x", "devtmpfs"])
        .output()
    {
        Ok(result) if result.status.success() => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let mut response = String::from("Disk Usage:\n\n");

            for line in stdout.lines() {
                response.push_str(&format!("{}\n", line));
            }

            (response, "df".to_string())
        }
        _ => (
            "Cannot get disk usage information.".to_string(),
            "error".to_string(),
        ),
    }
}

// =============================================================================
// Editor Query Handler
// =============================================================================

fn handle_editor_query(_pre_result: &PreRouterResult) -> DirectHandlerResult {
    let (response, evidence_id) = detect_editor();

    DirectHandlerResult {
        handled: true,
        response,
        reliability: 100,
        evidence_id: Some(evidence_id),
        intent: "editor_query".to_string(),
    }
}

fn detect_editor() -> (String, String) {
    let mut editors_found = Vec::new();

    // Check $VISUAL first (highest priority)
    if let Ok(visual) = env::var("VISUAL") {
        editors_found.push(format!("$VISUAL: {}", visual));
    }

    // Check $EDITOR
    if let Ok(editor) = env::var("EDITOR") {
        editors_found.push(format!("$EDITOR: {}", editor));
    }

    // Check for installed editors
    let editor_commands = [
        ("nvim", "Neovim"),
        ("vim", "Vim"),
        ("emacs", "Emacs"),
        ("code", "VS Code"),
        ("nano", "Nano"),
        ("helix", "Helix"),
        ("kate", "Kate"),
        ("gedit", "Gedit"),
        ("micro", "Micro"),
        ("sublime_text", "Sublime Text"),
    ];

    let mut installed = Vec::new();
    for (cmd, name) in editor_commands {
        if Command::new("which").arg(cmd).output().map_or(false, |o| o.status.success()) {
            installed.push(name.to_string());
        }
    }

    let mut response = String::new();

    if !editors_found.is_empty() {
        response.push_str("Default Editor:\n");
        for e in &editors_found {
            response.push_str(&format!("  {}\n", e));
        }
        response.push('\n');
    }

    if !installed.is_empty() {
        response.push_str("Installed Editors:\n");
        for e in &installed {
            response.push_str(&format!("  - {}\n", e));
        }
    }

    if response.is_empty() {
        response = "No editor configured. Set $EDITOR or $VISUAL environment variable.".to_string();
    }

    (response, "editor_detection".to_string())
}

// =============================================================================
// Capabilities Query Handler
// =============================================================================

fn handle_capabilities_query() -> DirectHandlerResult {
    let response = r#"I'm Anna, your Arch Linux assistant. Here's what I can help with:

SYSTEM QUERIES
  - "how much ram/memory"    - Check memory usage
  - "disk space"             - Check storage
  - "what cpu"               - CPU information
  - "kernel version"         - Linux kernel info
  - "check updates"          - Pending system updates
  - "stats"                  - Your RPG stats and level

TROUBLESHOOTING
  - "wifi not working"       - Network diagnosis
  - "audio issues"           - Sound troubleshooting
  - "slow boot"              - Boot time analysis
  - "display problems"       - Graphics diagnosis

ACTIONS (with confirmation)
  - "install <package>"      - Install packages
  - "restart <service>"      - Service management
  - "edit <config>"          - Configuration changes

DEBUG
  - "enable debug"           - Show internal details
  - "disable debug"          - Clean output mode

Ask me anything about your system!"#;

    DirectHandlerResult {
        handled: true,
        response: response.to_string(),
        reliability: 100,
        evidence_id: None,
        intent: "capabilities_query".to_string(),
    }
}

// =============================================================================
// CPU Query Handler
// =============================================================================

fn handle_cpu_query() -> DirectHandlerResult {
    let (response, evidence_id) = get_cpu_info();

    DirectHandlerResult {
        handled: true,
        response,
        reliability: 100,
        evidence_id: Some(evidence_id),
        intent: "cpu_query".to_string(),
    }
}

fn get_cpu_info() -> (String, String) {
    match std::fs::read_to_string("/proc/cpuinfo") {
        Ok(content) => {
            let mut model_name = String::new();
            let mut cores = 0;
            let mut cpu_mhz = String::new();

            for line in content.lines() {
                if let Some((key, value)) = line.split_once(':') {
                    let key = key.trim();
                    let value = value.trim();
                    match key {
                        "model name" if model_name.is_empty() => {
                            model_name = value.to_string();
                        }
                        "cpu MHz" if cpu_mhz.is_empty() => {
                            cpu_mhz = value.to_string();
                        }
                        "processor" => {
                            cores += 1;
                        }
                        _ => {}
                    }
                }
            }

            let response = format!(
                "CPU Information:\n\n\
                 Model:  {}\n\
                 Cores:  {}\n\
                 Speed:  {} MHz",
                model_name, cores, cpu_mhz
            );

            (response, "proc_cpuinfo".to_string())
        }
        Err(_) => (
            "Cannot read CPU info from /proc/cpuinfo".to_string(),
            "error".to_string(),
        ),
    }
}

// =============================================================================
// Kernel Query Handler
// =============================================================================

fn handle_kernel_query() -> DirectHandlerResult {
    let (response, evidence_id) = get_kernel_info();

    DirectHandlerResult {
        handled: true,
        response,
        reliability: 100,
        evidence_id: Some(evidence_id),
        intent: "kernel_query".to_string(),
    }
}

fn get_kernel_info() -> (String, String) {
    let kernel_release =
        std::fs::read_to_string("/proc/sys/kernel/osrelease").unwrap_or_else(|_| "unknown".into());

    let kernel_version =
        std::fs::read_to_string("/proc/sys/kernel/version").unwrap_or_else(|_| "unknown".into());

    let response = format!(
        "Kernel Information:\n\n\
         Release: {}\n\
         Version: {}",
        kernel_release.trim(),
        kernel_version.trim()
    );

    (response, "proc_kernel".to_string())
}

// =============================================================================
// Network Query Handler
// =============================================================================

fn handle_network_query() -> DirectHandlerResult {
    let (response, evidence_id) = get_network_info();

    DirectHandlerResult {
        handled: true,
        response,
        reliability: 100,
        evidence_id: Some(evidence_id),
        intent: "network_query".to_string(),
    }
}

fn get_network_info() -> (String, String) {
    // Check connectivity
    let is_connected = Command::new("ping")
        .args(["-c", "1", "-W", "2", "8.8.8.8"])
        .output()
        .map_or(false, |o| o.status.success());

    // Get interface info
    let ip_output = Command::new("ip")
        .args(["addr", "show"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).to_string())
            } else {
                None
            }
        });

    let mut response = String::new();

    response.push_str(&format!(
        "Network Status: {}\n\n",
        if is_connected { "Connected" } else { "Disconnected" }
    ));

    if let Some(ip_info) = ip_output {
        response.push_str("Interfaces:\n");
        for line in ip_info.lines() {
            if line.contains("inet ") || line.starts_with(|c: char| c.is_ascii_digit()) {
                response.push_str(&format!("  {}\n", line.trim()));
            }
        }
    }

    (response, "ip_addr".to_string())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_direct_handle_stats() {
        let result = try_direct_handle("stats");
        assert!(result.handled);
        assert_eq!(result.intent, "stats_query");
        assert_eq!(result.reliability, 100);
    }

    #[test]
    fn test_try_direct_handle_memory() {
        let result = try_direct_handle("how much ram");
        assert!(result.handled);
        assert_eq!(result.intent, "memory_query");
    }

    #[test]
    fn test_try_direct_handle_memory_typo() {
        // "rum" typo should be handled as memory
        let result = try_direct_handle("how much free rum");
        assert!(result.handled);
        assert_eq!(result.intent, "memory_query");
    }

    #[test]
    fn test_try_direct_handle_disk() {
        let result = try_direct_handle("disk space");
        assert!(result.handled);
        assert_eq!(result.intent, "disk_query");
    }

    #[test]
    fn test_try_direct_handle_updates() {
        let result = try_direct_handle("check for updates");
        assert!(result.handled);
        assert_eq!(result.intent, "updates_query");
    }

    #[test]
    fn test_try_direct_handle_editor() {
        let result = try_direct_handle("what editor am i using");
        assert!(result.handled);
        assert_eq!(result.intent, "editor_query");
    }

    #[test]
    fn test_try_direct_handle_capabilities() {
        let result = try_direct_handle("what can you do");
        assert!(result.handled);
        assert_eq!(result.intent, "capabilities_query");
    }

    #[test]
    fn test_try_direct_handle_cpu() {
        let result = try_direct_handle("what cpu");
        assert!(result.handled);
        assert_eq!(result.intent, "cpu_query");
    }

    #[test]
    fn test_try_direct_handle_kernel() {
        let result = try_direct_handle("kernel version");
        assert!(result.handled);
        assert_eq!(result.intent, "kernel_query");
    }

    #[test]
    fn test_try_direct_handle_no_match() {
        let result = try_direct_handle("restart nginx");
        assert!(!result.handled);
    }
}
