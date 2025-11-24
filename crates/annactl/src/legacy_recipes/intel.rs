//! Intel GPU Driver Recipe
//!
//! Handles Intel GPU driver installation and configuration on Arch Linux.
//!
//! **Operations:**
//! - Install: Install Intel GPU drivers (Mesa, Vulkan, VA-API)
//! - CheckStatus: Verify Intel driver installation and GPU detection
//! - Enable32bit: Enable 32-bit library support for gaming
//! - InstallMediaTools: Install Intel media tools (intel-gpu-tools, intel-media-driver)
//!
//! **Risk Levels:**
//! - MEDIUM: Driver installation (Mesa, Vulkan libraries)
//! - INFO: Status checks (read-only)
//!
//! **Key Points:**
//! - Intel uses open-source drivers (i915 + Mesa) - integrated into kernel
//! - Generally "just works" on Linux
//! - Vulkan support via vulkan-intel
//! - VA-API for hardware video acceleration (crucial for modern codecs)

use anna_common::action_plan_v3::{
    ActionPlan, CommandStep, DetectionResults, NecessaryCheck, PlanMeta, RiskLevel, RollbackStep,
};
use anyhow::Result;
use std::collections::HashMap;

pub struct IntelRecipe;

#[derive(Debug, PartialEq)]
enum IntelOperation {
    Install,
    CheckStatus,
    Enable32bit,
    InstallMediaTools,
}

impl IntelRecipe {
    /// Check if user request matches Intel GPU driver management
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();

        // Exclude informational queries
        if input_lower.contains("what is")
            || input_lower.contains("tell me about")
            || input_lower.contains("explain")
            || input_lower.contains("how does")
        {
            return false;
        }

        // Intel-related keywords
        let has_intel_context = input_lower.contains("intel")
            || input_lower.contains("i915")
            || input_lower.contains("iris")
            || (input_lower.contains("gpu driver")
                && !input_lower.contains("nvidia")
                && !input_lower.contains("amd"));

        // Action keywords
        let has_action = input_lower.contains("install")
            || input_lower.contains("setup")
            || input_lower.contains("configure")
            || input_lower.contains("check")
            || input_lower.contains("status")
            || input_lower.contains("enable")
            || input_lower.contains("media");

        has_intel_context && has_action
    }

    /// Detect which Intel operation the user wants
    fn detect_operation(user_input: &str) -> IntelOperation {
        let input_lower = user_input.to_lowercase();

        if input_lower.contains("media")
            || input_lower.contains("video")
            || input_lower.contains("tools")
        {
            IntelOperation::InstallMediaTools
        } else if input_lower.contains("32") || input_lower.contains("multilib") {
            IntelOperation::Enable32bit
        } else if input_lower.contains("status") || input_lower.contains("check") {
            IntelOperation::CheckStatus
        } else {
            // Default: install drivers
            IntelOperation::Install
        }
    }

    /// Build ActionPlan based on detected operation
    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_request = telemetry
            .get("user_request")
            .map(|s| s.as_str())
            .unwrap_or("");

        let operation = Self::detect_operation(user_request);

        match operation {
            IntelOperation::Install => Self::build_install_plan(telemetry),
            IntelOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            IntelOperation::Enable32bit => Self::build_enable_32bit_plan(telemetry),
            IntelOperation::InstallMediaTools => Self::build_install_media_tools_plan(telemetry),
        }
    }

    /// Build plan for installing Intel drivers
    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let has_internet = telemetry
            .get("internet_connected")
            .map(|v| v == "true")
            .unwrap_or(false);

        let analysis = "Installing Intel open-source GPU drivers.\n\n\
            Intel uses the i915 kernel driver (built into the kernel) with Mesa for OpenGL/Vulkan. \
            This is the most mature open-source GPU stack on Linux and generally requires minimal configuration.\n\n\
            **What's installed:**\n\
            - Mesa: OpenGL and Vulkan implementation\n\
            - Vulkan-Intel: Vulkan driver (ANV) for Intel GPUs\n\
            - intel-media-driver: VA-API driver for hardware video acceleration\n\
            - intel-gmmlib: Graphics Memory Management Library";

        let goals = vec![
            "Install Mesa OpenGL/Vulkan drivers".to_string(),
            "Install Vulkan support for Intel".to_string(),
            "Install hardware video acceleration".to_string(),
            "Verify GPU detection".to_string(),
        ];

        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-intel-gpu".to_string(),
                description: "Verify Intel GPU is present".to_string(),
                command: "lspci | grep -iE '(vga|3d).*intel'".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "check-existing-drivers".to_string(),
                description: "Check if i915 kernel module is loaded".to_string(),
                command: "lsmod | grep i915".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let mut command_plan = vec![
            CommandStep {
                id: "install-mesa".to_string(),
                description: "Install Mesa OpenGL and Vulkan drivers".to_string(),
                command: "sudo pacman -S --noconfirm mesa".to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: Some("uninstall-mesa".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "install-vulkan".to_string(),
                description: "Install Vulkan driver for Intel (vulkan-intel)".to_string(),
                command: "sudo pacman -S --noconfirm vulkan-intel".to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: Some("uninstall-vulkan".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "install-media-driver".to_string(),
                description: "Install Intel media driver for video acceleration".to_string(),
                command: "sudo pacman -S --noconfirm intel-media-driver intel-gmmlib".to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: Some("uninstall-media-driver".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "verify-driver".to_string(),
                description: "Verify i915 kernel module is loaded".to_string(),
                command: "lsmod | grep i915".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-vulkan".to_string(),
                description: "Check Vulkan device detection".to_string(),
                command: "vulkaninfo --summary 2>/dev/null | head -20 || echo 'Install vulkan-tools for detailed info: sudo pacman -S vulkan-tools'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![
            RollbackStep {
                id: "uninstall-mesa".to_string(),
                description: "Uninstall Mesa drivers".to_string(),
                command: "sudo pacman -R --noconfirm mesa".to_string(),
            },
            RollbackStep {
                id: "uninstall-vulkan".to_string(),
                description: "Uninstall Vulkan driver".to_string(),
                command: "sudo pacman -R --noconfirm vulkan-intel".to_string(),
            },
            RollbackStep {
                id: "uninstall-media-driver".to_string(),
                description: "Uninstall media driver".to_string(),
                command: "sudo pacman -R --noconfirm intel-media-driver intel-gmmlib".to_string(),
            },
        ];

        let mut notes_for_user = String::from(
            "**Intel GPU Driver Installation**\n\n\
            **What's happening:**\n\
            1. Installing Mesa (OpenGL + Vulkan)\n\
            2. Installing vulkan-intel (Vulkan ICD - ANV driver)\n\
            3. Installing intel-media-driver (VA-API for hardware video)\n\
            4. Verifying driver and Vulkan detection\n\n\
            **Key points:**\n\
            - ✅ i915 kernel driver is built into Linux kernel (no installation needed)\n\
            - ✅ Open-source drivers with excellent support\n\
            - ✅ Generally \"just works\" - minimal configuration\n\
            - ✅ Great Wayland support\n\n\
            **After installation:**\n\
            - No reboot required (kernel driver already loaded)\n\
            - Check driver: `lsmod | grep i915`\n\
            - Check OpenGL: `glxinfo | grep renderer`\n\
            - Check Vulkan: `vulkaninfo --summary`\n\n\
            **Gaming (32-bit support):**\n\
            - For Steam/Wine, enable multilib repository\n\
            - Install: `lib32-mesa lib32-vulkan-intel`\n\
            - Run: `annactl \"enable 32-bit Intel drivers\"`\n\n\
            **Hardware video acceleration:**\n\
            - intel-media-driver provides VA-API support\n\
            - Critical for 4K/AV1 video playback\n\
            - Test with: `vainfo` (install libva-utils)\n\n\
            **Old Intel GPUs (pre-2015):**\n\
            - Broadwell and older may need legacy drivers\n\
            - Consider: `xf86-video-intel` (usually not needed)",
        );

        if !has_internet {
            notes_for_user.push_str(
                "\n\n⚠️ **WARNING**: No internet connection detected. \
                Package installation requires internet access.",
            );
            command_plan.insert(
                0,
                CommandStep {
                    id: "check-internet".to_string(),
                    description: "Verify internet connectivity".to_string(),
                    command: "ping -c 1 archlinux.org".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            );
        }

        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("intel.rs".to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("intel_install".to_string()),
            llm_version: "deterministic_recipe_v1".to_string(),
        };

        Ok(ActionPlan {
            analysis: analysis.to_string(),
            goals,
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            meta,
        })
    }

    /// Build plan for checking Intel driver status
    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let analysis = "Checking Intel GPU driver status.\n\n\
            This will verify if the Intel drivers are installed and functioning correctly.";

        let goals = vec![
            "Check if Intel drivers are installed".to_string(),
            "Verify GPU detection".to_string(),
            "Display driver information".to_string(),
        ];

        let command_plan = vec![
            CommandStep {
                id: "check-mesa-packages".to_string(),
                description: "Check if Mesa packages are installed".to_string(),
                command: "pacman -Q mesa vulkan-intel 2>/dev/null || echo 'Intel drivers not fully installed'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-kernel-module".to_string(),
                description: "Check if i915 kernel module is loaded".to_string(),
                command: "lsmod | grep i915 || echo 'i915 kernel module not loaded'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-gpu-in-pci".to_string(),
                description: "List Intel GPUs detected by PCI".to_string(),
                command: "lspci | grep -iE '(vga|3d).*intel'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-opengl".to_string(),
                description: "Check OpenGL renderer".to_string(),
                command: "glxinfo 2>/dev/null | grep -i 'renderer\\|vendor' | head -3 || echo 'Install mesa-utils: sudo pacman -S mesa-utils'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-vulkan-devices".to_string(),
                description: "List Vulkan devices".to_string(),
                command: "vulkaninfo --summary 2>/dev/null | grep -A 5 'GPU' || echo 'Install vulkan-tools: sudo pacman -S vulkan-tools'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-va-api".to_string(),
                description: "Check VA-API video acceleration support".to_string(),
                command: "vainfo 2>/dev/null | head -15 || echo 'Install libva-utils: sudo pacman -S libva-utils'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let notes_for_user = "**Intel Driver Status Check**\n\n\
            This performs read-only checks to verify your Intel driver installation.\n\n\
            **What's checked:**\n\
            1. Mesa and Vulkan packages\n\
            2. i915 kernel module\n\
            3. PCI bus GPU detection\n\
            4. OpenGL renderer (via glxinfo)\n\
            5. Vulkan devices (via vulkaninfo)\n\
            6. VA-API video acceleration (via vainfo)\n\n\
            **If drivers are not working:**\n\
            - Verify GPU is detected: `lspci | grep -i intel`\n\
            - Check kernel module: `lsmod | grep i915`\n\
            - Check kernel messages: `dmesg | grep i915`\n\
            - For old GPUs, you may need: `xf86-video-intel`"
            .to_string();

        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("intel.rs".to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("intel_check_status".to_string()),
            llm_version: "deterministic_recipe_v1".to_string(),
        };

        Ok(ActionPlan {
            analysis: analysis.to_string(),
            goals,
            necessary_checks: vec![],
            command_plan,
            rollback_plan: vec![],
            notes_for_user,
            meta,
        })
    }

    /// Build plan for enabling 32-bit driver support (for gaming)
    fn build_enable_32bit_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let has_internet = telemetry
            .get("internet_connected")
            .map(|v| v == "true")
            .unwrap_or(false);

        let analysis = "Enabling 32-bit Intel driver support for gaming.\n\n\
            Many games (especially via Steam/Wine) require 32-bit OpenGL and Vulkan libraries.\n\n\
            **Prerequisites**: multilib repository must be enabled in /etc/pacman.conf";

        let goals = vec![
            "Install 32-bit Mesa libraries".to_string(),
            "Install 32-bit Vulkan driver".to_string(),
            "Verify 32-bit support".to_string(),
        ];

        let necessary_checks = vec![NecessaryCheck {
            id: "check-multilib-enabled".to_string(),
            description: "Check if multilib repository is enabled".to_string(),
            command: "grep -E '^\\[multilib\\]' /etc/pacman.conf".to_string(),
            risk_level: RiskLevel::Info,
            required: true,
        }];

        let mut command_plan = vec![
            CommandStep {
                id: "install-lib32-mesa".to_string(),
                description: "Install 32-bit Mesa libraries".to_string(),
                command: "sudo pacman -S --noconfirm lib32-mesa".to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: Some("uninstall-lib32-mesa".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "install-lib32-vulkan".to_string(),
                description: "Install 32-bit Vulkan driver".to_string(),
                command: "sudo pacman -S --noconfirm lib32-vulkan-intel".to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: Some("uninstall-lib32-vulkan".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "verify-32bit".to_string(),
                description: "Verify 32-bit libraries are installed".to_string(),
                command: "pacman -Q lib32-mesa lib32-vulkan-intel".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![
            RollbackStep {
                id: "uninstall-lib32-mesa".to_string(),
                description: "Uninstall 32-bit Mesa".to_string(),
                command: "sudo pacman -R --noconfirm lib32-mesa".to_string(),
            },
            RollbackStep {
                id: "uninstall-lib32-vulkan".to_string(),
                description: "Uninstall 32-bit Vulkan".to_string(),
                command: "sudo pacman -R --noconfirm lib32-vulkan-intel".to_string(),
            },
        ];

        let mut notes_for_user = String::from(
            "**32-bit Intel Driver Support**\n\n\
            **What's installed:**\n\
            - lib32-mesa: 32-bit OpenGL libraries\n\
            - lib32-vulkan-intel: 32-bit Vulkan driver\n\n\
            **Why you need this:**\n\
            - Steam games (many are 32-bit)\n\
            - Wine/Proton compatibility\n\
            - Older games and applications\n\n\
            **Enabling multilib repository:**\n\
            If the check fails, edit /etc/pacman.conf and uncomment:\n\
            ```\n\
            [multilib]\n\
            Include = /etc/pacman.d/mirrorlist\n\
            ```\n\
            Then run: `sudo pacman -Sy`\n\n\
            **After installation:**\n\
            - Steam should automatically detect 32-bit libraries\n\
            - Wine/Proton games will have full GPU acceleration\n\
            - No reboot required",
        );

        if !has_internet {
            notes_for_user.push_str(
                "\n\n⚠️ **WARNING**: No internet connection detected. \
                Package installation requires internet access.",
            );
            command_plan.insert(
                0,
                CommandStep {
                    id: "check-internet".to_string(),
                    description: "Verify internet connectivity".to_string(),
                    command: "ping -c 1 archlinux.org".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            );
        }

        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("intel.rs".to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("intel_enable_32bit".to_string()),
            llm_version: "deterministic_recipe_v1".to_string(),
        };

        Ok(ActionPlan {
            analysis: analysis.to_string(),
            goals,
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            meta,
        })
    }

    /// Build plan for installing Intel media tools
    fn build_install_media_tools_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let has_internet = telemetry
            .get("internet_connected")
            .map(|v| v == "true")
            .unwrap_or(false);

        let analysis = "Installing Intel GPU media tools and utilities.\n\n\
            These tools help with GPU monitoring, debugging, and video encoding/transcoding.";

        let goals = vec![
            "Install intel-gpu-tools for monitoring".to_string(),
            "Install additional media codecs".to_string(),
            "Verify installation".to_string(),
        ];

        let mut command_plan = vec![
            CommandStep {
                id: "install-gpu-tools".to_string(),
                description: "Install Intel GPU monitoring tools".to_string(),
                command: "sudo pacman -S --noconfirm intel-gpu-tools".to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: Some("uninstall-gpu-tools".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "install-libva-utils".to_string(),
                description: "Install VA-API utilities for testing".to_string(),
                command: "sudo pacman -S --noconfirm libva-utils".to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: Some("uninstall-libva-utils".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "verify-tools".to_string(),
                description: "Verify tools are installed".to_string(),
                command: "which intel_gpu_top vainfo".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![
            RollbackStep {
                id: "uninstall-gpu-tools".to_string(),
                description: "Uninstall Intel GPU tools".to_string(),
                command: "sudo pacman -R --noconfirm intel-gpu-tools".to_string(),
            },
            RollbackStep {
                id: "uninstall-libva-utils".to_string(),
                description: "Uninstall libva-utils".to_string(),
                command: "sudo pacman -R --noconfirm libva-utils".to_string(),
            },
        ];

        let mut notes_for_user = String::from(
            "**Intel Media Tools Installation**\n\n\
            **What's installed:**\n\
            - intel-gpu-tools: GPU monitoring and debugging\n\
            - libva-utils: VA-API utilities\n\n\
            **Useful commands:**\n\
            - `intel_gpu_top`: Real-time GPU monitoring (like top for GPU)\n\
            - `vainfo`: Display VA-API capabilities\n\
            - `intel_gpu_frequency`: Check GPU frequency\n\n\
            **Testing video acceleration:**\n\
            - Run `vainfo` to see supported codecs\n\
            - Look for H.264, HEVC, VP9, AV1 decode/encode\n\
            - Modern Intel GPUs (11th gen+) support AV1 hardware decode\n\n\
            **After installation:**\n\
            - No reboot required\n\
            - Tools are ready to use immediately",
        );

        if !has_internet {
            notes_for_user.push_str(
                "\n\n⚠️ **WARNING**: No internet connection detected. \
                Package installation requires internet access.",
            );
            command_plan.insert(
                0,
                CommandStep {
                    id: "check-internet".to_string(),
                    description: "Verify internet connectivity".to_string(),
                    command: "ping -c 1 archlinux.org".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            );
        }

        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("intel.rs".to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("intel_install_media_tools".to_string()),
            llm_version: "deterministic_recipe_v1".to_string(),
        };

        Ok(ActionPlan {
            analysis: analysis.to_string(),
            goals,
            necessary_checks: vec![],
            command_plan,
            rollback_plan,
            notes_for_user,
            meta,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_intel_requests() {
        assert!(IntelRecipe::matches_request("install Intel drivers"));
        assert!(IntelRecipe::matches_request("setup Intel GPU"));
        assert!(IntelRecipe::matches_request("configure intel graphics"));
        assert!(IntelRecipe::matches_request("check Intel status"));
        assert!(IntelRecipe::matches_request("enable 32-bit Intel drivers"));
        assert!(IntelRecipe::matches_request("install Intel media tools"));

        // Should not match informational queries
        assert!(!IntelRecipe::matches_request("what is Intel"));
        assert!(!IntelRecipe::matches_request("tell me about i915"));
        assert!(!IntelRecipe::matches_request("explain Intel graphics"));

        // Should not match unrelated queries
        assert!(!IntelRecipe::matches_request("install firefox"));
        assert!(!IntelRecipe::matches_request("update system"));
    }

    #[test]
    fn test_operation_detection() {
        assert_eq!(
            IntelRecipe::detect_operation("install Intel drivers"),
            IntelOperation::Install
        );
        assert_eq!(
            IntelRecipe::detect_operation("check Intel status"),
            IntelOperation::CheckStatus
        );
        assert_eq!(
            IntelRecipe::detect_operation("enable 32-bit drivers"),
            IntelOperation::Enable32bit
        );
        assert_eq!(
            IntelRecipe::detect_operation("install media tools"),
            IntelOperation::InstallMediaTools
        );
    }

    #[test]
    fn test_install_plan() {
        let mut telemetry = HashMap::new();
        telemetry.insert(
            "user_request".to_string(),
            "install Intel drivers".to_string(),
        );
        telemetry.insert("internet_connected".to_string(), "true".to_string());

        let plan = IntelRecipe::build_plan(&telemetry).unwrap();

        assert_eq!(plan.goals.len(), 4);
        assert!(plan.analysis.contains("Intel open-source"));
        assert!(plan.notes_for_user.contains("Mesa"));

        // Should have install commands
        assert!(plan
            .command_plan
            .iter()
            .any(|cmd| cmd.id == "install-mesa"));
        assert!(plan
            .command_plan
            .iter()
            .any(|cmd| cmd.id == "install-vulkan"));

        // Should have rollback
        assert!(!plan.rollback_plan.is_empty());
    }

    #[test]
    fn test_check_status_plan() {
        let mut telemetry = HashMap::new();
        telemetry.insert(
            "user_request".to_string(),
            "check Intel status".to_string(),
        );

        let plan = IntelRecipe::build_plan(&telemetry).unwrap();

        assert!(plan.analysis.contains("Checking Intel GPU driver status"));

        // All commands should be INFO level (read-only)
        assert!(plan
            .command_plan
            .iter()
            .all(|cmd| cmd.risk_level == RiskLevel::Info));

        // Should not require confirmation
        assert!(plan
            .command_plan
            .iter()
            .all(|cmd| !cmd.requires_confirmation));
    }

    #[test]
    fn test_enable_32bit_plan() {
        let mut telemetry = HashMap::new();
        telemetry.insert(
            "user_request".to_string(),
            "enable 32-bit Intel drivers".to_string(),
        );
        telemetry.insert("internet_connected".to_string(), "true".to_string());

        let plan = IntelRecipe::build_plan(&telemetry).unwrap();

        assert!(plan.analysis.contains("32-bit"));
        assert!(plan
            .command_plan
            .iter()
            .any(|cmd| cmd.id == "install-lib32-mesa"));
        assert!(plan.notes_for_user.contains("multilib"));
    }

    #[test]
    fn test_install_media_tools_plan() {
        let mut telemetry = HashMap::new();
        telemetry.insert(
            "user_request".to_string(),
            "install Intel media tools".to_string(),
        );
        telemetry.insert("internet_connected".to_string(), "true".to_string());

        let plan = IntelRecipe::build_plan(&telemetry).unwrap();

        assert!(plan.analysis.contains("media tools"));
        assert!(plan
            .command_plan
            .iter()
            .any(|cmd| cmd.id == "install-gpu-tools"));
        assert!(plan.notes_for_user.contains("intel_gpu_top"));
    }

    #[test]
    fn test_no_internet_warning() {
        let mut telemetry = HashMap::new();
        telemetry.insert(
            "user_request".to_string(),
            "install Intel drivers".to_string(),
        );
        telemetry.insert("internet_connected".to_string(), "false".to_string());

        let plan = IntelRecipe::build_plan(&telemetry).unwrap();

        assert!(plan.notes_for_user.contains("No internet connection"));
        assert!(plan
            .command_plan
            .iter()
            .any(|cmd| cmd.id == "check-internet"));
    }
}
