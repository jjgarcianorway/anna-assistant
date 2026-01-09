//! Streaming Answer Validation (v0.0.889)
//!
//! Validates LLM answers as they stream, detecting:
//! - Hallucinations (claims not supported by command output)
//! - Uncertainty markers (vague language)
//! - Contradictions with command output
//! - Generic responses not specific to the system

use anna_shared::rpc::{ValidationIssueType, ValidationWarning};

/// Streaming validator that accumulates tokens and checks for issues
pub struct StreamingValidator {
    /// Accumulated answer text so far
    accumulated: String,
    /// Command output to validate against
    command_output: String,
    /// Keywords/values from command output for grounding check
    grounding_values: Vec<String>,
    /// Detected warnings (deduplicated)
    warnings: Vec<ValidationWarning>,
    /// Last sentence validated (to avoid re-validating)
    last_validated_len: usize,
}

impl StreamingValidator {
    /// Create a new validator with command output to check against
    pub fn new(command_output: &str) -> Self {
        let grounding_values = extract_grounding_values(command_output);
        Self {
            accumulated: String::new(),
            command_output: command_output.to_string(),
            grounding_values,
            warnings: Vec::new(),
            last_validated_len: 0,
        }
    }

    /// Add a token and check for new validation issues
    /// Returns any new warnings detected
    pub fn add_token(&mut self, token: &str) -> Vec<ValidationWarning> {
        self.accumulated.push_str(token);

        // Only validate when we have a complete sentence or phrase
        // (ends with punctuation or is sufficiently long)
        if !should_validate(&self.accumulated, self.last_validated_len) {
            return Vec::new();
        }

        let new_text = &self.accumulated[self.last_validated_len..];
        let mut new_warnings = Vec::new();

        // Check for uncertainty markers
        if let Some(warning) = check_uncertainty(new_text) {
            if !self.has_warning(&warning) {
                self.warnings.push(warning.clone());
                new_warnings.push(warning);
            }
        }

        // Check for hallucinations (specific values not in command output)
        if let Some(warning) = check_hallucination(new_text, &self.command_output, &self.grounding_values) {
            if !self.has_warning(&warning) {
                self.warnings.push(warning.clone());
                new_warnings.push(warning);
            }
        }

        // Check for generic responses
        if let Some(warning) = check_too_generic(new_text) {
            if !self.has_warning(&warning) {
                self.warnings.push(warning.clone());
                new_warnings.push(warning);
            }
        }

        // v0.0.890: Check for contradictions with command output
        if let Some(warning) = check_contradiction(new_text, &self.command_output) {
            if !self.has_warning(&warning) {
                self.warnings.push(warning.clone());
                new_warnings.push(warning);
            }
        }

        self.last_validated_len = self.accumulated.len();
        new_warnings
    }

    /// Get all accumulated warnings
    pub fn get_warnings(&self) -> &[ValidationWarning] {
        &self.warnings
    }

    /// Check if we already have a similar warning
    fn has_warning(&self, warning: &ValidationWarning) -> bool {
        self.warnings.iter().any(|w| {
            std::mem::discriminant(&w.issue_type) == std::mem::discriminant(&warning.issue_type)
                && w.message == warning.message
        })
    }
}

/// Determine if we should validate now (have complete sentence)
fn should_validate(text: &str, last_len: usize) -> bool {
    let new_text = &text[last_len..];

    // Validate if we have at least 50 new characters
    if new_text.len() < 50 {
        return false;
    }

    // Validate if we ended a sentence
    new_text.contains(". ")
        || new_text.contains(".\n")
        || new_text.ends_with('.')
        || new_text.ends_with('!')
        || new_text.ends_with('?')
        || new_text.ends_with(':')
}

/// Extract specific values from command output for grounding check
fn extract_grounding_values(output: &str) -> Vec<String> {
    let mut values = Vec::new();

    // Extract numbers (file sizes, versions, counts, etc.)
    for word in output.split_whitespace() {
        let clean = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '.' && c != '-');

        // Version numbers like "6.8.1"
        if clean.contains('.') && clean.chars().any(|c| c.is_numeric()) {
            if clean.len() >= 3 && clean.len() <= 20 {
                values.push(clean.to_string());
            }
        }

        // Package names (lowercase with dashes)
        if clean.len() > 3 && clean.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            values.push(clean.to_lowercase());
        }

        // Numbers with units (4GB, 500MB, etc.)
        if clean.chars().any(|c| c.is_numeric()) {
            if let Some(num_part) = extract_number_with_unit(clean) {
                values.push(num_part);
            }
        }
    }

    // Extract file paths
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
    let num: String = s.chars().take_while(|c| c.is_numeric() || *c == '.').collect();
    if num.len() >= 1 && num != "0" {
        Some(num)
    } else {
        None
    }
}

/// Check for uncertainty markers
fn check_uncertainty(text: &str) -> Option<ValidationWarning> {
    let text_lower = text.to_lowercase();

    let uncertainty_phrases = [
        ("might be", "low"),
        ("could be", "low"),
        ("perhaps", "low"),
        ("possibly", "low"),
        ("i think", "medium"),
        ("i believe", "medium"),
        ("probably", "low"),
        ("likely", "low"),
        ("not sure", "medium"),
        ("unsure", "medium"),
        ("hard to say", "medium"),
        ("can't tell", "medium"),
        ("unable to determine", "medium"),
        ("without more information", "medium"),
    ];

    for (phrase, severity) in uncertainty_phrases {
        if text_lower.contains(phrase) {
            return Some(ValidationWarning {
                issue_type: ValidationIssueType::Uncertainty,
                message: format!("Response uses uncertain language: '{}'", phrase),
                severity: severity.to_string(),
            });
        }
    }

    None
}

/// Check for hallucinations - specific claims not grounded in command output
fn check_hallucination(text: &str, command_output: &str, grounding_values: &[String]) -> Option<ValidationWarning> {
    let text_lower = text.to_lowercase();
    let output_lower = command_output.to_lowercase();

    // Check for specific numeric claims not in output
    let number_pattern = regex::Regex::new(r"\b(\d+\.?\d*)\s*(GB|MB|KB|TB|GiB|MiB|cores?|threads?|%)\b").ok()?;

    for cap in number_pattern.captures_iter(text) {
        let full_match = cap.get(0)?.as_str();
        let number = cap.get(1)?.as_str();

        // Check if this specific number appears in command output
        if !output_lower.contains(number) && !output_lower.contains(&full_match.to_lowercase()) {
            // Only flag if it's a specific enough claim
            if number.len() >= 2 || full_match.contains('%') {
                return Some(ValidationWarning {
                    issue_type: ValidationIssueType::Hallucination,
                    message: format!("Specific value '{}' not found in command output", full_match),
                    severity: "high".to_string(),
                });
            }
        }
    }

    // Check for service/package names that weren't in output
    let name_pattern = regex::Regex::new(r"\b([a-z][\w-]*\.service|[a-z][\w-]{3,})\b").ok()?;

    for cap in name_pattern.captures_iter(&text_lower) {
        let name = cap.get(1)?.as_str();

        // Skip common words
        if is_common_word(name) {
            continue;
        }

        // Check if this name appears in command output or grounding values
        if !output_lower.contains(name) && !grounding_values.contains(&name.to_string()) {
            // Only flag .service names with high confidence
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
fn check_too_generic(text: &str) -> Option<ValidationWarning> {
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

    // Only flag if the response is primarily generic advice
    let generic_count = generic_phrases.iter().filter(|p| text_lower.contains(*p)).count();

    if generic_count >= 2 {
        return Some(ValidationWarning {
            issue_type: ValidationIssueType::TooGeneric,
            message: "Response contains generic advice instead of system-specific information".to_string(),
            severity: "medium".to_string(),
        });
    }

    None
}

// ============================================================================
// CONTRADICTION DETECTION (v0.0.890)
// ============================================================================

/// Check for contradictions between answer text and command output
fn check_contradiction(text: &str, command_output: &str) -> Option<ValidationWarning> {
    let text_lower = text.to_lowercase();
    let output_lower = command_output.to_lowercase();

    // 1. Status contradictions: answer says "running/active" but output shows "stopped/inactive"
    if let Some(warning) = check_status_contradiction(&text_lower, &output_lower) {
        return Some(warning);
    }

    // 2. Numeric contradictions: answer gives different number than output
    if let Some(warning) = check_numeric_contradiction(&text_lower, &output_lower) {
        return Some(warning);
    }

    // 3. Presence contradictions: answer says "installed" but output shows "not found"
    if let Some(warning) = check_presence_contradiction(&text_lower, &output_lower) {
        return Some(warning);
    }

    // 4. Boolean contradictions: answer says "yes/enabled" but output shows "no/disabled"
    if let Some(warning) = check_boolean_contradiction(&text_lower, &output_lower) {
        return Some(warning);
    }

    None
}

/// Check for status contradictions (running vs stopped, active vs inactive)
fn check_status_contradiction(answer: &str, output: &str) -> Option<ValidationWarning> {
    // Pairs of contradicting status terms
    let status_pairs = [
        ("running", "stopped"),
        ("running", "dead"),
        ("running", "not running"),
        ("active", "inactive"),
        ("active", "failed"),
        ("enabled", "disabled"),
        ("up", "down"),
        ("online", "offline"),
        ("started", "stopped"),
        ("healthy", "unhealthy"),
        ("connected", "disconnected"),
        ("mounted", "unmounted"),
        ("loaded", "not loaded"),
    ];

    for (positive, negative) in status_pairs {
        // Answer says positive, but output shows negative
        if answer.contains(positive) && output.contains(negative) && !output.contains(positive) {
            return Some(ValidationWarning {
                issue_type: ValidationIssueType::Contradiction,
                message: format!(
                    "Answer says '{}' but command output shows '{}'",
                    positive, negative
                ),
                severity: "high".to_string(),
            });
        }
        // Answer says negative, but output shows positive
        if answer.contains(negative) && output.contains(positive) && !output.contains(negative) {
            return Some(ValidationWarning {
                issue_type: ValidationIssueType::Contradiction,
                message: format!(
                    "Answer says '{}' but command output shows '{}'",
                    negative, positive
                ),
                severity: "high".to_string(),
            });
        }
    }

    None
}

/// Check for numeric contradictions with tolerance for units
fn check_numeric_contradiction(answer: &str, output: &str) -> Option<ValidationWarning> {
    // Pattern to find memory/storage values with units (case-insensitive)
    // Note: "Gi" is common shorthand for GiB in `free` output
    let mem_pattern = regex::Regex::new(r"(?i)(\d+(?:\.\d+)?)\s*(gb|mb|tb|gib|mib|tib|gi|mi|ti|g|m|t)\b").ok()?;

    // Extract all numeric values with memory units from answer
    for cap in mem_pattern.captures_iter(answer) {
        let answer_num: f64 = cap.get(1)?.as_str().parse().ok()?;
        let answer_unit = cap.get(2)?.as_str();

        // Normalize to a common unit (GB)
        let answer_gb = normalize_to_gb(answer_num, answer_unit);

        // Look for contradicting values in output
        for out_cap in mem_pattern.captures_iter(output) {
            let output_num: f64 = out_cap.get(1)?.as_str().parse().ok()?;
            let output_unit = out_cap.get(2)?.as_str();
            let output_gb = normalize_to_gb(output_num, output_unit);

            // Check if values are in same ballpark but significantly different
            // Allow 15% tolerance for rounding differences (16GB vs 15.9Gi)
            if answer_gb > 0.5 && output_gb > 0.5 {
                let ratio = answer_gb / output_gb;
                if ratio < 0.5 || ratio > 2.0 {
                    // More than 2x difference - likely a contradiction
                    return Some(ValidationWarning {
                        issue_type: ValidationIssueType::Contradiction,
                        message: format!(
                            "Answer states '{:.1}{} ' but output shows '{:.1}{}'",
                            answer_num, answer_unit, output_num, output_unit
                        ),
                        severity: "high".to_string(),
                    });
                }
            }
        }
    }

    None
}

/// Normalize memory value to GB for comparison
fn normalize_to_gb(value: f64, unit: &str) -> f64 {
    match unit.to_lowercase().as_str() {
        "tb" | "tib" | "ti" | "t" => value * 1024.0,
        "gb" | "gib" | "gi" | "g" => value,
        "mb" | "mib" | "mi" | "m" => value / 1024.0,
        "kb" | "kib" | "ki" | "k" => value / (1024.0 * 1024.0),
        _ => value,
    }
}

/// Check for presence contradictions (installed vs not found)
fn check_presence_contradiction(answer: &str, output: &str) -> Option<ValidationWarning> {
    // Answer says something is installed/present but output shows otherwise
    let presence_positive = ["installed", "available", "found", "exists", "present"];
    let presence_negative = ["not installed", "not found", "not available", "does not exist",
                             "no such", "error: target not found", "package not found"];

    // If answer claims something is installed
    for positive in presence_positive {
        if answer.contains(positive) {
            // Check if output contradicts
            for negative in presence_negative {
                if output.contains(negative) {
                    return Some(ValidationWarning {
                        issue_type: ValidationIssueType::Contradiction,
                        message: format!(
                            "Answer says '{}' but command output shows '{}'",
                            positive, negative
                        ),
                        severity: "high".to_string(),
                    });
                }
            }
        }
    }

    // If answer claims something is not installed
    for negative in presence_negative {
        let neg_parts: Vec<&str> = negative.split_whitespace().collect();
        if neg_parts.iter().any(|&n| answer.contains(n)) {
            // Check if output shows it IS installed (package name in output without "not")
            if presence_positive.iter().any(|&p| output.contains(p)) && !output.contains("not") {
                return Some(ValidationWarning {
                    issue_type: ValidationIssueType::Contradiction,
                    message: "Answer claims something is not present but command output suggests it exists".to_string(),
                    severity: "medium".to_string(),
                });
            }
        }
    }

    None
}

/// Check for boolean contradictions (yes/no, enabled/disabled)
fn check_boolean_contradiction(answer: &str, output: &str) -> Option<ValidationWarning> {
    let bool_pairs = [
        ("yes", "no"),
        ("true", "false"),
        ("on", "off"),
        ("1", "0"),
        ("success", "failed"),
        ("passed", "failed"),
        ("ok", "error"),
    ];

    // Only check these in specific contexts (after "is" or "=" or ":")
    // to avoid false positives from natural language
    let context_pattern = regex::Regex::new(r"(?:is|=|:)\s*(\w+)").ok()?;

    for cap in context_pattern.captures_iter(answer) {
        let answer_val = cap.get(1)?.as_str().to_lowercase();

        for (positive, negative) in bool_pairs {
            // Answer has positive value
            if answer_val == positive {
                // Check if output shows negative
                for out_cap in context_pattern.captures_iter(output) {
                    let output_val = out_cap.get(1)?.as_str().to_lowercase();
                    if output_val == negative {
                        return Some(ValidationWarning {
                            issue_type: ValidationIssueType::Contradiction,
                            message: format!("Answer shows '{}' but output shows '{}'", positive, negative),
                            severity: "high".to_string(),
                        });
                    }
                }
            }
        }
    }

    None
}

/// Check if a word is a common English word (not a package/service name)
fn is_common_word(word: &str) -> bool {
    const COMMON_WORDS: &[&str] = &[
        "the", "and", "for", "are", "but", "not", "you", "all", "can", "had",
        "her", "was", "one", "our", "out", "has", "his", "how", "its", "may",
        "new", "now", "old", "see", "way", "who", "did", "get", "has", "him",
        "into", "just", "made", "many", "over", "such", "than", "them", "then",
        "there", "these", "they", "this", "time", "very", "when", "which", "will",
        "with", "would", "your", "about", "after", "being", "below", "could",
        "every", "first", "found", "great", "have", "here", "into", "know",
        "like", "line", "look", "make", "more", "most", "name", "need", "next",
        "number", "only", "other", "over", "part", "people", "place", "point",
        "right", "said", "same", "should", "show", "since", "some", "state",
        "still", "such", "take", "than", "that", "their", "them", "then", "there",
        "these", "they", "thing", "think", "those", "through", "time", "under",
        "using", "want", "water", "well", "were", "what", "where", "while",
        "work", "world", "write", "year", "also", "back", "been", "before",
        "come", "down", "even", "from", "good", "help", "here", "home", "just",
        "keep", "last", "long", "much", "must", "never", "only", "other", "over",
        "system", "running", "output", "command", "information", "available",
        "installed", "version", "current", "directory", "service", "process",
    ];

    COMMON_WORDS.contains(&word)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uncertainty_detection() {
        let warning = check_uncertainty("I think the disk might be full");
        assert!(warning.is_some());
        assert!(matches!(warning.unwrap().issue_type, ValidationIssueType::Uncertainty));
    }

    #[test]
    fn test_grounding_extraction() {
        let output = "Mem: 16Gi total\n/dev/sda1 500G\nlinux 6.8.1-arch1-1";
        let values = extract_grounding_values(output);
        assert!(values.iter().any(|v| v.contains("16")));
        assert!(values.iter().any(|v| v.contains("6.8.1")));
    }

    #[test]
    fn test_no_false_positives() {
        let text = "Your system has 16GB of RAM and is running kernel 6.8.1";
        let output = "Mem: 16Gi total\nlinux 6.8.1-arch1-1";
        let warning = check_hallucination(text, output, &extract_grounding_values(output));
        assert!(warning.is_none());
    }

    // v0.0.890: Contradiction detection tests
    #[test]
    fn test_status_contradiction() {
        let answer = "The service is running normally";
        let output = "nginx.service - A high performance web server\n   Active: inactive (dead)";
        let warning = check_contradiction(answer, output);
        assert!(warning.is_some());
        assert!(matches!(warning.unwrap().issue_type, ValidationIssueType::Contradiction));
    }

    #[test]
    fn test_no_status_contradiction() {
        let answer = "The service is running";
        let output = "nginx.service\n   Active: active (running)";
        let warning = check_contradiction(answer, output);
        assert!(warning.is_none());
    }

    #[test]
    fn test_numeric_contradiction() {
        let answer = "You have 32GB of RAM";
        let output = "Mem:           15Gi";
        let warning = check_numeric_contradiction(answer, output);
        assert!(warning.is_some());
    }

    #[test]
    fn test_presence_contradiction() {
        let answer = "Firefox is installed on your system";
        let output = "error: package 'firefox' was not found";
        let warning = check_presence_contradiction(answer, output);
        assert!(warning.is_some());
    }
}
