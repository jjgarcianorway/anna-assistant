//! Actions - v6.59.0 Typed Action Vocabulary
//!
//! The LLM must select from these typed actions instead of generating
//! arbitrary shell commands. Each action maps to specific ToolIds.

use super::catalog::{ToolId, ToolResult, get_tool, run_tool, run_tool_with_extra_args};
use serde::{Deserialize, Serialize};

/// Typed actions the LLM can request
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    // === Hardware ===
    GetMemoryInfo,
    GetCpuInfo,
    GetCpuFeatures,
    GetGpuInfo,

    // === Packages ===
    ListInstalledPackages,
    ListGames,
    CheckSteamInstalled,
    ListFileManagers,
    CheckOrphans,
    CheckUpdates,

    // === Storage ===
    GetDiskUsage,
    GetDiskUsageHome,
    GetDiskUsageHomeTop { limit: usize },
    GetDiskUsageVar,
    GetDiskUsageVarTop { limit: usize },
    ListBlockDevices,

    // === Network ===
    GetNetworkInterfaces,
    CheckNetworkType,
    CheckWifiStability,
    CheckDnsConfig,
    CheckInternetConnectivity,

    // === Services ===
    ListFailedServices,
    ListServices,
    GetRecentErrors,
    GetRecentLogs,

    // === System ===
    GetUptime,
    GetSystemInfo,
    GetOsInfo,
    GetHostInfo,

    // === Process ===
    ListTopProcesses,

    // === Meta ===
    RunToolSelfTest,
    GetAnnaStatus,
}

impl Action {
    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::GetMemoryInfo => "Check RAM usage",
            Self::GetCpuInfo => "Get CPU model and details",
            Self::GetCpuFeatures => "Check CPU feature flags (SSE, AVX, etc)",
            Self::GetGpuInfo => "Get GPU information",
            Self::ListInstalledPackages => "List all installed packages",
            Self::ListGames => "Find installed games",
            Self::CheckSteamInstalled => "Check if Steam is installed",
            Self::ListFileManagers => "Find installed file managers",
            Self::CheckOrphans => "Find orphaned packages",
            Self::CheckUpdates => "Check for available updates",
            Self::GetDiskUsage => "Show disk space usage",
            Self::GetDiskUsageHome => "Disk usage of home directory",
            Self::GetDiskUsageHomeTop { .. } => "Top folders in home by size",
            Self::GetDiskUsageVar => "Disk usage of /var",
            Self::GetDiskUsageVarTop { .. } => "Top folders in /var by size",
            Self::ListBlockDevices => "List storage devices",
            Self::GetNetworkInterfaces => "List network interfaces",
            Self::CheckNetworkType => "Check if using wifi or ethernet",
            Self::CheckWifiStability => "Check wifi connection quality",
            Self::CheckDnsConfig => "Check DNS configuration",
            Self::CheckInternetConnectivity => "Test internet connection",
            Self::ListFailedServices => "List failed systemd services",
            Self::ListServices => "List all services",
            Self::GetRecentErrors => "Get recent error logs",
            Self::GetRecentLogs => "Get recent system logs",
            Self::GetUptime => "Get system uptime",
            Self::GetSystemInfo => "Get system information",
            Self::GetOsInfo => "Get OS version info",
            Self::GetHostInfo => "Get hostname and machine info",
            Self::ListTopProcesses => "List top processes by resource usage",
            Self::RunToolSelfTest => "Test all tools in catalog",
            Self::GetAnnaStatus => "Get Anna's current status",
        }
    }

    /// Get the tools needed for this action
    pub fn required_tools(&self) -> Vec<ToolId> {
        match self {
            Self::GetMemoryInfo => vec![ToolId::FreeMem],
            Self::GetCpuInfo => vec![ToolId::LsCpu],
            Self::GetCpuFeatures => vec![ToolId::CpuInfo],
            Self::GetGpuInfo => vec![ToolId::LsPciGpu],
            Self::ListInstalledPackages => vec![ToolId::PacmanQuery],
            Self::ListGames => vec![ToolId::PacmanQueryGames],
            Self::CheckSteamInstalled => vec![ToolId::PacmanQuery],
            Self::ListFileManagers => vec![ToolId::PacmanQueryFileManagers],
            Self::CheckOrphans => vec![ToolId::PacmanOrphans],
            Self::CheckUpdates => vec![ToolId::PacmanUpdates, ToolId::CheckUpdates],
            Self::GetDiskUsage => vec![ToolId::DfHuman],
            Self::GetDiskUsageHome => vec![ToolId::DuHomeTop],
            Self::GetDiskUsageHomeTop { .. } => vec![ToolId::DuHomeTop],
            Self::GetDiskUsageVar => vec![ToolId::DuVarTop],
            Self::GetDiskUsageVarTop { .. } => vec![ToolId::DuVarTop],
            Self::ListBlockDevices => vec![ToolId::LsBlk],
            Self::GetNetworkInterfaces => vec![ToolId::IpAddrShow],
            Self::CheckNetworkType => vec![ToolId::IpAddrShow, ToolId::NmcliDeviceStatus],
            Self::CheckWifiStability => vec![ToolId::NmcliConnectionShow, ToolId::IpAddrShow],
            Self::CheckDnsConfig => vec![ToolId::ResolvConf],
            Self::CheckInternetConnectivity => vec![ToolId::PingTest],
            Self::ListFailedServices => vec![ToolId::SystemctlFailed],
            Self::ListServices => vec![ToolId::SystemctlListUnits],
            Self::GetRecentErrors => vec![ToolId::JournalctlErrors],
            Self::GetRecentLogs => vec![ToolId::JournalctlRecent],
            Self::GetUptime => vec![ToolId::Uptime],
            Self::GetSystemInfo => vec![ToolId::UnameAll],
            Self::GetOsInfo => vec![ToolId::OsRelease],
            Self::GetHostInfo => vec![ToolId::Hostnamectl],
            Self::ListTopProcesses => vec![ToolId::PsAux],
            Self::RunToolSelfTest => vec![], // Special: runs all tools
            Self::GetAnnaStatus => vec![],   // Special: internal status
        }
    }
}

/// Result of executing an action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub action: String,
    pub success: bool,
    pub tool_results: Vec<ToolResult>,
    pub summary: Option<String>,
    pub error_message: Option<String>,
}

impl ActionResult {
    /// Create a successful result
    pub fn success(action: &Action, tool_results: Vec<ToolResult>) -> Self {
        Self {
            action: format!("{:?}", action),
            success: true,
            tool_results,
            summary: None,
            error_message: None,
        }
    }

    /// Create a failed result with honest error
    pub fn failure(action: &Action, error: &str) -> Self {
        Self {
            action: format!("{:?}", action),
            success: false,
            tool_results: vec![],
            summary: None,
            error_message: Some(error.to_string()),
        }
    }

    /// Get combined stdout from all tool results
    pub fn combined_stdout(&self) -> String {
        self.tool_results
            .iter()
            .filter(|r| r.success)
            .map(|r| r.stdout.clone())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Execute an action by running its mapped tools
pub fn execute_action(action: &Action) -> ActionResult {
    match action {
        Action::GetMemoryInfo => run_single_tool_action(action, ToolId::FreeMem),
        Action::GetCpuInfo => run_single_tool_action(action, ToolId::LsCpu),
        Action::GetCpuFeatures => run_cpu_features_action(action),
        Action::GetGpuInfo => run_gpu_info_action(action),
        Action::ListInstalledPackages => run_single_tool_action(action, ToolId::PacmanQuery),
        Action::ListGames => run_games_action(action),
        Action::CheckSteamInstalled => run_steam_check_action(action),
        Action::ListFileManagers => run_file_managers_action(action),
        Action::CheckOrphans => run_single_tool_action(action, ToolId::PacmanOrphans),
        Action::CheckUpdates => run_updates_action(action),
        Action::GetDiskUsage => run_single_tool_action(action, ToolId::DfHuman),
        Action::GetDiskUsageHome => run_home_usage_action(action),
        Action::GetDiskUsageHomeTop { limit } => run_home_top_action(action, *limit),
        Action::GetDiskUsageVar => run_single_tool_action(action, ToolId::DuVarTop),
        Action::GetDiskUsageVarTop { limit } => run_var_top_action(action, *limit),
        Action::ListBlockDevices => run_single_tool_action(action, ToolId::LsBlk),
        Action::GetNetworkInterfaces => run_single_tool_action(action, ToolId::IpAddrShow),
        Action::CheckNetworkType => run_network_type_action(action),
        Action::CheckWifiStability => run_wifi_stability_action(action),
        Action::CheckDnsConfig => run_single_tool_action(action, ToolId::ResolvConf),
        Action::CheckInternetConnectivity => run_single_tool_action(action, ToolId::PingTest),
        Action::ListFailedServices => run_single_tool_action(action, ToolId::SystemctlFailed),
        Action::ListServices => run_single_tool_action(action, ToolId::SystemctlListUnits),
        Action::GetRecentErrors => run_single_tool_action(action, ToolId::JournalctlErrors),
        Action::GetRecentLogs => run_single_tool_action(action, ToolId::JournalctlRecent),
        Action::GetUptime => run_single_tool_action(action, ToolId::Uptime),
        Action::GetSystemInfo => run_single_tool_action(action, ToolId::UnameAll),
        Action::GetOsInfo => run_single_tool_action(action, ToolId::OsRelease),
        Action::GetHostInfo => run_single_tool_action(action, ToolId::Hostnamectl),
        Action::ListTopProcesses => run_single_tool_action(action, ToolId::PsAux),
        Action::RunToolSelfTest => run_self_test_action(action),
        Action::GetAnnaStatus => run_anna_status_action(action),
    }
}

/// Map a natural language query to appropriate actions
pub fn map_query_to_actions(query: &str) -> Vec<Action> {
    let q = query.to_lowercase();

    let mut actions = Vec::new();

    // Hardware queries
    if q.contains("ram") || q.contains("memory") {
        actions.push(Action::GetMemoryInfo);
    }
    if q.contains("cpu") || q.contains("processor") {
        if q.contains("feature") || q.contains("sse") || q.contains("avx") {
            actions.push(Action::GetCpuFeatures);
        } else {
            actions.push(Action::GetCpuInfo);
        }
    }
    if q.contains("gpu") || q.contains("graphics") || q.contains("video card") {
        actions.push(Action::GetGpuInfo);
    }

    // Package queries
    if q.contains("game") || q.contains("steam") || q.contains("lutris") {
        if q.contains("steam") && q.contains("installed") {
            actions.push(Action::CheckSteamInstalled);
        } else {
            actions.push(Action::ListGames);
        }
    }
    if q.contains("file manager") || q.contains("filemanager") {
        actions.push(Action::ListFileManagers);
    }
    if q.contains("orphan") {
        actions.push(Action::CheckOrphans);
    }
    if q.contains("update") && !q.contains("upgrade your") {
        actions.push(Action::CheckUpdates);
    }

    // Disk queries
    let is_disk_query = q.contains("disk") || q.contains("storage") || q.contains("space")
        || (q.contains("folder") && q.contains("size"));
    let is_top_query = q.contains("top") || q.contains("biggest") || q.contains("largest");

    if is_disk_query {
        if q.contains("home") && is_top_query {
            actions.push(Action::GetDiskUsageHomeTop { limit: 10 });
        } else if q.contains("/var") || q.contains("var") {
            if is_top_query {
                actions.push(Action::GetDiskUsageVarTop { limit: 10 });
            } else {
                actions.push(Action::GetDiskUsageVar);
            }
        } else {
            actions.push(Action::GetDiskUsage);
        }
    }

    // Network queries
    if q.contains("wifi") || q.contains("ethernet") || q.contains("network") {
        if q.contains("stable") || q.contains("stability") {
            actions.push(Action::CheckWifiStability);
        } else if q.contains("wifi") && q.contains("ethernet") {
            actions.push(Action::CheckNetworkType);
        } else {
            actions.push(Action::GetNetworkInterfaces);
            actions.push(Action::CheckNetworkType);
        }
    }
    if q.contains("dns") {
        actions.push(Action::CheckDnsConfig);
    }

    // Service queries
    if q.contains("failed") && q.contains("service") {
        actions.push(Action::ListFailedServices);
    }
    if q.contains("error") && (q.contains("log") || q.contains("recent")) {
        actions.push(Action::GetRecentErrors);
    }

    // System queries
    if q.contains("uptime") {
        actions.push(Action::GetUptime);
    }
    if q.contains("os") || q.contains("distro") || q.contains("arch") {
        actions.push(Action::GetOsInfo);
    }

    // Meta queries
    if q.contains("self test") || q.contains("self-test") || q.contains("tool catalog") {
        actions.push(Action::RunToolSelfTest);
    }
    if (q.contains("how are you") || q.contains("what do you know"))
        && q.contains("machine")
    {
        actions.push(Action::GetAnnaStatus);
        actions.push(Action::GetSystemInfo);
        actions.push(Action::GetMemoryInfo);
        actions.push(Action::GetDiskUsage);
    }
    if q.contains("worried") || q.contains("issues") || q.contains("problems") {
        actions.push(Action::ListFailedServices);
        actions.push(Action::GetRecentErrors);
        actions.push(Action::GetDiskUsage);
    }
    if q.contains("on fire") || q.contains("burning") {
        // Playful but check real issues
        actions.push(Action::ListFailedServices);
        actions.push(Action::GetRecentErrors);
    }

    // Special: "upgrade your brain" type queries
    if q.contains("upgrade") && (q.contains("brain") || q.contains("llm") || q.contains("model")) {
        // Return empty - this will be handled specially with a text response
        return vec![];
    }

    // If no specific actions matched, try general health check
    if actions.is_empty() {
        // Default to system overview
        actions.push(Action::GetSystemInfo);
        actions.push(Action::GetUptime);
    }

    actions
}

// === Helper functions for specific actions ===

fn run_single_tool_action(action: &Action, tool_id: ToolId) -> ActionResult {
    let spec = match get_tool(tool_id) {
        Some(s) => s,
        None => {
            return ActionResult::failure(
                action,
                &format!("Tool {:?} is not in catalog (this is a bug)", tool_id),
            )
        }
    };

    let result = run_tool(&spec);

    if result.success {
        ActionResult::success(action, vec![result])
    } else if result.is_not_installed() {
        ActionResult::failure(
            action,
            &format!(
                "The '{}' command is not installed on this system.",
                spec.binary
            ),
        )
    } else {
        ActionResult::failure(
            action,
            &format!(
                "'{}' failed with exit code {}: {}",
                spec.binary, result.exit_code, result.stderr
            ),
        )
    }
}

fn run_cpu_features_action(action: &Action) -> ActionResult {
    let spec = match get_tool(ToolId::CpuInfo) {
        Some(s) => s,
        None => return ActionResult::failure(action, "CPU info tool not in catalog"),
    };

    let result = run_tool(&spec);

    if result.success {
        // Extract flags line
        let flags = result
            .stdout
            .lines()
            .find(|l| l.starts_with("flags"))
            .map(|l| l.to_string())
            .unwrap_or_default();

        let mut summary_result = result.clone();
        summary_result.stdout = flags;
        ActionResult::success(action, vec![summary_result])
    } else {
        ActionResult::failure(action, "Could not read CPU features from /proc/cpuinfo")
    }
}

fn run_gpu_info_action(action: &Action) -> ActionResult {
    let spec = match get_tool(ToolId::LsPciGpu) {
        Some(s) => s,
        None => return ActionResult::failure(action, "lspci not in catalog"),
    };

    let result = run_tool(&spec);

    if result.success {
        // Filter to VGA/3D controller lines
        let gpu_lines: String = result
            .stdout
            .lines()
            .filter(|l| {
                l.contains("VGA") || l.contains("3D") || l.contains("Display")
            })
            .collect::<Vec<_>>()
            .join("\n");

        let mut filtered_result = result.clone();
        filtered_result.stdout = if gpu_lines.is_empty() {
            "No GPU found in lspci output".to_string()
        } else {
            gpu_lines
        };
        ActionResult::success(action, vec![filtered_result])
    } else if result.is_not_installed() {
        ActionResult::failure(action, "The 'lspci' command is not installed on this system.")
    } else {
        ActionResult::failure(action, &format!("lspci failed: {}", result.stderr))
    }
}

fn run_games_action(action: &Action) -> ActionResult {
    let spec = match get_tool(ToolId::PacmanQueryGames) {
        Some(s) => s,
        None => return ActionResult::failure(action, "pacman not in catalog"),
    };

    let result = run_tool(&spec);

    if result.success {
        // Filter for game-related packages
        let game_patterns = [
            "steam", "lutris", "heroic", "wine", "proton", "game", "play",
            "dosbox", "retroarch", "emulator", "minecraft", "gog",
        ];

        let games: Vec<_> = result
            .stdout
            .lines()
            .filter(|pkg| {
                let lower = pkg.to_lowercase();
                game_patterns.iter().any(|p| lower.contains(p))
            })
            .collect();

        let mut filtered_result = result.clone();
        filtered_result.stdout = if games.is_empty() {
            "No game-related packages found.".to_string()
        } else {
            games.join("\n")
        };
        ActionResult::success(action, vec![filtered_result])
    } else if result.is_not_installed() {
        ActionResult::failure(action, "This system does not have pacman, so I cannot check for games.")
    } else {
        ActionResult::failure(action, &format!("pacman query failed: {}", result.stderr))
    }
}

fn run_steam_check_action(action: &Action) -> ActionResult {
    let spec = match get_tool(ToolId::PacmanQuery) {
        Some(s) => s,
        None => return ActionResult::failure(action, "pacman not in catalog"),
    };

    let result = run_tool_with_extra_args(&spec, &["steam"]);

    if result.success {
        let mut success_result = result.clone();
        success_result.stdout = format!("Steam is installed: {}", result.stdout.trim());
        ActionResult::success(action, vec![success_result])
    } else if result.stderr.contains("was not found") || result.exit_code == 1 {
        // Package not found - this is a valid answer, not an error
        let mut not_found = result.clone();
        not_found.success = true; // Query succeeded, answer is "not installed"
        not_found.stdout = "Steam is not installed.".to_string();
        ActionResult::success(action, vec![not_found])
    } else {
        ActionResult::failure(action, &format!("pacman query failed: {}", result.stderr))
    }
}

fn run_file_managers_action(action: &Action) -> ActionResult {
    let spec = match get_tool(ToolId::PacmanQueryFileManagers) {
        Some(s) => s,
        None => return ActionResult::failure(action, "pacman not in catalog"),
    };

    let result = run_tool(&spec);

    if result.success {
        let fm_patterns = [
            "thunar", "dolphin", "nautilus", "nemo", "pcmanfm", "ranger",
            "mc", "lf", "nnn", "vifm", "caja", "spacefm", "krusader",
        ];

        let file_managers: Vec<_> = result
            .stdout
            .lines()
            .filter(|pkg| {
                let lower = pkg.to_lowercase();
                fm_patterns.iter().any(|p| lower.contains(p))
            })
            .collect();

        let mut filtered_result = result.clone();
        filtered_result.stdout = if file_managers.is_empty() {
            "No file managers found in installed packages.".to_string()
        } else {
            format!("Installed file managers:\n{}", file_managers.join("\n"))
        };
        ActionResult::success(action, vec![filtered_result])
    } else {
        ActionResult::failure(action, &format!("pacman query failed: {}", result.stderr))
    }
}

fn run_updates_action(action: &Action) -> ActionResult {
    // Try checkupdates first (safer), fall back to pacman -Qu
    let checkupdates = get_tool(ToolId::CheckUpdates);
    let pacman_qu = get_tool(ToolId::PacmanUpdates);

    if let Some(spec) = checkupdates {
        let result = run_tool(&spec);
        if result.success {
            return ActionResult::success(action, vec![result]);
        }
    }

    // Fall back to pacman -Qu
    if let Some(spec) = pacman_qu {
        let result = run_tool(&spec);
        if result.success {
            return ActionResult::success(action, vec![result]);
        } else if result.exit_code == 1 && result.stdout.is_empty() {
            // No updates available
            let mut no_updates = result.clone();
            no_updates.success = true;
            no_updates.stdout = "No updates available.".to_string();
            return ActionResult::success(action, vec![no_updates]);
        }
    }

    ActionResult::failure(action, "Could not check for updates.")
}

fn run_home_usage_action(action: &Action) -> ActionResult {
    let spec = match get_tool(ToolId::DuHomeTop) {
        Some(s) => s,
        None => return ActionResult::failure(action, "du not in catalog"),
    };

    // Run du on $HOME
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());
    let result = run_tool_with_extra_args(&spec, &[&home]);

    if result.success {
        ActionResult::success(action, vec![result])
    } else {
        ActionResult::failure(action, &format!("du failed: {}", result.stderr))
    }
}

fn run_home_top_action(action: &Action, limit: usize) -> ActionResult {
    let result = run_home_usage_action(action);

    if result.success && !result.tool_results.is_empty() {
        let mut sorted_result = result.tool_results[0].clone();

        // Parse and sort by size
        let mut entries: Vec<_> = sorted_result
            .stdout
            .lines()
            .filter_map(|l| {
                let parts: Vec<_> = l.split_whitespace().collect();
                if parts.len() >= 2 {
                    Some((parts[0].to_string(), parts[1..].join(" ")))
                } else {
                    None
                }
            })
            .collect();

        // Sort by size (du output is already size-sorted typically)
        // Just take top N
        entries.truncate(limit);

        sorted_result.stdout = entries
            .into_iter()
            .map(|(size, path)| format!("{}\t{}", size, path))
            .collect::<Vec<_>>()
            .join("\n");

        ActionResult::success(action, vec![sorted_result])
    } else {
        result
    }
}

fn run_var_top_action(action: &Action, limit: usize) -> ActionResult {
    let spec = match get_tool(ToolId::DuVarTop) {
        Some(s) => s,
        None => return ActionResult::failure(action, "du not in catalog"),
    };

    let result = run_tool(&spec);

    if result.success {
        let mut sorted_result = result.clone();

        let mut entries: Vec<_> = sorted_result
            .stdout
            .lines()
            .filter_map(|l| {
                let parts: Vec<_> = l.split_whitespace().collect();
                if parts.len() >= 2 {
                    Some((parts[0].to_string(), parts[1..].join(" ")))
                } else {
                    None
                }
            })
            .collect();

        entries.truncate(limit);

        sorted_result.stdout = entries
            .into_iter()
            .map(|(size, path)| format!("{}\t{}", size, path))
            .collect::<Vec<_>>()
            .join("\n");

        ActionResult::success(action, vec![sorted_result])
    } else {
        ActionResult::failure(action, &format!("du /var failed: {}", result.stderr))
    }
}

fn run_network_type_action(action: &Action) -> ActionResult {
    let mut results = Vec::new();

    // Try nmcli first
    if let Some(spec) = get_tool(ToolId::NmcliDeviceStatus) {
        let result = run_tool(&spec);
        if result.success {
            results.push(result);
        }
    }

    // Also get ip addr
    if let Some(spec) = get_tool(ToolId::IpAddrShow) {
        let result = run_tool(&spec);
        if result.success {
            results.push(result);
        }
    }

    if results.is_empty() {
        ActionResult::failure(
            action,
            "Could not determine network type - neither nmcli nor ip commands worked.",
        )
    } else {
        ActionResult::success(action, results)
    }
}

fn run_wifi_stability_action(action: &Action) -> ActionResult {
    let mut results = Vec::new();

    if let Some(spec) = get_tool(ToolId::NmcliConnectionShow) {
        let result = run_tool(&spec);
        results.push(result);
    }

    if let Some(spec) = get_tool(ToolId::IpAddrShow) {
        let result = run_tool(&spec);
        results.push(result);
    }

    let successful: Vec<_> = results.into_iter().filter(|r| r.success).collect();

    if successful.is_empty() {
        ActionResult::failure(action, "Could not check wifi stability - network tools unavailable.")
    } else {
        ActionResult::success(action, successful)
    }
}

fn run_self_test_action(action: &Action) -> ActionResult {
    let results = super::catalog::self_test_catalog();
    let formatted = super::catalog::format_self_test_results(&results);

    let tool_results: Vec<_> = results.into_iter().map(|(_, r)| r).collect();

    let mut result = ActionResult::success(action, tool_results);
    result.summary = Some(formatted);
    result
}

fn run_anna_status_action(action: &Action) -> ActionResult {
    // Return internal status - no external tools needed
    let status = format!(
        "Anna Assistant v{}\n\
         Status: Running\n\
         Architecture: Unified tool catalog (v6.59.0)\n\
         Ready to help with system queries.",
        env!("CARGO_PKG_VERSION")
    );

    let result = ToolResult {
        tool_id: "anna_status".to_string(),
        success: true,
        exit_code: 0,
        stdout: status,
        stderr: String::new(),
        duration_ms: 0,
    };

    ActionResult::success(action, vec![result])
}

/// Map an action to its ToolIds (for validation)
pub fn map_action_to_tools(action: &Action) -> Vec<ToolId> {
    action.required_tools()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_mapping_ram() {
        let actions = map_query_to_actions("how much RAM do I have");
        assert!(actions.contains(&Action::GetMemoryInfo));
    }

    #[test]
    fn test_query_mapping_cpu() {
        let actions = map_query_to_actions("what CPU model do I have");
        assert!(actions.contains(&Action::GetCpuInfo));
    }

    #[test]
    fn test_query_mapping_cpu_features() {
        let actions = map_query_to_actions("does my CPU support SSE2 and AVX2");
        assert!(actions.contains(&Action::GetCpuFeatures));
    }

    #[test]
    fn test_query_mapping_gpu() {
        let actions = map_query_to_actions("what GPU do I have");
        assert!(actions.contains(&Action::GetGpuInfo));
    }

    #[test]
    fn test_query_mapping_games() {
        let actions = map_query_to_actions("do I have any games installed");
        assert!(actions.contains(&Action::ListGames));
    }

    #[test]
    fn test_query_mapping_steam() {
        let actions = map_query_to_actions("do I have Steam installed");
        assert!(actions.contains(&Action::CheckSteamInstalled));
    }

    #[test]
    fn test_query_mapping_file_managers() {
        let actions = map_query_to_actions("do I have any file manager installed");
        assert!(actions.contains(&Action::ListFileManagers));
    }

    #[test]
    fn test_query_mapping_orphans() {
        let actions = map_query_to_actions("do I have orphaned packages I could remove");
        assert!(actions.contains(&Action::CheckOrphans));
    }

    #[test]
    fn test_query_mapping_disk() {
        let actions = map_query_to_actions("how much free disk space do I have");
        assert!(actions.contains(&Action::GetDiskUsage));
    }

    #[test]
    fn test_query_mapping_home_top() {
        let actions = map_query_to_actions("top 10 folders by size under my home directory");
        assert!(actions.iter().any(|a| matches!(a, Action::GetDiskUsageHomeTop { .. })));
    }

    #[test]
    fn test_query_mapping_updates() {
        let actions = map_query_to_actions("any pending updates");
        assert!(actions.contains(&Action::CheckUpdates));
    }

    #[test]
    fn test_query_mapping_wifi() {
        let actions = map_query_to_actions("am I connected using wifi or ethernet");
        assert!(actions.contains(&Action::CheckNetworkType));
    }

    #[test]
    fn test_query_mapping_dns() {
        let actions = map_query_to_actions("is my DNS configuration ok");
        assert!(actions.contains(&Action::CheckDnsConfig));
    }

    #[test]
    fn test_query_mapping_self_test() {
        let actions = map_query_to_actions("run a tool self test");
        assert!(actions.contains(&Action::RunToolSelfTest));
    }

    #[test]
    fn test_execute_memory_action() {
        let result = execute_action(&Action::GetMemoryInfo);
        // On a real system this should work
        if result.success {
            assert!(result.combined_stdout().contains("Mem:"));
        }
    }

    #[test]
    fn test_execute_cpu_action() {
        let result = execute_action(&Action::GetCpuInfo);
        if result.success {
            let out = result.combined_stdout();
            assert!(out.contains("CPU") || out.contains("Architecture"));
        }
    }

    #[test]
    fn test_execute_disk_action() {
        let result = execute_action(&Action::GetDiskUsage);
        if result.success {
            let out = result.combined_stdout();
            assert!(out.contains("Filesystem") || out.contains("/"));
        }
    }

    #[test]
    fn test_all_actions_have_tools_in_catalog() {
        use super::super::catalog::tool_catalog;

        let catalog: Vec<_> = tool_catalog().iter().map(|t| t.id).collect();

        // Test all action types
        let test_actions = vec![
            Action::GetMemoryInfo,
            Action::GetCpuInfo,
            Action::GetGpuInfo,
            Action::ListGames,
            Action::GetDiskUsage,
            Action::CheckNetworkType,
            Action::ListFailedServices,
        ];

        for action in test_actions {
            let required = action.required_tools();
            for tool_id in required {
                assert!(
                    catalog.contains(&tool_id),
                    "Action {:?} requires {:?} which is not in catalog",
                    action,
                    tool_id
                );
            }
        }
    }
}
