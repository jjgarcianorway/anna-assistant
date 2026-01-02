//! Arch Wiki fetching (v0.0.422).

use super::types::SourceFetchResult;

/// Fetch Arch Wiki page content
pub fn fetch_arch_wiki(topic: &str) -> Option<SourceFetchResult> {
    // Try local cache first
    if let Some(result) = fetch_wiki_from_cache(topic) {
        return Some(result.cached());
    }

    // Arch Wiki online fetch would go here, but for now we only support cache
    // This keeps Anna local-first
    None
}

/// Fetch from local wiki cache
fn fetch_wiki_from_cache(topic: &str) -> Option<SourceFetchResult> {
    let home_cache = dirs::home_dir()
        .map(|h| h.join(".anna/wiki-cache"))
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp/anna-wiki-cache"));

    let cache_paths = [
        std::path::PathBuf::from("/var/lib/anna/wiki-cache"),
        home_cache,
    ];

    // Normalize topic to filename
    let filename = topic.replace(' ', "_").replace('/', "_").to_lowercase();

    for cache_path in &cache_paths {
        // Try exact match
        let exact_path = cache_path.join(format!("{}.txt", filename));
        if exact_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&exact_path) {
                return Some(SourceFetchResult::new(
                    content,
                    exact_path.to_string_lossy().as_ref(),
                ));
            }
        }

        // Try partial match
        if let Ok(entries) = std::fs::read_dir(&cache_path) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if name.contains(&filename) && name.ends_with(".txt") {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        return Some(SourceFetchResult::new(
                            content,
                            entry.path().to_string_lossy().as_ref(),
                        ));
                    }
                }
            }
        }
    }

    None
}
