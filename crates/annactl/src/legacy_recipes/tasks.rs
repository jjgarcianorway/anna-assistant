// Beta.172: Task Management Application Tools Recipe
use anna_common::action_plan_v3::{ActionPlan, CommandStep, RiskLevel, RollbackStep};
use anyhow::Result;
use std::collections::HashMap;

pub struct TasksRecipe;

#[derive(Debug, PartialEq)]
enum TasksOperation {
    Install,
    CheckStatus,
    ListTools,
}

impl TasksOperation {
    fn detect(user_input: &str) -> Self {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("check") || input_lower.contains("status") {
            TasksOperation::CheckStatus
        } else if input_lower.contains("list") || input_lower.contains("show") {
            TasksOperation::ListTools
        } else {
            TasksOperation::Install
        }
    }
}

impl TasksRecipe {
    pub fn matches_request(user_input: &str) -> bool {
        let input_lower = user_input.to_lowercase();
        let has_context = input_lower.contains("taskwarrior") || input_lower.contains("todoman")
            || input_lower.contains("gnome-todo") || input_lower.contains("task warrior")
            || input_lower.contains("task management") || input_lower.contains("todo app");
        let has_action = input_lower.contains("install") || input_lower.contains("setup")
            || input_lower.contains("check") || input_lower.contains("list")
            || input_lower.contains("configure");
        let is_info_only = (input_lower.starts_with("what is")
            || input_lower.starts_with("tell me about")) && !has_action;
        has_context && has_action && !is_info_only
    }

    pub fn build_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let operation = TasksOperation::detect(user_input);
        match operation {
            TasksOperation::Install => Self::build_install_plan(telemetry),
            TasksOperation::CheckStatus => Self::build_check_status_plan(telemetry),
            TasksOperation::ListTools => Self::build_list_tools_plan(telemetry),
        }
    }

    fn detect_tool(user_input: &str) -> &str {
        let input_lower = user_input.to_lowercase();
        if input_lower.contains("taskwarrior") || input_lower.contains("task warrior") { "taskwarrior" }
        else if input_lower.contains("todoman") { "todoman" }
        else if input_lower.contains("gnome-todo") { "gnome-todo" }
        else if input_lower.contains("taskell") { "taskell" }
        else { "taskwarrior" }
    }

    fn build_install_plan(telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let user_input = telemetry.get("user_request").map(|s| s.as_str()).unwrap_or("");
        let tool = Self::detect_tool(user_input);
        let (tool_name, package_name, description) = match tool {
            "taskwarrior" => ("Taskwarrior", "task", "Command-line task management with powerful filtering and GTD support"),
            "todoman" => ("todoman", "todoman", "Terminal-based todo manager with CalDAV sync support"),
            "gnome-todo" => ("GNOME To Do", "gnome-todo", "Simple GNOME todo list application"),
            "taskell" => ("Taskell", "taskell", "Terminal-based Kanban board task manager"),
            _ => ("Taskwarrior", "task", "Command-line task management tool"),
        };

        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("tasks.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("Install"));
        meta_other.insert("tool".to_string(), serde_json::json!(tool_name));

        let install_cmd = format!("sudo pacman -S --needed --noconfirm {}", package_name);
        let notes = format!("{} installed. {}. Run '{}' to start managing tasks.",
            tool_name, description, package_name);

        Ok(ActionPlan {
            analysis: format!("Installing {} task management application", tool_name),
            goals: vec![format!("Install {}", tool_name)],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: format!("install-{}", tool),
                    description: format!("Install {}", tool_name),
                    command: install_cmd,
                    risk_level: RiskLevel::Low,
                    rollback_id: Some(format!("remove-{}", tool)),
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![
                RollbackStep {
                    id: format!("remove-{}", tool),
                    description: format!("Remove {}", tool_name),
                    command: format!("sudo pacman -Rns --noconfirm {}", package_name),
                },
            ],
            notes_for_user: notes,
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("tasks_install".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_check_status_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("tasks.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("CheckStatus"));

        Ok(ActionPlan {
            analysis: "Checking task management application tools".to_string(),
            goals: vec!["List installed task management apps".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "list-tasks-tools".to_string(),
                    description: "List task management apps".to_string(),
                    command: "pacman -Q task todoman gnome-todo taskell 2>/dev/null || echo 'No task management apps installed'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Shows installed task management application tools".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("tasks_check_status".to_string()),
                llm_version: "deterministic_recipe_v1".to_string(),
            },
        })
    }

    fn build_list_tools_plan(_telemetry: &HashMap<String, String>) -> Result<ActionPlan> {
        let mut meta_other = HashMap::new();
        meta_other.insert("recipe_module".to_string(), serde_json::json!("tasks.rs"));
        meta_other.insert("operation".to_string(), serde_json::json!("ListTools"));

        Ok(ActionPlan {
            analysis: "Showing available task management application tools".to_string(),
            goals: vec!["List available task management apps".to_string()],
            necessary_checks: vec![],
            command_plan: vec![
                CommandStep {
                    id: "show-tools".to_string(),
                    description: "Show available tools".to_string(),
                    command: r"echo 'Task Management Applications:

Terminal/CLI Task Managers:
- Taskwarrior (official) - Powerful task management with GTD, filters, reports, hooks
- todoman (official) - CalDAV-based todo manager, simple and standards-compliant
- taskell (AUR) - Terminal Kanban board with boards, cards, drag-drop
- t (AUR) - Minimalist command-line task manager
- todo.txt-cli (AUR) - Plain-text todo manager following todo.txt format

Desktop GUI Task Managers:
- GNOME To Do (official) - Simple todo app with Evolution Data Server integration
- Endeavour (AUR) - GNOME To Do fork with additional features
- Planner (AUR) - Project and task manager with Gantt charts
- Zanshin (official) - KDE task manager based on Akonadi
- Organice (AUR) - Org-mode compatible web-based task manager

Kanban Boards:
- Planka (AUR) - Self-hosted Trello alternative
- Wekan (AUR) - Open-source Kanban board
- Focalboard (AUR) - Project management with Kanban, table, gallery views

GTD (Getting Things Done):
- Taskwarrior - Best CLI GTD implementation with contexts, projects, tags
- GTG (Getting Things GNOME) (official) - Desktop GTD application
- Org-mode (via Emacs) - Text-based GTD in Emacs

Comparison:
- Taskwarrior: Best for CLI power users, complex workflows, scripting
- todoman: Best for CalDAV sync, simple terminal usage
- GNOME To Do: Best for GNOME desktop users, simple tasks
- taskell: Best for Kanban-style project management in terminal

Features:
- Taskwarrior: Tags, projects, contexts, priorities, dependencies, recurring tasks, hooks
- todoman: CalDAV sync, multiple todo lists, simple interface
- GNOME To Do: Evolution integration, online accounts, simple UI
- taskell: Boards, lists, cards, markdown descriptions, vim keybindings

Sync Support:
- Taskwarrior: Taskserver (taskd), inthe.am web service, custom sync
- todoman: CalDAV servers (Nextcloud, Radicale, etc.)
- GNOME To Do: Evolution Data Server, online accounts
- Org-mode: Git, Dropbox, Syncthing for plain-text files

CLI vs GUI:
- CLI (Taskwarrior, todoman): Fast, scriptable, works over SSH
- GUI (GNOME To Do, Planner): Visual, mouse-friendly, drag-and-drop
- Hybrid (taskell): Terminal UI with visual Kanban board'".to_string(),
                    risk_level: RiskLevel::Info,
                    rollback_id: None,
                    requires_confirmation: false,
                },
            ],
            rollback_plan: vec![],
            notes_for_user: "Task management applications for Arch Linux".to_string(),
            meta: anna_common::action_plan_v3::PlanMeta {
                detection_results: anna_common::action_plan_v3::DetectionResults {
                    de: None, wm: None, wallpaper_backends: vec![], display_protocol: None, other: meta_other,
                },
                template_used: Some("tasks_list_tools".to_string()),
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
        assert!(TasksRecipe::matches_request("install taskwarrior"));
        assert!(TasksRecipe::matches_request("install task management app"));
        assert!(TasksRecipe::matches_request("setup todoman"));
        assert!(!TasksRecipe::matches_request("what is taskwarrior"));
    }
    #[test]
    fn test_install_plan() {
        let mut t = HashMap::new();
        t.insert("user_request".to_string(), "install task warrior".to_string());
        let plan = TasksRecipe::build_install_plan(&t).unwrap();
        assert!(!plan.command_plan.is_empty());
    }
}
