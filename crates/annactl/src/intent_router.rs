//! Intent Router - Natural language to intent mapping
//!
//! Phase 5.1: Conversational UX
//! Maps user's natural language input to one of Anna's internal intents

/// User intent parsed from natural language
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Intent {
    /// Show system status (more detailed than AnnaStatus)
    SystemStatus,
    /// Summarize historian data
    HistorianSummary,
    /// Improvement suggestions
    Improve,
    /// Check Anna's own health
    AnnaStatus,
    /// Repair Anna's own components (daemon, etc.)
    AnnaSelfRepair,
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
    /// Set up or reconfigure LLM brain
    SetupBrain,
    /// Show help/examples
    Help,
    /// Exit the REPL
    Exit,
    /// Off-topic query (not sysadmin related)
    OffTopic,
    /// Unclear intent (Task 12: route to LLM)
    Unclear(String),
}

/// Personality adjustment types (Beta.89: Full 16-trait control)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersonalityAdjustment {
    /// Show all 16 personality traits
    Show,
    /// Set a specific trait to a value (0-10)
    SetTrait { trait_key: String, value: u8 },
    /// Adjust by descriptor ("be more concise", "be warmer", etc.)
    AdjustByDescriptor { descriptor: String },
    /// Reset all traits to defaults
    Reset,
    /// Validate configuration for conflicts
    Validate,
    // Legacy adjustments (kept for backwards compatibility)
    IncreaseHumor,
    DecreaseHumor,
    MoreBrief,
    MoreDetailed,
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

    // Exit intents (multilingual)
    // EN: exit, quit, bye, goodbye
    // ES: salir, adiós, chao
    // NO: avslutt, hade
    // DE: beenden, tschüss
    // FR: quitter, au revoir
    // PT: sair, tchau
    if matches!(
        lower.trim(),
        "exit"
            | "quit"
            | "bye"
            | "goodbye"
            | "salir"
            | "adiós"
            | "adios"
            | "chao"
            | "avslutt"
            | "hade"
            | "beenden"
            | "tschüss"
            | "tschuess"
            | "quitter"
            | "au revoir"
            | "sair"
            | "tchau"
    ) {
        return Intent::Exit;
    }

    // Anna status (self-health check) - multilingual
    // EN: status, health, ok, working, alive, how are you
    // ES: estado, salud, cómo estás
    // NO: status, helse, hvordan har du det
    // DE: status, gesundheit, wie geht's
    // FR: statut, santé, comment vas-tu
    // PT: estado, saúde, como está
    if contains_any(
        &words,
        &[
            "status",
            "health",
            "ok",
            "working",
            "alive",
            "estado",
            "salud",
            "helse",
            "gesundheit",
            "santé",
            "saúde",
        ],
    ) && contains_any(&words, &["anna", "you", "your", "tu", "du", "você"])
    {
        return Intent::AnnaStatus;
    }

    // Also catch "how are you" variants in multiple languages
    if (lower.contains("how are you")
        || lower.contains("how's it going")
        || lower.contains("cómo estás")
        || lower.contains("como estas")
        || lower.contains("hvordan har du det")
        || lower.contains("wie geht's")
        || lower.contains("wie gehts")
        || lower.contains("comment vas-tu")
        || lower.contains("ça va")
        || lower.contains("como está")
        || lower.contains("tudo bem"))
        && (lower.contains("anna") || words.len() <= 5)
        && !contains_any(
            &words,
            &["hello", "hi", "hey", "hola", "hei", "bonjour", "olá"],
        )
    {
        return Intent::AnnaStatus;
    }

    // System status (system health)
    if lower.contains("system status")
        || lower.contains("system health")
        || (contains_any(&words, &["status", "health"])
            && contains_any(&words, &["system", "machine", "computer", "pc", "server"]))
    {
        return Intent::SystemStatus;
    }

    // Improvement suggestions
    if lower.contains("what should i improve")
        || (lower.contains("improve") && !lower.contains("suggest"))
        || lower.contains("mejorar")
        || lower.contains("what should i do")
    {
        return Intent::Improve;
    }

    // Anna self-repair (fix yourself, repair anna, etc.) - multilingual
    if (contains_any(
        &words,
        &[
            "fix",
            "repair",
            "heal",
            "reparar",
            "arreglar",
            "réparer",
            "consertar",
        ],
    ) && (contains_any(
        &words,
        &[
            "yourself",
            "anna",
            "self",
            "ti misma",
            "deg selv",
            "dich selbst",
        ],
    ) || lower.contains("your own")
        || lower.contains("auto repair")))
        || lower.contains("fix yourself")
        || lower.contains("repair yourself")
        || lower.contains("repárate")
        || lower.contains("check your own service")
        || lower.contains("repair anna")
        || lower.contains("fix anna")
    {
        return Intent::AnnaSelfRepair;
    }

    // Privacy explanation - multilingual
    if contains_any(
        &words,
        &[
            "privacy",
            "store",
            "data",
            "telemetry",
            "tracking",
            "privacidad",
            "datos",
            "personvern",
            "lagring",
            "datenschutz",
            "daten",
            "vie privée",
            "données",
            "privacidade",
            "armazenamento",
        ],
    ) {
        return Intent::Privacy;
    }

    // Report generation - multilingual
    // EN: report, summary, document
    // ES: informe, reporte, resumen
    // NO: rapport, sammendrag
    // DE: bericht, zusammenfassung
    // FR: rapport, résumé
    // PT: relatório, resumo
    if contains_any(
        &words,
        &[
            "report",
            "summary",
            "boss",
            "document",
            "overview",
            "informe",
            "reporte",
            "resumen",
            "rapport",
            "sammendrag",
            "bericht",
            "zusammenfassung",
            "résumé",
            "relatório",
            "resumo",
        ],
    ) {
        return Intent::Report;
    }

    // Historian summary intent
    if lower.contains("historian")
        || lower.contains("history")
        || lower.contains("what you learned")
        || lower.contains("what have you learned")
    {
        return Intent::HistorianSummary;
    }

    // Apply/Execute - User wants to apply a suggestion
    if (contains_any(&words, &["apply", "do", "execute", "run"])
        && (contains_any(&words, &["that", "it", "this", "suggestion", "fix"])
            || lower.contains("go ahead")
            || lower.contains("proceed")))
        || lower.contains("yes apply")
        || lower.contains("yes do it")
        || (contains_any(&words, &["fix"]) && contains_any(&words, &["that", "it", "now"]))
    {
        return Intent::Apply {
            suggestion_key: None,
        };
    }

    // Repair (check before Suggest to prioritize "fix" over "problem")
    if contains_any(&words, &["fix", "repair"])
        && !contains_any(&words, &["how", "what", "should", "that", "it", "this"])
    {
        // Try to extract action ID if present
        // For now, just return repair intent without specific action
        return Intent::Repair { action_id: None };
    }

    // Suggestions (what should I improve, top suggestions, recommendations)
    if contains_any(
        &words,
        &[
            "suggest",
            "suggestion",
            "suggestions",
            "recommend",
            "recommendations",
            "should",
            "improve",
            "better",
        ],
    ) || (contains_any(&words, &["what", "how"]) && contains_any(&words, &["fix", "improve"]))
        || lower.contains("top suggestions")
        || lower.contains("what should i improve")
        || lower.contains("what are your recommendations")
    {
        return Intent::Suggest;
    }

    // Problems/issues (also suggest)
    if contains_any(
        &words,
        &["slow", "slower", "problem", "issue", "wrong", "broken"],
    ) {
        return Intent::Suggest;
    }

    // Discard/ignore
    if contains_any(&words, &["ignore", "discard", "dismiss", "hide", "skip"]) {
        return Intent::Discard {
            suggestion_key: None,
        };
    }

    // Autonomy
    if contains_any(
        &words,
        &["autonomy", "automatic", "auto", "control", "permission"],
    ) {
        return Intent::Autonomy { level: None };
    }

    // Language change
    // Detect patterns like:
    // - "use English"
    // - "cambia al español"
    // - "speak Norwegian"
    // - "Anna, use Spanish from now on"
    if contains_any(
        &words,
        &["language", "idioma", "språk", "sprache", "langue", "língua"],
    ) || (contains_any(
        &words,
        &["use", "speak", "talk", "habla", "parle", "fala", "snakk"],
    ) && contains_any(
        &words,
        &[
            "english",
            "spanish",
            "español",
            "norwegian",
            "norsk",
            "german",
            "deutsch",
            "french",
            "français",
            "portuguese",
            "português",
        ],
    )) || (contains_any(&words, &["cambia", "change", "switch"])
        && contains_any(&words, &["al", "to", "til", "zu", "à", "para"]))
    {
        // Try to extract the language
        let lang = extract_language(&words, &lower);
        return Intent::Language { language: lang };
    }

    // Beta.89: Enhanced personality adjustments

    // "reset your personality" | "reset personality"
    if contains_any(&words, &["reset"])
        && contains_any(&words, &["personality", "traits", "settings"])
    {
        return Intent::Personality {
            adjustment: PersonalityAdjustment::Reset,
        };
    }

    // "validate your personality" | "check personality"
    if contains_any(&words, &["validate", "check", "verify"])
        && contains_any(&words, &["personality", "traits", "settings", "conflicts"])
    {
        return Intent::Personality {
            adjustment: PersonalityAdjustment::Validate,
        };
    }

    // "set warm_vs_cold to 8" | "set your warm_vs_cold to 8"
    // Pattern: "set <trait_name> to <number>"
    if contains_any(&words, &["set"]) && lower.contains(" to ") {
        // Try to extract trait and value
        if let Some((trait_key, value)) = extract_trait_set(&lower) {
            return Intent::Personality {
                adjustment: PersonalityAdjustment::SetTrait { trait_key, value },
            };
        }
    }

    // "be more concise" | "be warmer" | "be less formal"
    // Pattern: "be [more|less] <descriptor>"
    if contains_any(&words, &["be"])
        && (contains_any(&words, &["more", "less", "warmer", "colder", "formal", "casual",
                                     "concise", "verbose", "friendly", "professional"]))
    {
        // Extract the full descriptor
        let descriptor = lower
            .trim_start_matches("be ")
            .trim_start_matches("be more ")
            .trim_start_matches("be less ")
            .trim();

        if !descriptor.is_empty() {
            return Intent::Personality {
                adjustment: PersonalityAdjustment::AdjustByDescriptor {
                    descriptor: descriptor.to_string()
                },
            };
        }
    }

    // Check for "show personality"
    if contains_any(&words, &["show", "display", "current", "what"])
        && contains_any(&words, &["personality", "settings", "preferences", "traits"])
    {
        return Intent::Personality {
            adjustment: PersonalityAdjustment::Show,
        };
    }

    // Legacy personality humor (backwards compatibility)
    if contains_any(&words, &["humor", "humour", "ironic", "playful", "serious"])
        || (contains_any(&words, &["joke", "funny"])
            && contains_any(
                &words,
                &["be", "more", "less", "not", "dont", "don't", "stop"],
            ))
    {
        if contains_any(
            &words,
            &["not", "dont", "don't", "stop", "no", "less", "serious"],
        ) {
            return Intent::Personality {
                adjustment: PersonalityAdjustment::DecreaseHumor,
            };
        } else if contains_any(&words, &["more", "be", "increase"]) {
            return Intent::Personality {
                adjustment: PersonalityAdjustment::IncreaseHumor,
            };
        }
    }

    // Legacy brief/detailed (backwards compatibility)
    if contains_any(&words, &["brief", "concise", "short", "shorter"])
        && (contains_any(&words, &["answer", "response", "talk", "be"])
            || (contains_any(&words, &["more", "be"]) && lower.contains("brief")))
    {
        return Intent::Personality {
            adjustment: PersonalityAdjustment::MoreBrief,
        };
    }

    if (contains_any(&words, &["detailed", "verbose"])
        || (contains_any(&words, &["more"]) && contains_any(&words, &["detail"])))
        && (contains_any(&words, &["answer", "response", "talk", "be", "explain"])
            || lower.contains("more detail"))
    {
        return Intent::Personality {
            adjustment: PersonalityAdjustment::MoreDetailed,
        };
    }

    // LLM brain setup or reconfiguration
    if (contains_any(&words, &["set", "setup", "configure", "install"])
        && contains_any(&words, &["brain", "llm", "model"]))
        || (lower.contains("set up") && contains_any(&words, &["brain", "llm"]))
        || lower.contains("setup brain")
        || lower.contains("configure brain")
    {
        return Intent::SetupBrain;
    }

    // Off-topic detection (weather, jokes, general chitchat) - check before Help
    if contains_any(
        &words,
        &["weather", "joke", "funny", "hello", "hi", "good", "morning"],
    ) {
        return Intent::OffTopic;
    }

    // Help - ONLY for explicit help requests, everything else goes to LLM (multilingual)
    // EN: help, what can you do
    // ES: ayuda, qué puedes hacer
    // NO: hjelp, hva kan du gjøre
    // DE: hilfe, was kannst du
    // FR: aide, que peux-tu faire
    // PT: ajuda, o que você pode fazer
    if contains_any(
        &words,
        &["help", "ayuda", "hjelp", "hilfe", "aide", "ajuda"],
    ) || lower == "?"
        || lower.contains("show me examples")
        || lower.contains("what can you do")
        || lower.contains("qué puedes hacer")
        || lower.contains("que puedes hacer")
        || lower.contains("hva kan du gjøre")
        || lower.contains("was kannst du")
        || lower.contains("que peux-tu faire")
        || lower.contains("o que você pode fazer")
        || lower.contains("list commands")
        || lower.contains("lista comandos")
        || lower.contains("liste kommandoer")
        || lower.contains("available commands")
        || lower.contains("comandos disponibles")
    {
        return Intent::Help;
    }

    // Default: unclear intent (Task 12: route to LLM with original input)
    Intent::Unclear(input.to_string())
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

/// Extract trait name and value from "set <trait> to <value>" pattern
/// Beta.89: Supports patterns like "set warm_vs_cold to 8" or "set your introvert_vs_extrovert to 5"
fn extract_trait_set(input: &str) -> Option<(String, u8)> {
    // Remove common prefixes like "set", "set your", "set my"
    let cleaned = input
        .trim_start_matches("set ")
        .trim_start_matches("your ")
        .trim_start_matches("my ")
        .trim();

    // Split by " to " to get trait and value
    if let Some(to_pos) = cleaned.find(" to ") {
        let trait_part = &cleaned[..to_pos].trim();
        let value_part = &cleaned[to_pos + 4..].trim();

        // Parse the value (should be 0-10)
        if let Ok(value) = value_part.parse::<u8>() {
            if value <= 10 {
                return Some((trait_part.to_string(), value));
            }
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
     Try rephrasing your question, or type 'help' for examples."
        .to_string()
}

/// Generate response for off-topic queries
pub fn offtopic_response() -> String {
    "I appreciate the chat, but I focus only on this machine's health \
     and administration. I'm here to help with:\n\
     • Hardware and software issues\n\
     • System configuration\n\
     • Performance and security\n\
     • Desktop workflows\n\n\
     Ask me something about your Arch system!"
        .to_string()
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
    fn test_anna_self_repair_intent() {
        assert_eq!(route_intent("fix yourself"), Intent::AnnaSelfRepair);
        assert_eq!(route_intent("repair yourself"), Intent::AnnaSelfRepair);
        assert_eq!(
            route_intent("check your own service"),
            Intent::AnnaSelfRepair
        );
        assert_eq!(route_intent("auto repair"), Intent::AnnaSelfRepair);
        assert_eq!(route_intent("Anna, fix yourself"), Intent::AnnaSelfRepair);
        assert_eq!(route_intent("repair anna"), Intent::AnnaSelfRepair);
        assert_eq!(route_intent("fix anna"), Intent::AnnaSelfRepair);
    }

    #[test]
    fn test_report_intent() {
        assert_eq!(route_intent("Generate a report"), Intent::Report);
        assert_eq!(route_intent("I need a summary for my boss"), Intent::Report);
        assert_eq!(route_intent("System overview"), Intent::Report);
    }

    #[test]
    fn test_suggest_intent() {
        assert_eq!(route_intent("Any suggestions?"), Intent::Suggest);
        assert_eq!(route_intent("What do you suggest?"), Intent::Suggest);
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
        // "Fix this problem" contains "this" which is excluded from Repair intent
        // (it triggers Apply or Suggest instead)
        assert_eq!(
            route_intent("Repair the system"),
            Intent::Repair { action_id: None }
        );
        assert_eq!(
            route_intent("Fix something"),
            Intent::Repair { action_id: None }
        );
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
        assert!(matches!(route_intent("asdfasdf"), Intent::Unclear(_)));
        assert!(matches!(
            route_intent("random words here"),
            Intent::Unclear(_)
        ));

        // Verify the text is preserved
        if let Intent::Unclear(text) = route_intent("test message") {
            assert_eq!(text, "test message");
        } else {
            panic!("Expected Unclear intent");
        }
    }

    #[test]
    fn test_personality_intents() {
        assert_eq!(
            route_intent("show personality settings"),
            Intent::Personality {
                adjustment: PersonalityAdjustment::Show
            }
        );
        assert_eq!(
            route_intent("Anna, be more funny"),
            Intent::Personality {
                adjustment: PersonalityAdjustment::IncreaseHumor
            }
        );
        assert_eq!(
            route_intent("Don't joke around"),
            Intent::Personality {
                adjustment: PersonalityAdjustment::DecreaseHumor
            }
        );
        assert_eq!(
            route_intent("Be more brief"),
            Intent::Personality {
                adjustment: PersonalityAdjustment::MoreBrief
            }
        );
        assert_eq!(
            route_intent("Explain in more detail"),
            Intent::Personality {
                adjustment: PersonalityAdjustment::MoreDetailed
            }
        );
    }
}
