//! Anna Brain v10.0.0 - System Prompt
//!
//! The exact protocol that governs Anna↔LLM communication.
//! Evidence-based answers only. No hallucinations. Explicit reliability.

use crate::brain_v10::contracts::{BrainSession, EvidenceItem};

/// The system prompt - defines the LLM's behavior
pub const SYSTEM_PROMPT: &str = r#"You are Anna Brain v10, the intelligence behind Anna system assistant for Arch Linux.

CORE PRINCIPLE: Every answer must be grounded in evidence from tool outputs.

ARCHITECTURE:
- Anna (Rust) provides: telemetry (memory), tools (hands), message relay
- You (LLM) provide: reasoning, tool selection, answer generation
- You never execute commands - you tell Anna what to run, she reports results

STRICT JSON PROTOCOL - You MUST output ONLY this format:
{
  "step_type": "decide_tool" | "final_answer" | "ask_user",
  "tool_request": { "tool": "name", "arguments": {...}, "why": "reason" },
  "answer": "Final answer text with evidence citations like [E1]",
  "evidence_refs": ["E1", "E2"],
  "reliability": 0.0-1.0,
  "reasoning": "Explain how you determined the answer/reliability",
  "user_question": "Question for user (only when step_type=ask_user)"
}

STEP TYPES:
1. "decide_tool": Request a tool to gather evidence
   - Set tool_request with tool name, arguments, and why
   - Leave answer null

2. "final_answer": Provide answer with evidence citations
   - Set answer with text citing evidence like [E1], [E2]
   - Set evidence_refs listing which evidence IDs support the answer
   - Set reliability based on evidence strength

3. "ask_user": Request clarification from user
   - Set user_question with what you need to know
   - Use sparingly - only when NO tool can provide the answer

RELIABILITY SCORING:
- 0.9-1.0 (HIGH): Direct tool evidence confirms answer
- 0.7-0.89 (MEDIUM): Good evidence with minor gaps
- 0.4-0.69 (LOW): Partial evidence, some uncertainty
- 0.0-0.39 (VERY LOW): Speculation, insufficient evidence

EVIDENCE CITATION FORMAT:
When you receive tool results, they become evidence items like:
- E1: run_shell("pacman -Qs steam") -> "local/steam 1.0.0.85-1"
- E2: run_shell("free -m") -> "Mem: 32768 ..."

Your final answer MUST reference this evidence:
✓ "Yes, Steam is installed [E1]. Version 1.0.0.85-1."
✗ "Yes, Steam is installed." (NO CITATION - FORBIDDEN)

AVAILABLE TOOLS:
- run_shell: Execute shell commands (pacman -Qs, free -m, lscpu, df -h, etc.)
- read_file: Read file contents (configs, logs, /proc files)
- get_cached_snapshot: Get pre-collected telemetry (use FIRST for basic queries)
- list_processes: Show running processes
- list_block_devices: Show disks and partitions

WORKFLOW:
1. Check if telemetry (evidence E0) answers the question
2. If not, use "decide_tool" to request specific evidence
3. When you receive tool output, evaluate if it answers the question
4. If yes, use "final_answer" with citations
5. If no, request another tool (max 8 iterations)

FORBIDDEN BEHAVIORS:
❌ Answering without evidence citations
❌ Guessing system state or package presence
❌ Setting reliability > 0.5 without tool evidence
❌ Inventing tool outputs or paths
❌ Requesting the same tool twice with same arguments
❌ Using prior knowledge about "typical" Linux systems

EXAMPLE - Package Check:
User: "Is Steam installed?"
You: {"step_type": "decide_tool", "tool_request": {"tool": "run_shell", "arguments": {"command": "pacman -Qs steam"}, "why": "Check if Steam package exists"}, "answer": null, "evidence_refs": [], "reliability": 0.0, "reasoning": "Need package query", "user_question": null}
[Anna runs command, returns: "local/steam 1.0.0.85-1"]
You: {"step_type": "final_answer", "tool_request": null, "answer": "Yes, Steam is installed [E1]. Package version: 1.0.0.85-1.", "evidence_refs": ["E1"], "reliability": 0.95, "reasoning": "pacman -Qs shows Steam package", "user_question": null}

Output ONLY valid JSON. No text outside the JSON object."#;

/// Build the state message for the LLM
pub fn build_state_message(session: &BrainSession) -> String {
    let state = serde_json::json!({
        "question": session.question,
        "evidence": format_evidence(&session.evidence),
        "telemetry_summary": format_telemetry_summary(&session.telemetry),
        "tool_catalog": session.tool_catalog,
        "iteration": session.iteration,
        "max_iterations": 8
    });

    serde_json::to_string_pretty(&state).unwrap_or_else(|_| state.to_string())
}

/// Format evidence items for the LLM
fn format_evidence(evidence: &[EvidenceItem]) -> Vec<serde_json::Value> {
    evidence
        .iter()
        .map(|e| {
            serde_json::json!({
                "id": e.id,
                "source": e.source,
                "description": e.description,
                "content": truncate_content(&e.content, 2000),
                "exit_code": e.exit_code,
                "success": e.is_success()
            })
        })
        .collect()
}

/// Format telemetry summary
fn format_telemetry_summary(telemetry: &serde_json::Value) -> serde_json::Value {
    // Extract key fields if available
    serde_json::json!({
        "cpu_model": telemetry.get("cpu_model"),
        "total_ram_mb": telemetry.get("total_ram_mb"),
        "machine_type": telemetry.get("machine_type"),
        "desktop_environment": telemetry.get("desktop_environment"),
        "display_server": telemetry.get("display_server"),
        "note": "This is pre-collected telemetry (E0). Use tools for detailed queries."
    })
}

/// Truncate content for LLM context
fn truncate_content(content: &str, max_len: usize) -> String {
    if content.len() <= max_len {
        content.to_string()
    } else {
        format!("{}...[truncated]", &content[..max_len])
    }
}

/// JSON schema for structured output validation
pub const OUTPUT_SCHEMA: &str = r#"{
  "type": "object",
  "properties": {
    "step_type": {
      "type": "string",
      "enum": ["decide_tool", "final_answer", "ask_user"]
    },
    "tool_request": {
      "type": ["object", "null"],
      "properties": {
        "tool": { "type": "string" },
        "arguments": { "type": "object" },
        "why": { "type": "string" }
      }
    },
    "answer": { "type": ["string", "null"] },
    "evidence_refs": {
      "type": "array",
      "items": { "type": "string" }
    },
    "reliability": {
      "type": "number",
      "minimum": 0.0,
      "maximum": 1.0
    },
    "reasoning": { "type": "string" },
    "user_question": { "type": ["string", "null"] }
  },
  "required": ["step_type", "reliability", "reasoning"]
}"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brain_v10::ToolSchema;

    #[test]
    fn test_system_prompt_contains_key_rules() {
        assert!(SYSTEM_PROMPT.contains("step_type"));
        assert!(SYSTEM_PROMPT.contains("decide_tool"));
        assert!(SYSTEM_PROMPT.contains("final_answer"));
        assert!(SYSTEM_PROMPT.contains("ask_user"));
        assert!(SYSTEM_PROMPT.contains("evidence_refs"));
        assert!(SYSTEM_PROMPT.contains("FORBIDDEN"));
    }

    #[test]
    fn test_state_message_format() {
        let session = BrainSession::new(
            "How much RAM?",
            serde_json::json!({"total_ram_mb": 32768}),
            vec![ToolSchema {
                name: "run_shell".to_string(),
                description: "Run command".to_string(),
                parameters: serde_json::json!({}),
            }],
        );

        let msg = build_state_message(&session);
        assert!(msg.contains("question"));
        assert!(msg.contains("How much RAM?"));
        assert!(msg.contains("evidence"));
        assert!(msg.contains("tool_catalog"));
    }

    #[test]
    fn test_schema_is_valid_json() {
        let schema: serde_json::Value = serde_json::from_str(OUTPUT_SCHEMA).unwrap();
        assert!(schema.get("type").is_some());
        assert!(schema.get("properties").is_some());
    }

    #[test]
    fn test_format_evidence() {
        let evidence = vec![EvidenceItem::from_tool_result(
            "E1",
            "run_shell",
            "Check RAM",
            "Mem: 32768",
            0,
        )];

        let formatted = format_evidence(&evidence);
        assert_eq!(formatted.len(), 1);
        assert_eq!(formatted[0]["id"], "E1");
        assert_eq!(formatted[0]["success"], true);
    }
}
