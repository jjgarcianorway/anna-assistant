// nodejs.rs - Node.js development environment recipe
// Beta.154: Node.js and npm setup

use anna_common::action_plan_v3::{
    ActionPlan, CommandStep, DetectionResults, NecessaryCheck, PlanMeta, RiskLevel, RollbackStep,
};
use anyhow::Result;
use serde_json;
use std::collections::HashMap;

pub struct NodeJsRecipe;

#[derive(Debug, PartialEq)]
enum NodeJsOperation {
    Install,           // Install Node.js and npm
    InstallTools,      // Install development tools (typescript, eslint, etc.)
    CheckStatus,       // Check Node.js installation status
    InitProject,       // Initialize new npm project
}

impl NodeJsRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();

        // Exclude informational queries
        if input_lower.contains("what is")
            || input_lower.contains("tell me about")
            || input_lower.contains("explain")
        {
            return false;
        }

        // Node.js-related keywords
        let has_nodejs_context = input_lower.contains("node")
            || input_lower.contains("nodejs")
            || input_lower.contains("npm")
            || input_lower.contains("javascript");

        // Action keywords
        let has_action = input_lower.contains("install")
            || input_lower.contains("setup")
            || input_lower.contains("check")
            || input_lower.contains("status")
            || input_lower.contains("init")
            || input_lower.contains("initialize")
            || input_lower.contains("configure");

        has_nodejs_context && has_action
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_request = telemetry
            .get("user_request")
            .map(|s| s.as_str())
            .unwrap_or("");

        let operation = Self::detect_operation(user_request);

        match operation {
            NodeJsOperation::Install => Self::build_install_plan(telemetry),
            NodeJsOperation::InstallTools => Self::build_install_tools_plan(telemetry),
            NodeJsOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            NodeJsOperation::InitProject => Self::build_init_project_plan(telemetry),
        }
    }

    fn detect_operation(user_input: &str) -> NodeJsOperation {
        let input_lower = user_input.to_lowercase();

        // Check for project initialization
        if input_lower.contains("init") || input_lower.contains("initialize") {
            return NodeJsOperation::InitProject;
        }

        // Check for status
        if input_lower.contains("check") || input_lower.contains("status") {
            return NodeJsOperation::CheckStatus;
        }

        // Check for tools installation
        if (input_lower.contains("tool") || input_lower.contains("typescript") || input_lower.contains("eslint"))
            && (input_lower.contains("install") || input_lower.contains("setup"))
        {
            return NodeJsOperation::InstallTools;
        }

        // Default to install
        NodeJsOperation::Install
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let has_internet = telemetry
            .get("internet_connected")
            .map(|v| v == "true")
            .unwrap_or(false);

        let mut warning = String::new();
        if !has_internet {
            warning = "⚠️ No internet detected. Node.js installation requires internet connection.\n\n"
                .to_string();
        }

        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-node-installed".to_string(),
                description: "Check if Node.js is already installed".to_string(),
                command: "node --version".to_string(),
                risk_level: RiskLevel::Info,
                required: false,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "install-nodejs".to_string(),
                description: "Install Node.js and npm".to_string(),
                command: "sudo pacman -S --noconfirm nodejs npm".to_string(),
                risk_level: RiskLevel::Medium,
                rollback_id: Some("uninstall-nodejs".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "verify-node".to_string(),
                description: "Verify Node.js installation".to_string(),
                command: "node --version".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "verify-npm".to_string(),
                description: "Verify npm installation".to_string(),
                command: "npm --version".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "configure-npm-prefix".to_string(),
                description: "Configure npm to install global packages in user directory".to_string(),
                command: "mkdir -p ~/.npm-global && npm config set prefix ~/.npm-global".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![
            RollbackStep {
                id: "uninstall-nodejs".to_string(),
                description: "Uninstall Node.js and npm".to_string(),
                command: "sudo pacman -Rns nodejs npm".to_string(),
            },
        ];

        let notes_for_user = format!(
            "{}📦 Installing Node.js Development Environment\n\n\
             This will install:\n\
             - Node.js: JavaScript runtime (latest LTS from Arch repos)\n\
             - npm: Node package manager\n\n\
             Installation locations:\n\
             - Node.js: /usr/bin/node\n\
             - npm: /usr/bin/npm\n\
             - Global packages: ~/.npm-global/\n\n\
             ⚠️ IMPORTANT:\n\
             1. Add ~/.npm-global/bin to your PATH:\n\
                echo 'export PATH=~/.npm-global/bin:$PATH' >> ~/.bashrc\n\
                source ~/.bashrc\n\n\
             2. Global packages will install to ~/.npm-global/ (no sudo needed)\n\n\
             3. Project packages install to ./node_modules/ (per-project)\n\n\
             Common next steps:\n\
             - Install dev tools: `annactl \"install Node.js development tools\"`\n\
             - Initialize project: `annactl \"initialize npm project\"`\n\
             - Install package: `npm install express`\n\
             - Install globally: `npm install -g typescript`\n\n\
             Documentation: https://nodejs.org/docs/",
            warning
        );

        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("nodejs.rs".to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("nodejs_install".to_string()),
            llm_version: "deterministic_recipe_v1".to_string(),
        };

        Ok(ActionPlan {
            analysis: "User wants to install Node.js development environment. This will install Node.js runtime and npm package manager from Arch repositories.".to_string(),
            goals: vec![
                "Install Node.js runtime".to_string(),
                "Install npm package manager".to_string(),
                "Configure npm for user-level global packages".to_string(),
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
                id: "check-npm".to_string(),
                description: "Check if npm is installed".to_string(),
                command: "which npm".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "install-typescript".to_string(),
                description: "Install TypeScript compiler".to_string(),
                command: "npm install -g typescript".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "install-eslint".to_string(),
                description: "Install ESLint (linter)".to_string(),
                command: "npm install -g eslint".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "install-prettier".to_string(),
                description: "Install Prettier (code formatter)".to_string(),
                command: "npm install -g prettier".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "install-nodemon".to_string(),
                description: "Install nodemon (auto-restart on changes)".to_string(),
                command: "npm install -g nodemon".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "verify-tools".to_string(),
                description: "Verify tool installation".to_string(),
                command: "tsc --version && eslint --version && prettier --version && nodemon --version".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![
            RollbackStep {
                id: "note-manual-uninstall".to_string(),
                description: "Tools can be uninstalled with: npm uninstall -g <tool>".to_string(),
                command: "echo 'npm uninstall -g typescript eslint prettier nodemon'".to_string(),
            },
        ];

        let notes_for_user = format!(
            "{}🔧 Installing Node.js Development Tools\n\n\
             This will install essential Node.js development tools globally:\n\n\
             **Language Tools**:\n\
             - TypeScript: Typed superset of JavaScript\n\
               Usage: tsc file.ts\n\
               Init: tsc --init\n\n\
             **Code Quality**:\n\
             - ESLint: JavaScript/TypeScript linter\n\
               Usage: eslint .\n\
               Init: eslint --init\n\
             - Prettier: Opinionated code formatter\n\
               Usage: prettier --write .\n\n\
             **Development**:\n\
             - nodemon: Auto-restart on file changes\n\
               Usage: nodemon server.js\n\n\
             Installation location: ~/.npm-global/bin/\n\
             Make sure this is in your PATH!\n\n\
             Common workflows:\n\
             - Compile TypeScript: `tsc`\n\
             - Lint code: `eslint .`\n\
             - Format code: `prettier --write .`\n\
             - Dev server: `nodemon index.js`\n\n\
             To uninstall:\n\
             npm uninstall -g typescript eslint prettier nodemon",
            warning
        );

        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("nodejs.rs".to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("nodejs_install_tools".to_string()),
            llm_version: "deterministic_recipe_v1".to_string(),
        };

        Ok(ActionPlan {
            analysis: "User wants to install essential Node.js development tools including TypeScript, ESLint, Prettier, and nodemon.".to_string(),
            goals: vec![
                "Install TypeScript compiler".to_string(),
                "Install code quality tools (ESLint, Prettier)".to_string(),
                "Install development tools (nodemon)".to_string(),
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
                id: "check-node".to_string(),
                description: "Check Node.js version".to_string(),
                command: "node --version || echo 'Node.js not installed'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "check-npm".to_string(),
                description: "Check npm version".to_string(),
                command: "npm --version || echo 'npm not installed'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "list-global-packages".to_string(),
                description: "List globally installed packages".to_string(),
                command: "npm list -g --depth=0 2>/dev/null | head -20 || echo 'No global packages'".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "npm-config".to_string(),
                description: "Show npm configuration".to_string(),
                command: "npm config get prefix".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
            CommandStep {
                id: "node-path".to_string(),
                description: "Show Node.js executable location".to_string(),
                command: "which node".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![];

        let notes_for_user = "ℹ️ Node.js Development Environment Status\n\n\
             This shows the current state of your Node.js installation:\n\
             - Node.js version\n\
             - npm version and location\n\
             - Globally installed packages (first 20)\n\
             - npm prefix (global package location)\n\
             - Node.js executable path\n\n\
             Common next steps:\n\
             - Install Node.js: `annactl \"install Node.js\"`\n\
             - Install dev tools: `annactl \"install Node.js development tools\"`\n\
             - Initialize project: `annactl \"initialize npm project\"`\n\n\
             Package management:\n\
             - Local packages: npm install <package>\n\
             - Global packages: npm install -g <package>\n\
             - Dev dependencies: npm install --save-dev <package>"
            .to_string();

        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("nodejs.rs".to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("nodejs_check_status".to_string()),
            llm_version: "deterministic_recipe_v1".to_string(),
        };

        Ok(ActionPlan {
            analysis: "User wants to check Node.js installation status. This is a read-only operation showing Node.js version, npm, and installed packages.".to_string(),
            goals: vec![
                "Check Node.js and npm versions".to_string(),
                "List globally installed packages".to_string(),
                "Show npm configuration".to_string(),
            ],
            necessary_checks,
            command_plan,
            rollback_plan,
            notes_for_user,
            meta,
        })
    }

    fn build_init_project_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let necessary_checks = vec![
            NecessaryCheck {
                id: "check-npm".to_string(),
                description: "Check if npm is installed".to_string(),
                command: "which npm".to_string(),
                risk_level: RiskLevel::Info,
                required: true,
            },
        ];

        let command_plan = vec![
            CommandStep {
                id: "npm-init".to_string(),
                description: "Initialize npm project (creates package.json)".to_string(),
                command: "npm init -y".to_string(),
                risk_level: RiskLevel::Low,
                rollback_id: Some("remove-package-json".to_string()),
                requires_confirmation: true,
            },
            CommandStep {
                id: "show-package-json".to_string(),
                description: "Show generated package.json".to_string(),
                command: "cat package.json".to_string(),
                risk_level: RiskLevel::Info,
                rollback_id: None,
                requires_confirmation: false,
            },
        ];

        let rollback_plan = vec![
            RollbackStep {
                id: "remove-package-json".to_string(),
                description: "Remove package.json".to_string(),
                command: "rm -f package.json".to_string(),
            },
        ];

        let notes_for_user = "📦 Initializing npm Project\n\n\
             This creates a package.json file in the current directory with default values.\n\n\
             **package.json** contains:\n\
             - Project metadata (name, version, description)\n\
             - Dependencies and devDependencies\n\
             - Scripts (build, test, start, etc.)\n\
             - Entry point and other configuration\n\n\
             **Next steps**:\n\
             1. Install dependencies:\n\
                npm install express\n\
                npm install --save-dev jest\n\n\
             2. Add scripts to package.json:\n\
                \"scripts\": {\n\
                  \"start\": \"node index.js\",\n\
                  \"dev\": \"nodemon index.js\",\n\
                  \"test\": \"jest\"\n\
                }\n\n\
             3. Run scripts:\n\
                npm start\n\
                npm run dev\n\
                npm test\n\n\
             **Common commands**:\n\
             - Install package: npm install <package>\n\
             - Install dev dependency: npm install --save-dev <package>\n\
             - Remove package: npm uninstall <package>\n\
             - Update packages: npm update\n\
             - Audit security: npm audit\n\n\
             To customize package.json, you can edit it manually or run:\n\
             npm init (interactive mode)"
            .to_string();

        let mut other = HashMap::new();
        other.insert(
            "recipe_module".to_string(),
            serde_json::Value::String("nodejs.rs".to_string()),
        );

        let meta = PlanMeta {
            detection_results: DetectionResults {
                de: None,
                wm: None,
                wallpaper_backends: vec![],
                display_protocol: None,
                other,
            },
            template_used: Some("nodejs_init_project".to_string()),
            llm_version: "deterministic_recipe_v1".to_string(),
        };

        Ok(ActionPlan {
            analysis: "User wants to initialize a new npm project. This will create a package.json file with default configuration.".to_string(),
            goals: vec![
                "Create package.json with defaults".to_string(),
                "Display generated configuration".to_string(),
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
    fn test_matches_nodejs_requests() {
        // Should match
        assert!(NodeJsRecipe::matches_request("install Node.js"));
        assert!(NodeJsRecipe::matches_request("setup Node development environment"));
        assert!(NodeJsRecipe::matches_request("install npm"));
        assert!(NodeJsRecipe::matches_request("install Node.js development tools"));
        assert!(NodeJsRecipe::matches_request("check Node.js status"));
        assert!(NodeJsRecipe::matches_request("initialize npm project"));
        assert!(NodeJsRecipe::matches_request("install JavaScript tools"));

        // Should NOT match
        assert!(!NodeJsRecipe::matches_request("what is Node.js"));
        assert!(!NodeJsRecipe::matches_request("tell me about npm"));
        assert!(!NodeJsRecipe::matches_request("install docker"));
        assert!(!NodeJsRecipe::matches_request("how much RAM do I have"));
    }

    #[test]
    fn test_operation_detection() {
        assert_eq!(
            NodeJsRecipe::detect_operation("install Node.js"),
            NodeJsOperation::Install
        );
        assert_eq!(
            NodeJsRecipe::detect_operation("install Node.js development tools"),
            NodeJsOperation::InstallTools
        );
        assert_eq!(
            NodeJsRecipe::detect_operation("check Node.js status"),
            NodeJsOperation::CheckStatus
        );
        assert_eq!(
            NodeJsRecipe::detect_operation("initialize npm project"),
            NodeJsOperation::InitProject
        );
    }

    #[test]
    fn test_install_plan() {
        let mut telemetry = HashMap::new();
        telemetry.insert("internet_connected".to_string(), "true".to_string());

        let plan = NodeJsRecipe::build_install_plan(&telemetry).unwrap();

        assert_eq!(plan.goals.len(), 4);
        assert!(plan.analysis.contains("Node.js"));
        assert!(plan.command_plan.len() >= 3);
        assert!(plan.command_plan[0].command.contains("nodejs"));
        assert_eq!(plan.command_plan[0].risk_level, RiskLevel::Medium);
    }

    #[test]
    fn test_install_tools_plan() {
        let mut telemetry = HashMap::new();
        telemetry.insert("internet_connected".to_string(), "true".to_string());

        let plan = NodeJsRecipe::build_install_tools_plan(&telemetry).unwrap();

        assert!(plan.analysis.contains("tools"));
        assert!(plan.command_plan.iter().any(|c| c.command.contains("typescript")));
        assert!(plan.command_plan.iter().any(|c| c.command.contains("eslint")));
        assert!(plan.command_plan.iter().any(|c| c.command.contains("prettier")));
        assert!(plan.command_plan.iter().any(|c| c.command.contains("nodemon")));
    }

    #[test]
    fn test_check_status_plan() {
        let telemetry = HashMap::new();
        let plan = NodeJsRecipe::build_check_status_plan(&telemetry).unwrap();

        assert_eq!(plan.command_plan[0].risk_level, RiskLevel::Info);
        assert!(!plan.command_plan[0].requires_confirmation);
        assert!(plan.notes_for_user.contains("Status"));
    }

    #[test]
    fn test_init_project_plan() {
        let telemetry = HashMap::new();
        let plan = NodeJsRecipe::build_init_project_plan(&telemetry).unwrap();

        assert!(plan.command_plan[0].command.contains("npm init"));
        assert!(plan.notes_for_user.contains("package.json"));
    }

    #[test]
    fn test_no_internet_warning() {
        let mut telemetry = HashMap::new();
        telemetry.insert("internet_connected".to_string(), "false".to_string());

        let plan = NodeJsRecipe::build_install_plan(&telemetry).unwrap();

        assert!(plan.notes_for_user.contains("No internet detected"));
    }
}
