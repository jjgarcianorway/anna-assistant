//! HTML and text cleaning utilities (v0.0.429).

/// Clean HTML content to plain text
pub fn clean_html(html: &str) -> String {
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
pub fn clean_plain_text(text: &str) -> String {
    clean_whitespace(text)
}

/// Normalize whitespace
pub fn clean_whitespace(text: &str) -> String {
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

/// Truncate string to max length
pub fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        let truncate_at = s[..max]
            .rfind(|c: char| c == '\n' || c == '.' || c == ' ')
            .unwrap_or(max);
        format!("{}...", &s[..truncate_at])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
