//! NVIDIA GPU Driver Recipe
//!
//! Handles NVIDIA GPU driver installation and configuration on Arch Linux.
//!
//! **Operations:**
//! - Install: Install NVIDIA proprietary drivers and utilities
//! - CheckStatus: Verify NVIDIA driver installation and GPU detection
//! - ConfigureXorg: Generate Xorg configuration for NVIDIA
//! - InstallCuda: Install CUDA toolkit for GPU computing
//!
//! **Risk Levels:**
//! - HIGH: Driver installation (kernel modules, potential boot issues)
//! - MEDIUM: CUDA installation, Xorg configuration
//! - INFO: Status checks (read-only)
//!
//! **Safety Features:**
//! - Detects NVIDIA GPU before installation
//! - Warns about proprietary drivers vs open-source
//! - Includes fallback instructions for boot issues
//! - Recommends kernel parameter adjustments

use anna_common::action_plan_v3::{
    ActionPlan, CommandStep, DetectionResults, NecessaryCheck, PlanMeta, RiskLevel, RollbackStep,
};
use anyhow::Result;
use std::collections::HashMap;

pub struct NvidiaRecipe;

#[derive(Debug, PartialEq)]
enum NvidiaOperation {
    Install,
    CheckStatus,
    ConfigureXorg,
    InstallCuda,
}

impl NvidiaRecipe {
    /// Check if user request matches NVIDIA GPU driver management
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

        // NVIDIA-related keywords
        let has_nvidia_context = input_lower.contains("nvidia")
            || input_lower.contains("cuda")  // CUDA is NVIDIA-specific
            || input_lower.contains("gpu driver")
            || input_lower.contains("graphics driver")
            || (input_lower.contains("driver") && input_lower.contains("gpu"));

        // Action keywords
        let has_action = input_lower.contains("install")
            || input_lower.contains("setup")
            || input_lower.contains("configure")
            || input_lower.contains("check")
            || input_lower.contains("status")
            || input_lower.contains("enable")
            || input_lower.contains("cuda");

        has_nvidia_context && has_action
    }

    /// Detect which NVIDIA operation the user wants
    fn detect_operation(user_input: &str) -> NvidiaOperation {
        let input_lower = user_input.to_lowercase();

        if input_lower.contains("cuda") {
            NvidiaOperation::InstallCuda
        } else if input_lower.contains("xorg")
            || input_lower.contains("x11")
            || input_lower.contains("configure")
        {
            NvidiaOperation::ConfigureXorg
        } else if input_lower.contains("status") || input_lower.contains("check") {
            NvidiaOperation::CheckStatus
        } else {
            // Default: install drivers
            NvidiaOperation::Install
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
            NvidiaOperation::Install => Self::build_install_plan(telemetry),
            NvidiaOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            NvidiaOperation::ConfigureXorg => Self::build_configure_xorg_plan(telemetry),
            NvidiaOperation::InstallCuda => Self::build_install_cuda_plan(telemetry),
        }
    }

    /// Build plan for installing NVIDIA drivers
    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let has_internet = telemetry
            .get("internet_connected")
            .map(|v| v == "true")
            .unwrap_or(false);

        let analysis = "Installing NVIDIA proprietary drivers for GPU support.\n\n\
            This will install the nvidia package (or nvidia-open for newer GPUs), \
            which provides kernel modules, OpenGL/Vulkan libraries, and X11/Wayland support.\n\n\
            **Important**: After installation, you must reboot for the kernel modules to load properly.";

        let goals = vec![
            "Install NVIDIA proprietary drivers".to_string(),
            "Install NVIDIA utilities (nvidia-settings, nvidia-smi)".to_string(),
            "Load kernel modules".to_string(),
            "Verify GPU detection".to_string(),
        ];

        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-nvidia-gpu".to_string(),
                description: "Verify NVIDIA GPU is present".to_string(),
                command: "lspci | grep -i nvidia".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "check-existing-drivers".to_string(),
                description: "Check for existing GPU drivers".to_string(),
                command: "lsmod | grep -E '(nvidia|nouveau)'".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let mut command_plan = vec![
            CommandStep {
                id: "remove-nouveau".to_string(),
                description: "Remove open-source nouveau driver (conflicts with NVIDIA)".to_string(),
                command: "sudo pacman -R --noconfirm xf86-video-nouveau 2>/dev/null || true".to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: None,
                requires_confirmation: true,
            },
            CommandStep {
                id: "install-nvidia".to_string(),
                description: "Install NVIDIA proprietary drivers and utilities".to_string(),
                command: "sudo pacman -S --noconfirm nvidia nvidia-utils nvidia-settings".to_string(),
                risk_level: RiskLevel::High,
                rollback_id: Some("uninstall-nvidia".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "load-kernel-module".to_string(),
                description: "Load NVIDIA kernel module".to_string(),
                command: "sudo modprobe nvidia".to_string(),
                risk_level: RiskLevel::High,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "verify-gpu".to_string(),
                description: "Verify NVIDIA GPU is detected".to_string(),
                command: "nvidia-smi".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![
            RollbackStep {
                id: "uninstall-nvidia".to_string(),
                description: "Uninstall NVIDIA drivers".to_string(),
                command: "sudo pacman -R --noconfirm nvidia nvidia-utils nvidia-settings".to_string(),
            },
            RollbackStep {
                id: "reinstall-nouveau".to_string(),
                description: "Reinstall open-source nouveau driver".to_string(),
                command: "sudo pacman -S --noconfirm xf86-video-nouveau".to_string(),
            },
        ];

        let mut notes_for_user = String::from(
            "**NVIDIA Driver Installation**\n\n\
            **What's happening:**\n\
            1. Removing the open-source nouveau driver (conflicts with NVIDIA proprietary)\n\
            2. Installing NVIDIA proprietary drivers (nvidia package)\n\
            3. Loading the nvidia kernel module\n\
            4. Verifying GPU detection with nvidia-smi\n\n\
            **After installation:**\n\
            - **REBOOT REQUIRED**: You must reboot for the kernel modules to load properly\n\
            - Run `nvidia-smi` to verify GPU detection\n\
            - Use `nvidia-settings` for GUI configuration\n\n\
            **Driver Variants:**\n\
            - `nvidia`: Proprietary driver for most GPUs (default)\n\
            - `nvidia-open`: Open kernel modules for Turing+ GPUs (RTX 20xx and newer)\n\
            - `nvidia-lts`: For linux-lts kernel users\n\n\
            **If you have boot issues after reboot:**\n\
            1. Boot to TTY (Ctrl+Alt+F2)\n\
            2. Uninstall: `sudo pacman -R nvidia nvidia-utils`\n\
            3. Reinstall nouveau: `sudo pacman -S xf86-video-nouveau`\n\
            4. Reboot\n\n\
            **Wayland users:**\n\
            - NVIDIA + Wayland can be problematic on older drivers\n\
            - Consider adding `nvidia-drm.modeset=1` to kernel parameters\n\
            - Or use X11 instead",
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
            serde_json::Value::String("nvidia.rs".to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("nvidia_install".to_string()),
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

    /// Build plan for checking NVIDIA driver status
    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let analysis = "Checking NVIDIA GPU driver status.\n\n\
            This will verify if the NVIDIA drivers are installed and functioning correctly.";

        let goals = vec![
            "Check if NVIDIA drivers are installed".to_string(),
            "Verify GPU detection".to_string(),
            "Display driver version".to_string(),
        ];

        let command_plan = vec![
            CommandStep {
                id: "check-nvidia-packages".to_string(),
                description: "Check if NVIDIA packages are installed".to_string(),
                command: "pacman -Q nvidia nvidia-utils 2>/dev/null || echo 'NVIDIA drivers not installed'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-kernel-module".to_string(),
                description: "Check if NVIDIA kernel module is loaded".to_string(),
                command: "lsmod | grep nvidia || echo 'NVIDIA kernel module not loaded'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-gpu-detection".to_string(),
                description: "Check NVIDIA GPU detection".to_string(),
                command: "nvidia-smi 2>/dev/null || echo 'nvidia-smi not available or GPU not detected'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-gpu-in-pci".to_string(),
                description: "List NVIDIA GPUs detected by PCI".to_string(),
                command: "lspci | grep -i nvidia".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let notes_for_user = "**NVIDIA Status Check**\n\n\
            This performs read-only checks to verify your NVIDIA driver installation.\n\n\
            **What's checked:**\n\
            1. NVIDIA packages (nvidia, nvidia-utils)\n\
            2. NVIDIA kernel module (lsmod)\n\
            3. GPU detection (nvidia-smi)\n\
            4. PCI bus GPU detection (lspci)\n\n\
            **If drivers are not working:**\n\
            - Ensure you've rebooted after installation\n\
            - Check for conflicting nouveau driver: `lsmod | grep nouveau`\n\
            - Verify kernel module loads: `sudo modprobe nvidia`\n\
            - Check kernel logs: `dmesg | grep nvidia`"
            .to_string();

        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("nvidia.rs".to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("nvidia_check_status".to_string()),
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

    /// Build plan for configuring Xorg for NVIDIA
    fn build_configure_xorg_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let analysis = "Generating Xorg configuration for NVIDIA GPU.\n\n\
            This creates /etc/X11/xorg.conf.d/20-nvidia.conf to ensure X11 uses the NVIDIA driver.";

        let goals = vec![
            "Generate Xorg configuration for NVIDIA".to_string(),
            "Enable NVIDIA as primary GPU".to_string(),
        ];

        let command_plan = vec![
            CommandStep {
                id: "create-xorg-config-dir".to_string(),
                description: "Create Xorg configuration directory".to_string(),
                command: "sudo mkdir -p /etc/X11/xorg.conf.d".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "generate-xorg-config".to_string(),
                description: "Generate NVIDIA Xorg configuration".to_string(),
                command: r#"sudo bash -c 'cat > /etc/X11/xorg.conf.d/20-nvidia.conf << EOF
Section "Device"
    Identifier "NVIDIA Card"
    Driver "nvidia"
    VendorName "NVIDIA Corporation"
EndSection
EOF'"#.to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: Some("remove-xorg-config".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "verify-xorg-config".to_string(),
                description: "Verify Xorg configuration was created".to_string(),
                command: "cat /etc/X11/xorg.conf.d/20-nvidia.conf".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![RollbackStep {
            id: "remove-xorg-config".to_string(),
            description: "Remove NVIDIA Xorg configuration".to_string(),
            command: "sudo rm -f /etc/X11/xorg.conf.d/20-nvidia.conf".to_string(),
        }];

        let notes_for_user = "**Xorg Configuration for NVIDIA**\n\n\
            This creates a minimal Xorg configuration to explicitly use the NVIDIA driver.\n\n\
            **What's created:**\n\
            - `/etc/X11/xorg.conf.d/20-nvidia.conf`: Basic NVIDIA driver configuration\n\n\
            **After configuration:**\n\
            - Restart your display manager or reboot\n\
            - X11 will use the NVIDIA driver\n\n\
            **Advanced configuration:**\n\
            - Use `nvidia-settings` for GUI configuration\n\
            - Use `nvidia-xconfig` to generate full xorg.conf (not recommended)\n\
            - Consider adding options like `Option \"Coolbits\" \"4\"` for overclocking\n\n\
            **If X11 won't start:**\n\
            1. Boot to TTY (Ctrl+Alt+F2)\n\
            2. Remove config: `sudo rm /etc/X11/xorg.conf.d/20-nvidia.conf`\n\
            3. Restart display manager or reboot"
            .to_string();

        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("nvidia.rs".to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("nvidia_configure_xorg".to_string()),
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

    /// Build plan for installing CUDA toolkit
    fn build_install_cuda_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let has_internet = telemetry
            .get("internet_connected")
            .map(|v| v == "true")
            .unwrap_or(false);

        let analysis = "Installing NVIDIA CUDA toolkit for GPU computing.\n\n\
            CUDA enables GPU-accelerated applications for deep learning, scientific computing, and more.\n\n\
            **Prerequisites**: NVIDIA drivers must be installed first.";

        let goals = vec![
            "Install CUDA toolkit".to_string(),
            "Verify CUDA installation".to_string(),
            "Check CUDA version compatibility".to_string(),
        ];

        let necessary_checks = vec![NecessaryCheck {
            id: "check-nvidia-driver".to_string(),
            description: "Verify NVIDIA drivers are installed".to_string(),
            command: "nvidia-smi".to_string(),
            risk_level: RiskLevel::Info,
            required: true,
        }];

        let mut command_plan = vec![
            CommandStep {
                id: "install-cuda".to_string(),
                description: "Install CUDA toolkit from Arch repos".to_string(),
                command: "sudo pacman -S --noconfirm cuda".to_string(),
                risk_level: RiskLevel::High,
                rollback_id: Some("uninstall-cuda".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "verify-cuda".to_string(),
                description: "Verify CUDA installation".to_string(),
                command: "nvcc --version".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![RollbackStep {
            id: "uninstall-cuda".to_string(),
            description: "Uninstall CUDA toolkit".to_string(),
            command: "sudo pacman -R --noconfirm cuda".to_string(),
        }];

        let mut notes_for_user = String::from(
            "**CUDA Toolkit Installation**\n\n\
            **What's installed:**\n\
            - CUDA compiler (nvcc)\n\
            - CUDA libraries\n\
            - CUDA samples and documentation\n\n\
            **After installation:**\n\
            - CUDA binaries are in `/opt/cuda/bin/`\n\
            - Libraries are in `/opt/cuda/lib64/`\n\
            - Add to PATH: `export PATH=/opt/cuda/bin:$PATH`\n\
            - Add to LD_LIBRARY_PATH: `export LD_LIBRARY_PATH=/opt/cuda/lib64:$LD_LIBRARY_PATH`\n\n\
            **Usage:**\n\
            - Compile CUDA programs: `nvcc myprogram.cu -o myprogram`\n\
            - Check CUDA-enabled GPUs: `nvidia-smi`\n\n\
            **Package size warning:**\n\
            - CUDA toolkit is large (~3-4 GB)\n\
            - Ensure you have sufficient disk space",
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
            serde_json::Value::String("nvidia.rs".to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("nvidia_install_cuda".to_string()),
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_nvidia_requests() {
        assert!(NvidiaRecipe::matches_request("install nvidia drivers"));
        assert!(NvidiaRecipe::matches_request("setup NVIDIA GPU"));
        assert!(NvidiaRecipe::matches_request("configure nvidia"));
        assert!(NvidiaRecipe::matches_request("check nvidia status"));
        assert!(NvidiaRecipe::matches_request("install CUDA"));
        assert!(NvidiaRecipe::matches_request("setup graphics driver"));

        // Should not match informational queries
        assert!(!NvidiaRecipe::matches_request("what is nvidia"));
        assert!(!NvidiaRecipe::matches_request("tell me about GPU drivers"));
        assert!(!NvidiaRecipe::matches_request("explain nvidia"));

        // Should not match unrelated queries
        assert!(!NvidiaRecipe::matches_request("install firefox"));
        assert!(!NvidiaRecipe::matches_request("update system"));
    }

    #[test]
    fn test_operation_detection() {
        assert_eq!(
            NvidiaRecipe::detect_operation("install nvidia drivers"),
            NvidiaOperation::Install
        );
        assert_eq!(
            NvidiaRecipe::detect_operation("check nvidia status"),
            NvidiaOperation::CheckStatus
        );
        assert_eq!(
            NvidiaRecipe::detect_operation("configure xorg for nvidia"),
            NvidiaOperation::ConfigureXorg
        );
        assert_eq!(
            NvidiaRecipe::detect_operation("install CUDA"),
            NvidiaOperation::InstallCuda
        );
    }

    #[test]
    fn test_install_plan() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "install nvidia".to_string());
        telemetry.insert("internet_connected".to_string(), "true".to_string());

        let plan = NvidiaRecipe::build_plan(&telemetry).unwrap();

        assert_eq!(plan.goals.len(), 4);
        assert!(plan.analysis.contains("NVIDIA proprietary drivers"));
        assert!(plan.notes_for_user.contains("REBOOT REQUIRED"));

        // Should have install commands
        assert!(plan
            .command_plan
            .iter()
            .any(|cmd| cmd.id == "install-nvidia"));
        assert!(plan
            .command_plan
            .iter()
            .any(|cmd| cmd.id == "remove-nouveau"));

        // Should have rollback
        assert!(!plan.rollback_plan.is_empty());
    }

    #[test]
    fn test_check_status_plan() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "check nvidia status".to_string());

        let plan = NvidiaRecipe::build_plan(&telemetry).unwrap();

        assert!(plan.analysis.contains("Checking NVIDIA GPU driver status"));

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
    fn test_configure_xorg_plan() {
        let mut telemetry = HashMap::new();
        telemetry.insert(
            "user_request".to_string(),
            "configure xorg for nvidia".to_string(),
        );

        let plan = NvidiaRecipe::build_plan(&telemetry).unwrap();

        assert!(plan.analysis.contains("Xorg configuration"));
        assert!(plan
            .command_plan
            .iter()
            .any(|cmd| cmd.id == "generate-xorg-config"));
        assert!(!plan.rollback_plan.is_empty());
    }

    #[test]
    fn test_install_cuda_plan() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "install CUDA".to_string());
        telemetry.insert("internet_connected".to_string(), "true".to_string());

        let plan = NvidiaRecipe::build_plan(&telemetry).unwrap();

        assert!(plan.analysis.contains("CUDA toolkit"));
        assert!(plan
            .command_plan
            .iter()
            .any(|cmd| cmd.id == "install-cuda"));

        // Should have prerequisite check
        assert!(plan
            .necessary_checks
            .iter()
            .any(|check| check.id == "check-nvidia-driver"));
    }

    #[test]
    fn test_no_internet_warning() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "install nvidia".to_string());
        telemetry.insert("internet_connected".to_string(), "false".to_string());

        let plan = NvidiaRecipe::build_plan(&telemetry).unwrap();

        assert!(plan.notes_for_user.contains("No internet connection"));
        assert!(plan
            .command_plan
            .iter()
            .any(|cmd| cmd.id == "check-internet"));
    }
}
