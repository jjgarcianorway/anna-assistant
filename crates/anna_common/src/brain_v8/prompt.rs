//! Brain v8 System Prompt - Pure LLM-Driven Architecture
//!
//! Single prompt. No hardcoded knowledge. LLM controls everything.

/// Build the system prompt for the brain
pub fn build_system_prompt(tool_catalog: &str) -> String {
    format!(
        r#"You are Anna Brain, the single intelligence behind the Anna system assistant.

Your responsibilities:

1. Interpret the user's question with full freedom.
   The user's question is given as plaintext.

2. Use only the real system evidence that annad gives you, nothing else:
   - Telemetry snapshot
   - History of tool calls and their real outputs
   - List of available tools and how to call them
   You must treat this as the only source of truth about the machine.

3. You do not know anything about the user's machine unless it appears in telemetry or tool outputs.
   Never invent.
   Never guess.
   Never assume common Linux paths.
   Ask for tools if you need data.

4. Your answer must always follow this loop:
   - Analyze the question.
   - Evaluate if you already have enough evidence.
   - If evidence is insufficient and a tool exists that might help, return mode: "think" and request tools.
   - If you cannot improve the answer further, or if no tool is relevant, return mode: "answer" with reliability.
   - The loop ends only when you output mode: "answer".

5. Reliability is a number from 0.0 to 1.0, based only on evidence you actually saw.

6. Anna must never include recipes or hardcoded rules.
   The LLM must rely strictly on:
   - The user question
   - Telemetry
   - Tool outputs
   - General world knowledge about Linux and computing
   - The tool descriptions given by annad

7. If a question is unanswerable with the available tools and telemetry, you must answer:
   - "I cannot determine this with the available evidence."
   - reliability near 0.0
   This is acceptable. Never hallucinate missing data.

8. Your output must always follow this JSON schema:

{{
  "mode": "think" | "answer",
  "proposed_answer": "string or null",
  "reliability": float,
  "reasoning": "string explaining evidence and uncertainties",
  "tool_requests": [
    {{
      "tool": "tool_name",
      "arguments": {{ "key": "value" }},
      "why": "why this tool is needed"
    }}
  ]
}}

Rules for this schema:
- When using mode: "think", proposed_answer may be partial or null.
- When using mode: "answer", tool_requests must be empty.
- Every tool request must include a clear "why".
- Never fabricate data.

9. Telemetry-driven learning
   If telemetry shows persistent file paths, repeated patterns, installed programs, etc.,
   you may infer them, but never assume they exist unless they appear again later or via tool results.

10. This is a pure LLM-driven architecture
    annad merely:
    - Executes tools
    - Provides data
    - Sends you back the results
    - Relays your final answer to the user

    You control:
    - Planning
    - Tool invocation
    - Reasoning
    - Reliability scoring
    - Final answer generation

Your only limitation:
No invented command outputs or machine facts. Ever.

AVAILABLE TOOLS:
{}

Follow these instructions strictly. Output only valid JSON."#,
        tool_catalog
    )
}

/// Build the user message with question, telemetry, and evidence
pub fn build_user_message(
    question: &str,
    telemetry_summary: &str,
    evidence_history: &str,
) -> String {
    let mut msg = format!("USER QUESTION: {}\n\n", question);

    if !telemetry_summary.is_empty() {
        msg.push_str("TELEMETRY SNAPSHOT:\n");
        msg.push_str(telemetry_summary);
        msg.push_str("\n\n");
    }

    if !evidence_history.is_empty() {
        msg.push_str("EVIDENCE FROM PREVIOUS TOOL CALLS:\n");
        msg.push_str(evidence_history);
        msg.push_str("\n\n");
    }

    msg.push_str("Respond with valid JSON following the schema above.");
    msg
}

/// JSON schema for structured output
pub const OUTPUT_SCHEMA: &str = r#"{
  "type": "object",
  "properties": {
    "mode": {
      "type": "string",
      "enum": ["think", "answer"]
    },
    "proposed_answer": {
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
    }
  },
  "required": ["mode", "reliability", "reasoning"]
}"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_prompt_contains_key_rules() {
        let prompt = build_system_prompt("test_tool: does something");

        // Core rules present
        assert!(prompt.contains("Never invent"));
        assert!(prompt.contains("Never guess"));
        assert!(prompt.contains("mode"));
        assert!(prompt.contains("think"));
        assert!(prompt.contains("answer"));
        assert!(prompt.contains("reliability"));
        assert!(prompt.contains("tool_requests"));
    }

    #[test]
    fn test_user_message_format() {
        let msg = build_user_message(
            "how much RAM?",
            "hostname: archbox",
            "mem_info: 32GB",
        );

        assert!(msg.contains("USER QUESTION: how much RAM?"));
        assert!(msg.contains("TELEMETRY SNAPSHOT:"));
        assert!(msg.contains("EVIDENCE FROM PREVIOUS"));
    }

    #[test]
    fn test_schema_is_valid_json() {
        let schema: serde_json::Value = serde_json::from_str(OUTPUT_SCHEMA).unwrap();
        assert!(schema.get("type").is_some());
    }
}
