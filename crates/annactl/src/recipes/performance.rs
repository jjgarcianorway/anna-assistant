// Beta.157: Performance Tuning Recipe
// Handles system performance optimization (CPU governor, I/O scheduler, swappiness)

use anna_common::action_plan_v3::{ActionPlan, CommandStep, NecessaryCheck, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct PerformanceRecipe;

#[derive(Debug, PartialEq)]
enum PerformanceOperation {
    Install,          // Install performance tuning tools (cpupower)
    CheckStatus,      // Check current performance settings
    SetCpuGovernor,   // Set CPU frequency governor
    TuneSwappiness,   // Configure swappiness value
}

impl PerformanceOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();

        // Check status (highest priority)
        if input_lower.contains("check")
            || input_lower.contains("status")
            || input_lower.contains("show")
            || input_lower.contains("current")
            || input_lower.contains("what is")
            || input_lower.contains("display")
        {
            return PerformanceOperation::CheckStatus;
        }

        // Swappiness tuning
        if input_lower.contains("swappiness")
            || input_lower.contains("swap")
            || (input_lower.contains("memory") && input_lower.contains("tune"))
        {
            return PerformanceOperation::TuneSwappiness;
        }

        // CPU governor
        if input_lower.contains("governor")
            || input_lower.contains("cpu freq")
            || input_lower.contains("performance mode")
            || input_lower.contains("power sav")
            || (input_lower.contains("cpu") && (input_lower.contains("set") || input_lower.contains("configure")))
        {
            return PerformanceOperation::SetCpuGovernor;
        }

        // Default to install
        PerformanceOperation::Install
    }
}

impl PerformanceRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();

        let has_performance_context = input_lower.contains("performance")
            || input_lower.contains("cpu governor")
            || input_lower.contains("cpupower")
            || input_lower.contains("swappiness")
            || input_lower.contains("i/o scheduler")
            || input_lower.contains("tune")
            || input_lower.contains("optimize")
            || input_lower.contains("power save")
            || input_lower.contains("power saving")
            || (input_lower.contains("cpu") && input_lower.contains("freq"))
            || (input_lower.contains("system") && input_lower.contains("tune"));

        let has_action = input_lower.contains("install")
            || input_lower.contains("setup")
            || input_lower.contains("configure")
            || input_lower.contains("set")
            || input_lower.contains("check")
            || input_lower.contains("status")
            || input_lower.contains("show")
            || input_lower.contains("change")
            || input_lower.contains("adjust")
            || input_lower.contains("tune")
            || input_lower.contains("optimize")
            || input_lower.contains("enable")
            || input_lower.contains("disable")
            || input_lower.contains("current")
            || (input_lower.contains("what is") && (input_lower.contains("my ") || input_lower.contains("the current")));

        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")
            || input_lower.starts_with("explain"))
            && !input_lower.contains("what is my")
            && !input_lower.contains("what is the current");

        has_performance_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry
            .get("user_request")
            .map(|s| s.as_str())
            .unwrap_or("");

        let operation = PerformanceOperation::detect(user_input);

        match operation {
            PerformanceOperation::Install => Self::build_install_plan(telemetry),
            PerformanceOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            PerformanceOperation::SetCpuGovernor => Self::build_set_cpu_governor_plan(telemetry),
            PerformanceOperation::TuneSwappiness => Self::build_tune_swappiness_plan(telemetry),
        }
    }

    fn build_install_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-pacman".to_string(),
                description: "Verify pacman is available".to_string(),
                command: "which pacman".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "check-cpupower".to_string(),
                description: "Check if cpupower is already installed".to_string(),
                command: "pacman -Q cpupower 2>/dev/null || true".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "install-cpupower".to_string(),
                description: "Install cpupower (CPU frequency scaling tool)".to_string(),
                command: "sudo pacman -S --needed --noconfirm cpupower".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: Some("remove-cpupower".to_string()),
                requires_confirmation: false,
            },
            CommandStep {
                id: "verify-installation".to_string(),
                description: "Verify cpupower is installed".to_string(),
                command: "cpupower --version".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![
            RollbackStep {
                id: "remove-cpupower".to_string(),
                description: "Remove cpupower".to_string(),
                command: "sudo pacman -Rns --noconfirm cpupower".to_string(),
            },
        ];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("performance.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("tools".to_string(), serde_json::json!("cpupower"));

        Ok(ActionPlan {
            analysis: "Installing cpupower for CPU frequency scaling and performance tuning.".to_string(),
            goals: vec![
                "Install cpupower package".to_string(),
                "Enable CPU governor management".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user: "cpupower installed successfully.\n\nUsage:\n- Check current governor: cpupower frequency-info\n- Set governor: sudo cpupower frequency-set -g <governor>\n\nAvailable governors: performance, powersave, ondemand, conservative, schedutil".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("performance_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-cpupower-installed".to_string(),
                description: "Check if cpupower is installed".to_string(),
                command: "which cpupower".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "show-cpu-governor".to_string(),
                description: "Show current CPU governor".to_string(),
                command: "cat /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor 2>/dev/null | head -1 || echo 'CPU governor info not available (might need cpupower installed)'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "show-swappiness".to_string(),
                description: "Show current swappiness value".to_string(),
                command: "cat /proc/sys/vm/swappiness".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "show-io-scheduler".to_string(),
                description: "Show I/O scheduler for primary disk".to_string(),
                command: "cat /sys/block/$(lsblk -ndo NAME | head -1)/queue/scheduler 2>/dev/null || echo 'I/O scheduler info not available'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("performance.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking current performance tuning settings for CPU governor, swappiness, and I/O scheduler.".to_string(),
            goals: vec![
                "Show current CPU frequency governor".to_string(),
                "Show current swappiness value".to_string(),
                "Show I/O scheduler configuration".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user: "Performance Settings:\n\nCPU Governor: Controls CPU frequency scaling\n- performance: Max frequency (high power usage)\n- powersave: Min frequency (low power usage)\n- ondemand/schedutil: Dynamic scaling (recommended)\n\nSwappiness: Controls swap usage (0-100)\n- Low (10): Prefer RAM, avoid swap\n- High (60+): Use swap more aggressively\n- Default: 60\n\nI/O Scheduler: Controls disk I/O prioritization\n- mq-deadline: Good for SSDs\n- bfq: Better for HDDs and interactive use\n- none: No scheduling (some NVMe drives)".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("performance_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_set_cpu_governor_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-cpupower-installed".to_string(),
                description: "Verify cpupower is installed".to_string(),
                command: "which cpupower".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "check-current-governor".to_string(),
                description: "Check current CPU governor".to_string(),
                command: "cpupower frequency-info --policy".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "show-available-governors".to_string(),
                description: "Show available CPU governors".to_string(),
                command: "cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_available_governors".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "show-governor-instructions".to_string(),
                description: "Show instructions for setting CPU governor".to_string(),
                command: "echo 'To set CPU governor, run ONE of these commands:\n\nPerformance (max speed, high power):\nsudo cpupower frequency-set -g performance\n\nPowersave (min speed, low power):\nsudo cpupower frequency-set -g powersave\n\nOndemand (dynamic, recommended for desktops):\nsudo cpupower frequency-set -g ondemand\n\nSchedutil (dynamic, recommended for modern kernels):\nsudo cpupower frequency-set -g schedutil\n\nTo make permanent, create /etc/default/cpupower:\nGOVERNOR=\"schedutil\"\n\nThen enable service:\nsudo systemctl enable cpupower.service'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("performance.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("SetCpuGovernor"));

        Ok(ActionPlan {
            analysis: "Providing guidance on setting CPU frequency governor for performance or power saving.".to_string(),
            goals: vec![
                "Show available CPU governors".to_string(),
                "Explain governor options".to_string(),
                "Provide configuration commands".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user: "CPU Governor Selection:\n\nperformance: Best for gaming, rendering, compute tasks\n- Pros: Maximum CPU speed\n- Cons: Higher power usage, more heat\n\npowersave: Best for battery life\n- Pros: Minimal power usage\n- Cons: Slower performance\n\nschedutil/ondemand: Best for general use\n- Pros: Balanced performance and power\n- Cons: None (recommended)\n\nNote: Anna cannot set the governor automatically (requires root). Run the appropriate cpupower command manually.".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("performance_set_cpu_governor".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_tune_swappiness_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-current-swappiness".to_string(),
                description: "Check current swappiness value".to_string(),
                command: "cat /proc/sys/vm/swappiness".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "show-swappiness-instructions".to_string(),
                description: "Show instructions for tuning swappiness".to_string(),
                command: "echo 'Swappiness controls how aggressively the kernel swaps memory to disk.\n\nCurrent value: '$(cat /proc/sys/vm/swappiness)'\n\nRecommended values:\n- 10: Prefer RAM, avoid swap (recommended for desktop with plenty of RAM)\n- 60: Default (balanced)\n- 100: Use swap aggressively\n\nTo set temporarily (until reboot):\nsudo sysctl vm.swappiness=10\n\nTo make permanent, add to /etc/sysctl.d/99-swappiness.conf:\nvm.swappiness=10\n\nThen apply:\nsudo sysctl --system'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("performance.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("TuneSwappiness"));

        Ok(ActionPlan {
            analysis: "Providing guidance on configuring swappiness for optimal memory management.".to_string(),
            goals: vec![
                "Show current swappiness value".to_string(),
                "Explain swappiness options".to_string(),
                "Provide configuration commands".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user: "Swappiness Tuning:\n\nLow swappiness (10):\n- Good for: Systems with plenty of RAM (8GB+)\n- Effect: Keeps more data in RAM, faster performance\n- Trade-off: May cause OOM if RAM fills up\n\nDefault swappiness (60):\n- Good for: General use, systems with moderate RAM\n- Effect: Balanced approach\n\nHigh swappiness (100):\n- Good for: Systems with limited RAM\n- Effect: Uses swap more, prevents OOM\n- Trade-off: Slower performance\n\nNote: Anna cannot modify swappiness automatically (requires root). Run the sysctl commands manually.".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None,
                    wm: None,
                    wallpaper_backends: vec![],
                    display_protocol: None,
                    other: meta_other,
                },
                template_used: Some("performance_tune_swappiness".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_performance_keywords() {
        assert!(PerformanceRecipe::matches_request("install cpupower"));
        assert!(PerformanceRecipe::matches_request("setup performance tuning"));
        assert!(PerformanceRecipe::matches_request("optimize system performance"));
        assert!(PerformanceRecipe::matches_request("tune cpu governor"));
    }

    #[test]
    fn test_matches_specific_tuning() {
        assert!(PerformanceRecipe::matches_request("set swappiness"));
        assert!(PerformanceRecipe::matches_request("configure cpu governor"));
        assert!(PerformanceRecipe::matches_request("change power saving mode"));
    }

    #[test]
    fn test_does_not_match_generic_info_queries() {
        assert!(!PerformanceRecipe::matches_request("what is performance"));
        assert!(!PerformanceRecipe::matches_request("tell me about cpu"));
        assert!(!PerformanceRecipe::matches_request("explain swappiness"));
    }

    #[test]
    fn test_matches_status_queries_with_my() {
        assert!(PerformanceRecipe::matches_request("what is my cpu governor"));
        assert!(PerformanceRecipe::matches_request("show current swappiness"));
    }

    #[test]
    fn test_operation_detection() {
        assert_eq!(
            PerformanceOperation::detect("install cpupower"),
            PerformanceOperation::Install
        );
        assert_eq!(
            PerformanceOperation::detect("check performance status"),
            PerformanceOperation::CheckStatus
        );
        assert_eq!(
            PerformanceOperation::detect("set cpu governor"),
            PerformanceOperation::SetCpuGovernor
        );
        assert_eq!(
            PerformanceOperation::detect("tune swappiness"),
            PerformanceOperation::TuneSwappiness
        );
    }

    #[test]
    fn test_install_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "install performance tools".to_string());

        let plan = PerformanceRecipe::build_install_plan(&telemetry).unwrap();

        assert!(plan.goals.iter().any(|g| g.to_lowercase().contains("cpupower")));
        assert!(!plan.command_plan.is_empty());
        assert!(!plan.rollback_plan.is_empty());
    }

    #[test]
    fn test_check_status_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "check performance settings".to_string());

        let plan = PerformanceRecipe::build_check_status_plan(&telemetry).unwrap();

        assert!(plan.goals.iter().any(|g| g.to_lowercase().contains("cpu") || g.to_lowercase().contains("swap")));
        assert!(!plan.command_plan.is_empty());
        assert!(plan.rollback_plan.is_empty());
    }

    #[test]
    fn test_set_cpu_governor_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "set cpu governor".to_string());

        let plan = PerformanceRecipe::build_set_cpu_governor_plan(&telemetry).unwrap();

        assert!(plan.goals.iter().any(|g| g.to_lowercase().contains("governor")));
        assert!(!plan.necessary_checks.is_empty());
        assert!(!plan.command_plan.is_empty());
    }

    #[test]
    fn test_tune_swappiness_plan_structure() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "tune swappiness".to_string());

        let plan = PerformanceRecipe::build_tune_swappiness_plan(&telemetry).unwrap();

        assert!(plan.goals.iter().any(|g| g.to_lowercase().contains("swappiness")));
        assert!(!plan.command_plan.is_empty());
    }
}
