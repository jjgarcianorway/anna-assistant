//! Parse wiki articles into sections and find relevant sections.

use serde::{Deserialize, Serialize};

/// A section of a wiki article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiSection {
    /// Section header (e.g., "HiDPI", "Configuration")
    pub header: String,
    /// Section content
    pub content: String,
    /// Nesting level (1 = top-level, 2 = subsection, etc.)
    pub level: u8,
}

/// Parse an article into sections by markdown headers
pub fn parse_sections(content: &str) -> Vec<WikiSection> {
    let mut sections = Vec::new();
    let mut current_header = String::new();
    let mut current_content = String::new();
    let mut current_level: u8 = 0;

    for line in content.lines() {
        // Check for markdown headers (##, ###, etc.)
        if line.starts_with('#') {
            // Save previous section if any
            if !current_header.is_empty() || !current_content.is_empty() {
                sections.push(WikiSection {
                    header: current_header.clone(),
                    content: current_content.trim().to_string(),
                    level: current_level,
                });
            }

            // Parse new header
            let level = line.chars().take_while(|&c| c == '#').count() as u8;
            let header = line.trim_start_matches('#').trim().to_string();

            current_header = header;
            current_level = level;
            current_content = String::new();
        } else {
            // Add line to current section content
            current_content.push_str(line);
            current_content.push('\n');
        }
    }

    // Don't forget the last section
    if !current_header.is_empty() || !current_content.is_empty() {
        sections.push(WikiSection {
            header: current_header,
            content: current_content.trim().to_string(),
            level: current_level,
        });
    }

    sections
}

/// Words to ignore when matching section headers (too generic)
const HEADER_STOP_WORDS: &[&str] = &[
    "change", "make", "set", "get", "the", "how", "can", "do", "to", "if", "or",
    "not", "everything", "something", "nothing", "anything", "tried", "works",
    "want", "need", "help", "please", "using", "use",
];

/// Related terms mapping for better semantic matching
fn get_related_terms(word: &str) -> Vec<&'static str> {
    match word {
        "scale" | "scaling" | "2x" | "hidpi" | "dpi" =>
            vec!["scale", "scaling", "hidpi", "dpi", "monitor", "display", "font", "resolution"],
        "resolution" | "1920" | "1080" | "4k" | "1440" =>
            vec!["resolution", "monitor", "display", "screen", "mode"],
        "small" | "tiny" | "big" | "large" | "size" =>
            vec!["font", "scale", "scaling", "hidpi", "size", "larger", "smaller"],
        "display" | "monitor" | "screen" =>
            vec!["display", "monitor", "screen", "resolution", "mode"],
        _ => vec![],
    }
}

/// Find the most relevant section for a query using keyword matching
pub fn find_relevant_sections<'a>(sections: &'a [WikiSection], query: &str, max_sections: usize) -> Vec<&'a WikiSection> {
    let query_lower = query.to_lowercase();

    // Extract significant words (skip stop words for header matching)
    let query_words: Vec<&str> = query_lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 2 && !HEADER_STOP_WORDS.contains(w))
        .collect();

    // Expand query with related terms
    let mut expanded_terms: Vec<&str> = query_words.clone();
    for word in &query_words {
        for related in get_related_terms(word) {
            if !expanded_terms.contains(&related) {
                expanded_terms.push(related);
            }
        }
    }

    let mut scored: Vec<(&WikiSection, i32)> = sections
        .iter()
        .map(|section| {
            let header_lower = section.header.to_lowercase();
            let content_lower = section.content.to_lowercase();
            let mut score = 0i32;

            // Check expanded terms against header (high value)
            for term in &expanded_terms {
                if header_lower.contains(term) {
                    score += 10;
                }
            }

            // Check original query words against content (lower value)
            for word in &query_words {
                if content_lower.contains(word) {
                    score += 1;
                }
            }

            // Bonus for sections with commands ($ or #)
            if section.content.contains("$ ") || section.content.contains("# ") {
                score += 2;
            }

            // Bonus for sections with code blocks
            if section.content.contains("```") {
                score += 2;
            }

            (section, score)
        })
        .filter(|(_, score)| *score > 0)
        .collect();

    // Sort by score descending
    scored.sort_by(|a, b| b.1.cmp(&a.1));

    // Return top sections
    scored.into_iter()
        .take(max_sections)
        .map(|(section, _)| section)
        .collect()
}

/// Format sections for LLM context (concise, focused)
pub fn format_sections_for_context(sections: &[&WikiSection], article_title: &str) -> String {
    if sections.is_empty() {
        return String::new();
    }

    let mut result = format!("--- From {} ---\n", article_title);

    for section in sections {
        if !section.header.is_empty() {
            result.push_str(&format!("\n## {}\n", section.header));
        }
        // Truncate very long sections
        let content = if section.content.len() > 1500 {
            format!("{}...", &section.content[..1500])
        } else {
            section.content.clone()
        };
        result.push_str(&content);
        result.push('\n');
    }

    result
}

/// Build a prompt to ask the LLM which section is most relevant
pub fn build_section_selection_prompt(sections: &[WikiSection], query: &str) -> String {
    let mut prompt = format!(
        r#"Given this user query: "{}"

Which of these wiki sections is MOST relevant? Reply with ONLY the section number (1, 2, 3...) or "0" if none are relevant.

Sections:
"#,
        query
    );

    for (i, section) in sections.iter().enumerate() {
        let preview = if section.content.len() > 200 {
            format!("{}...", &section.content[..200])
        } else {
            section.content.clone()
        };
        prompt.push_str(&format!("\n{}. {} - {}\n", i + 1, section.header, preview));
    }

    prompt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sections() {
        let content = r#"# Main Title
Some intro text.

## Section One
Content of section one.

### Subsection
Nested content.

## Section Two
Content of section two.
"#;
        let sections = parse_sections(content);
        assert!(sections.len() >= 3);
        assert!(sections.iter().any(|s| s.header == "Section One"));
        assert!(sections.iter().any(|s| s.header == "Section Two"));
    }
}
