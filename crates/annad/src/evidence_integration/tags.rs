//! Tag extraction from translator tickets and questions (v0.0.410).

use anna_shared::rpc::TranslatorTicket;

/// Build tags from translator ticket and question
pub fn build_tags_from_ticket(ticket: &TranslatorTicket, question: &str) -> Vec<String> {
    let mut tags = Vec::new();

    // Add probes as tags (they often indicate topic)
    for probe in &ticket.needs_probes {
        // Extract key terms from probe commands
        if let Some(tag) = extract_tag_from_probe(probe) {
            tags.push(tag);
        }
    }

    // Extract keywords from question
    let keywords = extract_keywords_from_question(question);
    tags.extend(keywords);

    // Deduplicate
    tags.sort();
    tags.dedup();

    tags
}

/// Extract tag from probe command
pub fn extract_tag_from_probe(probe: &str) -> Option<String> {
    // Simple extraction - get the main command/topic
    let parts: Vec<&str> = probe.split_whitespace().collect();

    match parts.first()? {
        &"free" | &"memory" => Some("memory".to_string()),
        &"df" | &"lsblk" => Some("disk".to_string()),
        &"ip" | &"ss" | &"netstat" => Some("network".to_string()),
        &"systemctl" => {
            if probe.contains("status") {
                parts.get(2).map(|s| s.to_string())
            } else {
                Some("services".to_string())
            }
        }
        &"pacman" => Some("packages".to_string()),
        &"pactl" | &"wpctl" => Some("audio".to_string()),
        &"cat" | &"ls" => {
            // Try to extract config topic from path
            if probe.contains(".config") {
                Some("config".to_string())
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Extract keywords from question
pub fn extract_keywords_from_question(question: &str) -> Vec<String> {
    let stopwords = [
        "is", "my", "do", "i", "have", "what", "how", "much", "the", "a", "an",
    ];

    question
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 2 && !stopwords.contains(w))
        .map(|s| s.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_keywords() {
        let keywords = extract_keywords_from_question("do I have swap enabled?");
        assert!(keywords.contains(&"swap".to_string()));
        assert!(keywords.contains(&"enabled".to_string()));
    }

    #[test]
    fn test_extract_tag_from_probe() {
        assert_eq!(
            extract_tag_from_probe("free -h"),
            Some("memory".to_string())
        );
        assert_eq!(extract_tag_from_probe("df -h"), Some("disk".to_string()));
        assert_eq!(
            extract_tag_from_probe("pacman -Q vim"),
            Some("packages".to_string())
        );
    }
}
