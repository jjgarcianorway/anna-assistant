//! AUR Package Installation Recipe
//!
//! Beta.152: Deterministic recipe for AUR package management
//!
//! This module generates safe ActionPlans for:
//! - Detecting installed AUR helpers (yay, paru)
//! - Installing AUR helpers if missing
//! - Installing packages from AUR
//! - Checking AUR package information

use anna_common::action_plan_v3::{
    ActionPlan, CommandStep, DetectionResults, NecessaryCheck, PlanMeta, RiskLevel, RollbackStep,
};
use anyhow::Result;
use std::collections::HashMap;

/// AUR package management scenario detector
pub struct AurRecipe;

/// AUR operation types
#[derive(Debug, Clone, PartialEq)]
enum AurOperation {
    CheckHelper,       // "do I have yay installed"
    InstallHelper,     // "install yay" / "setup AUR helper"
    InstallPackage,    // "install X from AUR"
    SearchPackage,     // "search AUR for X"
}

impl AurRecipe {
    /// Check if user request matches AUR operations
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();

        // Exclude informational queries about AUR itself
        if input_lower.contains("what is") || input_lower.contains("tell me about") {
            return false;
        }

        // Must mention AUR or AUR helpers with action context
        let has_aur_context = input_lower.contains("aur")
            || input_lower.contains("yay")
            || input_lower.contains("paru")
            || input_lower.contains("user repository");

        // Need action keywords
        let has_action = input_lower.contains("install")
            || input_lower.contains("get")
            || input_lower.contains("search")
            || input_lower.contains("check")
            || input_lower.contains("do I have")
            || input_lower.contains("setup")
            || input_lower.contains("from");

        has_aur_context && has_action
    }

    /// Generate AUR operation ActionPlan
    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let operation = Self::detect_operation(
            telemetry
                .get("user_request")
                .map(|s| s.as_str())
                .unwrap_or(""),
        );

        match operation {
            AurOperation::CheckHelper => Self::build_check_helper_plan(),
            AurOperation::InstallHelper => Self::build_install_helper_plan(telemetry),
            AurOperation::InstallPackage => {
                let package_name = telemetry
                    .get("package_name")
                    .map(|s| s.as_str())
                    .unwrap_or("<package>");
                Self::build_install_package_plan(package_name, telemetry)
            }
            AurOperation::SearchPackage => {
                let search_term = telemetry
                    .get("search_term")
                    .map(|s| s.as_str())
                    .unwrap_or("<search-term>");
                Self::build_search_package_plan(search_term)
            }
        }
    }

    fn detect_operation(user_input: &str) -> AurOperation {
        let input_lower = user_input.to_lowercase();

        // Check first (most specific)
        if input_lower.contains("search") || input_lower.contains("find") {
            AurOperation::SearchPackage
        } else if input_lower.contains("check")
            || input_lower.contains("do i have")
            || input_lower.contains("have i got")
            || (input_lower.contains("is") && (input_lower.contains("installed") || input_lower.contains("available")))
        {
            AurOperation::CheckHelper
        } else if (input_lower.contains("install") || input_lower.contains("setup"))
            && (input_lower.contains("yay") || input_lower.contains("paru") || input_lower.contains("helper"))
        {
            AurOperation::InstallHelper
        } else {
            // Default to package installation
            AurOperation::InstallPackage
        }
    }

    fn build_check_helper_plan() -> Result<ActionPlan> {
        let necessary_checks = vec![];

        let command_plan = vec![
            CommandStep {
                id: "check-yay".to_string(),
                description: "Check if yay is installed".to_string(),
                command: "which yay && yay --version || echo 'yay not found'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-paru".to_string(),
                description: "Check if paru is installed".to_string(),
                command: "which paru && paru --version || echo 'paru not found'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-base-devel".to_string(),
                description: "Check if base-devel is installed (required for building AUR packages)".to_string(),
                command: "pacman -Qg base-devel | wc -l".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let analysis = "User requests check for AUR helper availability. Checking for yay, paru, \
                        and base-devel (required for building packages).".to_string();

        let goals = vec![
            "Check if yay AUR helper is installed".to_string(),
            "Check if paru AUR helper is installed".to_string(),
            "Verify base-devel group is installed".to_string(),
        ];

        let notes_for_user = "AUR (Arch User Repository) helpers:\n\n\
             Popular AUR helpers:\n\
             • yay - Go-based, most popular, feature-rich\n\
             • paru - Rust-based, newer, similar features to yay\n\n\
             To install yay (if not present):\n\
             1. Install base-devel:\n\
                sudo pacman -S --needed base-devel git\n\n\
             2. Clone and build yay:\n\
                git clone https://aur.archlinux.org/yay.git\n\
                cd yay\n\
                makepkg -si\n\n\
             To install paru (if not present):\n\
             1. Install base-devel:\n\
                sudo pacman -S --needed base-devel git\n\n\
             2. Clone and build paru:\n\
                git clone https://aur.archlinux.org/paru.git\n\
                cd paru\n\
                makepkg -si\n\n\
             After installing an AUR helper, you can install AUR packages with:\n\
             yay -S <package-name>\n\
             # or\n\
             paru -S <package-name>\n\n\
             Risk: INFO - Read-only checks".to_string();

        Self::build_action_plan(
            analysis,
            goals,
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            "aur_check_helper",
        )
    }

    fn build_install_helper_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let has_internet = telemetry
            .get("internet_connected")
            .map(|v| v == "true")
            .unwrap_or(false);

        let disk_space_gb = telemetry
            .get("disk_free_gb")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0);

        let user_home = telemetry.get("home").map(|s| s.as_str()).unwrap_or("~");

        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-internet".to_string(),
                description: "Verify internet connectivity".to_string(),
                command: "ping -c 1 archlinux.org".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "check-base-devel".to_string(),
                description: "Check if base-devel is installed".to_string(),
                command: "pacman -Qg base-devel".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "check-git".to_string(),
                description: "Check if git is installed".to_string(),
                command: "which git".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "install-prerequisites".to_string(),
                description: "Install base-devel and git (required for building AUR packages)".to_string(),
                command: "sudo pacman -S --needed --noconfirm base-devel git".to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: None,
                requires_confirmation: true,
            },
            CommandStep {
                id: "clone-yay".to_string(),
                description: "Clone yay repository from AUR".to_string(),
                command: format!("cd {} && git clone https://aur.archlinux.org/yay.git", user_home),
                risk_level: RiskLevel::Low,
                rollback_id: Some("cleanup-yay-dir".to_string()),
                requires_confirmation: false,
            },
            CommandStep {
                id: "build-yay".to_string(),
                description: "Build yay from source".to_string(),
                command: format!("cd {}/yay && makepkg -si --noconfirm", user_home),
                risk_level: RiskLevel::High,
                rollback_id: Some("remove-yay".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "verify-yay".to_string(),
                description: "Verify yay installation".to_string(),
                command: "yay --version".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "cleanup".to_string(),
                description: "Remove yay build directory".to_string(),
                command: format!("rm -rf {}/yay", user_home),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![
            RollbackStep {
                id: "remove-yay".to_string(),
                description: "Uninstall yay".to_string(),
                command: "sudo pacman -Rns --noconfirm yay".to_string(),
            },
            RollbackStep {
                id: "cleanup-yay-dir".to_string(),
                description: "Remove yay build directory".to_string(),
                command: format!("rm -rf {}/yay", user_home),
            },
        ];

        let mut analysis_parts = vec![
            "User requests installation of AUR helper (yay). This will install build dependencies, \
             clone yay from AUR, and build it from source."
                .to_string(),
            format!("System has {:.1} GB free disk space.", disk_space_gb),
        ];

        if !has_internet {
            analysis_parts.push(
                "⚠️ WARNING: Internet connectivity not confirmed. AUR helper installation requires network access."
                    .to_string(),
            );
        }

        if disk_space_gb < 1.0 {
            analysis_parts
                .push("⚠️ WARNING: Low disk space. Building packages requires ~500MB-1GB.".to_string());
        }

        let analysis = analysis_parts.join(" ");

        let goals = vec![
            "Install base-devel and git (build prerequisites)".to_string(),
            "Clone yay repository from AUR".to_string(),
            "Build and install yay from source".to_string(),
            "Verify yay installation".to_string(),
            "Clean up build files".to_string(),
        ];

        let notes_for_user = format!(
            "⚠️ IMPORTANT: AUR packages are user-submitted and not officially supported.\n\n\
             This will install yay, the most popular AUR helper:\n\
             1. Install build dependencies (base-devel, git)\n\
             2. Clone yay repository to {}/yay\n\
             3. Build yay using makepkg\n\
             4. Install yay system-wide\n\
             5. Clean up build directory\n\n\
             After installation, you can:\n\
             • Install AUR packages: yay -S <package>\n\
             • Search AUR: yay -Ss <search-term>\n\
             • Update AUR packages: yay -Syu\n\
             • Get package info: yay -Si <package>\n\n\
             ⚠️ AUR Package Safety:\n\
             • Always review PKGBUILD before installing: yay -Gp <package>\n\
             • Check package popularity and votes on aur.archlinux.org\n\
             • Be cautious with orphaned or low-vote packages\n\
             • Never run makepkg as root\n\n\
             Alternative: paru (Rust-based AUR helper)\n\
             If you prefer paru instead:\n\
             git clone https://aur.archlinux.org/paru.git\n\
             cd paru\n\
             makepkg -si\n\n\
             Risk: HIGH - Building and installing system software from source\n\
             Estimated time: 2-5 minutes\n\
             Estimated disk usage: ~50-100MB",
            user_home
        );

        Self::build_action_plan(
            analysis,
            goals,
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            "aur_install_helper",
        )
    }

    fn build_install_package_plan(package_name: &str, telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let has_internet = telemetry
            .get("internet_connected")
            .map(|v| v == "true")
            .unwrap_or(false);

        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-aur-helper".to_string(),
                description: "Check if AUR helper (yay or paru) is installed".to_string(),
                command: "which yay || which paru || echo 'No AUR helper found'".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
            NecessaryCheck {
                id: "check-package-exists".to_string(),
                description: format!("Check if {} exists in AUR", package_name),
                command: format!("yay -Ss '^{}$' || paru -Ss '^{}$' || echo 'Package not found in AUR'", package_name, package_name),
                risk_level: RiskLevel::Info,
                required: true,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "show-package-info".to_string(),
                description: format!("Show information about {} from AUR", package_name),
                command: format!("yay -Si {} || paru -Si {}", package_name, package_name),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "review-pkgbuild".to_string(),
                description: format!("Display PKGBUILD for {} (REVIEW THIS BEFORE PROCEEDING)", package_name),
                command: format!("yay -Gp {} || paru -Gp {}", package_name, package_name),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "install-aur-package".to_string(),
                description: format!("Install {} from AUR (will build from source)", package_name),
                command: format!("yay -S --noconfirm {} || paru -S --noconfirm {}", package_name, package_name),
                risk_level: RiskLevel::High,
                rollback_id: Some("remove-aur-package".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "verify-installation".to_string(),
                description: format!("Verify {} was installed successfully", package_name),
                command: format!("pacman -Q {}", package_name),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![RollbackStep {
            id: "remove-aur-package".to_string(),
            description: format!("Uninstall {} and its dependencies", package_name),
            command: format!("sudo pacman -Rns --noconfirm {}", package_name),
        }];

        let mut analysis_parts = vec![format!(
            "User requests installation of {} from AUR. ⚠️ AUR packages are user-submitted and \
             require careful review before installation.",
            package_name
        )];

        if !has_internet {
            analysis_parts.push(
                "⚠️ WARNING: Internet connectivity not confirmed. AUR installation requires network access."
                    .to_string(),
            );
        }

        let analysis = analysis_parts.join(" ");

        let goals = vec![
            "Verify AUR helper is installed".to_string(),
            format!("Display information and PKGBUILD for {}", package_name),
            format!("Build and install {} from AUR", package_name),
            format!("Verify {} installation", package_name),
        ];

        let notes_for_user = format!(
            "⚠️ CRITICAL: Review PKGBUILD before installing!\n\n\
             AUR packages are USER-SUBMITTED and not officially vetted by Arch Linux.\n\n\
             Security checklist:\n\
             1. Review the PKGBUILD output above\n\
             2. Check for suspicious commands (curl | sh, downloading from unknown sources)\n\
             3. Verify package has good votes/popularity on aur.archlinux.org\n\
             4. Check last update date and maintainer reputation\n\
             5. Look for comments reporting issues\n\n\
             Installation process:\n\
             1. Clone package source from AUR\n\
             2. Download dependencies\n\
             3. Build package using makepkg\n\
             4. Install built package\n\n\
             After installation:\n\
             • AUR packages are updated with: yay -Syu (includes official + AUR)\n\
             • View package files: pacman -Ql {}\n\
             • View package info: pacman -Qi {}\n\n\
             To uninstall:\n\
             sudo pacman -Rns {}\n\n\
             Common AUR packages:\n\
             • yay - AUR helper itself\n\
             • google-chrome - Chrome browser\n\
             • visual-studio-code-bin - VS Code\n\
             • spotify - Spotify client\n\
             • zoom - Zoom video conferencing\n\n\
             ⚠️ Never install AUR packages you don't trust!\n\
             ⚠️ Always review PKGBUILD and .install scripts!\n\n\
             Risk: HIGH - Installing user-submitted software, building from source\n\
             Estimated time: 1-10 minutes depending on package size",
            package_name, package_name, package_name
        );

        Self::build_action_plan(
            analysis,
            goals,
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            "aur_install_package",
        )
    }

    fn build_search_package_plan(search_term: &str) -> Result<ActionPlan> {
        let necessary_checks = vec![NecessaryCheck {
            id: "check-aur-helper".to_string(),
            description: "Check if AUR helper is installed".to_string(),
            command: "which yay || which paru || echo 'No AUR helper found'".to_string(),
            risk_level: RiskLevel::Info,
            required: false,
        }];

        let command_plan = vec![
            CommandStep {
                id: "search-aur".to_string(),
                description: format!("Search AUR for packages matching '{}'", search_term),
                command: format!("yay -Ss {} || paru -Ss {}", search_term, search_term),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "search-count".to_string(),
                description: "Count number of matching packages".to_string(),
                command: format!("yay -Ss {} | grep -c 'aur/' || echo '0'", search_term),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let analysis = format!(
            "User requests search for '{}' in AUR. This is a read-only operation.",
            search_term
        );

        let goals = vec![
            format!("Search AUR for packages matching '{}'", search_term),
            "Display package names, versions, and descriptions".to_string(),
        ];

        let notes_for_user = format!(
            "AUR search results for '{}':\n\n\
             Results show:\n\
             • Package name (aur/ prefix indicates AUR package)\n\
             • Version\n\
             • Description\n\
             • Number of votes (popularity)\n\
             • Maintainer status\n\n\
             To get detailed info about a package:\n\
             yay -Si <package-name>\n\n\
             To install a package:\n\
             yay -S <package-name>\n\n\
             To review PKGBUILD before installing:\n\
             yay -Gp <package-name>\n\n\
             Browse AUR packages online:\n\
             https://aur.archlinux.org/packages/?K={}\n\n\
             Risk: INFO - Read-only search operation",
            search_term, search_term
        );

        Self::build_action_plan(
            analysis,
            goals,
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            "aur_search_package",
        )
    }

    fn build_action_plan(
        analysis: String,
        goals: Vec<String>,
        necessary_checks: Vec<NecessaryCheck>,
        command_plan: Vec<CommandStep>,
        rollback_plan: Vec<RollbackStep>,
        notes_for_user: String,
        template_name: &str,
    ) -> Result<ActionPlan> {
        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("aur.rs".to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some(template_name.to_string()),
            llm_version: "deterministic_recipe_v1".to_string(),
        };

        Ok(ActionPlan {
            analysis,
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
    fn test_matches_aur_requests() {
        // Direct AUR mentions
        assert!(AurRecipe::matches_request("install package from AUR"));
        assert!(AurRecipe::matches_request("get yay"));
        assert!(AurRecipe::matches_request("install paru"));

        // AUR helper context
        assert!(AurRecipe::matches_request("do I have yay installed"));
        assert!(AurRecipe::matches_request("setup AUR helper"));

        // Search
        assert!(AurRecipe::matches_request("search AUR for chrome"));

        // "from" context (might be AUR)
        assert!(AurRecipe::matches_request("install spotify from aur"));

        // Should not match
        assert!(!AurRecipe::matches_request("what is AUR"));
        assert!(!AurRecipe::matches_request("install firefox")); // No AUR context
    }

    #[test]
    fn test_operation_detection() {
        assert_eq!(
            AurRecipe::detect_operation("do I have yay"),
            AurOperation::CheckHelper
        );
        assert_eq!(
            AurRecipe::detect_operation("install yay"),
            AurOperation::InstallHelper
        );
        assert_eq!(
            AurRecipe::detect_operation("install chrome from AUR"),
            AurOperation::InstallPackage
        );
        assert_eq!(
            AurRecipe::detect_operation("search AUR for vim"),
            AurOperation::SearchPackage
        );
    }

    #[test]
    fn test_check_helper_plan_is_info_only() {
        let plan = AurRecipe::build_check_helper_plan().unwrap();

        // All commands should be INFO level
        for cmd in &plan.command_plan {
            assert_eq!(cmd.risk_level, RiskLevel::Info);
            assert!(!cmd.requires_confirmation);
        }

        assert_eq!(plan.meta.template_used.as_ref().unwrap(), "aur_check_helper");
    }

    #[test]
    fn test_install_helper_plan_is_high_risk() {
        let mut telemetry = HashMap::new();
        telemetry.insert("internet_connected".to_string(), "true".to_string());
        telemetry.insert("disk_free_gb".to_string(), "10.0".to_string());
        telemetry.insert("home".to_string(), "/home/testuser".to_string());

        let plan = AurRecipe::build_install_helper_plan(&telemetry).unwrap();

        // Build step should be HIGH risk
        let build_step = plan.command_plan.iter().find(|c| c.id == "build-yay").unwrap();
        assert_eq!(build_step.risk_level, RiskLevel::High);
        assert!(build_step.requires_confirmation);

        assert_eq!(plan.meta.template_used.as_ref().unwrap(), "aur_install_helper");
    }

    #[test]
    fn test_install_package_shows_security_warnings() {
        let mut telemetry = HashMap::new();
        telemetry.insert("user_request".to_string(), "install chrome from AUR".to_string());
        telemetry.insert("package_name".to_string(), "google-chrome".to_string());
        telemetry.insert("internet_connected".to_string(), "true".to_string());

        let plan = AurRecipe::build_install_package_plan("google-chrome", &telemetry).unwrap();

        // Should include PKGBUILD review step
        assert!(plan.command_plan.iter().any(|c| c.id == "review-pkgbuild"));

        // Should have strong security warnings
        assert!(plan.notes_for_user.contains("⚠️ CRITICAL"));
        assert!(plan.notes_for_user.contains("Review PKGBUILD"));
        assert!(plan.notes_for_user.contains("USER-SUBMITTED"));

        // Install step should be HIGH risk
        let install_step = plan.command_plan.iter().find(|c| c.id == "install-aur-package").unwrap();
        assert_eq!(install_step.risk_level, RiskLevel::High);
        assert!(install_step.requires_confirmation);

        assert_eq!(plan.meta.template_used.as_ref().unwrap(), "aur_install_package");
    }

    #[test]
    fn test_search_package_is_read_only() {
        let plan = AurRecipe::build_search_package_plan("spotify").unwrap();

        // All commands should be INFO level
        for cmd in &plan.command_plan {
            assert_eq!(cmd.risk_level, RiskLevel::Info);
            assert!(!cmd.requires_confirmation);
        }

        assert!(plan.command_plan[0].command.contains("yay -Ss spotify"));
        assert_eq!(plan.meta.template_used.as_ref().unwrap(), "aur_search_package");
    }
}
