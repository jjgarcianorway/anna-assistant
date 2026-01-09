//! Intent classification module - LLM-based question understanding.
//!
//! Uses chain-of-thought reasoning to deeply understand user requests
//! before taking action. This makes Anna think like Claude - paraphrasing,
//! identifying missing info, and asking for clarification when needed.

use anna_shared::rpc::{DeepUnderstanding, IntentCategory, IntentClassification};
use anyhow::Result;
use tracing::{debug, info, warn};

use crate::ollama;

/// Timeout for understanding (slightly longer for chain-of-thought)
const UNDERSTAND_TIMEOUT_SECS: u64 = 25;

/// Confidence threshold below which we ask for clarification
const CLARIFICATION_THRESHOLD: f32 = 0.7;

/// Deep understanding of a user request using chain-of-thought reasoning
/// This makes Anna think through the request before acting
pub async fn understand_request(
    model: &str,
    question: &str,
    session_context: Option<&str>,
) -> Result<DeepUnderstanding> {
    let context_section = session_context
        .filter(|c| !c.is_empty())
        .map(|c| format!("\nPrevious context:\n{}", c))
        .unwrap_or_default();

    // Chain-of-thought prompt that makes the LLM think step by step
    let prompt = format!(
        r#"Think through this user request step by step before responding.

User request: "{question}"
{context}
THINK STEP BY STEP:

1. PARAPHRASE: What is the user actually asking for? Restate in your own words.

2. REQUIRED INFO: What information do you need to answer this? List specific things.

3. MISSING INFO: Is anything critical missing from the request? (which service? which file? which package?)
   - If they say "the service" but didn't specify which one = missing
   - If they say "install it" but didn't say what = missing
   - If they reference "that error" without showing it = missing

4. AMBIGUITY CHECK: Could this request mean multiple different things?
   - "check the log" = which log? system log? application log?
   - "enable bluetooth" = just enable? or also pair a device?

5. CONFIDENCE: How confident are you that you understand exactly what they want? (0.0-1.0)
   - 0.9+ = Crystal clear, no ambiguity
   - 0.7-0.9 = Mostly clear, minor assumptions needed
   - 0.5-0.7 = Somewhat unclear, should probably ask
   - <0.5 = Too vague, must ask for clarification

6. CATEGORY: FACTUAL / HOWTO / TROUBLESHOOT / MULTI / UNCLEAR

7. ENTITIES: List any packages, services, files, commands mentioned

8. TOPIC: network / audio / storage / boot / packages / services / security / performance / display / null

Now output your analysis as JSON:
{{"interpreted_as":"what you think they're asking","required_info":["item1","item2"],"missing_info":["missing1"],"ambiguities":["possible interpretation 1","possible interpretation 2"],"confidence":0.85,"category":"FACTUAL","entities":["entity1"],"topic":"topic_or_null","sub_questions":null,"clarification_needed":"question to ask if unclear","needs_confirmation":false}}"#,
        question = question,
        context = context_section
    );

    debug!("Understanding prompt: {} chars", prompt.len());

    let response = ollama::chat_with_timeout(model, &prompt, UNDERSTAND_TIMEOUT_SECS).await?;

    debug!("Understanding response: {}", response.trim());

    parse_understanding_response(&response, question)
}

/// Parse the LLM's chain-of-thought response into DeepUnderstanding
fn parse_understanding_response(response: &str, original_question: &str) -> Result<DeepUnderstanding> {
    // Try to extract JSON from response (handle markdown code blocks and reasoning text)
    let json_str = extract_json_from_response(response);

    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_str) {
        let category = match parsed.get("category").and_then(|c| c.as_str()) {
            Some("FACTUAL") => IntentCategory::Factual,
            Some("HOWTO") => IntentCategory::HowTo,
            Some("TROUBLESHOOT") => IntentCategory::Troubleshoot,
            Some("MULTI") => IntentCategory::Multi,
            Some("UNCLEAR") => IntentCategory::Unclear,
            _ => IntentCategory::Factual,
        };

        let confidence = parsed
            .get("confidence")
            .and_then(|c| c.as_f64())
            .map(|c| c as f32)
            .unwrap_or(0.5);

        let interpreted_as = parsed
            .get("interpreted_as")
            .and_then(|s| s.as_str())
            .unwrap_or(original_question)
            .to_string();

        let required_info = extract_string_array(&parsed, "required_info");
        let missing_info = extract_string_array(&parsed, "missing_info");
        let ambiguities = extract_string_array(&parsed, "ambiguities");
        let entities = extract_string_array(&parsed, "entities");

        let topic = parsed
            .get("topic")
            .and_then(|t| t.as_str())
            .filter(|t| !t.is_empty() && *t != "null")
            .map(String::from);

        let sub_questions = parsed
            .get("sub_questions")
            .and_then(|s| s.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect());

        let clarification_needed = parsed
            .get("clarification_needed")
            .and_then(|c| c.as_str())
            .filter(|s| !s.is_empty() && *s != "null")
            .map(String::from);

        // Determine if confirmation is needed
        let needs_confirmation = should_ask_confirmation(
            confidence,
            &missing_info,
            &ambiguities,
            &category,
            original_question,
        );

        return Ok(DeepUnderstanding {
            interpreted_as,
            required_info,
            missing_info,
            ambiguities,
            confidence,
            category,
            entities,
            topic,
            sub_questions,
            clarification_needed,
            needs_confirmation,
        });
    }

    // Fallback if JSON parsing fails
    warn!("Failed to parse understanding JSON, using fallback");
    Ok(fallback_understanding(original_question))
}

/// Extract JSON from a response that may contain reasoning text
fn extract_json_from_response(response: &str) -> String {
    let response = response.trim();

    // Try to find JSON object in the response
    if let Some(start) = response.find('{') {
        if let Some(end) = response.rfind('}') {
            if end > start {
                return response[start..=end].to_string();
            }
        }
    }

    // Try handling markdown code blocks
    response
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim()
        .to_string()
}

/// Extract a string array from JSON
fn extract_string_array(parsed: &serde_json::Value, key: &str) -> Vec<String> {
    parsed
        .get(key)
        .and_then(|arr| arr.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default()
}

/// Determine if Anna should ask for confirmation before proceeding
fn should_ask_confirmation(
    confidence: f32,
    missing_info: &[String],
    ambiguities: &[String],
    category: &IntentCategory,
    question: &str,
) -> bool {
    let q_lower = question.to_lowercase();

    // v0.0.890: ALWAYS check destructive patterns FIRST, regardless of category or confidence
    // This prevents "how do I format my disk?" from bypassing safety checks
    if is_semantically_destructive(&q_lower) {
        info!("Potentially destructive action detected, will confirm");
        return true;
    }

    // FACTUAL questions with decent confidence - just answer, don't ask
    // Users asking "what is X?" want an answer, not more questions
    if matches!(category, IntentCategory::Factual) && confidence >= 0.6 {
        return false;
    }

    // v0.0.894: HOWTO questions about common operations - don't ask for OS/tool clarification
    // We already know it's Arch Linux from system context
    if matches!(category, IntentCategory::HowTo) {
        // Filter out things we already know from system context
        let relevant_missing: Vec<&String> = missing_info.iter()
            .filter(|m| {
                let m_lower = m.to_lowercase();
                // We already know the OS is Arch/CachyOS - don't ask
                !m_lower.contains("operating") && !m_lower.contains("os") &&
                !m_lower.contains("distro") && !m_lower.contains("distribution") &&
                // We know the package manager is pacman
                !m_lower.contains("package manager") && !m_lower.contains("update_tool") &&
                // Generic "method" questions - just use the default Arch way
                !m_lower.contains("method") && !m_lower.contains("tool_or")
            })
            .collect();

        // If no relevant missing info remains, don't ask for clarification
        if relevant_missing.is_empty() && confidence >= 0.5 {
            debug!("HOWTO question - filtered out known context, proceeding without clarification");
            return false;
        }
    }

    // Very low confidence = definitely ask for clarification
    if confidence < 0.5 {
        info!("Confidence {:.0}% very low, will ask for clarification", confidence * 100.0);
        return true;
    }

    // Missing critical info ONLY if confidence is also low
    // High confidence + missing info means the LLM is being pedantic
    if !missing_info.is_empty() && confidence < CLARIFICATION_THRESHOLD {
        info!("Missing info with low confidence: {:?}", missing_info);
        return true;
    }

    // Multiple valid interpretations ONLY with low confidence
    if ambiguities.len() > 2 && confidence < 0.75 {
        info!("Multiple interpretations with low confidence: {:?}", ambiguities);
        return true;
    }

    // TROUBLESHOOT with vague description AND low confidence
    if matches!(category, IntentCategory::Troubleshoot)
        && question.split_whitespace().count() < 4
        && confidence < CLARIFICATION_THRESHOLD {
        info!("Short troubleshoot question with low confidence, will ask for more details");
        return true;
    }

    false
}

/// Check if a question is semantically destructive (v0.0.890)
/// Uses multiple detection strategies beyond simple keyword matching
fn is_semantically_destructive(question: &str) -> bool {
    // Direct destructive keywords
    let direct_destructive = [
        "delete", "remove", "uninstall", "wipe", "format", "reset",
        "overwrite", "replace", "drop", "purge", "clean", "erase",
        "destroy", "clear", "truncate", "shred",
    ];
    if direct_destructive.iter().any(|p| question.contains(p)) {
        return true;
    }

    // Semantic patterns that imply destruction even without direct keywords
    let semantic_patterns = [
        // Disk/storage operations
        ("partition", "create"),    // creating partitions destroys data
        ("partition", "resize"),
        ("disk", "prepare"),
        ("drive", "initialize"),
        ("filesystem", "create"),
        ("mkfs", ""),               // direct command reference
        ("fdisk", ""),
        ("gdisk", ""),
        ("parted", ""),
        // System modification
        ("factory", "reset"),
        ("fresh", "install"),
        ("reinstall", ""),
        ("downgrade", ""),
        // Permission/ownership changes
        ("chmod", "recursive"),
        ("chown", "recursive"),
        // Service/daemon control
        ("disable", "service"),
        ("stop", "all"),
        ("kill", "process"),
        // Package management destructive ops
        ("pacman", "-Rns"),
        ("pacman", "-Rdd"),
        ("orphan", "remove"),
    ];

    for (pattern1, pattern2) in &semantic_patterns {
        if question.contains(pattern1) {
            if pattern2.is_empty() || question.contains(pattern2) {
                return true;
            }
        }
    }

    // Check for dangerous target paths in questions
    let dangerous_targets = [
        "all files", "everything", "entire", "whole disk", "root",
        "/home", "/etc", "/var", "/usr", "/boot", "system",
    ];
    let action_words = ["from", "on", "in", "at"];

    for target in &dangerous_targets {
        if question.contains(target) {
            // Check if there's an action word suggesting modification
            if action_words.iter().any(|a| question.contains(a)) {
                return true;
            }
        }
    }

    false
}

/// Fallback understanding using keyword analysis
pub fn fallback_understanding(question: &str) -> DeepUnderstanding {
    let fallback = fallback_classification(question);

    DeepUnderstanding {
        interpreted_as: question.to_string(),
        required_info: vec![],
        missing_info: vec![],
        ambiguities: vec![],
        confidence: fallback.confidence,
        category: fallback.category,
        entities: fallback.entities,
        topic: fallback.topic,
        sub_questions: fallback.sub_questions,
        clarification_needed: fallback.clarification,
        needs_confirmation: fallback.confidence < CLARIFICATION_THRESHOLD,
    }
}

/// Legacy classify_intent for backward compatibility
pub async fn classify_intent(
    model: &str,
    question: &str,
    session_context: Option<&str>,
) -> Result<IntentClassification> {
    // Use the new understanding system and convert to legacy format
    let understanding = understand_request(model, question, session_context).await?;

    Ok(IntentClassification {
        category: understanding.category,
        confidence: understanding.confidence,
        sub_questions: understanding.sub_questions,
        clarification: understanding.clarification_needed,
        entities: understanding.entities,
        topic: understanding.topic,
    })
}

/// v0.0.894: Check if a question is off-topic (not related to Linux/system administration)
/// Returns Some(response) if off-topic, None if it's a valid system question
pub fn detect_off_topic(question: &str) -> Option<String> {
    let q = question.to_lowercase();

    // Off-topic patterns - questions that have nothing to do with Linux
    let off_topic_patterns = [
        // Philosophy/existential
        ("meaning of life", "That's a profound question, but I'm specialized in Arch Linux system administration. Try asking me about your system instead!"),
        ("purpose of life", "Philosophy is beyond my expertise - I'm here for Linux questions!"),
        // Cooking
        ("cook", "I'm not a chef, but I can help you configure your system!"),
        ("recipe", "I only have recipes for Linux commands, not food!"),
        ("spaghetti", "I can't help with cooking, but ask me anything about Arch Linux!"),
        // Entertainment
        ("write me a poem", "I'm an IT assistant, not a poet. How about a system check instead?"),
        ("tell me a joke", "My jokes are all about segfaults. Ask me something technical!"),
        ("play music", "I can help you configure PipeWire, but I can't play music myself."),
        // General knowledge unrelated to systems
        ("capital of", "I'm specialized in Linux, not geography!"),
        ("world cup", "I track system metrics, not sports scores!"),
        ("weather", "I monitor system temperature, not the weather outside!"),
        ("stock", "I handle system processes, not financial ones!"),
        ("train my dog", "I can only train neural networks and configure services!"),
        // Meta questions
        ("are you sentient", "I'm Anna, an Arch Linux assistant. Sentience is above my pay grade!"),
        ("favorite color", "My favorite color is whatever your terminal theme is set to!"),
        ("how old are you", "I'm as old as my last deployment. Ask me about your system!"),
    ];

    for (pattern, response) in &off_topic_patterns {
        if q.contains(pattern) {
            return Some(response.to_string());
        }
    }

    // Generic off-topic detection: questions with no technical keywords at all
    let has_tech_keywords = [
        "install", "update", "package", "service", "file", "disk", "network",
        "kernel", "boot", "driver", "cpu", "ram", "memory", "gpu", "audio",
        "bluetooth", "wifi", "ssh", "sudo", "permission", "user", "group",
        "systemd", "pacman", "aur", "config", "log", "error", "fail",
        "mount", "partition", "grub", "systemd-boot", "firewall", "port",
        "process", "kill", "running", "status", "version", "arch", "linux",
    ].iter().any(|kw| q.contains(kw));

    if !has_tech_keywords && q.len() > 20 {
        // Long question with no technical keywords - probably off-topic
        return Some("I'm specialized in Arch Linux system administration. Could you rephrase your question in terms of system configuration, troubleshooting, or Linux commands?".to_string());
    }

    None
}

/// Fallback classification using keywords (when LLM response is malformed)
pub fn fallback_classification(question: &str) -> IntentClassification {
    let q = question.to_lowercase();

    // Check for multi-question patterns first
    let has_and = q.contains(" and ") || q.contains(" also ");
    let has_multiple_questions = q.matches('?').count() > 1;
    if has_and && (q.contains("what") || q.contains("how")) || has_multiple_questions {
        return IntentClassification {
            category: IntentCategory::Multi,
            confidence: 0.4,
            sub_questions: None, // Can't reliably extract without LLM
            clarification: None,
            entities: vec![],
            topic: None,
        };
    }

    // Check for unclear/vague patterns
    let vague_patterns = ["fix it", "help me", "the thing", "that stuff", "do it"];
    if vague_patterns.iter().any(|p| q.contains(p)) || q.split_whitespace().count() <= 2 {
        return IntentClassification {
            category: IntentCategory::Unclear,
            confidence: 0.5,
            sub_questions: None,
            clarification: Some("Could you please be more specific about what you're asking?".into()),
            entities: vec![],
            topic: None,
        };
    }

    // Check for troubleshooting patterns
    let troubleshoot_patterns = [
        "not working", "doesn't work", "error", "fail", "broken",
        "why is", "why does", "why can't", "fix", "problem", "issue",
    ];
    if troubleshoot_patterns.iter().any(|p| q.contains(p)) {
        return IntentClassification {
            category: IntentCategory::Troubleshoot,
            confidence: 0.5,
            sub_questions: None,
            clarification: None,
            entities: vec![],
            topic: None,
        };
    }

    // Check for how-to patterns
    let howto_patterns = [
        "how do i", "how can i", "how to", "how should i",
        "install", "configure", "setup", "set up", "enable", "disable",
    ];
    if howto_patterns.iter().any(|p| q.contains(p)) {
        return IntentClassification {
            category: IntentCategory::HowTo,
            confidence: 0.5,
            sub_questions: None,
            clarification: None,
            entities: vec![],
            topic: None,
        };
    }

    // Default to factual
    IntentClassification {
        category: IntentCategory::Factual,
        confidence: 0.5,
        sub_questions: None,
        clarification: None,
        entities: vec![],
        topic: None,
    }
}

/// Format intent result for display
pub fn format_intent_result(intent: &IntentClassification) -> String {
    let category_str = match intent.category {
        IntentCategory::Factual => "FACTUAL",
        IntentCategory::HowTo => "HOWTO",
        IntentCategory::Troubleshoot => "TROUBLESHOOT",
        IntentCategory::Multi => "MULTI",
        IntentCategory::Unclear => "UNCLEAR",
    };

    let mut result = format!("{} ({:.0}%)", category_str, intent.confidence * 100.0);

    if let Some(ref topic) = intent.topic {
        result.push_str(&format!(" [{}]", topic));
    }

    if !intent.entities.is_empty() {
        result.push_str(&format!(" entities: {}", intent.entities.join(", ")));
    }

    result
}

/// Format deep understanding result for display
pub fn format_understanding_result(understanding: &DeepUnderstanding) -> String {
    let category_str = match understanding.category {
        IntentCategory::Factual => "FACTUAL",
        IntentCategory::HowTo => "HOWTO",
        IntentCategory::Troubleshoot => "TROUBLESHOOT",
        IntentCategory::Multi => "MULTI",
        IntentCategory::Unclear => "UNCLEAR",
    };

    let mut result = format!("{} ({:.0}%)", category_str, understanding.confidence * 100.0);

    if let Some(ref topic) = understanding.topic {
        result.push_str(&format!(" [{}]", topic));
    }

    if !understanding.entities.is_empty() {
        result.push_str(&format!(" | entities: {}", understanding.entities.join(", ")));
    }

    if understanding.needs_confirmation {
        result.push_str(" | NEEDS CONFIRMATION");
    }

    if !understanding.missing_info.is_empty() {
        result.push_str(&format!(" | missing: {}", understanding.missing_info.join(", ")));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_factual() {
        let result = fallback_classification("what is my kernel version?");
        assert_eq!(result.category, IntentCategory::Factual);
    }

    #[test]
    fn test_fallback_howto() {
        let result = fallback_classification("how do I install neovim?");
        assert_eq!(result.category, IntentCategory::HowTo);
    }

    #[test]
    fn test_fallback_troubleshoot() {
        let result = fallback_classification("wifi is not working");
        assert_eq!(result.category, IntentCategory::Troubleshoot);
    }

    #[test]
    fn test_fallback_unclear() {
        let result = fallback_classification("fix it");
        assert_eq!(result.category, IntentCategory::Unclear);
    }

    #[test]
    fn test_parse_understanding_json() {
        let json = r#"{"interpreted_as":"User wants to install neovim","required_info":["package manager"],"missing_info":[],"ambiguities":[],"confidence":0.95,"category":"HOWTO","entities":["neovim"],"topic":"packages","sub_questions":null,"clarification_needed":null,"needs_confirmation":false}"#;
        let result = parse_understanding_response(json, "install neovim").unwrap();
        assert_eq!(result.category, IntentCategory::HowTo);
        assert!(result.confidence > 0.9);
        assert_eq!(result.entities, vec!["neovim"]);
        assert_eq!(result.topic, Some("packages".into()));
        assert!(!result.needs_confirmation);
    }

    #[test]
    fn test_needs_confirmation_very_low_confidence() {
        // Very low confidence (< 0.5) should trigger confirmation
        let result = should_ask_confirmation(0.4, &[], &[], &IntentCategory::HowTo, "do something");
        assert!(result);
    }

    #[test]
    fn test_needs_confirmation_missing_info_with_low_confidence() {
        // Missing info only triggers confirmation if confidence is also low
        let missing = vec!["which service".to_string()];
        let result = should_ask_confirmation(0.6, &missing, &[], &IntentCategory::HowTo, "enable the service");
        assert!(result); // Missing info with low confidence should trigger confirmation
    }

    #[test]
    fn test_no_confirmation_missing_info_high_confidence() {
        // High confidence with missing info should NOT trigger confirmation (LLM is being pedantic)
        let missing = vec!["which service".to_string()];
        let result = should_ask_confirmation(0.9, &missing, &[], &IntentCategory::HowTo, "enable the service");
        assert!(!result);
    }

    #[test]
    fn test_factual_no_confirmation() {
        // FACTUAL questions with decent confidence should never ask for confirmation
        let result = should_ask_confirmation(0.6, &[], &[], &IntentCategory::Factual, "what is X?");
        assert!(!result);
    }

    #[test]
    fn test_no_confirmation_high_confidence() {
        let result = should_ask_confirmation(0.95, &[], &[], &IntentCategory::Factual, "what is my kernel version?");
        assert!(!result); // High confidence, no missing info = no confirmation needed
    }

    #[test]
    fn test_extract_json_from_response() {
        // Test with reasoning text before JSON
        let response = "Let me think about this...\n\n{\"category\":\"FACTUAL\",\"confidence\":0.9}";
        let json = extract_json_from_response(response);
        assert!(json.contains("FACTUAL"));

        // Test with markdown code block
        let response = "```json\n{\"category\":\"HOWTO\"}\n```";
        let json = extract_json_from_response(response);
        assert!(json.contains("HOWTO"));
    }
}
