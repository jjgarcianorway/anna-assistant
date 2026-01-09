//! Streaming Answer Validation (v0.0.889)
//!
//! Validates LLM answers as they stream, detecting:
//! - Hallucinations (claims not supported by command output)
//! - Uncertainty markers (vague language)
//! - Contradictions with command output
//! - Generic responses not specific to the system

use anna_shared::rpc::{ValidationIssueType, ValidationWarning};
use regex::Regex;
use std::sync::LazyLock;

/// v0.0.893: Pre-compiled regexes for validation (avoids per-call compilation)
static RE_NUMBER: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r"\b(\d+\.?\d*)\s*(GB|MB|KB|TB|GiB|MiB|cores?|threads?|%)\b").unwrap()
);
static RE_NAME: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r"\b([a-z][\w-]*\.service|[a-z][\w-]{3,})\b").unwrap()
);
static RE_MEM: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r"(?i)(\d+(?:\.\d+)?)\s*(gb|mb|tb|gib|mib|tib|gi|mi|ti|g|m|t)\b").unwrap()
);
static RE_CONTEXT: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r"(?:is|=|:)\s*(\w+)").unwrap()
);

/// Streaming validator that accumulates tokens and checks for issues
/// v0.0.897: Added confidence tracking for self-correction
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
    /// v0.0.897: Running confidence score (starts at 1.0, decreases with issues)
    confidence: f32,
}

/// v0.0.897: Result of validation with confidence info
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// New warnings detected
    pub warnings: Vec<ValidationWarning>,
    /// Current confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Whether self-correction is recommended
    pub needs_correction: bool,
    /// Specific issue to address if correction needed
    pub correction_hint: Option<String>,
}

/// v0.0.897: Confidence penalty for each issue type
const PENALTY_HALLUCINATION: f32 = 0.25;
const PENALTY_CONTRADICTION: f32 = 0.30;
const PENALTY_UNCERTAINTY: f32 = 0.10;
const PENALTY_TOO_GENERIC: f32 = 0.15;
/// v0.0.897: Threshold below which self-correction is recommended
const CORRECTION_THRESHOLD: f32 = 0.5;

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
            confidence: 1.0,
        }
    }

    /// Add a token and check for new validation issues
    /// Returns any new warnings detected
    pub fn add_token(&mut self, token: &str) -> Vec<ValidationWarning> {
        let result = self.add_token_with_confidence(token);
        result.warnings
    }

    /// v0.0.897: Add a token and return full validation result with confidence
    pub fn add_token_with_confidence(&mut self, token: &str) -> ValidationResult {
        self.accumulated.push_str(token);

        // Only validate when we have a complete sentence or phrase
        if !should_validate(&self.accumulated, self.last_validated_len) {
            return ValidationResult {
                warnings: Vec::new(),
                confidence: self.confidence,
                needs_correction: false,
                correction_hint: None,
            };
        }

        let new_text = &self.accumulated[self.last_validated_len..];
        let mut new_warnings = Vec::new();
        let mut correction_hint = None;

        // Check for uncertainty markers
        if let Some(warning) = check_uncertainty(new_text) {
            if !self.has_warning(&warning) {
                self.confidence -= PENALTY_UNCERTAINTY;
                self.warnings.push(warning.clone());
                new_warnings.push(warning);
            }
        }

        // Check for hallucinations (specific values not in command output)
        if let Some(warning) = check_hallucination(new_text, &self.command_output, &self.grounding_values) {
            if !self.has_warning(&warning) {
                self.confidence -= PENALTY_HALLUCINATION;
                correction_hint = Some(format!("Verify claim: {}", warning.message));
                self.warnings.push(warning.clone());
                new_warnings.push(warning);
            }
        }

        // Check for generic responses
        if let Some(warning) = check_too_generic(new_text) {
            if !self.has_warning(&warning) {
                self.confidence -= PENALTY_TOO_GENERIC;
                self.warnings.push(warning.clone());
                new_warnings.push(warning);
            }
        }

        // v0.0.890: Check for contradictions with command output
        if let Some(warning) = check_contradiction(new_text, &self.command_output) {
            if !self.has_warning(&warning) {
                self.confidence -= PENALTY_CONTRADICTION;
                correction_hint = Some(format!("Contradiction detected: {}", warning.message));
                self.warnings.push(warning.clone());
                new_warnings.push(warning);
            }
        }

        self.last_validated_len = self.accumulated.len();
        self.confidence = self.confidence.max(0.0); // Don't go below 0

        ValidationResult {
            warnings: new_warnings,
            confidence: self.confidence,
            needs_correction: self.confidence < CORRECTION_THRESHOLD,
            correction_hint,
        }
    }

    /// v0.0.897: Get current confidence score
    pub fn get_confidence(&self) -> f32 {
        self.confidence
    }

    /// v0.0.897: Check if self-correction is recommended
    pub fn needs_correction(&self) -> bool {
        self.confidence < CORRECTION_THRESHOLD
    }

    /// v0.0.897: Get summary of issues for correction prompt
    pub fn get_correction_summary(&self) -> Option<String> {
        if self.warnings.is_empty() {
            return None;
        }

        let issues: Vec<String> = self.warnings.iter()
            .map(|w| format!("- {:?}: {}", w.issue_type, w.message))
            .collect();

        Some(format!(
            "Issues detected (confidence: {:.0}%):\n{}",
            self.confidence * 100.0,
            issues.join("\n")
        ))
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
/// v0.0.891: Smarter detection to avoid false positives on valid hedging
fn check_uncertainty(text: &str) -> Option<ValidationWarning> {
    let text_lower = text.to_lowercase();

    // Only check early in the response - uncertainty at the end is often valid hedging
    // e.g., "The disk is 80% full, though this might increase if you download more files"
    let check_text = if text_lower.len() > 150 {
        &text_lower[..150]
    } else {
        &text_lower
    };

    // High-confidence uncertainty markers (always flag)
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

    // Weak uncertainty markers - only flag if they appear at the very start
    // These are often valid hedging when used mid-sentence
    let weak_uncertainty = ["might be", "could be", "perhaps", "possibly", "probably", "likely"];

    // Only flag if the answer STARTS with uncertainty (first 50 chars)
    let start_text = if text_lower.len() > 50 { &text_lower[..50] } else { &text_lower };

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
/// v0.0.893: Uses pre-compiled regexes
fn check_hallucination(text: &str, command_output: &str, grounding_values: &[String]) -> Option<ValidationWarning> {
    let text_lower = text.to_lowercase();
    let output_lower = command_output.to_lowercase();

    // Check for specific numeric claims not in output (v0.0.893: pre-compiled)
    for cap in RE_NUMBER.captures_iter(text) {
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

    // Check for service/package names that weren't in output (v0.0.893: pre-compiled)
    for cap in RE_NAME.captures_iter(&text_lower) {
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
/// v0.0.893: Uses pre-compiled regex
fn check_numeric_contradiction(answer: &str, output: &str) -> Option<ValidationWarning> {
    // Extract all numeric values with memory units from answer (v0.0.893: pre-compiled)
    for cap in RE_MEM.captures_iter(answer) {
        let answer_num: f64 = cap.get(1)?.as_str().parse().ok()?;
        let answer_unit = cap.get(2)?.as_str();
        let answer_gb = normalize_to_gb(answer_num, answer_unit);

        // Look for contradicting values in output
        for out_cap in RE_MEM.captures_iter(output) {
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

    // Only check in contexts after "is", "=", or ":" (v0.0.893: pre-compiled)
    for cap in RE_CONTEXT.captures_iter(answer) {
        let answer_val = cap.get(1)?.as_str().to_lowercase();

        for (positive, negative) in bool_pairs {
            if answer_val == positive {
                for out_cap in RE_CONTEXT.captures_iter(output) {
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

/// v0.0.898: Validate a complete answer against command output
/// Returns validation result with confidence score and potential correction hints
pub fn validate_complete_answer(answer: &str, command_output: &str) -> ValidationResult {
    validate_and_learn(answer, command_output, None)
}

/// v0.0.899: Validate and optionally record contradictions for learning
pub fn validate_and_learn(answer: &str, command_output: &str, source_cmd: Option<&str>) -> ValidationResult {
    use anna_shared::session::ContradictionStore;

    let mut warnings = Vec::new();
    let mut confidence = 1.0f32;
    let mut correction_hints = Vec::new();
    let mut detected_contradictions: Vec<(String, String, String)> = Vec::new();

    // Check uncertainty
    if let Some(warning) = check_uncertainty(answer) {
        confidence -= PENALTY_UNCERTAINTY;
        warnings.push(warning);
    }

    // Check hallucinations
    let grounding_values = extract_grounding_values(command_output);
    if let Some(warning) = check_hallucination(answer, command_output, &grounding_values) {
        confidence -= PENALTY_HALLUCINATION;
        correction_hints.push(format!("Verify: {}", warning.message));

        // v0.0.899: Record for learning
        if let Some(claim) = extract_claim_from_warning(&warning.message) {
            detected_contradictions.push(("numeric_claim".to_string(), claim, "see output".to_string()));
        }
        warnings.push(warning);
    }

    // Check generic responses
    if let Some(warning) = check_too_generic(answer) {
        confidence -= PENALTY_TOO_GENERIC;
        correction_hints.push("Be more specific with system data".to_string());
        warnings.push(warning);
    }

    // Check contradictions
    if let Some(warning) = check_contradiction(answer, command_output) {
        confidence -= PENALTY_CONTRADICTION;
        correction_hints.push(format!("Fix: {}", warning.message));

        // v0.0.899: Record specific contradiction for learning
        if let Some((wrong, correct)) = extract_contradiction_details(&warning.message) {
            detected_contradictions.push(("status".to_string(), wrong, correct));
        }
        warnings.push(warning);
    }

    // v0.0.898: Check for "does not exist" contradictions
    if let Some(warning) = check_existence_contradiction(answer, command_output) {
        confidence -= PENALTY_CONTRADICTION;
        correction_hints.push(format!("Entity issue: {}", warning.message));
        detected_contradictions.push(("existence".to_string(), "exists".to_string(), "does not exist".to_string()));
        warnings.push(warning);
    }

    // v0.0.898: Check arithmetic errors (sums, differences)
    if let Some(warning) = check_arithmetic_error(answer, command_output) {
        confidence -= PENALTY_HALLUCINATION;
        correction_hints.push(format!("Math error: {}", warning.message));
        if let Some(claim) = extract_claim_from_warning(&warning.message) {
            detected_contradictions.push(("arithmetic".to_string(), claim, "incorrect math".to_string()));
        }
        warnings.push(warning);
    }

    // v0.0.899: Record contradictions for future prevention
    // v0.0.902: Also penalize experiences that led to contradictions
    if !detected_contradictions.is_empty() {
        let cmd = source_cmd.unwrap_or("unknown");
        let mut store = ContradictionStore::load();
        for (claim_type, wrong, correct) in &detected_contradictions {
            store.record(claim_type, wrong, correct, cmd);
        }
        let _ = store.save();

        // Penalize experiences that suggested this command
        if let Ok(mut memory) = anna_shared::memory::Memory::load() {
            let mut penalized = false;
            for exp in memory.experiences.iter_mut() {
                if exp.successful_commands.iter().any(|c| c == cmd || cmd.starts_with(c.split_whitespace().next().unwrap_or(""))) {
                    // Reduce usefulness but don't go below 1
                    exp.usefulness_score = exp.usefulness_score.saturating_sub(1).max(1);
                    penalized = true;
                }
            }
            if penalized {
                let _ = memory.save();
                tracing::debug!("Penalized experiences for contradiction from command: {}", cmd);
            }
        }
    }

    confidence = confidence.max(0.0);

    ValidationResult {
        warnings,
        confidence,
        needs_correction: confidence < CORRECTION_THRESHOLD,
        correction_hint: if correction_hints.is_empty() {
            None
        } else {
            Some(correction_hints.join("; "))
        },
    }
}

/// v0.0.899: Extract a claim value from warning message
fn extract_claim_from_warning(message: &str) -> Option<String> {
    // Look for quoted values or numbers with units
    if let Some(start) = message.find('\'') {
        if let Some(end) = message[start+1..].find('\'') {
            return Some(message[start+1..start+1+end].to_string());
        }
    }
    // Try to find a number pattern
    for word in message.split_whitespace() {
        if word.chars().any(|c| c.is_numeric()) && word.len() < 20 {
            return Some(word.to_string());
        }
    }
    None
}

/// v0.0.899: Extract wrong/correct values from contradiction message
fn extract_contradiction_details(message: &str) -> Option<(String, String)> {
    // Message format: "Answer says 'X' but command output shows 'Y'"
    if message.contains("says") && message.contains("but") {
        let parts: Vec<&str> = message.split("but").collect();
        if parts.len() == 2 {
            let wrong = extract_claim_from_warning(parts[0]);
            let correct = extract_claim_from_warning(parts[1]);
            if let (Some(w), Some(c)) = (wrong, correct) {
                return Some((w, c));
            }
        }
    }
    None
}

/// v0.0.898: Check if answer assumes something exists but output shows it doesn't
fn check_existence_contradiction(answer: &str, output: &str) -> Option<ValidationWarning> {
    let output_lower = output.to_lowercase();

    // Patterns indicating non-existence
    let nonexist_patterns = [
        "does not exist",
        "no such file",
        "not found",
        "unit .* could not be found",
        "no packages found",
        "command not found",
    ];

    // If output shows something doesn't exist
    for pattern in nonexist_patterns {
        if output_lower.contains(pattern) || (pattern.contains(".*") && {
            let re = Regex::new(&format!("(?i){}", pattern)).ok();
            re.map(|r| r.is_match(&output_lower)).unwrap_or(false)
        }) {
            // Check if answer makes claims about properties of that thing
            let answer_lower = answer.to_lowercase();
            let property_claims = ["is ", "has ", "uses ", "runs ", "contains "];

            for claim in property_claims {
                if answer_lower.contains(claim) && !answer_lower.contains("does not exist")
                    && !answer_lower.contains("not found") && !answer_lower.contains("doesn't exist") {
                    return Some(ValidationWarning {
                        issue_type: ValidationIssueType::Contradiction,
                        message: format!(
                            "Answer describes properties of something that doesn't exist (output shows: {})",
                            pattern.replace(".*", "...")
                        ),
                        severity: "high".to_string(),
                    });
                }
            }
        }
    }

    None
}

/// v0.0.898: Check for arithmetic errors in derived values
fn check_arithmetic_error(answer: &str, output: &str) -> Option<ValidationWarning> {
    // Extract all memory values from output
    let output_values: Vec<f64> = RE_MEM.captures_iter(output)
        .filter_map(|cap| {
            let num: f64 = cap.get(1)?.as_str().parse().ok()?;
            let unit = cap.get(2)?.as_str();
            Some(normalize_to_gb(num, unit))
        })
        .collect();

    if output_values.len() < 2 {
        return None; // Need multiple values to check arithmetic
    }

    // Check answer values against reasonable sums/differences
    for cap in RE_MEM.captures_iter(answer) {
        let answer_num: f64 = cap.get(1)?.as_str().parse().ok()?;
        let answer_unit = cap.get(2)?.as_str();
        let answer_gb = normalize_to_gb(answer_num, answer_unit);

        // Check if answer value is close to any output value
        let matches_single = output_values.iter().any(|&v| {
            let ratio = answer_gb / v;
            ratio > 0.85 && ratio < 1.15  // 15% tolerance
        });

        if matches_single {
            continue; // This value is directly from output
        }

        // Check if it could be a sum of output values
        let total: f64 = output_values.iter().sum();
        let sum_ratio = answer_gb / total;
        let is_reasonable_sum = sum_ratio > 0.85 && sum_ratio < 1.15;

        // Check if answer is way off (more than 2x any individual or sum)
        let max_output = output_values.iter().cloned().fold(f64::MIN, f64::max);
        if answer_gb > max_output * 2.5 && !is_reasonable_sum {
            return Some(ValidationWarning {
                issue_type: ValidationIssueType::Hallucination,
                message: format!(
                    "Answer claims {:.1}{} but output values don't support this (max: {:.1}GB, sum: {:.1}GB)",
                    answer_num, answer_unit, max_output, total
                ),
                severity: "high".to_string(),
            });
        }
    }

    None
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
