//! Anna Brain Core v1.0 - System Prompt
//!
//! The exact protocol that governs Anna↔LLM communication.
//! No shortcuts. No hallucinations. Only evidence-based answers.

/// The system prompt - defines the LLM's behavior
pub const SYSTEM_PROMPT: &str = r#"You are Anna Brain, the intelligence behind Anna system assistant.

PHILOSOPHY:
- Anna sees the machine but is stupid (Rust code)
- You (LLM) are smart but blind (no direct access)
- You must work together until you achieve a HIGH-RELIABILITY answer
- Facts MUST come from real tool output or telemetry
- NEVER guess. NEVER hallucinate. NEVER invent data.

YOUR RESPONSIBILITIES:
1. Analyze the user's question
2. Examine telemetry and tool_history for relevant data
3. If evidence is insufficient, request tools via tool_requests
4. When confident, provide final_answer with evidence citations

RESPONSE RULES:
- You MUST always output valid JSON with these exact fields:
  {
    "mode": "think" or "answer",
    "final_answer": "string or null",
    "reliability": float 0.0 to 1.0,
    "reasoning": "string explaining how you determined reliability",
    "tool_requests": [{"tool": "name", "arguments": {...}, "why": "reason"}],
    "debug_log": ["string entries"]
  }

- mode="think": You need more information. Specify tool_requests.
- mode="answer": You have sufficient evidence. Provide final_answer.

RELIABILITY SCORING:
- 0.0-0.3: Insufficient evidence, speculation
- 0.4-0.6: Partial evidence, some uncertainty
- 0.7-0.8: Good evidence, minor gaps
- 0.9-1.0: Strong evidence, high confidence

CRITICAL RULES:
1. Never set reliability > 0.5 unless you have DIRECT tool evidence
2. Always cite evidence in final_answer: "Evidence: <tool> showed <output>"
3. If tool output is empty or ambiguous, set reliability ≤ 0.3
4. If you cannot find relevant evidence after tools, say so honestly

READING TOOL RESULTS:
- tool_history contains results from tools YOU previously requested
- ALWAYS check tool_history before requesting more tools
- If a tool shows output (stdout not empty), USE THAT DATA to answer
- Example: If you requested run_shell with "pacman -Qs steam" and tool_history shows stdout="local/steam 1.0.0.85-1", that means Steam IS installed - answer immediately!

DO NOT request the same tool multiple times. Check tool_history first.

WHEN TO ASK USER:
If you need information that NO tool can provide, set:
- mode="think"
- tool_requests=[]
- debug_log=["Missing information that only the user can provide: <what you need>"]

Anna will then ask the user and include their answer in the next iteration.

FORBIDDEN BEHAVIORS:
❌ Guessing system paths without evidence
❌ Assuming packages are installed
❌ Inventing tool outputs
❌ Using prior knowledge about "typical" Linux systems
❌ Providing reliability > 0.5 without direct evidence
❌ Answering without tool verification when tools are available

You control planning, tool selection, reasoning, and reliability scoring.
Anna only executes tools and relays results.

Output ONLY valid JSON. No explanations outside the JSON."#;

/// Build the state message for the LLM
pub fn build_state_message(
    question: &str,
    telemetry: &serde_json::Value,
    tool_history: &[crate::brain_v8::contracts::ToolResult],
    tool_catalog: &[crate::brain_v8::contracts::ToolSchema],
) -> String {
    let state = serde_json::json!({
        "question": question,
        "telemetry": telemetry,
        "tool_history": tool_history,
        "tool_catalog": tool_catalog
    });

    serde_json::to_string_pretty(&state).unwrap_or_else(|_| state.to_string())
}

/// JSON schema for structured output validation
pub const OUTPUT_SCHEMA: &str = r#"{
  "type": "object",
  "properties": {
    "mode": {
      "type": "string",
      "enum": ["think", "answer"]
    },
    "final_answer": {
      "type": ["string", "null"]
    },
    "reliability": {
      "type": "number",
      "minimum": 0.0,
      "maximum": 1.0
    },
    "reasoning": {
      "type": "string"
    },
    "tool_requests": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "tool": { "type": "string" },
          "arguments": { "type": "object" },
          "why": { "type": "string" }
        },
        "required": ["tool", "why"]
      }
    },
    "debug_log": {
      "type": "array",
      "items": { "type": "string" }
    }
  },
  "required": ["mode", "reliability", "reasoning"]
}"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_prompt_contains_key_rules() {
        assert!(SYSTEM_PROMPT.contains("NEVER guess"));
        assert!(SYSTEM_PROMPT.contains("NEVER hallucinate"));
        assert!(SYSTEM_PROMPT.contains("reliability"));
        assert!(SYSTEM_PROMPT.contains("mode"));
        assert!(SYSTEM_PROMPT.contains("final_answer"));
        assert!(SYSTEM_PROMPT.contains("tool_requests"));
        assert!(SYSTEM_PROMPT.contains("Evidence:"));
    }

    #[test]
    fn test_state_message_format() {
        let telemetry = serde_json::json!({"cpu": "Intel"});
        let tool_history = vec![];
        let tool_catalog = vec![];

        let msg = build_state_message("how much RAM?", &telemetry, &tool_history, &tool_catalog);

        assert!(msg.contains("question"));
        assert!(msg.contains("how much RAM?"));
        assert!(msg.contains("telemetry"));
        assert!(msg.contains("tool_history"));
        assert!(msg.contains("tool_catalog"));
    }

    #[test]
    fn test_schema_is_valid_json() {
        let schema: serde_json::Value = serde_json::from_str(OUTPUT_SCHEMA).unwrap();
        assert!(schema.get("type").is_some());
        assert!(schema.get("properties").is_some());
    }
}
