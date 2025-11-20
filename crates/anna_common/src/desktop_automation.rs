//! Desktop Automation Helpers
//!
//! Safe helper functions for desktop environment automation tasks like
//! changing wallpapers, updating configs, and reloading desktop environments.
//!
//! All operations create backups before making changes.

use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config_file::DesktopConfig;
use crate::desktop::{DesktopEnvironment, DesktopInfo};
use crate::file_backup::FileBackup;

/// Wallpaper change result
#[derive(Debug, Clone)]
pub struct WallpaperChangeResult {
    /// Previous wallpaper path (if any)
    pub previous_wallpaper: Option<PathBuf>,
    /// New wallpaper path
    pub new_wallpaper: PathBuf,
    /// Backup info for rollback
    pub backup: Option<FileBackup>,
    /// Commands executed
    pub commands_executed: Vec<String>,
}

/// List available wallpapers in a directory
pub fn list_wallpapers<P: AsRef<Path>>(directory: P) -> Result<Vec<PathBuf>> {
    let dir = directory.as_ref();

    if !dir.exists() {
        bail!("Directory does not exist: {}", dir.display());
    }

    if !dir.is_dir() {
        bail!("Path is not a directory: {}", dir.display());
    }

    let mut wallpapers = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            // Check if it's an image file
            if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                if matches!(
                    ext.as_str(),
                    "jpg" | "jpeg" | "png" | "webp" | "gif" | "bmp"
                ) {
                    wallpapers.push(path);
                }
            }
        }
    }

    Ok(wallpapers)
}

/// Pick a random wallpaper from a directory
pub fn pick_random_wallpaper<P: AsRef<Path>>(directory: P) -> Result<PathBuf> {
    let wallpapers = list_wallpapers(directory)?;

    if wallpapers.is_empty() {
        bail!("No wallpapers found in directory");
    }

    // Use a simple random selection based on current time
    let index = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize)
        % wallpapers.len();

    Ok(wallpapers[index].clone())
}

/// Change wallpaper for the current desktop environment
pub fn change_wallpaper<P: AsRef<Path>>(new_wallpaper: P) -> Result<WallpaperChangeResult> {
    let desktop_info = DesktopInfo::detect();
    let new_wallpaper = new_wallpaper.as_ref();

    if !new_wallpaper.exists() {
        bail!("Wallpaper file does not exist: {}", new_wallpaper.display());
    }

    let config = DesktopConfig::parse().context("Could not parse desktop configuration")?;

    match desktop_info.environment {
        DesktopEnvironment::Hyprland => change_wallpaper_hyprland(new_wallpaper, &config),
        DesktopEnvironment::I3 | DesktopEnvironment::Sway => {
            change_wallpaper_i3_sway(new_wallpaper, &config, &desktop_info.environment)
        }
        _ => bail!("Wallpaper changing not supported for this desktop environment"),
    }
}

/// Change wallpaper for Hyprland (using hyprpaper)
fn change_wallpaper_hyprland(
    new_wallpaper: &Path,
    config: &DesktopConfig,
) -> Result<WallpaperChangeResult> {
    let wallpaper_config = config
        .wallpaper
        .as_ref()
        .context("No wallpaper configuration found")?;

    let previous_wallpaper = wallpaper_config.paths.first().cloned();

    // Determine hyprpaper config path
    let config_file = if let Some(ref path) = wallpaper_config.config_file {
        path.clone()
    } else {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
        PathBuf::from(home).join(".config/hypr/hyprpaper.conf")
    };

    // Backup current config
    use crate::file_backup::FileOperation;
    let change_set_id = format!(
        "wallpaper-change-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );

    let backup = FileBackup::create_backup(&config_file, change_set_id, FileOperation::Modified)
        .context("Failed to backup hyprpaper config")?;

    // Read current config
    let content = fs::read_to_string(&config_file).context("Failed to read hyprpaper config")?;

    // Update wallpaper paths in config
    let mut new_content = String::new();
    for line in content.lines() {
        if line.trim().starts_with("preload") || line.trim().starts_with("wallpaper") {
            // Replace wallpaper path
            if line.contains('=') {
                let parts: Vec<&str> = line.splitn(2, '=').collect();
                if parts.len() == 2 {
                    let key = parts[0].trim();
                    let new_wallpaper_str = new_wallpaper.display().to_string();

                    if key.starts_with("wallpaper") {
                        // wallpaper = monitor, path
                        // Keep monitor, replace path
                        if let Some(comma_pos) = parts[1].find(',') {
                            let monitor = &parts[1][..comma_pos];
                            new_content.push_str(&format!(
                                "{} = {},{}\n",
                                key, monitor, new_wallpaper_str
                            ));
                        } else {
                            new_content.push_str(&format!("{} = {}\n", key, new_wallpaper_str));
                        }
                    } else {
                        // preload = path
                        new_content.push_str(&format!("{} = {}\n", key, new_wallpaper_str));
                    }
                    continue;
                }
            }
        }
        new_content.push_str(line);
        new_content.push('\n');
    }

    // Write updated config
    fs::write(&config_file, &new_content).context("Failed to write updated hyprpaper config")?;

    // Reload hyprpaper
    let mut commands_executed = Vec::new();

    // Kill existing hyprpaper
    if Command::new("pkill").arg("hyprpaper").status().is_ok() {
        commands_executed.push("pkill hyprpaper".to_string());
    }

    // Start hyprpaper
    Command::new("hyprpaper")
        .spawn()
        .context("Failed to start hyprpaper")?;
    commands_executed.push("hyprpaper".to_string());

    Ok(WallpaperChangeResult {
        previous_wallpaper,
        new_wallpaper: new_wallpaper.to_path_buf(),
        backup: Some(backup),
        commands_executed,
    })
}

/// Change wallpaper for i3/Sway (using feh or swaybg)
fn change_wallpaper_i3_sway(
    new_wallpaper: &Path,
    config: &DesktopConfig,
    _env: &DesktopEnvironment,
) -> Result<WallpaperChangeResult> {
    let wallpaper_config = config
        .wallpaper
        .as_ref()
        .context("No wallpaper configuration found")?;

    let previous_wallpaper = wallpaper_config.paths.first().cloned();
    let mut commands_executed = Vec::new();

    match wallpaper_config.setter.as_str() {
        "feh" => {
            // Use feh to set wallpaper
            Command::new("feh")
                .arg("--bg-scale")
                .arg(new_wallpaper)
                .status()
                .context("Failed to execute feh")?;
            commands_executed.push(format!("feh --bg-scale {}", new_wallpaper.display()));
        }
        "swaybg" => {
            // Kill existing swaybg
            if Command::new("pkill").arg("swaybg").status().is_ok() {
                commands_executed.push("pkill swaybg".to_string());
            }

            // Start swaybg with new wallpaper
            Command::new("swaybg")
                .arg("-i")
                .arg(new_wallpaper)
                .arg("-m")
                .arg("fill")
                .spawn()
                .context("Failed to start swaybg")?;
            commands_executed.push(format!("swaybg -i {} -m fill", new_wallpaper.display()));
        }
        "nitrogen" => {
            // Use nitrogen to set wallpaper
            Command::new("nitrogen")
                .arg("--set-zoom-fill")
                .arg(new_wallpaper)
                .status()
                .context("Failed to execute nitrogen")?;
            commands_executed.push(format!(
                "nitrogen --set-zoom-fill {}",
                new_wallpaper.display()
            ));
        }
        _ => bail!("Unsupported wallpaper setter: {}", wallpaper_config.setter),
    }

    Ok(WallpaperChangeResult {
        previous_wallpaper,
        new_wallpaper: new_wallpaper.to_path_buf(),
        backup: None, // feh/swaybg don't use config files
        commands_executed,
    })
}

/// Reload desktop environment
pub fn reload_desktop() -> Result<Vec<String>> {
    let desktop_info = DesktopInfo::detect();
    let mut commands_executed = Vec::new();

    match desktop_info.environment {
        DesktopEnvironment::Hyprland => {
            Command::new("hyprctl")
                .arg("reload")
                .status()
                .context("Failed to reload Hyprland")?;
            commands_executed.push("hyprctl reload".to_string());
        }
        DesktopEnvironment::I3 => {
            Command::new("i3-msg")
                .arg("reload")
                .status()
                .context("Failed to reload i3")?;
            commands_executed.push("i3-msg reload".to_string());
        }
        DesktopEnvironment::Sway => {
            Command::new("swaymsg")
                .arg("reload")
                .status()
                .context("Failed to reload Sway")?;
            commands_executed.push("swaymsg reload".to_string());
        }
        _ => bail!("Desktop reload not supported for this environment"),
    }

    Ok(commands_executed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_wallpapers() {
        // This test will vary by environment
        // Just ensure it doesn't crash
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let pictures = PathBuf::from(home).join("Pictures");

        if pictures.exists() {
            let result = list_wallpapers(&pictures);
            // Don't assert anything specific, just ensure it runs
            let _ = result;
        }
    }
}
