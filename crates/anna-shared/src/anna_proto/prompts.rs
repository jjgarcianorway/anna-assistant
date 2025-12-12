//! Model Prompts with Framing (Part G) - v0.0.436.
//!
//! Every model prompt enforces:
//! - Output MUST be wrapped in <<<ANNA_PROTO_V1>>> ... <<<END_ANNA_PROTO_V1>>>
//! - JSON MUST conform to ModelResultEnvelope schema
//! - NO text outside the frame

use super::framing::{PROTO_END, PROTO_START};

/// Protocol instruction that MUST be included in every model prompt.
pub const PROTOCOL_INSTRUCTION: &str = r#"
## OUTPUT FORMAT (MANDATORY)

You MUST wrap your response in EXACTLY this format:

<<<ANNA_PROTO_V1>>>
{json_response}
<<<END_ANNA_PROTO_V1>>>

Rules:
1. Output EXACTLY ONE frame - no text before or after
2. JSON must be valid and complete
3. Do NOT add any commentary outside the frame
4. Do NOT use markdown code blocks inside the frame
"#;

/// JSON schema instruction for the envelope.
pub const ENVELOPE_SCHEMA: &str = r#"
## JSON SCHEMA

{
  "ok": boolean,           // true if task succeeded
  "role": "junior"|"senior"|"translator",
  "ticket_id": "DSK-XXX",  // ticket being answered
  "confidence": 0.0-1.0,   // your confidence level
  "summary": "string",     // human-readable answer (if ok=true)
  "claims": [              // factual claims made
    {
      "text": "claim text",
      "supports": ["evidence_id_1"]  // evidence IDs supporting this
    }
  ],
  "next_actions": [        // proposed next steps
    {
      "type": "probe"|"ask_user"|"propose_change"|"install_helper",
      "payload": {...},
      "risk": "safe"|"risky",
      "requires_confirmation": boolean
    }
  ],
  "evidence_used": [       // evidence you referenced
    {
      "id": "ev_xxx",
      "kind": "probe"|"man"|"help"|"wiki",
      "title": "Human readable title"
    }
  ],
  "errors": []             // only if ok=false
}
"#;

/// Build complete protocol prompt suffix.
pub fn protocol_suffix() -> String {
    format!("{}\n{}", PROTOCOL_INSTRUCTION, ENVELOPE_SCHEMA)
}

/// Build example output for the model.
pub fn example_output(ticket_id: &str, role: &str, summary: &str) -> String {
    format!(
        r#"{}
{{
  "ok": true,
  "role": "{}",
  "ticket_id": "{}",
  "confidence": 0.85,
  "summary": "{}",
  "claims": [],
  "next_actions": [],
  "evidence_used": [],
  "errors": []
}}
{}"#,
        PROTO_START, role, ticket_id, summary, PROTO_END
    )
}

/// Junior specialist prompt with protocol.
pub fn junior_prompt(ticket_id: &str, question: &str, evidence: &str) -> String {
    format!(
        r#"You are a Linux system diagnostic specialist (junior level).

## TASK
Answer the user's question based on the evidence provided.

Ticket: {}
Question: {}

## EVIDENCE
{}

## GUIDELINES
- Only claim what the evidence supports
- If unsure, set confidence < 0.7 and suggest probes
- Never fabricate system information
{}

## EXAMPLE OUTPUT
{}"#,
        ticket_id,
        question,
        evidence,
        protocol_suffix(),
        example_output(ticket_id, "junior", "Based on the evidence...")
    )
}

/// Senior specialist prompt with protocol.
pub fn senior_prompt(ticket_id: &str, question: &str, evidence: &str, context: &str) -> String {
    format!(
        r#"You are a senior Linux system expert with deep diagnostic experience.

## TASK
Provide expert analysis for this escalated ticket.

Ticket: {}
Question: {}

## CONTEXT FROM JUNIOR
{}

## EVIDENCE
{}

## GUIDELINES
- Provide root cause analysis when possible
- Reference specific evidence for each claim
- Suggest corrective actions with risk levels
- If proposing changes, set requires_confirmation=true
{}

## EXAMPLE OUTPUT
{}"#,
        ticket_id,
        question,
        context,
        evidence,
        protocol_suffix(),
        example_output(ticket_id, "senior", "Root cause analysis...")
    )
}

/// Translator prompt with protocol.
pub fn translator_prompt(ticket_id: &str, internal_response: &str) -> String {
    format!(
        r#"You are a friendly translator that converts technical responses for users.

## TASK
Translate the internal specialist response into user-friendly language.

Ticket: {}

## INTERNAL RESPONSE
{}

## GUIDELINES
- Keep it concise and helpful
- Don't expose internal details or evidence IDs
- Use simple language
- If there are errors, explain what happened simply
{}

## EXAMPLE OUTPUT
{}"#,
        ticket_id,
        internal_response,
        protocol_suffix(),
        example_output(ticket_id, "translator", "Here's what I found...")
    )
}

/// Validate that a prompt includes protocol instructions.
pub fn prompt_has_protocol(prompt: &str) -> bool {
    prompt.contains("ANNA_PROTO_V1") && prompt.contains("OUTPUT FORMAT")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_suffix() {
        let suffix = protocol_suffix();
        assert!(suffix.contains("ANNA_PROTO_V1"));
        assert!(suffix.contains("OUTPUT FORMAT"));
        assert!(suffix.contains("JSON SCHEMA"));
    }

    #[test]
    fn test_example_output() {
        let example = example_output("DSK-001", "junior", "Test summary");
        assert!(example.contains(PROTO_START));
        assert!(example.contains(PROTO_END));
        assert!(example.contains("DSK-001"));
        assert!(example.contains("junior"));
    }

    #[test]
    fn test_junior_prompt() {
        let prompt = junior_prompt("DSK-001", "Why is boot slow?", "Boot took 15s");
        assert!(prompt_has_protocol(&prompt));
        assert!(prompt.contains("DSK-001"));
        assert!(prompt.contains("boot slow"));
        assert!(prompt.contains("junior level"));
    }

    #[test]
    fn test_senior_prompt() {
        let prompt = senior_prompt(
            "DSK-002",
            "Root cause of network issues",
            "Junior said: possible DNS",
            "DNS probe: timeout",
        );
        assert!(prompt_has_protocol(&prompt));
        assert!(prompt.contains("DSK-002"));
        assert!(prompt.contains("senior"));
        assert!(prompt.contains("root cause"));
    }

    #[test]
    fn test_translator_prompt() {
        let prompt = translator_prompt("DSK-003", r#"{"ok": true, "summary": "test"}"#);
        assert!(prompt_has_protocol(&prompt));
        assert!(prompt.contains("DSK-003"));
        assert!(prompt.contains("translator"));
    }

    #[test]
    fn test_prompt_has_protocol() {
        assert!(prompt_has_protocol(&protocol_suffix()));
        assert!(!prompt_has_protocol("Just a regular prompt"));
    }
}
