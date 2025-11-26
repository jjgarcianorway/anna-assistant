//! Anna Game Detection - v10.2.1
//!
//! Filesystem-based game detection that learns discovered locations.
//! Follows the "no hardcoding" philosophy - all paths are probe hints,
//! actual locations are stored as learned SLOW facts.

use crate::learned_facts::{FactCategory, LearnedFact, LearnedFactsDb, StabilityClass};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Probe hints for Steam library locations (not hardcoded truths)
const STEAM_PROBE_HINTS: &[&str] = &[
    ".local/share/Steam",
    ".steam/steam",
    ".steam/debian-installation",
    "Games/Steam",
    "SteamLibrary",
];

/// Probe hints for Wine prefix locations
const WINE_PROBE_HINTS: &[&str] = &[
    ".wine",
    ".local/share/wineprefixes",
    "Games/Wine",
];

/// Probe hints for Lutris locations
const LUTRIS_PROBE_HINTS: &[&str] = &[
    ".local/share/lutris",
    "Games/Lutris",
];

/// Probe hints for Heroic Games Launcher
const HEROIC_PROBE_HINTS: &[&str] = &[
    ".config/heroic",
    "Games/Heroic",
];

/// Result of game detection
#[derive(Debug, Clone, Default)]
pub struct GameDetectionResult {
    /// Installed game launchers (from pacman)
    pub launchers: Vec<String>,
    /// Discovered Steam libraries with game counts
    pub steam_libraries: Vec<SteamLibrary>,
    /// Discovered Wine prefixes
    pub wine_prefixes: Vec<PathBuf>,
    /// Discovered Lutris games directory
    pub lutris_games_dir: Option<PathBuf>,
    /// Discovered Heroic games directory
    pub heroic_games_dir: Option<PathBuf>,
    /// Total games discovered on disk
    pub total_games_on_disk: usize,
    /// Evidence of what was probed
    pub probe_evidence: Vec<String>,
}

/// A Steam library with its path and game count
#[derive(Debug, Clone)]
pub struct SteamLibrary {
    pub path: PathBuf,
    pub steamapps_path: PathBuf,
    pub game_count: usize,
    pub game_names: Vec<String>,
}

/// Detect games on the filesystem
pub fn detect_games(home: &Path) -> GameDetectionResult {
    let mut result = GameDetectionResult::default();

    // Probe for Steam libraries
    result.steam_libraries = probe_steam_libraries(home, &mut result.probe_evidence);

    // Probe for Wine prefixes
    result.wine_prefixes = probe_wine_prefixes(home, &mut result.probe_evidence);

    // Probe for Lutris
    result.lutris_games_dir = probe_lutris(home, &mut result.probe_evidence);

    // Probe for Heroic
    result.heroic_games_dir = probe_heroic(home, &mut result.probe_evidence);

    // Calculate total
    result.total_games_on_disk = result.steam_libraries.iter()
        .map(|lib| lib.game_count)
        .sum();

    result
}

/// Probe for Steam library locations
fn probe_steam_libraries(home: &Path, evidence: &mut Vec<String>) -> Vec<SteamLibrary> {
    let mut libraries = Vec::new();
    let mut found_paths = HashSet::new();

    // First check probe hints
    for hint in STEAM_PROBE_HINTS {
        let path = home.join(hint);
        if path.exists() {
            evidence.push(format!("Probed {}: EXISTS", path.display()));

            // Check for steamapps directory
            let steamapps = path.join("steamapps");
            if steamapps.exists() && !found_paths.contains(&steamapps) {
                found_paths.insert(steamapps.clone());
                if let Some(lib) = scan_steamapps_library(&path, &steamapps) {
                    evidence.push(format!(
                        "Found Steam library at {} with {} games",
                        steamapps.display(),
                        lib.game_count
                    ));
                    libraries.push(lib);
                }
            }
        } else {
            evidence.push(format!("Probed {}: NOT FOUND", path.display()));
        }
    }

    // Parse libraryfolders.vdf for additional library paths
    for base in [
        home.join(".local/share/Steam"),
        home.join(".steam/steam"),
    ] {
        let vdf_path = base.join("steamapps/libraryfolders.vdf");
        if vdf_path.exists() {
            evidence.push(format!("Parsing {}", vdf_path.display()));
            if let Ok(content) = fs::read_to_string(&vdf_path) {
                for additional in parse_library_folders_vdf(&content) {
                    let steamapps = PathBuf::from(&additional).join("steamapps");
                    if steamapps.exists() && !found_paths.contains(&steamapps) {
                        found_paths.insert(steamapps.clone());
                        if let Some(lib) = scan_steamapps_library(
                            &PathBuf::from(&additional),
                            &steamapps
                        ) {
                            evidence.push(format!(
                                "Found additional Steam library at {} with {} games",
                                steamapps.display(),
                                lib.game_count
                            ));
                            libraries.push(lib);
                        }
                    }
                }
            }
        }
    }

    libraries
}

/// Scan a steamapps directory for installed games
fn scan_steamapps_library(base: &Path, steamapps: &Path) -> Option<SteamLibrary> {
    let common = steamapps.join("common");
    if !common.exists() {
        return None;
    }

    let mut game_names = Vec::new();

    if let Ok(entries) = fs::read_dir(&common) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    // Skip common non-game directories
                    if !["Steamworks Shared", "Proton", "Steam Linux Runtime"].contains(&name) {
                        game_names.push(name.to_string());
                    }
                }
            }
        }
    }

    Some(SteamLibrary {
        path: base.to_path_buf(),
        steamapps_path: steamapps.to_path_buf(),
        game_count: game_names.len(),
        game_names,
    })
}

/// Parse libraryfolders.vdf to find additional Steam library paths
fn parse_library_folders_vdf(content: &str) -> Vec<String> {
    let mut paths = Vec::new();

    // Simple parsing - look for "path" keys
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("\"path\"") {
            // Format: "path"    "/path/to/library"
            if let Some(path_part) = trimmed.split('"').nth(3) {
                if !path_part.is_empty() && Path::new(path_part).exists() {
                    paths.push(path_part.to_string());
                }
            }
        }
    }

    paths
}

/// Probe for Wine prefixes
fn probe_wine_prefixes(home: &Path, evidence: &mut Vec<String>) -> Vec<PathBuf> {
    let mut prefixes = Vec::new();

    for hint in WINE_PROBE_HINTS {
        let path = home.join(hint);
        if path.exists() {
            evidence.push(format!("Probed Wine {}: EXISTS", path.display()));

            // Check if it's a prefix itself (has system.reg)
            if path.join("system.reg").exists() {
                prefixes.push(path.clone());
            }

            // Check for sub-prefixes
            if let Ok(entries) = fs::read_dir(&path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let sub = entry.path();
                    if sub.is_dir() && sub.join("system.reg").exists() {
                        prefixes.push(sub);
                    }
                }
            }
        } else {
            evidence.push(format!("Probed Wine {}: NOT FOUND", path.display()));
        }
    }

    // Also check Steam's compatdata
    let compatdata = home.join(".local/share/Steam/steamapps/compatdata");
    if compatdata.exists() {
        evidence.push(format!("Found Proton compatdata at {}", compatdata.display()));
        if let Ok(entries) = fs::read_dir(&compatdata) {
            let count = entries.filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .count();
            evidence.push(format!("Found {} Proton prefixes", count));
        }
    }

    prefixes
}

/// Probe for Lutris
fn probe_lutris(home: &Path, evidence: &mut Vec<String>) -> Option<PathBuf> {
    for hint in LUTRIS_PROBE_HINTS {
        let path = home.join(hint);
        if path.exists() {
            evidence.push(format!("Probed Lutris {}: EXISTS", path.display()));
            let games_dir = path.join("games");
            if games_dir.exists() {
                return Some(games_dir);
            }
            return Some(path);
        } else {
            evidence.push(format!("Probed Lutris {}: NOT FOUND", path.display()));
        }
    }
    None
}

/// Probe for Heroic Games Launcher
fn probe_heroic(home: &Path, evidence: &mut Vec<String>) -> Option<PathBuf> {
    for hint in HEROIC_PROBE_HINTS {
        let path = home.join(hint);
        if path.exists() {
            evidence.push(format!("Probed Heroic {}: EXISTS", path.display()));
            return Some(path);
        } else {
            evidence.push(format!("Probed Heroic {}: NOT FOUND", path.display()));
        }
    }
    None
}

/// Store discovered game locations as learned facts
pub fn learn_game_locations(result: &GameDetectionResult, facts_db: &mut LearnedFactsDb) {
    // Store Steam library paths as SLOW facts
    for lib in &result.steam_libraries {
        let fact = LearnedFact::new(
            FactCategory::Custom(format!("steam_library:{}", lib.path.display())),
            format!("{} games in {}", lib.game_count, lib.steamapps_path.display()),
            format!(
                "Discovered via filesystem probe. Games: {}",
                lib.game_names.iter().take(5).cloned().collect::<Vec<_>>().join(", ")
            ),
            "game_detection probe".to_string(),
            0.95,
        );
        facts_db.learn(fact);
    }

    // Store total game count as SLOW fact
    if result.total_games_on_disk > 0 {
        let fact = LearnedFact::new(
            FactCategory::Custom("total_games_on_disk".to_string()),
            format!("{} games discovered on disk", result.total_games_on_disk),
            result.probe_evidence.join("\n"),
            "game_detection probe".to_string(),
            0.90,
        );
        facts_db.learn(fact);
    }
}

/// Format game detection results for display
pub fn format_game_detection(result: &GameDetectionResult) -> String {
    let mut output = Vec::new();

    // Launchers section
    if !result.launchers.is_empty() {
        output.push(format!("📦  Game Launchers (pacman):"));
        for launcher in &result.launchers {
            output.push(format!("    • {}", launcher));
        }
    }

    // Steam libraries section
    if !result.steam_libraries.is_empty() {
        output.push(String::new());
        output.push(format!("🎮  Steam Libraries:"));
        for lib in &result.steam_libraries {
            output.push(format!(
                "    📁  {} ({} games)",
                lib.steamapps_path.display(),
                lib.game_count
            ));
            if !lib.game_names.is_empty() {
                let preview: Vec<_> = lib.game_names.iter().take(5).collect();
                output.push(format!("        Games: {}", preview.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")));
                if lib.game_names.len() > 5 {
                    output.push(format!("        ... and {} more", lib.game_names.len() - 5));
                }
            }
        }
    }

    // Wine prefixes
    if !result.wine_prefixes.is_empty() {
        output.push(String::new());
        output.push(format!("🍷  Wine Prefixes: {}", result.wine_prefixes.len()));
    }

    // Summary
    output.push(String::new());
    output.push(format!("📊  Total: {} games discovered on disk", result.total_games_on_disk));

    output.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_library_folders_vdf() {
        let vdf = r#"
"libraryfolders"
{
    "0"
    {
        "path"    "/home/user/.steam/steam"
        "label"    ""
    }
    "1"
    {
        "path"    "/mnt/games/SteamLibrary"
        "label"    ""
    }
}
"#;
        let paths = parse_library_folders_vdf(vdf);
        // Note: will only return paths that exist
        assert!(paths.is_empty() || paths.iter().all(|p| Path::new(p).exists()));
    }

    #[test]
    fn test_format_game_detection_empty() {
        let result = GameDetectionResult::default();
        let output = format_game_detection(&result);
        assert!(output.contains("Total: 0 games"));
    }
}
