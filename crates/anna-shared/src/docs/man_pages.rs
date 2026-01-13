//! Man page retrieval and caching.
//!
//! v0.3.26: Provides local man page access with caching.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::LazyLock;
use std::sync::Mutex;

use super::{DocCitation, DocSource, man_cache_dir};

/// In-memory cache for man pages
static MAN_CACHE: LazyLock<Mutex<HashMap<String, CachedManPage>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Cached man page entry
#[derive(Clone)]
struct CachedManPage {
    content: String,
    section: u8,
    cached_at: std::time::Instant,
}

/// Get the path to a man page
pub fn locate_man_page(command: &str) -> Option<(PathBuf, u8)> {
    // Try man --where to find the page
    let output = Command::new("man")
        .args(["--where", command])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path_str.is_empty() {
        return None;
    }

    let path = PathBuf::from(&path_str);

    // Extract section from path (e.g., /usr/share/man/man1/ls.1.gz -> 1)
    let section = path.file_name()
        .and_then(|f| f.to_str())
        .and_then(|f| {
            // Parse section from filename like "ls.1.gz" or "systemctl.1"
            f.split('.')
                .rev()
                .find(|s| s.len() == 1 && s.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false))
                .and_then(|s| s.parse::<u8>().ok())
        })
        .unwrap_or(1);

    Some((path, section))
}

/// Get man page content (cached)
pub fn get_man_page(command: &str) -> Option<(String, u8)> {
    // Check in-memory cache first
    {
        let cache = MAN_CACHE.lock().ok()?;
        if let Some(cached) = cache.get(command) {
            // Cache valid for 1 hour
            if cached.cached_at.elapsed().as_secs() < 3600 {
                return Some((cached.content.clone(), cached.section));
            }
        }
    }

    // Check disk cache
    let cache_path = man_cache_path(command);
    if cache_path.exists() {
        if let Ok(content) = fs::read_to_string(&cache_path) {
            // Parse section from filename
            let section = cache_path.file_name()
                .and_then(|f| f.to_str())
                .and_then(|f| f.split('.').nth(1))
                .and_then(|s| s.parse::<u8>().ok())
                .unwrap_or(1);

            // Update in-memory cache
            if let Ok(mut cache) = MAN_CACHE.lock() {
                cache.insert(command.to_string(), CachedManPage {
                    content: content.clone(),
                    section,
                    cached_at: std::time::Instant::now(),
                });
            }

            return Some((content, section));
        }
    }

    // Fetch from system
    let (_, section) = locate_man_page(command)?;

    // Render man page to text
    let output = Command::new("man")
        .args(["-P", "cat", command])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let content = String::from_utf8_lossy(&output.stdout).to_string();

    // Cache to disk
    if let Err(e) = fs::create_dir_all(man_cache_dir()) {
        tracing::warn!("Failed to create man cache dir: {}", e);
    } else if let Err(e) = fs::write(&cache_path, &content) {
        tracing::warn!("Failed to cache man page: {}", e);
    }

    // Update in-memory cache
    if let Ok(mut cache) = MAN_CACHE.lock() {
        cache.insert(command.to_string(), CachedManPage {
            content: content.clone(),
            section,
            cached_at: std::time::Instant::now(),
        });
    }

    Some((content, section))
}

/// Get cache path for a man page
fn man_cache_path(command: &str) -> PathBuf {
    // Sanitize command name for filesystem
    let safe_name = command.replace(['/', '\\', ':'], "_");
    man_cache_dir().join(format!("{}.1.txt", safe_name))
}

/// Search man page for relevant section
pub fn search_man(command: &str, query: &str) -> Option<DocCitation> {
    let (content, section) = get_man_page(command)?;

    // Find relevant section based on query
    let query_lower = query.to_lowercase();
    let query_words: Vec<&str> = query_lower
        .split_whitespace()
        .filter(|w| w.len() >= 3)
        .collect();

    // Split into sections (man pages use headers like "NAME", "SYNOPSIS", etc.)
    let sections: Vec<&str> = content.split("\n\n").collect();

    let mut best_section = None;
    let mut best_score = 0;
    let mut best_line_start = 0;
    let mut current_line = 0;

    for section_text in &sections {
        let section_lower = section_text.to_lowercase();
        let mut score = 0;

        for word in &query_words {
            if section_lower.contains(word) {
                score += 1;
            }
        }

        // Bonus for sections with common documentation patterns
        if section_lower.contains("--") || section_lower.contains("-") {
            score += 1; // Likely options section
        }

        if score > best_score {
            best_score = score;
            best_section = Some(section_text.to_string());
            best_line_start = current_line;
        }

        current_line += section_text.lines().count();
    }

    if best_score == 0 {
        // Fallback to NAME section
        best_section = sections.first().map(|s| s.to_string());
    }

    let excerpt = best_section.as_ref()
        .map(|s| {
            s.lines().take(5).collect::<Vec<_>>().join("\n")
        })
        .unwrap_or_default();

    let line_end = best_line_start + excerpt.lines().count();

    Some(DocCitation {
        source: DocSource::ManPage { section },
        title: command.to_string(),
        section: None, // Could extract section header
        excerpt,
        local_path: man_cache_path(command).display().to_string(),
        line_range: Some((best_line_start, line_end)),
    })
}

/// Clear man page cache
pub fn clear_cache() {
    if let Ok(mut cache) = MAN_CACHE.lock() {
        cache.clear();
    }

    // Also clear disk cache
    if let Ok(entries) = fs::read_dir(man_cache_dir()) {
        for entry in entries.flatten() {
            let _ = fs::remove_file(entry.path());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_man_cache_path() {
        let path = man_cache_path("systemctl");
        assert!(path.to_string_lossy().contains("systemctl.1.txt"));
    }

    #[test]
    fn test_locate_man_page() {
        // This test requires man to be installed
        // On most systems, 'ls' should have a man page
        if let Some((path, section)) = locate_man_page("ls") {
            assert!(path.exists() || path.to_string_lossy().contains(".gz"));
            assert!(section >= 1 && section <= 8);
        }
    }
}
