//! Arch Wiki Provider - v0.0.443.
//!
//! Provides offline-first access to cached Arch Wiki pages.

/// Arch Wiki provider (offline-first).
pub struct ArchWikiProvider {
    /// Cache directory.
    cache_dir: String,
}

impl ArchWikiProvider {
    /// Default cache directory.
    pub const DEFAULT_CACHE_DIR: &'static str = "/var/lib/anna/sources/archwiki";

    /// Create new provider.
    pub fn new() -> Self {
        Self {
            cache_dir: Self::DEFAULT_CACHE_DIR.to_string(),
        }
    }

    /// Create with custom cache dir.
    pub fn with_cache_dir(dir: &str) -> Self {
        Self {
            cache_dir: dir.to_string(),
        }
    }

    /// Fetch wiki page (offline only).
    pub fn fetch(&self, page: &str) -> Result<String, String> {
        let page_file = format!("{}/{}.txt", self.cache_dir, Self::normalize_page_name(page));

        std::fs::read_to_string(&page_file)
            .map_err(|_| format!("Arch Wiki page '{}' not available offline", page))
    }

    /// Normalize page name for filesystem.
    fn normalize_page_name(page: &str) -> String {
        page.replace(' ', "_").replace('/', "_")
    }

    /// Extract section from wiki page.
    pub fn extract_section(content: &str, section: &str) -> Option<String> {
        let section_lower = section.to_lowercase();
        let mut in_section = false;
        let mut section_level = 0;
        let mut result = Vec::new();

        for line in content.lines() {
            // Check for heading (== Heading ==)
            if line.starts_with('=') && line.ends_with('=') {
                let level = line.chars().take_while(|&c| c == '=').count();
                let heading = line.trim_matches('=').trim().to_lowercase();

                if heading.contains(&section_lower) {
                    in_section = true;
                    section_level = level;
                    result.push(line.to_string());
                } else if in_section && level <= section_level {
                    // New section at same or higher level, stop
                    break;
                }
            } else if in_section {
                result.push(line.to_string());
            }
        }

        if result.is_empty() {
            None
        } else {
            Some(result.join("\n"))
        }
    }

    /// Check if page is cached.
    pub fn is_cached(&self, page: &str) -> bool {
        let page_file = format!("{}/{}.txt", self.cache_dir, Self::normalize_page_name(page));
        std::path::Path::new(&page_file).exists()
    }
}

impl Default for ArchWikiProvider {
    fn default() -> Self {
        Self::new()
    }
}
