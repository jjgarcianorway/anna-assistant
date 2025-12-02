//! Game Platforms v7.32.0 - Platform Game Detection
//!
//! Detects games from various platforms:
//! - Steam (via steam.rs)
//! - Heroic/Legendary (Epic Games)
//! - Lutris
//! - Bottles (Wine prefixes)
//!
//! All detection is local - no remote API calls.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub use super::steam::{SteamGame, detect_steam_games, is_steam_installed, find_steam_game};

/// Platform identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Platform {
    Steam,
    Heroic,
    Lutris,
    Bottles,
    Native,  // Native Linux games (pacman packages)
}

impl Platform {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Steam => "Steam",
            Self::Heroic => "Heroic/Epic",
            Self::Lutris => "Lutris",
            Self::Bottles => "Bottles",
            Self::Native => "Native",
        }
    }
}

/// A detected game from any platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformGame {
    /// Game name
    pub name: String,
    /// Platform
    pub platform: Platform,
    /// Install path if known
    pub install_path: Option<PathBuf>,
    /// Platform-specific ID
    pub platform_id: Option<String>,
    /// Evidence source
    pub evidence: String,
}

impl PlatformGame {
    /// Format for display
    pub fn display_name(&self) -> String {
        match self.platform {
            Platform::Steam => format!("steam:{}", self.name),
            Platform::Heroic => format!("heroic:{}", self.name),
            Platform::Lutris => format!("lutris:{}", self.name),
            Platform::Bottles => format!("bottles:{}", self.name),
            Platform::Native => self.name.clone(),
        }
    }
}

/// Detect games from Heroic/Legendary (Epic Games Store)
pub fn detect_heroic_games() -> Vec<PlatformGame> {
    let mut games = Vec::new();
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return games,
    };

    // Check Heroic config locations
    let config_paths = [
        format!("{}/.config/heroic/store_cache/gog_library.json", home),
        format!("{}/.config/heroic/store_cache/legendary_library.json", home),
        format!("{}/.config/legendary/installed.json", home),
    ];

    for path in &config_paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            // Try to parse as JSON and extract game names
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                extract_heroic_games(&json, &mut games, path);
            }
        }
    }

    games
}

fn extract_heroic_games(json: &serde_json::Value, games: &mut Vec<PlatformGame>, source: &str) {
    // Handle different JSON structures from Heroic/Legendary
    if let Some(library) = json.get("library").and_then(|l| l.as_array()) {
        for game in library {
            if let Some(title) = game.get("title").or(game.get("app_name")).and_then(|t| t.as_str()) {
                games.push(PlatformGame {
                    name: title.to_string(),
                    platform: Platform::Heroic,
                    install_path: game.get("install_path")
                        .and_then(|p| p.as_str())
                        .map(PathBuf::from),
                    platform_id: game.get("app_name")
                        .and_then(|n| n.as_str())
                        .map(|s| s.to_string()),
                    evidence: format!("heroic: {}", source),
                });
            }
        }
    }

    // Legendary installed.json format
    if json.is_object() && !json.get("library").is_some() {
        if let Some(obj) = json.as_object() {
            for (app_name, info) in obj {
                if let Some(title) = info.get("title").and_then(|t| t.as_str()) {
                    games.push(PlatformGame {
                        name: title.to_string(),
                        platform: Platform::Heroic,
                        install_path: info.get("install_path")
                            .and_then(|p| p.as_str())
                            .map(PathBuf::from),
                        platform_id: Some(app_name.clone()),
                        evidence: format!("legendary: {}", source),
                    });
                }
            }
        }
    }
}

/// Detect games from Lutris
pub fn detect_lutris_games() -> Vec<PlatformGame> {
    let mut games = Vec::new();
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return games,
    };

    // Check Lutris database
    let db_path = format!("{}/.local/share/lutris/pga.db", home);
    if std::path::Path::new(&db_path).exists() {
        // Parse SQLite database for games
        if let Ok(games_list) = parse_lutris_db(&db_path) {
            games.extend(games_list);
        }
    }

    // Also check lutris config files
    let config_dir = format!("{}/.config/lutris/games", home);
    if let Ok(entries) = std::fs::read_dir(&config_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "yml").unwrap_or(false) {
                if let Some(game) = parse_lutris_config(&path) {
                    // Avoid duplicates
                    if !games.iter().any(|g| g.name == game.name) {
                        games.push(game);
                    }
                }
            }
        }
    }

    games
}

fn parse_lutris_db(db_path: &str) -> Result<Vec<PlatformGame>, Box<dyn std::error::Error>> {
    use rusqlite::Connection;

    let conn = Connection::open_with_flags(
        db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
    )?;

    let mut stmt = conn.prepare("SELECT name, slug, directory FROM games WHERE installed = 1")?;
    let games = stmt.query_map([], |row| {
        Ok(PlatformGame {
            name: row.get(0)?,
            platform: Platform::Lutris,
            install_path: row.get::<_, Option<String>>(2)?.map(PathBuf::from),
            platform_id: row.get(1)?,
            evidence: format!("lutris: {}", db_path),
        })
    })?;

    Ok(games.filter_map(|g| g.ok()).collect())
}

fn parse_lutris_config(path: &std::path::Path) -> Option<PlatformGame> {
    let content = std::fs::read_to_string(path).ok()?;

    // Simple YAML parsing for game name
    let mut name = None;
    let mut slug = None;

    for line in content.lines() {
        if line.starts_with("name:") {
            name = line.split(':').nth(1).map(|s| s.trim().trim_matches('"').to_string());
        } else if line.starts_with("slug:") {
            slug = line.split(':').nth(1).map(|s| s.trim().to_string());
        }
    }

    name.map(|n| PlatformGame {
        name: n,
        platform: Platform::Lutris,
        install_path: None,
        platform_id: slug,
        evidence: format!("lutris: {}", path.display()),
    })
}

/// Detect apps from Bottles (Wine prefixes)
pub fn detect_bottles_games() -> Vec<PlatformGame> {
    let mut games = Vec::new();
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return games,
    };

    // Check Bottles config
    let bottles_dir = format!("{}/.local/share/bottles/bottles", home);
    if let Ok(entries) = std::fs::read_dir(&bottles_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Each directory is a bottle
                let bottle_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown");

                // Check for bottle.yml
                let config_path = path.join("bottle.yml");
                if config_path.exists() {
                    if let Some(bottle_games) = parse_bottles_config(&config_path, bottle_name) {
                        games.extend(bottle_games);
                    }
                }
            }
        }
    }

    games
}

fn parse_bottles_config(path: &std::path::Path, bottle_name: &str) -> Option<Vec<PlatformGame>> {
    let content = std::fs::read_to_string(path).ok()?;
    let mut games = Vec::new();

    // Parse installed programs from bottle config
    let mut in_programs = false;
    for line in content.lines() {
        if line.starts_with("Programs:") || line.starts_with("External_Programs:") {
            in_programs = true;
            continue;
        }

        if in_programs {
            if !line.starts_with("  ") && !line.starts_with("-") {
                in_programs = false;
                continue;
            }

            // Extract program name
            if line.trim().starts_with("name:") {
                if let Some(name) = line.split(':').nth(1) {
                    let name = name.trim().trim_matches('"');
                    if !name.is_empty() {
                        games.push(PlatformGame {
                            name: name.to_string(),
                            platform: Platform::Bottles,
                            install_path: Some(path.parent().unwrap().to_path_buf()),
                            platform_id: Some(bottle_name.to_string()),
                            evidence: format!("bottles: {} in {}", path.display(), bottle_name),
                        });
                    }
                }
            }
        }
    }

    if games.is_empty() {
        // If no specific programs, add the bottle itself as an entry
        Some(vec![PlatformGame {
            name: format!("Bottle: {}", bottle_name),
            platform: Platform::Bottles,
            install_path: Some(path.parent().unwrap().to_path_buf()),
            platform_id: Some(bottle_name.to_string()),
            evidence: format!("bottles: {}", path.display()),
        }])
    } else {
        Some(games)
    }
}

/// Detect all games from all platforms
pub fn detect_all_platform_games() -> Vec<PlatformGame> {
    let mut games = Vec::new();

    // Steam games
    for steam_game in detect_steam_games() {
        games.push(PlatformGame {
            name: steam_game.name.clone(),
            platform: Platform::Steam,
            install_path: Some(steam_game.install_path.clone()),
            platform_id: Some(steam_game.appid.to_string()),
            evidence: format!("steam: appid {} in {}", steam_game.appid, steam_game.library_path.display()),
        });
    }

    // Heroic/Legendary games
    games.extend(detect_heroic_games());

    // Lutris games
    games.extend(detect_lutris_games());

    // Bottles games
    games.extend(detect_bottles_games());

    // Sort by platform then name
    games.sort_by(|a, b| {
        match a.platform.label().cmp(b.platform.label()) {
            std::cmp::Ordering::Equal => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            other => other,
        }
    });

    games
}

/// Get summary of detected platforms
pub fn get_platforms_summary() -> Vec<(Platform, usize)> {
    let games = detect_all_platform_games();
    let mut counts: std::collections::HashMap<Platform, usize> = std::collections::HashMap::new();

    for game in &games {
        *counts.entry(game.platform.clone()).or_insert(0) += 1;
    }

    let mut summary: Vec<_> = counts.into_iter().collect();
    summary.sort_by_key(|(p, _)| p.label());
    summary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_labels() {
        assert_eq!(Platform::Steam.label(), "Steam");
        assert_eq!(Platform::Heroic.label(), "Heroic/Epic");
    }

    #[test]
    fn test_display_name() {
        let game = PlatformGame {
            name: "Counter-Strike 2".to_string(),
            platform: Platform::Steam,
            install_path: None,
            platform_id: Some("730".to_string()),
            evidence: "test".to_string(),
        };
        assert_eq!(game.display_name(), "steam:Counter-Strike 2");
    }
}
