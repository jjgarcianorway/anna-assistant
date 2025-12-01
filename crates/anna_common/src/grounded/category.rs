//! Dynamic Categorization v7.1.0 - Real source-based categories
//!
//! Sources:
//! 1. pacman -Si <pkg> -> Groups field
//! 2. .desktop files -> Categories field
//! 3. man page section number
//!
//! Rules:
//! - Every category must cite its source
//! - No hardcoded category lists
//! - If no source found, return "unknown"

use std::process::Command;
use std::path::Path;

/// A category with its source
#[derive(Debug, Clone)]
pub struct CategoryInfo {
    /// The category name
    pub category: String,
    /// Source of this categorization (e.g., "pacman -Si vim")
    pub source: String,
}

/// Get category from pacman package groups
pub fn category_from_pacman(package: &str) -> Option<CategoryInfo> {
    let output = Command::new("pacman")
        .args(["-Si", package])
        .output()
        .ok()?;

    if !output.status.success() {
        // Try -Qi for installed packages
        let local_output = Command::new("pacman")
            .args(["-Qi", package])
            .output()
            .ok()?;

        if !local_output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&local_output.stdout);
        return parse_pacman_groups(&stdout, package, "pacman -Qi");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_pacman_groups(&stdout, package, "pacman -Si")
}

fn parse_pacman_groups(output: &str, package: &str, cmd: &str) -> Option<CategoryInfo> {
    for line in output.lines() {
        if line.starts_with("Groups") {
            let groups = line.split(':').nth(1)?.trim();
            if groups != "None" && !groups.is_empty() {
                // Map common group names to user-friendly categories
                let category = map_group_to_category(groups);
                return Some(CategoryInfo {
                    category,
                    source: format!("{} {} -> Groups", cmd, package),
                });
            }
        }
    }
    None
}

/// Map pacman group names to user-friendly categories
fn map_group_to_category(groups: &str) -> String {
    let groups_lower = groups.to_lowercase();

    // Check for common group patterns
    if groups_lower.contains("base-devel") {
        return "Development".to_string();
    }
    if groups_lower.contains("xorg") {
        return "X11/Display".to_string();
    }
    if groups_lower.contains("kde") || groups_lower.contains("plasma") {
        return "KDE/Desktop".to_string();
    }
    if groups_lower.contains("gnome") {
        return "GNOME/Desktop".to_string();
    }
    if groups_lower.contains("xfce") {
        return "XFCE/Desktop".to_string();
    }

    // Return first group as-is if no mapping
    groups.split_whitespace().next().unwrap_or(groups).to_string()
}

/// Get category from .desktop file
pub fn category_from_desktop(name: &str) -> Option<CategoryInfo> {
    // Common desktop file locations
    let desktop_paths = [
        format!("/usr/share/applications/{}.desktop", name),
        format!("/usr/share/applications/{}.desktop", name.to_lowercase()),
        format!("/usr/local/share/applications/{}.desktop", name),
    ];

    for path in &desktop_paths {
        if Path::new(path).exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                for line in content.lines() {
                    if line.starts_with("Categories=") {
                        let categories = line.trim_start_matches("Categories=");
                        if let Some(primary) = parse_desktop_categories(categories) {
                            return Some(CategoryInfo {
                                category: primary,
                                source: format!("{} -> Categories", path),
                            });
                        }
                    }
                }
            }
        }
    }

    None
}

/// Parse .desktop Categories field and return primary category
fn parse_desktop_categories(categories: &str) -> Option<String> {
    // Categories are semicolon-separated
    // e.g., "Development;IDE;TextEditor;"
    let cats: Vec<&str> = categories.split(';').filter(|s| !s.is_empty()).collect();

    if cats.is_empty() {
        return None;
    }

    // Return first meaningful category (skip generic ones)
    for cat in &cats {
        let cat_lower = cat.to_lowercase();
        // Skip overly generic categories
        if cat_lower == "application" || cat_lower == "utility" || cat_lower == "gtk" || cat_lower == "qt" {
            continue;
        }
        return Some(cat.to_string());
    }

    // If all were generic, return first
    Some(cats[0].to_string())
}

/// Get category from man page section
pub fn category_from_man(name: &str) -> Option<CategoryInfo> {
    let output = Command::new("man")
        .args(["-w", name])  // -w returns path to man page
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Extract section number from path
    // e.g., /usr/share/man/man1/vim.1.gz -> section 1
    if let Some(section) = extract_man_section(&path) {
        let category = man_section_to_category(section);
        return Some(CategoryInfo {
            category,
            source: format!("man -w {} -> section {}", name, section),
        });
    }

    None
}

fn extract_man_section(path: &str) -> Option<u8> {
    // Look for /manN/ in path
    for segment in path.split('/') {
        if segment.starts_with("man") && segment.len() > 3 {
            if let Ok(n) = segment[3..].parse::<u8>() {
                return Some(n);
            }
        }
    }
    None
}

fn man_section_to_category(section: u8) -> String {
    match section {
        1 => "User Command".to_string(),
        2 => "System Call".to_string(),
        3 => "Library Function".to_string(),
        4 => "Device/Special File".to_string(),
        5 => "File Format".to_string(),
        6 => "Game".to_string(),
        7 => "Miscellaneous".to_string(),
        8 => "System Admin".to_string(),
        9 => "Kernel Routine".to_string(),
        _ => format!("Man Section {}", section),
    }
}

/// Get category for an object from all sources
/// Returns the most specific category found
pub fn get_category(name: &str) -> Option<CategoryInfo> {
    // Try .desktop first (most specific for GUI apps)
    if let Some(info) = category_from_desktop(name) {
        return Some(info);
    }

    // Try pacman groups
    if let Some(info) = category_from_pacman(name) {
        return Some(info);
    }

    // Fall back to man page section
    if let Some(info) = category_from_man(name) {
        return Some(info);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_group_to_category() {
        assert_eq!(map_group_to_category("base-devel"), "Development");
        assert_eq!(map_group_to_category("xorg"), "X11/Display");
        assert_eq!(map_group_to_category("kde-applications"), "KDE/Desktop");
    }

    #[test]
    fn test_parse_desktop_categories() {
        assert_eq!(
            parse_desktop_categories("Development;IDE;TextEditor;"),
            Some("Development".to_string())
        );
        // When all categories are generic, return first
        assert_eq!(
            parse_desktop_categories("Application;Utility;"),
            Some("Application".to_string())
        );
        // When there's a specific category, skip generic ones
        assert_eq!(
            parse_desktop_categories("Application;Network;Browser;"),
            Some("Network".to_string())
        );
    }

    #[test]
    fn test_man_section_to_category() {
        assert_eq!(man_section_to_category(1), "User Command");
        assert_eq!(man_section_to_category(8), "System Admin");
    }
}
