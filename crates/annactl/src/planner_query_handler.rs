//! Planner Query Handler - Routes queries through Planner → Executor → Interpreter
//!
//! v6.41.0: This is the new core architecture for handling inspection queries.
//! Instead of hard-coded handlers, we use LLM-driven planning and execution.

use anna_common::{
    executor_core, interpreter_core, planner_core, trace_renderer,
    telemetry::SystemTelemetry,
};
use anyhow::{anyhow, Result};

/// Check if query should use planner core
pub fn should_use_planner(query: &str) -> bool {
    let query_lower = query.to_lowercase();

    // Pilot queries for v6.41.0
    let pilot_patterns = [
        "do i have games",
        "do i have any wm",
        "do i have any de",
        "what de",
        "what wm",
        "desktop environment",
        "window manager",
        "does my cpu have",
        "cpu feature",
        "do i have any file manager",
        "do i have a file manager",
    ];

    pilot_patterns.iter().any(|pattern| query_lower.contains(pattern))
}

/// Handle query using Planner → Executor → Interpreter core
pub async fn handle_with_planner(
    query: &str,
    telemetry: &SystemTelemetry,
    llm_client: Option<&str>, // URL to LLM service
) -> Result<String> {

    // Step 1: Interpret user intent
    let intent = planner_core::interpret_intent(query);

    // Step 2: Generate command plan (using LLM or fallback)
    let plan = if let Some(_llm_url) = llm_client {
        // TODO: Call LLM to generate plan
        // For now, use fallback planning
        generate_fallback_plan(&intent, telemetry)?
    } else {
        generate_fallback_plan(&intent, telemetry)?
    };

    // Step 3: Execute the plan
    let exec_result = executor_core::execute_plan(&plan)?;

    // Step 4: Interpret results (using LLM or fallback)
    let answer = if let Some(_llm_url) = llm_client {
        // TODO: Call LLM to interpret results
        // For now, use fallback interpretation
        interpreter_core::interpret_without_llm(&intent, &exec_result)
    } else {
        interpreter_core::interpret_without_llm(&intent, &exec_result)
    };

    // Step 5: Render output with trace
    let mut output = String::new();

    // Main answer
    output.push_str(&answer.answer);
    output.push('\n');

    // Show trace always for v6.41.0 (visible thinking traces requirement)
    output.push_str(&trace_renderer::render_trace(&intent, &exec_result, &answer));

    // Source attribution
    output.push_str(&format!("\nSource: {}", answer.source));

    Ok(output)
}

/// Fallback plan generator (deterministic, no LLM)
fn generate_fallback_plan(
    intent: &planner_core::Intent,
    _telemetry: &SystemTelemetry,
) -> Result<planner_core::CommandPlan> {
    use planner_core::{CommandPlan, PlannedCommand, SafetyLevel};

    let mut commands = Vec::new();

    // Generate commands based on domain
    match intent.domain {
        planner_core::DomainType::Packages => {
            // Check if query is about games
            if intent.query.to_lowercase().contains("game") {
                // Use shell piping to filter for game packages
                commands.push(PlannedCommand {
                    command: "sh".to_string(),
                    args: vec![
                        "-c".to_string(),
                        "pacman -Qq | grep -Ei '(steam|game|lutris|heroic|wine|proton)'".to_string(),
                    ],
                    purpose: "Find game-related packages".to_string(),
                    requires_tools: vec!["pacman".to_string(), "grep".to_string()],
                });
            } else if intent.query.to_lowercase().contains("file manager") {
                commands.push(PlannedCommand {
                    command: "sh".to_string(),
                    args: vec![
                        "-c".to_string(),
                        "pacman -Qq | grep -Ei '(thunar|dolphin|nautilus|nemo|pcmanfm|ranger|mc)'".to_string(),
                    ],
                    purpose: "Find file manager packages".to_string(),
                    requires_tools: vec!["pacman".to_string(), "grep".to_string()],
                });
            }
        }
        planner_core::DomainType::Hardware => {
            // CPU features - extract the specific features being asked about
            let features: Vec<String> = intent
                .constraints
                .iter()
                .filter_map(|c| {
                    if let planner_core::Constraint::Feature(f) = c {
                        Some(f.to_uppercase())
                    } else {
                        None
                    }
                })
                .collect();

            if !features.is_empty() {
                // Build a grep pattern for the requested features
                let pattern = features.join("|");
                commands.push(PlannedCommand {
                    command: "sh".to_string(),
                    args: vec![
                        "-c".to_string(),
                        format!("lscpu | grep -i 'flags' || grep -i 'flags' /proc/cpuinfo | head -1"),
                    ],
                    purpose: format!("Get CPU flags to check for: {}", features.join(", ")),
                    requires_tools: vec!["lscpu".to_string(), "grep".to_string()],
                });
            }
        }
        planner_core::DomainType::Gui => {
            // DE/WM detection - use de_wm_detector directly
            // This is a special case where we don't need commands,
            // we'll handle it in the interpreter
            commands.push(PlannedCommand {
                command: "echo".to_string(),
                args: vec!["DE_WM_DETECTOR".to_string()],
                purpose: "Use de_wm_detector module for accurate detection".to_string(),
                requires_tools: vec![],
            });
        }
        _ => {
            return Err(anyhow!("Unsupported domain for fallback planner"));
        }
    }

    if commands.is_empty() {
        return Err(anyhow!("Could not generate command plan"));
    }

    Ok(CommandPlan {
        commands,
        safety_level: SafetyLevel::ReadOnly,
        fallbacks: vec![],
        expected_output: "Command outputs for analysis".to_string(),
        reasoning: "Fallback plan generated without LLM".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_use_planner() {
        assert!(should_use_planner("do I have games installed?"));
        assert!(should_use_planner("what DE am I running?"));
        assert!(should_use_planner("does my CPU have SSE?"));
        assert!(should_use_planner("do I have any file manager?"));

        // Should not use planner
        assert!(!should_use_planner("how much ram do I have?"));
        assert!(!should_use_planner("show me system status"));
    }

    #[test]
    fn test_fallback_plan_packages() {
        use anna_common::telemetry::SystemTelemetry;
        use anna_common::planner_core::{DomainType, GoalType, Intent};

        let intent = Intent {
            goal: GoalType::Inspect,
            domain: DomainType::Packages,
            constraints: vec![],
            query: "do I have games?".to_string(),
        };

        let telemetry = SystemTelemetry::default();
        let plan = generate_fallback_plan(&intent, &telemetry);

        assert!(plan.is_ok());
        let plan = plan.unwrap();
        assert!(!plan.commands.is_empty());
        assert_eq!(plan.safety_level, anna_common::planner_core::SafetyLevel::ReadOnly);
    }
}
