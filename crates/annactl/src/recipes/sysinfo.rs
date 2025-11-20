// Beta.173: System Information Tools Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct SysinfoRecipe;

#[derive(Debug, PartialEq)]
enum SysinfoOperation {
    Install,
    CheckStatus,
    ListTools,
}

impl SysinfoOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("check") || input_lower.contains("status") {
            SysinfoOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            SysinfoOperation::ListTools
        } else {
            SysinfoOperation::Install
        }
    }
}

impl SysinfoRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("neofetch") || input_lower.contains("screenfetch")
            || input_lower.contains("fastfetch") || input_lower.contains("inxi")
            || input_lower.contains("hwinfo") || input_lower.contains("system info")
            || input_lower.contains("sysinfo");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("configure");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = SysinfoOperation::detect(user_input);
        match operation {
            SysinfoOperation::Install => Self::build_install_plan(telemetry),
            SysinfoOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            SysinfoOperation::ListTools => Self::build_list_tools_plan(telemetry),
        }
    }

    fn detect_tool(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("neofetch") { "neofetch" }
        else if input_lower.contains("fastfetch") { "fastfetch" }
        else if input_lower.contains("screenfetch") { "screenfetch" }
        else if input_lower.contains("inxi") { "inxi" }
        else if input_lower.contains("hwinfo") { "hwinfo" }
        else { "fastfetch" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let tool = Self::detect_tool(user_input);
        let (tool_name, package_name, description) = match tool {
            "neofetch" => ("Neofetch", "neofetch", "Fast system information tool with ASCII art logo display"),
            "fastfetch" => ("Fastfetch", "fastfetch", "Modern and faster alternative to neofetch with extensive customization"),
            "screenfetch" => ("Screenfetch", "screenfetch", "System information tool with distribution logo ASCII art"),
            "inxi" => ("inxi", "inxi", "Comprehensive system information script with detailed hardware info"),
            "hwinfo" => ("hwinfo", "hwinfo", "Hardware detection tool that probes system devices"),
            _ => ("Fastfetch", "fastfetch", "System information tool"),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("sysinfo.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("tool".to_string(), serde_json::json!(tool_name));

        let install_cmd = format!("sudo pacman -S --needed --noconfirm {}", package_name);
        let notes = format!("{} installed. {}. Run '{}' to display system information.",
            tool_name, description, package_name);

        Ok(ActionPlan {
            analysis: format!("Installing {} system information tool", tool_name),
            goals: vec![format!("Install {}", tool_name)],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: format!("install-{}", tool),
                    description: format!("Install {}", tool_name),
                    command: install_cmd,
                    risk_level: RiskLevel::Low,
                    rollback_id: Some(format!("remove-{}", tool)),
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: format!("remove-{}", tool),
                    description: format!("Remove {}", tool_name),
                    command: format!("sudo pacman -Rns --noconfirm {}", package_name),
                },
            ],
            notes_for_user: notes,
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("sysinfo_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("sysinfo.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking system information tools".to_string(),
            goals: vec!["List installed system info tools".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-sysinfo-tools".to_string(),
                    description: "List system info tools".to_string(),
                    command: "pacman -Q neofetch fastfetch screenfetch inxi hwinfo 2>/dev/null || echo 'No system info tools installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed system information tools".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("sysinfo_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_tools_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("sysinfo.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListTools"));

        Ok(ActionPlan {
            analysis: "Showing available system information tools".to_string(),
            goals: vec!["List available system info tools".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-tools".to_string(),
                    description: "Show available tools".to_string(),
                    command: r"echo 'System Information Tools:

ASCII Art System Info:
- Fastfetch (official) - Modern, faster neofetch alternative with extensive customization
- Neofetch (official) - Fast, customizable system info with logo art
- Screenfetch (official) - System info with distribution logo
- Archey3 (AUR) - Python system info with Arch logo
- Pfetch (AUR) - Minimal system info script
- Cpufetch (AUR) - CPU information with ASCII art

Detailed Hardware Info:
- inxi (official) - Comprehensive CLI system info with detailed hardware data
- hwinfo (official) - Hardware detection tool that probes all devices
- lshw (official) - Hardware lister with detailed device information
- dmidecode (official) - DMI table decoder for hardware specs

Specialized Tools:
- CPU-X (AUR) - CPU-Z like tool for detailed CPU/system info
- hardinfo (AUR) - GUI system profiler and benchmark tool
- i-nex (AUR) - System information tool with GUI (CPU-Z alternative)
- GPU-Viewer (AUR) - Display GPU information

Monitoring/Stats:
- htop (official) - Interactive process viewer
- glances (official) - System monitoring with web UI
- s-tui (official) - Terminal CPU stress and monitoring
- stacer (AUR) - System optimizer and monitoring

Comparison:
- Fastfetch: Best for speed and modern features, neofetch replacement
- Neofetch: Best for quick visual system overview with logo (classic choice)
- inxi: Best for detailed CLI hardware information
- hwinfo: Best for comprehensive hardware detection
- CPU-X: Best for detailed CPU specifications in GUI

Features:
- Fastfetch: Faster than neofetch, more modules, better logo support, JSON output
- Neofetch: Custom ASCII art, color schemes, screenshot mode, module configuration
- inxi: Network info, sensors, repos, processes, partitions, RAID, full system report
- hwinfo: Probe all hardware, show device tree, detailed PCI/USB info
- CPU-X: Benchmarking, cache info, motherboard details, sensors

Output Detail:
- Fastfetch: Similar to neofetch but faster, with more modules and better detection
- Neofetch: OS, kernel, packages, shell, DE/WM, theme, icons, CPU, GPU, memory, uptime
- inxi: Full system (use -F), audio, battery, CPU, disk, graphics, machine, network, sensors
- hwinfo: All hardware classes, individual device details, driver information
- CPU-X: CPU name, architecture, caches, clocks, motherboard, BIOS, memory

Use Cases:
- Fastfetch: Daily driver replacement for neofetch, faster terminal startup
- Neofetch: Screenshots, rice posts, terminal setup display (legacy standard)
- inxi: System diagnosis, hardware inventory, support requests
- hwinfo: Driver troubleshooting, hardware detection, device information
- CPU-X: CPU analysis, overclock validation, hardware specs

Usage Examples:
- fastfetch: Show system info with logo (fast)
- neofetch: Show system info with logo (classic)
- inxi -F: Full system information
- hwinfo --short: Brief hardware summary
- hwinfo --cpu --gfxcard: Specific hardware info

Configuration:
- Fastfetch: ~/.config/fastfetch/config.jsonc (JSON config with more options)
- Neofetch: ~/.config/neofetch/config.conf (extensive customization)
- inxi: Command-line options (no config file)
- hwinfo: Command-line options only
- Screenfetch: ~/.screenFetchrc (custom ASCII, colors)'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "System information tools for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("sysinfo_list_tools".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_matches() {
        assert!(SysinfoRecipe::matches_request("install neofetch"));
        assert!(SysinfoRecipe::matches_request("install fastfetch"));
        assert!(SysinfoRecipe::matches_request("install system info tool"));
        assert!(SysinfoRecipe::matches_request("setup inxi"));
        assert!(!SysinfoRecipe::matches_request("what is neofetch"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install screenfetch".to_string());
        let plan = SysinfoRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
