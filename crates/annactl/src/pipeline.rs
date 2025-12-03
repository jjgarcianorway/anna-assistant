//! Anna Request Pipeline v0.0.3
//!
//! Multi-party dialogue transcript with deterministic mocks:
//! - Translator: intent classification (question, system_query, action_request, unknown)
//! - Evidence retrieval from snapshots
//! - Junior: reliability scoring (0-100%)
//!
//! No LLM integration yet - all responses are deterministic based on keywords.

use owo_colors::OwoColorize;
use std::fmt;

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

/// Junior's reliability assessment
#[derive(Debug, Clone)]
pub struct ReliabilityScore {
    pub score: u8,         // 0-100
    pub breakdown: String, // explanation of score components
}

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

/// Evidence retrieval mock: attempts to get data from snapshots
/// In v0.0.3, this is a stub that simulates snapshot access
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

/// Junior scoring mock: calculates reliability based on rubric
/// Rubric:
/// - +40: evidence exists
/// - +30: confident classification (>70%)
/// - +20: observational + cited (read-only with evidence)
/// - +10: read-only operation
pub fn junior_score(intent: &Intent, evidence: &[Evidence]) -> ReliabilityScore {
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

    ReliabilityScore {
        score,
        breakdown: breakdown_parts.join(", "),
    }
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

    // Translator classifies the intent
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
        || intent.intent_type == IntentType::Question && !intent.targets.is_empty()
    {
        // [anna] to [annad]: request for evidence
        dialogue(
            Actor::Anna,
            Actor::Annad,
            &format!(
                "Retrieve evidence for: {}",
                intent.targets.join(", ")
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

    // [anna] to [junior]: request for verification
    dialogue(
        Actor::Anna,
        Actor::Junior,
        "Please verify and score this response.",
    );
    println!();

    // Junior scores the response
    let reliability = junior_score(&intent, &evidence);

    // [junior] to [anna]: reliability score
    let junior_response = format!(
        "Reliability: {}%\nBreakdown: {}",
        reliability.score, reliability.breakdown
    );
    dialogue(Actor::Junior, Actor::Anna, &junior_response);
    println!();

    // [anna] to [you]: final response
    let final_response = generate_response(&intent, &evidence, &reliability);
    dialogue(Actor::Anna, Actor::You, &final_response);
    println!();

    // Display reliability score prominently
    let reliability_display = format!("Reliability: {}%", reliability.score);
    if reliability.score >= 80 {
        println!("  {}", reliability_display.green());
    } else if reliability.score >= 50 {
        println!("  {}", reliability_display.yellow());
    } else {
        println!("  {}", reliability_display.red());
    }
}

/// Generate a response based on intent and evidence (mock for v0.0.3)
fn generate_response(intent: &Intent, evidence: &[Evidence], _reliability: &ReliabilityScore) -> String {
    match intent.intent_type {
        IntentType::SystemQuery => {
            if evidence.is_empty() {
                "I need to query system data to answer this, but no evidence is available yet.\n\
                 (v0.0.3: Evidence retrieval is mocked - real snapshot integration coming soon)"
                    .to_string()
            } else {
                let sources: Vec<_> = evidence.iter().map(|e| e.source.as_str()).collect();
                format!(
                    "Based on system data from: {}\n\n\
                     (v0.0.3: Response generation is mocked - LLM integration coming soon)\n\
                     Evidence sources identified but not yet parsed.",
                    sources.join(", ")
                )
            }
        }
        IntentType::ActionRequest => {
            format!(
                "This is an action request classified as {}.\n\n\
                 (v0.0.3: Action execution is not yet implemented)\n\
                 Risk level: {} - would require appropriate confirmation.",
                intent.intent_type, intent.risk
            )
        }
        IntentType::Question => {
            "This appears to be a general question.\n\n\
             (v0.0.3: LLM response generation is not yet implemented)\n\
             General knowledge questions will be answered in a future version."
                .to_string()
        }
        IntentType::Unknown => {
            "I wasn't able to classify this request with confidence.\n\n\
             Could you rephrase or provide more details about what you'd like to know?"
                .to_string()
        }
    }
}
