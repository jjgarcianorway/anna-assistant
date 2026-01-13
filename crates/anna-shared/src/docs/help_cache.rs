//! Command --help output caching.
//!
//! v0.3.26: Caches --help output for ClaimGate citations.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::LazyLock;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use super::{DocCitation, DocSource, help_cache_dir};

/// In-memory cache for help output
static HELP_CACHE: LazyLock<Mutex<HashMap<String, CachedHelp>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Cached help entry
#[derive(Clone, Serialize, Deserialize)]
struct CachedHelp {
    content: String,
    version: Option<String>,
    captured_at: String,
}

/// Get --help output for a command (cached)
pub fn get_help_output(command: &str) -> Option<(String, Option<String>, String)> {
    let now = chrono::Utc::now().to_rfc3339();

    // Check in-memory cache first
    {
        let cache = HELP_CACHE.lock().ok()?;
        if let Some(cached) = cache.get(command) {
            return Some((cached.content.clone(), cached.version.clone(), cached.captured_at.clone()));
        }
    }

    // Check disk cache
    let cache_path = help_cache_path(command);
    if cache_path.exists() {
        if let Ok(content) = fs::read_to_string(&cache_path) {
            if let Ok(cached) = serde_json::from_str::<CachedHelp>(&content) {
                // Update in-memory cache
                if let Ok(mut cache) = HELP_CACHE.lock() {
                    cache.insert(command.to_string(), cached.clone());
                }
                return Some((cached.content, cached.version, cached.captured_at));
            }
        }
    }

    // Fetch from system
    let output = Command::new(command)
        .arg("--help")
        .output()
        .ok()?;

    // Some commands output help to stderr
    let content = if output.stdout.is_empty() {
        String::from_utf8_lossy(&output.stderr).to_string()
    } else {
        String::from_utf8_lossy(&output.stdout).to_string()
    };

    if content.is_empty() {
        return None;
    }

    // Try to get version
    let version = get_command_version(command);

    let cached = CachedHelp {
        content: content.clone(),
        version: version.clone(),
        captured_at: now.clone(),
    };

    // Cache to disk
    if let Err(e) = fs::create_dir_all(help_cache_dir()) {
        tracing::warn!("Failed to create help cache dir: {}", e);
    } else if let Ok(json) = serde_json::to_string(&cached) {
        if let Err(e) = fs::write(&cache_path, &json) {
            tracing::warn!("Failed to cache help output: {}", e);
        }
    }

    // Update in-memory cache
    if let Ok(mut cache) = HELP_CACHE.lock() {
        cache.insert(command.to_string(), cached);
    }

    Some((content, version, now))
}

/// Get cache path for help output
fn help_cache_path(command: &str) -> PathBuf {
    // Sanitize command name for filesystem
    let safe_name = command.replace(['/', '\\', ':'], "_");
    help_cache_dir().join(format!("{}.json", safe_name))
}

/// Try to get command version
fn get_command_version(command: &str) -> Option<String> {
    // Try --version first
    let output = Command::new(command)
        .arg("--version")
        .output()
        .ok()?;

    let version_output = if output.stdout.is_empty() {
        String::from_utf8_lossy(&output.stderr).to_string()
    } else {
        String::from_utf8_lossy(&output.stdout).to_string()
    };

    // Extract version number from first line
    version_output.lines().next().and_then(|line| {
        // Look for version patterns like "1.0", "v1.0", "1.0.0"
        let words: Vec<&str> = line.split_whitespace().collect();
        for word in words {
            let clean = word.trim_start_matches('v').trim_end_matches(',');
            if clean.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                if clean.contains('.') || clean.len() <= 10 {
                    return Some(clean.to_string());
                }
            }
        }
        None
    })
}

/// Search help output for relevant section
pub fn search_help(command: &str, query: &str) -> Option<DocCitation> {
    let (content, version, captured_at) = get_help_output(command)?;

    // Find relevant section based on query
    let query_lower = query.to_lowercase();
    let query_words: Vec<&str> = query_lower
        .split_whitespace()
        .filter(|w| w.len() >= 2)
        .collect();

    // Split into sections (help output often uses blank lines or headers)
    let lines: Vec<&str> = content.lines().collect();

    let mut best_start = 0;
    let mut best_score = 0;
    let mut best_excerpt = String::new();

    for (i, line) in lines.iter().enumerate() {
        let line_lower = line.to_lowercase();
        let mut score = 0;

        for word in &query_words {
            if line_lower.contains(word) {
                score += 1;
            }
        }

        // Bonus for option lines (start with - or contain --)
        if line.trim().starts_with('-') || line.contains("--") {
            score += 1;
        }

        if score > best_score {
            best_score = score;
            best_start = i;
            // Get context (this line + next 2)
            best_excerpt = lines[i..].iter().take(3).copied().collect::<Vec<_>>().join("\n");
        }
    }

    if best_score == 0 {
        // No match found
        return None;
    }

    let line_end = best_start + best_excerpt.lines().count();

    Some(DocCitation {
        source: DocSource::HelpOutput {
            version,
            captured_at,
        },
        title: command.to_string(),
        section: None,
        excerpt: best_excerpt,
        local_path: help_cache_path(command).display().to_string(),
        line_range: Some((best_start, line_end)),
    })
}

/// Capture help output for a command (call when Anna runs a command)
pub fn capture_help(command: &str) {
    // Just calling get_help_output will cache it
    let _ = get_help_output(command);
}

/// Clear help cache
pub fn clear_cache() {
    if let Ok(mut cache) = HELP_CACHE.lock() {
        cache.clear();
    }

    // Also clear disk cache
    if let Ok(entries) = fs::read_dir(help_cache_dir()) {
        for entry in entries.flatten() {
            let _ = fs::remove_file(entry.path());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_cache_path() {
        let path = help_cache_path("pacman");
        assert!(path.to_string_lossy().contains("pacman.json"));
    }

    #[test]
    fn test_get_command_version() {
        // Test with a common command
        // This test may fail if the command isn't available
        if let Some(version) = get_command_version("ls") {
            assert!(!version.is_empty());
        }
    }
}
