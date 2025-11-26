//! Anna Brain Core v1.0 - Contracts
//!
//! Strict JSON protocol between Anna (Rust) and LLM.
//! No hardcoded logic. The LLM decides what tools to run.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The mode of the brain's response
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BrainMode {
    /// Need more information - request tools
    Think,
    /// Ready to answer - no more tools needed
    Answer,
}

/// A tool request from the brain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRequest {
    /// Tool name from catalog
    pub tool: String,
    /// Arguments for the tool
    #[serde(default)]
    pub arguments: HashMap<String, String>,
    /// Why this tool is needed
    pub why: String,
}

/// The unified output from the brain (matches spec exactly)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainOutput {
    /// "think" or "answer"
    pub mode: BrainMode,
    /// The answer (null during think, string during answer)
    pub final_answer: Option<String>,
    /// Reliability score 0.0-1.0 based on evidence
    pub reliability: f32,
    /// Explanation of how reliability was determined
    pub reasoning: String,
    /// Tools to run (only when mode=think)
    #[serde(default)]
    pub tool_requests: Vec<ToolRequest>,
    /// Debug entries for transparency
    #[serde(default)]
    pub debug_log: Vec<String>,
}

impl BrainOutput {
    /// Check if this is a final answer
    pub fn is_final(&self) -> bool {
        self.mode == BrainMode::Answer
    }

    /// Get the answer text
    pub fn answer(&self) -> &str {
        self.final_answer.as_deref().unwrap_or("")
    }

    /// Check if user input is needed
    pub fn needs_user_input(&self) -> bool {
        self.mode == BrainMode::Think
            && self.tool_requests.is_empty()
            && self.debug_log.iter().any(|s| s.contains("only the user can provide"))
    }

    /// Extract what info is needed from user
    pub fn missing_user_info(&self) -> Option<&str> {
        for entry in &self.debug_log {
            if entry.contains("only the user can provide:") {
                return entry.strip_prefix("Missing information that only the user can provide: ");
            }
        }
        None
    }
}

/// Result of executing a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Tool that was executed
    pub tool: String,
    /// Arguments used
    #[serde(default)]
    pub arguments: HashMap<String, String>,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Exit code
    pub exit_code: i32,
}

impl ToolResult {
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }
}

/// State sent to LLM each iteration (matches spec exactly)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainState {
    /// The user's question
    pub question: String,
    /// System telemetry snapshot
    pub telemetry: serde_json::Value,
    /// History of tool executions
    pub tool_history: Vec<ToolResult>,
    /// Available tools
    pub tool_catalog: Vec<ToolSchema>,
}

/// Tool definition in catalog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    /// Tool name
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Parameter schema
    pub schema: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brain_output_serialization() {
        let output = BrainOutput {
            mode: BrainMode::Think,
            final_answer: None,
            reliability: 0.0,
            reasoning: "Need to run shell command".to_string(),
            tool_requests: vec![ToolRequest {
                tool: "run_shell".to_string(),
                arguments: [("command".to_string(), "free -m".to_string())].into_iter().collect(),
                why: "Get memory statistics".to_string(),
            }],
            debug_log: vec![],
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"mode\":\"think\""));
        assert!(json.contains("run_shell"));
    }

    #[test]
    fn test_final_answer() {
        let output = BrainOutput {
            mode: BrainMode::Answer,
            final_answer: Some("32 GB RAM. Evidence: free -m showed Mem: 32768".to_string()),
            reliability: 0.95,
            reasoning: "Verified from free -m output".to_string(),
            tool_requests: vec![],
            debug_log: vec![],
        };

        assert!(output.is_final());
        assert!(output.answer().contains("Evidence:"));
    }

    #[test]
    fn test_needs_user_input() {
        let output = BrainOutput {
            mode: BrainMode::Think,
            final_answer: None,
            reliability: 0.1,
            reasoning: "Cannot determine without user input".to_string(),
            tool_requests: vec![],
            debug_log: vec![
                "Missing information that only the user can provide: preferred browser".to_string()
            ],
        };

        assert!(output.needs_user_input());
    }
}
