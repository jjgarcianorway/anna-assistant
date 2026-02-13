//! Wallpaper automation — detects DE/WM, sets/schedules wallpaper.
//!
//! Uses de_config.rs to detect DE/WM. Queries wiki for the correct
//! wallpaper tool per environment. Creates a systemd timer via automation_creator.rs.
//!
//! Wallpaper sources:
//! - Unsplash random: https://source.unsplash.com/random/1920x1080
//! - Local directory (user-specified)

use anyhow::{anyhow, Result};
use tracing::info;

/// Detect the current DE/WM and return the correct wallpaper command.
/// Returns (tool_name, set_command_template) where %WALLPAPER% is replaced with path.
pub async fn detect_wallpaper_tool(model: &str) -> Result<(String, String)> {
    let de_env = std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("DESKTOP_SESSION"))
        .unwrap_or_default()
        .to_lowercase();

    let wm_check = std::process::Command::new("pgrep")
        .args(["-x", "Hyprland"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    let detected_de = if wm_check || de_env.contains("hyprland") {
        "Hyprland"
    } else if de_env.contains("sway") {
        "Sway"
    } else if de_env.contains("gnome") {
        "GNOME"
    } else if de_env.contains("kde") || de_env.contains("plasma") {
        "KDE Plasma"
    } else if de_env.contains("xfce") {
        "XFCE"
    } else if de_env.contains("i3") {
        "i3"
    } else {
        "Unknown"
    };

    // Query wiki for wallpaper tools for this DE
    let wiki_topic = match detected_de {
        "Hyprland" => "Hyprland",
        "Sway" => "Sway",
        "GNOME" => "GNOME",
        "KDE Plasma" => "KDE",
        _ => "desktop wallpaper",
    };

    let wiki_content = anna_shared::wiki::search::keyword_search_text(wiki_topic, 800)
        .unwrap_or_default();

    let prompt = format!(
        "You are setting up wallpaper for {detected_de} on Arch Linux.\n\
        Arch Wiki on {wiki_topic}:\n{wiki_content}\n\n\
        What is the correct command to set a wallpaper image file at PATH on {detected_de}?\n\
        Output EXACTLY in this format (no other text):\n\
        TOOL: <tool name e.g. hyprpaper, swww, feh, gsettings>\n\
        CMD: <full command to set wallpaper, use PATH as placeholder for the image file path>\n\
        INSTALL: <pacman package name to install if tool not present, or 'none'>"
    );

    let response = crate::ollama::chat_with_timeout(model, &prompt, 30).await
        .map_err(|e| anyhow!("LLM error: {}", e))?;

    let tool = extract_field(&response, "TOOL:")
        .unwrap_or_else(|| "feh".into());
    let cmd = extract_field(&response, "CMD:")
        .unwrap_or_else(|| "feh --bg-scale PATH".into());
    let install_pkg = extract_field(&response, "INSTALL:")
        .unwrap_or_else(|| "feh".into());

    // Check if tool is installed, offer to install if not
    let tool_available = std::process::Command::new("which")
        .arg(&tool)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !tool_available && install_pkg != "none" {
        info!("Wallpaper tool '{}' not found, installing package '{}'", tool, install_pkg);
        let installed = std::process::Command::new("pkexec")
            .args(["pacman", "-S", "--noconfirm", &install_pkg])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if !installed {
            return Err(anyhow!("Could not install wallpaper tool '{}'", tool));
        }
    }

    Ok((tool, cmd))
}

fn extract_field(text: &str, key: &str) -> Option<String> {
    text.lines()
        .find(|l| l.trim_start().starts_with(key))
        .map(|l| l.trim_start_matches(key).trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Download a random wallpaper from Unsplash.
/// Returns local path to downloaded image.
pub fn download_random_wallpaper(dest_path: &str) -> Result<String> {
    use std::process::Command;

    let url = "https://source.unsplash.com/random/1920x1080";

    // Use curl to follow redirects and save
    let status = Command::new("curl")
        .args(["-L", "-s", "-o", dest_path, url])
        .status()
        .map_err(|e| anyhow!("curl not available: {}", e))?;

    if !status.success() {
        return Err(anyhow!("Failed to download wallpaper from Unsplash"));
    }

    // Verify it's an image (basic check)
    let output = Command::new("file").arg(dest_path).output().ok();
    if let Some(o) = output {
        let out_str = String::from_utf8_lossy(&o.stdout);
        if !out_str.contains("image") && !out_str.contains("JPEG") && !out_str.contains("PNG") {
            return Err(anyhow!("Downloaded file does not appear to be an image"));
        }
    }

    Ok(dest_path.to_string())
}

/// Set the wallpaper immediately using detected tool.
pub async fn set_wallpaper_now(model: &str, image_path: &str) -> Result<String> {
    let (tool, cmd_template) = detect_wallpaper_tool(model).await?;
    let cmd = cmd_template.replace("PATH", image_path);

    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return Err(anyhow!("Empty wallpaper command"));
    }

    let status = std::process::Command::new(parts[0])
        .args(&parts[1..])
        .status()
        .map_err(|e| anyhow!("Failed to run {}: {}", tool, e))?;

    if status.success() {
        Ok(format!("Wallpaper set using {} successfully.", tool))
    } else {
        Err(anyhow!("{} returned an error setting wallpaper", tool))
    }
}

/// Create a daily wallpaper automation:
/// 1. Set wallpaper immediately
/// 2. Create systemd timer for daily rotation
pub async fn setup_wallpaper_automation(model: &str) -> Result<String> {
    let real_user = crate::user_context::get_real_user()
        .unwrap_or_else(|_| "user".to_string());
    let wallpaper_dir = format!("/home/{}/.local/share/anna/wallpapers", real_user);
    let wallpaper_path = format!("{}/current.jpg", wallpaper_dir);

    // Create wallpaper directory
    std::fs::create_dir_all(&wallpaper_dir).ok();

    // Download and set immediately
    info!("Downloading initial wallpaper...");
    if let Ok(path) = download_random_wallpaper(&wallpaper_path) {
        set_wallpaper_now(model, &path).await.ok();
    }

    // Create automation for daily rotation
    let intent = format!(
        "download a random wallpaper from https://source.unsplash.com/random/1920x1080 \
        to {wallpaper_path} using curl -L -s, then set it as the desktop wallpaper for user {real_user}"
    );

    crate::automation_creator::create_and_register_automation(model, &intent).await
        .map(|msg| format!("Daily wallpaper automation created!\n\n{}", msg))
        .map_err(|e| anyhow!("Could not create wallpaper timer: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_field() {
        let text = "TOOL: hyprpaper\nCMD: hyprpaper --wallpaper PATH\nINSTALL: hyprpaper";
        assert_eq!(extract_field(text, "TOOL:"), Some("hyprpaper".into()));
        assert_eq!(extract_field(text, "CMD:"), Some("hyprpaper --wallpaper PATH".into()));
    }

    #[test]
    fn test_extract_field_missing() {
        assert_eq!(extract_field("nothing here", "TOOL:"), None);
    }
}
