//! REPL - Conversational interface for Anna
//!
//! Phase 5.1: Conversational UX
//! Interactive Read-Eval-Print Loop for natural language interaction

use anyhow::Result;
use anna_common::display::{UI, print_repl_welcome, print_prompt, print_privacy_explanation};
use anna_common::context::db::{ContextDb, DbLocation};
use std::io::{self, BufRead};

use crate::intent_router::{self, Intent};

/// Start the conversational REPL
pub async fn start_repl() -> Result<()> {
    let db_location = DbLocation::auto_detect();

    // Open database and load language config
    let (db, lang_config) = match ContextDb::open(db_location).await {
        Ok(db) => {
            // Load saved language config from database
            let config = db.load_language_config().await.unwrap_or_default();
            (db, config)
        },
        Err(e) => {
            eprintln!("Warning: Failed to open context database: {}", e);
            eprintln!("Continuing without database features...");
            // Use default config (English) if database not available
            let config = anna_common::language::LanguageConfig::new();
            let ui = UI::new(&config);
            ui.repl_welcome();
            // Continue without DB features
            return run_repl_loop().await;
        }
    };

    // Create UI with loaded language config
    let ui = UI::new(&lang_config);

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

    // Show welcome message in user's language
    ui.repl_welcome();

    // Proactive startup summary - inform user of any issues
    display_startup_summary(&ui).await;

    run_repl_loop().await
}

/// Display proactive startup summary with system issues
async fn display_startup_summary(ui: &UI) {
    use crate::rpc_client::RpcClient;
    use anna_common::ipc::Method;

    // Fetch system facts from daemon
    let facts = match RpcClient::connect().await {
        Ok(mut rpc) => {
            match rpc.call(Method::GetFacts).await {
                Ok(anna_common::ipc::ResponseData::Facts(f)) => Some(f),
                _ => None,
            }
        },
        Err(_) => None,
    };

    let facts = match facts {
        Some(f) => f,
        None => return, // Can't connect to daemon, skip summary
    };

    let mut issues: Vec<String> = Vec::new();

    // Check for failed services (CRITICAL)
    if !facts.failed_services.is_empty() {
        let count = facts.failed_services.len();
        let services_list = if count <= 3 {
            facts.failed_services.join(", ")
        } else {
            format!("{}, {} and {} more",
                facts.failed_services[0],
                facts.failed_services[1],
                count - 2)
        };
        issues.push(format!("⚠️  {} failed service{}: {}",
            count,
            if count == 1 { "" } else { "s" },
            services_list));
    }

    // Check for critical disk usage (>90%)
    for disk in &facts.storage_devices {
        let usage_pct = (disk.used_gb / disk.size_gb) * 100.0;
        if usage_pct > 90.0 {
            issues.push(format!("💾 {} is {:.0}% full ({:.1}/{:.1} GB)",
                disk.mount_point, usage_pct, disk.used_gb, disk.size_gb));
        }
    }

    // Check for slow boot time (>60s)
    if let Some(boot_time) = facts.boot_time_seconds {
        if boot_time > 60.0 {
            let slow = if !facts.slow_services.is_empty() {
                format!(" (slowest: {})", facts.slow_services[0].name)
            } else {
                String::new()
            };
            issues.push(format!("🐌 Slow boot time: {:.1}s{}", boot_time, slow));
        }
    }

    // Check for orphaned packages (>50)
    if facts.orphan_packages.len() > 50 {
        issues.push(format!("📦 {} orphaned packages can be removed",
            facts.orphan_packages.len()));
    }

    // Check for large package cache (>10GB)
    if facts.package_cache_size_gb > 10.0 {
        issues.push(format!("🗑️  Package cache: {:.1} GB (consider cleanup)",
            facts.package_cache_size_gb));
    }

    // Display summary if there are issues
    if !issues.is_empty() {
        ui.warning("System Status:");
        for issue in &issues {
            println!("  {}", issue);
        }

        // Offer help
        if !facts.failed_services.is_empty() || issues.iter().any(|i| i.contains("full")) {
            ui.info("💡 Ask me for suggestions to fix these issues");
        }
    } else {
        // Everything is fine - show brief positive message
        ui.success("System is healthy");
    }
}

/// Main REPL loop (factored out for clarity)
async fn run_repl_loop() -> Result<()> {
    use anna_common::llm::ChatMessage;

    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    // Conversation memory - stores full conversation history for LLM context
    let mut conversation_history: Vec<ChatMessage> = Vec::new();

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
                    auto_fixable.into_iter().cloned().collect();
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
                // Task 12: Route to LLM with conversation memory
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
                                    // Serialize complete system facts as JSON for full context
                                    match serde_json::to_string_pretty(&facts) {
                                        Ok(json_facts) => {
                                            format!(
                                                "SYSTEM CONTEXT (Complete system telemetry data - DO NOT mention reading this JSON to the user):\n\
                                                 \n\
                                                 {}\n\
                                                 \n\
                                                 CRITICAL ANTI-HALLUCINATION RULES:\n\
                                                 1. ONLY state facts explicitly present in the JSON above\n\
                                                 2. If a field is null, empty string, or empty array: DO NOT claim it exists\n\
                                                 3. Examples of what NOT to do:\n\
                                                    ❌ If window_manager is null → DON'T say \"you're running [any WM]\"\n\
                                                    ❌ If desktop_environment is null → DON'T say \"you're running [any DE]\"\n\
                                                    ❌ If display_server is \"Wayland\" → DON'T say \"you're running X11/Xorg\"\n\
                                                 4. When a field is empty/null, you can say \"I don't see any [thing] installed\"\n\
                                                 5. Check the EXACT values in: window_manager, desktop_environment, display_server\n\
                                                 6. Failed services are in 'failed_services' array - if empty, there are NONE\n\
                                                 \n\
                                                 ANSWER FROM DATA FIRST:\n\
                                                 - If user asks \"what GPU/nvidia card do I have?\" → Check gpu_model and gpu_vram_mb fields and TELL THEM directly\n\
                                                 - If user asks about CPU → Tell them from cpu_model field\n\
                                                 - If user asks about RAM → Tell them from total_memory_gb field\n\
                                                 - If user asks about disk/storage → Tell them from storage_devices array\n\
                                                 - DON'T suggest commands like 'lspci' or 'free' when the answer is ALREADY in the JSON\n\
                                                 - ONLY suggest commands if the data is NOT in the JSON or user explicitly asks how to check\n\
                                                 \n\
                                                 PACMAN COMMAND RULES (CRITICAL - DO NOT MAKE UP OPTIONS):\n\
                                                 - For orphan packages: Use 'sudo pacman -Rns $(pacman -Qtdq)' - NOT 'pacman -Ds' (doesn't exist!)\n\
                                                 - For package cache cleanup: Use 'sudo pacman -Sc' or 'paccache -r'\n\
                                                 - For installed packages: Use 'pacman -Q' (query) - NOT 'pacman -D'\n\
                                                 - NEVER invent pacman options - if unsure, don't suggest the command\n\
                                                 - Valid pacman operations: -S (sync/install), -R (remove), -Q (query), -U (upgrade), -F (files)\n\
                                                 \n\
                                                 Response Guidelines:\n\
                                                 - Be specific using ACTUAL data from JSON (GPU model, service names, versions)\n\
                                                 - Check failed_services, slow_services, orphan_packages, storage_devices for issues\n\
                                                 - Don't mention JSON or data structures - speak naturally as Anna\n\
                                                 - If you're unsure, check the JSON field value again before answering\n\
                                                 - NEVER make assumptions based on \"typical Arch Linux setups\"",
                                                json_facts
                                            )
                                        }
                                        Err(_) => String::new()
                                    }
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

                // Build conversation history with system message + all previous messages + current user message
                let mut messages = vec![
                    ChatMessage {
                        role: "system".to_string(),
                        content: system_prompt.clone(),
                    }
                ];

                // Add all previous conversation turns
                messages.extend(conversation_history.clone());

                // Add current user message
                messages.push(ChatMessage {
                    role: "user".to_string(),
                    content: user_text.clone(),
                });

                // Create prompt with conversation history
                let prompt = LlmPrompt {
                    system: system_prompt,
                    user: user_text.clone(),
                    conversation_history: Some(messages),
                };

                // Query LLM with streaming
                println!();
                ui.section_header("💬", "Anna");
                println!();

                // Capture assistant response for conversation memory
                let mut assistant_response = String::new();

                // Stream response word-by-word
                use std::io::Write;
                match client.chat_stream(&prompt, &mut |chunk| {
                    print!("{}", chunk);
                    let _ = std::io::stdout().flush();
                    assistant_response.push_str(chunk);  // Capture for memory
                }) {
                    Ok(_) => {
                        // Stream complete, add final newline
                        println!();
                        println!();

                        // Add both user and assistant messages to conversation history
                        conversation_history.push(ChatMessage {
                            role: "user".to_string(),
                            content: user_text,
                        });
                        conversation_history.push(ChatMessage {
                            role: "assistant".to_string(),
                            content: assistant_response,
                        });

                        // Keep conversation history to last 10 turns (20 messages: 10 user + 10 assistant)
                        // to prevent context from growing too large
                        if conversation_history.len() > 20 {
                            conversation_history.drain(0..(conversation_history.len() - 20));
                        }
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
