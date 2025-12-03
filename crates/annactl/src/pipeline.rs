//! Anna Request Pipeline v0.0.44 - Translator Stabilization
//!
//! Multi-party dialogue transcript with:
//! - Translator: LLM-backed intent classification with simplified canonical format
//! - Tool execution via read-only tool catalog with Evidence IDs
//! - Junior: LLM verification with strict no-guessing enforcement
//! - Doctor Registry: Automatic routing to domain-specific doctors
//! - Case logging: All requests logged for troubleshooting
//!
//! v0.0.44: Translator format simplified, doctor integration, case logging
//! v0.0.43: Doctor Registry module added (unused in pipeline)
//! v0.0.21: Performance and latency optimizations
//! v0.0.18: Secrets hygiene with redaction and leak prevention
//! v0.0.17: Multi-user correctness with target user awareness
//! v0.0.16: Mutation safety system with preflight, dry-run, auto-rollback
//! v0.0.8: First safe mutations (medium-risk only) with rollback + confirmation
//! v0.0.7: Read-only tool catalog, Evidence IDs, citations
//! v0.0.6: Real Translator LLM with clarification loop
//! v0.0.4: Junior becomes real via Ollama

use anna_common::{
    AnnaConfig, OllamaClient, OllamaError,
    select_junior_model,
    daemon_state::StatusSnapshot,
    ToolCatalog, ToolRequest, ToolResult, ToolPlan,
    EvidenceCollector, parse_tool_plan,
    execute_tool, execute_tool_plan,
    // v0.0.8: Mutation tools
    MutationToolCatalog, MutationPlan, MutationRequest, MutationResult,
    MutationRisk, MutationError, FileEditOp, RollbackInfo,
    RollbackManager, is_path_allowed, validate_confirmation,
    execute_mutation, generate_request_id,
    MEDIUM_RISK_CONFIRMATION,
    // v0.0.17: Target user system
    TargetUserSelector, TargetUserSelection, SelectionResult, UserInfo,
    get_policy,
    // v0.0.18: Secrets redaction
    redact_transcript, redact_evidence, is_path_restricted, get_restriction_message,
    // v0.0.21: Performance and caching
    ToolCache, ToolCacheKey, LlmCache, LlmCacheKey,
    PerfStats, LatencySample, BudgetViolation,
    get_snapshot_hash, get_policy_version,
    TOOL_CACHE_TTL_SECS, LLM_CACHE_TTL_SECS,
    // v0.0.31: Reliability metrics
    MetricsStore, MetricType,
    // v0.0.44: Doctor Registry integration
    DoctorRegistry, DoctorSelection,
    // v0.0.44: Case file logging for all requests
    CaseFile, CaseOutcome, CaseTiming, generate_case_id,
};
use owo_colors::OwoColorize;
use std::fmt;
use std::io::{self, Write, BufRead};

/// Maximum evidence excerpt size in bytes (8KB)
const MAX_EVIDENCE_BYTES: usize = 8 * 1024;

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

/// Get current debug level from config
fn get_debug_level() -> u8 {
    AnnaConfig::load().ui.debug_level
}

/// Check if a dialogue should be shown based on debug level
/// Level 0 (Minimal): only [you]->[anna] and final [anna]->[you], plus confirmations
/// Level 1 (Normal): dialogues condensed, tool calls summarized, evidence IDs included
/// Level 2 (Full): full dialogues between all players
fn should_show_dialogue(from: Actor, to: Actor, debug_level: u8) -> bool {
    match debug_level {
        0 => {
            // Minimal: only you<->anna exchanges
            matches!((from, to), (Actor::You, Actor::Anna) | (Actor::Anna, Actor::You))
        }
        1 => {
            // Normal: all main dialogue except internal translator/junior details
            // Show all exchanges except very verbose internal ones
            true
        }
        _ => {
            // Full: show everything
            true
        }
    }
}

/// Print a dialogue line in debug format (respects debug level)
/// v0.0.18: All dialogue output is redacted for secrets
pub fn dialogue(from: Actor, to: Actor, message: &str) {
    let debug_level = get_debug_level();
    if !should_show_dialogue(from, to, debug_level) {
        return;
    }

    // v0.0.18: Apply secrets redaction to all dialogue output
    let redacted = redact_transcript(message);

    let header = format!("[{}] to [{}]:", from, to);
    println!("  {}", header.dimmed());

    // At debug level 0, condense long messages
    if debug_level == 0 && redacted.lines().count() > 5 {
        // Show first 2 lines and summary
        for (i, line) in redacted.lines().take(2).enumerate() {
            println!("  {}", line);
            if i == 1 {
                println!("  {}", format!("... ({} more lines)", redacted.lines().count() - 2).dimmed());
            }
        }
    } else {
        for line in redacted.lines() {
            println!("  {}", line);
        }
    }
}

/// Print dialogue unconditionally (for confirmations/critical messages)
/// v0.0.18: All dialogue output is redacted for secrets
pub fn dialogue_always(from: Actor, to: Actor, message: &str) {
    // v0.0.18: Apply secrets redaction
    let redacted = redact_transcript(message);

    let header = format!("[{}] to [{}]:", from, to);
    println!("  {}", header.dimmed());
    for line in redacted.lines() {
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
    /// v0.0.34: Fix-It mode troubleshooting (e.g., "WiFi keeps disconnecting")
    FixIt,
    /// Cannot classify
    Unknown,
}

impl fmt::Display for IntentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IntentType::Question => write!(f, "question"),
            IntentType::SystemQuery => write!(f, "system_query"),
            IntentType::ActionRequest => write!(f, "action_request"),
            IntentType::FixIt => write!(f, "fix_it"),
            IntentType::Unknown => write!(f, "unknown"),
        }
    }
}

impl std::str::FromStr for IntentType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "question" => Ok(IntentType::Question),
            "system_query" => Ok(IntentType::SystemQuery),
            "action_request" => Ok(IntentType::ActionRequest),
            "fix_it" | "fixit" | "fix-it" | "troubleshoot" => Ok(IntentType::FixIt),
            "unknown" => Ok(IntentType::Unknown),
            _ => Err(()),
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
            RiskLevel::ReadOnly => write!(f, "read_only"),
            RiskLevel::LowRisk => write!(f, "low"),
            RiskLevel::MediumRisk => write!(f, "medium"),
            RiskLevel::HighRisk => write!(f, "high"),
        }
    }
}

impl std::str::FromStr for RiskLevel {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "read_only" | "read-only" | "readonly" => Ok(RiskLevel::ReadOnly),
            "low" | "low_risk" | "low-risk" => Ok(RiskLevel::LowRisk),
            "medium" | "medium_risk" | "medium-risk" => Ok(RiskLevel::MediumRisk),
            "high" | "high_risk" | "high-risk" => Ok(RiskLevel::HighRisk),
            _ => Err(()),
        }
    }
}

/// Clarification question with options (v0.0.6)
#[derive(Debug, Clone)]
pub struct Clarification {
    pub question: String,
    pub options: Vec<String>,
    pub default_option: usize, // Index of default option
}

/// Structured intent from Translator (v0.0.7: extended with tool_plan)
#[derive(Debug, Clone)]
pub struct TranslatorOutput {
    pub intent_type: IntentType,
    pub targets: Vec<String>,
    pub risk: RiskLevel,
    pub evidence_needs: Vec<String>,
    pub tool_plan: Option<ToolPlan>, // v0.0.7: planned tool calls
    pub clarification: Option<Clarification>,
    pub confidence: u8,
    pub llm_backed: bool, // True if from LLM, false if deterministic fallback
}


/// Evidence from snapshots
#[derive(Debug, Clone)]
pub struct Evidence {
    pub source: String,
    pub data: String,
    pub timestamp: String,
    pub excerpted: bool, // True if data was truncated
}

/// Action plan for action requests (v0.0.6)
#[derive(Debug, Clone)]
pub struct ActionPlan {
    pub steps: Vec<String>,
    pub affected_files: Vec<String>,
    pub affected_services: Vec<String>,
    pub affected_packages: Vec<String>,
    pub risk: RiskLevel,
    pub confirmation_phrase: String,
    pub rollback_outline: String,
    // v0.0.8: Mutation plan for executable actions
    pub mutation_plan: Option<MutationPlan>,
    pub is_medium_risk_executable: bool, // true if can be executed in v0.0.8
}

/// Junior's reliability assessment (v0.0.7: with uncited claims tracking)
#[derive(Debug, Clone)]
pub struct JuniorVerification {
    pub score: u8,              // 0-100
    pub critique: String,       // What is missing, speculative
    pub uncited_claims: Vec<String>, // v0.0.7: claims without [E#] citations
    pub suggestions: String,    // Minimal edits to improve
    pub mutation_warning: bool, // If action request, warn about mutations
}

impl Default for JuniorVerification {
    fn default() -> Self {
        Self {
            score: 0,
            critique: String::new(),
            uncited_claims: Vec::new(),
            suggestions: String::new(),
            mutation_warning: false,
        }
    }
}

// =============================================================================
// Translator LLM (v0.0.44 - simplified canonical format)
// =============================================================================

/// System prompt for Translator (v0.0.44: simplified canonical format for small models)
/// This format is designed to be extremely easy for small models to follow.
const TRANSLATOR_SYSTEM_PROMPT: &str = r#"Classify the user request. Output EXACTLY 6 lines:

INTENT: system_query OR action_request OR knowledge_query OR doctor_query
TARGETS: word1,word2 OR none
RISK: read_only OR low OR medium OR high
TOOLS: tool1,tool2 OR none
DOCTOR: networking OR graphics OR audio OR storage OR boot OR none
CONFIDENCE: 0 to 100

INTENT RULES:
- system_query = asks about THIS machine (CPU, RAM, disk, services, processes)
- action_request = wants to change something (install, restart, edit, delete)
- knowledge_query = asks HOW TO do something or WHAT IS something
- doctor_query = reports a problem (slow, broken, disconnecting, not working)

TOOLS (pick relevant ones):
- hw_snapshot_summary = CPU, RAM, GPU, storage info
- sw_snapshot_summary = packages, services
- disk_usage = filesystem usage
- service_status = check a service
- journal_warnings = recent errors/warnings
- knowledge_search = documentation lookup

DOCTOR (only for problems):
- networking = wifi, ethernet, DNS, connection issues
- graphics = display, GPU, resolution, tearing
- audio = sound, speakers, microphone
- storage = disk, mount, filesystem
- boot = startup, systemd, slow boot

EXAMPLES:
User: "what cpu do I have"
INTENT: system_query
TARGETS: cpu
RISK: read_only
TOOLS: hw_snapshot_summary
DOCTOR: none
CONFIDENCE: 95

User: "install nginx"
INTENT: action_request
TARGETS: nginx
RISK: medium
TOOLS: sw_snapshot_summary
DOCTOR: none
CONFIDENCE: 90

User: "wifi keeps disconnecting"
INTENT: doctor_query
TARGETS: wifi,network
RISK: read_only
TOOLS: hw_snapshot_summary,journal_warnings
DOCTOR: networking
CONFIDENCE: 85"#;

/// Maximum retries for Translator LLM before fallback
const TRANSLATOR_MAX_RETRIES: u32 = 2;

/// Call Translator LLM for intent classification with retry
/// v0.0.44: Adds retry logic before falling back to deterministic
async fn call_translator_llm(
    client: &OllamaClient,
    model: &str,
    request: &str,
) -> Result<TranslatorOutput, OllamaError> {
    // v0.0.44: Simplified prompt - just show the request
    let prompt = format!("User: \"{}\"\n\nClassify:", request);

    let mut last_error = None;

    for attempt in 0..TRANSLATOR_MAX_RETRIES {
        let response = match client
            .generate(model, &prompt, Some(TRANSLATOR_SYSTEM_PROMPT))
            .await {
                Ok(r) => r,
                Err(e) => {
                    last_error = Some(e);
                    continue;
                }
            };

        // Parse the response
        match parse_translator_response(&response.response) {
            Some(output) => return Ok(output),
            None => {
                // Log the failure
                if attempt == 0 {
                    tracing::warn!("Translator LLM parse failed (attempt {}): {}",
                        attempt + 1,
                        response.response.lines().take(3).collect::<Vec<_>>().join(" | "));
                }
                last_error = Some(OllamaError::ParseError(
                    format!("Invalid format on attempt {}", attempt + 1)
                ));
            }
        }
    }

    // All retries exhausted
    Err(last_error.unwrap_or_else(||
        OllamaError::ParseError("All Translator retries failed".to_string())))
}

/// Parse Translator LLM response into structured output
/// v0.0.44: Simplified canonical format parser
/// Expected format:
///   INTENT: system_query|action_request|knowledge_query|doctor_query
///   TARGETS: word1,word2 OR none
///   RISK: read_only|low|medium|high
///   TOOLS: tool1,tool2 OR none
///   DOCTOR: networking|graphics|audio|storage|boot OR none
///   CONFIDENCE: 0-100
fn parse_translator_response(response: &str) -> Option<TranslatorOutput> {
    let mut intent_type = None;
    let mut targets = Vec::new();
    let mut risk = RiskLevel::ReadOnly;
    let mut tools_list: Vec<String> = Vec::new();
    let mut doctor_domain: Option<String> = None;
    let mut confidence: u8 = 85;

    for line in response.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Parse INTENT (case-insensitive prefix)
        if let Some(value) = strip_prefix_case_insensitive(line, "INTENT:") {
            let value_lower = value.trim().to_lowercase();
            intent_type = match value_lower.as_str() {
                "system_query" | "system query" => Some(IntentType::SystemQuery),
                "action_request" | "action request" => Some(IntentType::ActionRequest),
                "knowledge_query" | "knowledge query" | "question" => Some(IntentType::Question),
                "doctor_query" | "doctor query" | "fix_it" | "fixit" => Some(IntentType::FixIt),
                "unknown" => Some(IntentType::Unknown),
                _ => {
                    // Fallback: check for keywords in value
                    if value_lower.contains("system") || value_lower.contains("query") {
                        Some(IntentType::SystemQuery)
                    } else if value_lower.contains("action") || value_lower.contains("install")
                        || value_lower.contains("restart") {
                        Some(IntentType::ActionRequest)
                    } else if value_lower.contains("doctor") || value_lower.contains("fix") {
                        Some(IntentType::FixIt)
                    } else if value_lower.contains("knowledge") || value_lower.contains("how") {
                        Some(IntentType::Question)
                    } else {
                        None
                    }
                }
            };
        }
        // Parse TARGETS
        else if let Some(value) = strip_prefix_case_insensitive(line, "TARGETS:") {
            let value = value.trim().trim_matches('"');
            if !value.eq_ignore_ascii_case("none") && !value.is_empty() {
                targets = value.split(',')
                    .map(|s| s.trim().to_lowercase())
                    .filter(|s| !s.is_empty() && s != "none")
                    .collect();
            }
        }
        // Parse RISK
        else if let Some(value) = strip_prefix_case_insensitive(line, "RISK:") {
            risk = value.trim().to_lowercase().parse().unwrap_or(RiskLevel::ReadOnly);
        }
        // Parse TOOLS
        else if let Some(value) = strip_prefix_case_insensitive(line, "TOOLS:") {
            let value = value.trim();
            if !value.eq_ignore_ascii_case("none") && !value.is_empty() {
                tools_list = value.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("none"))
                    .collect();
            }
        }
        // Parse DOCTOR (v0.0.44: new field for doctor routing)
        else if let Some(value) = strip_prefix_case_insensitive(line, "DOCTOR:") {
            let value = value.trim().to_lowercase();
            if !value.eq_ignore_ascii_case("none") && !value.is_empty() {
                doctor_domain = Some(value);
            }
        }
        // Parse CONFIDENCE
        else if let Some(value) = strip_prefix_case_insensitive(line, "CONFIDENCE:") {
            if let Ok(c) = value.trim().parse::<u8>() {
                confidence = c.min(100);
            }
        }
    }

    // Must have at least INTENT to be valid
    let intent_type = intent_type?;

    // Generate tool_plan from tools_list
    let tool_plan = generate_tool_plan_from_tools_list(&tools_list);

    // Generate evidence_needs from tools (for backwards compat)
    let evidence_needs = tools_list.iter()
        .map(|t| {
            if t.contains("hw_snapshot") { "hw_snapshot".to_string() }
            else if t.contains("sw_snapshot") { "sw_snapshot".to_string() }
            else if t.contains("journal") { "journalctl".to_string() }
            else if t.contains("status") { "status".to_string() }
            else { t.clone() }
        })
        .collect();

    // If doctor_query with doctor domain, trigger FixIt mode
    let final_intent = if doctor_domain.is_some() && intent_type == IntentType::Question {
        IntentType::FixIt
    } else {
        intent_type
    };

    Some(TranslatorOutput {
        intent_type: final_intent,
        targets,
        risk,
        evidence_needs,
        tool_plan,
        clarification: None, // v0.0.44: Removed clarification from canonical format
        confidence,
        llm_backed: true,
    })
}

/// Case-insensitive prefix stripping helper
fn strip_prefix_case_insensitive<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
    let line_upper = line.to_uppercase();
    let prefix_upper = prefix.to_uppercase();
    if line_upper.starts_with(&prefix_upper) {
        Some(&line[prefix.len()..])
    } else {
        None
    }
}

/// Generate a ToolPlan from a list of tool names (v0.0.44)
fn generate_tool_plan_from_tools_list(tools: &[String]) -> Option<ToolPlan> {
    use std::collections::HashMap;

    if tools.is_empty() {
        return None;
    }

    let mut plan = ToolPlan::new();

    for tool in tools {
        let tool_lower = tool.to_lowercase();
        // Parse tool(args) format
        if let Some(paren_idx) = tool.find('(') {
            let tool_name = &tool[..paren_idx];
            let args_str = tool.get(paren_idx + 1..tool.len().saturating_sub(1)).unwrap_or("");
            let mut params = HashMap::new();

            // Parse key=value pairs
            for arg in args_str.split(',') {
                if let Some((key, value)) = arg.split_once('=') {
                    let key = key.trim();
                    let value = value.trim();
                    // Try to parse as number, else string
                    if let Ok(n) = value.parse::<i64>() {
                        params.insert(key.to_string(), serde_json::json!(n));
                    } else {
                        params.insert(key.to_string(), serde_json::json!(value));
                    }
                }
            }
            plan.add_tool(tool_name.trim(), params);
        } else {
            // Simple tool name without args
            plan.add_tool(&tool_lower, HashMap::new());
        }
    }

    plan.rationale = "Tools selected by Translator LLM".to_string();

    if plan.tools.is_empty() {
        None
    } else {
        Some(plan)
    }
}

/// Parse clarification string: "question|option1|option2|default:N"
/// v0.0.44: Clarification removed from canonical format, kept for potential future use
#[allow(dead_code)]
fn parse_clarification(s: &str) -> Option<Clarification> {
    // Reject if it looks like echoed prompt instructions
    if s.contains("empty if not needed") || s.contains("OR \"question") || s.starts_with('[') {
        return None;
    }

    let parts: Vec<&str> = s.split('|').collect();
    if parts.len() < 3 {
        return None;
    }

    let question = parts[0].trim().to_string();

    // Question must end with ? to be valid
    if !question.ends_with('?') {
        return None;
    }

    let mut options = Vec::new();
    let mut default_option = 0;

    for part in &parts[1..] {
        let part = part.trim();
        if let Some(n) = part.strip_prefix("default:") {
            default_option = n.trim_matches(|c| c == '"' || c == ']').parse().unwrap_or(0);
        } else if !part.is_empty() && !part.starts_with('[') && !part.ends_with(']') {
            // Reject placeholder options like "option1", "option2", etc.
            if part == "option1" || part == "option2" || part == "option3" || part == "option4" {
                return None;
            }
            options.push(part.to_string());
        }
    }

    if options.is_empty() {
        return None;
    }

    // Ensure default is valid
    if default_option >= options.len() {
        default_option = 0;
    }

    Some(Clarification {
        question,
        options,
        default_option,
    })
}

// =============================================================================
// Translator Deterministic Fallback (v0.0.3 logic)
// =============================================================================

/// Deterministic translator fallback when LLM is unavailable
pub fn translator_classify_deterministic(request: &str) -> TranslatorOutput {
    let request_lower = request.to_lowercase();

    // Detect targets (common system entities)
    let mut targets = Vec::new();
    let target_patterns = [
        "cpu", "memory", "ram", "disk", "network", "wifi", "ethernet",
        "nginx", "docker", "systemd", "kernel", "pacman", "yay",
        "battery", "temperature", "fan", "gpu", "audio", "bluetooth",
        "ssh", "sshd", "firewall", "ufw", "iptables",
    ];
    for pattern in target_patterns {
        if request_lower.contains(pattern) {
            targets.push(pattern.to_string());
        }
    }

    // Classify intent type and risk
    let (intent_type, risk, evidence_needs) = classify_intent_deterministic(&request_lower, &targets);

    // v0.0.7: Generate tool_plan from evidence_needs
    let tool_plan = generate_tool_plan_from_evidence_needs(&evidence_needs, &targets);

    TranslatorOutput {
        intent_type,
        targets,
        risk,
        evidence_needs,
        tool_plan,
        clarification: None, // Deterministic doesn't do clarification
        confidence: 70, // Lower confidence for deterministic
        llm_backed: false,
    }
}

/// Generate a ToolPlan from evidence_needs (v0.0.7, v0.0.31: reliability tools, v0.0.46: domain-specific)
fn generate_tool_plan_from_evidence_needs(evidence_needs: &[String], targets: &[String]) -> Option<ToolPlan> {
    use std::collections::HashMap;

    if evidence_needs.is_empty() {
        return None;
    }

    let mut plan = ToolPlan::new();

    for need in evidence_needs {
        match need.as_str() {
            // v0.0.46: Domain-specific tools take priority over generic snapshots
            "mount_usage" => {
                plan.add_tool("mount_usage", HashMap::new());
            }
            "uname_summary" => {
                plan.add_tool("uname_summary", HashMap::new());
            }
            "mem_summary" => {
                plan.add_tool("mem_summary", HashMap::new());
            }
            "network_tools" => {
                plan.add_tool("nm_summary", HashMap::new());
                plan.add_tool("ip_route_summary", HashMap::new());
                plan.add_tool("link_state_summary", HashMap::new());
            }
            "audio_tools" => {
                plan.add_tool("audio_services_summary", HashMap::new());
                plan.add_tool("pactl_summary", HashMap::new());
            }
            "boot_tools" => {
                plan.add_tool("boot_time_summary", HashMap::new());
            }
            "error_tools" => {
                let mut params = HashMap::new();
                params.insert("minutes".to_string(), serde_json::json!(30));
                plan.add_tool("recent_errors_summary", params);
            }
            // Legacy tools (still valid for general queries)
            "hw_snapshot" => {
                plan.add_tool("hw_snapshot_summary", HashMap::new());
            }
            "sw_snapshot" => {
                plan.add_tool("sw_snapshot_summary", HashMap::new());
            }
            "status" => {
                plan.add_tool("status_snapshot", HashMap::new());
            }
            "journalctl" => {
                // Add journal_warnings for relevant targets
                for target in targets {
                    if ["nginx", "docker", "sshd", "systemd"].contains(&target.as_str()) {
                        let mut params = HashMap::new();
                        params.insert("service".to_string(), serde_json::json!(target));
                        params.insert("minutes".to_string(), serde_json::json!(60));
                        plan.add_tool("journal_warnings", params);
                        break; // Only one journal query
                    }
                }
                if plan.tools.iter().all(|t| t.tool_name != "journal_warnings") {
                    // Generic system journal
                    let mut params = HashMap::new();
                    params.insert("minutes".to_string(), serde_json::json!(60));
                    plan.add_tool("journal_warnings", params);
                }
            }
            // v0.0.31: Reliability engineering tools
            "self_diagnostics" => {
                plan.add_tool("self_diagnostics", HashMap::new());
            }
            "metrics_summary" => {
                let mut params = HashMap::new();
                params.insert("days".to_string(), serde_json::json!(1));
                plan.add_tool("metrics_summary", params);
            }
            "error_budgets" => {
                plan.add_tool("error_budgets", HashMap::new());
            }
            _ => {}
        }
    }

    // Add target-specific tools
    for target in targets {
        if ["nginx", "docker", "sshd", "ssh"].contains(&target.as_str()) {
            let mut params = HashMap::new();
            let svc_name = if target == "ssh" { "sshd" } else { target };
            params.insert("name".to_string(), serde_json::json!(svc_name));
            plan.add_tool("service_status", params);
            break;
        }
    }

    plan.rationale = "Gathering evidence based on detected targets".to_string();

    if plan.tools.is_empty() {
        None
    } else {
        Some(plan)
    }
}

fn classify_intent_deterministic(request: &str, targets: &[String]) -> (IntentType, RiskLevel, Vec<String>) {
    // v0.0.31: Check for reliability engineering requests first
    let reliability_keywords = [
        "diagnostics", "diagnostic", "self-diagnostics", "bug report",
        "metrics", "reliability", "error budget", "error budgets",
    ];
    for keyword in reliability_keywords {
        if request.contains(keyword) {
            // Map to specific tool
            if request.contains("diagnostics") || request.contains("bug report") {
                return (IntentType::SystemQuery, RiskLevel::ReadOnly, vec!["self_diagnostics".to_string()]);
            } else if request.contains("budget") {
                return (IntentType::SystemQuery, RiskLevel::ReadOnly, vec!["error_budgets".to_string()]);
            } else {
                return (IntentType::SystemQuery, RiskLevel::ReadOnly, vec!["metrics_summary".to_string()]);
            }
        }
    }

    // v0.0.33: Check for case file requests
    let case_keywords = ["case", "cases", "failure", "failures", "transcript", "conversation"];
    let case_action_words = ["show", "list", "get", "what happened", "last", "today", "recent"];

    let has_case_keyword = case_keywords.iter().any(|k| request.contains(k));
    let has_case_action = case_action_words.iter().any(|k| request.contains(k));

    if has_case_keyword && has_case_action {
        // Determine which case tool to use
        if request.contains("failure") {
            return (IntentType::SystemQuery, RiskLevel::ReadOnly, vec!["last_failure_summary".to_string()]);
        } else if request.contains("today") {
            return (IntentType::SystemQuery, RiskLevel::ReadOnly, vec!["list_today_cases".to_string()]);
        } else if request.contains("recent") || request.contains("list") {
            return (IntentType::SystemQuery, RiskLevel::ReadOnly, vec!["list_recent_cases".to_string()]);
        } else {
            // Default: show last case summary
            return (IntentType::SystemQuery, RiskLevel::ReadOnly, vec!["last_case_summary".to_string()]);
        }
    }

    // v0.0.34: Check for Fix-It mode requests (troubleshooting)
    let fix_patterns = [
        "fix my", "fix the", "repair", "troubleshoot", "debug",
        "not working", "won't work", "doesn't work", "broken",
        "keeps disconnecting", "keeps crashing", "keeps failing",
        "is slow", "is slower", "is broken", "is failing",
        "won't start", "can't connect", "cannot connect",
        "help me fix", "something wrong", "having issues",
        "having problems", "having trouble",
    ];

    if fix_patterns.iter().any(|p| request.contains(p)) {
        // Fix-It mode: collect evidence based on problem category
        let evidence_needs = vec![
            "hw_snapshot".to_string(),
            "sw_snapshot".to_string(),
            "journal_warnings".to_string(),
        ];
        return (IntentType::FixIt, RiskLevel::MediumRisk, evidence_needs);
    }

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
        "status", "state", "info", "information", "details", "version",
    ];

    // Check for action requests first (higher risk)
    for keyword in action_keywords {
        if request.contains(keyword) {
            let risk = determine_action_risk(keyword);
            let evidence_needs = vec!["sw_snapshot".to_string()];
            return (IntentType::ActionRequest, risk, evidence_needs);
        }
    }

    // v0.0.46: Domain-specific evidence routing
    // These domain queries MUST use domain-specific tools, NOT generic snapshots
    let evidence_needs = route_to_domain_evidence(request, targets);
    if !evidence_needs.is_empty() {
        return (IntentType::SystemQuery, RiskLevel::ReadOnly, evidence_needs);
    }

    // Check for system queries (needs snapshots) - fallback for non-domain queries
    for keyword in system_query_keywords {
        if request.contains(keyword) {
            let evidence_needs = if targets.iter().any(|t| ["nginx", "docker", "systemd", "ssh", "sshd"].contains(&t.as_str())) {
                vec!["sw_snapshot".to_string(), "status".to_string()]
            } else if targets.iter().any(|t| ["cpu", "gpu", "battery", "temperature", "fan"].contains(&t.as_str())) {
                // CPU/GPU can still use hw_snapshot as they need model info
                vec!["hw_snapshot".to_string()]
            } else {
                vec!["hw_snapshot".to_string(), "sw_snapshot".to_string()]
            };
            return (IntentType::SystemQuery, RiskLevel::ReadOnly, evidence_needs);
        }
    }

    // Check if it's a general question
    if request.contains('?') || request.starts_with("is ") || request.starts_with("are ")
        || request.starts_with("does ") || request.starts_with("do ")
        || request.starts_with("can ") || request.starts_with("will ")
    {
        if !targets.is_empty() {
            // Try domain routing first
            let domain_evidence = route_to_domain_evidence(request, targets);
            if !domain_evidence.is_empty() {
                return (IntentType::SystemQuery, RiskLevel::ReadOnly, domain_evidence);
            }
            let evidence_needs = vec!["hw_snapshot".to_string(), "sw_snapshot".to_string()];
            return (IntentType::SystemQuery, RiskLevel::ReadOnly, evidence_needs);
        }
        return (IntentType::Question, RiskLevel::ReadOnly, vec![]);
    }

    // Unknown
    (IntentType::Unknown, RiskLevel::ReadOnly, vec![])
}

/// v0.0.46: Route requests to domain-specific evidence tools
/// Returns empty vec if no domain match, allowing fallback to generic snapshots
fn route_to_domain_evidence(request: &str, targets: &[String]) -> Vec<String> {
    // Disk/storage domain - MUST use mount_usage, NOT hw_snapshot
    let disk_patterns = [
        "disk space", "disk free", "free space", "storage space",
        "how much space", "space left", "space available", "space on /",
        "disk usage", "disk full", "running out of space",
    ];
    if disk_patterns.iter().any(|p| request.contains(p))
        || targets.iter().any(|t| t == "disk")
    {
        return vec!["mount_usage".to_string()];
    }

    // Kernel domain - MUST use uname_summary, NOT hw_snapshot
    let kernel_patterns = [
        "kernel version", "kernel release", "what kernel",
        "linux version", "uname",
    ];
    if kernel_patterns.iter().any(|p| request.contains(p))
        || targets.iter().any(|t| t == "kernel")
    {
        return vec!["uname_summary".to_string()];
    }

    // Memory domain - MUST use mem_summary, NOT hw_snapshot
    let memory_patterns = [
        "memory available", "memory free", "memory used", "memory usage",
        "how much memory", "how much ram", "ram available", "ram free",
        "ram usage", "ram used",
    ];
    if memory_patterns.iter().any(|p| request.contains(p))
        || (targets.iter().any(|t| t == "memory" || t == "ram")
            && (request.contains("how much") || request.contains("available")
                || request.contains("free") || request.contains("used")
                || request.contains("usage")))
    {
        return vec!["mem_summary".to_string()];
    }

    // Network domain - MUST use network tools, NOT hw_snapshot
    let network_patterns = [
        "network status", "network connection", "internet connection",
        "default route", "network interface", "networkmanager",
        "is network", "is internet", "can i connect", "am i online",
        "wifi status", "ethernet status", "connection status",
    ];
    if network_patterns.iter().any(|p| request.contains(p))
        || (targets.iter().any(|t| t == "network" || t == "wifi" || t == "ethernet")
            && (request.contains("status") || request.contains("running")
                || request.contains("working") || request.contains("connected")))
    {
        return vec!["network_tools".to_string()];
    }

    // Audio domain - MUST use audio tools, NOT hw_snapshot
    let audio_patterns = [
        "audio status", "audio working", "sound working", "sound status",
        "pipewire", "wireplumber", "pulseaudio", "audio output",
        "no sound", "no audio", "is audio", "is sound",
        "speaker", "headphone", "microphone",
    ];
    if audio_patterns.iter().any(|p| request.contains(p))
        || (targets.iter().any(|t| t == "audio")
            && (request.contains("status") || request.contains("working")
                || request.contains("running")))
    {
        return vec!["audio_tools".to_string()];
    }

    // Boot domain - MUST use boot tools
    let boot_patterns = [
        "boot time", "boot speed", "boot slow", "startup time",
        "how long to boot", "systemd-analyze",
    ];
    if boot_patterns.iter().any(|p| request.contains(p)) {
        return vec!["boot_tools".to_string()];
    }

    // Error/logs domain - MUST use error tools
    let error_patterns = [
        "recent errors", "show errors", "system errors", "error log",
        "journal errors", "warnings", "what errors", "any errors",
    ];
    if error_patterns.iter().any(|p| request.contains(p)) {
        return vec!["error_tools".to_string()];
    }

    // No domain match - return empty to allow fallback
    vec![]
}

/// v0.0.46: Tool sanity gate - ensures domain queries have domain tools
/// Returns modified tool plan if fixup needed, None if plan is acceptable
pub fn apply_tool_sanity_gate(
    translator_output: &TranslatorOutput,
    tool_plan: &Option<ToolPlan>,
) -> Option<ToolPlan> {
    use std::collections::HashMap;

    // Only apply for SystemQuery intent
    if translator_output.intent_type != IntentType::SystemQuery {
        return None;
    }

    let Some(plan) = tool_plan else {
        return None;
    };

    // Check what domain tools are present
    let has_mount_usage = plan.tools.iter().any(|t| t.tool_name == "mount_usage");
    let has_uname = plan.tools.iter().any(|t| t.tool_name == "uname_summary");
    let has_mem = plan.tools.iter().any(|t| t.tool_name == "mem_summary");
    let has_network = plan.tools.iter().any(|t|
        t.tool_name == "nm_summary" || t.tool_name == "ip_route_summary"
        || t.tool_name == "link_state_summary" || t.tool_name == "network_status"
    );
    let has_audio = plan.tools.iter().any(|t|
        t.tool_name == "audio_services_summary" || t.tool_name == "pactl_summary"
        || t.tool_name == "audio_status"
    );

    let has_generic_hw = plan.tools.iter().any(|t| t.tool_name == "hw_snapshot_summary");

    // Check targets to see if we need domain tools
    let needs_disk = translator_output.targets.iter().any(|t| t == "disk");
    let needs_kernel = translator_output.targets.iter().any(|t| t == "kernel");
    let needs_memory = translator_output.targets.iter().any(|t| t == "memory" || t == "ram");
    let needs_network = translator_output.targets.iter().any(|t|
        t == "network" || t == "wifi" || t == "ethernet"
    );
    let needs_audio = translator_output.targets.iter().any(|t| t == "audio");

    // If using generic hw_snapshot but needs domain tools, replace
    if has_generic_hw {
        let mut new_plan = ToolPlan::new();

        // Add domain-specific tools based on targets
        if needs_disk && !has_mount_usage {
            new_plan.add_tool("mount_usage", HashMap::new());
        }
        if needs_kernel && !has_uname {
            new_plan.add_tool("uname_summary", HashMap::new());
        }
        if needs_memory && !has_mem {
            new_plan.add_tool("mem_summary", HashMap::new());
        }
        if needs_network && !has_network {
            new_plan.add_tool("nm_summary", HashMap::new());
            new_plan.add_tool("ip_route_summary", HashMap::new());
            new_plan.add_tool("link_state_summary", HashMap::new());
        }
        if needs_audio && !has_audio {
            new_plan.add_tool("audio_services_summary", HashMap::new());
            new_plan.add_tool("pactl_summary", HashMap::new());
        }

        // If we added domain tools, return the new plan
        if !new_plan.tools.is_empty() {
            new_plan.rationale = "Domain-specific tools added by sanity gate".to_string();
            return Some(new_plan);
        }
    }

    None
}

fn determine_action_risk(keyword: &str) -> RiskLevel {
    match keyword {
        "delete" | "remove" | "uninstall" | "format" | "kill" | "clean" | "clear" => RiskLevel::HighRisk,
        "install" | "update" | "upgrade" | "change" | "modify" | "edit" | "create" | "add" | "set" => RiskLevel::MediumRisk,
        "start" | "stop" | "restart" | "enable" | "disable" | "mount" | "unmount" => RiskLevel::LowRisk,
        _ => RiskLevel::LowRisk,
    }
}

// =============================================================================
// Evidence Retrieval (v0.0.6 - real snapshot integration)
// =============================================================================

/// Retrieve real evidence from snapshots with excerpting
pub fn retrieve_evidence(translator_output: &TranslatorOutput) -> Vec<Evidence> {
    let mut evidence = Vec::new();
    let mut total_bytes = 0;

    for need in &translator_output.evidence_needs {
        if total_bytes >= MAX_EVIDENCE_BYTES {
            break;
        }

        let remaining_bytes = MAX_EVIDENCE_BYTES - total_bytes;

        match need.as_str() {
            "hw_snapshot" => {
                if let Some(ev) = load_hw_evidence(remaining_bytes) {
                    total_bytes += ev.data.len();
                    evidence.push(ev);
                }
            }
            "sw_snapshot" => {
                if let Some(ev) = load_sw_evidence(&translator_output.targets, remaining_bytes) {
                    total_bytes += ev.data.len();
                    evidence.push(ev);
                }
            }
            "status" => {
                if let Some(ev) = load_status_evidence(remaining_bytes) {
                    total_bytes += ev.data.len();
                    evidence.push(ev);
                }
            }
            "journalctl" => {
                if let Some(ev) = load_journal_evidence(&translator_output.targets, remaining_bytes) {
                    total_bytes += ev.data.len();
                    evidence.push(ev);
                }
            }
            _ => {}
        }
    }

    // Also load target-specific evidence
    for target in &translator_output.targets {
        if total_bytes >= MAX_EVIDENCE_BYTES {
            break;
        }
        let remaining_bytes = MAX_EVIDENCE_BYTES - total_bytes;
        if let Some(ev) = load_target_evidence(target, remaining_bytes) {
            total_bytes += ev.data.len();
            evidence.push(ev);
        }
    }

    evidence
}

fn load_hw_evidence(max_bytes: usize) -> Option<Evidence> {
    // Try to load hw.json snapshot
    let hw_path = "/var/lib/anna/internal/snapshots/hw.json";
    if let Ok(content) = std::fs::read_to_string(hw_path) {
        let (data, excerpted) = excerpt_data(&content, max_bytes);
        return Some(Evidence {
            source: "snapshot:hw".to_string(),
            data,
            timestamp: get_file_timestamp(hw_path),
            excerpted,
        });
    }
    None
}

fn load_sw_evidence(targets: &[String], max_bytes: usize) -> Option<Evidence> {
    // Try to load sw.json snapshot
    let sw_path = "/var/lib/anna/internal/snapshots/sw.json";
    if let Ok(content) = std::fs::read_to_string(sw_path) {
        // If we have specific targets, try to extract relevant sections
        let relevant = if targets.is_empty() {
            content.clone()
        } else {
            extract_relevant_sw_data(&content, targets)
        };
        let (data, excerpted) = excerpt_data(&relevant, max_bytes);
        return Some(Evidence {
            source: "snapshot:sw".to_string(),
            data,
            timestamp: get_file_timestamp(sw_path),
            excerpted,
        });
    }
    None
}

fn load_status_evidence(max_bytes: usize) -> Option<Evidence> {
    if let Some(snapshot) = StatusSnapshot::load() {
        let data = format!(
            "Daemon: v{}, uptime: {}s, healthy: {}, objects: {}",
            snapshot.version,
            snapshot.uptime_secs,
            snapshot.healthy,
            snapshot.knowledge_objects
        );
        let (data, excerpted) = excerpt_data(&data, max_bytes);
        return Some(Evidence {
            source: "snapshot:status".to_string(),
            data,
            timestamp: snapshot.generated_at.map(|t| t.to_rfc3339()).unwrap_or_else(|| "unknown".to_string()),
            excerpted,
        });
    }
    None
}

fn load_journal_evidence(targets: &[String], max_bytes: usize) -> Option<Evidence> {
    // Get recent journal entries for relevant services
    let services: Vec<&str> = targets.iter()
        .filter(|t| ["nginx", "docker", "sshd", "ssh", "systemd"].contains(&t.as_str()))
        .map(|s| s.as_str())
        .collect();

    if services.is_empty() {
        return None;
    }

    // Try to get journal entries (limited)
    let unit = services.first().unwrap_or(&"");
    let output = std::process::Command::new("journalctl")
        .args(["-u", unit, "-n", "10", "--no-pager", "-q"])
        .output()
        .ok()?;

    if output.status.success() {
        let content = String::from_utf8_lossy(&output.stdout).to_string();
        let (data, excerpted) = excerpt_data(&content, max_bytes);
        return Some(Evidence {
            source: format!("journalctl:{}", unit),
            data,
            timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            excerpted,
        });
    }
    None
}

fn load_target_evidence(target: &str, max_bytes: usize) -> Option<Evidence> {
    match target {
        "cpu" => {
            if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
                // Extract just model name and core count
                let lines: Vec<&str> = content.lines()
                    .filter(|l| l.starts_with("model name") || l.starts_with("processor"))
                    .take(5)
                    .collect();
                let summary = lines.join("\n");
                let (data, excerpted) = excerpt_data(&summary, max_bytes);
                return Some(Evidence {
                    source: "/proc/cpuinfo".to_string(),
                    data,
                    timestamp: "live".to_string(),
                    excerpted,
                });
            }
        }
        "memory" | "ram" => {
            if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
                let lines: Vec<&str> = content.lines()
                    .filter(|l| l.starts_with("MemTotal") || l.starts_with("MemFree") || l.starts_with("MemAvailable"))
                    .collect();
                let summary = lines.join("\n");
                let (data, excerpted) = excerpt_data(&summary, max_bytes);
                return Some(Evidence {
                    source: "/proc/meminfo".to_string(),
                    data,
                    timestamp: "live".to_string(),
                    excerpted,
                });
            }
        }
        "disk" => {
            if let Ok(output) = std::process::Command::new("df").args(["-h"]).output() {
                if output.status.success() {
                    let content = String::from_utf8_lossy(&output.stdout).to_string();
                    let (data, excerpted) = excerpt_data(&content, max_bytes);
                    return Some(Evidence {
                        source: "df -h".to_string(),
                        data,
                        timestamp: "live".to_string(),
                        excerpted,
                    });
                }
            }
        }
        "kernel" => {
            if let Ok(output) = std::process::Command::new("uname").args(["-r"]).output() {
                if output.status.success() {
                    let content = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    return Some(Evidence {
                        source: "uname -r".to_string(),
                        data: content,
                        timestamp: "live".to_string(),
                        excerpted: false,
                    });
                }
            }
        }
        _ => {}
    }
    None
}

fn extract_relevant_sw_data(content: &str, targets: &[String]) -> String {
    // Simple extraction - look for target mentions in JSON
    // In a real implementation, this would parse JSON properly
    let mut relevant_lines = Vec::new();
    for line in content.lines() {
        let line_lower = line.to_lowercase();
        if targets.iter().any(|t| line_lower.contains(&t.to_lowercase())) {
            relevant_lines.push(line);
            if relevant_lines.len() > 50 {
                break;
            }
        }
    }
    if relevant_lines.is_empty() {
        // Return first 50 lines as context
        content.lines().take(50).collect::<Vec<_>>().join("\n")
    } else {
        relevant_lines.join("\n")
    }
}

fn excerpt_data(data: &str, max_bytes: usize) -> (String, bool) {
    if data.len() <= max_bytes {
        (data.to_string(), false)
    } else {
        let truncated = &data[..max_bytes.min(data.len())];
        // Find last newline to avoid cutting mid-line
        let end = truncated.rfind('\n').unwrap_or(truncated.len());
        (format!("{}...[EXCERPT truncated]", &truncated[..end]), true)
    }
}

fn get_file_timestamp(path: &str) -> String {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .map(|t| {
            let datetime: chrono::DateTime<chrono::Utc> = t.into();
            datetime.format("%Y-%m-%d %H:%M:%S UTC").to_string()
        })
        .unwrap_or_else(|_| "unknown".to_string())
}

// =============================================================================
// Action Plan Generation (v0.0.6)
// =============================================================================

/// Generate an action plan for action requests (no execution)
fn generate_action_plan(translator_output: &TranslatorOutput, request: &str) -> ActionPlan {
    let request_lower = request.to_lowercase();

    let mut steps = Vec::new();
    let mut affected_packages = Vec::new();
    let mut affected_services = Vec::new();
    let mut affected_files = Vec::new();
    let mut mutation_plan: Option<MutationPlan> = None;
    let mut is_medium_risk_executable = false;

    // Determine action type and build steps + mutation plan
    if request_lower.contains("install") {
        // Package install - NOT executable in v0.0.8
        for target in &translator_output.targets {
            steps.push(format!("1. Check if '{}' is already installed", target));
            steps.push(format!("2. Run: pacman -S {} (or yay -S {})", target, target));
            steps.push(format!("3. Verify installation: pacman -Qi {}", target));
            affected_packages.push(target.clone());
        }
    } else if request_lower.contains("remove") || request_lower.contains("uninstall") {
        // Package remove - NOT executable in v0.0.8
        for target in &translator_output.targets {
            steps.push(format!("1. Check dependencies of '{}'", target));
            steps.push(format!("2. Backup config: /etc/{}/", target));
            steps.push(format!("3. Run: pacman -Rs {}", target));
            affected_packages.push(target.clone());
            affected_files.push(format!("/etc/{}/", target));
        }
    } else if request_lower.contains("restart") {
        // Service restart - EXECUTABLE in v0.0.8
        let mut plan = MutationPlan::new();
        for target in &translator_output.targets {
            let service_name = ensure_service_suffix(target);
            steps.push(format!("1. Check current status of {}", service_name));
            steps.push(format!("2. Restart service: {}", service_name));
            steps.push(format!("3. Verify service is running"));
            affected_services.push(service_name.clone());

            // Add to mutation plan
            let request_id = generate_request_id();
            plan.mutations.push(MutationRequest {
                tool_name: "systemd_restart".to_string(),
                parameters: {
                    let mut p = std::collections::HashMap::new();
                    p.insert("service".to_string(), serde_json::json!(service_name));
                    p
                },
                confirmation_token: None, // Will be set after user confirmation
                evidence_ids: Vec::new(),
                request_id,
            });
        }
        plan.what_will_change = format!("Services to restart: {}", affected_services.join(", "));
        plan.why_required = "Service restart requested".to_string();
        plan.risk = MutationRisk::Medium;
        plan.rollback_plan = "Service state can be restored by restarting again".to_string();
        mutation_plan = Some(plan);
        is_medium_risk_executable = true;
    } else if request_lower.contains("reload") {
        // Service reload - EXECUTABLE in v0.0.8
        let mut plan = MutationPlan::new();
        for target in &translator_output.targets {
            let service_name = ensure_service_suffix(target);
            steps.push(format!("1. Reload configuration for {}", service_name));
            steps.push(format!("2. Verify service is still running"));
            affected_services.push(service_name.clone());

            let request_id = generate_request_id();
            plan.mutations.push(MutationRequest {
                tool_name: "systemd_reload".to_string(),
                parameters: {
                    let mut p = std::collections::HashMap::new();
                    p.insert("service".to_string(), serde_json::json!(service_name));
                    p
                },
                confirmation_token: None,
                evidence_ids: Vec::new(),
                request_id,
            });
        }
        plan.what_will_change = format!("Services to reload: {}", affected_services.join(", "));
        plan.why_required = "Service configuration reload requested".to_string();
        plan.risk = MutationRisk::Medium;
        plan.rollback_plan = "Configuration can be reloaded again if needed".to_string();
        mutation_plan = Some(plan);
        is_medium_risk_executable = true;
    } else if request_lower.contains("enable") && !request_lower.contains("disable") {
        // Service enable - EXECUTABLE in v0.0.8
        let mut plan = MutationPlan::new();
        for target in &translator_output.targets {
            let service_name = ensure_service_suffix(target);
            steps.push(format!("1. Enable and start service: {}", service_name));
            steps.push(format!("2. Verify service is enabled and running"));
            affected_services.push(service_name.clone());

            let request_id = generate_request_id();
            plan.mutations.push(MutationRequest {
                tool_name: "systemd_enable_now".to_string(),
                parameters: {
                    let mut p = std::collections::HashMap::new();
                    p.insert("service".to_string(), serde_json::json!(service_name));
                    p
                },
                confirmation_token: None,
                evidence_ids: Vec::new(),
                request_id,
            });
        }
        plan.what_will_change = format!("Services to enable: {}", affected_services.join(", "));
        plan.why_required = "Service enable requested".to_string();
        plan.risk = MutationRisk::Medium;
        plan.rollback_plan = "Run 'systemctl disable --now <service>' to revert".to_string();
        mutation_plan = Some(plan);
        is_medium_risk_executable = true;
    } else if request_lower.contains("disable") {
        // Service disable - EXECUTABLE in v0.0.8
        let mut plan = MutationPlan::new();
        for target in &translator_output.targets {
            let service_name = ensure_service_suffix(target);
            steps.push(format!("1. Disable and stop service: {}", service_name));
            steps.push(format!("2. Verify service is disabled and stopped"));
            affected_services.push(service_name.clone());

            let request_id = generate_request_id();
            plan.mutations.push(MutationRequest {
                tool_name: "systemd_disable_now".to_string(),
                parameters: {
                    let mut p = std::collections::HashMap::new();
                    p.insert("service".to_string(), serde_json::json!(service_name));
                    p
                },
                confirmation_token: None,
                evidence_ids: Vec::new(),
                request_id,
            });
        }
        plan.what_will_change = format!("Services to disable: {}", affected_services.join(", "));
        plan.why_required = "Service disable requested".to_string();
        plan.risk = MutationRisk::Medium;
        plan.rollback_plan = "Run 'systemctl enable --now <service>' to revert".to_string();
        mutation_plan = Some(plan);
        is_medium_risk_executable = true;
    } else if request_lower.contains("start") {
        // Service start only (no enable) - partial support in v0.0.8
        for target in &translator_output.targets {
            let service_name = ensure_service_suffix(target);
            steps.push(format!("1. Start service: {}", service_name));
            steps.push(format!("2. Verify service is running"));
            affected_services.push(service_name);
        }
        // Note: 'start' alone not in mutation catalog yet, use enable_now for now
    } else if request_lower.contains("stop") {
        // Service stop only - partial support in v0.0.8
        for target in &translator_output.targets {
            let service_name = ensure_service_suffix(target);
            steps.push(format!("1. Stop service: {}", service_name));
            steps.push(format!("2. Verify service is stopped"));
            affected_services.push(service_name);
        }
    } else if request_lower.contains("update") || request_lower.contains("upgrade") {
        // System update - NOT executable in v0.0.8
        steps.push("1. Sync package database: pacman -Sy".to_string());
        steps.push("2. Review available updates: pacman -Qu".to_string());
        steps.push("3. Apply updates: pacman -Syu".to_string());
        steps.push("4. Reboot if kernel updated".to_string());
        affected_packages.push("(system-wide)".to_string());
    } else {
        // Generic action
        steps.push("1. Verify current system state".to_string());
        steps.push("2. Execute requested action".to_string());
        steps.push("3. Verify changes applied".to_string());
    }

    // v0.0.8: Use new confirmation phrase format
    let confirmation_phrase = if is_medium_risk_executable {
        MEDIUM_RISK_CONFIRMATION.to_string()
    } else {
        match translator_output.risk {
            RiskLevel::HighRisk => "I assume the risk".to_string(),
            RiskLevel::MediumRisk => "yes".to_string(), // Non-executable medium risk uses old phrase
            RiskLevel::LowRisk => "y".to_string(),
            RiskLevel::ReadOnly => String::new(),
        }
    };

    let rollback_outline = if is_medium_risk_executable {
        format!(
            "Rollback plan:\n\
             - All operations logged to /var/lib/anna/rollback/logs/\n\
             - Service state changes can be reverted with systemctl\n\
             - Rollback instructions provided after execution"
        )
    } else {
        format!(
            "Rollback plan:\n\
             - Affected packages can be reinstalled/removed via pacman\n\
             - Service state can be reverted with systemctl\n\
             - Config backups stored in /var/lib/anna/rollback/ (future)\n\
             - Note: This action type not yet executable in v0.0.8"
        )
    };

    ActionPlan {
        steps,
        affected_files,
        affected_services,
        affected_packages,
        risk: translator_output.risk,
        confirmation_phrase,
        rollback_outline,
        mutation_plan,
        is_medium_risk_executable,
    }
}

/// Ensure service name has .service suffix
fn ensure_service_suffix(name: &str) -> String {
    if name.ends_with(".service") || name.ends_with(".socket") || name.ends_with(".timer") {
        name.to_string()
    } else {
        format!("{}.service", name)
    }
}

// =============================================================================
// Junior LLM Verification (v0.0.4 - real via Ollama)
// =============================================================================

/// System prompt for Junior verifier (v0.0.20: with source label enforcement)
const JUNIOR_SYSTEM_PROMPT: &str = r#"You are Junior, a verification assistant for Anna (a Linux system assistant).

Your job is to verify Anna's draft responses and provide a reliability score.

CRITICAL NO-GUESSING RULES:
1. EVERY machine-specific claim MUST cite evidence: [E1], [E2], etc.
2. Uncited machine facts = SPECULATION = must be removed or marked [UNVERIFIED]
3. Prefer "unknown" over guessing - this is ALWAYS better than speculation
4. Label any inference explicitly: "Based on [E2], likely..." not "The system is..."
5. Recommend REMOVING claims that lack evidence citations
6. For action requests: mutations NEVER execute without explicit confirmation

SOURCE LABEL ENFORCEMENT (v0.0.20):
Every claim in a response MUST be labeled with its source:
- [E#] for system evidence (measurements, snapshots, tool output)
- [K#] for knowledge pack documentation (man pages, package docs)
- (Reasoning) for general inference not from direct evidence

Examples:
   BAD: "To enable syntax highlighting, add 'syntax on' to .vimrc" (no source label)
   GOOD: "To enable syntax highlighting, add 'syntax on' to .vimrc [K1]"
   BAD: "Your CPU is running at high load" (no evidence citation)
   GOOD: "Your CPU is at 85% usage [E1]"
   BAD: "This is generally recommended"
   GOOD: "(Reasoning) This is generally recommended based on common practice"

LEARNING CLAIMS ENFORCEMENT (v0.0.13):
7. Claims about what Anna "learned", "remembers", "knows", or "has recipes for" MUST cite:
   - Memory evidence: [MEM#####]
   - Recipe evidence: [RCP#####]
8. Learning claims without MEM/RCP citations are FABRICATIONS and must be penalized heavily

POLICY ENFORCEMENT (v0.0.14):
9. Risky operations MUST cite policy evidence: [POL#####]
10. Refusals MUST explain which policy rule was applied
11. Plans that suggest bypassing confirmations or policy = DANGEROUS = max penalty
12. Protected packages/services/paths require policy citation to modify
13. Examples:
   BAD: "I'll install the linux kernel package for you"
   GOOD: "This package is blocked by policy [POL12345]: kernel modifications require manual intervention"
   BAD: "I can edit /etc/shadow"
   GOOD: "Path blocked by policy [POL67890]: /etc/shadow is in blocked paths list"

MUTATION SAFETY ENFORCEMENT (v0.0.16):
14. Mutation plans MUST include preflight checks summary before execution
15. File edits MUST show diff preview before confirmation is requested
16. Post-checks MUST be defined for every mutation (verify expected state)
17. Rollback MUST be defined and cited before execution proceeds
18. Plans lacking preflight, diff preview, or post-checks = INCOMPLETE = penalty
19. Examples:
   BAD: "I'll edit the config file for you" (no preflight, no diff)
   GOOD: "Preflight checks passed [PRE001]. Changes: +1 line. Backup: /var/lib/anna/rollback/..."
   BAD: "Changed the file successfully" (no post-check)
   GOOD: "Post-check verified [POST001]: config contains expected setting"

SECRETS LEAK PREVENTION (v0.0.18):
20. Responses MUST NOT reveal: passwords, tokens, API keys, bearer tokens, private keys, PEM blocks, SSH keys, cookies, cloud credentials, git credentials, database URLs, connection strings
21. Evidence from restricted paths MUST show [REDACTED:TYPE] or "Evidence exists but content is restricted by policy"
22. If secrets detected in response: force redaction with [REDACTED:TYPE], cite redaction ID, downscore heavily
23. Restricted paths (NEVER excerpt content): ~/.ssh/**, ~/.gnupg/**, /etc/shadow, /proc/*/environ, browser profiles, keyrings, password stores, AWS/Azure/GCP credential files
24. Examples:
   BAD: "Your API key is sk-abc123def456" (leaking secret)
   GOOD: "Your API key is [REDACTED:ApiKey]"
   BAD: "Contents of ~/.ssh/id_rsa: -----BEGIN RSA PRIVATE KEY-----..." (restricted path)
   GOOD: "Evidence exists but content is restricted by policy [E-restrict-12345]"
   BAD: "Found password in config: mySecretPass123" (leaking password)
   GOOD: "Found password in config: [REDACTED:Password]"

OUTPUT FORMAT (follow exactly):
SCORE: [0-100]
CRITIQUE: [List uncited claims, source label violations, policy violations, and speculation, 1-2 sentences]
UNCITED_CLAIMS: [comma-separated list of claims without [E#], [K#], (Reasoning), [MEM#], [RCP#], or [POL#] labels, or "none"]
SUGGESTIONS: [Minimal edits: add source labels, remove uncited claims, or add [UNVERIFIED] tags]
MUTATION_WARNING: [yes/no - only "yes" if this is an action request]

Scoring rubric:
- All claims cite evidence [E#] or [K#]: +40
- No speculation or uncited claims: +30
- Read-only operation: +20
- High confidence from Translator: +10
PENALTIES:
- Each uncited machine-specific claim: -15
- Factual claim without [E#], [K#], or (Reasoning) label: -10 (UNLABELED SOURCE)
- Speculation presented as fact: -20
- Missing evidence for key claims: -30
- Action request without plan: -10
- Learning claim without MEM/RCP citation: -25 (FABRICATION)
- Risky operation without policy citation: -20
- Suggesting policy bypass or confirmation skip: -50 (DANGEROUS)
- Mutation without preflight checks: -25 (INCOMPLETE)
- File edit without diff preview: -20 (INCOMPLETE)
- Mutation without post-check definition: -25 (INCOMPLETE)
- Mutation without rollback plan: -30 (UNSAFE)
- Secret revealed in response (password, key, token): -50 (SECURITY LEAK)
- Unredacted content from restricted path: -40 (RESTRICTED PATH VIOLATION)
- Missing [REDACTED:TYPE] for detected secret: -30 (INCOMPLETE REDACTION)

EXAMPLES OF GOOD vs BAD:
BAD: "Your CPU is an AMD Ryzen" (no citation)
GOOD: "Your CPU is an AMD Ryzen 7 5800X [E1]"
BAD: "The service is probably running"
GOOD: "nginx status is active [E2]" OR "nginx status unknown (no evidence)"
BAD: "I've learned how to handle this" (learning claim without citation)
GOOD: "Based on recipe [RCP12345], I can handle this"
BAD: "I can modify the kernel package" (policy violation)
GOOD: "Kernel packages are blocked by policy [POL54321]""#;

/// Call Junior LLM for verification (v0.0.7: with Evidence IDs)
/// v0.0.18: Evidence is redacted for secrets before sending to LLM
async fn call_junior_llm(
    client: &OllamaClient,
    model: &str,
    request: &str,
    translator_output: &TranslatorOutput,
    tool_results: &[ToolResult],
    draft_answer: &str,
) -> Result<JuniorVerification, OllamaError> {
    // Build evidence text with Evidence IDs (v0.0.18: redacted for secrets)
    let evidence_text = if tool_results.is_empty() {
        "No evidence available.".to_string()
    } else {
        tool_results
            .iter()
            .map(|r| {
                let status = if r.success { "OK" } else { "FAILED" };
                // v0.0.18: Redact secrets from evidence summaries
                let redacted_summary = redact_evidence(&r.human_summary, None)
                    .unwrap_or_else(|e| e);
                format!("[{}] {} ({}): {}", r.evidence_id, r.tool_name, status, redacted_summary)
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let translator_source = if translator_output.llm_backed { "LLM" } else { "deterministic fallback" };

    // v0.0.18: Redact draft answer before sending to Junior
    let redacted_draft = redact_transcript(draft_answer);

    let prompt = format!(
        r#"Verify this response. Claims about the machine MUST cite evidence IDs like [E1], [E2].

USER REQUEST: {}

TRANSLATOR ANALYSIS (via {}):
- Intent: {}
- Targets: {}
- Risk: {}
- Confidence: {}%

EVIDENCE (cite by ID in your review):
{}

ANNA'S DRAFT ANSWER:
{}

Check that all machine-specific claims cite an evidence ID. Uncited claims = speculation.
Provide your verification in the exact format specified."#,
        request,
        translator_source,
        translator_output.intent_type,
        if translator_output.targets.is_empty() { "(none)".to_string() } else { translator_output.targets.join(", ") },
        translator_output.risk,
        translator_output.confidence,
        evidence_text,
        redacted_draft
    );

    let response = client
        .generate(model, &prompt, Some(JUNIOR_SYSTEM_PROMPT))
        .await?;

    Ok(parse_junior_response(&response.response, translator_output))
}

/// Call Junior LLM for verification (legacy: with Evidence)
/// v0.0.18: Evidence and draft answers are redacted for secrets
async fn call_junior_llm_legacy(
    client: &OllamaClient,
    model: &str,
    request: &str,
    translator_output: &TranslatorOutput,
    evidence: &[Evidence],
    draft_answer: &str,
) -> Result<JuniorVerification, OllamaError> {
    // Build evidence text with excerpting indication (v0.0.18: redacted for secrets)
    let evidence_text = if evidence.is_empty() {
        "No evidence available.".to_string()
    } else {
        evidence
            .iter()
            .enumerate()
            .map(|(i, e)| {
                let excerpt_note = if e.excerpted { " [EXCERPT]" } else { "" };
                // v0.0.18: Redact secrets from evidence data
                let redacted_data = redact_evidence(&e.data, Some(&e.source))
                    .unwrap_or_else(|e| e);
                format!("[E{}] {}{}: {}", i + 1, e.source, excerpt_note, redacted_data)
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let translator_source = if translator_output.llm_backed { "LLM" } else { "deterministic fallback" };

    // v0.0.18: Redact draft answer before sending to Junior
    let redacted_draft = redact_transcript(draft_answer);

    let prompt = format!(
        r#"Verify this response. Claims about the machine MUST cite evidence IDs like [E1], [E2].

USER REQUEST: {}

TRANSLATOR ANALYSIS (via {}):
- Intent: {}
- Targets: {}
- Risk: {}
- Confidence: {}%

EVIDENCE (cite by ID in your review):
{}

ANNA'S DRAFT ANSWER:
{}

Check that all machine-specific claims cite an evidence ID. Uncited claims = speculation.
Provide your verification in the exact format specified."#,
        request,
        translator_source,
        translator_output.intent_type,
        if translator_output.targets.is_empty() { "(none)".to_string() } else { translator_output.targets.join(", ") },
        translator_output.risk,
        translator_output.confidence,
        evidence_text,
        redacted_draft
    );

    let response = client
        .generate(model, &prompt, Some(JUNIOR_SYSTEM_PROMPT))
        .await?;

    Ok(parse_junior_response(&response.response, translator_output))
}

/// Parse Junior's LLM response into structured verification (v0.0.7: with uncited claims)
fn parse_junior_response(response: &str, translator_output: &TranslatorOutput) -> JuniorVerification {
    let mut verification = JuniorVerification::default();

    // Parse SCORE
    if let Some(score_line) = response.lines().find(|l| l.trim().starts_with("SCORE:")) {
        if let Some(score_str) = score_line.strip_prefix("SCORE:").or_else(|| score_line.trim().strip_prefix("SCORE:")) {
            if let Ok(score) = score_str.trim().parse::<u8>() {
                verification.score = score.min(100);
            }
        }
    }

    // Parse CRITIQUE
    if let Some(critique_line) = response.lines().find(|l| l.trim().starts_with("CRITIQUE:")) {
        if let Some(critique) = critique_line.trim().strip_prefix("CRITIQUE:") {
            verification.critique = critique.trim().to_string();
        }
    }

    // Parse UNCITED_CLAIMS (v0.0.7)
    if let Some(uncited_line) = response.lines().find(|l| l.trim().starts_with("UNCITED_CLAIMS:")) {
        if let Some(uncited) = uncited_line.trim().strip_prefix("UNCITED_CLAIMS:") {
            let uncited = uncited.trim();
            if uncited != "none" && !uncited.is_empty() {
                verification.uncited_claims = uncited
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
    }

    // Parse SUGGESTIONS
    if let Some(suggestions_line) = response.lines().find(|l| l.trim().starts_with("SUGGESTIONS:")) {
        if let Some(suggestions) = suggestions_line.trim().strip_prefix("SUGGESTIONS:") {
            verification.suggestions = suggestions.trim().to_string();
        }
    }

    // Parse MUTATION_WARNING
    if let Some(warning_line) = response.lines().find(|l| l.trim().starts_with("MUTATION_WARNING:")) {
        if let Some(warning) = warning_line.trim().strip_prefix("MUTATION_WARNING:") {
            verification.mutation_warning = warning.trim().eq_ignore_ascii_case("yes");
        }
    } else if translator_output.intent_type == IntentType::ActionRequest {
        verification.mutation_warning = true;
    }

    // If score is still 0 and we got a response, try to extract from text
    if verification.score == 0 && !response.is_empty() {
        for word in response.split_whitespace() {
            if let Ok(num) = word.trim_matches(|c: char| !c.is_numeric()).parse::<u8>() {
                if num <= 100 && num > 0 {
                    verification.score = num;
                    break;
                }
            }
        }
    }

    verification
}

/// Detect uncited learning claims in text (v0.0.13)
/// Returns list of claims that mention learning/memory/recipes without proper citations
fn detect_uncited_learning_claims(text: &str) -> Vec<String> {
    let mut uncited = Vec::new();

    // Learning claim patterns
    let learning_patterns = [
        "i learned", "i've learned", "i have learned",
        "i remember", "i've remembered", "i recalled",
        "i know how", "i know that", "i know about",
        "i have a recipe", "i've got a recipe",
        "my recipe", "my memory", "my knowledge",
        "based on what i learned", "from what i remember",
        "as i recall", "as i remember",
    ];

    let text_lower = text.to_lowercase();

    // Check for MEM/RCP citations
    let has_mem_citation = text.contains("[MEM") || text.contains("MEM#");
    let has_rcp_citation = text.contains("[RCP") || text.contains("RCP#");

    for pattern in &learning_patterns {
        if text_lower.contains(pattern) {
            // Check if there's a citation nearby
            if !has_mem_citation && !has_rcp_citation {
                uncited.push(format!("Learning claim '{}...' without MEM/RCP citation",
                    &text_lower[text_lower.find(pattern).unwrap_or(0)..]
                        .chars()
                        .take(40)
                        .collect::<String>()));
            }
        }
    }

    uncited
}

/// Apply learning claim enforcement to Junior verification (v0.0.13)
/// Adds penalty for uncited learning claims in the final response
fn enforce_learning_claims(
    mut verification: JuniorVerification,
    final_response: &str,
) -> JuniorVerification {
    let uncited_learning = detect_uncited_learning_claims(final_response);

    if !uncited_learning.is_empty() {
        // Apply penalty: -25 per learning claim (max -50)
        let penalty = (uncited_learning.len() as u8 * 25).min(50);
        verification.score = verification.score.saturating_sub(penalty);

        // Add to uncited claims
        verification.uncited_claims.extend(uncited_learning);

        // Update critique
        if !verification.critique.is_empty() {
            verification.critique.push_str("; ");
        }
        verification.critique.push_str("LEARNING CLAIMS FABRICATION DETECTED");

        // Update suggestions
        if !verification.suggestions.is_empty() {
            verification.suggestions.push_str("; ");
        }
        verification.suggestions.push_str(
            "Remove learning claims without [MEM#####] or [RCP#####] citations"
        );
    }

    verification
}

/// Fallback scoring when Junior LLM is unavailable
fn fallback_junior_score(translator_output: &TranslatorOutput, evidence: &[Evidence]) -> JuniorVerification {
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
    if translator_output.confidence > 70 {
        score += 30;
        breakdown_parts.push(format!("+30 confident ({}%)", translator_output.confidence));
    } else {
        breakdown_parts.push(format!("+0 low confidence ({}%)", translator_output.confidence));
    }

    // +20: observational + cited (read-only with evidence)
    if translator_output.risk == RiskLevel::ReadOnly && !evidence.is_empty() {
        score += 20;
        breakdown_parts.push("+20 observational+cited".to_string());
    }

    // +10: read-only operation
    if translator_output.risk == RiskLevel::ReadOnly {
        score += 10;
        breakdown_parts.push("+10 read-only".to_string());
    }

    // -10: LLM-backed translator not available
    if !translator_output.llm_backed {
        score = score.saturating_sub(10);
        breakdown_parts.push("-10 translator fallback".to_string());
    }

    JuniorVerification {
        score,
        critique: format!("(fallback scoring: {})", breakdown_parts.join(", ")),
        uncited_claims: Vec::new(), // Fallback can't detect uncited claims
        suggestions: "Junior LLM unavailable - using deterministic scoring".to_string(),
        mutation_warning: translator_output.intent_type == IntentType::ActionRequest,
    }
}

/// Fallback scoring with ToolResults (v0.0.7)
fn fallback_junior_score_v2(translator_output: &TranslatorOutput, tool_results: &[ToolResult]) -> JuniorVerification {
    let mut score: u8 = 0;
    let mut breakdown_parts = Vec::new();

    // +40: evidence exists
    let successful_results: Vec<_> = tool_results.iter().filter(|r| r.success).collect();
    if !successful_results.is_empty() {
        score += 40;
        breakdown_parts.push(format!("+40 evidence ({} tools)", successful_results.len()));
    } else {
        breakdown_parts.push("+0 no evidence".to_string());
    }

    // +30: confident classification (>70%)
    if translator_output.confidence > 70 {
        score += 30;
        breakdown_parts.push(format!("+30 confident ({}%)", translator_output.confidence));
    } else {
        breakdown_parts.push(format!("+0 low confidence ({}%)", translator_output.confidence));
    }

    // +20: observational + cited (read-only with evidence)
    if translator_output.risk == RiskLevel::ReadOnly && !successful_results.is_empty() {
        score += 20;
        breakdown_parts.push("+20 observational+cited".to_string());
    }

    // +10: read-only operation
    if translator_output.risk == RiskLevel::ReadOnly {
        score += 10;
        breakdown_parts.push("+10 read-only".to_string());
    }

    // -10: LLM-backed translator not available
    if !translator_output.llm_backed {
        score = score.saturating_sub(10);
        breakdown_parts.push("-10 translator fallback".to_string());
    }

    JuniorVerification {
        score,
        critique: format!("(fallback scoring: {})", breakdown_parts.join(", ")),
        uncited_claims: Vec::new(), // Fallback can't detect uncited claims
        suggestions: "Junior LLM unavailable - using deterministic scoring".to_string(),
        mutation_warning: translator_output.intent_type == IntentType::ActionRequest,
    }
}

// =============================================================================
// Spinner helpers
// =============================================================================

fn show_spinner(message: &str) {
    print!("  {} ", message.dimmed());
    io::stdout().flush().ok();
}

fn clear_spinner() {
    print!("\r\x1b[K");
    io::stdout().flush().ok();
}

// =============================================================================
// LLM Availability
// =============================================================================

/// LLM availability result
#[derive(Debug)]
pub enum LlmAvailability {
    Ready(OllamaClient, String),
    NotReady(LlmNotReadyReason),
}

#[derive(Debug, Clone)]
pub enum LlmNotReadyReason {
    Disabled,
    OllamaUnavailable,
    Pulling { model: String, percent: f64, eta_secs: Option<u64>, speed: Option<f64> },
    Benchmarking,
    Error(String),
    NoModel,
}

impl LlmNotReadyReason {
    pub fn format(&self) -> String {
        match self {
            LlmNotReadyReason::Disabled => "LLM disabled in config".to_string(),
            LlmNotReadyReason::OllamaUnavailable => "Ollama not available".to_string(),
            LlmNotReadyReason::Pulling { model, percent, eta_secs, speed } => {
                let speed_str = speed.map(|s| {
                    if s >= 1024.0 * 1024.0 {
                        format!(" @ {:.1} MB/s", s / (1024.0 * 1024.0))
                    } else if s >= 1024.0 {
                        format!(" @ {:.1} KB/s", s / 1024.0)
                    } else {
                        String::new()
                    }
                }).unwrap_or_default();
                let eta_str = eta_secs.map(|s| {
                    if s < 60 { format!(" ETA {}s", s) }
                    else if s < 3600 { format!(" ETA {}m {}s", s / 60, s % 60) }
                    else { format!(" ETA {}h {}m", s / 3600, (s % 3600) / 60) }
                }).unwrap_or_else(|| " ETA calculating...".to_string());
                format!("Pulling '{}': {:.1}%{}{}", model, percent, speed_str, eta_str)
            }
            LlmNotReadyReason::Benchmarking => "Benchmarking models...".to_string(),
            LlmNotReadyReason::Error(e) => format!("LLM error: {}", e),
            LlmNotReadyReason::NoModel => "No suitable model".to_string(),
        }
    }
}

/// Check Translator availability
async fn get_translator_client() -> LlmAvailability {
    let config = AnnaConfig::load();

    if !config.llm.enabled || !config.llm.translator.enabled {
        return LlmAvailability::NotReady(LlmNotReadyReason::Disabled);
    }

    // Check bootstrap state
    if let Some(snapshot) = StatusSnapshot::load() {
        match snapshot.llm_bootstrap_phase.as_deref() {
            Some("pulling_models") => {
                return LlmAvailability::NotReady(LlmNotReadyReason::Pulling {
                    model: snapshot.llm_downloading_model.unwrap_or_else(|| "unknown".to_string()),
                    percent: snapshot.llm_download_percent.unwrap_or(0.0),
                    eta_secs: snapshot.llm_download_eta_secs,
                    speed: snapshot.llm_download_speed,
                });
            }
            Some("benchmarking") => {
                return LlmAvailability::NotReady(LlmNotReadyReason::Benchmarking);
            }
            Some("error") => {
                return LlmAvailability::NotReady(LlmNotReadyReason::Error(
                    snapshot.llm_error.unwrap_or_else(|| "unknown".to_string())
                ));
            }
            _ => {}
        }
    }

    let client = OllamaClient::with_url(&config.llm.ollama_url)
        .with_timeout(config.llm.translator.timeout_ms);

    if !client.is_available().await {
        return LlmAvailability::NotReady(LlmNotReadyReason::OllamaUnavailable);
    }

    // Get model - prefer config, then snapshot
    let model = if !config.llm.translator.model.is_empty() {
        config.llm.translator.model.clone()
    } else if let Some(snapshot) = StatusSnapshot::load() {
        snapshot.llm_translator_model.unwrap_or_default()
    } else {
        String::new()
    };

    if model.is_empty() {
        // Auto-select smallest model for translator
        match client.list_models().await {
            Ok(models) => {
                let names: Vec<String> = models.iter().map(|m| m.name.clone()).collect();
                // Prefer small fast models for translator - exact match or closest match
                let preferred = ["qwen2.5:0.5b", "qwen2.5:1.5b", "phi3:mini", "gemma2:2b", "llama3.2:1b"];
                for pref in preferred {
                    // First try exact match
                    if let Some(exact) = names.iter().find(|n| *n == pref) {
                        return LlmAvailability::Ready(client, exact.clone());
                    }
                    // Then try prefix match (e.g., qwen2.5:1.5b-instruct matches qwen2.5:1.5b)
                    if let Some(match_name) = names.iter().find(|n| n.starts_with(pref)) {
                        return LlmAvailability::Ready(client, match_name.clone());
                    }
                }
                // Fall back to first available model
                if let Some(first) = names.first() {
                    return LlmAvailability::Ready(client, first.clone());
                }
                return LlmAvailability::NotReady(LlmNotReadyReason::NoModel);
            }
            Err(_) => return LlmAvailability::NotReady(LlmNotReadyReason::NoModel),
        }
    }

    match client.has_model(&model).await {
        Ok(true) => LlmAvailability::Ready(client, model),
        _ => LlmAvailability::NotReady(LlmNotReadyReason::NoModel),
    }
}

/// Check Junior availability
async fn get_junior_client() -> LlmAvailability {
    let config = AnnaConfig::load();

    if !config.llm.enabled || !config.llm.junior.enabled {
        return LlmAvailability::NotReady(LlmNotReadyReason::Disabled);
    }

    if let Some(snapshot) = StatusSnapshot::load() {
        match snapshot.llm_bootstrap_phase.as_deref() {
            Some("pulling_models") => {
                return LlmAvailability::NotReady(LlmNotReadyReason::Pulling {
                    model: snapshot.llm_downloading_model.unwrap_or_else(|| "unknown".to_string()),
                    percent: snapshot.llm_download_percent.unwrap_or(0.0),
                    eta_secs: snapshot.llm_download_eta_secs,
                    speed: snapshot.llm_download_speed,
                });
            }
            Some("benchmarking") => {
                return LlmAvailability::NotReady(LlmNotReadyReason::Benchmarking);
            }
            Some("error") => {
                return LlmAvailability::NotReady(LlmNotReadyReason::Error(
                    snapshot.llm_error.unwrap_or_else(|| "unknown".to_string())
                ));
            }
            _ => {}
        }
    }

    let client = OllamaClient::with_url(&config.llm.ollama_url)
        .with_timeout(config.llm.junior.timeout_ms);

    if !client.is_available().await {
        return LlmAvailability::NotReady(LlmNotReadyReason::OllamaUnavailable);
    }

    let model = if !config.llm.junior.model.is_empty() {
        config.llm.junior.model.clone()
    } else if let Some(snapshot) = StatusSnapshot::load() {
        snapshot.llm_junior_model.unwrap_or_default()
    } else {
        String::new()
    };

    if model.is_empty() {
        match client.list_models().await {
            Ok(models) => {
                let names: Vec<String> = models.iter().map(|m| m.name.clone()).collect();
                match select_junior_model(&names) {
                    Some(m) => return LlmAvailability::Ready(client, m),
                    None => return LlmAvailability::NotReady(LlmNotReadyReason::NoModel),
                }
            }
            Err(_) => return LlmAvailability::NotReady(LlmNotReadyReason::NoModel),
        }
    }

    match client.has_model(&model).await {
        Ok(true) => LlmAvailability::Ready(client, model),
        _ => LlmAvailability::NotReady(LlmNotReadyReason::NoModel),
    }
}

// =============================================================================
// Clarification Loop (v0.0.6)
// =============================================================================

/// Present clarification question and get user choice
fn ask_clarification(clarification: &Clarification) -> usize {
    println!();
    println!("  {}", "Clarification needed:".cyan());
    println!("  {}", clarification.question);
    println!();

    for (i, option) in clarification.options.iter().enumerate() {
        let marker = if i == clarification.default_option { "*" } else { " " };
        println!("  {}[{}] {}", marker, i + 1, option);
    }

    println!();
    println!("  {} [default: {}]", "Enter choice (1-N):".dimmed(), clarification.default_option + 1);
    print!("  > ");
    io::stdout().flush().ok();

    // Read user input
    let stdin = io::stdin();
    let mut input = String::new();

    if stdin.lock().read_line(&mut input).is_ok() {
        let input = input.trim();
        if input.is_empty() {
            return clarification.default_option;
        }
        if let Ok(n) = input.parse::<usize>() {
            if n > 0 && n <= clarification.options.len() {
                return n - 1;
            }
        }
    }

    // Default on invalid input
    clarification.default_option
}

/// Apply clarification choice to targets
fn apply_clarification(translator_output: &mut TranslatorOutput, choice_idx: usize) {
    if let Some(ref clarification) = translator_output.clarification {
        if choice_idx < clarification.options.len() {
            let chosen = &clarification.options[choice_idx];
            // Replace/add the chosen target
            if !translator_output.targets.contains(chosen) {
                translator_output.targets.clear();
                translator_output.targets.push(chosen.clone());
            }
        }
    }
    // Clear clarification after applying
    translator_output.clarification = None;
}

// =============================================================================
// v0.0.17: Target User Selection
// =============================================================================

use std::sync::Mutex;
use once_cell::sync::Lazy;

/// Global target user selector (for REPL session continuity)
static TARGET_USER_SELECTOR: Lazy<Mutex<TargetUserSelector>> = Lazy::new(|| {
    Mutex::new(TargetUserSelector::new())
});

/// Select the target user, with clarification prompt if ambiguous
fn select_target_user() -> Option<TargetUserSelection> {
    let mut selector = TARGET_USER_SELECTOR.lock().ok()?;

    match selector.select_target_user() {
        SelectionResult::Determined(selection) => Some(selection),
        SelectionResult::NeedsClarification(ambiguous) => {
            // Ask user to select
            dialogue_always(Actor::Anna, Actor::You, &ambiguous.format_prompt());

            // Read user input
            let stdin = io::stdin();
            let mut input = String::new();
            print!("  > ");
            io::stdout().flush().ok();

            if stdin.lock().read_line(&mut input).is_ok() {
                let input = input.trim();
                if let Ok(choice) = input.parse::<usize>() {
                    if let Some(selection) = ambiguous.resolve(choice) {
                        // Store for session
                        selector.set_session_user(&selection.user.username);
                        return Some(selection);
                    }
                }
            }

            // Default to first candidate if invalid input
            ambiguous.resolve(1)
        }
    }
}

/// Check if a request involves user home directory changes
fn request_involves_user_home(request: &str) -> bool {
    let home_keywords = [
        "~", "$HOME", ".bashrc", ".zshrc", ".config", ".local",
        "dotfile", "profile", "shell config", "vim", "nvim", "neovim",
        "syntax highlight", "theme", "colorscheme", "terminal",
    ];

    let lower = request.to_lowercase();
    home_keywords.iter().any(|k| lower.contains(&k.to_lowercase()))
}

// =============================================================================
// v0.0.21: TTFO (Time to First Output) and Performance Tracking
// =============================================================================

/// Print fast header within 150ms target (TTFO improvement)
fn print_fast_header(request: &str) {
    println!();
    // Header line - immediate output
    let header = format!("[{}] to [{}]:", Actor::You, Actor::Anna);
    println!("  {}", header.dimmed());
    // Show request immediately
    for line in request.lines() {
        println!("  {}", line);
    }
    println!();
    // Working indicator - shows we're processing
    print!("  {} ", "I'm starting analysis and gathering evidence...".dimmed());
    let _ = io::stdout().flush();
}

/// Clear the working indicator line
fn clear_working_indicator() {
    // Move cursor back and clear line
    print!("\r  {}\r", " ".repeat(60));
    let _ = io::stdout().flush();
}

// =============================================================================
// Main Pipeline (v0.0.21: with TTFO, caching, and token budgets)
// =============================================================================

/// Process a request through the full pipeline with dialogue transcript
pub async fn process(request: &str) {
    // v0.0.21: Track latency from start
    let start_time = std::time::Instant::now();
    let mut translator_ms: u64 = 0;
    let mut junior_ms: u64 = 0;
    let mut tools_ms: u64 = 0;
    let mut cache_hit = false;

    // v0.0.31: Record request start for reliability metrics
    let mut metrics = MetricsStore::load();
    metrics.record(MetricType::RequestStart);

    // v0.0.44: Create case file for logging all requests
    let case_id = generate_case_id();
    let mut case_file = CaseFile::new(&case_id, request);

    // v0.0.21: TTFO - print header and working indicator within 150ms
    print_fast_header(request);

    // Initialize tool catalog
    let catalog = ToolCatalog::new();

    // v0.0.21: Initialize caches
    let tool_cache = ToolCache::new();
    let llm_cache = LlmCache::new();
    let snapshot_hash = get_snapshot_hash();
    let policy_version = get_policy_version();
    let config = AnnaConfig::load();

    // v0.0.17: Check if we need target user awareness for this request
    let target_user = if request_involves_user_home(request) {
        match select_target_user() {
            Some(selection) => {
                // Clear working indicator before dialogue
                clear_working_indicator();
                // Show target user in transcript
                dialogue(Actor::Anna, Actor::You, &selection.format_transcript());
                println!();
                Some(selection)
            }
            None => None,
        }
    } else {
        None
    };

    // Clear working indicator - we're about to show real progress
    clear_working_indicator();

    // [you] to [anna]: user's request (already shown in fast header, skip in normal flow)
    // dialogue(Actor::You, Actor::Anna, request); // Shown in print_fast_header
    // println!();

    // [anna] to [translator]: request for classification
    dialogue(
        Actor::Anna,
        Actor::Translator,
        &format!("Classify this request:\n\"{}\"", request),
    );
    println!();

    // Try real Translator LLM, fall back to deterministic
    // v0.0.31: Track translator LLM metrics
    metrics.record(MetricType::TranslatorStart);
    let translator_start = std::time::Instant::now();
    let (mut translator_output, translator_reason) = match get_translator_client().await {
        LlmAvailability::Ready(client, model) => {
            show_spinner(&format!("[translator analyzing via {}...]", model));

            match call_translator_llm(&client, &model, request).await {
                Ok(output) => {
                    clear_spinner();
                    metrics.record(MetricType::TranslatorSuccess);
                    (output, None)
                }
                Err(e) => {
                    clear_spinner();
                    // Record as timeout if it looks like a timeout, otherwise just not record success
                    let err_str = e.to_string().to_lowercase();
                    if err_str.contains("timeout") || err_str.contains("timed out") {
                        metrics.record(MetricType::TranslatorTimeout);
                    }
                    println!("  {} {} - using deterministic fallback", "Translator LLM error:".yellow(), e);
                    (translator_classify_deterministic(request), Some(format!("LLM error: {}", e)))
                }
            }
        }
        LlmAvailability::NotReady(reason) => {
            let reason_str = reason.format();
            println!("  {} {} - using deterministic fallback", "Translator:".cyan(), reason_str.dimmed());
            (translator_classify_deterministic(request), Some(reason_str))
        }
    };
    translator_ms = translator_start.elapsed().as_millis() as u64;
    metrics.record_latency("translator", translator_ms);

    // [translator] to [anna]: classification result (v0.0.7: include tool plan)
    let tool_plan_str = translator_output.tool_plan.as_ref()
        .map(|p| format!("\nTools: {}", p.tools.iter().map(|t| t.tool_name.as_str()).collect::<Vec<_>>().join(", ")))
        .unwrap_or_default();
    let translator_response = format!(
        "Intent: {}\nTargets: {}\nRisk: {}{}\nConfidence: {}%{}{}",
        translator_output.intent_type,
        if translator_output.targets.is_empty() { "(none)".to_string() } else { translator_output.targets.join(", ") },
        translator_output.risk,
        tool_plan_str,
        translator_output.confidence,
        if translator_output.llm_backed { "" } else { "\n[deterministic fallback]" },
        if translator_output.clarification.is_some() { "\n[clarification requested]" } else { "" }
    );
    dialogue(Actor::Translator, Actor::Anna, &translator_response);
    println!();

    // Handle clarification if requested
    if let Some(ref clarification) = translator_output.clarification.clone() {
        // [anna] to [you]: clarification question
        let clarification_msg = format!(
            "I need to clarify before proceeding:\n{}\n\nOptions:\n{}",
            clarification.question,
            clarification.options.iter().enumerate()
                .map(|(i, o)| format!("  [{}] {}{}", i + 1, o, if i == clarification.default_option { " (default)" } else { "" }))
                .collect::<Vec<_>>()
                .join("\n")
        );
        dialogue(Actor::Anna, Actor::You, &clarification_msg);
        println!();

        let choice = ask_clarification(clarification);
        apply_clarification(&mut translator_output, choice);

        // [you] to [anna]: clarification answer
        let answer = translator_output.targets.first().cloned().unwrap_or_else(|| format!("option {}", choice + 1));
        dialogue(Actor::You, Actor::Anna, &format!("Selected: {}", answer));
        println!();
    }

    // v0.0.44: Doctor Registry integration for FixIt mode
    let doctor_selection: Option<DoctorSelection> = if translator_output.intent_type == IntentType::FixIt {
        match DoctorRegistry::load() {
            Ok(registry) => {
                let intent_tags: Vec<String> = translator_output.targets.clone();
                if let Some(selection) = registry.select_doctors(request, &intent_tags) {
                    // Show doctor selection in transcript
                    dialogue(
                        Actor::Anna,
                        Actor::Translator,
                        &format!(
                            "Routing to doctor: {} ({})\nReason: {}",
                            selection.primary.doctor_name,
                            selection.primary.doctor_id,
                            selection.reasoning
                        ),
                    );
                    println!();
                    Some(selection)
                } else {
                    // No specific doctor matched - fallback to general evidence gathering
                    dialogue(
                        Actor::Anna,
                        Actor::Translator,
                        "No specific doctor matched - using general troubleshooting approach",
                    );
                    println!();
                    None
                }
            }
            Err(e) => {
                tracing::warn!("Failed to load doctor registry: {}", e);
                None
            }
        }
    } else {
        None
    };

    // Silence unused variable warning - doctor_selection is used for tracking
    let _ = &doctor_selection;

    // v0.0.7: Execute tools and gather evidence with Evidence IDs
    let (tool_results, evidence) = if translator_output.intent_type == IntentType::SystemQuery
        || translator_output.intent_type == IntentType::ActionRequest
        || translator_output.intent_type == IntentType::FixIt
        || (translator_output.intent_type == IntentType::Question && !translator_output.targets.is_empty())
    {
        // Use tool_plan if available, otherwise fall back to legacy evidence retrieval
        if let Some(ref original_plan) = translator_output.tool_plan {
            // v0.0.46: Apply tool sanity gate to ensure domain-specific tools are used
            let plan = if let Some(fixed_plan) = apply_tool_sanity_gate(&translator_output, &translator_output.tool_plan) {
                println!("  {} Tool sanity gate applied: using domain-specific tools", "[v0.0.46]".cyan());
                fixed_plan
            } else {
                original_plan.clone()
            };

            // Natural language: Anna asks annad to gather evidence
            let tool_names: Vec<_> = plan.tools.iter().map(|t| t.tool_name.as_str()).collect();
            let human_request = format!(
                "Please gather evidence using: {}",
                tool_names.join(", ")
            );
            dialogue(Actor::Anna, Actor::Annad, &human_request);
            println!();

            // Execute tools
            // v0.0.31: Track tool execution metrics
            let tools_start = std::time::Instant::now();
            let mut collector = EvidenceCollector::new();
            let results = execute_tool_plan(&plan.tools, &catalog, &mut collector);

            // v0.0.31: Record tool success/failure metrics
            for r in &results {
                metrics.record(MetricType::ToolStart);
                if r.success {
                    metrics.record(MetricType::ToolSuccess);
                } else {
                    metrics.record(MetricType::ToolFailure);
                }
            }
            tools_ms = tools_start.elapsed().as_millis() as u64;
            metrics.record_latency("tools", tools_ms);

            // Natural language: annad reports what was found
            let evidence_summary = if results.is_empty() {
                "No evidence gathered - tools returned no results.".to_string()
            } else {
                results.iter()
                    .map(|r| {
                        let status = if r.success { "found" } else { "failed" };
                        format!("[{}] {}: {} ({})", r.evidence_id, r.tool_name, r.human_summary, status)
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            dialogue(Actor::Annad, Actor::Anna, &evidence_summary);
            println!();

            (results, Vec::new())
        } else {
            // Legacy path: use old evidence retrieval
            dialogue(
                Actor::Anna,
                Actor::Annad,
                &format!(
                    "Retrieve evidence for: {}",
                    if translator_output.targets.is_empty() { "(general query)".to_string() } else { translator_output.targets.join(", ") }
                ),
            );
            println!();

            let ev = retrieve_evidence(&translator_output);

            let evidence_summary = if ev.is_empty() {
                "No evidence found in snapshots.".to_string()
            } else {
                ev.iter()
                    .enumerate()
                    .map(|(i, e)| {
                        let excerpt_note = if e.excerpted { " [EXCERPT]" } else { "" };
                        let data_preview = if e.data.len() > 200 {
                            format!("{}...", &e.data[..200])
                        } else {
                            e.data.clone()
                        };
                        format!("[E{}] {}{} ({}): {}", i + 1, e.source, excerpt_note, e.timestamp, data_preview)
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            dialogue(Actor::Annad, Actor::Anna, &evidence_summary);
            println!();

            (Vec::new(), ev)
        }
    } else {
        (Vec::new(), Vec::new())
    };

    // Generate draft answer with Evidence IDs (and action plan if action request)
    let (draft_answer, action_plan) = if translator_output.intent_type == IntentType::ActionRequest {
        let plan = generate_action_plan(&translator_output, request);
        let draft = format!(
            "This is an action request.\n\n\
             Proposed steps:\n{}\n\n\
             Would affect:\n\
             - Packages: {}\n\
             - Services: {}\n\
             - Files: {}\n\n\
             Risk level: {}\n\
             Required confirmation: \"{}\"\n\n\
             {}\n\n\
             [Action NOT executed - confirmation required]",
            plan.steps.iter().map(|s| format!("  {}", s)).collect::<Vec<_>>().join("\n"),
            if plan.affected_packages.is_empty() { "(none)".to_string() } else { plan.affected_packages.join(", ") },
            if plan.affected_services.is_empty() { "(none)".to_string() } else { plan.affected_services.join(", ") },
            if plan.affected_files.is_empty() { "(none)".to_string() } else { plan.affected_files.join(", ") },
            plan.risk,
            if plan.confirmation_phrase.is_empty() { "none" } else { &plan.confirmation_phrase },
            plan.rollback_outline
        );
        (draft, Some(plan))
    } else if !tool_results.is_empty() {
        (generate_draft_response_v2(&translator_output, &tool_results), None)
    } else {
        (generate_draft_response(&translator_output, &evidence), None)
    };

    // [anna] to [junior]: request for verification
    dialogue(
        Actor::Anna,
        Actor::Junior,
        &format!("Verify this draft response:\n\n{}", draft_answer),
    );
    println!();

    // Junior verification (v0.0.7: uses tool_results or legacy evidence)
    // v0.0.31: Track Junior LLM metrics
    metrics.record(MetricType::JuniorStart);
    let junior_start = std::time::Instant::now();
    let (verification, junior_reason) = match get_junior_client().await {
        LlmAvailability::Ready(client, model) => {
            show_spinner(&format!("[junior verifying via {}...]", model));

            let result = if !tool_results.is_empty() {
                call_junior_llm(&client, &model, request, &translator_output, &tool_results, &draft_answer).await
            } else {
                call_junior_llm_legacy(&client, &model, request, &translator_output, &evidence, &draft_answer).await
            };

            match result {
                Ok(v) => {
                    clear_spinner();
                    metrics.record(MetricType::JuniorSuccess);
                    (v, None)
                }
                Err(e) => {
                    clear_spinner();
                    // Record as timeout if it looks like a timeout
                    let err_str = e.to_string().to_lowercase();
                    if err_str.contains("timeout") || err_str.contains("timed out") {
                        metrics.record(MetricType::JuniorTimeout);
                    }
                    println!("  {} {}", "Junior LLM error:".yellow(), e);
                    if !tool_results.is_empty() {
                        (fallback_junior_score_v2(&translator_output, &tool_results), Some(format!("LLM error: {}", e)))
                    } else {
                        (fallback_junior_score(&translator_output, &evidence), Some(format!("LLM error: {}", e)))
                    }
                }
            }
        }
        LlmAvailability::NotReady(reason) => {
            let reason_str = reason.format();
            println!("  {} {}", "Junior:".cyan(), reason_str.dimmed());

            if let LlmNotReadyReason::Pulling { percent, .. } = &reason {
                let filled = (percent / 5.0) as usize;
                let empty = 20 - filled.min(20);
                println!("  {} [{}{}] {:.1}%", "Progress:".cyan(), "=".repeat(filled), " ".repeat(empty), percent);
            }

            let mut fallback = if !tool_results.is_empty() {
                fallback_junior_score_v2(&translator_output, &tool_results)
            } else {
                fallback_junior_score(&translator_output, &evidence)
            };
            fallback.score = fallback.score.saturating_sub(10);
            fallback.critique = format!("{} ({})", fallback.critique, reason_str);
            (fallback, Some(reason_str))
        }
    };
    junior_ms = junior_start.elapsed().as_millis() as u64;
    metrics.record_latency("junior", junior_ms);

    // [junior] to [anna]: verification result (v0.0.7: include uncited claims)
    let uncited_str = if verification.uncited_claims.is_empty() {
        String::new()
    } else {
        format!("\nUncited claims: {}", verification.uncited_claims.join(", "))
    };
    let junior_response = format!(
        "Reliability: {}%\nCritique: {}{}\nSuggestions: {}{}",
        verification.score,
        if verification.critique.is_empty() { "(none)" } else { &verification.critique },
        uncited_str,
        if verification.suggestions.is_empty() { "(none)" } else { &verification.suggestions },
        if verification.mutation_warning { "\n\n*** DO NOT EXECUTE without confirmation ***" } else { "" }
    );
    dialogue(Actor::Junior, Actor::Anna, &junior_response);
    println!();

    // [anna] to [you]: final response (v0.0.7: with evidence citations)
    let final_response = if !tool_results.is_empty() {
        generate_final_response_v2(&translator_output, &tool_results, &verification, action_plan.as_ref())
    } else {
        generate_final_response(&translator_output, &evidence, &verification, action_plan.as_ref())
    };

    // v0.0.13: Enforce learning claims - apply penalty for uncited learning statements
    let verification = enforce_learning_claims(verification, &final_response);

    dialogue(Actor::Anna, Actor::You, &final_response);
    println!();

    // Display reliability score (with any learning claim penalties applied)
    let reliability_display = format!("Reliability: {}%", verification.score);
    if verification.score >= 80 {
        println!("  {}", reliability_display.green());
    } else if verification.score >= 50 {
        println!("  {}", reliability_display.yellow());
    } else {
        println!("  {}", reliability_display.red());
    }

    // Notes about fallbacks
    if translator_reason.is_some() || junior_reason.is_some() {
        let reasons: Vec<String> = [translator_reason, junior_reason].into_iter().flatten().collect();
        println!("  {} {}", "Note:".dimmed(), format!("Fallback used - {}", reasons.join("; ")).dimmed());
    }

    // v0.0.8: Handle mutation execution for medium-risk operations
    if let Some(ref plan) = action_plan {
        if plan.is_medium_risk_executable && plan.mutation_plan.is_some() {
            handle_mutation_execution(plan, &verification, &tool_results).await;
        }
    }

    // v0.0.21: Record performance metrics
    let total_ms = start_time.elapsed().as_millis() as u64;
    let sample = LatencySample {
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        total_ms,
        translator_ms,
        junior_ms,
        tools_ms,
        cache_hit,
    };

    // Save performance stats (fire and forget)
    let mut stats = PerfStats::load();
    stats.record_sample(sample);
    let _ = stats.save();

    // v0.0.31: Record request success and e2e latency, then save reliability metrics
    metrics.record(MetricType::RequestSuccess);
    metrics.record_latency("e2e", total_ms);
    if cache_hit {
        metrics.record(MetricType::CacheHit);
    } else {
        metrics.record(MetricType::CacheMiss);
    }
    metrics.prune(); // Clean up old data
    let _ = metrics.save();

    // v0.0.44: Update and save case file
    case_file.summary.intent_type = translator_output.intent_type.to_string();
    case_file.summary.reliability_score = verification.score;
    case_file.summary.evidence_count = tool_results.len();
    case_file.summary.duration_ms = total_ms;
    case_file.timing = CaseTiming {
        translator_ms,
        evidence_ms: tools_ms,
        junior_ms,
        total_ms,
    };
    case_file.result.success = verification.score >= 50;
    case_file.result.reliability_score = verification.score;
    if !verification.critique.is_empty() {
        case_file.result.errors.push(verification.critique.clone());
    }
    case_file.summary.outcome = if verification.score >= 50 {
        CaseOutcome::Success
    } else {
        CaseOutcome::Partial
    };

    // Save case file (ignore errors - logging shouldn't break the request)
    if let Err(e) = case_file.save() {
        tracing::warn!("Failed to save case file: {}", e);
    }

    // Show performance note at debug level 2
    if config.ui.debug_level >= 2 {
        let cache_note = if cache_hit { " (cache hit)" } else { "" };
        println!("  {} total: {}ms, translator: {}ms, tools: {}ms, junior: {}ms{}",
            "[perf]".dimmed(),
            total_ms, translator_ms, tools_ms, junior_ms, cache_note.dimmed());
    }
}

/// Handle mutation execution with confirmation gate (v0.0.8)
async fn handle_mutation_execution(
    action_plan: &ActionPlan,
    verification: &JuniorVerification,
    evidence: &[ToolResult],
) {
    let mutation_plan = match &action_plan.mutation_plan {
        Some(p) => p,
        None => return,
    };

    println!();

    // Check Junior reliability threshold
    if verification.score < 70 {
        dialogue(
            Actor::Anna,
            Actor::You,
            &format!(
                "I cannot execute this mutation because Junior's reliability score ({}%) is below the required 70% threshold.\n\
                 The plan is provided above for reference only.\n\
                 Please verify the plan manually before executing.",
                verification.score
            ),
        );
        return;
    }

    // Show confirmation prompt
    let confirmation_msg = format!(
        "I can execute this action for you.\n\n\
         What will change:\n  {}\n\n\
         Why:\n  {}\n\n\
         Risk level: {}\n\n\
         Rollback plan:\n  {}\n\n\
         To proceed, type exactly: {}",
        mutation_plan.what_will_change,
        mutation_plan.why_required,
        mutation_plan.risk,
        action_plan.rollback_outline.lines().map(|l| format!("  {}", l)).collect::<Vec<_>>().join("\n"),
        action_plan.confirmation_phrase.bold()
    );
    dialogue_always(Actor::Anna, Actor::You, &confirmation_msg);
    println!();

    // Get user confirmation
    print!("  {} ", "you>".bright_white().bold());
    io::stdout().flush().ok();

    let stdin = io::stdin();
    let mut input = String::new();
    if stdin.lock().read_line(&mut input).is_err() {
        println!("  {} Failed to read input", "Error:".red());
        return;
    }

    let input = input.trim();
    dialogue(Actor::You, Actor::Anna, input);
    println!();

    // Validate confirmation
    if input != action_plan.confirmation_phrase {
        dialogue_always(
            Actor::Anna,
            Actor::You,
            &format!(
                "Confirmation not received. Expected: \"{}\"\n\
                 Action NOT executed.",
                action_plan.confirmation_phrase
            ),
        );
        println!();
        println!("  {}", "Action cancelled - wrong or missing confirmation phrase".yellow());
        return;
    }

    // Execute mutations
    dialogue(
        Actor::Anna,
        Actor::Annad,
        &format!(
            "User confirmed. Please execute the following operations:\n{}",
            mutation_plan.mutations.iter()
                .map(|m| format!("  - {} ({})", m.tool_name,
                    m.parameters.get("service")
                        .and_then(|v| v.as_str())
                        .unwrap_or("system")))
                .collect::<Vec<_>>()
                .join("\n")
        ),
    );
    println!();

    let rollback_manager = RollbackManager::new();
    let mutation_catalog = MutationToolCatalog::new();

    let evidence_ids: Vec<String> = evidence.iter().map(|e| e.evidence_id.clone()).collect();
    let mut all_results = Vec::new();
    let mut all_success = true;

    for mut mutation in mutation_plan.mutations.clone() {
        // Set confirmation token and evidence
        mutation.confirmation_token = Some(MEDIUM_RISK_CONFIRMATION.to_string());
        mutation.evidence_ids = evidence_ids.clone();

        match execute_mutation(&mutation, &mutation_catalog, &rollback_manager) {
            Ok(result) => {
                let status = if result.success { "SUCCESS".green().to_string() } else { "FAILED".red().to_string() };
                println!("  {} {}: {}", status, result.tool_name, result.human_summary);

                if let Some(ref rollback) = result.rollback_info {
                    println!("    {}", format!("Rollback: {}", rollback.rollback_instructions.lines().next().unwrap_or("N/A")).dimmed());
                }

                if !result.success {
                    all_success = false;
                    if let Some(ref err) = result.error {
                        println!("    {}", format!("Error: {}", err).red());
                    }
                }
                all_results.push(result);
            }
            Err(e) => {
                println!("  {} {}: {}", "ERROR".red(), mutation.tool_name, e);
                all_success = false;
                break;
            }
        }
    }

    println!();

    // Summary
    let summary = if all_success {
        format!(
            "All operations completed successfully.\n\n\
             {} mutation(s) executed.\n\
             Logs saved to: /var/lib/anna/rollback/logs/",
            all_results.len()
        )
    } else {
        format!(
            "Some operations failed. Check the output above.\n\n\
             Executed: {} of {} mutation(s)\n\
             Logs saved to: /var/lib/anna/rollback/logs/",
            all_results.iter().filter(|r| r.success).count(),
            mutation_plan.mutations.len()
        )
    };
    dialogue(Actor::Annad, Actor::Anna, &summary);
    println!();

    // Final response
    let final_msg = if all_success {
        "Operations completed. Rollback instructions are available in the logs if needed.".to_string()
    } else {
        "Some operations failed. Please check the logs and consider manual rollback if needed.".to_string()
    };
    dialogue(Actor::Anna, Actor::You, &final_msg);
    println!();

    if all_success {
        println!("  {}", "Mutation(s) executed successfully".green());
    } else {
        println!("  {}", "Some mutation(s) failed".red());
    }
}

/// Generate a draft response based on intent and evidence
fn generate_draft_response(translator_output: &TranslatorOutput, evidence: &[Evidence]) -> String {
    match translator_output.intent_type {
        IntentType::SystemQuery => {
            if evidence.is_empty() {
                "I need to query system data to answer this, but no evidence is available.\n\
                 The snapshot system is not providing data for these targets."
                    .to_string()
            } else {
                let sources: Vec<_> = evidence.iter().map(|e| e.source.as_str()).collect();
                let mut response = format!("Based on system data from: {}\n\n", sources.join(", "));

                // Add actual evidence data
                for ev in evidence {
                    response.push_str(&format!("{}:\n{}\n\n", ev.source, ev.data));
                }
                response
            }
        }
        IntentType::Question => {
            "This appears to be a general question.\n\n\
             General knowledge questions require LLM response generation,\n\
             which will be fully implemented in a future version."
                .to_string()
        }
        IntentType::Unknown => {
            "I wasn't able to classify this request with confidence.\n\n\
             Could you rephrase or provide more details?"
                .to_string()
        }
        IntentType::ActionRequest => {
            // This case is handled separately with action plan
            "Action request - see plan above.".to_string()
        }
        IntentType::FixIt => {
            // v0.0.34: Fix-It mode - handled by separate troubleshooting loop
            "Fix-It mode - troubleshooting in progress...".to_string()
        }
    }
}

/// Generate a draft response with Evidence IDs from tool results (v0.0.7)
fn generate_draft_response_v2(translator_output: &TranslatorOutput, tool_results: &[ToolResult]) -> String {
    match translator_output.intent_type {
        IntentType::SystemQuery => {
            let successful: Vec<_> = tool_results.iter().filter(|r| r.success).collect();
            if successful.is_empty() {
                "I need to query system data to answer this, but no evidence is available.\n\
                 The tool execution did not provide data for these targets."
                    .to_string()
            } else {
                let mut response = String::from("Based on system data:\n\n");

                // Add evidence with IDs for citation
                for result in &successful {
                    response.push_str(&format!(
                        "[{}] {}: {}\n\n",
                        result.evidence_id,
                        result.tool_name,
                        result.human_summary
                    ));
                }

                // Add citation reminder
                response.push_str("\n(Evidence IDs above can be cited as [E1], [E2], etc.)\n");
                response
            }
        }
        IntentType::Question => {
            "This appears to be a general question.\n\n\
             General knowledge questions require LLM response generation,\n\
             which will be fully implemented in a future version."
                .to_string()
        }
        IntentType::Unknown => {
            "I wasn't able to classify this request with confidence.\n\n\
             Could you rephrase or provide more details?"
                .to_string()
        }
        IntentType::ActionRequest => {
            // This case is handled separately with action plan
            "Action request - see plan above.".to_string()
        }
        IntentType::FixIt => {
            // v0.0.34: Fix-It mode - handled by separate troubleshooting loop
            "Fix-It mode - troubleshooting in progress...".to_string()
        }
    }
}

/// Generate final response incorporating Junior's feedback
fn generate_final_response(
    translator_output: &TranslatorOutput,
    evidence: &[Evidence],
    verification: &JuniorVerification,
    action_plan: Option<&ActionPlan>,
) -> String {
    let mut response = if let Some(plan) = action_plan {
        format!(
            "Proposed action plan:\n\n\
             Steps:\n{}\n\n\
             Would affect:\n\
             - Packages: {}\n\
             - Services: {}\n\
             - Files: {}\n\n\
             Risk: {} | Confirmation: \"{}\"",
            plan.steps.iter().enumerate().map(|(i, s)| format!("{}. {}", i + 1, s.trim_start_matches(|c: char| c.is_numeric() || c == '.'))).collect::<Vec<_>>().join("\n"),
            if plan.affected_packages.is_empty() { "(none)".to_string() } else { plan.affected_packages.join(", ") },
            if plan.affected_services.is_empty() { "(none)".to_string() } else { plan.affected_services.join(", ") },
            if plan.affected_files.is_empty() { "(none)".to_string() } else { plan.affected_files.join(", ") },
            plan.risk,
            if plan.confirmation_phrase.is_empty() { "none" } else { &plan.confirmation_phrase }
        )
    } else {
        generate_draft_response(translator_output, evidence)
    };

    // Add Junior's suggestions
    if !verification.suggestions.is_empty()
        && verification.suggestions != "(none)"
        && !verification.suggestions.contains("unavailable")
    {
        response.push_str("\n\n[Junior: ");
        response.push_str(&verification.suggestions);
        response.push(']');
    }

    // Add mutation warning
    if verification.mutation_warning {
        response.push_str("\n\n[Action NOT executed - explicit confirmation required]");
    }

    response
}

/// Generate final response with Evidence ID citations (v0.0.7)
/// v0.0.18: Evidence summaries are redacted for secrets
fn generate_final_response_v2(
    translator_output: &TranslatorOutput,
    tool_results: &[ToolResult],
    verification: &JuniorVerification,
    action_plan: Option<&ActionPlan>,
) -> String {
    let mut response = if let Some(plan) = action_plan {
        format!(
            "Proposed action plan:\n\n\
             Steps:\n{}\n\n\
             Would affect:\n\
             - Packages: {}\n\
             - Services: {}\n\
             - Files: {}\n\n\
             Risk: {} | Confirmation: \"{}\"",
            plan.steps.iter().enumerate().map(|(i, s)| format!("{}. {}", i + 1, s.trim_start_matches(|c: char| c.is_numeric() || c == '.'))).collect::<Vec<_>>().join("\n"),
            if plan.affected_packages.is_empty() { "(none)".to_string() } else { plan.affected_packages.join(", ") },
            if plan.affected_services.is_empty() { "(none)".to_string() } else { plan.affected_services.join(", ") },
            if plan.affected_files.is_empty() { "(none)".to_string() } else { plan.affected_files.join(", ") },
            plan.risk,
            if plan.confirmation_phrase.is_empty() { "none" } else { &plan.confirmation_phrase }
        )
    } else {
        // Generate response with citations
        let successful: Vec<_> = tool_results.iter().filter(|r| r.success).collect();
        if successful.is_empty() {
            "No evidence available to answer this query.".to_string()
        } else {
            let mut response = String::from("Based on gathered evidence:\n\n");

            // Add each piece of evidence with its ID (v0.0.18: redacted for secrets)
            for result in &successful {
                let redacted_summary = redact_evidence(&result.human_summary, None)
                    .unwrap_or_else(|e| e);
                response.push_str(&format!(
                    "[{}] {}\n",
                    result.evidence_id,
                    redacted_summary
                ));
            }

            // Add evidence legend
            response.push_str("\n---\nEvidence sources:\n");
            for result in &successful {
                response.push_str(&format!(
                    "  [{}]: {} ({})\n",
                    result.evidence_id,
                    result.tool_name,
                    if result.success { "OK" } else { "failed" }
                ));
            }
            response
        }
    };

    // Add uncited claims warning if any
    if !verification.uncited_claims.is_empty() {
        response.push_str(&format!(
            "\n\n[Warning: Uncited claims detected: {}]",
            verification.uncited_claims.join(", ")
        ));
    }

    // Add Junior's suggestions
    if !verification.suggestions.is_empty()
        && verification.suggestions != "(none)"
        && !verification.suggestions.contains("unavailable")
    {
        response.push_str("\n\n[Junior: ");
        response.push_str(&verification.suggestions);
        response.push(']');
    }

    // Add mutation warning
    if verification.mutation_warning {
        response.push_str("\n\n[Action NOT executed - explicit confirmation required]");
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
    fn test_translator_deterministic_system_query() {
        let output = translator_classify_deterministic("what CPU do I have?");
        assert_eq!(output.intent_type, IntentType::SystemQuery);
        assert!(output.targets.contains(&"cpu".to_string()));
        assert_eq!(output.risk, RiskLevel::ReadOnly);
        assert!(!output.llm_backed);
    }

    #[test]
    fn test_translator_deterministic_action_request() {
        let output = translator_classify_deterministic("install nginx");
        assert_eq!(output.intent_type, IntentType::ActionRequest);
        assert!(output.targets.contains(&"nginx".to_string()));
        assert_eq!(output.risk, RiskLevel::MediumRisk);
    }

    #[test]
    fn test_translator_deterministic_high_risk() {
        let output = translator_classify_deterministic("delete all docker containers");
        assert_eq!(output.intent_type, IntentType::ActionRequest);
        assert!(output.targets.contains(&"docker".to_string()));
        assert_eq!(output.risk, RiskLevel::HighRisk);
    }

    #[test]
    fn test_parse_translator_response_valid() {
        let response = "INTENT: system_query\nTARGETS: cpu, memory\nRISK: read_only\nEVIDENCE_NEEDS: hw_snapshot\nCLARIFICATION:";
        let output = parse_translator_response(response);
        assert!(output.is_some());
        let output = output.unwrap();
        assert_eq!(output.intent_type, IntentType::SystemQuery);
        assert_eq!(output.targets, vec!["cpu", "memory"]);
        assert_eq!(output.risk, RiskLevel::ReadOnly);
        assert!(output.llm_backed);
    }

    #[test]
    fn test_parse_translator_response_with_doctor() {
        // v0.0.44: New canonical format with DOCTOR field
        let response = "INTENT: doctor_query\nTARGETS: wifi,network\nRISK: read_only\nTOOLS: hw_snapshot_summary,journal_warnings\nDOCTOR: networking\nCONFIDENCE: 85";
        let output = parse_translator_response(response);
        assert!(output.is_some());
        let output = output.unwrap();
        assert_eq!(output.intent_type, IntentType::FixIt);
        assert!(output.targets.contains(&"wifi".to_string()));
        assert_eq!(output.confidence, 85);
    }

    #[test]
    fn test_parse_translator_response_invalid() {
        let response = "This is not valid output";
        let output = parse_translator_response(response);
        assert!(output.is_none());
    }

    #[test]
    fn test_parse_clarification() {
        let s = "Which service?|nginx|apache|default:0";
        let c = parse_clarification(s);
        assert!(c.is_some());
        let c = c.unwrap();
        assert_eq!(c.question, "Which service?");
        assert_eq!(c.options, vec!["nginx", "apache"]);
        assert_eq!(c.default_option, 0);
    }

    #[test]
    fn test_parse_clarification_rejects_garbage() {
        // Reject echoed prompt instructions
        let s = "[empty if not needed, OR \"question|option1|option2|option3|default:N\"]";
        assert!(parse_clarification(s).is_none());

        // Reject placeholder options
        let s = "Which editor?|option1|option2|default:0";
        assert!(parse_clarification(s).is_none());

        // Reject missing question mark
        let s = "Which editor|vim|neovim|default:0";
        assert!(parse_clarification(s).is_none());

        // Accept valid clarification
        let s = "Which editor?|vim|neovim|default:1";
        let c = parse_clarification(s);
        assert!(c.is_some());
        let c = c.unwrap();
        assert_eq!(c.question, "Which editor?");
        assert_eq!(c.options, vec!["vim", "neovim"]);
        assert_eq!(c.default_option, 1);
    }

    #[test]
    fn test_risk_level_parsing() {
        assert_eq!("read_only".parse::<RiskLevel>().unwrap(), RiskLevel::ReadOnly);
        assert_eq!("low".parse::<RiskLevel>().unwrap(), RiskLevel::LowRisk);
        assert_eq!("medium".parse::<RiskLevel>().unwrap(), RiskLevel::MediumRisk);
        assert_eq!("high".parse::<RiskLevel>().unwrap(), RiskLevel::HighRisk);
    }

    #[test]
    fn test_intent_type_parsing() {
        assert_eq!("question".parse::<IntentType>().unwrap(), IntentType::Question);
        assert_eq!("system_query".parse::<IntentType>().unwrap(), IntentType::SystemQuery);
        assert_eq!("action_request".parse::<IntentType>().unwrap(), IntentType::ActionRequest);
        assert_eq!("unknown".parse::<IntentType>().unwrap(), IntentType::Unknown);
    }

    #[test]
    fn test_fallback_scoring() {
        let output = TranslatorOutput {
            intent_type: IntentType::SystemQuery,
            targets: vec!["cpu".to_string()],
            risk: RiskLevel::ReadOnly,
            evidence_needs: vec!["hw_snapshot".to_string()],
            tool_plan: None,
            clarification: None,
            confidence: 85,
            llm_backed: true,
        };
        let evidence = vec![Evidence {
            source: "test".to_string(),
            data: "test data".to_string(),
            timestamp: "now".to_string(),
            excerpted: false,
        }];

        let verification = fallback_junior_score(&output, &evidence);
        // +40 evidence + +30 confident + +20 observational + +10 read-only = 100
        assert_eq!(verification.score, 100);
    }

    #[test]
    fn test_fallback_scoring_with_deterministic_translator() {
        let output = TranslatorOutput {
            intent_type: IntentType::SystemQuery,
            targets: vec!["cpu".to_string()],
            risk: RiskLevel::ReadOnly,
            evidence_needs: vec![],
            tool_plan: None,
            clarification: None,
            confidence: 85,
            llm_backed: false, // Deterministic
        };
        let evidence = vec![Evidence {
            source: "test".to_string(),
            data: "test data".to_string(),
            timestamp: "now".to_string(),
            excerpted: false,
        }];

        let verification = fallback_junior_score(&output, &evidence);
        // 100 - 10 (translator fallback) = 90
        assert_eq!(verification.score, 90);
    }

    #[test]
    fn test_action_plan_generation() {
        let output = TranslatorOutput {
            intent_type: IntentType::ActionRequest,
            targets: vec!["nginx".to_string()],
            risk: RiskLevel::MediumRisk,
            evidence_needs: vec!["sw_snapshot".to_string()],
            tool_plan: None,
            clarification: None,
            confidence: 85,
            llm_backed: true,
        };

        let plan = generate_action_plan(&output, "install nginx");
        assert!(!plan.steps.is_empty());
        assert!(plan.affected_packages.contains(&"nginx".to_string()));
        assert_eq!(plan.risk, RiskLevel::MediumRisk);
        assert_eq!(plan.confirmation_phrase, "yes");
    }

    #[test]
    fn test_excerpt_data() {
        let long_data = "a".repeat(10000);
        let (excerpted, is_excerpted) = excerpt_data(&long_data, 100);
        assert!(is_excerpted);
        assert!(excerpted.len() < long_data.len());
        assert!(excerpted.contains("[EXCERPT"));
    }

    #[test]
    fn test_parse_junior_response() {
        let response = "SCORE: 75\nCRITIQUE: Missing disk info\nSUGGESTIONS: Add disk usage\nMUTATION_WARNING: no";
        let output = TranslatorOutput {
            intent_type: IntentType::SystemQuery,
            targets: vec![],
            risk: RiskLevel::ReadOnly,
            evidence_needs: vec![],
            tool_plan: None,
            clarification: None,
            confidence: 80,
            llm_backed: true,
        };

        let verification = parse_junior_response(response, &output);
        assert_eq!(verification.score, 75);
        assert_eq!(verification.critique, "Missing disk info");
        assert_eq!(verification.suggestions, "Add disk usage");
        assert!(!verification.mutation_warning);
    }

    // ==========================================================================
    // v0.0.7: Tool Plan and Evidence ID Tests
    // ==========================================================================

    #[test]
    fn test_deterministic_generates_tool_plan() {
        // CPU query should generate hw_snapshot_summary tool
        let output = translator_classify_deterministic("what CPU do I have?");
        assert!(output.tool_plan.is_some());
        let plan = output.tool_plan.unwrap();
        assert!(!plan.tools.is_empty());
        assert!(plan.tools.iter().any(|t| t.tool_name == "hw_snapshot_summary"));
    }

    #[test]
    fn test_deterministic_service_query_generates_service_tool() {
        // nginx query should generate service_status tool
        let output = translator_classify_deterministic("is nginx running?");
        assert!(output.tool_plan.is_some());
        let plan = output.tool_plan.unwrap();
        // Should have service_status for nginx
        assert!(plan.tools.iter().any(|t| t.tool_name == "service_status"));
    }

    #[test]
    fn test_parse_junior_response_with_uncited_claims() {
        let response = "SCORE: 50\nCRITIQUE: Missing citations\nUNCITED_CLAIMS: CPU model, memory size, disk space\nSUGGESTIONS: Add [E1], [E2] citations\nMUTATION_WARNING: no";
        let output = TranslatorOutput {
            intent_type: IntentType::SystemQuery,
            targets: vec![],
            risk: RiskLevel::ReadOnly,
            evidence_needs: vec![],
            tool_plan: None,
            clarification: None,
            confidence: 80,
            llm_backed: true,
        };

        let verification = parse_junior_response(response, &output);
        assert_eq!(verification.score, 50);
        assert_eq!(verification.uncited_claims.len(), 3);
        assert!(verification.uncited_claims.contains(&"CPU model".to_string()));
        assert!(verification.uncited_claims.contains(&"memory size".to_string()));
        assert!(verification.uncited_claims.contains(&"disk space".to_string()));
    }

    #[test]
    fn test_parse_junior_response_no_uncited_claims() {
        let response = "SCORE: 95\nCRITIQUE: All claims cited\nUNCITED_CLAIMS: none\nSUGGESTIONS: None needed\nMUTATION_WARNING: no";
        let output = TranslatorOutput {
            intent_type: IntentType::SystemQuery,
            targets: vec![],
            risk: RiskLevel::ReadOnly,
            evidence_needs: vec![],
            tool_plan: None,
            clarification: None,
            confidence: 80,
            llm_backed: true,
        };

        let verification = parse_junior_response(response, &output);
        assert_eq!(verification.score, 95);
        assert!(verification.uncited_claims.is_empty());
    }

    #[test]
    fn test_tool_catalog_creation() {
        let catalog = ToolCatalog::new();
        assert!(catalog.get("hw_snapshot_summary").is_some());
        assert!(catalog.get("sw_snapshot_summary").is_some());
        assert!(catalog.get("status_snapshot").is_some());
        assert!(catalog.get("service_status").is_some());
        assert!(catalog.get("nonexistent_tool").is_none());
    }

    #[test]
    fn test_evidence_collector() {
        let mut collector = EvidenceCollector::new();
        let id1 = collector.next_id();
        let id2 = collector.next_id();
        assert_eq!(id1, "E1");
        assert_eq!(id2, "E2");
    }

    #[test]
    fn test_fallback_scoring_v2_with_tool_results() {
        let output = TranslatorOutput {
            intent_type: IntentType::SystemQuery,
            targets: vec!["cpu".to_string()],
            risk: RiskLevel::ReadOnly,
            evidence_needs: vec![],
            tool_plan: None,
            clarification: None,
            confidence: 85,
            llm_backed: true,
        };
        let tool_results = vec![ToolResult {
            tool_name: "hw_snapshot_summary".to_string(),
            evidence_id: "E1".to_string(),
            data: serde_json::json!({"cpu": "AMD Ryzen"}),
            human_summary: "CPU: AMD Ryzen 7 5800X".to_string(),
            success: true,
            error: None,
            timestamp: 0,
        }];

        let verification = fallback_junior_score_v2(&output, &tool_results);
        // +40 evidence + +30 confident + +20 observational + +10 read-only = 100
        assert_eq!(verification.score, 100);
    }
}
