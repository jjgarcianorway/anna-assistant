//! File operations for wiki cache (v0.0.429).

use super::types::WikiReadError;
use std::fs;
use std::path::{Path, PathBuf};

/// Find wiki page file in cache
pub fn find_wiki_page(name: &str, cache_path: &Path) -> Result<PathBuf, WikiReadError> {
    // Normalize name for file search
    let name_lower = name.to_lowercase().replace(' ', "_");
    let name_variants = vec![
        name.to_string(),
        name_lower.clone(),
        name.replace(' ', "_"),
        name.replace('_', " "),
    ];

    // Extensions to try
    let extensions = ["txt", "md", "html", ""];

    for variant in &name_variants {
        for ext in &extensions {
            let filename = if ext.is_empty() {
                variant.clone()
            } else {
                format!("{}.{}", variant, ext)
            };

            let path = cache_path.join(&filename);
            if path.exists() {
                return Ok(path);
            }
        }
    }

    // Try subdirectories (some caches organize by first letter)
    if let Some(first_char) = name.chars().next() {
        let subdir = cache_path.join(first_char.to_uppercase().to_string());
        if subdir.exists() {
            for variant in &name_variants {
                for ext in &extensions {
                    let filename = if ext.is_empty() {
                        variant.clone()
                    } else {
                        format!("{}.{}", variant, ext)
                    };

                    let path = subdir.join(&filename);
                    if path.exists() {
                        return Ok(path);
                    }
                }
            }
        }
    }

    Err(WikiReadError::NotFound(name.to_string()))
}

/// List available wiki pages in cache
pub fn list_wiki_pages(cache_path: &Path) -> Vec<String> {
    let mut pages = Vec::new();

    if let Ok(entries) = fs::read_dir(cache_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(stem) = path.file_stem() {
                    let name = stem.to_string_lossy().to_string();
                    // Skip hidden files and index files
                    if !name.starts_with('.') && !name.starts_with("index") {
                        pages.push(name);
                    }
                }
            }
        }
    }

    // Also check subdirectories
    if let Ok(entries) = fs::read_dir(cache_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(subpages) = list_subdir_pages(&path) {
                    pages.extend(subpages);
                }
            }
        }
    }

    pages.sort();
    pages.dedup();
    pages
}

/// List pages in subdirectory
fn list_subdir_pages(dir: &Path) -> Option<Vec<String>> {
    let mut pages = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(stem) = path.file_stem() {
                    let name = stem.to_string_lossy().to_string();
                    if !name.starts_with('.') {
                        pages.push(name);
                    }
                }
            }
        }
    }

    Some(pages)
}
