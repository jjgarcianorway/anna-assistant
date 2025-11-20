//! ActionPlan system - Execution, generation, and rendering

use crate::tui_state::AnnaTuiState;
use anyhow::Result;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use tokio::sync::mpsc;

use super::event_loop::TuiMessage;
use super::utils::wrap_text;

/// Beta.147: Handle action plan execution
pub fn handle_action_plan_execution(state: &mut AnnaTuiState, tx: mpsc::Sender<TuiMessage>) {
    use crate::action_plan_executor::ActionPlanExecutor;

    // Take the action plan (so we can move it into the async task)
    if let Some(plan) = state.last_action_plan.take() {
        state.add_system_message("⚙️ Executing action plan...".to_string());

        // Spawn async execution task
        tokio::spawn(async move {
            let executor = ActionPlanExecutor::new(*plan);

            match executor.execute().await {
                Ok(result) => {
                    // Send execution results back to TUI
                    let mut summary = String::new();

                    if result.success {
                        summary.push_str("✅ Execution completed successfully!\n\n");
                    } else {
                        summary.push_str("❌ Execution failed.\n\n");
                    }

                    if !result.checks_passed.is_empty() {
                        summary
                            .push_str(&format!("Checks passed: {}\n", result.checks_passed.len()));
                    }

                    if !result.checks_failed.is_empty() {
                        summary
                            .push_str(&format!("Checks failed: {}\n", result.checks_failed.len()));
                    }

                    summary.push_str(&format!(
                        "Steps completed: {}\n",
                        result.steps_completed.len()
                    ));

                    if !result.steps_failed.is_empty() {
                        summary.push_str(&format!("Steps failed: {}\n", result.steps_failed.len()));
                    }

                    if result.rollback_performed {
                        summary.push_str(&format!(
                            "\n🔄 Rollback performed ({} steps)\n",
                            result.rollback_results.len()
                        ));
                    }

                    let _ = tx.send(TuiMessage::AnnaReply(summary)).await;
                }
                Err(e) => {
                    let error_msg = format!("❌ Execution error: {}", e);
                    let _ = tx.send(TuiMessage::AnnaReply(error_msg)).await;
                }
            }
        });
    }
}

/// Beta.147: Send demo ActionPlan for testing
pub fn send_demo_action_plan(tx: mpsc::Sender<TuiMessage>, risky: bool) {
    use anna_common::action_plan_v3::{
        ActionPlan, CommandStep, NecessaryCheck, PlanMeta, RiskLevel, RollbackStep,
    };

    tokio::spawn(async move {
        // Simulate thinking time
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        let demo_plan = if risky {
            // Risky demo with rollback
            ActionPlan {
                analysis: "The user wants to create a test file in /tmp and then remove it. This demonstrates a risky operation with rollback capability. The file creation is medium risk as it modifies the filesystem.".to_string(),

                goals: vec![
                    "Create a test file in /tmp".to_string(),
                    "Demonstrate rollback on cleanup".to_string(),
                ],

                necessary_checks: vec![NecessaryCheck {
                    id: "check-tmp-writable".to_string(),
                    description: "Verify /tmp directory is writable".to_string(),
                    command: "test -w /tmp && echo 'writable'".to_string(),
                    risk_level: RiskLevel::Info,
                    required: true,
                }],

                command_plan: vec![
                    CommandStep {
                        id: "create-test-file".to_string(),
                        description: "Create test file with timestamp".to_string(),
                        command: "echo 'Anna Demo Test' > /tmp/anna_demo_test.txt".to_string(),
                        risk_level: RiskLevel::Medium,
                        rollback_id: Some("remove-test-file".to_string()),
                        requires_confirmation: true,
                    },
                    CommandStep {
                        id: "show-file-content".to_string(),
                        description: "Display the created file content".to_string(),
                        command: "cat /tmp/anna_demo_test.txt".to_string(),
                        risk_level: RiskLevel::Info,
                        rollback_id: None,
                        requires_confirmation: false,
                    },
                ],

                rollback_plan: vec![RollbackStep {
                    id: "remove-test-file".to_string(),
                    description: "Remove the test file".to_string(),
                    command: "rm -f /tmp/anna_demo_test.txt".to_string(),
                }],

                notes_for_user: "This demo creates a test file in /tmp to show how Anna handles operations with rollback. The file will be automatically removed if any step fails. You can also manually delete it with: rm /tmp/anna_demo_test.txt".to_string(),

                meta: PlanMeta {
                    detection_results: Default::default(),
                    template_used: Some("risky_demo".to_string()),
                    llm_version: "demo-v1".to_string(),
                },
            }
        } else {
            // Safe demo
            ActionPlan {
                analysis: "The user wants to check free disk space on their system. This is a safe, read-only operation that uses the 'df' command, which is a standard Unix utility available on all Linux systems including Arch Linux.".to_string(),

                goals: vec![
                    "Display disk usage in human-readable format".to_string(),
                    "Show filesystem types and mount points".to_string(),
                ],

                necessary_checks: vec![NecessaryCheck {
                    id: "check-df".to_string(),
                    description: "Verify 'df' command is available".to_string(),
                    command: "which df".to_string(),
                    risk_level: RiskLevel::Info,
                    required: true,
                }],

                command_plan: vec![
                    CommandStep {
                        id: "show-disk-usage".to_string(),
                        description: "Display disk usage with human-readable sizes".to_string(),
                        command: "df -h".to_string(),
                        risk_level: RiskLevel::Info,
                        rollback_id: None,
                        requires_confirmation: false,
                    },
                    CommandStep {
                        id: "show-inodes".to_string(),
                        description: "Display inode usage information".to_string(),
                        command: "df -i".to_string(),
                        risk_level: RiskLevel::Info,
                        rollback_id: None,
                        requires_confirmation: false,
                    },
                ],

                rollback_plan: vec![],

                notes_for_user: "This is a completely safe operation that only reads system information. No changes will be made to your system. The 'df' command shows disk space usage across all mounted filesystems.".to_string(),

                meta: PlanMeta {
                    detection_results: Default::default(),
                    template_used: Some("disk_space_check".to_string()),
                    llm_version: "demo-v1".to_string(),
                },
            }
        };

        let _ = tx.send(TuiMessage::ActionPlanReply(demo_plan)).await;
    });
}

/// Beta.148: Determine if query should generate an ActionPlan
///
/// Queries that involve actions, commands, or system changes should use ActionPlan mode.
/// Informational queries should use standard reply mode.
pub fn should_generate_action_plan(input: &str) -> bool {
    let input_lower = input.to_lowercase();

    // Action keywords indicate user wants to DO something
    let action_keywords = [
        "install",
        "remove",
        "uninstall",
        "delete",
        "create",
        "make",
        "setup",
        "configure",
        "fix",
        "repair",
        "solve",
        "troubleshoot",
        "start",
        "stop",
        "restart",
        "enable",
        "disable",
        "update",
        "upgrade",
        "downgrade",
        "clean",
        "clear",
        "purge",
        "add",
        "set",
        "change",
        "modify",
        "download",
        "build",
        "compile",
        "run",
        "execute",
        "kill",
        "terminate",
    ];

    // Check for action verbs
    for keyword in &action_keywords {
        if input_lower.contains(keyword) {
            return true;
        }
    }

    // Questions and informational queries should NOT use ActionPlan
    if input_lower.starts_with("what")
        || input_lower.starts_with("how")
        || input_lower.starts_with("why")
        || input_lower.starts_with("when")
        || input_lower.starts_with("who")
        || input_lower.starts_with("where")
        || input_lower.starts_with("is ")
        || input_lower.starts_with("are ")
        || input_lower.starts_with("do you")
        || input_lower.starts_with("can you explain")
        || input_lower.starts_with("tell me")
        || input_lower.starts_with("show me")
    {
        return false;
    }

    false
}

/// Beta.148: Generate ActionPlan from LLM using V3 JSON dialogue
pub async fn generate_action_plan_from_llm(
    input: &str,
    state: &AnnaTuiState,
    tx: mpsc::Sender<TuiMessage>,
) -> Result<()> {
    use crate::dialogue_v3_json;
    use crate::system_query::query_system_telemetry;
    use anna_common::llm::LlmConfig;

    // Version 150: Use SystemTelemetry from unified query system
    let telemetry = query_system_telemetry()?;

    // Use detected model from state
    let model_name = if state.llm_panel.model_name == "None"
        || state.llm_panel.model_name == "Unknown"
        || state.llm_panel.model_name == "Ollama N/A"
    {
        "llama3.1:8b"
    } else {
        &state.llm_panel.model_name
    };

    let llm_config = LlmConfig::local("http://127.0.0.1:11434/v1", model_name);

    // Run V3 JSON dialogue
    let result = dialogue_v3_json::run_dialogue_v3_json(input, &telemetry, &llm_config).await?;

    // Send ActionPlan to TUI for display
    tx.send(TuiMessage::ActionPlanReply(result.action_plan))
        .await
        .map_err(|e| anyhow::anyhow!("Failed to send ActionPlan: {}", e))?;

    Ok(())
}

/// Beta.147: Render action plan as formatted lines
pub fn render_action_plan_lines(
    lines: &mut Vec<Line<'static>>,
    plan: &anna_common::action_plan_v3::ActionPlan,
    content_width: usize,
) {
    use anna_common::action_plan_v3::RiskLevel;

    // Header
    lines.push(Line::from(vec![Span::styled(
        "📋 Action Plan".to_string(),
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(""));

    // Analysis section
    lines.push(Line::from(vec![Span::styled(
        "Analysis:".to_string(),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]));
    let analysis_lines = wrap_text(&plan.analysis, content_width.saturating_sub(2));
    for line in analysis_lines {
        lines.push(Line::from(format!("  {}", line)));
    }
    lines.push(Line::from(""));

    // Goals section
    if !plan.goals.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "Goals:".to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]));
        for (i, goal) in plan.goals.iter().enumerate() {
            let goal_lines = wrap_text(goal, content_width.saturating_sub(5));
            for (j, line) in goal_lines.iter().enumerate() {
                if j == 0 {
                    lines.push(Line::from(format!("  {}. {}", i + 1, line)));
                } else {
                    lines.push(Line::from(format!("     {}", line)));
                }
            }
        }
        lines.push(Line::from(""));
    }

    // Necessary checks section
    if !plan.necessary_checks.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "Necessary Checks:".to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]));
        for check in &plan.necessary_checks {
            let risk_color = match check.risk_level {
                RiskLevel::Info => Color::Blue,
                RiskLevel::Low => Color::Green,
                RiskLevel::Medium => Color::Yellow,
                RiskLevel::High => Color::Red,
            };
            let emoji = check.risk_level.emoji().to_string();
            let desc = check.description.clone();

            lines.push(Line::from(vec![
                Span::raw("  ".to_string()),
                Span::styled(emoji, Style::default().fg(risk_color)),
                Span::raw(" ".to_string()),
                Span::styled(desc, Style::default().fg(Color::White)),
            ]));
            lines.push(Line::from(format!("    $ {}", check.command)));
        }
        lines.push(Line::from(""));
    }

    // Command plan section
    if !plan.command_plan.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "Command Plan:".to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]));
        for (i, step) in plan.command_plan.iter().enumerate() {
            let risk_color = match step.risk_level {
                RiskLevel::Info => Color::Blue,
                RiskLevel::Low => Color::Green,
                RiskLevel::Medium => Color::Yellow,
                RiskLevel::High => Color::Red,
            };
            let emoji = step.risk_level.emoji().to_string();

            // Description (use owned strings)
            let desc_lines = wrap_text(&step.description, content_width.saturating_sub(10));
            for (j, desc_line) in desc_lines.iter().enumerate() {
                if j == 0 {
                    lines.push(Line::from(vec![
                        Span::raw(format!("  {}. ", i + 1)),
                        Span::styled(emoji.clone(), Style::default().fg(risk_color)),
                        Span::raw(" ".to_string()),
                        Span::styled(desc_line.clone(), Style::default().fg(Color::White)),
                    ]));
                } else {
                    lines.push(Line::from(format!("        {}", desc_line)));
                }
            }

            lines.push(Line::from(format!("      $ {}", step.command)));

            if let Some(rollback_id) = &step.rollback_id {
                lines.push(Line::from(vec![
                    Span::raw("      ".to_string()),
                    Span::styled("↩ Rollback: ".to_string(), Style::default().fg(Color::Gray)),
                    Span::styled(rollback_id.clone(), Style::default().fg(Color::DarkGray)),
                ]));
            }
        }
        lines.push(Line::from(""));
    }

    // Rollback plan section
    if !plan.rollback_plan.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "Rollback Plan:".to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]));
        for rollback in &plan.rollback_plan {
            lines.push(Line::from(vec![
                Span::raw("  ↩ ".to_string()),
                Span::styled(rollback.id.clone(), Style::default().fg(Color::Yellow)),
                Span::raw(": ".to_string()),
                Span::styled(
                    rollback.description.clone(),
                    Style::default().fg(Color::White),
                ),
            ]));
            lines.push(Line::from(format!("    $ {}", rollback.command)));
        }
        lines.push(Line::from(""));
    }

    // Notes for user
    if !plan.notes_for_user.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "Notes:".to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]));
        let notes_lines = wrap_text(&plan.notes_for_user, content_width.saturating_sub(2));
        for line in notes_lines {
            lines.push(Line::from(format!("  {}", line)));
        }
        lines.push(Line::from(""));
    }

    // Max risk indicator
    if let Some(max_risk) = plan.max_risk_level() {
        let risk_color = match max_risk {
            RiskLevel::Info => Color::Blue,
            RiskLevel::Low => Color::Green,
            RiskLevel::Medium => Color::Yellow,
            RiskLevel::High => Color::Red,
        };
        lines.push(Line::from(vec![
            Span::styled("Max Risk: ".to_string(), Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{:?}", max_risk),
                Style::default().fg(risk_color).add_modifier(Modifier::BOLD),
            ),
        ]));
    }
}
