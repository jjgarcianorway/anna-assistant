//! Anna v7 Brain - LLM Prompts
//!
//! Strict, minimal system prompts for planner and interpreter roles.
//! These prompts enforce the JSON contract and prevent hallucination.

use super::contracts::ToolDescriptor;

/// Build the planner system prompt
pub fn planner_system_prompt(tools: &[ToolDescriptor]) -> String {
    let tool_list = tools
        .iter()
        .map(|t| {
            if let Some(schema) = &t.parameters_schema {
                format!(
                    "- {}: {}\n  Parameters: {}",
                    t.name,
                    t.description,
                    serde_json::to_string(schema).unwrap_or_default()
                )
            } else {
                format!("- {}: {}", t.name, t.description)
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"You are Anna's planner. Your job is to create an execution plan for system queries.

AVAILABLE TOOLS (use ONLY these exact names):
{tool_list}

OUTPUT FORMAT (strict JSON, no other text):
{{
  "intent": "one sentence describing what the user wants",
  "subtasks": [
    {{"id": "st1", "description": "what needs to be determined"}}
  ],
  "tool_calls": [
    {{
      "subtask_id": "st1",
      "tool": "exact_tool_name_from_list",
      "parameters": {{}},
      "reason": "why this tool is needed"
    }}
  ],
  "expected_evidence": [
    {{
      "subtask_id": "st1",
      "tool": "tool_name",
      "evidence_needed": "what data this tool should provide"
    }}
  ],
  "limitations": {{
    "missing_tools": [],
    "unanswerable_parts": ""
  }}
}}

RULES:
1. Use ONLY tools from the list above. If a tool doesn't exist, put it in missing_tools.
2. Use the MINIMUM number of tools needed.
3. Do NOT invent tool names or parameters.
4. If the query cannot be answered with available tools, explain in limitations.
5. Output ONLY valid JSON. No explanations before or after."#
    )
}

/// Build the interpreter system prompt
pub fn interpreter_system_prompt() -> String {
    r#"You are Anna's interpreter. Your job is to answer the user's question using ONLY the provided evidence.

INPUT:
- user_query: The original question
- planner: The execution plan that was run
- evidence_bundle: Actual output from each tool

OUTPUT FORMAT (strict JSON, no other text):
{
  "answer": "Direct answer to the question in clear, concise English",
  "evidence_used": [
    {"tool": "tool_name", "summary": "what data was extracted from this tool"}
  ],
  "reliability": {
    "score": 0.0 to 1.0,
    "level": "LOW" or "MEDIUM" or "HIGH",
    "reason": "why this score"
  },
  "uncertainty": {
    "has_unknowns": true or false,
    "details": "what is unknown or uncertain"
  }
}

RELIABILITY SCORING:
- 0.8-1.0 (HIGH): All needed evidence found, clear answer
- 0.4-0.79 (MEDIUM): Some evidence found, minor gaps or assumptions
- 0.0-0.39 (LOW): Little evidence, major assumptions, or tool failures

CRITICAL RULES:
1. Use ONLY information from the evidence_bundle. Nothing else.
2. If a tool failed (exit_code != 0), treat its output as unavailable.
3. If information is not in the evidence, say "unknown" or "cannot determine".
4. NEVER invent device names, package names, or system properties.
5. NEVER make recommendations not directly supported by evidence.
6. Output ONLY valid JSON. No explanations before or after.
7. The answer field must be a complete sentence, not raw data.

EXAMPLES OF BAD BEHAVIOR (do NOT do these):
- Saying "You have an RTX 3070" when evidence shows "RTX 4060"
- Listing packages not in the pacman_search output
- Saying "Steam is installed" when the evidence doesn't contain "steam"
- Making up filesystem sizes not in df output"#
        .to_string()
}

/// JSON schema for planner output (used with Ollama's format parameter)
pub fn planner_json_schema() -> &'static str {
    r#"{
  "type": "object",
  "properties": {
    "intent": {"type": "string"},
    "subtasks": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "id": {"type": "string"},
          "description": {"type": "string"}
        },
        "required": ["id", "description"]
      }
    },
    "tool_calls": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "subtask_id": {"type": "string"},
          "tool": {"type": "string"},
          "parameters": {"type": "object"},
          "reason": {"type": "string"}
        },
        "required": ["subtask_id", "tool", "parameters", "reason"]
      }
    },
    "expected_evidence": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "subtask_id": {"type": "string"},
          "tool": {"type": "string"},
          "evidence_needed": {"type": "string"}
        },
        "required": ["subtask_id", "tool", "evidence_needed"]
      }
    },
    "limitations": {
      "type": "object",
      "properties": {
        "missing_tools": {"type": "array", "items": {"type": "string"}},
        "unanswerable_parts": {"type": "string"}
      },
      "required": ["missing_tools", "unanswerable_parts"]
    }
  },
  "required": ["intent", "subtasks", "tool_calls", "expected_evidence", "limitations"]
}"#
}

/// JSON schema for interpreter output
pub fn interpreter_json_schema() -> &'static str {
    r#"{
  "type": "object",
  "properties": {
    "answer": {"type": "string"},
    "evidence_used": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "tool": {"type": "string"},
          "summary": {"type": "string"}
        },
        "required": ["tool", "summary"]
      }
    },
    "reliability": {
      "type": "object",
      "properties": {
        "score": {"type": "number"},
        "level": {"type": "string"},
        "reason": {"type": "string"}
      },
      "required": ["score", "level", "reason"]
    },
    "uncertainty": {
      "type": "object",
      "properties": {
        "has_unknowns": {"type": "boolean"},
        "details": {"type": "string"}
      },
      "required": ["has_unknowns", "details"]
    }
  },
  "required": ["answer", "evidence_used", "reliability", "uncertainty"]
}"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_planner_prompt_contains_tools() {
        let tools = vec![
            ToolDescriptor {
                name: "mem_info".to_string(),
                description: "Get memory info".to_string(),
                parameters_schema: None,
            },
            ToolDescriptor {
                name: "cpu_info".to_string(),
                description: "Get CPU info".to_string(),
                parameters_schema: None,
            },
        ];

        let prompt = planner_system_prompt(&tools);
        assert!(prompt.contains("mem_info"));
        assert!(prompt.contains("cpu_info"));
        assert!(prompt.contains("ONLY these exact names"));
    }

    #[test]
    fn test_interpreter_prompt_has_rules() {
        let prompt = interpreter_system_prompt();
        assert!(prompt.contains("NEVER invent"));
        assert!(prompt.contains("ONLY information from the evidence"));
        assert!(prompt.contains("reliability"));
    }

    #[test]
    fn test_json_schemas_are_valid() {
        // Just verify they parse as JSON
        let planner: serde_json::Value =
            serde_json::from_str(planner_json_schema()).expect("Invalid planner schema");
        let interpreter: serde_json::Value =
            serde_json::from_str(interpreter_json_schema()).expect("Invalid interpreter schema");

        assert_eq!(planner["type"], "object");
        assert_eq!(interpreter["type"], "object");
    }
}
