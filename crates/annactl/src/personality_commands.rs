//! Personality Management Commands
//!
//! CLI commands for managing Anna's 16-trait personality system
//! Beta.87: Database-backed personality customization

use anna_common::context::db::{ContextDb, DbLocation};
use anna_common::personality::PersonalityConfig;
use anna_common::terminal_format as fmt;
use anyhow::Result;

/// Show all personality traits with current values
pub async fn handle_personality_show() -> Result<()> {
    // Load database
    let db_location = DbLocation::auto_detect();
    let db = ContextDb::open(db_location).await?;

    // Load personality from database
    let personality = PersonalityConfig::load_from_db(&db).await?;

    if !personality.active {
        println!("{}", fmt::warning("⚠ Personality system is DISABLED"));
        println!();
        return Ok(());
    }

    println!("{}", fmt::bold("Anna's Personality Traits"));
    println!("{}", "=".repeat(60));
    println!();

    for trait_item in &personality.traits {
        // Display trait name
        println!("{}", fmt::bold(&trait_item.name));

        // Display visual bar
        let bar_length = 30;
        let filled = (trait_item.value as f32 / 10.0 * bar_length as f32) as usize;
        let empty = bar_length - filled;

        print!("  [");
        print!("{}", "█".repeat(filled));
        print!("{}", "░".repeat(empty));
        print!("] {}/10", trait_item.value);
        println!();

        // Display meaning
        println!("  {}", fmt::dimmed(&trait_item.meaning));
        println!();
    }

    println!("{}", fmt::dimmed("Use 'annactl personality set <trait> <value>' to customize"));
    println!();

    Ok(())
}

/// Set a specific personality trait value
pub async fn handle_personality_set(trait_key: String, value: u8) -> Result<()> {
    // Validate value range
    if value > 10 {
        anyhow::bail!("Value must be between 0 and 10");
    }

    // Load database
    let db_location = DbLocation::auto_detect();
    let db = ContextDb::open(db_location).await?;

    // Load current personality
    let mut personality = PersonalityConfig::load_from_db(&db).await?;

    // Find and update the trait
    let trait_found = personality.traits.iter_mut().find(|t| t.key == trait_key);

    match trait_found {
        Some(trait_item) => {
            let old_value = trait_item.value;
            trait_item.value = value;

            // Save to database
            personality.save_to_db(&db).await?;

            println!(
                "{}",
                fmt::success(&format!(
                    "✓ Updated {}: {} → {}",
                    trait_item.name, old_value, value
                ))
            );
            println!();
            println!("{}", fmt::dimmed("Restart Anna for changes to take effect"));
            Ok(())
        }
        None => {
            anyhow::bail!("Trait '{}' not found. Use 'annactl personality show' to see available traits", trait_key);
        }
    }
}

/// Adjust a personality trait by a delta (+/-)
pub async fn handle_personality_adjust(trait_key: String, delta: i8) -> Result<()> {
    // Load database
    let db_location = DbLocation::auto_detect();
    let db = ContextDb::open(db_location).await?;

    // Load current personality
    let mut personality = PersonalityConfig::load_from_db(&db).await?;

    // Find and update the trait
    let trait_found = personality.traits.iter_mut().find(|t| t.key == trait_key);

    match trait_found {
        Some(trait_item) => {
            let old_value = trait_item.value;
            let new_value = (trait_item.value as i16 + delta as i16).clamp(0, 10) as u8;
            trait_item.value = new_value;

            // Save to database
            personality.save_to_db(&db).await?;

            let direction = if delta > 0 { "+" } else { "" };
            println!(
                "{}",
                fmt::success(&format!(
                    "✓ Adjusted {}: {} {}{} = {}",
                    trait_item.name, old_value, direction, delta, new_value
                ))
            );
            println!();
            println!("{}", fmt::dimmed("Restart Anna for changes to take effect"));
            Ok(())
        }
        None => {
            anyhow::bail!("Trait '{}' not found. Use 'annactl personality show' to see available traits", trait_key);
        }
    }
}

/// Reset all personality traits to defaults
pub async fn handle_personality_reset() -> Result<()> {
    // Load database
    let db_location = DbLocation::auto_detect();
    let db = ContextDb::open(db_location).await?;

    // Create default personality
    let default_personality = PersonalityConfig::default();

    // Save to database
    default_personality.save_to_db(&db).await?;

    println!("{}", fmt::success("✓ Reset all personality traits to defaults"));
    println!();
    println!("{}", fmt::dimmed("Restart Anna for changes to take effect"));

    Ok(())
}

/// Validate personality configuration for conflicts
pub async fn handle_personality_validate() -> Result<()> {
    // Load database
    let db_location = DbLocation::auto_detect();
    let db = ContextDb::open(db_location).await?;

    // Load personality
    let personality = PersonalityConfig::load_from_db(&db).await?;

    // Validate
    match personality.validate_interactions() {
        Ok(_) => {
            println!("{}", fmt::success("✓ Personality configuration is valid"));
            println!();
            println!("{}", fmt::dimmed("No conflicting trait combinations detected"));
            Ok(())
        }
        Err(e) => {
            println!("{}", fmt::error(&format!("✗ Validation failed: {}", e)));
            println!();
            println!("{}", fmt::dimmed("Fix conflicts with 'annactl personality set'"));
            Err(e)
        }
    }
}

/// Export personality configuration to TOML
pub async fn handle_personality_export(path: Option<String>) -> Result<()> {
    // Load database
    let db_location = DbLocation::auto_detect();
    let db = ContextDb::open(db_location).await?;

    // Load personality
    let personality = PersonalityConfig::load_from_db(&db).await?;

    // Determine export path
    let export_path = match path {
        Some(p) => std::path::PathBuf::from(p),
        None => {
            let mut default_path = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?;
            default_path.push("anna_personality_export.toml");
            default_path
        }
    };

    // Export to TOML
    let toml_content = toml::to_string_pretty(&personality)?;
    std::fs::write(&export_path, toml_content)?;

    println!(
        "{}",
        fmt::success(&format!("✓ Exported personality to: {}", export_path.display()))
    );
    println!();
    println!("{}", fmt::dimmed("You can edit this file and import it back"));

    Ok(())
}
