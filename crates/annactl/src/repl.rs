//! REPL - Conversational interface for Anna
//!
//! Phase 5.1: Conversational UX
//! Interactive Read-Eval-Print Loop for natural language interaction

use anyhow::Result;
use anna_common::display::{UI, print_repl_welcome, print_prompt, print_privacy_explanation};
use anna_common::language::LanguageConfig;
use std::io::{self, BufRead};

use crate::intent_router::{self, Intent};

/// Start the conversational REPL
pub async fn start_repl() -> Result<()> {
    print_repl_welcome();

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
                crate::suggestion_display::show_suggestions_conversational();
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

            Intent::Unclear => {
                let ui = UI::auto();
                println!();
                ui.info(&intent_router::unclear_response());
            }
        }
    }

    Ok(())
}
