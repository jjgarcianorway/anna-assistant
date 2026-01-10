//! Basic validation checks (uncertainty, hallucination, generic responses).

use anna_shared::rpc::{ValidationIssueType, ValidationWarning};

use super::{RE_NAME, RE_NUMBER};

/// Check for uncertainty markers
/// v0.0.891: Smarter detection to avoid false positives
pub fn check_uncertainty(text: &str) -> Option<ValidationWarning> {
    let text_lower = text.to_lowercase();

    let check_text = if text_lower.len() > 150 {
        &text_lower[..150]
    } else {
        &text_lower
    };

    // High-confidence uncertainty markers
    let strong_uncertainty = [
        ("i'm not sure", "medium"),
        ("i don't know", "medium"),
        ("unable to determine", "medium"),
        ("cannot determine", "medium"),
        ("hard to say", "medium"),
        ("can't tell", "medium"),
    ];

    for (phrase, severity) in strong_uncertainty {
        if check_text.contains(phrase) {
            return Some(ValidationWarning {
                issue_type: ValidationIssueType::Uncertainty,
                message: format!("Response uses uncertain language: '{}'", phrase),
                severity: severity.to_string(),
            });
        }
    }

    // Weak uncertainty markers - only flag at the start
    let weak_uncertainty = [
        "might be", "could be", "perhaps", "possibly", "probably", "likely",
    ];

    let start_text = if text_lower.len() > 50 {
        &text_lower[..50]
    } else {
        &text_lower
    };

    for phrase in weak_uncertainty {
        if start_text.contains(phrase) {
            return Some(ValidationWarning {
                issue_type: ValidationIssueType::Uncertainty,
                message: format!("Response starts with uncertain language: '{}'", phrase),
                severity: "low".to_string(),
            });
        }
    }

    None
}

/// Check for hallucinations - specific claims not grounded in command output
pub fn check_hallucination(
    text: &str,
    command_output: &str,
    grounding_values: &[String],
) -> Option<ValidationWarning> {
    let text_lower = text.to_lowercase();
    let output_lower = command_output.to_lowercase();

    // Check for specific numeric claims not in output
    for cap in RE_NUMBER.captures_iter(text) {
        let full_match = cap.get(0)?.as_str();
        let number = cap.get(1)?.as_str();

        if !output_lower.contains(number) && !output_lower.contains(&full_match.to_lowercase()) {
            if number.len() >= 2 || full_match.contains('%') {
                return Some(ValidationWarning {
                    issue_type: ValidationIssueType::Hallucination,
                    message: format!(
                        "Specific value '{}' not found in command output",
                        full_match
                    ),
                    severity: "high".to_string(),
                });
            }
        }
    }

    // Check for service/package names not in output
    for cap in RE_NAME.captures_iter(&text_lower) {
        let name = cap.get(1)?.as_str();

        if is_common_word(name) {
            continue;
        }

        if !output_lower.contains(name) && !grounding_values.contains(&name.to_string()) {
            if name.ends_with(".service") {
                return Some(ValidationWarning {
                    issue_type: ValidationIssueType::Hallucination,
                    message: format!("Service '{}' not mentioned in command output", name),
                    severity: "medium".to_string(),
                });
            }
        }
    }

    None
}

/// Check if response is too generic
pub fn check_too_generic(text: &str) -> Option<ValidationWarning> {
    let text_lower = text.to_lowercase();

    let generic_phrases = [
        "in general",
        "typically",
        "usually",
        "on most systems",
        "it depends on",
        "varies depending",
        "check your configuration",
        "refer to the documentation",
        "consult the manual",
    ];

    let generic_count = generic_phrases
        .iter()
        .filter(|p| text_lower.contains(*p))
        .count();

    if generic_count >= 2 {
        return Some(ValidationWarning {
            issue_type: ValidationIssueType::TooGeneric,
            message: "Response contains generic advice instead of system-specific information"
                .to_string(),
            severity: "medium".to_string(),
        });
    }

    None
}

/// Check if a word is a common English word
pub fn is_common_word(word: &str) -> bool {
    const COMMON_WORDS: &[&str] = &[
        "the", "and", "for", "are", "but", "not", "you", "all", "can", "had", "her", "was", "one",
        "our", "out", "has", "his", "how", "its", "may", "new", "now", "old", "see", "way", "who",
        "did", "get", "has", "him", "into", "just", "made", "many", "over", "such", "than", "them",
        "then", "there", "these", "they", "this", "time", "very", "when", "which", "will", "with",
        "would", "your", "about", "after", "being", "below", "could", "every", "first", "found",
        "great", "have", "here", "into", "know", "like", "line", "look", "make", "more", "most",
        "name", "need", "next", "number", "only", "other", "over", "part", "people", "place",
        "point", "right", "said", "same", "should", "show", "since", "some", "state", "still",
        "such", "take", "than", "that", "their", "them", "then", "there", "these", "they", "thing",
        "think", "those", "through", "time", "under", "using", "want", "water", "well", "were",
        "what", "where", "while", "work", "world", "write", "year", "also", "back", "been",
        "before", "come", "down", "even", "from", "good", "help", "here", "home", "just", "keep",
        "last", "long", "much", "must", "never", "only", "other", "over", "system", "running",
        "output", "command", "information", "available", "installed", "version", "current",
        "directory", "service", "process",
    ];

    COMMON_WORDS.contains(&word)
}

/// Extract specific values from command output for grounding check
pub fn extract_grounding_values(output: &str) -> Vec<String> {
    let mut values = Vec::new();

    for word in output.split_whitespace() {
        let clean = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '.' && c != '-');

        if clean.contains('.') && clean.chars().any(|c| c.is_numeric()) {
            if clean.len() >= 3 && clean.len() <= 20 {
                values.push(clean.to_string());
            }
        }

        if clean.len() > 3
            && clean
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            values.push(clean.to_lowercase());
        }

        if clean.chars().any(|c| c.is_numeric()) {
            if let Some(num_part) = extract_number_with_unit(clean) {
                values.push(num_part);
            }
        }
    }

    for line in output.lines() {
        for word in line.split_whitespace() {
            if word.starts_with('/') && word.len() > 3 {
                values.push(word.to_string());
            }
        }
    }

    values.sort();
    values.dedup();
    values
}

/// Extract number with unit (e.g., "16GB" -> "16")
fn extract_number_with_unit(s: &str) -> Option<String> {
    let num: String = s
        .chars()
        .take_while(|c| c.is_numeric() || *c == '.')
        .collect();
    if !num.is_empty() && num != "0" {
        Some(num)
    } else {
        None
    }
}
