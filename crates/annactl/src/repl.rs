//! REPL - Conversational interface for Anna
//!
//! Phase 5.1: Conversational UX
//! Interactive Read-Eval-Print Loop for natural language interaction

use anyhow::Result;
use anna_common::display::{UI, print_repl_welcome, print_prompt, print_privacy_explanation};
use anna_common::language::LanguageConfig;
use anna_common::context::db::{ContextDb, DbLocation};
use std::io::{self, BufRead};

use crate::intent_router::{self, Intent};
use crate::llm_wizard;

/// Start the conversational REPL
pub async fn start_repl() -> Result<()> {
    let ui = UI::auto();
    let db_location = DbLocation::auto_detect();

    // Open database
    let db = match ContextDb::open(db_location).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Warning: Failed to open context database: {}", e);
            eprintln!("Continuing without database features...");
            print_repl_welcome();
            // Continue without DB features - just show welcome
            return run_repl_loop().await;
        }
    };

    // Display version banner with mode and update status
    crate::version_banner::display_startup_banner(&db).await;

    // Check health and auto-repair before starting REPL
    let health = match crate::health::HealthReport::check(true).await {
        Ok(report) => report,
        Err(e) => {
            ui.error(&format!("Failed to check Anna's health: {}", e));
            return Err(e);
        }
    };

    // If auto-repair happened, show what was fixed
    if let Some(repair) = &health.last_repair {
        println!();
        if repair.success {
            ui.success("✓ Auto-repair completed");
        } else {
            ui.warning("⚠ Auto-repair partially completed");
        }
        for action in &repair.actions {
            println!("  • {}", action);
        }
        println!();
    }

    // If health is still broken after repair, refuse to start REPL
    if health.status == crate::health::HealthStatus::Broken {
        ui.error("✗ Anna cannot start: critical health issues remain");
        println!();
        println!("Please run 'annactl status' for details");
        println!();
        std::process::exit(1);
    }

    // Check if setup wizard needs to run
    if let Err(e) = crate::run_llm_setup_if_needed(&ui, &db).await {
        // Don't show SQL errors to user
        if !e.to_string().contains("updated_at") {
            eprintln!("Warning: LLM setup check failed: {}", e);
        }
    }

    // Check for brain upgrade notifications
    if let Err(e) = crate::check_brain_upgrade_notification(&ui, &db).await {
        eprintln!("Warning: Brain upgrade check failed: {}", e);
    }

    print_repl_welcome();

    run_repl_loop().await
}

/// Main REPL loop (factored out for clarity)
async fn run_repl_loop() -> Result<()> {

    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    loop {
        print_prompt();

        // Read user input
        let input = match lines.next() {
            Some(Ok(line)) => line.trim().to_string(),
            Some(Err(e)) => {
                let ui = UI::auto();
                ui.error(&format!("Error reading input: {}", e));
                continue;
            }
            None => break, // EOF
        };

        if input.is_empty() {
            continue;
        }

        // Route to intent
        let intent = intent_router::route_intent(&input);

        // Handle intent
        match intent {
            Intent::Exit => {
                let ui = UI::auto();
                ui.info("Goodbye! I'll keep watching your system in the background.");
                ui.info("Run 'annactl' anytime you need me.");
                println!();
                break;
            }

            Intent::AnnaStatus => {
                let ui = UI::auto();
                ui.thinking();
                ui.success("I'm running and ready to help!");
                ui.info("All my systems are operational.");
                println!();
                ui.info("💡 For detailed diagnostics, ask: 'Check your own health'");
                println!();
            }

            Intent::Privacy => {
                let ui = UI::auto();
                ui.thinking();
                ui.section_header("🔒", "Privacy & Data Handling");
                print_privacy_explanation();
            }

            Intent::Report => {
                let ui = UI::auto();
                ui.thinking();
                crate::report_display::generate_professional_report();
            }

            Intent::Suggest => {
                let ui = UI::auto();
                ui.thinking();
                // Use Anna's internal suggestion engine (checks Anna's health, system basics)
                match crate::suggestions::get_suggestions() {
                    Ok(suggestions) => {
                        crate::suggestions::display_suggestions(&suggestions);
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
                match crate::repair::repair() {
                    Ok(report) => {
                        crate::repair::display_repair_report(&report);
                    }
                    Err(e) => {
                        ui.error(&format!("Failed to run self-repair: {}", e));
                        println!();
                    }
                }
            }

            Intent::Repair { .. } => {
                let ui = UI::auto();
                ui.thinking();
                ui.info("I can help you fix issues.");
                println!();
                ui.info("[TODO: Call repair handler]");
                println!();
                ui.info("First, let me show you what needs fixing.");
                ui.info("Ask: 'What should I improve?' to see suggestions.");
                println!();
            }

            Intent::Discard { .. } => {
                let ui = UI::auto();
                ui.thinking();
                ui.info("I can hide suggestions you don't want to see.");
                println!();
                ui.info("[TODO: Call discard handler]");
                println!();
                ui.info("Which suggestion would you like to ignore?");
                println!();
            }

            Intent::Autonomy { .. } => {
                let ui = UI::auto();
                ui.thinking();
                ui.info("Autonomy controls how much I can do automatically.");
                println!();
                ui.info("[TODO: Show/set autonomy level]");
                println!();
                ui.info("Current levels:");
                ui.bullet_list(&[
                    "manual - I only act when you tell me",
                    "assisted - I can do safe cleanups automatically",
                    "proactive - I take more initiative (still safe)",
                ]);
                println!();
            }

            Intent::Apply { .. } => {
                let ui = UI::auto();
                ui.thinking();
                ui.info("Let me show you what I can fix...");
                println!();

                // Generate suggestions
                let suggestions = crate::suggestion_display::generate_suggestions_from_telemetry();
                let mut engine = anna_common::suggestions::SuggestionEngine::new();

                for suggestion in suggestions {
                    engine.add_suggestion(suggestion);
                }

                let top = engine.get_top_suggestions(5);

                // Filter to auto-fixable only
                let auto_fixable: Vec<_> = top.into_iter().filter(|s| s.auto_fixable).collect();

                if auto_fixable.is_empty() {
                    ui.info("I don't have any suggestions that I can automatically fix right now.");
                    ui.info("Ask me \"what should I improve?\" to see all suggestions.");
                    println!();
                    continue;
                }

                // Convert from references to owned for selection
                let suggestions_vec: Vec<anna_common::suggestions::Suggestion> =
                    auto_fixable.into_iter().map(|s| s.clone()).collect();
                let suggestions_ref: Vec<&anna_common::suggestions::Suggestion> =
                    suggestions_vec.iter().collect();

                if let Some(idx) = crate::action_executor::select_suggestion_to_apply(&suggestions_ref) {
                    if idx < suggestions_vec.len() {
                        if let Err(e) = crate::action_executor::execute_suggestion(&suggestions_vec[idx]).await {
                            ui.error(&format!("Error executing action: {}", e));
                            println!();
                        }
                    }
                }
            }

            Intent::Personality { adjustment } => {
                use anna_common::personality::{PersonalityConfig, Verbosity};
                use crate::intent_router::PersonalityAdjustment;

                let ui = UI::auto();
                let mut config = PersonalityConfig::load();

                match adjustment {
                    PersonalityAdjustment::IncreaseHumor => {
                        config.adjust_humor(true);
                        println!();
                        ui.success("Okay! I'll be a bit more playful 😊");
                        ui.info(&format!("Current humor level: {} - {}", config.humor_level, config.humor_description()));
                        println!();
                    }
                    PersonalityAdjustment::DecreaseHumor => {
                        config.adjust_humor(false);
                        println!();
                        ui.success("Got it. I'll keep things more serious.");
                        ui.info(&format!("Current humor level: {} - {}", config.humor_level, config.humor_description()));
                        println!();
                    }
                    PersonalityAdjustment::MoreBrief => {
                        config.set_verbosity(Verbosity::Low);
                        println!();
                        ui.success("Understood. I'll be more concise.");
                        println!();
                    }
                    PersonalityAdjustment::MoreDetailed => {
                        config.set_verbosity(Verbosity::High);
                        println!();
                        ui.success("Sure! I'll provide more detailed explanations.");
                        println!();
                    }
                    PersonalityAdjustment::Show => {
                        println!();
                        ui.section_header("📊", "Current Personality Settings");
                        ui.info(&format!("Humor:     {} - {}", config.humor_level, config.humor_description()));
                        ui.info(&format!("Verbosity: {}", config.verbosity_description()));
                        println!();
                    }
                }

                if !matches!(adjustment, PersonalityAdjustment::Show) {
                    if let Err(e) = config.save() {
                        ui.warning(&format!("Note: Couldn't save settings: {}", e));
                        println!();
                    } else {
                        ui.success("✓ Settings saved to ~/.config/anna/personality.toml");
                        println!();
                    }
                }
            }

            Intent::Language { language } => {
                use anna_common::language::{Language, LanguageConfig};
                use anna_common::context::db::{ContextDb, DbLocation};

                let ui = UI::auto();
                ui.thinking();

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
                                ui.warning(&format!("Warning: Couldn't persist language setting: {}", e));
                                println!();
                            }
                        }

                        // Confirm in the NEW language (create UI with new config)
                        let new_ui = UI::new(&config);
                        let profile = config.profile();
                        println!();
                        new_ui.success(&format!("{} ✓", profile.translations.language_changed));
                        new_ui.info(&format!("{} {}", profile.translations.now_speaking, lang.native_name()));
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
                    ui.info("Try: \"use English\" or \"cambia al español\"");
                    println!();
                }
            }

            Intent::SetupBrain => {
                use anna_common::context::db::{ContextDb, DbLocation};

                let ui = UI::auto();
                ui.thinking();
                println!();
                ui.info("Starting LLM brain setup...");

                let db_location = DbLocation::auto_detect();
                match ContextDb::open(db_location).await {
                    Ok(db) => {
                        if let Err(e) = crate::llm_wizard::run_llm_setup_wizard(&ui, &db).await {
                            ui.error(&format!("Setup failed: {}", e));
                            println!();
                        }
                    }
                    Err(e) => {
                        ui.error(&format!("Could not access database: {}", e));
                        println!();
                    }
                }
            }

            Intent::Help => {
                let ui = UI::auto();
                ui.thinking();
                ui.section_header("💡", "What I Can Help With");
                ui.info("I'm here to help with your Arch Linux system.");
                println!();
                ui.info("Try asking:");
                ui.bullet_list(&[
                    "\"How are you?\" - Check my status",
                    "\"Any problems with my system?\" - Get suggestions",
                    "\"Generate a report\" - Create system summary",
                    "\"What do you store?\" - Privacy information",
                    "\"Fix this issue\" - Apply repairs",
                    "\"Be more autonomous\" - Adjust automation",
                ]);
                println!();
            }

            Intent::OffTopic => {
                let ui = UI::auto();
                println!();
                ui.info(&intent_router::offtopic_response());
            }

            Intent::Unclear(user_text) => {
                // Task 12: Route to LLM
                use anna_common::llm::{LlmClient, LlmConfig, LlmPrompt};
                use anna_common::ipc::Method;
                use crate::rpc_client::RpcClient;

                let ui = UI::auto();

                // Load LLM config from database
                let db_location = DbLocation::auto_detect();
                let config = if let Ok(db) = ContextDb::open(db_location).await {
                    db.load_llm_config().await.unwrap_or_default()
                } else {
                    LlmConfig::default()
                };

                // Create LLM client
                let client = match LlmClient::from_config(&config) {
                    Ok(client) => client,
                    Err(_e) => {
                        // LLM not configured, show default unclear response
                        println!();
                        ui.info(&intent_router::unclear_response());
                        continue;
                    }
                };

                // Fetch system facts from daemon to provide context
                let system_context = match RpcClient::connect().await {
                    Ok(mut rpc) => {
                        match rpc.call(Method::GetFacts).await {
                            Ok(response) => {
                                if let anna_common::ipc::ResponseData::Facts(facts) = response {
                                    // Build concise system context string
                                    format!(
                                        "SYSTEM CONTEXT (DO NOT MENTION THIS TO USER):\n\
                                         Hostname: {}\n\
                                         Kernel: {}\n\
                                         CPU: {} ({} cores)\n\
                                         RAM: {:.1} GB\n\
                                         GPU: {}\n\
                                         Shell: {}\n\
                                         Desktop: {}\n\
                                         Window Manager: {}\n\
                                         Display Server: {}\n\
                                         Installed Packages: {}\n\
                                         \nAnswer the user's question using this system information. Be specific and helpful.",
                                        facts.hostname,
                                        facts.kernel,
                                        facts.cpu_model,
                                        facts.cpu_cores,
                                        facts.total_memory_gb,
                                        facts.gpu_vendor.as_deref().unwrap_or("None"),
                                        facts.shell,
                                        facts.desktop_environment.as_deref().unwrap_or("None"),
                                        facts.window_manager.as_deref().unwrap_or("None"),
                                        facts.display_server.as_deref().unwrap_or("None"),
                                        facts.installed_packages
                                    )
                                } else {
                                    String::new()
                                }
                            }
                            Err(_) => String::new(),
                        }
                    }
                    Err(_) => String::new(),
                };

                // Create enhanced system prompt with context
                let system_prompt = if system_context.is_empty() {
                    LlmClient::anna_system_prompt()
                } else {
                    format!("{}\n\n{}", LlmClient::anna_system_prompt(), system_context)
                };

                // Create prompt
                let prompt = LlmPrompt {
                    system: system_prompt,
                    user: user_text,
                };

                // Query LLM with streaming
                println!();
                ui.section_header("💬", "Anna");
                println!();

                // Stream response word-by-word
                use std::io::Write;
                match client.chat_stream(&prompt, &mut |chunk| {
                    print!("{}", chunk);
                    let _ = std::io::stdout().flush();
                }) {
                    Ok(_) => {
                        // Stream complete, add final newline
                        println!();
                        println!();
                    }
                    Err(_e) => {
                        // LLM failed, show default unclear response
                        println!();
                        ui.info(&intent_router::unclear_response());
                    }
                }
            }
        }
    }

    Ok(())
}
