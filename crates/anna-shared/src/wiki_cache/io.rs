//! File I/O operations for wiki cache.

use std::fs;
use std::path::PathBuf;

use super::entry::WikiCacheEntry;
use super::index::WikiCacheIndex;
use super::utils::{normalize_title, simple_hash};

/// Get path to cached wiki page
pub fn get_cache_path(title: &str) -> PathBuf {
    let normalized = normalize_title(title);
    WikiCacheIndex::cache_dir().join(format!("{}.txt", normalized))
}

/// Read cached wiki page content
pub fn read_cached(title: &str) -> Option<String> {
    let path = get_cache_path(title);
    fs::read_to_string(path).ok()
}

/// Write wiki page to cache
pub fn write_cached(title: &str, content: &str) -> Result<WikiCacheEntry, std::io::Error> {
    let cache_dir = WikiCacheIndex::cache_dir();
    fs::create_dir_all(&cache_dir)?;

    let path = get_cache_path(title);
    fs::write(&path, content)?;

    let hash = simple_hash(content);
    let entry = WikiCacheEntry::new(
        title,
        &format!("https://wiki.archlinux.org/title/{}", title.replace(' ', "_")),
        content.len(),
        &hash,
    );

    Ok(entry)
}

/// Delete cached wiki page
pub fn delete_cached(title: &str) -> bool {
    let path = get_cache_path(title);
    fs::remove_file(path).is_ok()
}
