//! Model Setup Wizard
//!
//! Beta.53: First-run model selection and installation
//! Guides users to install the best model for their hardware

use crate::model_catalog::{self, ModelInfo};
use anna_common::display::UI;
use anna_common::types::SystemFacts;
use anyhow::Result;
use std::io::{self, Write};

/// Run the model setup wizard if needed
/// Returns true if user made a change, false if they kept current setup
pub async fn run_model_setup_wizard_if_needed(
    current_model: &str,
    facts: &SystemFacts,
) -> Result<bool> {
    let ui = UI::auto();

    // Check if current model is suboptimal
    let recommended = model_catalog::select_best_model(facts.total_memory_gb, None);

    // If current model is good enough, skip wizard
    if current_model == recommended.id {
        return Ok(false);
    }

    // If using llama3.2:3b on a capable machine, strongly recommend upgrade
    if current_model == "llama3.2:3b" && facts.total_memory_gb >= 8.0 {
        println!();
        ui.section_header("⚠️ ", "Model Recommendation");
        ui.warning("You're using llama3.2:3b, which is too small for effective system administration.");
        println!();

        ui.info("Your hardware can support better models:");
        print_hardware_summary(facts);
        println!();

        ui.info(&format!("Recommended: {} ({})", recommended.id, recommended.description));
        println!();

        // Ask user if they want to upgrade now
        print!("Would you like to install {} now? [Y/n]: ", recommended.id);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        if input.is_empty() || input == "y" || input == "yes" {
            // Install the model
            match model_catalog::install_model(&recommended.id) {
                Ok(_) => {
                    ui.success(&format!("✓ Installed {}", recommended.id));
                    ui.info("Please restart annactl to use the new model.");
                    println!();
                    return Ok(true);
                }
                Err(e) => {
                    ui.error(&format!("Failed to install model: {}", e));
                    ui.info("Continuing with current model.");
                    println!();
                    return Ok(false);
                }
            }
        }
    }

    Ok(false)
}

/// Show interactive model selection menu
pub fn show_model_selection_menu(facts: &SystemFacts) -> Result<Option<String>> {
    let ui = UI::auto();

    ui.section_header("🤖", "Model Selection");
    println!();

    print_hardware_summary(facts);
    println!();

    let recommended = model_catalog::select_best_model(facts.total_memory_gb, None);
    let alternatives = model_catalog::get_alternatives(&recommended, facts.total_memory_gb);

    ui.info("Available Models:");
    println!();

    println!("  [1] {} (RECOMMENDED)", recommended.id);
    println!("      • {}", recommended.description);
    println!("      • Size: {:.1} GB download", recommended.disk_size_gb);
    println!("      • RAM needed: ~{:.0} GB during operation", recommended.ram_requirement_gb);
    println!("      • Quality: {}/100", recommended.quality_score);
    println!();

    for (i, model) in alternatives.iter().enumerate() {
        println!("  [{}] {}", i + 2, model.id);
        println!("      • {}", model.description);
        println!("      • Size: {:.1} GB | RAM: {:.0} GB | Quality: {}/100",
                 model.disk_size_gb, model.ram_requirement_gb, model.quality_score);
        println!();
    }

    println!("  [{}] Keep current model", alternatives.len() + 2);
    println!("  [{}] Cancel", alternatives.len() + 3);
    println!();

    print!("Choice [1-{}]: ", alternatives.len() + 3);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let choice: usize = input.trim().parse().unwrap_or(0);

    if choice == 0 {
        return Ok(None);
    }

    if choice == 1 {
        return Ok(Some(recommended.id.clone()));
    }

    if choice >= 2 && choice <= alternatives.len() + 1 {
        let model = &alternatives[choice - 2];
        return Ok(Some(model.id.clone()));
    }

    // Keep current or cancel
    Ok(None)
}

/// Print hardware summary
fn print_hardware_summary(facts: &SystemFacts) {
    let ui = UI::auto();

    ui.info("Detected Hardware:");
    println!("  • CPU: {} ({} cores)", facts.cpu_model, facts.cpu_cores);
    println!("  • RAM: {:.0} GB", facts.total_memory_gb);

    if let Some(ref gpu) = facts.gpu_model {
        if let Some(vram) = facts.gpu_vram_mb {
            println!("  • GPU: {} ({} MB VRAM)", gpu, vram);
        } else {
            println!("  • GPU: {}", gpu);
        }
    } else {
        println!("  • GPU: Integrated (CPU graphics)");
    }
}

/// Quick check if model setup is needed (for startup)
pub fn should_show_model_wizard(current_model: &str, host_ram_gb: f64) -> bool {
    // Show wizard if using 3b model on a system with 8+ GB RAM
    current_model == "llama3.2:3b" && host_ram_gb >= 8.0
}
