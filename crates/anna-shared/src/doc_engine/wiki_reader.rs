//! Arch Wiki local reader (v0.0.429).
//!
//! Reads Arch Wiki pages from local cache/snapshot.
//! Supports both plain text and HTML formats.

use super::{DocSnippet, DocSourceKind, MAX_SNIPPET_SIZE};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Read Arch Wiki page from local cache
pub fn read_wiki_page(name: &str, cache_path: &Path) -> Result<Vec<DocSnippet>, WikiReadError> {
    let page_path = find_wiki_page(name, cache_path)?;
    let content =
        fs::read_to_string(&page_path).map_err(|e| WikiReadError::ReadFailed(e.to_string()))?;

    // Detect format and parse
    let is_html = page_path.extension().map(|e| e == "html").unwrap_or(false)
        || content.trim_start().starts_with('<');

    let clean_content = if is_html {
        clean_html(&content)
    } else {
        clean_plain_text(&content)
    };

    // Split into sections
    let sections = split_wiki_sections(&clean_content);

    let mut snippets = Vec::new();

    // Create main snippet
    let summary = extract_wiki_summary(&clean_content);
    snippets.push(DocSnippet::new(
        DocSourceKind::ArchWiki,
        name,
        None,
        &summary,
        &truncate(&clean_content, MAX_SNIPPET_SIZE),
    ));

    // Create snippets for key sections
    for (section_name, section_content) in sections {
        if section_content.len() > 50 && is_useful_section(&section_name) {
            let section_id = section_name.to_lowercase().replace(' ', "_");
            let mut snippet = DocSnippet::new(
                DocSourceKind::ArchWiki,
                name,
                Some(&section_id),
                &format!("{} - {}", name, section_name),
                &section_content,
            );
            snippet.truncate_content(MAX_SNIPPET_SIZE);
            snippets.push(snippet);
        }
    }

    if snippets.is_empty() {
        return Err(WikiReadError::NoContent(name.to_string()));
    }

    Ok(snippets)
}

/// Find wiki page file in cache
fn find_wiki_page(name: &str, cache_path: &Path) -> Result<PathBuf, WikiReadError> {
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

/// Clean HTML content to plain text
fn clean_html(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;

    let mut chars = html.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '<' {
            in_tag = true;

            // Check for script/style tags
            let tag_start: String = chars.clone().take(10).collect();
            let tag_lower = tag_start.to_lowercase();

            if tag_lower.starts_with("script") {
                in_script = true;
            } else if tag_lower.starts_with("/script") {
                in_script = false;
            } else if tag_lower.starts_with("style") {
                in_style = true;
            } else if tag_lower.starts_with("/style") {
                in_style = false;
            }

            // Convert certain tags to whitespace
            if tag_lower.starts_with("br")
                || tag_lower.starts_with("p")
                || tag_lower.starts_with("/p")
                || tag_lower.starts_with("div")
                || tag_lower.starts_with("/div")
                || tag_lower.starts_with("li")
                || tag_lower.starts_with("h")
            {
                result.push('\n');
            }
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag && !in_script && !in_style {
            // Decode common HTML entities
            if c == '&' {
                let entity: String = chars.clone().take(10).take_while(|&x| x != ';').collect();
                let decoded = match entity.as_str() {
                    "amp" => '&',
                    "lt" => '<',
                    "gt" => '>',
                    "quot" => '"',
                    "apos" => '\'',
                    "nbsp" => ' ',
                    "#39" => '\'',
                    _ => {
                        result.push(c);
                        continue;
                    }
                };
                result.push(decoded);
                // Skip past entity
                for _ in 0..=entity.len() {
                    chars.next();
                }
            } else {
                result.push(c);
            }
        }
    }

    clean_whitespace(&result)
}

/// Clean plain text content
fn clean_plain_text(text: &str) -> String {
    clean_whitespace(text)
}

/// Normalize whitespace
fn clean_whitespace(text: &str) -> String {
    let mut result = String::new();
    let mut prev_newlines = 0;

    for line in text.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            prev_newlines += 1;
            if prev_newlines <= 2 {
                result.push('\n');
            }
        } else {
            prev_newlines = 0;
            result.push_str(trimmed);
            result.push('\n');
        }
    }

    result.trim().to_string()
}

/// Split wiki content into sections
fn split_wiki_sections(content: &str) -> HashMap<String, String> {
    let mut sections = HashMap::new();
    let mut current_section = String::new();
    let mut current_content = String::new();

    for line in content.lines() {
        if is_wiki_heading(line) {
            // Save previous section
            if !current_section.is_empty() && !current_content.trim().is_empty() {
                sections.insert(current_section.clone(), current_content.trim().to_string());
            }
            current_section = extract_heading_text(line);
            current_content = String::new();
        } else {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }

    // Save last section
    if !current_section.is_empty() && !current_content.trim().is_empty() {
        sections.insert(current_section, current_content.trim().to_string());
    }

    sections
}

/// Check if line is a wiki heading
fn is_wiki_heading(line: &str) -> bool {
    let trimmed = line.trim();

    // Markdown style: ## Heading
    if trimmed.starts_with('#') && trimmed.len() > 2 {
        return true;
    }

    // MediaWiki style: == Heading ==
    if trimmed.starts_with("==") && trimmed.ends_with("==") {
        return true;
    }

    // Plain text style: all caps, short line
    if trimmed.len() < 50
        && trimmed.len() > 3
        && trimmed
            .chars()
            .filter(|c| c.is_alphabetic())
            .all(|c| c.is_uppercase())
    {
        return true;
    }

    false
}

/// Extract heading text
fn extract_heading_text(line: &str) -> String {
    let trimmed = line.trim();

    // Remove markdown #s
    let without_hashes = trimmed.trim_start_matches('#').trim();

    // Remove MediaWiki ==
    let without_equals = without_hashes
        .trim_start_matches('=')
        .trim_end_matches('=')
        .trim();

    without_equals.to_string()
}

/// Extract summary from wiki content
fn extract_wiki_summary(content: &str) -> String {
    // Get first paragraph (before first heading or empty line)
    let mut lines = Vec::new();

    for line in content.lines().take(20) {
        let trimmed = line.trim();

        if trimmed.is_empty() && !lines.is_empty() {
            break;
        }

        if is_wiki_heading(line) {
            break;
        }

        if !trimmed.is_empty() {
            lines.push(trimmed);
        }
    }

    let summary = lines.join(" ");

    if summary.len() > 200 {
        format!("{}...", &summary[..197])
    } else {
        summary
    }
}

/// Check if a section is useful to index separately
fn is_useful_section(name: &str) -> bool {
    let lower = name.to_lowercase();

    let useful = [
        "installation",
        "configuration",
        "usage",
        "troubleshooting",
        "tips and tricks",
        "options",
        "examples",
        "commands",
        "services",
        "timers",
        "see also",
    ];

    useful.iter().any(|u| lower.contains(u))
}

/// Truncate string to max length
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        let truncate_at = s[..max]
            .rfind(|c: char| c == '\n' || c == '.' || c == ' ')
            .unwrap_or(max);
        format!("{}...", &s[..truncate_at])
    }
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

/// Get essential wiki pages to sync
pub fn get_essential_wiki_pages() -> Vec<&'static str> {
    vec![
        // System
        "systemd",
        "Systemd/User",
        "Systemd/Timers",
        "Systemd-boot",
        // Packages
        "pacman",
        "Pacman/Tips_and_tricks",
        "Arch_User_Repository",
        "Makepkg",
        // Boot
        "Arch_boot_process",
        "GRUB",
        "Unified_kernel_image",
        "Mkinitcpio",
        // Filesystem
        "File_systems",
        "Fstab",
        "Btrfs",
        "Ext4",
        "Partitioning",
        // Storage
        "Solid_state_drive",
        "SSD/NVMe",
        "TRIM",
        "LVM",
        "RAID",
        // Network
        "Network_configuration",
        "Systemd-networkd",
        "NetworkManager",
        "Wireless",
        "Iwd",
        // Hardware
        "PCI_passthrough",
        "Kernel_module",
        "Power_management",
        "CPU_frequency_scaling",
        // Security
        "Security",
        "Users_and_groups",
        "Sudo",
        "SSH",
        "Firewall",
        // Desktop
        "Xorg",
        "Wayland",
        "Desktop_environment",
        "Display_manager",
        // Audio
        "PipeWire",
        "PulseAudio",
        "ALSA",
        // Troubleshooting
        "General_troubleshooting",
        "Boot_debugging",
    ]
}

/// Wiki reading errors
#[derive(Debug, Clone)]
pub enum WikiReadError {
    NotFound(String),
    ReadFailed(String),
    NoContent(String),
}

impl std::fmt::Display for WikiReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(n) => write!(f, "Wiki page not found: {}", n),
            Self::ReadFailed(e) => write!(f, "Failed to read wiki page: {}", e),
            Self::NoContent(n) => write!(f, "No content in wiki page: {}", n),
        }
    }
}

impl std::error::Error for WikiReadError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_wiki_heading() {
        assert!(is_wiki_heading("## Installation"));
        assert!(is_wiki_heading("=== Configuration ==="));
        assert!(is_wiki_heading("TROUBLESHOOTING"));

        assert!(!is_wiki_heading("This is normal text"));
        assert!(!is_wiki_heading(""));
    }

    #[test]
    fn test_extract_heading_text() {
        assert_eq!(extract_heading_text("## Installation"), "Installation");
        assert_eq!(extract_heading_text("=== Config ==="), "Config");
        assert_eq!(extract_heading_text("### Options"), "Options");
    }

    #[test]
    fn test_clean_html() {
        let html = "<p>Hello <b>world</b></p><script>bad()</script><p>End</p>";
        let clean = clean_html(html);
        assert!(clean.contains("Hello"));
        assert!(clean.contains("world"));
        assert!(clean.contains("End"));
        assert!(!clean.contains("bad"));
        assert!(!clean.contains("<"));
    }

    #[test]
    fn test_extract_wiki_summary() {
        let content = "This is the first line of the article.\nIt continues here.\n\n## First Section\nMore content.";
        let summary = extract_wiki_summary(content);
        assert!(summary.contains("first line"));
        assert!(!summary.contains("First Section"));
    }

    #[test]
    fn test_is_useful_section() {
        assert!(is_useful_section("Installation"));
        assert!(is_useful_section("Troubleshooting"));
        assert!(is_useful_section("Tips and tricks"));
        assert!(is_useful_section("See also"));

        assert!(!is_useful_section("Random section"));
        assert!(!is_useful_section("History"));
    }
}
