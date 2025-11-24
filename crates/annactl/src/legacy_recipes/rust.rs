// rust.rs - Rust development environment recipe
// Beta.154: Rust toolchain setup and configuration

use anna_common::action_plan_v3::{
    ActionPlan, CommandStep, DetectionResults, NecessaryCheck, PlanMeta, RiskLevel, RollbackStep,
};
use anyhow::Result;
use serde_json;
use std::collections::HashMap;

pub struct RustRecipe;

#[derive(Debug, PartialEq)]
enum RustOperation {
    Install,           // Install rustup and Rust toolchain
    InstallTools,      // Install common cargo tools
    CheckStatus,       // Check Rust installation status
    UpdateToolchain,   // Update Rust toolchain
}

impl RustRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();

        // Exclude informational queries
        if input_lower.contains("what is")
            || input_lower.contains("tell me about")
            || input_lower.contains("explain")
        {
            return false;
        }

        // Rust-related keywords
        let has_rust_context = input_lower.contains("rust")
            || input_lower.contains("rustup")
            || input_lower.contains("cargo");

        // Action keywords
        let has_action = input_lower.contains("install")
            || input_lower.contains("setup")
            || input_lower.contains("update")
            || input_lower.contains("upgrade")
            || input_lower.contains("check")
            || input_lower.contains("status")
            || input_lower.contains("configure");

        has_rust_context && has_action
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_request = telemetry
            .get("user_request")
            .map(|s| s.as_str())
            .unwrap_or("");

        let operation = Self::detect_operation(user_request);

        match operation {
            RustOperation::Install => Self::build_install_plan(telemetry),
            RustOperation::InstallTools => Self::build_install_tools_plan(telemetry),
            RustOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            RustOperation::UpdateToolchain => Self::build_update_toolchain_plan(telemetry),
        }
    }

    fn detect_operation(user_input: &str) -> RustOperation {
        let input_lower = user_input.to_lowercase();

        // Check for status first
        if input_lower.contains("check") || input_lower.contains("status") {
            return RustOperation::CheckStatus;
        }

        // Check for update
        if input_lower.contains("update") || input_lower.contains("upgrade") {
            return RustOperation::UpdateToolchain;
        }

        // Check for tools installation
        if (input_lower.contains("tool") || input_lower.contains("cargo-"))
            && (input_lower.contains("install") || input_lower.contains("setup"))
        {
            return RustOperation::InstallTools;
        }

        // Default to install
        RustOperation::Install
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let has_internet = telemetry
            .get("internet_connected")
            .map(|v| v == "true")
            .unwrap_or(false);

        let mut warning = String::new();
        if !has_internet {
            warning = "⚠️ No internet detected. Rust installation requires internet connection.\n\n"
                .to_string();
        }

        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-rustup-installed".to_string(),
                description: "Check if rustup is already installed".to_string(),
                command: "which rustup".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
            NecessaryCheck {
                id: "check-rust-installed".to_string(),
                description: "Check if Rust is already installed".to_string(),
                command: "rustc --version".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "download-rustup".to_string(),
                description: "Download rustup installer".to_string(),
                command: "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/rustup.sh".to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: Some("cleanup-installer".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "install-rustup".to_string(),
                description: "Install rustup and Rust toolchain (stable)".to_string(),
                command: "sh /tmp/rustup.sh -y --default-toolchain stable".to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: Some("uninstall-rust".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "source-env".to_string(),
                description: "Load Rust environment variables".to_string(),
                command: "source $HOME/.cargo/env".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "verify-installation".to_string(),
                description: "Verify Rust installation".to_string(),
                command: "$HOME/.cargo/bin/rustc --version && $HOME/.cargo/bin/cargo --version".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "cleanup-installer".to_string(),
                description: "Remove installer script".to_string(),
                command: "rm -f /tmp/rustup.sh".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![
            RollbackStep {
                id: "uninstall-rust".to_string(),
                description: "Uninstall Rust and rustup".to_string(),
                command: "rustup self uninstall -y".to_string(),
            },
            RollbackStep {
                id: "cleanup-installer".to_string(),
                description: "Remove installer script".to_string(),
                command: "rm -f /tmp/rustup.sh".to_string(),
            },
        ];

        let notes_for_user = format!(
            "{}🦀 Installing Rust Development Environment\n\n\
             This will install:\n\
             - rustup: Rust toolchain installer and manager\n\
             - rustc: Rust compiler (stable)\n\
             - cargo: Rust package manager and build tool\n\
             - rust-std: Rust standard library\n\n\
             Installation location: ~/.cargo/\n\
             Environment: ~/.cargo/env will be sourced\n\n\
             ⚠️ IMPORTANT:\n\
             1. After installation, restart your shell or run:\n\
                source $HOME/.cargo/env\n\
             2. The Rust binaries will be in: ~/.cargo/bin\n\
             3. Your PATH will be updated automatically\n\n\
             Common next steps:\n\
             - Install tools: `annactl \"install Rust development tools\"`\n\
             - Create project: `cargo new myproject`\n\
             - Update Rust: `annactl \"update Rust toolchain\"`\n\n\
             Documentation: https://doc.rust-lang.org/",
            warning
        );

        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("rust.rs".to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("rust_install".to_string()),
            llm_version: "deterministic_recipe_v1".to_string(),
        };

        Ok(ActionPlan {
            analysis: "User wants to install Rust development environment. This will install rustup, the Rust compiler, and cargo package manager.".to_string(),
            goals: vec![
                "Download rustup installer".to_string(),
                "Install Rust stable toolchain".to_string(),
                "Configure environment variables".to_string(),
                "Verify installation".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            meta,
        })
    }

    fn build_install_tools_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let has_internet = telemetry
            .get("internet_connected")
            .map(|v| v == "true")
            .unwrap_or(false);

        let mut warning = String::new();
        if !has_internet {
            warning = "⚠️ No internet detected. Tool installation requires internet connection.\n\n"
                .to_string();
        }

        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-cargo".to_string(),
                description: "Check if cargo is installed".to_string(),
                command: "which cargo".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "install-rustfmt".to_string(),
                description: "Install rustfmt (code formatter)".to_string(),
                command: "rustup component add rustfmt".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "install-clippy".to_string(),
                description: "Install clippy (linter)".to_string(),
                command: "rustup component add clippy".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "install-cargo-edit".to_string(),
                description: "Install cargo-edit (add/rm dependencies)".to_string(),
                command: "cargo install cargo-edit".to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: None,
                requires_confirmation: true,
            },
            CommandStep {
                id: "install-cargo-watch".to_string(),
                description: "Install cargo-watch (auto-rebuild on changes)".to_string(),
                command: "cargo install cargo-watch".to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: None,
                requires_confirmation: true,
            },
            CommandStep {
                id: "verify-tools".to_string(),
                description: "Verify tool installation".to_string(),
                command: "rustfmt --version && cargo clippy --version && cargo add --version && cargo watch --version".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![
            RollbackStep {
                id: "note-manual-uninstall".to_string(),
                description: "Cargo tools can be uninstalled with: cargo uninstall <tool>".to_string(),
                command: "echo 'cargo uninstall cargo-edit cargo-watch'".to_string(),
            },
        ];

        let notes_for_user = format!(
            "{}🔧 Installing Rust Development Tools\n\n\
             This will install essential Rust development tools:\n\n\
             **Components** (via rustup):\n\
             - rustfmt: Code formatter (rustfmt file.rs)\n\
             - clippy: Linter for catching common mistakes\n\n\
             **Cargo extensions** (compiled from source):\n\
             - cargo-edit: Add/remove dependencies easily\n\
               Usage: cargo add <crate>, cargo rm <crate>\n\
             - cargo-watch: Auto-rebuild on file changes\n\
               Usage: cargo watch -x run\n\n\
             ⏱️ Installation time: 5-10 minutes (cargo tools compile from source)\n\n\
             Common workflows:\n\
             - Format code: `cargo fmt`\n\
             - Lint code: `cargo clippy`\n\
             - Add dependency: `cargo add serde`\n\
             - Watch and run: `cargo watch -x run`\n\n\
             To uninstall tools:\n\
             - cargo uninstall cargo-edit\n\
             - cargo uninstall cargo-watch",
            warning
        );

        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("rust.rs".to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("rust_install_tools".to_string()),
            llm_version: "deterministic_recipe_v1".to_string(),
        };

        Ok(ActionPlan {
            analysis: "User wants to install essential Rust development tools including rustfmt, clippy, cargo-edit, and cargo-watch.".to_string(),
            goals: vec![
                "Install rustfmt and clippy components".to_string(),
                "Install cargo-edit for dependency management".to_string(),
                "Install cargo-watch for auto-rebuild".to_string(),
                "Verify tool installation".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            meta,
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let necessary_checks = vec![];

        let command_plan = vec![
            CommandStep {
                id: "check-rustup".to_string(),
                description: "Check rustup installation".to_string(),
                command: "which rustup && rustup --version || echo 'rustup not installed'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-rust".to_string(),
                description: "Check Rust compiler version".to_string(),
                command: "rustc --version || echo 'rustc not installed'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-cargo".to_string(),
                description: "Check cargo version".to_string(),
                command: "cargo --version || echo 'cargo not installed'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "list-toolchains".to_string(),
                description: "List installed Rust toolchains".to_string(),
                command: "rustup toolchain list 2>/dev/null || echo 'rustup not available'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-components".to_string(),
                description: "Check installed components".to_string(),
                command: "rustup component list --installed 2>/dev/null || echo 'No components found'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "list-cargo-tools".to_string(),
                description: "List installed cargo tools".to_string(),
                command: "ls ~/.cargo/bin/ 2>/dev/null | grep -v '^cargo$\\|^rustc$\\|^rustup$' | head -20 || echo 'No additional tools installed'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let notes_for_user = "ℹ️ Rust Development Environment Status\n\n\
             This shows the current state of your Rust installation:\n\
             - rustup version and availability\n\
             - Rust compiler (rustc) version\n\
             - Cargo package manager version\n\
             - Installed toolchains (stable, nightly, etc.)\n\
             - Installed components (rustfmt, clippy, etc.)\n\
             - Additional cargo tools\n\n\
             Common next steps:\n\
             - Install Rust: `annactl \"install Rust\"`\n\
             - Install tools: `annactl \"install Rust development tools\"`\n\
             - Update Rust: `annactl \"update Rust toolchain\"`"
            .to_string();

        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("rust.rs".to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("rust_check_status".to_string()),
            llm_version: "deterministic_recipe_v1".to_string(),
        };

        Ok(ActionPlan {
            analysis: "User wants to check Rust installation status. This is a read-only operation showing all Rust-related tools and versions.".to_string(),
            goals: vec![
                "Check rustup, rustc, and cargo versions".to_string(),
                "List installed toolchains and components".to_string(),
                "Show additional cargo tools".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            meta,
        })
    }

    fn build_update_toolchain_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-rustup".to_string(),
                description: "Check if rustup is installed".to_string(),
                command: "which rustup".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "update-rustup".to_string(),
                description: "Update rustup itself".to_string(),
                command: "rustup self update".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "update-toolchain".to_string(),
                description: "Update Rust toolchain".to_string(),
                command: "rustup update".to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: None,
                requires_confirmation: true,
            },
            CommandStep {
                id: "show-versions".to_string(),
                description: "Show updated versions".to_string(),
                command: "rustc --version && cargo --version".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![
            RollbackStep {
                id: "note-downgrade".to_string(),
                description: "To downgrade, install specific version: rustup install <version>".to_string(),
                command: "echo 'rustup toolchain install 1.XX.0'".to_string(),
            },
        ];

        let notes_for_user = "🔄 Updating Rust Toolchain\n\n\
             This will:\n\
             1. Update rustup to the latest version\n\
             2. Update all installed Rust toolchains to their latest versions\n\
             3. Update components (rustfmt, clippy, etc.)\n\n\
             The update is generally safe and backward-compatible.\n\
             Your existing Rust projects will continue to work.\n\n\
             After updating:\n\
             - Recompile your projects to use the new toolchain\n\
             - Check for deprecation warnings: `cargo build`\n\
             - Review changelog: https://github.com/rust-lang/rust/blob/master/RELEASES.md\n\n\
             To pin a specific version:\n\
             - Install: `rustup install 1.XX.0`\n\
             - Override: `rustup override set 1.XX.0`"
            .to_string();

        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("rust.rs".to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("rust_update".to_string()),
            llm_version: "deterministic_recipe_v1".to_string(),
        };

        Ok(ActionPlan {
            analysis: "User wants to update the Rust toolchain. This will update rustup and all installed Rust versions.".to_string(),
            goals: vec![
                "Update rustup to latest version".to_string(),
                "Update Rust toolchains".to_string(),
                "Verify updated versions".to_string(),
            ],
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
    fn test_matches_rust_requests() {
        // Should match
        assert!(RustRecipe::matches_request("install Rust"));
        assert!(RustRecipe::matches_request("setup Rust development environment"));
        assert!(RustRecipe::matches_request("install rustup"));
        assert!(RustRecipe::matches_request("install Rust development tools"));
        assert!(RustRecipe::matches_request("check Rust status"));
        assert!(RustRecipe::matches_request("update Rust toolchain"));
        assert!(RustRecipe::matches_request("install cargo tools"));

        // Should NOT match
        assert!(!RustRecipe::matches_request("what is Rust"));
        assert!(!RustRecipe::matches_request("tell me about Rust"));
        assert!(!RustRecipe::matches_request("install docker"));
        assert!(!RustRecipe::matches_request("how much RAM do I have"));
    }

    #[test]
    fn test_operation_detection() {
        assert_eq!(
            RustRecipe::detect_operation("install Rust"),
            RustOperation::Install
        );
        assert_eq!(
            RustRecipe::detect_operation("install Rust development tools"),
            RustOperation::InstallTools
        );
        assert_eq!(
            RustRecipe::detect_operation("check Rust status"),
            RustOperation::CheckStatus
        );
        assert_eq!(
            RustRecipe::detect_operation("update Rust toolchain"),
            RustOperation::UpdateToolchain
        );
    }

    #[test]
    fn test_install_plan() {
        let mut telemetry = HashMap::new();
        telemetry.insert("internet_connected".to_string(), "true".to_string());

        let plan = RustRecipe::build_install_plan(&telemetry).unwrap();

        assert_eq!(plan.goals.len(), 4);
        assert!(plan.analysis.contains("Rust"));
        assert!(plan.command_plan.len() >= 4);
        assert!(plan.command_plan[0].command.contains("rustup"));
        assert_eq!(plan.command_plan[0].risk_level, RiskLevel::Medium);
        assert!(plan.notes_for_user.contains("~/.cargo/"));
    }

    #[test]
    fn test_install_tools_plan() {
        let mut telemetry = HashMap::new();
        telemetry.insert("internet_connected".to_string(), "true".to_string());

        let plan = RustRecipe::build_install_tools_plan(&telemetry).unwrap();

        assert!(plan.analysis.contains("tools"));
        assert!(plan.command_plan.iter().any(|c| c.command.contains("rustfmt")));
        assert!(plan.command_plan.iter().any(|c| c.command.contains("clippy")));
        assert!(plan.command_plan.iter().any(|c| c.command.contains("cargo-edit")));
        assert!(plan.command_plan.iter().any(|c| c.command.contains("cargo-watch")));
    }

    #[test]
    fn test_check_status_plan() {
        let telemetry = HashMap::new();
        let plan = RustRecipe::build_check_status_plan(&telemetry).unwrap();

        assert_eq!(plan.command_plan[0].risk_level, RiskLevel::Info);
        assert!(!plan.command_plan[0].requires_confirmation);
        assert!(plan.notes_for_user.contains("Status"));
    }

    #[test]
    fn test_update_toolchain_plan() {
        let telemetry = HashMap::new();
        let plan = RustRecipe::build_update_toolchain_plan(&telemetry).unwrap();

        assert!(plan.command_plan.iter().any(|c| c.command.contains("rustup update")));
        assert!(plan.notes_for_user.contains("Updating"));
    }

    #[test]
    fn test_no_internet_warning() {
        let mut telemetry = HashMap::new();
        telemetry.insert("internet_connected".to_string(), "false".to_string());

        let plan = RustRecipe::build_install_plan(&telemetry).unwrap();

        assert!(plan.notes_for_user.contains("No internet detected"));
    }
}
