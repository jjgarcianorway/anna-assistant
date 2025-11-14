//! Intent Router - Natural language to intent mapping
//!
//! Phase 5.1: Conversational UX
//! Maps user's natural language input to one of Anna's internal intents

use anyhow::Result;

/// User intent parsed from natural language
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Intent {
    /// Check Anna's own health
    AnnaStatus,
    /// Generate system report
    Report,
    /// Show prioritized suggestions
    Suggest,
    /// Execute or explain a repair
    Repair { action_id: Option<String> },
    /// Ignore a suggestion
    Discard { suggestion_key: Option<String> },
    /// Adjust autonomy level
    Autonomy { level: Option<String> },
    /// Apply/execute a suggested fix
    Apply { suggestion_key: Option<String> },
    /// Explain privacy/data handling
    Privacy,
    /// Adjust personality (humor/verbosity)
    Personality { adjustment: PersonalityAdjustment },
    /// Change language
    Language { language: Option<String> },
    /// Show help/examples
    Help,
    /// Exit the REPL
    Exit,
    /// Off-topic query (not sysadmin related)
    OffTopic,
    /// Unclear intent
    Unclear,
}

/// Personality adjustment types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersonalityAdjustment {
    IncreaseHumor,
    DecreaseHumor,
    MoreBrief,
    MoreDetailed,
    Show, // Show current settings
}

/// Route natural language input to intent
pub fn route_intent(input: &str) -> Intent {
    let lower = input.to_lowercase();
    // Strip common punctuation from words
    let cleaned = lower
        .replace("?", " ")
        .replace("!", " ")
        .replace(",", " ")
        .replace(".", " ");
    let words: Vec<&str> = cleaned.split_whitespace().collect();

    // Exit intents
    if matches!(lower.trim(), "exit" | "quit" | "bye" | "goodbye") {
        return Intent::Exit;
    }

    // Anna status (self-health check)
    // Check this early to avoid "how" triggering Help
    if contains_any(&words, &["status", "health", "ok", "working", "alive"])
        && contains_any(&words, &["anna", "you", "your"]) {
        return Intent::AnnaStatus;
    }

    // Also catch "how are you" as status check (but not greetings like "hello how are you")
    if (lower.contains("how are you") || lower.contains("how's it going"))
        && (lower.contains("anna") || words.len() <= 5)
        && !contains_any(&words, &["hello", "hi", "hey"]) {
        return Intent::AnnaStatus;
    }

    // Privacy explanation
    if contains_any(&words, &["privacy", "store", "data", "telemetry", "tracking"]) {
        return Intent::Privacy;
    }

    // Report generation
    if contains_any(&words, &["report", "summary", "boss", "document", "overview"]) {
        return Intent::Report;
    }

    // Apply/Execute - User wants to apply a suggestion
    if (contains_any(&words, &["apply", "do", "execute", "run"])
        && (contains_any(&words, &["that", "it", "this", "suggestion", "fix"])
            || lower.contains("go ahead")
            || lower.contains("proceed")))
        || lower.contains("yes apply")
        || lower.contains("yes do it")
        || (contains_any(&words, &["fix"]) && contains_any(&words, &["that", "it", "now"])) {
        return Intent::Apply { suggestion_key: None };
    }

    // Repair (check before Suggest to prioritize "fix" over "problem")
    if contains_any(&words, &["fix", "repair"])
        && !contains_any(&words, &["how", "what", "should", "that", "it", "this"]) {
        // Try to extract action ID if present
        // For now, just return repair intent without specific action
        return Intent::Repair { action_id: None };
    }

    // Suggestions
    if contains_any(&words, &["suggest", "suggestion", "suggestions", "recommend", "should", "improve", "better"])
        || (contains_any(&words, &["what", "how"]) && contains_any(&words, &["fix", "improve"])) {
        return Intent::Suggest;
    }

    // Problems/issues (also suggest)
    if contains_any(&words, &["slow", "slower", "problem", "issue", "wrong", "broken"]) {
        return Intent::Suggest;
    }

    // Discard/ignore
    if contains_any(&words, &["ignore", "discard", "dismiss", "hide", "skip"]) {
        return Intent::Discard { suggestion_key: None };
    }

    // Autonomy
    if contains_any(&words, &["autonomy", "automatic", "auto", "control", "permission"]) {
        return Intent::Autonomy { level: None };
    }

    // Language change
    // Detect patterns like:
    // - "use English"
    // - "cambia al español"
    // - "speak Norwegian"
    // - "Anna, use Spanish from now on"
    if contains_any(&words, &["language", "idioma", "språk", "sprache", "langue", "língua"])
        || (contains_any(&words, &["use", "speak", "talk", "habla", "parle", "fala", "snakk"])
            && contains_any(&words, &["english", "spanish", "español", "norwegian", "norsk", "german", "deutsch", "french", "français", "portuguese", "português"]))
        || (contains_any(&words, &["cambia", "change", "switch"])
            && contains_any(&words, &["al", "to", "til", "zu", "à", "para"]))
    {
        // Try to extract the language
        let lang = extract_language(&words, &lower);
        return Intent::Language { language: lang };
    }

    // Personality adjustments
    // Check for "show personality" first
    if contains_any(&words, &["show", "display", "current", "what"])
        && contains_any(&words, &["personality", "settings", "preferences"]) {
        return Intent::Personality { adjustment: PersonalityAdjustment::Show };
    }

    // Personality humor (but not "tell me a joke" which is off-topic)
    if contains_any(&words, &["humor", "humour", "ironic", "playful", "serious"])
        || (contains_any(&words, &["joke", "funny"])
            && contains_any(&words, &["be", "more", "less", "not", "dont", "don't", "stop"])) {
        if contains_any(&words, &["not", "dont", "don't", "stop", "no", "less", "serious"]) {
            return Intent::Personality { adjustment: PersonalityAdjustment::DecreaseHumor };
        } else if contains_any(&words, &["more", "be", "increase"]) {
            return Intent::Personality { adjustment: PersonalityAdjustment::IncreaseHumor };
        }
    }

    if contains_any(&words, &["brief", "concise", "short", "shorter"])
        && (contains_any(&words, &["answer", "response", "talk", "be"])
            || (contains_any(&words, &["more", "be"]) && lower.contains("brief"))) {
        return Intent::Personality { adjustment: PersonalityAdjustment::MoreBrief };
    }

    if (contains_any(&words, &["detailed", "verbose"]) || (contains_any(&words, &["more"]) && contains_any(&words, &["detail"])))
        && (contains_any(&words, &["answer", "response", "talk", "be", "explain"])
            || lower.contains("more detail")) {
        return Intent::Personality { adjustment: PersonalityAdjustment::MoreDetailed };
    }

    // Off-topic detection (weather, jokes, general chitchat) - check before Help
    if contains_any(&words, &["weather", "joke", "funny", "hello", "hi", "good", "morning"]) {
        return Intent::OffTopic;
    }

    // Help
    if contains_any(&words, &["help", "how", "what", "example", "command"]) {
        return Intent::Help;
    }

    // Default: unclear intent
    Intent::Unclear
}

/// Check if words contains any of the targets
fn contains_any(words: &[&str], targets: &[&str]) -> bool {
    words.iter().any(|w| targets.contains(w))
}

/// Extract language from words
fn extract_language(words: &[&str], lower: &str) -> Option<String> {
    // Check for language names in various forms
    let language_keywords = [
        ("english", "english"),
        ("en", "english"),
        ("spanish", "spanish"),
        ("español", "spanish"),
        ("es", "spanish"),
        ("norwegian", "norwegian"),
        ("norsk", "norwegian"),
        ("no", "norwegian"),
        ("nb", "norwegian"),
        ("german", "german"),
        ("deutsch", "german"),
        ("de", "german"),
        ("french", "french"),
        ("français", "french"),
        ("fr", "french"),
        ("portuguese", "portuguese"),
        ("português", "portuguese"),
        ("pt", "portuguese"),
    ];

    for (keyword, lang) in &language_keywords {
        if words.contains(keyword) || lower.contains(keyword) {
            return Some(lang.to_string());
        }
    }

    None
}

/// Generate helpful response for unclear intent
pub fn unclear_response() -> String {
    "I'm not quite sure what you're asking. I focus on system administration \
     for this machine.\n\n\
     You can ask me about:\n\
     • System status and health\n\
     • Problems or suggestions for improvement\n\
     • Generating a system report\n\
     • Fixing specific issues\n\
     • Privacy and data handling\n\n\
     Try rephrasing your question, or type 'help' for examples.".to_string()
}

/// Generate response for off-topic queries
pub fn offtopic_response() -> String {
    "I appreciate the chat, but I focus only on this machine's health \
     and administration. I'm here to help with:\n\
     • Hardware and software issues\n\
     • System configuration\n\
     • Performance and security\n\
     • Desktop workflows\n\n\
     Ask me something about your Arch system!".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anna_status_intent() {
        assert_eq!(route_intent("Anna, are you ok?"), Intent::AnnaStatus);
        assert_eq!(route_intent("How are you anna"), Intent::AnnaStatus);
        assert_eq!(route_intent("Is Anna working?"), Intent::AnnaStatus);
    }

    #[test]
    fn test_report_intent() {
        assert_eq!(route_intent("Generate a report"), Intent::Report);
        assert_eq!(route_intent("I need a summary for my boss"), Intent::Report);
        assert_eq!(route_intent("System overview"), Intent::Report);
    }

    #[test]
    fn test_suggest_intent() {
        assert_eq!(route_intent("What should I improve?"), Intent::Suggest);
        assert_eq!(route_intent("Any suggestions?"), Intent::Suggest);
        assert_eq!(route_intent("My system feels slow"), Intent::Suggest);
        assert_eq!(route_intent("Something is wrong"), Intent::Suggest);
    }

    #[test]
    fn test_privacy_intent() {
        assert_eq!(route_intent("What do you store about me?"), Intent::Privacy);
        assert_eq!(route_intent("Privacy concerns"), Intent::Privacy);
        assert_eq!(route_intent("Tell me about data tracking"), Intent::Privacy);
    }

    #[test]
    fn test_repair_intent() {
        assert_eq!(route_intent("Fix this problem"), Intent::Repair { action_id: None });
        assert_eq!(route_intent("Apply the repair"), Intent::Repair { action_id: None });
    }

    #[test]
    fn test_exit_intent() {
        assert_eq!(route_intent("exit"), Intent::Exit);
        assert_eq!(route_intent("quit"), Intent::Exit);
        assert_eq!(route_intent("goodbye"), Intent::Exit);
    }

    #[test]
    fn test_offtopic_intent() {
        assert_eq!(route_intent("What's the weather?"), Intent::OffTopic);
        assert_eq!(route_intent("Tell me a joke"), Intent::OffTopic);
        assert_eq!(route_intent("Hello how are you"), Intent::OffTopic);
    }

    #[test]
    fn test_unclear_intent() {
        assert_eq!(route_intent("asdfasdf"), Intent::Unclear);
        assert_eq!(route_intent("random words here"), Intent::Unclear);
    }

    #[test]
    fn test_personality_intents() {
        assert_eq!(
            route_intent("show personality settings"),
            Intent::Personality { adjustment: PersonalityAdjustment::Show }
        );
        assert_eq!(
            route_intent("Anna, be more funny"),
            Intent::Personality { adjustment: PersonalityAdjustment::IncreaseHumor }
        );
        assert_eq!(
            route_intent("Don't joke around"),
            Intent::Personality { adjustment: PersonalityAdjustment::DecreaseHumor }
        );
        assert_eq!(
            route_intent("Be more brief"),
            Intent::Personality { adjustment: PersonalityAdjustment::MoreBrief }
        );
        assert_eq!(
            route_intent("Explain in more detail"),
            Intent::Personality { adjustment: PersonalityAdjustment::MoreDetailed }
        );
    }
}
