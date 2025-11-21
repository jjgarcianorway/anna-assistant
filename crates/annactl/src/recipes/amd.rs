//! AMD GPU Driver Recipe
//!
//! Handles AMD GPU driver installation and configuration on Arch Linux.
//!
//! **Operations:**
//! - Install: Install AMD open-source drivers (Mesa, AMDGPU, Vulkan)
//! - CheckStatus: Verify AMD driver installation and GPU detection
//! - InstallRocm: Install ROCm for GPU computing
//! - Enable32bit: Enable 32-bit library support for gaming
//!
//! **Risk Levels:**
//! - MEDIUM: Driver installation (Mesa, Vulkan libraries)
//! - HIGH: ROCm installation (large package, kernel dependencies)
//! - INFO: Status checks (read-only)
//!
//! **Key Points:**
//! - AMD uses open-source drivers (AMDGPU + Mesa) - generally more stable than NVIDIA
//! - No need to remove existing drivers (radeon for old GPUs, amdgpu for new)
//! - Vulkan support via vulkan-radeon
//! - ROCm is AMD's CUDA equivalent for GPU computing

use anna_common::action_plan_v3::{
    ActionPlan, CommandStep, DetectionResults, NecessaryCheck, PlanMeta, RiskLevel, RollbackStep,
};
use anyhow::Result;
use std::collections::HashMap;

pub struct AmdRecipe;

#[derive(Debug, PartialEq)]
enum AmdOperation {
    Install,
    CheckStatus,
    InstallRocm,
    Enable32bit,
}

impl AmdRecipe {
    /// Check if user request matches AMD GPU driver management
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

        // AMD-related keywords
        let has_amd_context = input_lower.contains("amd")
            || input_lower.contains("radeon")
            || input_lower.contains("amdgpu")
            || input_lower.contains("rocm")  // ROCm is AMD-specific
            || (input_lower.contains("gpu driver")
                && !input_lower.contains("nvidia")
                && !input_lower.contains("intel"));

        // Action keywords
        let has_action = input_lower.contains("install")
            || input_lower.contains("setup")
            || input_lower.contains("configure")
            || input_lower.contains("check")
            || input_lower.contains("status")
            || input_lower.contains("enable")
            || input_lower.contains("rocm");

        has_amd_context && has_action
    }

    /// Detect which AMD operation the user wants
    fn detect_operation(user_input: &str) -> AmdOperation {
        let input_lower = user_input.to_lowercase();

        if input_lower.contains("rocm") {
            AmdOperation::InstallRocm
        } else if input_lower.contains("32") || input_lower.contains("multilib") {
            AmdOperation::Enable32bit
        } else if input_lower.contains("status") || input_lower.contains("check") {
            AmdOperation::CheckStatus
        } else {
            // Default: install drivers
            AmdOperation::Install
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
            AmdOperation::Install => Self::build_install_plan(telemetry),
            AmdOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            AmdOperation::InstallRocm => Self::build_install_rocm_plan(telemetry),
            AmdOperation::Enable32bit => Self::build_enable_32bit_plan(telemetry),
        }
    }

    /// Build plan for installing AMD drivers
    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let has_internet = telemetry
            .get("internet_connected")
            .map(|v| v == "true")
            .unwrap_or(false);

        let analysis = "Installing AMD open-source GPU drivers.\n\n\
            AMD uses the open-source AMDGPU kernel driver with Mesa for OpenGL/Vulkan. \
            This stack is well-maintained and generally provides excellent performance.\n\n\
            **What's installed:**\n\
            - Mesa: OpenGL and Vulkan implementation\n\
            - Vulkan-Radeon: Vulkan driver for AMD GPUs\n\
            - AMDGPU firmware: GPU microcode\n\
            - VA-API: Hardware video acceleration";

        let goals = vec![
            "Install Mesa OpenGL/Vulkan drivers".to_string(),
            "Install Vulkan support for AMD".to_string(),
            "Install video acceleration libraries".to_string(),
            "Verify GPU detection".to_string(),
        ];

        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-amd-gpu".to_string(),
                description: "Verify AMD GPU is present".to_string(),
                command: "lspci | grep -iE '(vga|3d).*amd|radeon'".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "check-existing-drivers".to_string(),
                description: "Check for existing GPU drivers".to_string(),
                command: "lsmod | grep -E '(amdgpu|radeon)'".to_string(),
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
                description: "Install Vulkan driver for AMD (vulkan-radeon)".to_string(),
                command: "sudo pacman -S --noconfirm vulkan-radeon".to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: Some("uninstall-vulkan".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "install-video-accel".to_string(),
                description: "Install video acceleration libraries (libva-mesa-driver)".to_string(),
                command: "sudo pacman -S --noconfirm libva-mesa-driver mesa-vdpau".to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: Some("uninstall-video-accel".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "verify-driver".to_string(),
                description: "Verify AMDGPU kernel module is loaded".to_string(),
                command: "lsmod | grep amdgpu".to_string(),
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
                command: "sudo pacman -R --noconfirm vulkan-radeon".to_string(),
            },
            RollbackStep {
                id: "uninstall-video-accel".to_string(),
                description: "Uninstall video acceleration".to_string(),
                command: "sudo pacman -R --noconfirm libva-mesa-driver mesa-vdpau".to_string(),
            },
        ];

        let mut notes_for_user = String::from(
            "**AMD GPU Driver Installation**\n\n\
            **What's happening:**\n\
            1. Installing Mesa (OpenGL + Vulkan)\n\
            2. Installing vulkan-radeon (Vulkan ICD for AMD)\n\
            3. Installing VA-API for hardware video decoding\n\
            4. Verifying driver and Vulkan detection\n\n\
            **Key differences from NVIDIA:**\n\
            - ✅ Open-source drivers (no proprietary blob)\n\
            - ✅ Included in kernel (amdgpu module)\n\
            - ✅ No need to remove conflicting drivers\n\
            - ✅ Generally excellent Wayland support\n\n\
            **After installation:**\n\
            - Reboot recommended (not strictly required)\n\
            - Check driver: `lsmod | grep amdgpu`\n\
            - Check OpenGL: `glxinfo | grep renderer`\n\
            - Check Vulkan: `vulkaninfo --summary`\n\n\
            **Gaming (32-bit support):**\n\
            - For Steam/Wine, enable multilib repository\n\
            - Install: `lib32-mesa lib32-vulkan-radeon`\n\
            - Run: `annactl \"enable 32-bit AMD drivers\"`\n\n\
            **Older AMD GPUs (pre-2012):**\n\
            - May use 'radeon' driver instead of 'amdgpu'\n\
            - Check with: `lspci -k | grep -A 3 VGA`\n\
            - If using radeon, install: `xf86-video-ati`",
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
            serde_json::Value::String("amd.rs".to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("amd_install".to_string()),
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

    /// Build plan for checking AMD driver status
    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let analysis = "Checking AMD GPU driver status.\n\n\
            This will verify if the AMD drivers are installed and functioning correctly.";

        let goals = vec![
            "Check if AMD drivers are installed".to_string(),
            "Verify GPU detection".to_string(),
            "Display driver information".to_string(),
        ];

        let command_plan = vec![
            CommandStep {
                id: "check-mesa-packages".to_string(),
                description: "Check if Mesa packages are installed".to_string(),
                command: "pacman -Q mesa vulkan-radeon 2>/dev/null || echo 'AMD drivers not fully installed'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-kernel-module".to_string(),
                description: "Check if AMDGPU kernel module is loaded".to_string(),
                command: "lsmod | grep amdgpu || lsmod | grep radeon || echo 'No AMD kernel module loaded'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-gpu-in-pci".to_string(),
                description: "List AMD GPUs detected by PCI".to_string(),
                command: "lspci | grep -iE '(vga|3d).*amd|radeon'".to_string(),
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
        ];

        let notes_for_user = "**AMD Driver Status Check**\n\n\
            This performs read-only checks to verify your AMD driver installation.\n\n\
            **What's checked:**\n\
            1. Mesa and Vulkan packages\n\
            2. AMDGPU/Radeon kernel module\n\
            3. PCI bus GPU detection\n\
            4. OpenGL renderer (via glxinfo)\n\
            5. Vulkan devices (via vulkaninfo)\n\n\
            **If drivers are not working:**\n\
            - Verify GPU is detected: `lspci | grep -i amd`\n\
            - Check kernel module: `lsmod | grep amdgpu`\n\
            - Check kernel messages: `dmesg | grep amdgpu`\n\
            - For old GPUs (pre-2012), you may need: `xf86-video-ati`"
            .to_string();

        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("amd.rs".to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("amd_check_status".to_string()),
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

        let analysis = "Enabling 32-bit AMD driver support for gaming.\n\n\
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
                command: "sudo pacman -S --noconfirm lib32-vulkan-radeon".to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: Some("uninstall-lib32-vulkan".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "verify-32bit".to_string(),
                description: "Verify 32-bit libraries are installed".to_string(),
                command: "pacman -Q lib32-mesa lib32-vulkan-radeon".to_string(),
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
                command: "sudo pacman -R --noconfirm lib32-vulkan-radeon".to_string(),
            },
        ];

        let mut notes_for_user = String::from(
            "**32-bit AMD Driver Support**\n\n\
            **What's installed:**\n\
            - lib32-mesa: 32-bit OpenGL libraries\n\
            - lib32-vulkan-radeon: 32-bit Vulkan driver\n\n\
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
            serde_json::Value::String("amd.rs".to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("amd_enable_32bit".to_string()),
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

    /// Build plan for installing ROCm (AMD's CUDA equivalent)
    fn build_install_rocm_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let has_internet = telemetry
            .get("internet_connected")
            .map(|v| v == "true")
            .unwrap_or(false);

        let analysis = "Installing ROCm for AMD GPU computing.\n\n\
            ROCm (Radeon Open Compute) is AMD's platform for GPU computing, similar to NVIDIA CUDA.\n\n\
            **Prerequisites:**\n\
            - AMD GPU with GCN 3.0+ architecture (Fiji/Polaris or newer)\n\
            - AMDGPU drivers installed\n\n\
            **Warning**: ROCm is very large (~2-3 GB) and has many dependencies.";

        let goals = vec![
            "Install ROCm runtime".to_string(),
            "Verify ROCm installation".to_string(),
            "Check GPU compatibility".to_string(),
        ];

        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-amd-driver".to_string(),
                description: "Verify AMDGPU driver is loaded".to_string(),
                command: "lsmod | grep amdgpu".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "check-disk-space".to_string(),
                description: "Check available disk space (ROCm is ~3GB)".to_string(),
                command: "df -h / | tail -1 | awk '{print $4}'".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "install-rocm".to_string(),
                description: "Install ROCm from AUR (via yay/paru)".to_string(),
                command: "echo 'ROCm must be installed from AUR. Run: yay -S rocm-hip-sdk or paru -S rocm-hip-sdk'".to_string(),
                risk_level: RiskLevel::High,
                rollback_id: None,
                requires_confirmation: true,
            },
        ];

        let mut notes_for_user = String::from(
            "**ROCm Installation**\n\n\
            **Important**: ROCm is not in official Arch repos. You must install from AUR.\n\n\
            **Installation steps:**\n\
            1. Ensure AUR helper is installed: `annactl \"do I have yay installed\"`\n\
            2. Install ROCm: `yay -S rocm-hip-sdk rocm-opencl-sdk`\n\
            3. Add user to video group: `sudo usermod -aG video $USER`\n\
            4. Reboot\n\
            5. Verify: `rocminfo`\n\n\
            **What's included:**\n\
            - HIP: C++ GPU programming language (like CUDA)\n\
            - ROCm libraries for deep learning (MIOpen, ROCm-SMI)\n\
            - OpenCL support\n\n\
            **Supported GPUs:**\n\
            - GCN 3.0+: Fiji (R9 Fury), Polaris (RX 4xx/5xx)\n\
            - GCN 4.0+: Vega (RX Vega)\n\
            - RDNA 1/2/3: RX 5xxx, RX 6xxx, RX 7xxx\n\n\
            **After installation:**\n\
            - Check ROCm: `rocminfo`\n\
            - Check GPUs: `rocm-smi`\n\
            - Compile HIP code: `hipcc mycode.cpp -o mycode`\n\n\
            **Package size warning:**\n\
            - ROCm is ~2-3 GB\n\
            - Build from AUR takes 30-60 minutes",
        );

        if !has_internet {
            notes_for_user.push_str(
                "\n\n⚠️ **WARNING**: No internet connection detected. \
                AUR installation requires internet access.",
            );
        }

        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("amd.rs".to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("amd_install_rocm".to_string()),
            llm_version: "deterministic_recipe_v1".to_string(),
        };

        Ok(ActionPlan {
            analysis: analysis.to_string(),
            goals,
            necessary_checks,
            command_plan,
            rollback_plan: vec![],
            notes_for_user,
            meta,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_amd_requests() {
        assert!(AmdRecipe::matches_request("install AMD drivers"));
        assert!(AmdRecipe::matches_request("setup AMD GPU"));
        assert!(AmdRecipe::matches_request("configure amdgpu"));
        assert!(AmdRecipe::matches_request("check AMD status"));
        assert!(AmdRecipe::matches_request("install ROCm"));
        assert!(AmdRecipe::matches_request("enable 32-bit AMD drivers"));
        assert!(AmdRecipe::matches_request("setup radeon"));

        // Should not match informational queries
        assert!(!AmdRecipe::matches_request("what is AMD"));
        assert!(!AmdRecipe::matches_request("tell me about Radeon"));
        assert!(!AmdRecipe::matches_request("explain amdgpu"));

        // Should not match unrelated queries
        assert!(!AmdRecipe::matches_request("install firefox"));
        assert!(!AmdRecipe::matches_request("update system"));
    }

    #[test]
    fn test_operation_detection() {
        assert_eq!(
            AmdRecipe::detect_operation("install AMD drivers"),
            AmdOperation::Install
        );
        assert_eq!(
            AmdRecipe::detect_operation("check AMD status"),
            AmdOperation::CheckStatus
        );
        assert_eq!(
            AmdRecipe::detect_operation("install ROCm"),
            AmdOperation::InstallRocm
        );
        assert_eq!(
            AmdRecipe::detect_operation("enable 32-bit drivers"),
            AmdOperation::Enable32bit
        );
    }

    #[test]
    fn test_install_plan() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "install AMD drivers".to_string());
        telemetry.insert("internet_connected".to_string(), "true".to_string());

        let plan = AmdRecipe::build_plan(&telemetry).unwrap();

        assert_eq!(plan.goals.len(), 4);
        assert!(plan.analysis.contains("AMD open-source"));
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
        telemetry.insert("user_request".to_string(), "check AMD status".to_string());

        let plan = AmdRecipe::build_plan(&telemetry).unwrap();

        assert!(plan.analysis.contains("Checking AMD GPU driver status"));

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
            "enable 32-bit AMD drivers".to_string(),
        );
        telemetry.insert("internet_connected".to_string(), "true".to_string());

        let plan = AmdRecipe::build_plan(&telemetry).unwrap();

        assert!(plan.analysis.contains("32-bit"));
        assert!(plan
            .command_plan
            .iter()
            .any(|cmd| cmd.id == "install-lib32-mesa"));
        assert!(plan.notes_for_user.contains("multilib"));
    }

    #[test]
    fn test_install_rocm_plan() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "install ROCm".to_string());
        telemetry.insert("internet_connected".to_string(), "true".to_string());

        let plan = AmdRecipe::build_plan(&telemetry).unwrap();

        assert!(plan.analysis.contains("ROCm"));
        assert!(plan.notes_for_user.contains("AUR"));

        // Should have prerequisite check
        assert!(plan
            .necessary_checks
            .iter()
            .any(|check| check.id == "check-amd-driver"));
    }

    #[test]
    fn test_no_internet_warning() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "install AMD drivers".to_string());
        telemetry.insert("internet_connected".to_string(), "false".to_string());

        let plan = AmdRecipe::build_plan(&telemetry).unwrap();

        assert!(plan.notes_for_user.contains("No internet connection"));
        assert!(plan
            .command_plan
            .iter()
            .any(|cmd| cmd.id == "check-internet"));
    }
}
