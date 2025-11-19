//! Anna Control - CLI client for Anna Assistant
//!
//! Provides user interface to interact with the Anna daemon.
//!
//! Phase 0.3: State-aware command dispatch with no-ops
//! Citation: [archwiki:system_maintenance]

// Phase 0.3a: Commands module will be reimplemented in 0.3c
// mod commands;
pub mod errors;
// Core modules for conversational Anna (Phase 5.1)
mod intent_router; // Natural language → intent mapping
mod llm_wizard; // First-run LLM setup wizard
mod repl; // Conversational REPL
mod version_banner; // Startup version banner and update notifications

// Internal intent handlers (not exposed as CLI commands)
mod action_executor; // Execute approved suggestions
mod autonomy_command; // Handle autonomy intent
mod context_detection;
mod discard_command; // Handle discard intent
mod first_run;
mod health; // Central health model and auto-repair
mod health_commands; // Contains repair logic - will be refactored
mod help_commands;
mod historian_cli;
mod init_command;
mod install_command;
mod json_types;
pub mod logging;
mod internal_dialogue; // Beta.55: Telemetry-first internal dialogue
mod llm_integration; // Beta.53: LLM query with streaming support
mod model_catalog; // Beta.53: Intelligent model selection
mod model_setup_wizard; // Beta.53: First-run model setup
mod monitor_setup;
mod runtime_prompt; // Beta.53: Prompt builder with Historian data
mod startup_summary; // Beta.53: Startup health display
pub mod output;
mod repair; // Internal repair engine (not a CLI command)
mod report_command; // Handle report intent
mod report_display; // Display professional reports
mod rpc_client;
mod status_command; // Handle anna_status intent
mod suggestion_display; // Display suggestions with Arch Wiki links
mod suggestions; // Internal suggestions engine (not a CLI command)
mod system_query; // Query real system state
mod systemd; // Systemd service management utilities
mod upgrade_command;

// TODO: Delete these experimental/developer commands in next cleanup
mod adaptive_help;
mod chronos_commands;
mod collective_commands;
mod conscience_commands;
mod consensus_commands;
mod empathy_commands;
mod learning_commands;
mod mirror_commands;
mod personality_commands; // Beta.87: CLI personality management
mod predictive_hints;
mod sentinel_cli;

use anna_common::display::UI;
use anna_common::ipc::{CommandCapabilityData, ResponseData};
use anyhow::Result;
use clap::{Parser, Subcommand};
use errors::*;
use logging::{ErrorDetails, LogEntry};
use output::CommandOutput;
use std::process::Command;
use std::time::Instant;

// Version is embedded at build time
const VERSION: &str = env!("ANNA_VERSION");

#[derive(Parser)]
#[command(name = "annactl")]
#[command(about = "Anna Assistant - Autonomous system administrator", long_about = None)]
#[command(version = VERSION)]
#[command(disable_help_subcommand = true)]
struct Cli {
    /// Path to daemon socket (overrides $ANNAD_SOCKET and defaults)
    #[arg(long, global = true)]
    socket: Option<String>,

    /// Subcommand (if not provided, starts interactive REPL)
    #[command(subcommand)]
    command: Option<Commands>,
}

/// Beta.89: Only status command allowed, everything else is natural language
#[derive(Subcommand)]
enum Commands {
    /// Show system status and daemon health
    Status {
        /// Output JSON only
        #[arg(long)]
        json: bool,
    },

    /// Historian sanity inspection (developer-only, hidden)
    #[command(hide = true)]
    Historian {
        #[command(subcommand)]
        action: HistorianCommands,
    },

    /// Show version (hidden - use --version flag instead)
    #[command(hide = true)]
    Version,

    /// Ping daemon (hidden - for health checks only)
    #[command(hide = true)]
    Ping,

    /// Launch TUI REPL (hidden - experimental)
    #[command(hide = true)]
    Tui,
}

#[derive(Subcommand)]
enum HistorianCommands {
    Inspect,
}

// All legacy subcommands removed - REPL is the primary interface

/// Get canonical citation for a state (Phase 0.3d)
fn state_citation(state: &str) -> &'static str {
    match state {
        "iso_live" => "[archwiki:installation_guide]",
        "recovery_candidate" => "[archwiki:Chroot#Using_arch-chroot]",
        "post_install_minimal" => "[archwiki:General_recommendations]",
        "configured" => "[archwiki:System_maintenance]",
        "degraded" => "[archwiki:System_maintenance#Troubleshooting]",
        "unknown" => "[archwiki:General_recommendations]",
        _ => "[archwiki:General_recommendations]",
    }
}

/// Check if LLM setup is needed and run wizard if so (Phase Next: Step 2)
pub(crate) async fn run_llm_setup_if_needed(
    ui: &anna_common::display::UI,
    db: &anna_common::context::db::ContextDb,
) -> anyhow::Result<()> {
    if llm_wizard::needs_llm_setup(db).await? {
        llm_wizard::run_llm_setup_wizard(ui, db).await?;
    }
    Ok(())
}

/// Check for and display pending brain upgrade notification (Phase Next: Step 3)
pub(crate) async fn check_brain_upgrade_notification(
    ui: &anna_common::display::UI,
    db: &anna_common::context::db::ContextDb,
) -> anyhow::Result<()> {
    use anna_common::llm_upgrade::get_and_clear_pending_upgrade;

    if let Some((profile_id, model_name, size_gb)) = get_and_clear_pending_upgrade(db).await? {
        println!();
        ui.section_header("🚀", "My Brain Can Upgrade!");
        println!();
        ui.info("Great news! Your machine got more powerful.");
        ui.info("I can now upgrade to a better language model:");
        println!();
        ui.info(&format!("  New model: {}", model_name));
        ui.info(&format!("  Download size: ~{:.1} GB", size_gb));
        ui.info(&format!("  Profile: {}", profile_id));
        println!();
        ui.info("To upgrade, ask me: \"Upgrade your brain\" or \"Set up your brain\"");
        println!();
    }

    Ok(())
}

/// Check for and display update notification (Phase Next: Step 5)
pub(crate) async fn check_update_notification(ui: &anna_common::display::UI) -> anyhow::Result<()> {
    const PENDING_NOTICE_FILE: &str = "/var/lib/anna/pending_update_notice";
    const CHANGELOG_PATH: &str = "/usr/share/doc/anna/CHANGELOG.md";

    // Check if update notification is pending
    let update_record = match tokio::fs::read_to_string(PENDING_NOTICE_FILE).await {
        Ok(content) => content,
        Err(_) => return Ok(()), // No pending update
    };

    // Parse: "from_version|to_version"
    let parts: Vec<&str> = update_record.trim().split('|').collect();
    if parts.len() != 2 {
        return Ok(());
    }

    let from_version = parts[0];
    let to_version = parts[1];

    // Display update notification
    println!();
    ui.section_header("✨", "I Updated Myself!");
    println!();
    ui.success(&format!(
        "I upgraded from v{} to v{}",
        from_version, to_version
    ));
    println!();

    // Try to extract changelog for this version
    if let Ok(changes) = extract_version_changelog(to_version).await {
        if !changes.is_empty() {
            ui.info("What's new:");
            for change in changes.iter().take(4) {
                ui.info(&format!("  • {}", change));
            }
            println!();
        }
    } else {
        ui.info("Check the full changelog for details.");
        println!();
    }

    // Clear the pending notice file
    let _ = tokio::fs::remove_file(PENDING_NOTICE_FILE).await;

    Ok(())
}

/// Extract changelog entries for a specific version
async fn extract_version_changelog(version: &str) -> anyhow::Result<Vec<String>> {
    const CHANGELOG_PATH: &str = "/usr/share/doc/anna/CHANGELOG.md";

    // Try to read changelog from installed location
    let changelog = tokio::fs::read_to_string(CHANGELOG_PATH).await?;

    let mut changes = Vec::new();
    let mut in_version_section = false;
    let version_header = format!("## [{}]", version);

    for line in changelog.lines() {
        // Check if we've entered the version section
        if line.starts_with(&version_header) || line.starts_with(&format!("## v{}", version)) {
            in_version_section = true;
            continue;
        }

        // Stop if we hit the next version section
        if in_version_section && line.starts_with("## ") {
            break;
        }

        // Collect bullet points from the version section
        if in_version_section {
            let trimmed = line.trim();
            if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                // Remove bullet and trim
                let change = trimmed[2..].trim().to_string();
                if !change.is_empty() {
                    changes.push(change);
                }
            }
        }
    }

    Ok(changes)
}

/// Handle LLM query (Task 12)
async fn handle_llm_query(user_text: &str) {
    use anna_common::display::UI;
    use anna_common::llm::{LlmClient, LlmConfig, LlmPrompt};
    use anna_common::template_library::TemplateLibrary;
    use std::collections::HashMap;

    let ui = UI::auto();
    println!();
    ui.thinking();

    // Beta.91: Try template matching first (prevents hallucinations)
    let library = TemplateLibrary::default();
    let input_lower = user_text.to_lowercase();

    // Pattern matching for template selection (Beta.93: expanded library)
    let template_match = if input_lower.contains("swap") {
        Some(("check_swap_status", HashMap::new()))
    } else if input_lower.contains("gpu") || input_lower.contains("vram") {
        Some(("check_gpu_memory", HashMap::new()))
    } else if input_lower.contains("kernel") {
        Some(("check_kernel_version", HashMap::new()))
    } else if input_lower.contains("disk") || input_lower.contains("space") {
        Some(("check_disk_space", HashMap::new()))
    } else if input_lower.contains("ram") || input_lower.contains("memory") || input_lower.contains("mem") {
        Some(("check_memory", HashMap::new()))
    } else if input_lower.contains("uptime") {
        Some(("check_uptime", HashMap::new()))
    } else if input_lower.contains("cpu model") || input_lower.contains("processor") {
        Some(("check_cpu_model", HashMap::new()))
    } else if input_lower.contains("cpu load") || input_lower.contains("cpu usage") || input_lower.contains("load average") {
        Some(("check_cpu_load", HashMap::new()))
    } else if input_lower.contains("distro") || input_lower.contains("distribution") || input_lower.contains("os-release") {
        Some(("check_distro", HashMap::new()))
    } else if input_lower.contains("failed services") || (input_lower.contains("systemctl") && input_lower.contains("failed")) {
        Some(("check_failed_services", HashMap::new()))
    } else if input_lower.contains("journal") || (input_lower.contains("system") && input_lower.contains("errors")) {
        Some(("check_journal_errors", HashMap::new()))
    } else if input_lower.contains("wifi") || input_lower.contains("wireless") ||
              (input_lower.contains("network") && (input_lower.contains("slow") || input_lower.contains("issue") || input_lower.contains("problem"))) {
        // Beta.101: WiFi diagnostics - triggered by "wifi", "wireless", or "network slow/issue/problem"
        Some(("wifi_diagnostics", HashMap::new()))
    } else {
        None
    };

    // If template matches, use it (instant, accurate, no hallucination)
    if let Some((template_id, params)) = template_match {
        if let Some(template) = library.get(template_id) {
            match template.instantiate(&params) {
                Ok(recipe) => {
                    println!();
                    ui.section_header("💬", "Anna");
                    println!();

                    // Execute command and show output
                    ui.info(&format!("Running: {}", recipe.command));
                    println!();

                    match std::process::Command::new("sh")
                        .arg("-c")
                        .arg(&recipe.command)
                        .output()
                    {
                        Ok(output) => {
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            for line in stdout.lines() {
                                ui.info(line);
                            }
                        }
                        Err(e) => {
                            ui.info(&format!("⚠ Command failed: {}", e));
                        }
                    }
                    println!();
                    return;
                }
                Err(e) => {
                    // Template instantiation failed, fall through to LLM
                    eprintln!("Warning: Template instantiation failed: {}", e);
                }
            }
        }
    }

    // No template match - use LLM (with Ollama detection)
    // Beta.91: Use Ollama local LLM instead of database config

    // Detect model from ollama list
    let model_name = match Command::new("ollama").arg("list").output() {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Get first non-header line (most recently used model)
            stdout.lines()
                .skip(1)
                .next()
                .and_then(|line| line.split_whitespace().next())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "llama3.1:8b".to_string())
        }
        _ => "llama3.1:8b".to_string(), // Fallback
    };

    let config = LlmConfig::local("http://127.0.0.1:11434/v1", &model_name);

    // Create LLM client
    let client = match LlmClient::from_config(&config) {
        Ok(client) => client,
        Err(_) => {
            println!();
            ui.info("⚠ LLM not available (Ollama not running)");
            ui.info("I can still help with:");
            ui.bullet_list(&[
                "swap - Check swap status",
                "GPU/VRAM - Check GPU memory",
                "kernel - Check kernel version",
                "disk/space - Check disk space",
                "RAM/memory - Check system memory",
            ]);
            println!();
            ui.info("To enable LLM for other questions:");
            ui.info("  1. Install Ollama: curl -fsSL https://ollama.com/install.sh | sh");
            ui.info("  2. Pull a model: ollama pull llama3.1:8b");
            println!();
            return;
        }
    };

    // Build system context
    use crate::system_query::query_system_telemetry;
    let system_context = if let Ok(telemetry) = query_system_telemetry() {
        format!(
            "System Information:\n\
             - CPU: {}\n\
             - RAM: {:.1} GB used / {:.1} GB total\n\
             - GPU: {}\n\
             - OS: Arch Linux",
            telemetry.hardware.cpu_model,
            telemetry.memory.used_mb as f64 / 1024.0,
            telemetry.memory.total_mb as f64 / 1024.0,
            telemetry.hardware.gpu_info.as_ref().map(|s| s.as_str()).unwrap_or("None"),
        )
    } else {
        "OS: Arch Linux".to_string()
    };

    // Create prompt with system context
    let system_prompt = format!(
        "{}\n\n{}",
        LlmClient::anna_system_prompt(),
        system_context
    );

    let prompt = LlmPrompt {
        system: system_prompt,
        user: user_text.to_string(),
        conversation_history: None,
    };

    // Query LLM (blocking)
    match client.chat(&prompt) {
        Ok(response) => {
            println!();
            ui.section_header("💬", "Anna");
            println!();
            for line in response.text.lines() {
                ui.info(line);
            }
            println!();
        }
        Err(_) => {
            println!();
            ui.info("⚠ LLM request failed");
            ui.info("Try template-based questions:");
            ui.bullet_list(&[
                "swap, gpu, kernel, disk, ram/memory",
            ]);
            println!();
        }
    }
}

/// Handle a single conversational query (annactl "question")
async fn handle_one_shot_query(query: &str) -> Result<()> {
    use anna_common::context::db::{ContextDb, DbLocation};
    use anna_common::display::{print_privacy_explanation, UI};
    use intent_router::{Intent, PersonalityAdjustment};

    // Phase Next Step 2, 3 & 5: Check LLM setup, brain upgrade, and update notifications
    let ui = UI::auto();
    let db_location = DbLocation::auto_detect();

    // Check for update notification (Step 5)
    if let Err(e) = check_update_notification(&ui).await {
        eprintln!("Warning: Update notification check failed: {}", e);
    }

    if let Ok(db) = ContextDb::open(db_location).await {
        // Check if setup wizard needs to run (Step 2)
        if let Err(e) = run_llm_setup_if_needed(&ui, &db).await {
            eprintln!("Warning: LLM setup check failed: {}", e);
        }

        // Check for brain upgrade notifications (Step 3)
        if let Err(e) = check_brain_upgrade_notification(&ui, &db).await {
            eprintln!("Warning: Brain upgrade check failed: {}", e);
        }
    }

    let intent = intent_router::route_intent(query);

    match intent {
        Intent::Exit => {
            let ui = UI::auto();
            println!();
            ui.info("Goodbye!");
            println!();
        }

        Intent::AnnaStatus | Intent::SystemStatus => {
            let ui = UI::auto();
            ui.thinking();
            ui.success("I'm running and ready to help!");
            ui.info("All my systems are operational.");
            println!();
        }

        Intent::Privacy => {
            let ui = UI::auto();
            ui.section_header("🔒", "Privacy & Data Handling");
            print_privacy_explanation();
        }

        Intent::Report => {
            let ui = UI::auto();
            ui.thinking();
            report_display::generate_professional_report().await;
        }

        Intent::Suggest | Intent::Improve => {
            let ui = UI::auto();
            ui.thinking();
            // Use Anna's internal suggestion engine (checks Anna's health, system basics)
            match suggestions::get_suggestions() {
                Ok(suggestions) => {
                    suggestions::display_suggestions(&suggestions);
                }
                Err(e) => {
                    ui.error(&format!("Failed to generate suggestions: {}", e));
                    println!();
                }
            }
        }

        Intent::AnnaSelfRepair => {
            let ui = UI::auto();
            ui.thinking();
            ui.section_header("🔧", "Anna Self-Repair");
            println!();

            // Use Anna's internal repair engine
            match repair::repair() {
                Ok(report) => {
                    repair::display_repair_report(&report);
                }
                Err(e) => {
                    ui.error(&format!("Failed to run self-repair: {}", e));
                    println!();
                }
            }
        }

        Intent::Repair { .. } => {
            let ui = UI::auto();
            println!();
            ui.info("[General system repair coming soon]");
            ui.info("For now, ask me 'what should I improve?' to see suggestions.");
            println!();
        }

        Intent::Discard { .. } => {
            let ui = UI::auto();
            println!();
            ui.info("[Discard functionality coming soon]");
            ui.info("This will let you hide suggestions you don't want to see.");
            println!();
        }

        Intent::Autonomy { .. } => {
            let ui = UI::auto();
            println!();
            ui.info("[Autonomy controls coming soon]");
            ui.info("This will let you adjust how much Anna can do automatically.");
            println!();
        }

        Intent::Apply { .. } => {
            let ui = UI::auto();
            ui.thinking();
            ui.info("Let me show you the available fixes...");
            println!();

            // Generate suggestions
            let suggestions = suggestion_display::generate_suggestions_from_telemetry();
            let mut engine = anna_common::suggestions::SuggestionEngine::new();

            for suggestion in suggestions {
                engine.add_suggestion(suggestion);
            }

            let top = engine.get_top_suggestions(5);

            // Filter to auto-fixable only
            let auto_fixable: Vec<_> = top.into_iter().filter(|s| s.auto_fixable).collect();

            if auto_fixable.is_empty() {
                ui.info("I don't have any suggestions that I can automatically fix right now.");
                ui.info("Review the suggestions with: annactl \"what should I improve?\"");
                println!();
                return Ok(());
            }

            // Convert from references to owned for selection
            let suggestions_vec: Vec<anna_common::suggestions::Suggestion> =
                auto_fixable.into_iter().cloned().collect();
            let suggestions_ref: Vec<&anna_common::suggestions::Suggestion> =
                suggestions_vec.iter().collect();

            if let Some(idx) = action_executor::select_suggestion_to_apply(&suggestions_ref) {
                if idx < suggestions_vec.len() {
                    action_executor::execute_suggestion(&suggestions_vec[idx]).await?;
                }
            }
        }

        Intent::Personality { adjustment } => {
            use anna_common::personality::PersonalityConfig;

            let ui = UI::auto();
            let mut config = PersonalityConfig::load();

            match &adjustment {
                PersonalityAdjustment::IncreaseHumor => {
                    // Map to playful_vs_serious trait
                    let _ = config.adjust_trait("playful_vs_serious", 2);
                    println!();
                    ui.success("Okay! I'll be a bit more playful 😊");
                    if let Some(trait_ref) = config.get_trait("playful_vs_serious") {
                        ui.info(&format!("Playfulness: {}/10 - {}", trait_ref.value, trait_ref.meaning));
                    }
                    println!();
                }
                PersonalityAdjustment::DecreaseHumor => {
                    // Map to playful_vs_serious trait
                    let _ = config.adjust_trait("playful_vs_serious", -2);
                    println!();
                    ui.success("Got it. I'll keep things more serious.");
                    if let Some(trait_ref) = config.get_trait("playful_vs_serious") {
                        ui.info(&format!("Playfulness: {}/10 - {}", trait_ref.value, trait_ref.meaning));
                    }
                    println!();
                }
                PersonalityAdjustment::MoreBrief => {
                    // Map to minimalist_vs_verbose trait
                    let _ = config.adjust_trait("minimalist_vs_verbose", 2);
                    println!();
                    ui.success("Understood. I'll be more concise.");
                    println!();
                }
                PersonalityAdjustment::MoreDetailed => {
                    // Map to minimalist_vs_verbose trait
                    let _ = config.adjust_trait("minimalist_vs_verbose", -2);
                    println!();
                    ui.success("Sure! I'll provide more detailed explanations.");
                    println!();
                }
                PersonalityAdjustment::Show => {
                    println!();
                    ui.section_header("📊", "Personality Traits (0-10 scale)");
                    for trait_item in &config.traits {
                        ui.info(&format!("{}: {} {}",
                            trait_item.name,
                            trait_item.bar(),
                            trait_item.value
                        ));
                        ui.info(&format!("  → {}", trait_item.meaning));
                    }
                    println!();
                }
                PersonalityAdjustment::SetTrait { trait_key, value } => {
                    // Beta.89: Set specific trait to value
                    match config.set_trait(trait_key, *value) {
                        Ok(_) => {
                            println!();
                            ui.success(&format!("✓ Set {} to {}/10", trait_key, value));
                            if let Some(trait_ref) = config.get_trait(&trait_key) {
                                ui.info(&format!("  {}", trait_ref.meaning));
                            }
                            println!();
                        }
                        Err(e) => {
                            println!();
                            ui.error(&format!("Failed to set trait: {}", e));
                            println!();
                        }
                    }
                }
                PersonalityAdjustment::AdjustByDescriptor { descriptor } => {
                    // Beta.89: Adjust personality by natural language descriptor
                    println!();
                    ui.info(&format!("Adjusting personality: '{}'", descriptor));
                    ui.warning("Note: Descriptor-based adjustments not fully implemented yet");
                    println!();
                }
                PersonalityAdjustment::Reset => {
                    // Beta.89: Reset all traits to defaults
                    config = PersonalityConfig::default();
                    println!();
                    ui.success("✓ Reset all personality traits to defaults");
                    println!();
                }
                PersonalityAdjustment::Validate => {
                    // Beta.89: Validate personality configuration
                    match config.validate_interactions() {
                        Ok(_) => {
                            println!();
                            ui.success("✓ Personality configuration is valid");
                            ui.info("No conflicting trait combinations detected");
                            println!();
                        }
                        Err(issues) => {
                            println!();
                            ui.warning("⚠ Personality validation warnings:");
                            for issue in issues {
                                ui.warning(&format!("  • {}", issue));
                            }
                            println!();
                        }
                    }
                }
            }

            if !matches!(adjustment, PersonalityAdjustment::Show) {
                if let Err(e) = config.save() {
                    ui.warning(&format!("Note: Couldn't save settings: {}", e));
                    println!();
                } else {
                    ui.success("Settings saved");
                    println!();
                }
            }
        }

        Intent::Language { language } => {
            use anna_common::context::db::{ContextDb, DbLocation};
            use anna_common::language::{Language, LanguageConfig};

            let ui = UI::auto();

            if let Some(lang_str) = language {
                // Parse the requested language
                if let Some(lang) = Language::from_str(&lang_str) {
                    // Load current config
                    let db_location = DbLocation::auto_detect();
                    let db_result = ContextDb::open(db_location).await;

                    let mut config = if let Ok(db) = &db_result {
                        db.load_language_config().await.unwrap_or_default()
                    } else {
                        LanguageConfig::new()
                    };

                    // Set the new language
                    config.set_user_language(lang);

                    // Save to database
                    if let Ok(db) = &db_result {
                        if let Err(e) = db.save_language_config(&config).await {
                            println!();
                            ui.warning(&format!(
                                "Warning: Couldn't persist language setting: {}",
                                e
                            ));
                            println!();
                        }
                    }

                    // Confirm in the NEW language (create UI with new config)
                    let new_ui = UI::new(&config);
                    let profile = config.profile();
                    println!();
                    new_ui.success(&profile.translations.language_changed);
                    new_ui.info(&format!(
                        "{} {}",
                        profile.translations.now_speaking,
                        lang.native_name()
                    ));
                    println!();
                } else {
                    println!();
                    ui.info("I don't recognize that language. I can speak:");
                    ui.bullet_list(&[
                        "English",
                        "Español (Spanish)",
                        "Norsk (Norwegian)",
                        "Deutsch (German)",
                        "Français (French)",
                        "Português (Portuguese)",
                    ]);
                    println!();
                }
            } else {
                println!();
                ui.info("Which language would you like me to use?");
                ui.info(
                    "Try: \"annactl \\\"use English\\\"\" or \"annactl \\\"cambia al español\\\"",
                );
                println!();
            }
        }

        Intent::SetupBrain => {
            use anna_common::context::db::{ContextDb, DbLocation};

            let ui = UI::auto();
            let db_location = DbLocation::auto_detect();

            match ContextDb::open(db_location).await {
                Ok(db) => {
                    println!();
                    ui.info("Starting LLM brain setup...");
                    if let Err(e) = llm_wizard::run_llm_setup_wizard(&ui, &db).await {
                        ui.error(&format!("Setup failed: {}", e));
                    }
                }
                Err(e) => {
                    ui.error(&format!("Could not access database: {}", e));
                }
            }
        }

        Intent::Help => {
            let ui = UI::auto();
            ui.section_header("💡", "What I Can Help With");
            ui.info("I'm Anna, your Arch Linux caretaker.");
            println!();
            ui.info("Try asking:");
            ui.bullet_list(&[
                "\"How are you?\" - Check my status",
                "\"Any problems with my system?\" - Get suggestions",
                "\"Generate a report\" - Create system summary",
                "\"What do you store?\" - Privacy information",
            ]);
            println!();
            ui.info("Or just run 'annactl' to start a conversation.");
            println!();
        }

        Intent::HistorianSummary => {
            let ui = UI::auto();
            ui.info("Historian summary feature coming soon!");
            ui.info("This will show 30-day trend analysis once fully integrated.");
            println!();
        }

        Intent::OffTopic => {
            let ui = UI::auto();
            println!();
            ui.info(&intent_router::offtopic_response());
        }

        Intent::Unclear(user_text) => {
            // Task 12: Route to LLM
            handle_llm_query(&user_text).await;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Beta.90: Real TUI - Start TUI if no arguments
    let args: Vec<String> = std::env::args().collect();

    // If no arguments at all, start real TUI
    if args.len() == 1 {
        return annactl::tui_v2::run().await;
    }

    // Task 11: Natural language without quotes
    // If arguments present but first arg is not a known subcommand/flag,
    // treat all args as natural language query
    if args.len() >= 2 {
        let first_arg = &args[1];

        // Beta.89: Only "status" is a valid subcommand
        // Everything else is natural language
        let known_subcommands = ["status"];

        // Check if it's a flag (starts with - or --)
        let is_flag = first_arg.starts_with("--") || first_arg.starts_with("-");

        // Check if it's a known subcommand
        let is_known_command = known_subcommands.contains(&first_arg.as_str());

        // If not a flag and not a known command, treat as natural language
        if !is_flag && !is_known_command {
            // Join all args from index 1 onwards with spaces
            let query = args[1..].join(" ");
            return handle_one_shot_query(&query).await;
        }
    }

    // Phase 3.8: Intercept root help for adaptive display
    // Check for root help invocation
    if args.len() == 2 && (args[1] == "--help" || args[1] == "-h") {
        adaptive_help::display_adaptive_root_help(false, false);
        std::process::exit(0);
    }

    // Check for --help --all --json (all three)
    if args.len() >= 4
        && args.contains(&"--help".to_string())
        && args.contains(&"--all".to_string())
        && args.contains(&"--json".to_string())
    {
        adaptive_help::display_adaptive_root_help(true, true);
        std::process::exit(0);
    }

    // Check for --help --all
    if args.len() >= 3
        && args.contains(&"--help".to_string())
        && args.contains(&"--all".to_string())
    {
        adaptive_help::display_adaptive_root_help(true, false);
        std::process::exit(0);
    }

    // Check for --json help
    if args.len() >= 3
        && args.contains(&"--help".to_string())
        && args.contains(&"--json".to_string())
    {
        adaptive_help::display_adaptive_root_help(false, true);
        std::process::exit(0);
    }

    let start_time = Instant::now();
    let req_id = LogEntry::generate_req_id();

    // Phase 0.5c3: Custom error handling for unknown flags (exit 64)
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => {
            // Check if it's an unknown argument/flag error
            use clap::error::ErrorKind;

            // Handle --version and --help specially (these are not errors)
            match err.kind() {
                ErrorKind::DisplayVersion | ErrorKind::DisplayHelp => {
                    // Print the version/help output
                    let _ = err.print();
                    std::process::exit(0);
                }
                _ => {}
            }

            let exit_code = match err.kind() {
                ErrorKind::UnknownArgument => EXIT_COMMAND_NOT_AVAILABLE,
                ErrorKind::InvalidSubcommand => EXIT_COMMAND_NOT_AVAILABLE,
                _ => 2, // Default clap error code
            };

            // Print the error message using UI abstraction (language-aware)
            let ui = UI::auto();
            ui.error(&format!("{}", err));
            println!();
            ui.info("Try talking to Anna in natural language:");
            ui.info("  annactl \"how are you?\"");
            ui.info("  annactl \"what should I improve?\"");
            println!();
            ui.info("Or run: annactl --help");
            println!();

            // Log the error
            let duration_ms = start_time.elapsed().as_millis() as u64;
            let log_entry = LogEntry {
                ts: LogEntry::now(),
                req_id,
                state: "unknown".to_string(),
                command: "parse_error".to_string(),
                allowed: Some(false),
                args: std::env::args().skip(1).collect(),
                exit_code,
                citation: "[archwiki:General_recommendations]".to_string(),
                duration_ms,
                ok: false,
                error: Some(ErrorDetails {
                    code: "INVALID_ARGUMENT".to_string(),
                    message: err.to_string(),
                }),
            };
            let _ = log_entry.write();

            std::process::exit(exit_code);
        }
    };

    // Handle no command -> start interactive REPL
    // Beta.82: Default to TUI instead of line-based REPL (claude-cli style)
    if cli.command.is_none() {
        return annactl::tui::run().map_err(|e| anyhow::anyhow!("TUI error: {}", e));
    }

    // Unwrap command (we know it's Some at this point)
    let command = cli.command.as_ref().unwrap();

    // Handle version command
    if matches!(command, Commands::Version) {
        version_banner::display_version_only().await;
        return Ok(());
    }

    // Handle TUI command (Phase 5.7 - experimental)
    if matches!(command, Commands::Tui) {
        return annactl::tui::run().map_err(|e| anyhow::anyhow!("TUI error: {}", e));
    }

    // Phase 1.8: Handle consensus commands early (standalone PoC, no daemon)
    // Beta.89: All legacy commands removed
    // Only status, version, ping, tui, and historian remain
    // Everything else handled via natural language

    // Historian inspection (developer-only, no daemon required)
    if let Commands::Historian { action } = command {
        match action {
            HistorianCommands::Inspect => {
                historian_cli::run_historian_inspect().await?;
                return Ok(());
            }
        }
    }

    // Beta.89: Personality management removed from CLI
    // Now handled via natural language in the REPL

    // Phase 3.9: Handle init command early (doesn't need daemon)
    // Phase 3.9: Handle learning commands early (don't need daemon, use context DB directly)
    // TODO: Wire new real Anna commands here
    // - report
    // - suggest
    // - discard
    // - autonomy
    // - status (Anna self-health)

    // Beta.89: State-aware dispatch with minimal commands
    // Get command name first
    let command_name = match command {
        Commands::Status { .. } => "status",
        Commands::Historian { .. } => "historian",
        Commands::Version => "version",
        Commands::Ping => "ping",
        Commands::Tui => "tui",
    };

    // Try to connect to daemon and get state
    let socket_path = cli.socket.as_deref();
    let (state, state_citation, capabilities) = match get_state_and_capabilities(socket_path).await
    {
        Ok(result) => result,
        Err(e) => {
            // Daemon unavailable
            let duration_ms = start_time.elapsed().as_millis() as u64;
            let log_entry = LogEntry {
                ts: LogEntry::now(),
                req_id,
                state: "unknown".to_string(),
                command: command_name.to_string(),
                allowed: None,
                args: vec![],
                exit_code: EXIT_DAEMON_UNAVAILABLE,
                citation: "[archwiki:system_maintenance]".to_string(),
                duration_ms,
                ok: false,
                error: Some(ErrorDetails {
                    code: "DAEMON_UNAVAILABLE".to_string(),
                    message: format!("Failed to connect to daemon: {}", e),
                }),
            };
            let _ = log_entry.write();

            let output = CommandOutput::daemon_unavailable(command_name.to_string());
            output.print();
            std::process::exit(EXIT_DAEMON_UNAVAILABLE);
        }
    };

    // v1.16.3: Handle ping command (simple 1-RTT check)
    if matches!(command, Commands::Ping) {
        return execute_ping_command(socket_path, &req_id, start_time).await;
    }

    // Handle Status command
    if let Commands::Status { json } = command {
        return status_command::execute_anna_status_command(*json, &req_id, &state, start_time)
            .await;
    }
    // Check if command is allowed in current state
    let allowed = capabilities.iter().any(|cap| cap.name == command_name);

    if !allowed {
        let duration_ms = start_time.elapsed().as_millis() as u64;
        let log_entry = LogEntry {
            ts: LogEntry::now(),
            req_id,
            state: state.clone(),
            command: command_name.to_string(),
            allowed: Some(false),
            args: vec![],
            exit_code: EXIT_COMMAND_NOT_AVAILABLE,
            citation: state_citation.clone(),
            duration_ms,
            ok: false,
            error: None,
        };
        let _ = log_entry.write();

        let output =
            CommandOutput::not_available(state.clone(), command_name.to_string(), state_citation);
        output.print();
        std::process::exit(EXIT_COMMAND_NOT_AVAILABLE);
    }

    // Command is allowed - log and exit (all commands have their own handlers above)
    let exit_code = EXIT_SUCCESS;
    let duration_ms = start_time.elapsed().as_millis() as u64;

    // Find the citation for this specific command
    let command_citation = capabilities
        .iter()
        .find(|cap| cap.name == command_name)
        .map(|cap| cap.citation.clone())
        .unwrap_or_else(|| state_citation.clone());

    let log_entry = LogEntry {
        ts: LogEntry::now(),
        req_id,
        state: state.clone(),
        command: command_name.to_string(),
        allowed: Some(true),
        args: vec![],
        exit_code,
        citation: command_citation,
        duration_ms,
        ok: true,
        error: None,
    };
    let _ = log_entry.write();

    std::process::exit(exit_code);
}

/// Execute ping command (v1.16.3)
async fn execute_ping_command(
    socket_path: Option<&str>,
    req_id: &str,
    start_time: Instant,
) -> Result<()> {
    let ping_start = Instant::now();

    match rpc_client::RpcClient::connect_with_path(socket_path).await {
        Ok(mut client) => {
            match client.ping().await {
                Ok(_) => {
                    let rtt_ms = ping_start.elapsed().as_millis();
                    println!("Pong! RTT: {}ms", rtt_ms);

                    // Log the ping command
                    let duration_ms = start_time.elapsed().as_millis() as u64;
                    let log_entry = LogEntry {
                        ts: LogEntry::now(),
                        req_id: req_id.to_string(),
                        state: "unknown".to_string(),
                        command: "ping".to_string(),
                        allowed: Some(true),
                        args: vec![],
                        exit_code: EXIT_SUCCESS,
                        citation: "[archwiki:system_maintenance]".to_string(),
                        duration_ms,
                        ok: true,
                        error: None,
                    };
                    let _ = log_entry.write();

                    std::process::exit(EXIT_SUCCESS);
                }
                Err(e) => {
                    eprintln!("Ping failed: {}", e);

                    // Log the failed ping
                    let duration_ms = start_time.elapsed().as_millis() as u64;
                    let log_entry = LogEntry {
                        ts: LogEntry::now(),
                        req_id: req_id.to_string(),
                        state: "unknown".to_string(),
                        command: "ping".to_string(),
                        allowed: Some(true),
                        args: vec![],
                        exit_code: EXIT_DAEMON_UNAVAILABLE,
                        citation: "[archwiki:system_maintenance]".to_string(),
                        duration_ms,
                        ok: false,
                        error: Some(ErrorDetails {
                            code: "PING_FAILED".to_string(),
                            message: e.to_string(),
                        }),
                    };
                    let _ = log_entry.write();

                    std::process::exit(EXIT_DAEMON_UNAVAILABLE);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to connect to daemon: {}", e);

            // Log the connection failure
            let duration_ms = start_time.elapsed().as_millis() as u64;
            let log_entry = LogEntry {
                ts: LogEntry::now(),
                req_id: req_id.to_string(),
                state: "unknown".to_string(),
                command: "ping".to_string(),
                allowed: None,
                args: vec![],
                exit_code: EXIT_DAEMON_UNAVAILABLE,
                citation: "[archwiki:system_maintenance]".to_string(),
                duration_ms,
                ok: false,
                error: Some(ErrorDetails {
                    code: "DAEMON_UNAVAILABLE".to_string(),
                    message: e.to_string(),
                }),
            };
            let _ = log_entry.write();

            std::process::exit(EXIT_DAEMON_UNAVAILABLE);
        }
    }
}

/// Detect if Anna was installed via package manager (AUR, etc.)
fn detect_package_install() -> Option<String> {
    // Check if annactl is owned by pacman
    if let Ok(output) = std::process::Command::new("pacman")
        .args(["-Qo", "/usr/bin/annactl"])
        .output()
    {
        if output.status.success() {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                // Output format: "/usr/bin/annactl is owned by package-name version"
                if let Some(package) = stdout.split_whitespace().nth(4) {
                    return Some(package.to_string());
                }
            }
        }
    }

    // Check if in /usr/local (homebrew or manual install)
    if let Ok(exe_path) = std::env::current_exe() {
        if exe_path.starts_with("/usr/local") {
            return Some("homebrew/manual".to_string());
        }
    }

    None
}

/// Execute self-update command (Phase 2.0 + Phase 3.8 AUR awareness)
async fn execute_self_update_command(check: bool, list: bool) -> Result<()> {
    const REPO_OWNER: &str = "jjgarcianorway";
    const REPO_NAME: &str = "anna-assistant";
    const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

    // Phase 3.8: Check if installed via package manager
    if let Some(package) = detect_package_install() {
        println!("⚠️  Anna was installed via package manager: {}", package);
        println!();
        println!("Please use your package manager to update:");
        if package.contains("anna-assistant") {
            println!("  pacman -Syu              # System update (includes Anna)");
            println!("  yay -Sua                 # AUR update only");
        } else {
            println!("  Check your package manager documentation");
        }
        println!();
        println!("The self-update command is for manual/binary installations only.");
        std::process::exit(EXIT_SUCCESS);
    }

    if list {
        // List available versions from GitHub releases
        println!("Fetching available versions...");
        match reqwest::get(format!(
            "https://api.github.com/repos/{}/{}/releases",
            REPO_OWNER, REPO_NAME
        ))
        .await
        {
            Ok(response) => {
                if let Ok(releases) = response.json::<serde_json::Value>().await {
                    if let Some(releases_array) = releases.as_array() {
                        println!("\nAvailable versions:");
                        for release in releases_array.iter().take(10) {
                            if let Some(tag) = release["tag_name"].as_str() {
                                let prerelease = release["prerelease"].as_bool().unwrap_or(false);
                                let label = if prerelease { "(pre-release)" } else { "" };
                                println!("  {} {}", tag, label);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to fetch releases: {}", e);
                std::process::exit(EXIT_GENERAL_ERROR);
            }
        }
        std::process::exit(EXIT_SUCCESS);
    }

    if check {
        // Check for updates without installing
        println!("Current version: {}", CURRENT_VERSION);
        println!("Checking for updates...");

        match reqwest::get(format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            REPO_OWNER, REPO_NAME
        ))
        .await
        {
            Ok(response) => {
                if let Ok(release) = response.json::<serde_json::Value>().await {
                    if let Some(latest_tag) = release["tag_name"].as_str() {
                        // Strip 'v' prefix if present
                        let latest_version = latest_tag.trim_start_matches('v');

                        if latest_version != CURRENT_VERSION {
                            println!(
                                "\n✨ Update available: {} → {}",
                                CURRENT_VERSION, latest_version
                            );
                            println!("\nTo update:");
                            println!(
                                "  1. Download: https://github.com/{}/{}/releases/tag/{}",
                                REPO_OWNER, REPO_NAME, latest_tag
                            );
                            println!("  2. Or use package manager:");
                            println!("     - Arch: yay -S anna-assistant-bin");
                            println!("     - Homebrew: brew upgrade anna-assistant");
                        } else {
                            println!("✓ You are running the latest version ({})", CURRENT_VERSION);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to check for updates: {}", e);
                std::process::exit(EXIT_GENERAL_ERROR);
            }
        }

        std::process::exit(EXIT_SUCCESS);
    }

    // No flags - show usage
    eprintln!("Usage: annactl self-update --check | --list");
    std::process::exit(EXIT_GENERAL_ERROR);
}

/// Execute profile command (Phase 3.0)
async fn execute_profile_command(json: bool, socket_path: Option<&str>) -> Result<()> {
    // Connect to daemon
    let mut client = match rpc_client::RpcClient::connect_with_path(socket_path).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to connect to daemon: {}", e);
            eprintln!("Make sure annad is running (try: sudo systemctl start annad)");
            std::process::exit(EXIT_DAEMON_UNAVAILABLE);
        }
    };

    // Get system profile
    let profile_response = client.get_profile().await?;
    let profile = match profile_response {
        ResponseData::Profile(data) => data,
        _ => anyhow::bail!("Unexpected response type for GetProfile"),
    };

    if json {
        // JSON output
        let json_output = serde_json::to_string_pretty(&profile)?;
        println!("{}", json_output);
    } else {
        // Human-readable output
        println!("System Profile (Phase 3.0: Adaptive Intelligence)");
        println!("==================================================\n");

        println!("Resources:");
        println!(
            "  Memory:  {} MB total, {} MB available",
            profile.total_memory_mb, profile.available_memory_mb
        );
        println!("  CPU:     {} cores", profile.cpu_cores);
        println!(
            "  Disk:    {} GB total, {} GB available",
            profile.total_disk_gb, profile.available_disk_gb
        );
        println!(
            "  Uptime:  {} seconds ({:.1} hours)",
            profile.uptime_seconds,
            profile.uptime_seconds as f64 / 3600.0
        );
        println!();

        println!("Environment:");
        println!("  Virtualization: {}", profile.virtualization);
        println!("  Session Type:   {}", profile.session_type);
        if profile.gpu_present {
            println!(
                "  GPU:            {} ({})",
                profile.gpu_vendor.as_deref().unwrap_or("Unknown"),
                profile.gpu_model.as_deref().unwrap_or("Unknown model")
            );
        } else {
            println!("  GPU:            Not detected");
        }
        println!();

        println!("Adaptive Intelligence:");
        println!(
            "  Monitoring Mode: {}",
            profile.recommended_monitoring_mode.to_uppercase()
        );
        println!("  Rationale:       {}", profile.monitoring_rationale);
        println!(
            "  Constrained:     {}",
            if profile.is_constrained { "Yes" } else { "No" }
        );
        println!();

        // Phase 3.0: SSH tunnel suggestions (Remote Access Policy)
        if profile.session_type.starts_with("ssh:") {
            println!("Remote Access:");
            let has_display = profile.session_type.contains("forwarding=true");
            if has_display {
                println!("  SSH X11 forwarding detected - GUI tools available");
            } else {
                println!("  SSH session detected (no X11 forwarding)");
            }

            // Show tunnel instructions for Full mode
            if profile.recommended_monitoring_mode == "full" {
                println!();
                println!("  💡 To access Grafana dashboards, create an SSH tunnel:");
                println!("     ssh -L 3000:localhost:3000 user@host");
                println!("     Then browse to: http://localhost:3000");
            } else if profile.recommended_monitoring_mode == "light" {
                println!();
                println!("  💡 Prometheus metrics available at: http://localhost:9090");
                println!("     Create SSH tunnel: ssh -L 9090:localhost:9090 user@host");
            }
            println!();
        }

        println!("Timestamp: {}", profile.timestamp);
    }

    std::process::exit(EXIT_SUCCESS);
}

/// Execute monitor install command (Phase 3.0)
async fn execute_monitor_install_command(
    force_mode: Option<String>,
    dry_run: bool,
    socket_path: Option<&str>,
) -> Result<()> {
    // Connect to daemon to get system profile
    let mut client = match rpc_client::RpcClient::connect_with_path(socket_path).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to connect to daemon: {}", e);
            eprintln!("Make sure annad is running (try: sudo systemctl start annad)");
            std::process::exit(EXIT_DAEMON_UNAVAILABLE);
        }
    };

    // Get system profile
    let profile_response = client.get_profile().await?;
    let profile = match profile_response {
        ResponseData::Profile(data) => data,
        _ => anyhow::bail!("Unexpected response type for GetProfile"),
    };

    // Phase 3.2: Adaptive UI hint - warn if minimal mode
    if profile.recommended_monitoring_mode == "minimal" && force_mode.is_none() {
        println!("⚠️  Adaptive Intelligence Warning");
        println!("═══════════════════════════════════════════════════════════════");
        println!();
        println!("  Your system is running in MINIMAL mode due to limited resources.");
        println!("  Installing external monitoring tools (Prometheus/Grafana) is");
        println!("  NOT recommended as it may impact system performance.");
        println!();
        println!("  System Constraints:");
        println!(
            "    • RAM: {} MB (recommend >2GB for light mode)",
            profile.total_memory_mb
        );
        println!("    • CPU: {} cores", profile.cpu_cores);
        println!("    • Disk: {} GB available", profile.available_disk_gb);
        println!();
        println!("  Anna's internal monitoring is active and sufficient for your system.");
        println!("  Use 'annactl health' and 'annactl status' for system insights.");
        println!();
        println!("  To override this warning: annactl monitor install --force-mode <mode>");
        println!("═══════════════════════════════════════════════════════════════");
        println!();

        // Ask for confirmation
        eprint!("Continue anyway? [y/N]: ");
        use std::io::Write;
        std::io::stderr().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Installation cancelled.");
            return Ok(());
        }
        println!();
    }

    // Determine monitoring mode
    let mode = if let Some(forced) = force_mode {
        let normalized = forced.to_lowercase();
        if !["full", "light", "minimal"].contains(&normalized.as_str()) {
            eprintln!(
                "Error: Invalid mode '{}'. Must be: full, light, or minimal",
                forced
            );
            std::process::exit(EXIT_GENERAL_ERROR);
        }
        println!("⚠️  Using FORCED mode: {}", normalized.to_uppercase());
        println!(
            "   (System recommendation: {})\n",
            profile.recommended_monitoring_mode.to_uppercase()
        );
        normalized
    } else {
        profile.recommended_monitoring_mode.clone()
    };

    println!("Monitoring Stack Installation (Phase 3.0: Adaptive Intelligence)");
    println!("====================================================================\n");

    println!("System Profile:");
    println!(
        "  Memory: {} MB  |  CPU: {} cores  |  Constrained: {}",
        profile.total_memory_mb,
        profile.cpu_cores,
        if profile.is_constrained { "Yes" } else { "No" }
    );
    println!(
        "  Recommended Mode: {}",
        profile.recommended_monitoring_mode.to_uppercase()
    );
    println!();

    // Execute installation based on mode
    match mode.as_str() {
        "full" => monitor_setup::install_full_mode(dry_run)?,
        "light" => monitor_setup::install_light_mode(dry_run)?,
        "minimal" => monitor_setup::install_minimal_mode()?,
        _ => unreachable!("Mode validation should prevent this"),
    }

    println!();
    println!("Citation: [archwiki:prometheus][archwiki:grafana][observability:best-practices]");

    std::process::exit(EXIT_SUCCESS);
}

/// Execute monitor status command (Phase 3.0)
async fn execute_monitor_status_command(socket_path: Option<&str>) -> Result<()> {
    // Connect to daemon to get system profile
    let mut client = match rpc_client::RpcClient::connect_with_path(socket_path).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to connect to daemon: {}", e);
            eprintln!("Make sure annad is running (try: sudo systemctl start annad)");
            std::process::exit(EXIT_DAEMON_UNAVAILABLE);
        }
    };

    // Get system profile
    let profile_response = client.get_profile().await?;
    let profile = match profile_response {
        ResponseData::Profile(data) => data,
        _ => anyhow::bail!("Unexpected response type for GetProfile"),
    };

    println!("Monitoring Stack Status (Phase 3.0)");
    println!("====================================\n");

    println!(
        "System Mode: {}",
        profile.recommended_monitoring_mode.to_uppercase()
    );
    println!("Rationale:   {}\n", profile.monitoring_rationale);

    // Check Prometheus status
    let prometheus_active = monitor_setup::check_service_status("prometheus").unwrap_or(false);

    println!("Prometheus:");
    if prometheus_active {
        println!("  Status: ✓ Running");
        println!("  Access: http://localhost:9090");
    } else {
        println!("  Status: ✗ Not running");
        println!("  Install: annactl monitor install");
    }
    println!();

    // Check Grafana status (only for Full mode)
    if profile.recommended_monitoring_mode == "full" {
        let grafana_active = monitor_setup::check_service_status("grafana").unwrap_or(false);

        println!("Grafana:");
        if grafana_active {
            println!("  Status: ✓ Running");
            println!("  Access: http://localhost:3000");
        } else {
            println!("  Status: ✗ Not running");
            println!("  Install: annactl monitor install");
        }
        println!();
    }

    println!("Internal Stats: ✓ Available (via daemon)");
    println!("  Commands: annactl status, annactl health");
    println!();

    // Phase 3.2: Adaptive UI hints - mode-specific guidance
    match profile.recommended_monitoring_mode.as_str() {
        "minimal" => {
            println!("💡 Adaptive Intelligence Hint:");
            println!("   Your system is in MINIMAL mode. External monitoring tools are");
            println!("   not recommended due to limited resources. Anna's internal stats");
            println!("   provide all essential system health information.");
            println!();
            println!("   Recommended commands:");
            println!("   • annactl status     - View system state and recommendations");
            println!("   • annactl health     - Check system health metrics");
        }
        "light" => {
            println!("💡 Adaptive Intelligence Hint:");
            println!("   Your system is in LIGHT mode. Prometheus metrics are available");
            println!("   for advanced monitoring, but Grafana dashboards are not recommended");
            println!("   due to RAM or session constraints.");
            println!();
            println!("   Prometheus metrics: http://localhost:9090");
            println!("   Internal stats: annactl status, annactl health");
        }
        "full" => {
            println!("💡 Adaptive Intelligence Hint:");
            println!("   Your system is in FULL mode. All monitoring features are available");
            println!("   including Grafana dashboards for visualization.");
            println!();
            println!("   Grafana: http://localhost:3000");
            println!("   Prometheus: http://localhost:9090");
            println!("   Internal stats: annactl status, annactl health");
        }
        _ => {}
    }

    std::process::exit(EXIT_SUCCESS);
}

/// Execute metrics command (Phase 3.3)
async fn execute_metrics_command(
    prometheus: bool,
    json: bool,
    socket_path: Option<&str>,
) -> Result<()> {
    // Connect to daemon to get system profile
    let mut client = match rpc_client::RpcClient::connect_with_path(socket_path).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to connect to daemon: {}", e);
            eprintln!("Make sure annad is running (try: sudo systemctl start annad)");
            std::process::exit(EXIT_DAEMON_UNAVAILABLE);
        }
    };

    // Get system profile
    let profile_response = client.get_profile().await?;
    let profile = match profile_response {
        ResponseData::Profile(data) => data,
        _ => anyhow::bail!("Unexpected response type for GetProfile"),
    };

    if json {
        // JSON output
        let json_output = serde_json::json!({
            "total_memory_mb": profile.total_memory_mb,
            "available_memory_mb": profile.available_memory_mb,
            "cpu_cores": profile.cpu_cores,
            "total_disk_gb": profile.total_disk_gb,
            "available_disk_gb": profile.available_disk_gb,
            "uptime_seconds": profile.uptime_seconds,
            "monitoring_mode": profile.recommended_monitoring_mode,
            "is_constrained": profile.is_constrained,
            "timestamp": profile.timestamp
        });
        println!("{}", serde_json::to_string_pretty(&json_output)?);
    } else if prometheus {
        // Prometheus exposition format
        println!("# HELP anna_system_memory_total_mb Total system memory in MB");
        println!("# TYPE anna_system_memory_total_mb gauge");
        println!("anna_system_memory_total_mb {}", profile.total_memory_mb);
        println!();

        println!("# HELP anna_system_memory_available_mb Available system memory in MB");
        println!("# TYPE anna_system_memory_available_mb gauge");
        println!(
            "anna_system_memory_available_mb {}",
            profile.available_memory_mb
        );
        println!();

        println!("# HELP anna_system_cpu_cores Number of CPU cores");
        println!("# TYPE anna_system_cpu_cores gauge");
        println!("anna_system_cpu_cores {}", profile.cpu_cores);
        println!();

        println!("# HELP anna_system_disk_total_gb Total disk space in GB");
        println!("# TYPE anna_system_disk_total_gb gauge");
        println!("anna_system_disk_total_gb {}", profile.total_disk_gb);
        println!();

        println!("# HELP anna_system_disk_available_gb Available disk space in GB");
        println!("# TYPE anna_system_disk_available_gb gauge");
        println!(
            "anna_system_disk_available_gb {}",
            profile.available_disk_gb
        );
        println!();

        println!("# HELP anna_system_uptime_seconds System uptime in seconds");
        println!("# TYPE anna_system_uptime_seconds gauge");
        println!("anna_system_uptime_seconds {}", profile.uptime_seconds);
        println!();

        let mode_value = match profile.recommended_monitoring_mode.as_str() {
            "minimal" => 0,
            "light" => 1,
            "full" => 2,
            _ => 1, // default to light
        };
        println!("# HELP anna_profile_mode Monitoring mode (0=minimal, 1=light, 2=full)");
        println!("# TYPE anna_profile_mode gauge");
        println!("anna_profile_mode {}", mode_value);
        println!();

        let constrained_value = if profile.is_constrained { 1 } else { 0 };
        println!("# HELP anna_profile_constrained Resource-constrained status (0=no, 1=yes)");
        println!("# TYPE anna_profile_constrained gauge");
        println!("anna_profile_constrained {}", constrained_value);
    } else {
        // Human-readable output
        println!("System Metrics (Phase 3.3: Adaptive Intelligence)");
        println!("==================================================\n");

        println!("Memory:");
        println!("  Total:     {} MB", profile.total_memory_mb);
        println!(
            "  Available: {} MB ({:.1}%)",
            profile.available_memory_mb,
            (profile.available_memory_mb as f64 / profile.total_memory_mb as f64) * 100.0
        );
        println!();

        println!("CPU:");
        println!("  Cores: {}", profile.cpu_cores);
        println!();

        println!("Disk:");
        println!("  Total:     {} GB", profile.total_disk_gb);
        println!(
            "  Available: {} GB ({:.1}%)",
            profile.available_disk_gb,
            (profile.available_disk_gb as f64 / profile.total_disk_gb as f64) * 100.0
        );
        println!();

        println!("System:");
        println!(
            "  Uptime: {} seconds ({:.1} hours)",
            profile.uptime_seconds,
            profile.uptime_seconds as f64 / 3600.0
        );
        println!();

        println!("Adaptive Intelligence:");
        println!(
            "  Mode:        {}",
            profile.recommended_monitoring_mode.to_uppercase()
        );
        println!(
            "  Constrained: {}",
            if profile.is_constrained { "Yes" } else { "No" }
        );
        println!("  Rationale:   {}", profile.monitoring_rationale);
        println!();

        println!("Timestamp: {}", profile.timestamp);
        println!();

        println!("💡 Tip: Use --prometheus for Prometheus format, --json for JSON");
    }

    std::process::exit(EXIT_SUCCESS);
}

/// Check system resources and warn if constrained (Phase 3.4)
async fn check_resource_constraints(socket_path: Option<&str>, operation: &str) -> Result<bool> {
    // Try to get profile from daemon
    let mut client = match rpc_client::RpcClient::connect_with_path(socket_path).await {
        Ok(c) => c,
        Err(_) => return Ok(true), // If daemon unavailable, proceed anyway
    };

    let profile_response = client.get_profile().await?;
    let profile = match profile_response {
        ResponseData::Profile(data) => data,
        _ => return Ok(true), // If unexpected response, proceed anyway
    };

    // Check if system is resource-constrained
    if profile.is_constrained {
        println!("⚠️  Resource Constraint Warning");
        println!("═══════════════════════════════════════════════════════════════");
        println!();
        println!("  Your system is resource-constrained:");
        println!(
            "    • RAM: {} MB available of {} MB total ({:.1}%)",
            profile.available_memory_mb,
            profile.total_memory_mb,
            (profile.available_memory_mb as f64 / profile.total_memory_mb as f64) * 100.0
        );
        println!("    • CPU: {} cores", profile.cpu_cores);
        println!("    • Disk: {} GB available", profile.available_disk_gb);
        println!();
        println!("  Operation '{}' may:", operation);
        println!("    - Consume significant system resources");
        println!("    - Take longer than usual to complete");
        println!("    - Impact system responsiveness");
        println!();
        println!("  Consider:");
        println!("    - Closing other applications");
        println!("    - Running during off-peak hours");
        println!("    - Using --dry-run flag to preview changes");
        println!("═══════════════════════════════════════════════════════════════");
        println!();

        // Ask for confirmation
        eprint!("Proceed with operation? [y/N]: ");
        use std::io::Write;
        std::io::stderr().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Operation cancelled.");
            return Ok(false);
        }
        println!();
    }

    Ok(true)
}

/// Get state and capabilities from daemon (Phase 0.3d)
async fn get_state_and_capabilities(
    socket_path: Option<&str>,
) -> Result<(String, String, Vec<CommandCapabilityData>)> {
    let mut client = rpc_client::RpcClient::connect_with_path(socket_path).await?;

    // Get state
    let state_response = client.get_state().await?;
    let state_data = match state_response {
        ResponseData::StateDetection(data) => data,
        _ => anyhow::bail!("Unexpected response type for GetState"),
    };

    // Get capabilities
    let caps_response = client.get_capabilities().await?;
    let caps_data = match caps_response {
        ResponseData::Capabilities(data) => data,
        _ => anyhow::bail!("Unexpected response type for GetCapabilities"),
    };

    // Return state name, state citation, and full capabilities list
    // Phase 3.0: Extract commands from CapabilitiesData
    let citation = state_citation(&state_data.state);
    Ok((state_data.state, citation.to_string(), caps_data.commands))
}

/// Execute help command standalone (Phase 3.1 - doesn't require daemon)
// Beta.89: This function is deprecated - help is now natural language only
#[allow(dead_code)]
async fn execute_help_command_standalone(
    _command: &Commands,
    _socket_path: Option<&str>,
    _req_id: &str,
    _start_time: Instant,
) -> Result<()> {
    // Beta.89: Help command removed
    // Users should use natural language: annactl "how do I use this?"

    // Display minimal modern help
    println!("\nAnna Assistant - Arch Linux System Administration\n");
    println!("Anna is your intelligent system administrator that monitors,");
    println!("maintains, and helps you understand your Arch Linux system.\n");
    println!("Available commands:");
    println!("  annactl           Start interactive session with Anna");
    println!("  annactl status    Show Anna's health report");
    println!("  annactl help      Show this help message\n");
    println!("Usage:");
    println!("  annactl           # Start REPL - just talk to Anna");
    println!("  annactl status    # Check system health");
    println!("  annactl help      # Show this help\n");
    println!("Examples (in REPL):");
    println!("  > How is my system?");
    println!("  > Any problems I should know about?");
    println!("  > Tell me about my computer");
    println!("  > What's using all my disk space?\n");

    // Beta.89: Logging disabled for deprecated function
    // TODO: Remove this entire function in Beta.90
    /*
    let duration_ms = _start_time.elapsed().as_millis() as u64;
    let log_entry = LogEntry {
        ts: LogEntry::now(),
        req_id: _req_id.to_string(),
        state: "unknown".to_string(),
        command: "help".to_string(),
        allowed: Some(true),
        args: vec![],
        exit_code: EXIT_SUCCESS,
        citation: "[archwiki:System_maintenance]".to_string(),
        duration_ms,
        ok: true,
        error: None,
    };
    let _ = log_entry.write();
    */

    Ok(())
}

// LEGACY: /// Execute help command (Phase 0.3d - Legacy, requires daemon)
// LEGACY: async fn execute_help_command(
// LEGACY:     command: &Commands,
// LEGACY:     state: &str,
// LEGACY:     capabilities: &[CommandCapabilityData],
// LEGACY:     req_id: &str,
// LEGACY:     start_time: Instant,
// LEGACY: ) -> Result<()> {
// LEGACY:     let (cmd_name, show_all, json_only) = match command {
// LEGACY:         Commands::Help { command, all, json } => (command.clone(), *all, *json),
// LEGACY:         _ => unreachable!(),
// LEGACY:     };
// LEGACY:
// LEGACY:     // Phase 3.1: Use adaptive help system if not JSON mode
// LEGACY:     if !json_only {
// LEGACY:         // Build display context from current system state
// LEGACY:         let socket_path = std::env::var("ANNAD_SOCKET").ok();
// LEGACY:         let context = help_commands::build_context(socket_path.as_deref()).await;
// LEGACY:
// LEGACY:         // Display adaptive help
// LEGACY:         if let Err(e) = help_commands::display_help(cmd_name, show_all, context).await {
// LEGACY:             eprintln!("Error displaying help: {}", e);
// LEGACY:             std::process::exit(1);
// LEGACY:         }
// LEGACY:
// LEGACY:         // Log the help command
// LEGACY:         let duration_ms = start_time.elapsed().as_millis() as u64;
// LEGACY:         let log_entry = LogEntry {
// LEGACY:             ts: LogEntry::now(),
// LEGACY:             req_id: req_id.to_string(),
// LEGACY:             state: state.to_string(),
// LEGACY:             command: "help".to_string(),
// LEGACY:             allowed: Some(true),
// LEGACY:             args: if show_all {
// LEGACY:                 vec!["--all".to_string()]
// LEGACY:             } else {
// LEGACY:                 vec![]
// LEGACY:             },
// LEGACY:             exit_code: EXIT_SUCCESS,
// LEGACY:             citation: state_citation(state).to_string(),
// LEGACY:             duration_ms,
// LEGACY:             ok: true,
// LEGACY:             error: None,
// LEGACY:         };
// LEGACY:         let _ = log_entry.write();
// LEGACY:
// LEGACY:         std::process::exit(EXIT_SUCCESS);
// LEGACY:     }
// LEGACY:
// LEGACY:     // Legacy JSON output (for backwards compatibility)
// LEGACY:     let mut sorted_caps = capabilities.to_vec();
// LEGACY:     sorted_caps.sort_by(|a, b| a.name.cmp(&b.name));
// LEGACY:
// LEGACY:     let commands: Vec<serde_json::Value> = sorted_caps
// LEGACY:         .iter()
// LEGACY:         .map(|cap| {
// LEGACY:             serde_json::json!({
// LEGACY:                 "name": cap.name,
// LEGACY:                 "desc": cap.description,
// LEGACY:                 "citation": cap.citation
// LEGACY:             })
// LEGACY:         })
// LEGACY:         .collect();
// LEGACY:
// LEGACY:     let output = serde_json::json!({
// LEGACY:         "version": VERSION,
// LEGACY:         "ok": true,
// LEGACY:         "state": state,
// LEGACY:         "commands": commands
// LEGACY:     });
// LEGACY:     println!("{}", serde_json::to_string_pretty(&output)?);
// LEGACY:
// LEGACY:     // Log the help command
// LEGACY:     let duration_ms = start_time.elapsed().as_millis() as u64;
// LEGACY:     let log_entry = LogEntry {
// LEGACY:         ts: LogEntry::now(),
// LEGACY:         req_id: req_id.to_string(),
// LEGACY:         state: state.to_string(),
// LEGACY:         command: "help".to_string(),
// LEGACY:         allowed: Some(true),
// LEGACY:         args: vec!["--json".to_string()],
// LEGACY:         exit_code: EXIT_SUCCESS,
// LEGACY:         citation: state_citation(state).to_string(),
// LEGACY:         duration_ms,
// LEGACY:         ok: true,
// LEGACY:         error: None,
// LEGACY:     };
// LEGACY:     let _ = log_entry.write();
// LEGACY:
// LEGACY:     std::process::exit(EXIT_SUCCESS);
// LEGACY: }
// LEGACY:
// LEGACY: /// Execute no-op command handler (Phase 0.3c)
// LEGACY: /// All commands just print success message and return exit code
// LEGACY: async fn execute_noop_command(command: &Commands, state: &str) -> Result<i32> {
// LEGACY:     match command {
// LEGACY:     Ok(EXIT_SUCCESS)
// LEGACY: }
