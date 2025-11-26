//! Brain v8 Contracts - Pure data types
//!
//! Single output schema for all LLM responses.
//! No planner/interpreter split - just think or answer.

use serde::{Deserialize, Serialize};

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
    /// Arguments for the tool (optional key-value pairs)
    #[serde(default)]
    pub arguments: std::collections::HashMap<String, String>,
    /// Why this tool is needed
    pub why: String,
}

/// The unified output from the brain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainOutput {
    /// Current mode: think (need tools) or answer (done)
    pub mode: BrainMode,
    /// The answer (partial during think, final during answer)
    #[serde(default)]
    pub proposed_answer: Option<String>,
    /// Reliability score 0.0-1.0 based on evidence
    pub reliability: f32,
    /// Explanation of reasoning and uncertainties
    pub reasoning: String,
    /// Tools to run (only when mode=think)
    #[serde(default)]
    pub tool_requests: Vec<ToolRequest>,
}

impl BrainOutput {
    /// Check if this is a final answer
    pub fn is_final(&self) -> bool {
        self.mode == BrainMode::Answer
    }

    /// Get the answer text
    pub fn answer(&self) -> &str {
        self.proposed_answer.as_deref().unwrap_or("")
    }
}

/// Result of executing a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Tool that was executed
    pub tool: String,
    /// Whether execution succeeded
    pub success: bool,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Exit code
    pub exit_code: i32,
}

/// Evidence bundle passed to the brain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    /// Results from tool executions
    pub tool_results: Vec<ToolResult>,
}

impl Evidence {
    pub fn new() -> Self {
        Self { tool_results: Vec::new() }
    }

    pub fn add(&mut self, result: ToolResult) {
        self.tool_results.push(result);
    }

    pub fn is_empty(&self) -> bool {
        self.tool_results.is_empty()
    }
}

impl Default for Evidence {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brain_output_serialization() {
        let output = BrainOutput {
            mode: BrainMode::Think,
            proposed_answer: None,
            reliability: 0.0,
            reasoning: "Need RAM info".to_string(),
            tool_requests: vec![ToolRequest {
                tool: "mem_info".to_string(),
                arguments: Default::default(),
                why: "Get memory statistics".to_string(),
            }],
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"mode\":\"think\""));
        assert!(json.contains("\"tool\":\"mem_info\""));
    }

    #[test]
    fn test_final_answer() {
        let output = BrainOutput {
            mode: BrainMode::Answer,
            proposed_answer: Some("32 GB RAM".to_string()),
            reliability: 0.95,
            reasoning: "Verified from /proc/meminfo".to_string(),
            tool_requests: vec![],
        };

        assert!(output.is_final());
        assert_eq!(output.answer(), "32 GB RAM");
    }
}
