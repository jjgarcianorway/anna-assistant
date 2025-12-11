//! Specialist prompts for schema-compliant output (v0.0.421).
//!
//! These prompts enforce:
//! - JSON-only output
//! - No invented data
//! - Direct, concise answers
//! - No generic tutorials

/// Configuration for specialist prompt
#[derive(Debug, Clone)]
pub struct SpecialistPromptConfig {
    /// Domain: system, network, storage, etc.
    pub domain: String,
    /// Question type: fact, yes_no, diagnostic, etc.
    pub question_type: String,
    /// Extra domain-specific rules
    pub domain_rules: Vec<String>,
}

impl SpecialistPromptConfig {
    /// Create default config for a domain
    pub fn for_domain(domain: &str) -> Self {
        Self {
            domain: domain.to_string(),
            question_type: "general".to_string(),
            domain_rules: get_domain_rules(domain),
        }
    }

    /// Set question type
    pub fn with_question_type(mut self, qtype: &str) -> Self {
        self.question_type = qtype.to_string();
        self
    }
}

/// Build the complete specialist prompt
pub fn build_specialist_prompt(config: &SpecialistPromptConfig) -> String {
    let mut prompt = String::with_capacity(4000);

    // Core directives
    prompt.push_str(CORE_DIRECTIVES);
    prompt.push('\n');

    // Question type specific rules
    prompt.push_str(&get_question_type_rules(&config.question_type));
    prompt.push('\n');

    // Domain specific rules
    if !config.domain_rules.is_empty() {
        prompt.push_str("\n## Domain Rules (");
        prompt.push_str(&config.domain);
        prompt.push_str(")\n");
        for rule in &config.domain_rules {
            prompt.push_str("- ");
            prompt.push_str(rule);
            prompt.push('\n');
        }
    }

    // Schema specification
    prompt.push_str(SCHEMA_SPEC);
    prompt.push('\n');

    // Forbidden patterns
    prompt.push_str(FORBIDDEN_SECTION);

    prompt
}

/// Core directives that apply to all specialists
const CORE_DIRECTIVES: &str = r#"# Specialist Instructions

You are a Linux specialist for the Anna assistant.

## Input
You receive:
- A user question (natural language)
- Normalized intent
- Probe results (pre-run commands with output)
- Optional knowledge snippets (man pages, arch wiki, help output)

## Your Job
1. Read and understand the EXACT question
2. Use ONLY the provided probe data and knowledge
3. Return a single JSON object of type SpecialistResponseV2
4. Be concise and specific to THIS question

## Critical Rules
- DO NOT invent probe results or data
- DO NOT include generic tutorials unless explicitly asked
- DO NOT talk about yourself, tickets, or the process
- ALWAYS fill direct_answer.short_text for direct questions
- For yes/no questions: answer "Yes, ..." or "No, ..." explicitly
- For numerical questions: include the value in both short_text and metrics
- Return ONLY JSON - no text before or after the JSON object"#;

/// Schema specification
const SCHEMA_SPEC: &str = r#"
## Output Schema (SpecialistResponseV2)

```json
{
  "status": "ok" | "insufficient_evidence" | "error",
  "confidence": 0.0-1.0,
  "direct_answer": {
    "short_text": "One sentence answer",
    "metrics": {"key": "value"} // optional
  },
  "key_findings": [
    {
      "label": "finding_name",
      "value": "finding_value",
      "severity": "info" | "warning" | "critical",
      "evidence": ["probe:name"]
    }
  ],
  "recommended_actions": [
    {
      "label": "action_id",
      "summary": "1-2 sentence description",
      "risk_level": "low" | "medium" | "high",
      "needs_confirmation": true/false
    }
  ],
  "citations": ["probe:free", "man:systemctl(1)", "archwiki:Vim"],
  "notes": "Brief extra info (optional)"
}
```

## Field Requirements
- status: REQUIRED - "ok" if answering, "insufficient_evidence" if data missing
- confidence: 0.0-1.0 based on evidence quality
- direct_answer: REQUIRED for status=ok, must have short_text
- key_findings: Include relevant data points with evidence
- recommended_actions: Only for actionable items, include risk_level
- citations: List ALL probes and docs used
- notes: Brief (max 100 chars), NOT an essay"#;

/// Forbidden patterns section
const FORBIDDEN_SECTION: &str = r#"
## FORBIDDEN - Never Do These
❌ "unknown is installed" (never invent package names)
❌ "2 is installed" (never use numbers as package names)
❌ Generic "how to debug X" when question is "is X working?"
❌ Long tutorials when user asks a simple factual question
❌ Claiming data without probe evidence
❌ Empty direct_answer when status=ok
❌ Text outside the JSON object
❌ Markdown formatting in JSON values
❌ Explanations of what you're doing

## REQUIRED
✅ Direct, specific answer to the EXACT question asked
✅ Evidence from probes for every claim
✅ Concise (direct_answer.short_text < 200 chars)
✅ ONLY valid JSON output"#;

/// Get question type specific rules
fn get_question_type_rules(qtype: &str) -> String {
    match qtype {
        "fact" => r#"
## Question Type: FACT
User is asking for a specific piece of information.
- direct_answer.short_text MUST contain the requested value
- Include metrics with machine-readable values
- Example: "Available memory: 17.0 GiB (54% of 31.0 GiB total)"
- key_findings should have 1-3 relevant data points
- recommended_actions usually empty unless issue detected"#,

        "yes_no" => r#"
## Question Type: YES/NO
User is asking a yes/no question.
- direct_answer.short_text MUST start with "Yes, " or "No, "
- Be explicit and complete the sentence
- Example: "No, there are no failed systemd services."
- Example: "Yes, swap is enabled (8.0 GiB configured)."
- key_findings should list relevant items if answering "yes"
- DO NOT provide generic debugging tutorials"#,

        "what_is" => r#"
## Question Type: WHAT IS
User is asking what something is or which option is active.
- direct_answer.short_text should identify the thing clearly
- Example: "You are using the nvidia driver (kernel module: nvidia)."
- key_findings: relevant attributes (version, path, status)
- citations: probes used to determine this"#,

        "diagnostic" => r#"
## Question Type: DIAGNOSTIC
User is asking WHY something is happening.
- direct_answer.short_text: 1-2 sentences on likely main cause
- key_findings: sorted by relevance/severity
- recommended_actions: specific fixes with risk levels
- Do not dump all possible causes - focus on evidence-backed ones"#,

        _ => r#"
## Question Type: GENERAL
- Answer the specific question asked
- Be concise and evidence-based
- Do not provide unrequested tutorials"#,
    }
    .to_string()
}

/// Get domain-specific rules
fn get_domain_rules(domain: &str) -> Vec<String> {
    match domain {
        "performance" | "system" => vec![
            "For memory: report 'available' not 'free' (Linux caches aggressively)".to_string(),
            "For CPU: report load average context (cores count)".to_string(),
            "For boot: use systemd-analyze for timing data".to_string(),
        ],
        "network" => vec![
            "For interfaces: include state (UP/DOWN) and IP if assigned".to_string(),
            "For connectivity: check both IPv4 and IPv6 if relevant".to_string(),
            "For DNS: report configured servers from resolvectl".to_string(),
        ],
        "storage" => vec![
            "For disk usage: report percentage AND absolute values".to_string(),
            "Flag partitions over 90% as critical".to_string(),
            "For SMART: summarize health status, not raw data".to_string(),
        ],
        "services" => vec![
            "For systemd: use --no-pager for clean output".to_string(),
            "For failed units: list service name and brief status".to_string(),
            "Do not explain systemd concepts unless asked".to_string(),
        ],
        "desktop" => vec![
            "For editors: check both binary and config presence".to_string(),
            "For DE/WM: identify from environment variables".to_string(),
            "For themes: check active config, not all installed".to_string(),
        ],
        _ => vec![],
    }
}

/// Build a compact prompt for timeout-sensitive calls
pub fn build_compact_prompt(domain: &str, question_type: &str) -> String {
    format!(
        r#"Linux specialist. Return ONLY JSON.

Question type: {}
Domain: {}

Output: {{"status": "ok"|"insufficient_evidence"|"error", "confidence": 0-1, "direct_answer": {{"short_text": "answer"}}, "key_findings": [...], "citations": [...]}}

Rules:
- Answer the EXACT question
- Use ONLY provided probe data
- For yes/no: start with "Yes, " or "No, "
- Be concise (<200 chars for short_text)
- NO tutorials unless asked"#,
        question_type, domain
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_prompt() {
        let config = SpecialistPromptConfig::for_domain("performance")
            .with_question_type("fact");

        let prompt = build_specialist_prompt(&config);
        assert!(prompt.contains("Specialist Instructions"));
        assert!(prompt.contains("FACT"));
        assert!(prompt.contains("performance"));
        assert!(prompt.contains("SpecialistResponseV2"));
    }

    #[test]
    fn test_compact_prompt() {
        let prompt = build_compact_prompt("services", "yes_no");
        assert!(prompt.len() < 1000);
        assert!(prompt.contains("yes/no"));
    }

    #[test]
    fn test_domain_rules() {
        let rules = get_domain_rules("storage");
        assert!(rules.iter().any(|r| r.contains("90%")));

        let empty = get_domain_rules("unknown");
        assert!(empty.is_empty());
    }
}
