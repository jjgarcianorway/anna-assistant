// python.rs - Python development environment recipe
// Beta.154: Python setup and configuration

use anna_common::action_plan_v3::{
    ActionPlan, CommandStep, DetectionResults, NecessaryCheck, PlanMeta, RiskLevel, RollbackStep,
};
use anyhow::Result;
use serde_json;
use std::collections::HashMap;

pub struct PythonRecipe;

#[derive(Debug, PartialEq)]
enum PythonOperation {
    Install,           // Install Python and pip
    InstallTools,      // Install development tools (black, pylint, mypy, etc.)
    CheckStatus,       // Check Python installation status
    CreateVenv,        // Create virtual environment
}

impl PythonRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();

        // Exclude informational queries
        if input_lower.contains("what is")
            || input_lower.contains("tell me about")
            || input_lower.contains("explain")
        {
            return false;
        }

        // Python-related keywords
        let has_python_context = input_lower.contains("python")
            || input_lower.contains("pip")
            || input_lower.contains("venv")
            || input_lower.contains("virtualenv");

        // Action keywords
        let has_action = input_lower.contains("install")
            || input_lower.contains("setup")
            || input_lower.contains("create")
            || input_lower.contains("check")
            || input_lower.contains("status")
            || input_lower.contains("configure");

        has_python_context && has_action
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_request = telemetry
            .get("user_request")
            .map(|s| s.as_str())
            .unwrap_or("");

        let operation = Self::detect_operation(user_request);

        match operation {
            PythonOperation::Install => Self::build_install_plan(telemetry),
            PythonOperation::InstallTools => Self::build_install_tools_plan(telemetry),
            PythonOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            PythonOperation::CreateVenv => Self::build_create_venv_plan(telemetry),
        }
    }

    fn detect_operation(user_input: &str) -> PythonOperation {
        let input_lower = user_input.to_lowercase();

        // Check for venv creation
        if (input_lower.contains("create") || input_lower.contains("setup"))
            && (input_lower.contains("venv") || input_lower.contains("virtualenv") || input_lower.contains("virtual environment"))
        {
            return PythonOperation::CreateVenv;
        }

        // Check for status
        if input_lower.contains("check") || input_lower.contains("status") {
            return PythonOperation::CheckStatus;
        }

        // Check for tools installation
        if (input_lower.contains("tool") || input_lower.contains("black") || input_lower.contains("pylint"))
            && (input_lower.contains("install") || input_lower.contains("setup"))
        {
            return PythonOperation::InstallTools;
        }

        // Default to install
        PythonOperation::Install
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let has_internet = telemetry
            .get("internet_connected")
            .map(|v| v == "true")
            .unwrap_or(false);

        let mut warning = String::new();
        if !has_internet {
            warning = "⚠️ No internet detected. Python installation requires internet connection.\n\n"
                .to_string();
        }

        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-python-installed".to_string(),
                description: "Check if Python is already installed".to_string(),
                command: "python --version || python3 --version".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "install-python".to_string(),
                description: "Install Python 3 and pip".to_string(),
                command: "sudo pacman -S --noconfirm python python-pip".to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: Some("uninstall-python".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "verify-python".to_string(),
                description: "Verify Python installation".to_string(),
                command: "python --version".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "verify-pip".to_string(),
                description: "Verify pip installation".to_string(),
                command: "pip --version".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "upgrade-pip".to_string(),
                description: "Upgrade pip to latest version".to_string(),
                command: "python -m pip install --user --upgrade pip".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![
            RollbackStep {
                id: "uninstall-python".to_string(),
                description: "Uninstall Python and pip".to_string(),
                command: "sudo pacman -Rns python python-pip".to_string(),
            },
        ];

        let notes_for_user = format!(
            "{}🐍 Installing Python Development Environment\n\n\
             This will install:\n\
             - Python 3 (latest version from Arch repos)\n\
             - pip: Python package installer\n\
             - setuptools and other essentials\n\n\
             Installation location:\n\
             - Python: /usr/bin/python\n\
             - pip: /usr/bin/pip\n\
             - User packages: ~/.local/lib/python3.X/site-packages\n\n\
             ⚠️ IMPORTANT:\n\
             1. Use virtual environments for projects (recommended):\n\
                `annactl \"create Python virtual environment\"`\n\
             2. Install packages with --user flag or in venv\n\
             3. Avoid using sudo with pip (can break system packages)\n\n\
             Common next steps:\n\
             - Install dev tools: `annactl \"install Python development tools\"`\n\
             - Create venv: `python -m venv myproject`\n\
             - Install package: `pip install --user requests`\n\n\
             Documentation: https://docs.python.org/3/",
            warning
        );

        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("python.rs".to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("python_install".to_string()),
            llm_version: "deterministic_recipe_v1".to_string(),
        };

        Ok(ActionPlan {
            analysis: "User wants to install Python development environment. This will install Python 3 and pip package manager from Arch repositories.".to_string(),
            goals: vec![
                "Install Python 3".to_string(),
                "Install pip package manager".to_string(),
                "Upgrade pip to latest version".to_string(),
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
                id: "check-pip".to_string(),
                description: "Check if pip is installed".to_string(),
                command: "which pip".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "install-black".to_string(),
                description: "Install black (code formatter)".to_string(),
                command: "pip install --user black".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "install-pylint".to_string(),
                description: "Install pylint (linter)".to_string(),
                command: "pip install --user pylint".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "install-mypy".to_string(),
                description: "Install mypy (type checker)".to_string(),
                command: "pip install --user mypy".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "install-pytest".to_string(),
                description: "Install pytest (testing framework)".to_string(),
                command: "pip install --user pytest".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "install-ipython".to_string(),
                description: "Install IPython (enhanced REPL)".to_string(),
                command: "pip install --user ipython".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "verify-tools".to_string(),
                description: "Verify tool installation".to_string(),
                command: "black --version && pylint --version && mypy --version && pytest --version && ipython --version".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![
            RollbackStep {
                id: "note-manual-uninstall".to_string(),
                description: "Tools can be uninstalled with: pip uninstall <tool>".to_string(),
                command: "echo 'pip uninstall black pylint mypy pytest ipython'".to_string(),
            },
        ];

        let notes_for_user = format!(
            "{}🔧 Installing Python Development Tools\n\n\
             This will install essential Python development tools:\n\n\
             **Code Quality**:\n\
             - black: Opinionated code formatter\n\
               Usage: black myfile.py\n\
             - pylint: Code linter and static analyzer\n\
               Usage: pylint mymodule\n\
             - mypy: Static type checker\n\
               Usage: mypy myproject/\n\n\
             **Testing & Development**:\n\
             - pytest: Testing framework\n\
               Usage: pytest tests/\n\
             - IPython: Enhanced interactive Python shell\n\
               Usage: ipython\n\n\
             Installation location: ~/.local/bin/\n\
             Make sure ~/.local/bin is in your PATH\n\n\
             Common workflows:\n\
             - Format code: `black .`\n\
             - Lint code: `pylint mymodule`\n\
             - Type check: `mypy .`\n\
             - Run tests: `pytest`\n\n\
             To uninstall:\n\
             pip uninstall black pylint mypy pytest ipython",
            warning
        );

        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("python.rs".to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("python_install_tools".to_string()),
            llm_version: "deterministic_recipe_v1".to_string(),
        };

        Ok(ActionPlan {
            analysis: "User wants to install essential Python development tools including black, pylint, mypy, pytest, and IPython.".to_string(),
            goals: vec![
                "Install code quality tools (black, pylint, mypy)".to_string(),
                "Install testing framework (pytest)".to_string(),
                "Install enhanced REPL (IPython)".to_string(),
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
                id: "check-python".to_string(),
                description: "Check Python version".to_string(),
                command: "python --version || python3 --version || echo 'Python not installed'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-pip".to_string(),
                description: "Check pip version".to_string(),
                command: "pip --version || echo 'pip not installed'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "list-user-packages".to_string(),
                description: "List user-installed packages".to_string(),
                command: "pip list --user 2>/dev/null | head -20 || echo 'No user packages'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-venv".to_string(),
                description: "Check if currently in virtual environment".to_string(),
                command: "if [ -n \"$VIRTUAL_ENV\" ]; then echo \"In venv: $VIRTUAL_ENV\"; else echo 'Not in virtual environment'; fi".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "python-path".to_string(),
                description: "Show Python executable location".to_string(),
                command: "which python".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let notes_for_user = "ℹ️ Python Development Environment Status\n\n\
             This shows the current state of your Python installation:\n\
             - Python version\n\
             - pip version and location\n\
             - User-installed packages (first 20)\n\
             - Virtual environment status\n\
             - Python executable path\n\n\
             Common next steps:\n\
             - Install Python: `annactl \"install Python\"`\n\
             - Install dev tools: `annactl \"install Python development tools\"`\n\
             - Create venv: `annactl \"create Python virtual environment\"`\n\n\
             Virtual environment best practice:\n\
             Always use virtual environments for projects to avoid\n\
             dependency conflicts between different projects."
            .to_string();

        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("python.rs".to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("python_check_status".to_string()),
            llm_version: "deterministic_recipe_v1".to_string(),
        };

        Ok(ActionPlan {
            analysis: "User wants to check Python installation status. This is a read-only operation showing Python version, pip, and installed packages.".to_string(),
            goals: vec![
                "Check Python and pip versions".to_string(),
                "List installed packages".to_string(),
                "Check virtual environment status".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            meta,
        })
    }

    fn build_create_venv_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-python".to_string(),
                description: "Check if Python is installed".to_string(),
                command: "which python".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
        ];

        let venv_name = "myproject_venv";

        let command_plan = vec![
            CommandStep {
                id: "create-venv".to_string(),
                description: format!("Create virtual environment '{}'", venv_name),
                command: format!("python -m venv {}", venv_name),
                risk_level: RiskLevel::Low,
                rollback_id: Some("remove-venv".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "show-activation".to_string(),
                description: "Show activation instructions".to_string(),
                command: format!("echo 'To activate: source {}/bin/activate'", venv_name),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![
            RollbackStep {
                id: "remove-venv".to_string(),
                description: format!("Remove virtual environment '{}'", venv_name),
                command: format!("rm -rf {}", venv_name),
            },
        ];

        let notes_for_user = format!(
            "🔧 Creating Python Virtual Environment\n\n\
             Virtual environment name: {}\n\
             Location: ./{}/\n\n\
             A virtual environment is an isolated Python environment where you can:\n\
             - Install packages without affecting system Python\n\
             - Have different package versions per project\n\
             - Avoid dependency conflicts\n\n\
             **How to use**:\n\
             1. Activate the environment:\n\
                source {}/bin/activate\n\n\
             2. Install packages (pip install will go into venv):\n\
                pip install requests numpy pandas\n\n\
             3. Deactivate when done:\n\
                deactivate\n\n\
             **Project workflow**:\n\
             - Create requirements.txt: pip freeze > requirements.txt\n\
             - Install from requirements: pip install -r requirements.txt\n\
             - Add venv to .gitignore: echo '{}/' >> .gitignore\n\n\
             To remove this venv:\n\
             rm -rf {}",
            venv_name, venv_name, venv_name, venv_name, venv_name
        );

        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("python.rs".to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("python_create_venv".to_string()),
            llm_version: "deterministic_recipe_v1".to_string(),
        };

        Ok(ActionPlan {
            analysis: format!("User wants to create a Python virtual environment. This will create an isolated Python environment in the current directory as '{}'.", venv_name),
            goals: vec![
                format!("Create virtual environment '{}'", venv_name),
                "Show activation instructions".to_string(),
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
    fn test_matches_python_requests() {
        // Should match
        assert!(PythonRecipe::matches_request("install Python"));
        assert!(PythonRecipe::matches_request("setup Python development environment"));
        assert!(PythonRecipe::matches_request("install pip"));
        assert!(PythonRecipe::matches_request("install Python development tools"));
        assert!(PythonRecipe::matches_request("check Python status"));
        assert!(PythonRecipe::matches_request("create Python virtual environment"));
        assert!(PythonRecipe::matches_request("create venv"));

        // Should NOT match
        assert!(!PythonRecipe::matches_request("what is Python"));
        assert!(!PythonRecipe::matches_request("tell me about pip"));
        assert!(!PythonRecipe::matches_request("install docker"));
        assert!(!PythonRecipe::matches_request("how much RAM do I have"));
    }

    #[test]
    fn test_operation_detection() {
        assert_eq!(
            PythonRecipe::detect_operation("install Python"),
            PythonOperation::Install
        );
        assert_eq!(
            PythonRecipe::detect_operation("install Python development tools"),
            PythonOperation::InstallTools
        );
        assert_eq!(
            PythonRecipe::detect_operation("check Python status"),
            PythonOperation::CheckStatus
        );
        assert_eq!(
            PythonRecipe::detect_operation("create Python virtual environment"),
            PythonOperation::CreateVenv
        );
    }

    #[test]
    fn test_install_plan() {
        let mut telemetry = HashMap::new();
        telemetry.insert("internet_connected".to_string(), "true".to_string());

        let plan = PythonRecipe::build_install_plan(&telemetry).unwrap();

        assert_eq!(plan.goals.len(), 4);
        assert!(plan.analysis.contains("Python"));
        assert!(plan.command_plan.len() >= 3);
        assert!(plan.command_plan[0].command.contains("python"));
        assert_eq!(plan.command_plan[0].risk_level, RiskLevel::Medium);
    }

    #[test]
    fn test_install_tools_plan() {
        let mut telemetry = HashMap::new();
        telemetry.insert("internet_connected".to_string(), "true".to_string());

        let plan = PythonRecipe::build_install_tools_plan(&telemetry).unwrap();

        assert!(plan.analysis.contains("tools"));
        assert!(plan.command_plan.iter().any(|c| c.command.contains("black")));
        assert!(plan.command_plan.iter().any(|c| c.command.contains("pylint")));
        assert!(plan.command_plan.iter().any(|c| c.command.contains("mypy")));
        assert!(plan.command_plan.iter().any(|c| c.command.contains("pytest")));
    }

    #[test]
    fn test_check_status_plan() {
        let telemetry = HashMap::new();
        let plan = PythonRecipe::build_check_status_plan(&telemetry).unwrap();

        assert_eq!(plan.command_plan[0].risk_level, RiskLevel::Info);
        assert!(!plan.command_plan[0].requires_confirmation);
        assert!(plan.notes_for_user.contains("Status"));
    }

    #[test]
    fn test_create_venv_plan() {
        let telemetry = HashMap::new();
        let plan = PythonRecipe::build_create_venv_plan(&telemetry).unwrap();

        assert!(plan.command_plan[0].command.contains("venv"));
        assert!(plan.notes_for_user.contains("activate"));
    }

    #[test]
    fn test_no_internet_warning() {
        let mut telemetry = HashMap::new();
        telemetry.insert("internet_connected".to_string(), "false".to_string());

        let plan = PythonRecipe::build_install_plan(&telemetry).unwrap();

        assert!(plan.notes_for_user.contains("No internet detected"));
    }
}
