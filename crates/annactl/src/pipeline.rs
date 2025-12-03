//! Anna Request Pipeline v0.0.4
//!
//! Multi-party dialogue transcript with:
//! - Translator: deterministic intent classification (v0.0.3 logic preserved)
//! - Evidence retrieval from snapshots
//! - Junior: real LLM verification via local Ollama
//!
//! v0.0.4: Junior becomes real via Ollama, Translator stays deterministic.

use anna_common::{
    AnnaConfig, JuniorState, OllamaClient, OllamaError,
    select_junior_model,
};
use owo_colors::OwoColorize;
use std::fmt;
use std::io::{self, Write};

/// Actors in the dialogue system
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Actor {
    You,
    Anna,
    Translator,
    Junior,
    Annad,
}

impl fmt::Display for Actor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Actor::You => write!(f, "you"),
            Actor::Anna => write!(f, "anna"),
            Actor::Translator => write!(f, "translator"),
            Actor::Junior => write!(f, "junior"),
            Actor::Annad => write!(f, "annad"),
        }
    }
}

/// Print a dialogue line in debug format
pub fn dialogue(from: Actor, to: Actor, message: &str) {
    let header = format!("[{}] to [{}]:", from, to);
    println!("  {}", header.dimmed());
    for line in message.lines() {
        println!("  {}", line);
    }
}

/// Intent classification from Translator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentType {
    /// A general question (e.g., "what is Linux?")
    Question,
    /// Needs system data from snapshots (e.g., "what CPU do I have?")
    SystemQuery,
    /// Requests an action (e.g., "install nginx")
    ActionRequest,
    /// Cannot classify
    Unknown,
}

impl fmt::Display for IntentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IntentType::Question => write!(f, "question"),
            IntentType::SystemQuery => write!(f, "system_query"),
            IntentType::ActionRequest => write!(f, "action_request"),
            IntentType::Unknown => write!(f, "unknown"),
        }
    }
}

/// Risk classification for actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    ReadOnly,
    LowRisk,
    MediumRisk,
    HighRisk,
}

impl fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RiskLevel::ReadOnly => write!(f, "read-only"),
            RiskLevel::LowRisk => write!(f, "low-risk"),
            RiskLevel::MediumRisk => write!(f, "medium-risk"),
            RiskLevel::HighRisk => write!(f, "high-risk"),
        }
    }
}

/// Structured intent from Translator
#[derive(Debug, Clone)]
pub struct Intent {
    pub intent_type: IntentType,
    pub keywords: Vec<String>,
    pub targets: Vec<String>,
    pub risk: RiskLevel,
    pub confidence: u8, // 0-100
}

/// Evidence from snapshots
#[derive(Debug, Clone)]
pub struct Evidence {
    pub source: String,
    pub data: String,
    pub timestamp: String,
}

/// Junior's reliability assessment (v0.0.4: from LLM)
#[derive(Debug, Clone)]
pub struct JuniorVerification {
    pub score: u8,              // 0-100
    pub critique: String,       // What is missing, speculative
    pub suggestions: String,    // Minimal edits to improve
    pub mutation_warning: bool, // If action request, warn about mutations
}

impl Default for JuniorVerification {
    fn default() -> Self {
        Self {
            score: 0,
            critique: String::new(),
            suggestions: String::new(),
            mutation_warning: false,
        }
    }
}

// =============================================================================
// Translator (deterministic - unchanged from v0.0.3)
// =============================================================================

/// Translator mock: deterministic intent classification based on keywords
pub fn translator_classify(request: &str) -> Intent {
    let request_lower = request.to_lowercase();

    // Extract keywords (simple word extraction)
    let keywords: Vec<String> = request_lower
        .split_whitespace()
        .filter(|w| w.len() > 2)
        .map(|s| s.to_string())
        .collect();

    // Detect targets (common system entities)
    let mut targets = Vec::new();
    let target_patterns = [
        "cpu", "memory", "ram", "disk", "network", "wifi", "ethernet",
        "nginx", "docker", "systemd", "kernel", "pacman", "yay",
        "battery", "temperature", "fan", "gpu", "audio", "bluetooth",
    ];
    for pattern in target_patterns {
        if request_lower.contains(pattern) {
            targets.push(pattern.to_string());
        }
    }

    // Classify intent type based on keywords
    let (intent_type, confidence, risk) = classify_intent(&request_lower, &targets);

    Intent {
        intent_type,
        keywords,
        targets,
        risk,
        confidence,
    }
}

fn classify_intent(request: &str, targets: &[String]) -> (IntentType, u8, RiskLevel) {
    // Action keywords (verbs that imply mutation)
    let action_keywords = [
        "install", "remove", "uninstall", "delete", "update", "upgrade",
        "start", "stop", "restart", "enable", "disable", "kill",
        "create", "add", "set", "change", "modify", "edit",
        "mount", "unmount", "format", "clean", "clear",
    ];

    // System query keywords (need snapshot data)
    let system_query_keywords = [
        "what", "which", "how much", "how many", "show", "list", "display",
        "running", "installed", "using", "usage", "available", "free",
        "status", "state", "info", "information", "details",
    ];

    // Check for action requests first (higher risk)
    for keyword in action_keywords {
        if request.contains(keyword) {
            let risk = determine_action_risk(keyword);
            return (IntentType::ActionRequest, 80, risk);
        }
    }

    // Check for system queries (needs snapshots)
    for keyword in system_query_keywords {
        if request.contains(keyword) {
            // If we have specific targets, higher confidence
            let confidence = if targets.is_empty() { 60 } else { 85 };
            return (IntentType::SystemQuery, confidence, RiskLevel::ReadOnly);
        }
    }

    // Check if it's a general question
    if request.contains('?') || request.starts_with("is ") || request.starts_with("are ")
        || request.starts_with("does ") || request.starts_with("do ")
        || request.starts_with("can ") || request.starts_with("will ")
    {
        // If targets exist, likely system query
        if !targets.is_empty() {
            return (IntentType::SystemQuery, 70, RiskLevel::ReadOnly);
        }
        return (IntentType::Question, 60, RiskLevel::ReadOnly);
    }

    // Unknown - can't classify with confidence
    (IntentType::Unknown, 20, RiskLevel::ReadOnly)
}

fn determine_action_risk(keyword: &str) -> RiskLevel {
    match keyword {
        "delete" | "remove" | "uninstall" | "format" | "kill" => RiskLevel::HighRisk,
        "install" | "update" | "upgrade" | "change" | "modify" | "edit" => RiskLevel::MediumRisk,
        "start" | "stop" | "restart" | "enable" | "disable" => RiskLevel::LowRisk,
        _ => RiskLevel::LowRisk,
    }
}

// =============================================================================
// Evidence Retrieval (mock - will be real in later version)
// =============================================================================

/// Evidence retrieval mock: attempts to get data from snapshots
pub fn retrieve_evidence(intent: &Intent) -> Vec<Evidence> {
    let mut evidence = Vec::new();

    // Mock evidence based on targets
    for target in &intent.targets {
        match target.as_str() {
            "cpu" => {
                evidence.push(Evidence {
                    source: "snapshot:hw.cpu".to_string(),
                    data: "[CPU data would come from snapshot]".to_string(),
                    timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                });
            }
            "memory" | "ram" => {
                evidence.push(Evidence {
                    source: "snapshot:hw.memory".to_string(),
                    data: "[Memory data would come from snapshot]".to_string(),
                    timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                });
            }
            "disk" => {
                evidence.push(Evidence {
                    source: "snapshot:hw.disk".to_string(),
                    data: "[Disk data would come from snapshot]".to_string(),
                    timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                });
            }
            "docker" | "nginx" | "systemd" => {
                evidence.push(Evidence {
                    source: format!("snapshot:sw.services.{}", target),
                    data: format!("[{} service data would come from snapshot]", target),
                    timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                });
            }
            _ => {
                // Generic evidence placeholder
                evidence.push(Evidence {
                    source: format!("snapshot:{}", target),
                    data: format!("[{} data would come from snapshot]", target),
                    timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                });
            }
        }
    }

    evidence
}

// =============================================================================
// Junior LLM Verification (v0.0.4 - real via Ollama)
// =============================================================================

/// System prompt for Junior verifier
const JUNIOR_SYSTEM_PROMPT: &str = r#"You are Junior, a verification assistant for Anna (a Linux system assistant).

Your job is to verify Anna's draft responses and provide a reliability score.

CRITICAL RULES:
1. NEVER invent machine facts - if data is missing, say so
2. Downscore missing evidence - no evidence = low score
3. Prefer "unknown" over guessing
4. Keep output SHORT and STRUCTURED
5. For action requests: remind that mutations should NOT be executed without confirmation

OUTPUT FORMAT (follow exactly):
SCORE: [0-100]
CRITIQUE: [What is missing or speculative, 1-2 sentences max]
SUGGESTIONS: [Minimal edits to remove speculation, 1-2 sentences max]
MUTATION_WARNING: [yes/no - only "yes" if this is an action request]

Scoring rubric:
- Evidence exists and cited: +40
- High confidence classification: +30
- Read-only with citations: +20
- Read-only operation: +10
- Missing evidence: -30
- Speculative claims: -20
- Action request without plan: -10"#;

/// Call Junior LLM for verification
async fn call_junior_llm(
    client: &OllamaClient,
    model: &str,
    request: &str,
    intent: &Intent,
    evidence: &[Evidence],
    draft_answer: &str,
) -> Result<JuniorVerification, OllamaError> {
    // Build the verification prompt
    let evidence_text = if evidence.is_empty() {
        "No evidence available.".to_string()
    } else {
        evidence
            .iter()
            .map(|e| format!("- {}: {}", e.source, e.data))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let prompt = format!(
        r#"Verify this response:

USER REQUEST: {}

TRANSLATOR ANALYSIS:
- Intent: {}
- Targets: {}
- Risk: {}
- Confidence: {}%

EVIDENCE GATHERED:
{}

ANNA'S DRAFT ANSWER:
{}

Provide your verification in the exact format specified."#,
        request,
        intent.intent_type,
        if intent.targets.is_empty() { "(none)".to_string() } else { intent.targets.join(", ") },
        intent.risk,
        intent.confidence,
        evidence_text,
        draft_answer
    );

    // Call Ollama
    let response = client
        .generate(model, &prompt, Some(JUNIOR_SYSTEM_PROMPT))
        .await?;

    // Parse the response
    Ok(parse_junior_response(&response.response, intent))
}

/// Parse Junior's LLM response into structured verification
fn parse_junior_response(response: &str, intent: &Intent) -> JuniorVerification {
    let mut verification = JuniorVerification::default();

    // Parse SCORE
    if let Some(score_line) = response.lines().find(|l| l.starts_with("SCORE:")) {
        if let Some(score_str) = score_line.strip_prefix("SCORE:") {
            if let Ok(score) = score_str.trim().parse::<u8>() {
                verification.score = score.min(100);
            }
        }
    }

    // Parse CRITIQUE
    if let Some(critique_line) = response.lines().find(|l| l.starts_with("CRITIQUE:")) {
        if let Some(critique) = critique_line.strip_prefix("CRITIQUE:") {
            verification.critique = critique.trim().to_string();
        }
    }

    // Parse SUGGESTIONS
    if let Some(suggestions_line) = response.lines().find(|l| l.starts_with("SUGGESTIONS:")) {
        if let Some(suggestions) = suggestions_line.strip_prefix("SUGGESTIONS:") {
            verification.suggestions = suggestions.trim().to_string();
        }
    }

    // Parse MUTATION_WARNING
    if let Some(warning_line) = response.lines().find(|l| l.starts_with("MUTATION_WARNING:")) {
        if let Some(warning) = warning_line.strip_prefix("MUTATION_WARNING:") {
            verification.mutation_warning = warning.trim().eq_ignore_ascii_case("yes");
        }
    } else if intent.intent_type == IntentType::ActionRequest {
        // Default to warning for action requests
        verification.mutation_warning = true;
    }

    // If score is still 0 and we got a response, try to extract from text
    if verification.score == 0 && !response.is_empty() {
        // Look for any number that could be a score
        for word in response.split_whitespace() {
            if let Ok(num) = word.trim_matches(|c: char| !c.is_numeric()).parse::<u8>() {
                if num <= 100 {
                    verification.score = num;
                    break;
                }
            }
        }
    }

    verification
}

/// Fallback scoring when Junior LLM is unavailable (v0.0.3 logic)
fn fallback_junior_score(intent: &Intent, evidence: &[Evidence]) -> JuniorVerification {
    let mut score: u8 = 0;
    let mut breakdown_parts = Vec::new();

    // +40: evidence exists
    if !evidence.is_empty() {
        score += 40;
        breakdown_parts.push(format!("+40 evidence ({} sources)", evidence.len()));
    } else {
        breakdown_parts.push("+0 no evidence".to_string());
    }

    // +30: confident classification (>70%)
    if intent.confidence > 70 {
        score += 30;
        breakdown_parts.push(format!("+30 confident ({}%)", intent.confidence));
    } else {
        breakdown_parts.push(format!("+0 low confidence ({}%)", intent.confidence));
    }

    // +20: observational + cited (read-only with evidence)
    if intent.risk == RiskLevel::ReadOnly && !evidence.is_empty() {
        score += 20;
        breakdown_parts.push("+20 observational+cited".to_string());
    }

    // +10: read-only operation
    if intent.risk == RiskLevel::ReadOnly {
        score += 10;
        breakdown_parts.push("+10 read-only".to_string());
    }

    JuniorVerification {
        score,
        critique: format!("(fallback scoring: {})", breakdown_parts.join(", ")),
        suggestions: "Junior LLM unavailable - using deterministic scoring".to_string(),
        mutation_warning: intent.intent_type == IntentType::ActionRequest,
    }
}

// =============================================================================
// Spinner for Junior thinking
// =============================================================================

/// Display a simple spinner while waiting
fn show_spinner(message: &str) {
    print!("  {} ", message.dimmed());
    io::stdout().flush().ok();
}

fn clear_spinner() {
    print!("\r\x1b[K"); // Clear line
    io::stdout().flush().ok();
}

// =============================================================================
// Pipeline
// =============================================================================

/// Check Junior availability and return client + model if ready
async fn get_junior_client() -> Option<(OllamaClient, String)> {
    let config = AnnaConfig::load();

    if !config.junior.enabled {
        return None;
    }

    let client = OllamaClient::with_url(&config.junior.ollama_url)
        .with_timeout(config.junior.timeout_ms);

    // Check availability
    if !client.is_available().await {
        return None;
    }

    // Get model
    let model = if config.junior.model.is_empty() {
        // Auto-select model
        match client.list_models().await {
            Ok(models) => {
                let model_names: Vec<String> = models.iter().map(|m| m.name.clone()).collect();
                select_junior_model(&model_names)?
            }
            Err(_) => return None,
        }
    } else {
        // Check if configured model exists
        match client.has_model(&config.junior.model).await {
            Ok(true) => config.junior.model.clone(),
            _ => return None,
        }
    };

    Some((client, model))
}

/// Process a request through the full pipeline with dialogue transcript
pub async fn process(request: &str) {
    println!();

    // [you] to [anna]: user's request
    dialogue(Actor::You, Actor::Anna, request);
    println!();

    // [anna] to [translator]: request for classification
    dialogue(
        Actor::Anna,
        Actor::Translator,
        &format!("Please classify this request:\n\"{}\"", request),
    );
    println!();

    // Translator classifies the intent (deterministic)
    let intent = translator_classify(request);

    // [translator] to [anna]: classification result
    let translator_response = format!(
        "Intent: {}\nTargets: {}\nRisk: {}\nConfidence: {}%",
        intent.intent_type,
        if intent.targets.is_empty() {
            "(none detected)".to_string()
        } else {
            intent.targets.join(", ")
        },
        intent.risk,
        intent.confidence
    );
    dialogue(Actor::Translator, Actor::Anna, &translator_response);
    println!();

    // If system query, retrieve evidence from annad
    let evidence = if intent.intent_type == IntentType::SystemQuery
        || (intent.intent_type == IntentType::Question && !intent.targets.is_empty())
    {
        // [anna] to [annad]: request for evidence
        dialogue(
            Actor::Anna,
            Actor::Annad,
            &format!(
                "Retrieve evidence for: {}",
                if intent.targets.is_empty() { "(general query)".to_string() } else { intent.targets.join(", ") }
            ),
        );
        println!();

        let ev = retrieve_evidence(&intent);

        // [annad] to [anna]: evidence response
        let evidence_summary = if ev.is_empty() {
            "No evidence found in snapshots.".to_string()
        } else {
            ev.iter()
                .map(|e| format!("- {}: {}", e.source, e.data))
                .collect::<Vec<_>>()
                .join("\n")
        };
        dialogue(Actor::Annad, Actor::Anna, &evidence_summary);
        println!();

        ev
    } else {
        Vec::new()
    };

    // Generate draft answer
    let draft_answer = generate_draft_response(&intent, &evidence);

    // [anna] to [junior]: request for verification
    dialogue(
        Actor::Anna,
        Actor::Junior,
        &format!("Please verify this draft response:\n\n{}", draft_answer),
    );
    println!();

    // Try to use real Junior LLM, fall back to deterministic scoring
    let verification = if let Some((client, model)) = get_junior_client().await {
        // Show spinner while Junior thinks
        show_spinner(&format!("[junior thinking via {}...]", model));

        match call_junior_llm(&client, &model, request, &intent, &evidence, &draft_answer).await {
            Ok(v) => {
                clear_spinner();
                v
            }
            Err(e) => {
                clear_spinner();
                println!("  {} {}", "Junior LLM error:".yellow(), e);
                fallback_junior_score(&intent, &evidence)
            }
        }
    } else {
        // Ollama not available - use fallback
        fallback_junior_score(&intent, &evidence)
    };

    // [junior] to [anna]: verification result
    let junior_response = format!(
        "Reliability: {}%\nCritique: {}\nSuggestions: {}{}",
        verification.score,
        if verification.critique.is_empty() { "(none)" } else { &verification.critique },
        if verification.suggestions.is_empty() { "(none)" } else { &verification.suggestions },
        if verification.mutation_warning {
            "\n\n*** DO NOT EXECUTE MUTATIONS without explicit user confirmation ***"
        } else {
            ""
        }
    );
    dialogue(Actor::Junior, Actor::Anna, &junior_response);
    println!();

    // [anna] to [you]: final response
    let final_response = generate_final_response(&intent, &evidence, &verification);
    dialogue(Actor::Anna, Actor::You, &final_response);
    println!();

    // Display reliability score prominently
    let reliability_display = format!("Reliability: {}%", verification.score);
    if verification.score >= 80 {
        println!("  {}", reliability_display.green());
    } else if verification.score >= 50 {
        println!("  {}", reliability_display.yellow());
    } else {
        println!("  {}", reliability_display.red());
    }
}

/// Generate a draft response based on intent and evidence
fn generate_draft_response(intent: &Intent, evidence: &[Evidence]) -> String {
    match intent.intent_type {
        IntentType::SystemQuery => {
            if evidence.is_empty() {
                "I need to query system data to answer this, but no evidence is available yet.\n\
                 The snapshot system is not providing data for these targets."
                    .to_string()
            } else {
                let sources: Vec<_> = evidence.iter().map(|e| e.source.as_str()).collect();
                format!(
                    "Based on system data from: {}\n\n\
                     Evidence sources identified. Parsing and response generation pending.",
                    sources.join(", ")
                )
            }
        }
        IntentType::ActionRequest => {
            format!(
                "This is an action request.\n\n\
                 Risk level: {}\n\
                 Targets: {}\n\n\
                 Action execution is not yet implemented.\n\
                 Would require {} confirmation before proceeding.",
                intent.risk,
                if intent.targets.is_empty() { "(none specified)".to_string() } else { intent.targets.join(", ") },
                match intent.risk {
                    RiskLevel::ReadOnly => "no",
                    RiskLevel::LowRisk => "simple y/n",
                    RiskLevel::MediumRisk => "explicit",
                    RiskLevel::HighRisk => "\"I assume the risk\"",
                }
            )
        }
        IntentType::Question => {
            "This appears to be a general question.\n\n\
             General knowledge questions require LLM response generation,\n\
             which will be implemented in a future version."
                .to_string()
        }
        IntentType::Unknown => {
            "I wasn't able to classify this request with confidence.\n\n\
             Could you rephrase or provide more details about what you'd like to know?"
                .to_string()
        }
    }
}

/// Generate final response incorporating Junior's feedback
fn generate_final_response(intent: &Intent, evidence: &[Evidence], verification: &JuniorVerification) -> String {
    let mut response = generate_draft_response(intent, evidence);

    // Add Junior's suggestions if any
    if !verification.suggestions.is_empty() && verification.suggestions != "(none)"
        && !verification.suggestions.contains("unavailable") {
        response.push_str("\n\n[Junior suggests: ");
        response.push_str(&verification.suggestions);
        response.push(']');
    }

    // Add mutation warning for action requests
    if verification.mutation_warning {
        response.push_str("\n\n[Note: This action would require explicit confirmation before execution]");
    }

    response
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translator_system_query() {
        let intent = translator_classify("what CPU do I have?");
        assert_eq!(intent.intent_type, IntentType::SystemQuery);
        assert!(intent.targets.contains(&"cpu".to_string()));
        assert_eq!(intent.risk, RiskLevel::ReadOnly);
    }

    #[test]
    fn test_translator_action_request() {
        let intent = translator_classify("install nginx");
        assert_eq!(intent.intent_type, IntentType::ActionRequest);
        assert!(intent.targets.contains(&"nginx".to_string()));
        assert_eq!(intent.risk, RiskLevel::MediumRisk);
    }

    #[test]
    fn test_translator_high_risk_action() {
        let intent = translator_classify("delete all docker containers");
        assert_eq!(intent.intent_type, IntentType::ActionRequest);
        assert!(intent.targets.contains(&"docker".to_string()));
        assert_eq!(intent.risk, RiskLevel::HighRisk);
    }

    #[test]
    fn test_translator_question() {
        // Use "is Linux good?" - starts with "is" which triggers Question path
        // but doesn't contain system query keywords like "what", "which", etc.
        let intent = translator_classify("is Linux good?");
        assert_eq!(intent.intent_type, IntentType::Question);
        assert!(intent.targets.is_empty());
        assert_eq!(intent.risk, RiskLevel::ReadOnly);
    }

    #[test]
    fn test_translator_unknown() {
        let intent = translator_classify("hello");
        assert_eq!(intent.intent_type, IntentType::Unknown);
    }

    #[test]
    fn test_fallback_scoring() {
        let intent = Intent {
            intent_type: IntentType::SystemQuery,
            keywords: vec!["cpu".to_string()],
            targets: vec!["cpu".to_string()],
            risk: RiskLevel::ReadOnly,
            confidence: 85,
        };
        let evidence = vec![Evidence {
            source: "snapshot:hw.cpu".to_string(),
            data: "test data".to_string(),
            timestamp: "now".to_string(),
        }];

        let verification = fallback_junior_score(&intent, &evidence);
        // Should get: +40 evidence + +30 confident + +20 observational + +10 read-only = 100
        assert_eq!(verification.score, 100);
    }

    #[test]
    fn test_parse_junior_response() {
        let response = "SCORE: 75\nCRITIQUE: Missing disk info\nSUGGESTIONS: Add disk usage\nMUTATION_WARNING: no";
        let intent = Intent {
            intent_type: IntentType::SystemQuery,
            keywords: vec![],
            targets: vec![],
            risk: RiskLevel::ReadOnly,
            confidence: 80,
        };

        let verification = parse_junior_response(response, &intent);
        assert_eq!(verification.score, 75);
        assert_eq!(verification.critique, "Missing disk info");
        assert_eq!(verification.suggestions, "Add disk usage");
        assert!(!verification.mutation_warning);
    }
}
