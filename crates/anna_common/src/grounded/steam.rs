//! Steam Detection v7.32.0 - Local Steam Game Discovery
//!
//! Detects Steam games from local appmanifest files.
//! No remote API calls - all data from local filesystem.
//!
//! Sources:
//! - ~/.steam/steam/steamapps/libraryfolders.vdf
//! - ~/.local/share/Steam/steamapps/libraryfolders.vdf
//! - Per-library appmanifest_*.acf files

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// A detected Steam game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamGame {
    /// Steam App ID
    pub appid: u32,
    /// Game name from manifest
    pub name: String,
    /// Install directory name
    pub install_dir: String,
    /// Full install path
    pub install_path: PathBuf,
    /// Library folder containing this game
    pub library_path: PathBuf,
    /// Last update timestamp (Unix epoch)
    pub last_updated: Option<u64>,
    /// Install size in bytes
    pub size_on_disk: Option<u64>,
}

impl SteamGame {
    /// Format for display in category lists
    pub fn display_name(&self) -> String {
        format!("steam:{}", self.name)
    }
}

/// Steam library detection result
#[derive(Debug, Clone)]
pub struct SteamLibrary {
    /// Path to this library folder
    pub path: PathBuf,
    /// Games installed in this library
    pub games: Vec<SteamGame>,
}

/// Check if Steam is installed
pub fn is_steam_installed() -> bool {
    get_steam_root().is_some()
}

/// Get Steam root directory
pub fn get_steam_root() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;

    // Check common Steam locations
    let candidates = [
        format!("{}/.steam/steam", home),
        format!("{}/.local/share/Steam", home),
        format!("{}/.steam/debian-installation", home),
    ];

    for path in &candidates {
        let p = PathBuf::from(path);
        if p.join("steamapps").exists() {
            return Some(p);
        }
    }
    None
}

/// Detect all Steam libraries
pub fn detect_steam_libraries() -> Vec<SteamLibrary> {
    let mut libraries = Vec::new();

    let Some(steam_root) = get_steam_root() else {
        return libraries;
    };

    // Read libraryfolders.vdf to find all library locations
    let vdf_path = steam_root.join("steamapps/libraryfolders.vdf");
    let library_paths = if vdf_path.exists() {
        parse_library_folders(&vdf_path)
    } else {
        // Fallback: just use the main steamapps folder
        vec![steam_root.join("steamapps")]
    };

    for lib_path in library_paths {
        if let Some(library) = scan_library(&lib_path) {
            libraries.push(library);
        }
    }

    libraries
}

/// Detect all Steam games across all libraries
pub fn detect_steam_games() -> Vec<SteamGame> {
    detect_steam_libraries()
        .into_iter()
        .flat_map(|lib| lib.games)
        .collect()
}

/// Find a game by name or appid
pub fn find_steam_game(query: &str) -> Option<SteamGame> {
    let games = detect_steam_games();

    // Try exact appid match
    if let Ok(appid) = query.parse::<u32>() {
        if let Some(game) = games.iter().find(|g| g.appid == appid) {
            return Some(game.clone());
        }
    }

    // Try name match (case-insensitive)
    let query_lower = query.to_lowercase();
    games.into_iter()
        .find(|g| {
            g.name.to_lowercase() == query_lower ||
            g.name.to_lowercase().contains(&query_lower) ||
            g.display_name().to_lowercase().contains(&query_lower)
        })
}

/// Parse libraryfolders.vdf to get all library paths
fn parse_library_folders(vdf_path: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    let Ok(content) = std::fs::read_to_string(vdf_path) else {
        return paths;
    };

    // VDF is a simple key-value format
    // Look for "path" entries
    for line in content.lines() {
        let trimmed = line.trim();

        // Look for lines like: "path"		"/path/to/library"
        if trimmed.starts_with("\"path\"") {
            if let Some(path_str) = extract_vdf_string_value(trimmed) {
                let lib_path = PathBuf::from(path_str);
                let steamapps = lib_path.join("steamapps");
                if steamapps.exists() {
                    paths.push(steamapps);
                }
            }
        }
    }

    // Always include the main steamapps folder if not already included
    if let Some(root) = get_steam_root() {
        let main_steamapps = root.join("steamapps");
        if main_steamapps.exists() && !paths.contains(&main_steamapps) {
            paths.insert(0, main_steamapps);
        }
    }

    paths
}

/// Scan a library folder for games
fn scan_library(steamapps_path: &Path) -> Option<SteamLibrary> {
    if !steamapps_path.exists() {
        return None;
    }

    let mut games = Vec::new();

    // Scan for appmanifest_*.acf files
    if let Ok(entries) = std::fs::read_dir(steamapps_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("appmanifest_") && name.ends_with(".acf") {
                    if let Some(game) = parse_appmanifest(&path, steamapps_path) {
                        games.push(game);
                    }
                }
            }
        }
    }

    // Sort by name
    games.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Some(SteamLibrary {
        path: steamapps_path.to_path_buf(),
        games,
    })
}

/// Parse an appmanifest_*.acf file
fn parse_appmanifest(acf_path: &Path, steamapps_path: &Path) -> Option<SteamGame> {
    let content = std::fs::read_to_string(acf_path).ok()?;

    let mut appid: Option<u32> = None;
    let mut name: Option<String> = None;
    let mut install_dir: Option<String> = None;
    let mut last_updated: Option<u64> = None;
    let mut size_on_disk: Option<u64> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("\"appid\"") {
            if let Some(val) = extract_vdf_string_value(trimmed) {
                appid = val.parse().ok();
            }
        } else if trimmed.starts_with("\"name\"") {
            name = extract_vdf_string_value(trimmed);
        } else if trimmed.starts_with("\"installdir\"") {
            install_dir = extract_vdf_string_value(trimmed);
        } else if trimmed.starts_with("\"LastUpdated\"") {
            if let Some(val) = extract_vdf_string_value(trimmed) {
                last_updated = val.parse().ok();
            }
        } else if trimmed.starts_with("\"SizeOnDisk\"") {
            if let Some(val) = extract_vdf_string_value(trimmed) {
                size_on_disk = val.parse().ok();
            }
        }
    }

    let appid = appid?;
    let name = name?;
    let install_dir = install_dir?;

    let install_path = steamapps_path.join("common").join(&install_dir);

    Some(SteamGame {
        appid,
        name,
        install_dir,
        install_path,
        library_path: steamapps_path.to_path_buf(),
        last_updated,
        size_on_disk,
    })
}

/// Extract string value from VDF line like: "key"		"value"
fn extract_vdf_string_value(line: &str) -> Option<String> {
    // Split by tab or multiple spaces
    let parts: Vec<&str> = line.split(|c| c == '\t')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if parts.len() >= 2 {
        // Remove quotes from value
        let value = parts[1].trim_matches('"');
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }
    None
}

/// Get games count for display
pub fn get_steam_games_count() -> usize {
    detect_steam_games().len()
}

/// Format game size for display
pub fn format_game_size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1} GiB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.0} MiB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.0} KiB", bytes as f64 / 1024.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_vdf_value() {
        let line = r#""path"		"/home/user/.local/share/Steam""#;
        let val = extract_vdf_string_value(line);
        assert_eq!(val, Some("/home/user/.local/share/Steam".to_string()));
    }

    #[test]
    fn test_extract_appid() {
        let line = r#"	"appid"		"730""#;
        let val = extract_vdf_string_value(line);
        assert_eq!(val, Some("730".to_string()));
    }
}
