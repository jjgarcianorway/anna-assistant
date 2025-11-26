//! Brain v8 Orchestrator - Pure LLM-driven loop
//!
//! Single LLM. Single prompt. Think→Answer loop.
//! No planner. No interpreter. Just the brain.

use crate::brain_v8::contracts::{BrainOutput, BrainMode, Evidence, ToolResult};
use crate::brain_v8::prompt::{build_system_prompt, build_user_message, OUTPUT_SCHEMA};
use crate::brain_v8::tools::{ToolCatalog, execute_tool};
use crate::llm_client::{LlmConfig, HttpLlmClient, LlmClient};
use crate::telemetry::SystemTelemetry;
use anyhow::{Result, anyhow};

/// Maximum iterations of the think loop
const MAX_ITERATIONS: usize = 5;

/// The brain orchestrator
pub struct BrainOrchestrator {
    llm_client: HttpLlmClient,
    tool_catalog: ToolCatalog,
    system_prompt: String,
}

impl BrainOrchestrator {
    /// Create a new orchestrator
    pub fn new(config: LlmConfig) -> Result<Self> {
        let llm_client = HttpLlmClient::new(config)?;
        let tool_catalog = ToolCatalog::new();
        let system_prompt = build_system_prompt(&tool_catalog.to_prompt_string());

        Ok(Self {
            llm_client,
            tool_catalog,
            system_prompt,
        })
    }

    /// Process a query through the think→answer loop
    pub fn process(&self, query: &str, telemetry: &SystemTelemetry) -> Result<String> {
        let telemetry_summary = self.build_telemetry_summary(telemetry);
        let mut evidence = Evidence::new();
        let mut iterations = 0;
        let mut last_tools_requested: Vec<String> = Vec::new();

        loop {
            iterations += 1;

            // Smart early exit: if we have evidence and LLM keeps requesting same tools
            if iterations > 2 && !evidence.is_empty() {
                // After iteration 2 with evidence, force an answer
                return self.synthesize_answer_from_evidence(query, &evidence, &telemetry_summary);
            }

            if iterations > MAX_ITERATIONS {
                return self.synthesize_answer_from_evidence(query, &evidence, &telemetry_summary);
            }

            // Build the user message with current evidence
            let evidence_str = self.format_evidence(&evidence);
            let user_message = build_user_message(query, &telemetry_summary, &evidence_str);

            // Call the LLM
            let response = self.llm_client.call_json(
                &self.system_prompt,
                &user_message,
                OUTPUT_SCHEMA,
            )?;

            // Parse the response
            let output = self.parse_output(&response)?;

            match output.mode {
                BrainMode::Answer => {
                    // We have our answer
                    let answer = output.proposed_answer.unwrap_or_else(|| {
                        if evidence.is_empty() {
                            "I cannot determine this with the available evidence.".to_string()
                        } else {
                            // Try to synthesize from evidence
                            return self.synthesize_answer_from_evidence(query, &evidence, &telemetry_summary)
                                .unwrap_or_else(|_| "Unable to synthesize answer.".to_string());
                        }
                    });

                    // Add reliability context if low
                    if output.reliability < 0.5 {
                        return Ok(format!(
                            "{}\n\n(Reliability: {:.0}% - {})",
                            answer,
                            output.reliability * 100.0,
                            output.reasoning
                        ));
                    }
                    return Ok(answer);
                }
                BrainMode::Think => {
                    // Execute requested tools
                    if output.tool_requests.is_empty() {
                        // LLM said think but no tools - use evidence we have
                        if !evidence.is_empty() {
                            return self.synthesize_answer_from_evidence(query, &evidence, &telemetry_summary);
                        }
                        return Ok(output.proposed_answer.unwrap_or_else(|| {
                            "I need more information but don't know which tools to use.".to_string()
                        }));
                    }

                    // Check for repeated tool requests (stuck in loop)
                    let current_tools: Vec<String> = output.tool_requests.iter()
                        .map(|r| r.tool.clone())
                        .collect();

                    if current_tools == last_tools_requested && !evidence.is_empty() {
                        // Same tools requested again - force answer
                        return self.synthesize_answer_from_evidence(query, &evidence, &telemetry_summary);
                    }
                    last_tools_requested = current_tools;

                    for request in &output.tool_requests {
                        let result = execute_tool(
                            &self.tool_catalog,
                            &request.tool,
                            &request.arguments,
                        );
                        evidence.add(result);
                    }
                }
            }
        }
    }

    /// Synthesize an answer from collected evidence when LLM fails to answer properly
    fn synthesize_answer_from_evidence(
        &self,
        query: &str,
        evidence: &Evidence,
        _telemetry: &str,
    ) -> Result<String> {
        // Build a simpler prompt to just interpret the evidence
        let interpret_prompt = format!(
            r#"Answer the user's question based ONLY on this evidence. Give a direct answer.

QUESTION: {}

EVIDENCE:
{}

Respond with JSON: {{"mode":"answer","proposed_answer":"your answer","reliability":0.9,"reasoning":"based on evidence"}}"#,
            query,
            self.format_evidence(evidence)
        );

        // Simple schema for answer-only response
        let simple_schema = r#"{"type":"object","properties":{"proposed_answer":{"type":"string"}}}"#;

        // Try to get an LLM answer
        match self.llm_client.call_json(&self.system_prompt, &interpret_prompt, simple_schema) {
            Ok(response) => {
                if let Some(answer) = response.get("proposed_answer").and_then(|a| a.as_str()) {
                    if !answer.is_empty() && answer.len() > 5 {
                        return Ok(answer.to_string());
                    }
                }
                // Fallback to direct extraction
                Ok(self.extract_simple_answer(query, evidence))
            }
            Err(_) => Ok(self.extract_simple_answer(query, evidence)),
        }
    }

    /// Extract a simple answer directly from evidence when LLM fails
    fn extract_simple_answer(&self, query: &str, evidence: &Evidence) -> String {
        let query_lower = query.to_lowercase();

        // Package installation queries - handle both success and failure
        if query_lower.contains("installed") {
            for result in &evidence.tool_results {
                if result.tool == "pacman_query" {
                    // Extract package name from query (e.g., "is steam installed?" -> "steam")
                    let words: Vec<&str> = query_lower.split_whitespace().collect();
                    let package_name = words.iter()
                        .find(|w| !["is", "installed", "installed?", "?", "do", "i", "have"].contains(w))
                        .unwrap_or(&"the package");

                    if result.success && !result.stdout.is_empty() {
                        // Package found - extract version
                        if let Some(line) = result.stdout.lines().next() {
                            return format!("Yes, {} is installed ({})", package_name, line.trim());
                        }
                        return format!("Yes, {} is installed.", package_name);
                    } else {
                        // Package not found
                        return format!("No, {} is not installed.", package_name);
                    }
                }
            }
        }

        for result in &evidence.tool_results {
            if !result.success || result.stdout.is_empty() {
                continue;
            }

            // RAM queries
            if query_lower.contains("ram") || query_lower.contains("memory") {
                if result.tool == "mem_info" {
                    // Parse free -h output
                    for line in result.stdout.lines() {
                        if line.starts_with("Mem:") {
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if parts.len() >= 2 {
                                return format!("You have {} of RAM.", parts[1]);
                            }
                        }
                    }
                }
            }

            // CPU queries
            if query_lower.contains("cpu") || query_lower.contains("processor") {
                if result.tool == "cpu_info" {
                    for line in result.stdout.lines() {
                        if line.contains("Model name:") {
                            if let Some(name) = line.split(':').nth(1) {
                                return format!("Your CPU is: {}", name.trim());
                            }
                        }
                    }
                }
            }

            // GPU queries
            if query_lower.contains("gpu") || query_lower.contains("graphics") {
                if result.tool == "gpu_info" {
                    for line in result.stdout.lines() {
                        let line_lower = line.to_lowercase();
                        if line_lower.contains("vga") || line_lower.contains("3d controller") {
                            return format!("GPU: {}", line.trim());
                        }
                    }
                }
            }
        }

        // Fallback: show raw evidence
        format!(
            "Based on the evidence I collected:\n{}",
            self.format_evidence(evidence)
        )
    }

    /// Build a summary of telemetry for the prompt
    fn build_telemetry_summary(&self, telemetry: &SystemTelemetry) -> String {
        let mut lines = Vec::new();

        // Hardware - clear labels
        lines.push(format!("cpu_model: {}", telemetry.hardware.cpu_model));
        lines.push(format!("total_ram_mb: {}", telemetry.hardware.total_ram_mb));
        lines.push(format!("machine_type: {:?}", telemetry.hardware.machine_type));
        lines.push(format!("cpu_cores: {}", telemetry.cpu.cores));

        // Desktop
        if let Some(desktop) = &telemetry.desktop {
            if let Some(de) = &desktop.de_name {
                lines.push(format!("desktop_environment: {}", de));
            }
            if let Some(ds) = &desktop.display_server {
                lines.push(format!("display_server: {}", ds));
            }
        }

        lines.join("\n")
    }

    /// Format evidence for the prompt
    fn format_evidence(&self, evidence: &Evidence) -> String {
        if evidence.is_empty() {
            return String::new();
        }

        evidence.tool_results
            .iter()
            .map(|r| {
                // Special handling for pacman_query - exit code 1 means "not found"
                if r.tool == "pacman_query" && !r.success && r.stdout.is_empty() {
                    return format!("=== {} ===\nPackage NOT FOUND (not installed)", r.tool);
                }

                let status = if r.success { "SUCCESS" } else { "FAILED" };
                let output = if r.success {
                    if r.stdout.is_empty() {
                        "(no output)".to_string()
                    } else {
                        truncate(&r.stdout, 2000)
                    }
                } else {
                    format!("Error: {}", r.stderr)
                };
                format!("=== {} ({}) ===\n{}", r.tool, status, output)
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// Parse the LLM response into BrainOutput
    fn parse_output(&self, response: &serde_json::Value) -> Result<BrainOutput> {
        // Parse mode
        let mode = match response.get("mode").and_then(|m| m.as_str()) {
            Some("think") => BrainMode::Think,
            Some("answer") => BrainMode::Answer,
            _ => {
                // Default to answer if unclear
                BrainMode::Answer
            }
        };

        // Parse proposed_answer
        let proposed_answer = response
            .get("proposed_answer")
            .and_then(|a| a.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        // Parse reliability
        let reliability = response
            .get("reliability")
            .and_then(|r| r.as_f64())
            .map(|r| r as f32)
            .unwrap_or(0.5);

        // Parse reasoning
        let reasoning = response
            .get("reasoning")
            .and_then(|r| r.as_str())
            .unwrap_or("No reasoning provided")
            .to_string();

        // Parse tool_requests
        let tool_requests = response
            .get("tool_requests")
            .and_then(|t| t.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        let tool = item.get("tool")?.as_str()?.to_string();
                        let why = item.get("why")
                            .and_then(|w| w.as_str())
                            .unwrap_or("")
                            .to_string();
                        let arguments = item.get("arguments")
                            .and_then(|a| a.as_object())
                            .map(|obj| {
                                obj.iter()
                                    .filter_map(|(k, v)| {
                                        Some((k.clone(), v.as_str()?.to_string()))
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();

                        Some(crate::brain_v8::contracts::ToolRequest {
                            tool,
                            arguments,
                            why,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(BrainOutput {
            mode,
            proposed_answer,
            reliability,
            reasoning,
            tool_requests,
        })
    }
}

/// Truncate a string to a maximum length
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...[truncated]", &s[..max_len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_think_output() {
        let json = serde_json::json!({
            "mode": "think",
            "proposed_answer": null,
            "reliability": 0.0,
            "reasoning": "Need memory info",
            "tool_requests": [
                {
                    "tool": "mem_info",
                    "arguments": {},
                    "why": "Get RAM details"
                }
            ]
        });

        // Create a mock config for testing
        let config = LlmConfig::default();
        let orchestrator = BrainOrchestrator {
            llm_client: HttpLlmClient::new(config).unwrap(),
            tool_catalog: ToolCatalog::new(),
            system_prompt: String::new(),
        };

        let output = orchestrator.parse_output(&json).unwrap();
        assert_eq!(output.mode, BrainMode::Think);
        assert!(output.proposed_answer.is_none());
        assert_eq!(output.tool_requests.len(), 1);
        assert_eq!(output.tool_requests[0].tool, "mem_info");
    }

    #[test]
    fn test_parse_answer_output() {
        let json = serde_json::json!({
            "mode": "answer",
            "proposed_answer": "You have 32 GB of RAM",
            "reliability": 0.95,
            "reasoning": "Verified from free -h output",
            "tool_requests": []
        });

        let config = LlmConfig::default();
        let orchestrator = BrainOrchestrator {
            llm_client: HttpLlmClient::new(config).unwrap(),
            tool_catalog: ToolCatalog::new(),
            system_prompt: String::new(),
        };

        let output = orchestrator.parse_output(&json).unwrap();
        assert_eq!(output.mode, BrainMode::Answer);
        assert_eq!(output.answer(), "You have 32 GB of RAM");
        assert!(output.reliability > 0.9);
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 5), "hello...[truncated]");
    }
}
