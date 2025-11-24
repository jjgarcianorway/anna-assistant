//! Configuration Command (6.18.0)
//!
//! annactl config show - Display current configuration
//! annactl config set KEY VALUE - Update configuration

use anna_common::anna_config::{AnnaConfig, ColorMode, EmojiMode};
use anyhow::Result;

/// Execute `annactl config show`
pub fn execute_config_show() -> Result<()> {
    let config = AnnaConfig::load()?;

    println!("Anna Configuration");
    println!("==================");
    println!();

    // Output section
    println!("[output]");
    println!("  emojis = {:?}", format_emoji_mode(config.output.emojis));
    println!("  color  = {:?}", format_color_mode(config.output.color));
    println!();

    // Show file location
    if let Ok(path) = AnnaConfig::user_config_path() {
        if path.exists() {
            println!("Config file: {}", path.display());
        } else {
            println!("Config file: {} (using defaults, file not created yet)", path.display());
        }
    }

    Ok(())
}

/// Execute `annactl config set KEY VALUE`
pub fn execute_config_set(key: &str, value: &str) -> Result<()> {
    // Load existing config (or defaults)
    let mut config = AnnaConfig::load().unwrap_or_default();

    // Parse key and update config
    match key.to_lowercase().as_str() {
        "output.emojis" | "emojis" => {
            config.set_emoji_mode(value)?;
            println!("✓ Set output.emojis = {:?}", format_emoji_mode(config.output.emojis));
        }
        "output.color" | "color" => {
            config.set_color_mode(value)?;
            println!("✓ Set output.color = {:?}", format_color_mode(config.output.color));
        }
        _ => {
            anyhow::bail!(
                "Unknown configuration key: '{}'\n\nValid keys:\n  output.emojis\n  output.color",
                key
            );
        }
    }

    // Save config
    config.save()?;

    if let Ok(path) = AnnaConfig::user_config_path() {
        println!();
        println!("Configuration saved to: {}", path.display());
    }

    Ok(())
}

/// Format emoji mode for display
fn format_emoji_mode(mode: EmojiMode) -> &'static str {
    match mode {
        EmojiMode::Auto => "auto",
        EmojiMode::Enabled => "on",
        EmojiMode::Disabled => "off",
    }
}

/// Format color mode for display
fn format_color_mode(mode: ColorMode) -> &'static str {
    match mode {
        ColorMode::Auto => "auto",
        ColorMode::Basic => "basic",
        ColorMode::None => "none",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_emoji_mode() {
        assert_eq!(format_emoji_mode(EmojiMode::Auto), "auto");
        assert_eq!(format_emoji_mode(EmojiMode::Enabled), "on");
        assert_eq!(format_emoji_mode(EmojiMode::Disabled), "off");
    }

    #[test]
    fn test_format_color_mode() {
        assert_eq!(format_color_mode(ColorMode::Auto), "auto");
        assert_eq!(format_color_mode(ColorMode::Basic), "basic");
        assert_eq!(format_color_mode(ColorMode::None), "none");
    }
}
