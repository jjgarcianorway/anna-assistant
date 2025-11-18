//! Model Setup Wizard
//!
//! Beta.53: First-run model selection and installation
//! Beta.68: Enhanced with performance tiers and benchmark integration
//! Guides users to install the best model for their hardware

use crate::model_catalog::{self};
use anna_common::display::UI;
use anna_common::model_profiles::{
    get_profile_by_id, get_recommended_with_fallbacks, ModelProfile, QualityTier,
};
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

    // Get recommended model and fallbacks using new system
    let (recommended_opt, fallbacks) = get_recommended_with_fallbacks(
        facts.total_memory_gb,
        facts.cpu_cores,
    );

    let recommended = match recommended_opt {
        Some(profile) => profile,
        None => {
            ui.warning("No suitable models found for your hardware.");
            return Ok(false);
        }
    };

    // Check if current model matches recommended
    if current_model.contains(&recommended.model_name) {
        return Ok(false);
    }

    // Check for suboptimal model usage
    let should_upgrade = check_if_upgrade_recommended(current_model, &recommended, facts);

    if should_upgrade {
        println!();
        ui.section_header("⚠️ ", "Model Recommendation");
        ui.warning(&format!(
            "Your current model ({}) may not be optimal for your hardware.",
            current_model
        ));
        println!();

        ui.info("Your hardware capabilities:");
        print_hardware_summary(facts);
        println!();

        print_model_recommendation(&recommended, &ui);
        println!();

        // Ask user if they want to upgrade now
        print!("Would you like to install {} now? [Y/n]: ", recommended.model_name);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        if input.is_empty() || input == "y" || input == "yes" {
            // Install the model
            match model_catalog::install_model(&recommended.model_name) {
                Ok(_) => {
                    ui.success(&format!("Installed {}", recommended.model_name));
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

/// Show interactive model selection menu with performance expectations
pub fn show_model_selection_menu(facts: &SystemFacts) -> Result<Option<String>> {
    let ui = UI::auto();

    ui.section_header("🤖", "Model Selection");
    println!();

    print_hardware_summary(facts);
    println!();

    let (recommended_opt, fallbacks) = get_recommended_with_fallbacks(
        facts.total_memory_gb,
        facts.cpu_cores,
    );

    let recommended = match recommended_opt {
        Some(profile) => profile,
        None => {
            ui.warning("No suitable models found for your hardware.");
            return Ok(None);
        }
    };

    ui.info("Available Models:");
    println!();

    println!("  [1] {} (RECOMMENDED)", recommended.model_name);
    print_model_details(&recommended, &ui);
    println!();

    for (i, model) in fallbacks.iter().enumerate() {
        println!("  [{}] {}", i + 2, model.model_name);
        print_model_details(model, &ui);
        println!();
    }

    println!("  [{}] Keep current model", fallbacks.len() + 2);
    println!("  [{}] Cancel", fallbacks.len() + 3);
    println!();

    print!("Choice [1-{}]: ", fallbacks.len() + 3);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let choice: usize = input.trim().parse().unwrap_or(0);

    if choice == 0 {
        return Ok(None);
    }

    if choice == 1 {
        return Ok(Some(recommended.model_name.clone()));
    }

    if choice >= 2 && choice <= fallbacks.len() + 1 {
        let model = &fallbacks[choice - 2];
        return Ok(Some(model.model_name.clone()));
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
    // Show wizard if using 3b or smaller model on a system with 8+ GB RAM
    (current_model.contains("1b") || current_model.contains("3b")) && host_ram_gb >= 8.0
}

/// Check if upgrade is recommended for current model
fn check_if_upgrade_recommended(
    current_model: &str,
    recommended: &ModelProfile,
    facts: &SystemFacts,
) -> bool {
    // Recommend upgrade if:
    // 1. Using tiny model (1b/1.5b) on 8GB+ system
    if (current_model.contains("1b") || current_model.contains("1.5b")) && facts.total_memory_gb >= 8.0 {
        return true;
    }

    // 2. Using small model (3b) on 16GB+ system
    if current_model.contains("3b") && facts.total_memory_gb >= 16.0 {
        return true;
    }

    // 3. Current model is significantly lower tier than recommended
    if recommended.quality_tier >= QualityTier::Medium
        && (current_model.contains("1b") || current_model.contains("3b"))
    {
        return true;
    }

    false
}

/// Print detailed model recommendation with performance expectations
fn print_model_recommendation(profile: &ModelProfile, ui: &UI) {
    ui.info(&format!("Recommended: {} ({})", profile.model_name, profile.description));
    println!("  • Quality Tier: {:?}", profile.quality_tier);
    println!("  • Size: {:.1} GB download", profile.size_gb);
    println!("  • RAM required: ≥{} GB", profile.min_ram_gb);
    println!("  • CPU cores: ≥{}", profile.recommended_cores);
    println!();

    ui.info("Performance Expectations:");
    println!("  • Speed: ≥{:.0} tokens/sec", profile.quality_tier.min_tokens_per_second());
    println!("  • Quality: ≥{:.0}% accuracy", profile.quality_tier.min_quality_score() * 100.0);
    println!("  • {}", profile.quality_tier.description());
}

/// Print model details with tier information
fn print_model_details(profile: &ModelProfile, _ui: &UI) {
    println!("      • {}", profile.description);
    println!("      • Tier: {:?} | Size: {:.1} GB | RAM: ≥{} GB",
             profile.quality_tier, profile.size_gb, profile.min_ram_gb);
    println!("      • Expected: ≥{:.0} tok/s, ≥{:.0}% quality",
             profile.quality_tier.min_tokens_per_second(),
             profile.quality_tier.min_quality_score() * 100.0);
}
