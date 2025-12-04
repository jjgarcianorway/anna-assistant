//! Translator v0.0.82 - JSON Schema + Robust Parser
//!
//! Structured translator output with strict JSON schema.
//! Falls back to deterministic routing when JSON parsing fails.
//! Silent in normal mode (no parse warnings to stdout).

use serde::{Deserialize, Serialize};

/// Translator JSON output schema (v0.0.82)
/// This is what the LLM must produce as valid JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslatorJsonOutput {
    /// Intent classification
    pub intent: TranslatorIntent,
    /// Targets extracted from request (keywords)
    pub targets: Vec<String>,
    /// Risk level for the request
    pub risk: TranslatorRisk,
    /// Tools to execute (in order)
    pub tools: Vec<String>,
    /// Doctor domain if problem report
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doctor: Option<String>,
    /// Confidence 0-100
    pub confidence: u8,
}

/// Intent types for translator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranslatorIntent {
    /// System query (about this machine)
    SystemQuery,
    /// Action request (change something)
    ActionRequest,
    /// Knowledge query (how to / what is)
    KnowledgeQuery,
    /// Doctor query (problem report)
    DoctorQuery,
    /// Unknown / unclassified
    Unknown,
}

impl std::fmt::Display for TranslatorIntent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TranslatorIntent::SystemQuery => write!(f, "system_query"),
            TranslatorIntent::ActionRequest => write!(f, "action_request"),
            TranslatorIntent::KnowledgeQuery => write!(f, "knowledge_query"),
            TranslatorIntent::DoctorQuery => write!(f, "doctor_query"),
            TranslatorIntent::Unknown => write!(f, "unknown"),
        }
    }
}

/// Risk levels for translator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranslatorRisk {
    ReadOnly,
    Low,
    Medium,
    High,
}

impl std::fmt::Display for TranslatorRisk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TranslatorRisk::ReadOnly => write!(f, "read_only"),
            TranslatorRisk::Low => write!(f, "low"),
            TranslatorRisk::Medium => write!(f, "medium"),
            TranslatorRisk::High => write!(f, "high"),
        }
    }
}

/// Parse result from translator
#[derive(Debug, Clone)]
pub struct TranslatorParseResult {
    /// The parsed output (if successful)
    pub output: Option<TranslatorJsonOutput>,
    /// Parse error message (if failed)
    pub error: Option<String>,
    /// Whether the LLM was used
    pub llm_backed: bool,
    /// Number of retries attempted
    pub retries: u32,
}

/// JSON schema for translator (to include in LLM system prompt)
pub const TRANSLATOR_JSON_SCHEMA: &str = r#"{
  "type": "object",
  "required": ["intent", "targets", "risk", "tools", "confidence"],
  "properties": {
    "intent": {
      "type": "string",
      "enum": ["system_query", "action_request", "knowledge_query", "doctor_query", "unknown"]
    },
    "targets": {
      "type": "array",
      "items": { "type": "string" }
    },
    "risk": {
      "type": "string",
      "enum": ["read_only", "low", "medium", "high"]
    },
    "tools": {
      "type": "array",
      "items": { "type": "string" }
    },
    "doctor": {
      "type": "string",
      "enum": ["networking", "graphics", "audio", "storage", "boot"]
    },
    "confidence": {
      "type": "integer",
      "minimum": 0,
      "maximum": 100
    }
  }
}"#;

/// System prompt for JSON translator
pub const TRANSLATOR_JSON_SYSTEM_PROMPT: &str = r#"You are a request classifier. Output ONLY valid JSON matching this schema:

{
  "intent": "system_query|action_request|knowledge_query|doctor_query|unknown",
  "targets": ["keyword1", "keyword2"],
  "risk": "read_only|low|medium|high",
  "tools": ["tool1", "tool2"],
  "doctor": "networking|graphics|audio|storage|boot",
  "confidence": 0-100
}

INTENT RULES:
- system_query = asks about THIS machine (CPU, RAM, disk, services)
- action_request = wants to change something (install, restart, edit)
- knowledge_query = asks HOW TO do something or WHAT IS something
- doctor_query = reports a problem (slow, broken, disconnecting)

TOOLS (pick the RIGHT tool for the question):
- memory_info = RAM/memory questions
- mount_usage = disk space questions
- kernel_version = kernel version questions
- network_status = network status
- hw_snapshot_summary = hardware (CPU, GPU, specs)
- service_status = check a service
- sw_snapshot_summary = packages, services

DOCTOR (only for problems):
- networking = wifi, ethernet, DNS issues
- graphics = display, GPU, resolution
- audio = sound, speakers, microphone
- storage = disk, mount, filesystem
- boot = startup, systemd, slow boot

Output ONLY the JSON object. No explanation, no markdown, no extra text."#;

/// Parse translator JSON response with fallback extraction
pub fn parse_translator_json(response: &str) -> TranslatorParseResult {
    // Try direct JSON parse first
    if let Some(output) = try_parse_json(response) {
        return TranslatorParseResult {
            output: Some(output),
            error: None,
            llm_backed: true,
            retries: 0,
        };
    }

    // Try extracting JSON from markdown code block
    if let Some(output) = try_extract_json_from_markdown(response) {
        return TranslatorParseResult {
            output: Some(output),
            error: None,
            llm_backed: true,
            retries: 0,
        };
    }

    // Try extracting JSON object from mixed content
    if let Some(output) = try_extract_json_object(response) {
        return TranslatorParseResult {
            output: Some(output),
            error: None,
            llm_backed: true,
            retries: 0,
        };
    }

    // Try fallback to old text format
    if let Some(output) = try_parse_legacy_format(response) {
        return TranslatorParseResult {
            output: Some(output),
            error: None,
            llm_backed: true,
            retries: 0,
        };
    }

    // All parsing failed
    TranslatorParseResult {
        output: None,
        error: Some(format!(
            "Failed to parse translator response: {}",
            response.chars().take(100).collect::<String>()
        )),
        llm_backed: true,
        retries: 0,
    }
}

/// Try to parse raw JSON
fn try_parse_json(response: &str) -> Option<TranslatorJsonOutput> {
    let trimmed = response.trim();
    serde_json::from_str(trimmed).ok()
}

/// Try to extract JSON from markdown code block
fn try_extract_json_from_markdown(response: &str) -> Option<TranslatorJsonOutput> {
    // Look for ```json ... ``` or ``` ... ```
    let response = response.trim();

    // Find start of code block
    let start = if let Some(idx) = response.find("```json") {
        idx + 7
    } else if let Some(idx) = response.find("```") {
        idx + 3
    } else {
        return None;
    };

    // Find end of code block
    let remaining = &response[start..];
    let end = remaining.find("```")?;

    let json_str = remaining[..end].trim();
    serde_json::from_str(json_str).ok()
}

/// Try to extract JSON object from mixed content
fn try_extract_json_object(response: &str) -> Option<TranslatorJsonOutput> {
    // Find first { and last }
    let start = response.find('{')?;
    let end = response.rfind('}')?;

    if end <= start {
        return None;
    }

    let json_str = &response[start..=end];
    serde_json::from_str(json_str).ok()
}

/// Try to parse legacy text format (INTENT: ... TARGETS: ...)
fn try_parse_legacy_format(response: &str) -> Option<TranslatorJsonOutput> {
    let mut intent = None;
    let mut targets = Vec::new();
    let mut risk = TranslatorRisk::ReadOnly;
    let mut tools = Vec::new();
    let mut doctor = None;
    let mut confidence: u8 = 85;

    for line in response.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Parse INTENT
        if let Some(value) = strip_prefix_ci(line, "INTENT:") {
            let v = value.trim().to_lowercase();
            intent = match v.as_str() {
                "system_query" | "system query" => Some(TranslatorIntent::SystemQuery),
                "action_request" | "action request" => Some(TranslatorIntent::ActionRequest),
                "knowledge_query" | "knowledge query" | "question" => {
                    Some(TranslatorIntent::KnowledgeQuery)
                }
                "doctor_query" | "doctor query" | "fix_it" | "fixit" => {
                    Some(TranslatorIntent::DoctorQuery)
                }
                _ => Some(TranslatorIntent::Unknown),
            };
        }
        // Parse TARGETS
        else if let Some(value) = strip_prefix_ci(line, "TARGETS:") {
            let v = value.trim();
            if !v.eq_ignore_ascii_case("none") && !v.is_empty() {
                targets = v
                    .split(',')
                    .map(|s| s.trim().to_lowercase())
                    .filter(|s| !s.is_empty() && s != "none")
                    .collect();
            }
        }
        // Parse RISK
        else if let Some(value) = strip_prefix_ci(line, "RISK:") {
            let v = value.trim().to_lowercase();
            risk = match v.as_str() {
                "read_only" | "read-only" | "readonly" => TranslatorRisk::ReadOnly,
                "low" | "low_risk" => TranslatorRisk::Low,
                "medium" | "medium_risk" => TranslatorRisk::Medium,
                "high" | "high_risk" => TranslatorRisk::High,
                _ => TranslatorRisk::ReadOnly,
            };
        }
        // Parse TOOLS
        else if let Some(value) = strip_prefix_ci(line, "TOOLS:") {
            let v = value.trim();
            if !v.eq_ignore_ascii_case("none") && !v.is_empty() {
                tools = v
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("none"))
                    .collect();
            }
        }
        // Parse DOCTOR
        else if let Some(value) = strip_prefix_ci(line, "DOCTOR:") {
            let v = value.trim().to_lowercase();
            if !v.eq_ignore_ascii_case("none") && !v.is_empty() {
                doctor = Some(v);
            }
        }
        // Parse CONFIDENCE
        else if let Some(value) = strip_prefix_ci(line, "CONFIDENCE:") {
            if let Ok(c) = value.trim().parse::<u8>() {
                confidence = c.min(100);
            }
        }
    }

    let intent = intent?;

    Some(TranslatorJsonOutput {
        intent,
        targets,
        risk,
        tools,
        doctor,
        confidence,
    })
}

/// Case-insensitive prefix stripping
fn strip_prefix_ci<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
    let line_upper = line.to_uppercase();
    let prefix_upper = prefix.to_uppercase();
    if line_upper.starts_with(&prefix_upper) {
        Some(&line[prefix.len()..])
    } else {
        None
    }
}

/// Deterministic fallback classification (no LLM needed)
pub fn classify_deterministic(request: &str) -> TranslatorJsonOutput {
    let lower = request.to_lowercase();
    let words: Vec<&str> = lower.split_whitespace().collect();

    // Detect intent
    let intent = detect_intent(&lower, &words);

    // Detect targets
    let targets = detect_targets(&lower);

    // Detect risk
    let risk = detect_risk(&lower, intent);

    // Detect tools
    let tools = detect_tools(&lower, &targets);

    // Detect doctor domain
    let doctor = detect_doctor(&lower);

    // Confidence for deterministic is 90 (high but not certain)
    let confidence = 90;

    TranslatorJsonOutput {
        intent,
        targets,
        risk,
        tools,
        doctor,
        confidence,
    }
}

/// Detect intent from request
fn detect_intent(lower: &str, words: &[&str]) -> TranslatorIntent {
    // Action keywords
    let action_kw = [
        "install",
        "remove",
        "uninstall",
        "start",
        "stop",
        "restart",
        "enable",
        "disable",
        "edit",
        "change",
        "set",
        "configure",
        "delete",
        "create",
        "update",
    ];

    // Problem keywords
    let problem_kw = [
        "not working",
        "broken",
        "slow",
        "disconnecting",
        "failing",
        "error",
        "crash",
        "won't",
        "can't",
        "cannot",
        "doesn't",
        "does not",
        "problem",
        "issue",
        "help",
        "fix",
        "trouble",
    ];

    // Knowledge keywords
    let knowledge_kw = ["how to", "how do", "what is", "explain", "tutorial", "guide"];

    // Check for problem report first (doctor_query)
    for kw in problem_kw {
        if lower.contains(kw) {
            return TranslatorIntent::DoctorQuery;
        }
    }

    // Check for action request
    for kw in action_kw {
        if words.contains(&kw) || lower.starts_with(kw) {
            return TranslatorIntent::ActionRequest;
        }
    }

    // Check for knowledge query
    for kw in knowledge_kw {
        if lower.contains(kw) {
            return TranslatorIntent::KnowledgeQuery;
        }
    }

    // System query keywords
    let system_kw = [
        "what", "which", "how much", "how many", "show", "list", "status", "info", "version",
    ];

    for kw in system_kw {
        if lower.starts_with(kw) || lower.contains(kw) {
            return TranslatorIntent::SystemQuery;
        }
    }

    // Default to system query for unknown
    TranslatorIntent::SystemQuery
}

/// Detect targets from request
fn detect_targets(lower: &str) -> Vec<String> {
    let mut targets = Vec::new();

    let target_patterns = [
        ("cpu", "cpu"),
        ("processor", "cpu"),
        ("memory", "memory"),
        ("ram", "memory"),
        ("disk", "disk"),
        ("storage", "disk"),
        ("space", "disk"),
        ("network", "network"),
        ("wifi", "network"),
        ("ethernet", "network"),
        ("internet", "network"),
        ("kernel", "kernel"),
        ("linux", "kernel"),
        ("uname", "kernel"),
        ("audio", "audio"),
        ("sound", "audio"),
        ("speaker", "audio"),
        ("gpu", "gpu"),
        ("graphics", "gpu"),
        ("display", "gpu"),
        ("editor", "editor"),
        ("vim", "editor"),
        ("nvim", "editor"),
        ("emacs", "editor"),
        ("update", "updates"),
        ("upgrade", "updates"),
        ("pacman", "updates"),
    ];

    for (pattern, target) in target_patterns {
        if lower.contains(pattern) && !targets.contains(&target.to_string()) {
            targets.push(target.to_string());
        }
    }

    targets
}

/// Detect risk level
fn detect_risk(lower: &str, intent: TranslatorIntent) -> TranslatorRisk {
    match intent {
        TranslatorIntent::SystemQuery | TranslatorIntent::KnowledgeQuery => TranslatorRisk::ReadOnly,
        TranslatorIntent::DoctorQuery => TranslatorRisk::ReadOnly,
        TranslatorIntent::ActionRequest => {
            // High risk actions
            if lower.contains("delete")
                || lower.contains("remove")
                || lower.contains("format")
                || lower.contains("wipe")
            {
                TranslatorRisk::High
            }
            // Medium risk actions
            else if lower.contains("install")
                || lower.contains("restart")
                || lower.contains("enable")
                || lower.contains("disable")
            {
                TranslatorRisk::Medium
            }
            // Low risk actions
            else {
                TranslatorRisk::Low
            }
        }
        TranslatorIntent::Unknown => TranslatorRisk::ReadOnly,
    }
}

/// Detect tools needed
fn detect_tools(lower: &str, targets: &[String]) -> Vec<String> {
    let mut tools = Vec::new();

    for target in targets {
        match target.as_str() {
            "memory" => tools.push("memory_info".to_string()),
            "disk" => tools.push("mount_usage".to_string()),
            "kernel" => tools.push("kernel_version".to_string()),
            "network" => tools.push("network_status".to_string()),
            "audio" => tools.push("audio_status".to_string()),
            "cpu" | "gpu" => tools.push("hw_snapshot_summary".to_string()),
            "updates" => tools.push("sw_snapshot_summary".to_string()),
            "editor" => tools.push("editor_detection".to_string()),
            _ => {}
        }
    }

    // If no specific tools, pick based on keywords
    if tools.is_empty() {
        if lower.contains("service") || lower.contains("running") {
            tools.push("service_status".to_string());
        } else if lower.contains("package") || lower.contains("installed") {
            tools.push("sw_snapshot_summary".to_string());
        } else if lower.contains("hardware") || lower.contains("specs") {
            tools.push("hw_snapshot_summary".to_string());
        }
    }

    tools
}

/// Detect doctor domain
fn detect_doctor(lower: &str) -> Option<String> {
    if lower.contains("wifi")
        || lower.contains("network")
        || lower.contains("ethernet")
        || lower.contains("dns")
        || lower.contains("internet")
        || lower.contains("connection")
    {
        return Some("networking".to_string());
    }

    if lower.contains("display")
        || lower.contains("monitor")
        || lower.contains("resolution")
        || lower.contains("tearing")
        || lower.contains("gpu")
        || lower.contains("graphics")
    {
        return Some("graphics".to_string());
    }

    if lower.contains("audio")
        || lower.contains("sound")
        || lower.contains("speaker")
        || lower.contains("microphone")
        || lower.contains("volume")
    {
        return Some("audio".to_string());
    }

    if lower.contains("disk")
        || lower.contains("mount")
        || lower.contains("filesystem")
        || lower.contains("storage")
        || lower.contains("btrfs")
    {
        return Some("storage".to_string());
    }

    if lower.contains("boot")
        || lower.contains("startup")
        || lower.contains("systemd")
        || lower.contains("slow start")
    {
        return Some("boot".to_string());
    }

    None
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json_direct() {
        let json = r#"{"intent":"system_query","targets":["cpu"],"risk":"read_only","tools":["hw_snapshot_summary"],"confidence":95}"#;
        let result = parse_translator_json(json);
        assert!(result.output.is_some());
        let output = result.output.unwrap();
        assert_eq!(output.intent, TranslatorIntent::SystemQuery);
        assert_eq!(output.targets, vec!["cpu"]);
    }

    #[test]
    fn test_parse_json_from_markdown() {
        let response = r#"Here is the classification:

```json
{
  "intent": "action_request",
  "targets": ["nginx"],
  "risk": "medium",
  "tools": ["sw_snapshot_summary"],
  "confidence": 90
}
```

This is an install request."#;

        let result = parse_translator_json(response);
        assert!(result.output.is_some());
        let output = result.output.unwrap();
        assert_eq!(output.intent, TranslatorIntent::ActionRequest);
    }

    #[test]
    fn test_parse_json_embedded() {
        let response = r#"I'll classify this: {"intent":"system_query","targets":["memory"],"risk":"read_only","tools":["memory_info"],"confidence":95} done."#;
        let result = parse_translator_json(response);
        assert!(result.output.is_some());
    }

    #[test]
    fn test_parse_legacy_format() {
        let response = r#"INTENT: system_query
TARGETS: memory
RISK: read_only
TOOLS: memory_info
CONFIDENCE: 95"#;

        let result = parse_translator_json(response);
        assert!(result.output.is_some());
        let output = result.output.unwrap();
        assert_eq!(output.intent, TranslatorIntent::SystemQuery);
    }

    #[test]
    fn test_deterministic_system_query() {
        let output = classify_deterministic("how much ram do I have");
        assert_eq!(output.intent, TranslatorIntent::SystemQuery);
        assert!(output.targets.contains(&"memory".to_string()));
        assert_eq!(output.risk, TranslatorRisk::ReadOnly);
    }

    #[test]
    fn test_deterministic_action_request() {
        let output = classify_deterministic("install nginx");
        assert_eq!(output.intent, TranslatorIntent::ActionRequest);
        assert_eq!(output.risk, TranslatorRisk::Medium);
    }

    #[test]
    fn test_deterministic_doctor_query() {
        let output = classify_deterministic("wifi keeps disconnecting");
        assert_eq!(output.intent, TranslatorIntent::DoctorQuery);
        assert_eq!(output.doctor, Some("networking".to_string()));
    }

    #[test]
    fn test_deterministic_kernel() {
        let output = classify_deterministic("what kernel version");
        assert!(output.targets.contains(&"kernel".to_string()));
        assert!(output.tools.contains(&"kernel_version".to_string()));
    }

    #[test]
    fn test_deterministic_disk() {
        let output = classify_deterministic("how much disk space is free");
        assert!(output.targets.contains(&"disk".to_string()));
        assert!(output.tools.contains(&"mount_usage".to_string()));
    }
}
