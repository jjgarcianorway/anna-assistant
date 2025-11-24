//! LLM Query Handler - Natural language query processing
//!
//! Beta.200: Unified workflow for one-shot queries
//!
//! Implements telemetry-first architecture:
//! 1. Detect intent (informational vs actionable)
//! 2. Match to deterministic recipe (if actionable)
//! 3. Generate answer based on real telemetry
//!
//! Uses unified_query_handler for consistency with TUI mode.

use anna_common::answer_formatter::{render_human_answer, AnswerContext};
use anna_common::display::UI;
use anna_common::executor::{execute_plan, ExecutionReport, StepExecutionKind};
use anna_common::llm::LlmConfig;
use anna_common::orchestrator::{
    get_arch_help_dns, get_arch_help_service_failure,
    plan_dns_fix, plan_service_failure_fix,
    Plan, ServiceStatus, TelemetrySummary,
};
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::io::{self, BufRead, Write};
use std::time::Duration;

use crate::llm_integration::fetch_telemetry_snapshot;
use crate::output::normalize_for_cli;
use crate::startup::welcome::{generate_welcome_report, load_last_session, save_session_metadata};
use crate::system_query::query_system_telemetry;
use crate::unified_query_handler::{handle_unified_query, AnswerConfidence, UnifiedQueryResult};

/// Print execution report to user (6.5.0)
fn print_execution_report(report: &ExecutionReport, ui: &UI) {
    for result in &report.results {
        if let Some(reason) = &result.skipped_reason {
            // Skipped step
            if ui.capabilities().use_colors() {
                println!(
                    "[{}] {}",
                    "SKIPPED".yellow().bold(),
                    result.command.dimmed()
                );
                println!("  Reason: {}", reason.dimmed());
            } else {
                println!("[SKIPPED] {}", result.command);
                println!("  Reason: {}", reason);
            }
        } else if result.kind == StepExecutionKind::Harmless {
            // Executed step
            if result.success {
                if ui.capabilities().use_colors() {
                    println!("[{}] {}", "OK".green().bold(), result.command);
                } else {
                    println!("[OK] {}", result.command);
                }
            } else {
                if ui.capabilities().use_colors() {
                    println!(
                        "[{}] {}",
                        "FAILED".red().bold(),
                        result.command.bright_red()
                    );
                    if let Some(code) = result.exit_code {
                        println!("  Exit code: {}", code);
                    }
                } else {
                    println!("[FAILED] {}", result.command);
                    if let Some(code) = result.exit_code {
                        println!("  Exit code: {}", code);
                    }
                }
            }
        }
    }

    println!();

    // Summary
    if report.all_succeeded() {
        if ui.capabilities().use_colors() {
            println!(
                "{} All {} harmless steps executed successfully.",
                "✓".green().bold(),
                report.executed_count()
            );
        } else {
            println!(
                "All {} harmless steps executed successfully.",
                report.executed_count()
            );
        }
    } else {
        if ui.capabilities().use_colors() {
            println!(
                "{} Some steps failed.",
                "✗".red().bold()
            );
        } else {
            println!("Some steps failed.");
        }
    }

    if report.skipped_count() > 0 {
        println!(
            "{} file-writing steps were skipped for safety.",
            report.skipped_count()
        );
    }
}

/// Try to handle query with the planner (6.4.x, 6.5.0)
///
/// Returns Some((answer_string, plan)) if the planner can handle it, None otherwise.
fn try_planner_answer(
    user_text: &str,
    telemetry: &anna_common::telemetry::SystemTelemetry,
) -> Option<(String, Plan)> {
    // Convert to TelemetrySummary
    let telemetry_summary = convert_to_planner_summary(telemetry);

    // Check if we have anything to plan for
    let has_dns_issue = telemetry_summary.dns_suspected_broken && telemetry_summary.network_reachable;
    let has_failed_services = !telemetry_summary.failed_services.is_empty();

    if !has_dns_issue && !has_failed_services {
        return None; // Nothing for planner to handle
    }

    // Run relevant planner slices
    let mut all_plans = Vec::new();

    if has_dns_issue {
        let wiki = get_arch_help_dns();
        let plan = plan_dns_fix(user_text, &telemetry_summary, &wiki);
        if !plan.steps.is_empty() {
            all_plans.push((plan, wiki.sources));
        }
    }

    for service in &telemetry_summary.failed_services {
        if service.is_failed {
            let service_name = service.name.trim_end_matches(".service");
            let wiki = get_arch_help_service_failure(service_name);
            let plan = plan_service_failure_fix(user_text, &telemetry_summary, &wiki);
            if !plan.steps.is_empty() {
                all_plans.push((plan, wiki.sources));
            }
        }
    }

    if all_plans.is_empty() {
        return None;
    }

    // Combine plans and generate answer using standard formatter
    // For now, use the first plan (DNS or first service)
    let (plan, wiki_sources) = all_plans.into_iter().next().unwrap();

    let ctx = AnswerContext {
        user_goal: user_text.to_string(),
        telemetry_summary,
        plan: plan.clone(),
        wiki_sources,
    };

    let answer = render_human_answer(&ctx);
    Some((answer, plan))
}

/// Convert SystemTelemetry to TelemetrySummary for planner
fn convert_to_planner_summary(
    system_telemetry: &anna_common::telemetry::SystemTelemetry,
) -> TelemetrySummary {
    let network_reachable = system_telemetry.network.is_connected;
    let dns_suspected_broken = false; // TODO: Add DNS-specific checks to NetworkInfo

    let failed_services: Vec<ServiceStatus> = system_telemetry
        .services
        .failed_units
        .iter()
        .filter(|unit| unit.unit_type == "service")
        .map(|unit| ServiceStatus {
            name: unit.name.clone(),
            is_failed: true,
        })
        .collect();

    TelemetrySummary {
        dns_suspected_broken,
        network_reachable,
        failed_services,
    }
}

/// Handle a one-shot natural language query
///
/// This is used for: annactl "free storage space"
/// Version 149: Uses unified handler for consistency with TUI.
/// Beta.228: Added comprehensive logging
/// 6.4.x: Try planner first, fall back to LLM
pub async fn handle_one_shot_query(user_text: &str) -> Result<()> {
    let start_time = std::time::Instant::now();

    let ui = UI::auto();

    // Show user's question with nice formatting
    println!();
    if ui.capabilities().use_colors() {
        println!("{} {}", "you:".bright_cyan().bold(), user_text.white());
    } else {
        println!("you: {}", user_text);
    }
    println!();

    // Create spinner for thinking animation (Beta.202: Professional animation)
    let spinner = create_thinking_spinner(&ui);

    // Get telemetry
    let telemetry_start = std::time::Instant::now();
    let telemetry = query_system_telemetry()?;

    // Get LLM config
    let config = get_llm_config();

    // 6.8.1: Handle health questions with telemetry, not generic LLM
    if matches_health_question(user_text) {
        return handle_health_question(&ui).await;
    }

    // 6.10.0: Handle desktop/WM questions with real detection, not generic LLM
    if matches_desktop_question(user_text) {
        return handle_desktop_question(&ui).await;
    }

    // 6.4.x/6.5.0: Try planner first
    if let Some((planner_answer, plan)) = try_planner_answer(user_text, &telemetry) {
        spinner.finish_and_clear();

        // 6.7.0: Show reflection preamble first (limit to top 3 items)
        let reflection = crate::reflection_helper::build_local_reflection();
        if reflection.items.len() > 0 {
            let limited_reflection = anna_common::ipc::ReflectionSummaryData {
                items: reflection.items.into_iter().take(3).collect(),
                generated_at: reflection.generated_at,
            };
            let reflection_text = crate::reflection_helper::format_reflection(&limited_reflection, ui.capabilities().use_colors(), None);
            print!("{}", reflection_text);
            println!("---\n");
        }

        // Print "Now about your question..."
        if ui.capabilities().use_colors() {
            print!("{} ", "anna:".bright_magenta().bold());
        } else {
            print!("anna: ");
        }
        println!("Now about your question: \"{}\"\n", user_text);
        io::stdout().flush().unwrap();

        // Print the planner answer
        println!("{}", planner_answer);

        // 6.5.0: Read user confirmation for execution
        let stdin = io::stdin();
        let mut input = String::new();
        if let Ok(_) = stdin.lock().read_line(&mut input) {
            let trimmed = input.trim();
            if trimmed.starts_with('y') || trimmed.starts_with('Y') {
                // User confirmed - execute the plan
                println!();
                println!("Executing harmless steps:");
                println!();

                let report = execute_plan(&plan);
                print_execution_report(&report, &ui);

                // Exit with appropriate code
                if report.all_succeeded() {
                    return Ok(());
                } else {
                    std::process::exit(1);
                }
            }
        }

        // User declined or no input - just exit
        return Ok(());
    }

    // Beta.229: Stop spinner before unified handler to prevent corruption during streaming
    spinner.finish_and_clear();

    // 6.7.0: Show reflection preamble for ALL queries (not just planner)
    let reflection = crate::reflection_helper::build_local_reflection();
    if reflection.items.len() > 0 {
        let limited_reflection = anna_common::ipc::ReflectionSummaryData {
            items: reflection.items.into_iter().take(3).collect(),
            generated_at: reflection.generated_at,
        };
        let reflection_text = crate::reflection_helper::format_reflection(&limited_reflection, ui.capabilities().use_colors(), None);
        print!("{}", reflection_text);
        println!("---\n");
    }

    // Beta.237: Show "anna:" prefix to reduce perceived latency
    if ui.capabilities().use_colors() {
        print!("{} ", "anna:".bright_magenta().bold());
    } else {
        print!("anna: ");
    }
    io::stdout().flush().unwrap();

    // Use unified query handler (fallback for non-planner queries)
    let handler_start = std::time::Instant::now();
    match handle_unified_query(user_text, &telemetry, &config).await {
        Ok(UnifiedQueryResult::DeterministicRecipe {
            recipe_name,
            action_plan,
        }) => {
            // Beta.237: "anna:" already printed above, just add the recipe info
            println!(
                "{} {}",
                "Using deterministic recipe:".white(),
                recipe_name.bright_green()
            );
            println!();
            display_action_plan(&action_plan, &ui);
        }
        Ok(UnifiedQueryResult::Template {
            command, output, ..
        }) => {
            // Beta.237: "anna:" already printed above
            println!("{}", "Running:".white());
            ui.info(&format!("  $ {}", command));
            println!();
            for line in output.lines() {
                ui.info(line);
            }
            println!();
        }
        Ok(UnifiedQueryResult::ActionPlan {
            action_plan,
            raw_json: _,
        }) => {
            // Beta.237: "anna:" already printed above, just newline
            println!();
            display_action_plan(&action_plan, &ui);
        }
        Ok(UnifiedQueryResult::ConversationalAnswer {
            answer,
            confidence,
            sources,
        }) => {

            // Beta.229: DISABLED - Welcome report adds 19s delay to one-shot queries
            // The fetch_telemetry_snapshot() and generate_welcome_report() are extremely slow
            // Re-enable in Beta.230+ with performance optimization or async background task

            // Beta.229: Answer already streamed to stdout during LLM call
            // Don't print it again! Just show metadata
            println!();

            // Beta.245: Show source line based on confidence level
            // Deterministic answers (High confidence from diagnostic engine or telemetry) get a clear source line
            // LLM answers show confidence level
            if ui.capabilities().use_colors() {
                match confidence {
                    AnswerConfidence::High => {
                        // Deterministic answer - show source without confidence score
                        println!(
                            "{}",
                            format!("Source: {}", sources.join(", "))
                                .dimmed()
                        );
                    }
                    AnswerConfidence::Medium | AnswerConfidence::Low => {
                        // LLM answer - show confidence and sources
                        println!(
                            "{}",
                            format!("🔍 Confidence: {:?} | Sources: {}", confidence, sources.join(", "))
                                .dimmed()
                        );
                    }
                }
            }
            println!();
        }
        Err(e) => {
            spinner.finish_and_clear();
            ui.error(&format!("Query failed: {}", e));
            println!();
        }
    }

    Ok(())
}

/// Create thinking spinner (Beta.202: Professional animation)
fn create_thinking_spinner(ui: &UI) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();

    if ui.capabilities().use_colors() {
        spinner.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                .template("{spinner:.magenta} {msg}")
                .unwrap()
        );
        spinner.set_message("anna (thinking)...".to_string());
    } else {
        spinner.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["-", "\\", "|", "/"])
                .template("{spinner} {msg}")
                .unwrap()
        );
        spinner.set_message("anna (thinking)...".to_string());
    }

    spinner.enable_steady_tick(Duration::from_millis(80));
    spinner
}

/// Display ActionPlan
fn display_action_plan(plan: &anna_common::action_plan_v3::ActionPlan, ui: &UI) {
    // Show analysis
    ui.section_header("📋", "Analysis");
    println!("{}\n", plan.analysis);

    // Show goals
    ui.section_header("🎯", "Goals");
    for (i, goal) in plan.goals.iter().enumerate() {
        println!("  {}. {}", i + 1, goal);
    }
    println!();

    // Show command plan
    if !plan.command_plan.is_empty() {
        ui.section_header("⚡", "Command Plan");
        for (i, step) in plan.command_plan.iter().enumerate() {
            println!("  {}. {} [Risk: {:?}]", i + 1, step.description, step.risk_level);
            println!("     $ {}", step.command);
        }
        println!();
    }

    // Show notes
    if !plan.notes_for_user.is_empty() {
        ui.section_header("ℹ️", "Notes");
        println!("{}\n", plan.notes_for_user);
    }
}

/// Stream LLM response (Beta.202: Updated for spinner)
async fn stream_llm_response(prompt: &str, config: &LlmConfig, ui: &UI, spinner: &ProgressBar) -> Result<()> {
    use anna_common::llm::{LlmClient, LlmPrompt};

    let client = match LlmClient::from_config(config) {
        Ok(client) => client,
        Err(_) => {
            spinner.finish_and_clear();
            ui.info("⚠ LLM not available (Ollama not running)");
            return Ok(());
        }
    };

    let llm_prompt = LlmPrompt {
        system: LlmClient::anna_system_prompt().to_string(),
        user: prompt.to_string(),
        conversation_history: None,
    };

    let mut response_started = false;
    let mut callback = |chunk: &str| {
        if !response_started {
            spinner.finish_and_clear();
            if ui.capabilities().use_colors() {
                print!("{} ", "anna:".bright_magenta().bold());
            } else {
                print!("anna: ");
            }
            response_started = true;
        }

        if ui.capabilities().use_colors() {
            print!("{}", chunk.white());
        } else {
            print!("{}", chunk);
        }
        io::stdout().flush().unwrap();
    };

    match client.chat_stream(&llm_prompt, &mut callback) {
        Ok(_) => println!("\n"),
        Err(_) => {
            if !response_started {
                spinner.finish_and_clear();
            }
            println!();
            ui.info("⚠ LLM request failed");
        }
    }

    Ok(())
}

/// Get LLM config
fn get_llm_config() -> LlmConfig {
    use std::process::Command;

    let model_name = match Command::new("ollama").arg("list").output() {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout
                .lines()
                .nth(1)
                .and_then(|line| line.split_whitespace().next())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "llama3.1:8b".to_string())
        }
        _ => "llama3.1:8b".to_string(),
    };

    LlmConfig::local("http://127.0.0.1:11434/v1", &model_name)
}

// ============================================================================
// 6.8.1 Hotfix: Health Question Handling
// ============================================================================

/// Detect if a question is asking about system health
///
/// These questions should use telemetry, not generic LLM responses.
fn matches_health_question(question: &str) -> bool {
    let q_lower = question.to_lowercase();

    let health_patterns = [
        "how is my computer",
        "how is my system",
        "is my system healthy",
        "is my computer healthy",
        "status of my machine",
        "status of my system",
        "how is my machine",
        "system health",
        "computer health",
    ];

    for pattern in &health_patterns {
        if q_lower.contains(pattern) {
            return true;
        }
    }

    false
}

/// Handle health questions using telemetry instead of generic LLM
///
/// Produces a short health summary based on:
/// - Overall health status
/// - Reflection
/// - Brain diagnostics
async fn handle_health_question(ui: &UI) -> Result<()> {
    use crate::status_command::call_brain_analysis;
    use anna_common::ipc::Method;

    // Show "anna:" prefix
    if ui.capabilities().use_colors() {
        println!("{}", "anna:".bright_magenta().bold());
    } else {
        println!("anna:");
    }
    println!();

    // Get health data (same as status command)
    match call_brain_analysis().await {
        Ok(analysis) => {
            // Show reflection first
            let reflection = crate::reflection_helper::build_local_reflection();
            if !reflection.items.is_empty() {
                let reflection_text =
                    crate::reflection_helper::format_reflection(&reflection, ui.capabilities().use_colors(), None);
                print!("{}", reflection_text);
                println!();
            }

            // Compute overall health
            let overall_health = crate::diagnostic_formatter::compute_overall_health(&analysis);

            // Format health line (6.11.1: already has markdown formatting)
            let health_text = crate::diagnostic_formatter::format_today_health_line_from_health(overall_health);
            println!("{}", health_text);
            println!();

            // Show top issues if any
            if analysis.critical_count > 0 || analysis.warning_count > 0 {
                println!("Key issues detected:");
                println!();

                for (idx, insight) in analysis.insights.iter().take(3).enumerate() {
                    let severity_marker = match insight.severity.to_lowercase().as_str() {
                        "critical" => "✗",
                        "warning" => "⚠",
                        _ => "ℹ",
                    };
                    println!("  {} {}", severity_marker, insight.summary);

                    // Show evidence if available
                    if !insight.evidence.is_empty() {
                        println!("     {}", insight.evidence);
                    }
                }

                println!();
                println!("For the full report, run: {}", "annactl status".bright_cyan());
            } else {
                println!("No critical issues detected. System is operating normally.");
            }
        }
        Err(_) => {
            println!("Unable to fetch system health data (daemon may be offline).");
            println!();
            println!("Try running: {}", "annactl status".bright_cyan());
        }
    }

    Ok(())
}

/// Detect if a question is asking about desktop environment or window manager (6.10.0)
///
/// These questions should use actual detection, not generic LLM responses.
fn matches_desktop_question(question: &str) -> bool {
    let q_lower = question.to_lowercase();

    let desktop_patterns = [
        "what de am i using",
        "what desktop am i using",
        "what wm am i using",
        "which desktop environment",
        "which window manager",
        "what desktop environment",
        "what window manager",
        "which de am i",
        "which wm am i",
        "what is my de",
        "what is my wm",
        "what desktop",
        "what window manager",
    ];

    for pattern in &desktop_patterns {
        if q_lower.contains(pattern) {
            return true;
        }
    }

    false
}

/// Handle desktop/WM questions using real detection (6.10.0)
///
/// Runs safe read-only commands automatically:
/// - echo $XDG_CURRENT_DESKTOP
/// - echo $DESKTOP_SESSION
/// - ps -e (for WM process detection)
///
/// Then provides a clear answer based on what was detected.
async fn handle_desktop_question(ui: &UI) -> Result<()> {
    use anna_common::desktop::DesktopInfo;

    // Show "anna:" prefix
    if ui.capabilities().use_colors() {
        println!("{}", "anna:".bright_magenta().bold());
    } else {
        println!("anna:");
    }
    println!();

    // Detect desktop environment
    let desktop_info = DesktopInfo::detect();

    // Build list of commands we ran for transparency
    let mut commands_run = vec![
        "echo \"$XDG_CURRENT_DESKTOP\"".to_string(),
        "echo \"$DESKTOP_SESSION\"".to_string(),
    ];

    match &desktop_info.environment {
        anna_common::desktop::DesktopEnvironment::None => {
            // No desktop detected
            println!("I checked your current session but could not confidently detect a desktop environment or window manager.");
            println!();
            println!("**Detection methods tried:**");
            println!("- Environment variables: $XDG_CURRENT_DESKTOP, $DESKTOP_SESSION");
            println!("- Process inspection: ps -e");
            commands_run.push("ps -e".to_string());
            println!();
            println!("You may be running:");
            println!("- A headless/TTY session");
            println!("- An uncommon or custom window manager");
            println!("- A desktop that doesn't set standard environment variables");
        }
        _ => {
            // Desktop detected
            let de_name = desktop_info.environment.name();
            let is_wm = matches!(
                desktop_info.environment,
                anna_common::desktop::DesktopEnvironment::Hyprland
                    | anna_common::desktop::DesktopEnvironment::I3
                    | anna_common::desktop::DesktopEnvironment::Sway
            );

            let de_type = if is_wm {
                "window manager"
            } else {
                "desktop environment"
            };

            println!("**Detected:** {} ({})", de_name, de_type);
            println!();

            // Show session type
            match desktop_info.session_type {
                anna_common::desktop::SessionType::Wayland => {
                    println!("**Session type:** Wayland");
                }
                anna_common::desktop::SessionType::X11 => {
                    println!("**Session type:** X11");
                }
                _ => {}
            }

            // Show config file location if known
            if let Some(config_file) = &desktop_info.config_file {
                println!();
                println!(
                    "**Config file:** {}",
                    config_file.display()
                );
            }

            // Explain detection method
            println!();
            println!("**Detection method:**");
            println!("- Checked environment variables ($XDG_CURRENT_DESKTOP, $DESKTOP_SESSION)");
            if is_wm {
                println!("- Verified {} process is running", de_name);
                commands_run.push(format!("ps -C {}", de_name.to_lowercase()));
            }
        }
    }

    // Show commands run for transparency (6.10.0 requirement)
    println!();
    println!("**Commands I ran:**");
    for cmd in commands_run {
        println!("  {}", cmd);
    }

    Ok(())
}
