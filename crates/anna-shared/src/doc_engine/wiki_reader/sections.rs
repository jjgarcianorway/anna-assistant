//! Wiki section parsing and extraction (v0.0.429).

use std::collections::HashMap;

/// Split wiki content into sections
pub fn split_wiki_sections(content: &str) -> HashMap<String, String> {
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
pub fn is_wiki_heading(line: &str) -> bool {
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
pub fn extract_heading_text(line: &str) -> String {
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
pub fn extract_wiki_summary(content: &str) -> String {
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
pub fn is_useful_section(name: &str) -> bool {
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
