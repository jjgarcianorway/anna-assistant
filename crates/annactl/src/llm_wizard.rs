//! First-Run LLM Setup Wizard
//!
//! Guides the user through setting up Anna's "brain" - either a local LLM
//! (privacy-first, automatic) or a remote API (opt-in with warnings).

use anna_common::context::db::ContextDb;
use anna_common::display::UI;
use anna_common::hardware_capability::{HardwareAssessment, LlmCapability};
use anna_common::llm::LlmConfig;
use anna_common::model_profiles::select_model_for_capability;
use anna_common::ollama_installer::OllamaInstaller;
use anyhow::Result;
use std::io::{self, Write};

/// Check if LLM setup wizard needs to run
pub async fn needs_llm_setup(db: &ContextDb) -> Result<bool> {
    let config = db.load_llm_config().await?;

    // Need setup if not configured or explicitly disabled
    Ok(config.mode == anna_common::llm::LlmMode::NotConfigured)
}

/// Run the first-run LLM setup wizard
pub async fn run_llm_setup_wizard(ui: &UI, db: &ContextDb) -> Result<()> {
    // Starting first-run LLM setup wizard

    // Step 1: Explain what this is about
    println!();
    ui.section_header("🧠", "Setting Up My Brain");
    ui.info("I use a language model to understand your questions and explain");
    ui.info("things about your system in natural language.");
    ui.info("Let me check your machine's capabilities...");
    println!();

    // Step 2: Assess hardware
    let hw = HardwareAssessment::assess();

    ui.section_header("💻", "Hardware Assessment");
    ui.info(&format!("System: {}", hw.summary()));
    ui.info(&format!("Capability: {}", hw.llm_capability.description()));
    println!();

    // Step 3: Present options
    ui.section_header("⚙️", "Configuration Options");

    let options = if hw.llm_capability.is_local_recommended() {
        vec![
            "Set up a local model automatically (recommended - privacy-first)",
            "Configure a remote API (OpenAI-compatible) instead",
            "Skip for now and use rule-based assistance only",
        ]
    } else {
        vec![
            "Configure a remote API (OpenAI-compatible)",
            "Try to set up a local model anyway (may be slow)",
            "Skip for now and use rule-based assistance only",
        ]
    };

    for (i, option) in options.iter().enumerate() {
        ui.info(&format!("{}. {}", i + 1, option));
    }
    println!();

    // Get user choice
    print!("Choose an option (1-{}): ", options.len());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let choice = input.trim();

    let choice_num = choice.parse::<usize>().unwrap_or(0);

    // Route to appropriate setup
    match (hw.llm_capability.is_local_recommended(), choice_num) {
        (true, 1) | (false, 2) => {
            // Local model setup
            setup_local_model(ui, db, &hw).await
        }
        (true, 2) | (false, 1) => {
            // Remote API setup
            setup_remote_api(ui, db).await
        }
        (_, 3) => {
            // Skip
            setup_skip(ui, db).await
        }
        _ => {
            ui.warning("Invalid choice. Skipping for now.");
            setup_skip(ui, db).await
        }
    }
}

/// Set up local LLM using Ollama
async fn setup_local_model(ui: &UI, db: &ContextDb, hw: &HardwareAssessment) -> Result<()> {
    println!();
    ui.section_header("🏠", "Local Model Setup");

    // Select appropriate model profile
    let profile = select_model_for_capability(hw.llm_capability)
        .ok_or_else(|| anyhow::anyhow!("No suitable model profile found"))?;

    ui.info("I will:");
    ui.bullet_list(&[
        "Install or enable Ollama if needed",
        &format!(
            "Download model: {} (~{:.1} GB)",
            profile.model_name, profile.size_gb
        ),
        "Start the service and test it",
    ]);

    print!("Proceed with setup? (y/n): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if !input.trim().eq_ignore_ascii_case("y") {
        ui.info("Setup cancelled.");
        return setup_skip(ui, db).await;
    }

    ui.info("Setting up local model... This may take a few minutes.");

    // Run Ollama installer
    let installer = OllamaInstaller::new();

    match installer.auto_setup() {
        Ok(_commands) => {
            // Ollama setup completed successfully

            // Create and save LLM config
            let config = LlmConfig::from_profile(&profile);
            db.save_llm_config(&config).await?;

            // Store initial capability tier for upgrade detection
            store_initial_capability(db, hw.llm_capability).await?;

            ui.success("✓ My local brain is ready!");
            ui.info("I can now understand questions much better while keeping");
            ui.info("your data completely private on this machine.");
            println!();

            Ok(())
        }
        Err(e) => {
            // Local model setup failed
            ui.error("Local model setup failed.");
            eprintln!("{}", e);

            // Offer alternatives
            ui.info("What would you like to do?");
            ui.bullet_list(&[
                "1. Try again later (I'll use built-in rules for now)",
                "2. Configure a remote API instead",
            ]);

            print!("Choose an option (1-2): ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            match input.trim() {
                "2" => {
                    // User wants to try remote API
                    return setup_remote_api(ui, db).await;
                }
                _ => {
                    // Default: save disabled config and continue
                    ui.info("Okay, I'll use my built-in rules and the Arch Wiki for now.");
                    ui.info("You can set up my brain later by asking:");
                    ui.info("  \"Anna, set up your brain\"");
                    println!();

                    let config = LlmConfig::disabled();
                    db.save_llm_config(&config).await?;

                    Ok(())
                }
            }
        }
    }
}

/// Set up remote LLM API
async fn setup_remote_api(ui: &UI, db: &ContextDb) -> Result<()> {
    println!();
    ui.section_header("☁️", "Remote API Setup");

    ui.warning("⚠️ Important Privacy Notice");
    ui.info("Using a remote API means:");
    ui.bullet_list(&[
        "Your system information may be sent to the provider",
        "You may be charged per request by your provider",
        "Your data leaves this machine",
    ]);

    print!("Do you still want to configure a remote API? (y/n): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if !input.trim().eq_ignore_ascii_case("y") {
        return setup_skip(ui, db).await;
    }

    ui.info("Please provide the following information:");

    // Get API key
    print!("API key environment variable name (e.g., OPENAI_API_KEY): ");
    io::stdout().flush()?;
    let mut api_key_env = String::new();
    io::stdin().read_line(&mut api_key_env)?;
    let api_key_env = api_key_env.trim().to_string();

    // Get base URL
    print!("Base URL (default: https://api.openai.com/v1): ");
    io::stdout().flush()?;
    let mut base_url = String::new();
    io::stdin().read_line(&mut base_url)?;
    let base_url = if base_url.trim().is_empty() {
        "https://api.openai.com/v1".to_string()
    } else {
        base_url.trim().to_string()
    };

    // Get model name
    print!("Model name (default: gpt-4o-mini): ");
    io::stdout().flush()?;
    let mut model = String::new();
    io::stdin().read_line(&mut model)?;
    let model = if model.trim().is_empty() {
        "gpt-4o-mini".to_string()
    } else {
        model.trim().to_string()
    };

    // Create and save config
    let config = LlmConfig::remote(base_url, model, api_key_env, 0.00015);
    db.save_llm_config(&config).await?;

    ui.success("✓ Remote API configured");
    ui.info("I will now use the remote API to answer questions.");
    println!();

    Ok(())
}

/// Skip LLM setup for now
async fn setup_skip(ui: &UI, db: &ContextDb) -> Result<()> {
    ui.info("Okay, I will use my built-in rules and the Arch Wiki only.");
    ui.info("My answers will be more limited, but I can still help with:");
    ui.bullet_list(&[
        "System status and diagnostics",
        "Common issues and fixes",
        "Arch Wiki based suggestions",
    ]);
    ui.info("You can ask me to set up my brain anytime by saying:");
    ui.info("  \"Anna, set up your brain\"");
    println!();

    // Save disabled config
    let config = LlmConfig::disabled();
    db.save_llm_config(&config).await?;

    Ok(())
}

/// Store initial capability tier for upgrade detection
async fn store_initial_capability(db: &ContextDb, capability: LlmCapability) -> Result<()> {
    // Serialize capability as string
    let tier_str = match capability {
        LlmCapability::High => "high",
        LlmCapability::Medium => "medium",
        LlmCapability::Low => "low",
    };

    // Store in database using generic preference storage
    db.save_preference("llm_initial_capability", tier_str)
        .await?;

    Ok(())
}

/// Get stored initial capability tier
pub async fn get_initial_capability(db: &ContextDb) -> Result<Option<LlmCapability>> {
    // Load from database
    let tier_str = db.load_preference("llm_initial_capability").await?;

    // Parse string to capability enum
    Ok(tier_str.and_then(|s| match s.as_str() {
        "high" => Some(LlmCapability::High),
        "medium" => Some(LlmCapability::Medium),
        "low" => Some(LlmCapability::Low),
        _ => None,
    }))
}
