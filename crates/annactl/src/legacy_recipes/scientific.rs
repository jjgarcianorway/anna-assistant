// Beta.174: Scientific Computing Tools Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct ScientificRecipe;

#[derive(Debug, PartialEq)]
enum ScientificOperation {
    Install,
    CheckStatus,
    ListTools,
}

impl ScientificOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("check") || input_lower.contains("status") {
            ScientificOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            ScientificOperation::ListTools
        } else {
            ScientificOperation::Install
        }
    }
}

impl ScientificRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("octave") || input_lower.contains("maxima")
            || input_lower.contains("sagemath") || input_lower.contains("sage math")
            || input_lower.contains("rstudio") || input_lower.contains("r studio")
            || input_lower.contains("scientific computing") || input_lower.contains("matlab alternative");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("configure");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = ScientificOperation::detect(user_input);
        match operation {
            ScientificOperation::Install => Self::build_install_plan(telemetry),
            ScientificOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            ScientificOperation::ListTools => Self::build_list_tools_plan(telemetry),
        }
    }

    fn detect_tool(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("octave") { "octave" }
        else if input_lower.contains("maxima") { "maxima" }
        else if input_lower.contains("sagemath") || input_lower.contains("sage") { "sagemath" }
        else if input_lower.contains("rstudio") { "rstudio" }
        else { "octave" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let tool = Self::detect_tool(user_input);
        let (tool_name, package_name, description, is_aur) = match tool {
            "octave" => ("GNU Octave", "octave", "MATLAB-compatible numerical computation with plotting", false),
            "maxima" => ("Maxima", "maxima", "Computer algebra system for symbolic computation", false),
            "sagemath" => ("SageMath", "sagemath", "Open-source mathematics software system", true),
            "rstudio" => ("RStudio Desktop", "rstudio-desktop-bin", "IDE for R statistical computing", true),
            _ => ("GNU Octave", "octave", "Scientific computing environment", false),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("scientific.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("tool".to_string(), serde_json::json!(tool_name));

        let install_cmd = if is_aur {
            format!("yay -S --needed --noconfirm {} || paru -S --needed --noconfirm {}", package_name, package_name)
        } else {
            format!("sudo pacman -S --needed --noconfirm {}", package_name)
        };

        let notes = format!("{} installed. {}. Launch from app menu or run '{}' in terminal.",
            tool_name, description, if tool == "rstudio" { "rstudio" } else { tool });

        let risk_level = if is_aur { RiskLevel::Medium } else { RiskLevel::Low };
        let requires_confirmation = is_aur;

        Ok(ActionPlan {
            analysis: format!("Installing {} scientific computing tool", tool_name),
            goals: vec![format!("Install {}", tool_name)],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: format!("install-{}", tool),
                    description: format!("Install {}", tool_name),
                    command: install_cmd,
                    risk_level,
                    rollback_id: Some(format!("remove-{}", tool)),
                    requires_confirmation,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: format!("remove-{}", tool),
                    description: format!("Remove {}", tool_name),
                    command: if is_aur {
                        format!("yay -Rns --noconfirm {} || paru -Rns --noconfirm {}", package_name, package_name)
                    } else {
                        format!("sudo pacman -Rns --noconfirm {}", package_name)
                    },
                },
            ],
            notes_for_user: notes,
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("scientific_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("scientific.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking scientific computing tools".to_string(),
            goals: vec!["List installed scientific software".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-scientific-tools".to_string(),
                    description: "List scientific tools".to_string(),
                    command: "pacman -Q octave maxima sagemath rstudio-desktop-bin r 2>/dev/null || echo 'No scientific computing tools installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed scientific computing software".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("scientific_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_tools_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("scientific.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListTools"));

        Ok(ActionPlan {
            analysis: "Showing available scientific computing tools".to_string(),
            goals: vec!["List available scientific software".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-tools".to_string(),
                    description: "Show available tools".to_string(),
                    command: r"echo 'Scientific Computing Tools:

MATLAB-Compatible:
- GNU Octave (official) - High-level language for numerical computation, MATLAB-compatible
- FreeMat (AUR) - Open-source numerical computing environment similar to MATLAB
- Scilab (AUR) - Open platform for numerical computation and simulation

Computer Algebra Systems:
- Maxima (official) - Symbolic computation system derived from Macsyma
- wxMaxima (official) - Graphical interface for Maxima with 2D/3D plotting
- Reduce (AUR) - Portable computer algebra system
- Axiom (AUR) - General-purpose computer algebra system

General Mathematics:
- SageMath (AUR) - Open-source mathematics software combining many packages
- SymPy (pip) - Python library for symbolic mathematics
- GAP (official) - System for computational discrete algebra

Statistical Computing:
- R (official) - Language and environment for statistical computing
- RStudio Desktop (AUR) - IDE for R programming with data visualization
- RKWard (official) - KDE frontend for R statistical software
- PSPP (official) - Free alternative to IBM SPSS statistics

Data Analysis:
- Julia (official) - High-performance language for technical computing
- Orange (AUR) - Data mining and visualization tool
- RapidMiner (AUR) - Data science platform
- KNIME (AUR) - Analytics and reporting platform

Python Scientific Stack:
- NumPy (pip) - Numerical computing with arrays
- SciPy (pip) - Scientific and technical computing
- Pandas (pip) - Data analysis and manipulation
- Matplotlib (pip) - Plotting and visualization
- Jupyter (pip) - Interactive notebooks

Plotting/Visualization:
- Gnuplot (official) - Command-line driven graphing utility
- ParaView (official) - Data analysis and visualization application
- VisIt (AUR) - Interactive visualization and analysis tool
- Grace (official) - WYSIWYG 2D plotting tool

Specialized Tools:
- Geogebra (AUR) - Dynamic mathematics software for education
- Cadabra (AUR) - Computer algebra system for field theory
- Yacas (AUR) - Yet Another Computer Algebra System

Comparison:
- GNU Octave: Best MATLAB alternative, compatible with MATLAB code
- SageMath: Best all-in-one mathematics system, combines many tools
- Maxima: Best for symbolic computation and algebra
- R/RStudio: Best for statistics and data analysis

Features:
- Octave: Matrix operations, plotting, MATLAB syntax, function library, scripting
- Maxima: Symbolic integration, differentiation, Taylor series, Laplace transforms
- SageMath: Combines GAP, Maxima, R, NumPy, and 100+ packages
- RStudio: Code editor, console, plots, workspace, debugging, packages

MATLAB Compatibility:
- Octave: High compatibility, runs most MATLAB code
- FreeMat: Moderate compatibility with MATLAB syntax
- Scilab: Similar syntax but not fully compatible

Package Management:
- Octave: Use pkg install for Octave packages
- R: Use install.packages() for R packages
- SageMath: Includes most packages by default
- Python: Use pip for Python scientific packages

Performance:
- Julia: Fastest for numerical computation
- Octave: Good performance, optimized for matrix operations
- R: Excellent for statistics, slower for general computation
- Maxima: Optimized for symbolic computation

Use Cases:
- Engineering/Physics: Octave for numerical computation, Maxima for symbolic
- Statistics/Data Science: R with RStudio for analysis and visualization
- Research Mathematics: SageMath for comprehensive toolset
- Education: Geogebra for interactive geometry, Octave for computation

Learning Resources:
- Octave: Compatible with MATLAB tutorials and documentation
- R: CRAN documentation, RStudio tutorials
- Maxima: Maxima manual, wxMaxima for interactive learning
- SageMath: Comprehensive documentation and tutorials

IDE/Interface Options:
- Octave: Command-line, Octave GUI, or VS Code with extension
- Maxima: wxMaxima graphical interface
- R: RStudio, RKWard, or command-line
- Julia: Juno IDE, Jupyter notebooks, VS Code

Configuration:
- Octave: ~/.octaverc for startup configuration
- R: ~/.Rprofile for startup options
- Maxima: ~/.maxima-init.mac for initialization
- RStudio: Tools → Global Options for IDE settings'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Scientific computing tools for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("scientific_list_tools".to_string()),
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
        assert!(ScientificRecipe::matches_request("install octave"));
        assert!(ScientificRecipe::matches_request("install scientific computing"));
        assert!(ScientificRecipe::matches_request("setup sagemath"));
        assert!(!ScientificRecipe::matches_request("what is octave"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install maxima".to_string());
        let plan = ScientificRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
