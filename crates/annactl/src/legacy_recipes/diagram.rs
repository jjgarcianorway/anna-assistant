// Beta.172: Diagram and Flowchart Tools Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct DiagramRecipe;

#[derive(Debug, PartialEq)]
enum DiagramOperation {
    Install,
    CheckStatus,
    ListTools,
}

impl DiagramOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("check") || input_lower.contains("status") {
            DiagramOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            DiagramOperation::ListTools
        } else {
            DiagramOperation::Install
        }
    }
}

impl DiagramRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("draw.io") || input_lower.contains("drawio")
            || input_lower.contains("dia") || input_lower.contains("graphviz")
            || input_lower.contains("plantuml") || input_lower.contains("diagram tool")
            || input_lower.contains("flowchart");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("configure");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = DiagramOperation::detect(user_input);
        match operation {
            DiagramOperation::Install => Self::build_install_plan(telemetry),
            DiagramOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            DiagramOperation::ListTools => Self::build_list_tools_plan(telemetry),
        }
    }

    fn detect_tool(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("draw.io") || input_lower.contains("drawio") { "drawio" }
        else if input_lower.contains("dia") { "dia" }
        else if input_lower.contains("graphviz") { "graphviz" }
        else if input_lower.contains("plantuml") { "plantuml" }
        else { "graphviz" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let tool = Self::detect_tool(user_input);
        let (tool_name, package_name, description, is_aur) = match tool {
            "drawio" => ("draw.io Desktop", "drawio-desktop", "Diagram editor for flowcharts, UML, network diagrams", true),
            "dia" => ("Dia", "dia", "GTK-based diagram creation program for flowcharts and UML", false),
            "graphviz" => ("Graphviz", "graphviz", "Graph visualization software with DOT language", false),
            "plantuml" => ("PlantUML", "plantuml", "Create UML diagrams from plain text descriptions", false),
            _ => ("Graphviz", "graphviz", "Graph visualization software", false),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("diagram.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("tool".to_string(), serde_json::json!(tool_name));

        let install_cmd = if is_aur {
            format!("yay -S --needed --noconfirm {} || paru -S --needed --noconfirm {}", package_name, package_name)
        } else {
            format!("sudo pacman -S --needed --noconfirm {}", package_name)
        };

        let notes = if is_aur {
            format!("{} installed. {}. Launch from app menu.", tool_name, description)
        } else {
            format!("{} installed. {}. Run '{}' or launch from app menu.", tool_name, description, package_name)
        };

        let risk_level = if is_aur { RiskLevel::Medium } else { RiskLevel::Low };
        let requires_confirmation = is_aur;

        Ok(ActionPlan {
            analysis: format!("Installing {} diagram tool", tool_name),
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
                template_used: Some("diagram_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("diagram.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking diagram and flowchart tools".to_string(),
            goals: vec!["List installed diagram tools".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-diagram-tools".to_string(),
                    description: "List diagram tools".to_string(),
                    command: "pacman -Q drawio-desktop dia graphviz plantuml 2>/dev/null || echo 'No diagram tools installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed diagram and flowchart tools".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("diagram_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_tools_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("diagram.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListTools"));

        Ok(ActionPlan {
            analysis: "Showing available diagram and flowchart tools".to_string(),
            goals: vec!["List available diagram tools".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-tools".to_string(),
                    description: "Show available tools".to_string(),
                    command: r"echo 'Diagram and Flowchart Tools:

GUI Diagram Editors:
- draw.io Desktop (AUR) - Powerful flowchart and diagram editor (formerly diagrams.net)
- Dia (official) - GTK-based diagram creation for flowcharts, UML, network diagrams
- yEd (AUR) - Java-based diagram editor with automatic layout
- Pencil (AUR) - GUI prototyping and diagramming tool
- Umbrello (official) - KDE UML modeler

Text-Based Diagram Tools:
- Graphviz (official) - Graph visualization using DOT language
- PlantUML (official) - Create UML diagrams from plain text
- Mermaid (via npm) - Generate diagrams from markdown-like text
- ditaa (AUR) - Convert ASCII diagrams into bitmap graphics
- blockdiag (AUR) - Generate block diagrams from text

UML and Modeling:
- PlantUML (official) - UML, sequence, class, activity, component diagrams
- Umbrello (official) - UML diagrams for software modeling
- StarUML (AUR) - Sophisticated UML modeling tool
- ArgoUML (AUR) - Java-based UML modeling

Flowcharts and Process Diagrams:
- draw.io Desktop - Comprehensive flowchart tool with many templates
- Dia - Classic flowchart creation
- yEd - Automatic layout for complex flowcharts

Network Diagrams:
- draw.io Desktop - Network topology diagrams with icons
- Dia - Network diagram templates
- Graphviz - Automated network graph layouts

Comparison:
- draw.io: Best all-around tool, web-based UI, many templates
- Graphviz: Best for automated layouts from code/scripts
- PlantUML: Best for developers who want diagrams-as-code
- Dia: Best lightweight traditional diagramming tool

Features:
- draw.io: Templates, shapes library, export to PNG/SVG/PDF, integrations
- Graphviz: DOT language, automatic graph layout algorithms, scriptable
- PlantUML: Plain text syntax, version control friendly, IDE integration
- Dia: Simple interface, custom shapes, Python scripting

Use Cases:
- draw.io: Business flowcharts, org charts, infographics, presentations
- Graphviz: Software architecture, dependency graphs, state machines
- PlantUML: UML diagrams in documentation, sequence diagrams for APIs
- Dia: Quick technical diagrams, circuit diagrams, network topology

Export Formats:
- draw.io: PNG, JPEG, SVG, PDF, HTML, XML, VSDX
- Graphviz: PNG, PDF, SVG, PostScript
- PlantUML: PNG, SVG, LaTeX, ASCII art
- Dia: PNG, SVG, EPS, PDF, XFIG'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Diagram and flowchart tools for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("diagram_list_tools".to_string()),
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
        assert!(DiagramRecipe::matches_request("install draw.io"));
        assert!(DiagramRecipe::matches_request("install diagram tool"));
        assert!(DiagramRecipe::matches_request("setup graphviz"));
        assert!(!DiagramRecipe::matches_request("what is plantuml"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install dia".to_string());
        let plan = DiagramRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
