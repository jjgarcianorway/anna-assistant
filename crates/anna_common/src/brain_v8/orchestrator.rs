//! Anna Brain Core v1.0 - Orchestrator
//!
//! The main loop: INPUT → LLM → TOOL REQUESTS → TOOL OUTPUT → LLM → ANSWER
//! Max 8 iterations. Reliability threshold enforcement. User question support.

use crate::brain_v8::contracts::{BrainOutput, BrainMode, ToolResult, ToolSchema};
use crate::brain_v8::prompt::{SYSTEM_PROMPT, OUTPUT_SCHEMA, build_state_message};
use crate::brain_v8::tools::{ToolCatalog, execute_tool};
use crate::llm_client::{LlmConfig, HttpLlmClient, LlmClient};
use crate::telemetry::SystemTelemetry;
use anyhow::{Result, anyhow};

/// Maximum iterations of the think loop (spec says 8)
const MAX_ITERATIONS: usize = 8;

/// Minimum reliability to accept without forcing retry
const MIN_RELIABILITY: f32 = 0.4;

/// The brain orchestrator - implements the spec loop
pub struct BrainOrchestrator {
    llm_client: HttpLlmClient,
    tool_catalog: ToolCatalog,
}

impl BrainOrchestrator {
    /// Create a new orchestrator
    pub fn new(config: LlmConfig) -> Result<Self> {
        let llm_client = HttpLlmClient::new(config)?;
        let tool_catalog = ToolCatalog::new();

        Ok(Self {
            llm_client,
            tool_catalog,
        })
    }

    /// Process a query through the brain loop
    /// Returns (answer, needs_user_input, user_question)
    pub fn process(
        &self,
        query: &str,
        telemetry: &SystemTelemetry,
        user_context: Option<&str>,
    ) -> Result<BrainResult> {
        let telemetry_json = self.telemetry_to_json(telemetry);
        let tool_schemas: Vec<ToolSchema> = self.tool_catalog.to_schema_list().to_vec();
        let mut tool_history: Vec<ToolResult> = Vec::new();
        let mut iterations = 0;

        // If user provided additional context, add it to telemetry
        let telemetry_with_context = if let Some(ctx) = user_context {
            let mut t = telemetry_json.clone();
            if let Some(obj) = t.as_object_mut() {
                obj.insert("user_provided_info".to_string(), serde_json::json!(ctx));
            }
            t
        } else {
            telemetry_json.clone()
        };

        loop {
            iterations += 1;

            if iterations > MAX_ITERATIONS {
                return Ok(BrainResult::Answer {
                    text: "I cannot answer confidently with the tools available.".to_string(),
                    reliability: 0.0,
                });
            }

            // Build state message for LLM
            let state_msg = build_state_message(
                query,
                &telemetry_with_context,
                &tool_history,
                &tool_schemas,
            );

            // Call LLM
            let response = self.llm_client.call_json(
                SYSTEM_PROMPT,
                &state_msg,
                OUTPUT_SCHEMA,
            )?;

            // Parse response
            let output = self.parse_output(&response)?;

            match output.mode {
                BrainMode::Answer => {
                    let answer = output.final_answer.unwrap_or_else(|| {
                        "I cannot determine this with the available evidence.".to_string()
                    });

                    // Include reliability in response if it's low
                    if output.reliability < 0.5 {
                        return Ok(BrainResult::Answer {
                            text: format!(
                                "{}\n\n(Reliability: {:.0}% - {})",
                                answer,
                                output.reliability * 100.0,
                                output.reasoning
                            ),
                            reliability: output.reliability,
                        });
                    }

                    return Ok(BrainResult::Answer {
                        text: answer,
                        reliability: output.reliability,
                    });
                }
                BrainMode::Think => {
                    // Safety: If we have clear evidence but LLM keeps thinking, force answer
                    if iterations >= 3 && !tool_history.is_empty() {
                        if let Some(answer) = self.extract_obvious_answer(query, &tool_history) {
                            return Ok(BrainResult::Answer {
                                text: answer,
                                reliability: 0.8,
                            });
                        }
                    }

                    // Check if user input is needed
                    if output.needs_user_input() {
                        let question = output.missing_user_info()
                            .unwrap_or("Please provide additional information")
                            .to_string();
                        return Ok(BrainResult::NeedsUserInput { question });
                    }

                    // Check reliability threshold
                    if output.reliability < MIN_RELIABILITY && output.tool_requests.is_empty() {
                        // Force LLM to try harder
                        tool_history.push(ToolResult {
                            tool: "_system".to_string(),
                            arguments: Default::default(),
                            stdout: format!(
                                "Your reliability ({:.1}) is too low. What information is missing? Which tool should I run?",
                                output.reliability
                            ),
                            stderr: String::new(),
                            exit_code: 0,
                        });
                        continue;
                    }

                    // Execute requested tools
                    if output.tool_requests.is_empty() {
                        // LLM said think but no tools - force answer
                        return Ok(BrainResult::Answer {
                            text: output.final_answer.unwrap_or_else(|| {
                                "I need more information but don't know which tools to use.".to_string()
                            }),
                            reliability: output.reliability,
                        });
                    }

                    // Execute each tool and add to history
                    for request in &output.tool_requests {
                        let result = execute_tool(&request.tool, &request.arguments);
                        tool_history.push(result);
                    }
                }
            }
        }
    }

    /// Extract obvious answer from tool history when LLM fails to interpret
    fn extract_obvious_answer(&self, query: &str, tool_history: &[ToolResult]) -> Option<String> {
        let query_lower = query.to_lowercase();

        // Package installation queries
        if query_lower.contains("installed") {
            for result in tool_history {
                if result.stdout.contains("local/") && result.exit_code == 0 {
                    // Extract package name from query
                    let package = query_lower
                        .split_whitespace()
                        .find(|w| !["is", "installed", "installed?", "?", "do", "i", "have"].contains(w))
                        .unwrap_or("the package");

                    // Extract version from output
                    if let Some(line) = result.stdout.lines().next() {
                        return Some(format!(
                            "Yes, {} is installed. Evidence: {} showed \"{}\"",
                            package, result.tool, line.trim()
                        ));
                    }
                }
                // Check for "which" command success
                if result.arguments.get("command").map(|c| c.starts_with("which")).unwrap_or(false)
                    && result.exit_code == 0
                    && !result.stdout.is_empty()
                {
                    let package = query_lower
                        .split_whitespace()
                        .find(|w| !["is", "installed", "installed?", "?", "do", "i", "have"].contains(w))
                        .unwrap_or("the package");
                    return Some(format!(
                        "Yes, {} is installed. Evidence: found at {}",
                        package, result.stdout.trim()
                    ));
                }
            }
            // Check for failed package query (not installed)
            // But only if we don't have any successful results
            let has_successful_package = tool_history.iter().any(|r| {
                r.stdout.contains("local/") && r.exit_code == 0
            });
            if !has_successful_package {
                for result in tool_history {
                    if let Some(cmd) = result.arguments.get("command") {
                        // pacman -Qs returns exit code 1 when nothing found
                        if cmd.contains("pacman -Qs") && (result.exit_code != 0 || result.stdout.trim().is_empty()) {
                            let package = query_lower
                                .split_whitespace()
                                .find(|w| !["is", "installed", "installed?", "?", "do", "i", "have"].contains(w))
                                .unwrap_or("the package");
                            return Some(format!("No, {} is not installed.", package));
                        }
                        // which command fails when not found
                        if cmd.starts_with("which ") && (result.exit_code != 0 || result.stdout.trim().is_empty()) {
                            let package = query_lower
                                .split_whitespace()
                                .find(|w| !["is", "installed", "installed?", "?", "do", "i", "have"].contains(w))
                                .unwrap_or("the package");
                            return Some(format!("No, {} is not installed.", package));
                        }
                    }
                }
            }
        }

        // RAM queries
        if query_lower.contains("ram") || query_lower.contains("memory") {
            for result in tool_history {
                if result.stdout.contains("Mem:") {
                    for line in result.stdout.lines() {
                        if line.starts_with("Mem:") {
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if parts.len() >= 2 {
                                return Some(format!(
                                    "You have {} of RAM. Evidence: {} showed \"{}\"",
                                    parts[1], result.tool, line.trim()
                                ));
                            }
                        }
                    }
                }
            }
        }

        // CPU queries
        if query_lower.contains("cpu") || query_lower.contains("processor") {
            for result in tool_history {
                for line in result.stdout.lines() {
                    if line.contains("Model name:") {
                        if let Some(name) = line.split(':').nth(1) {
                            return Some(format!(
                                "Your CPU is: {}. Evidence: {} showed this.",
                                name.trim(), result.tool
                            ));
                        }
                    }
                }
            }
        }

        None
    }

    /// Convert telemetry to JSON for the LLM
    fn telemetry_to_json(&self, telemetry: &SystemTelemetry) -> serde_json::Value {
        serde_json::json!({
            "cpu_model": telemetry.hardware.cpu_model,
            "cpu_cores": telemetry.cpu.cores,
            "total_ram_mb": telemetry.hardware.total_ram_mb,
            "machine_type": format!("{:?}", telemetry.hardware.machine_type),
            "desktop_environment": telemetry.desktop.as_ref().and_then(|d| d.de_name.clone()),
            "display_server": telemetry.desktop.as_ref().and_then(|d| d.display_server.clone()),
        })
    }

    /// Parse the LLM response into BrainOutput
    fn parse_output(&self, response: &serde_json::Value) -> Result<BrainOutput> {
        // Parse mode
        let mode = match response.get("mode").and_then(|m| m.as_str()) {
            Some("think") => BrainMode::Think,
            Some("answer") => BrainMode::Answer,
            _ => BrainMode::Answer, // Default to answer if unclear
        };

        // Parse final_answer
        let final_answer = response
            .get("final_answer")
            .and_then(|a| a.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        // Parse reliability
        let reliability = response
            .get("reliability")
            .and_then(|r| r.as_f64())
            .map(|r| r as f32)
            .unwrap_or(0.0);

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

        // Parse debug_log
        let debug_log = response
            .get("debug_log")
            .and_then(|d| d.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(BrainOutput {
            mode,
            final_answer,
            reliability,
            reasoning,
            tool_requests,
            debug_log,
        })
    }
}

/// Result from the brain loop
#[derive(Debug)]
pub enum BrainResult {
    /// Final answer with reliability
    Answer { text: String, reliability: f32 },
    /// Need user input to continue
    NeedsUserInput { question: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_think_output() {
        let json = serde_json::json!({
            "mode": "think",
            "final_answer": null,
            "reliability": 0.0,
            "reasoning": "Need memory info",
            "tool_requests": [
                {
                    "tool": "memory_info",
                    "arguments": {},
                    "why": "Get RAM details"
                }
            ],
            "debug_log": []
        });

        let config = LlmConfig::default();
        let orchestrator = BrainOrchestrator {
            llm_client: HttpLlmClient::new(config).unwrap(),
            tool_catalog: ToolCatalog::new(),
        };

        let output = orchestrator.parse_output(&json).unwrap();
        assert_eq!(output.mode, BrainMode::Think);
        assert!(output.final_answer.is_none());
        assert_eq!(output.tool_requests.len(), 1);
        assert_eq!(output.tool_requests[0].tool, "memory_info");
    }

    #[test]
    fn test_parse_answer_output() {
        let json = serde_json::json!({
            "mode": "answer",
            "final_answer": "You have 32 GB of RAM. Evidence: memory_info showed Mem: 32768",
            "reliability": 0.95,
            "reasoning": "Verified from free -m output",
            "tool_requests": [],
            "debug_log": []
        });

        let config = LlmConfig::default();
        let orchestrator = BrainOrchestrator {
            llm_client: HttpLlmClient::new(config).unwrap(),
            tool_catalog: ToolCatalog::new(),
        };

        let output = orchestrator.parse_output(&json).unwrap();
        assert_eq!(output.mode, BrainMode::Answer);
        assert!(output.answer().contains("Evidence:"));
        assert!(output.reliability > 0.9);
    }

    #[test]
    fn test_parse_user_input_needed() {
        let json = serde_json::json!({
            "mode": "think",
            "final_answer": null,
            "reliability": 0.1,
            "reasoning": "Need user preference",
            "tool_requests": [],
            "debug_log": ["Missing information that only the user can provide: preferred browser"]
        });

        let config = LlmConfig::default();
        let orchestrator = BrainOrchestrator {
            llm_client: HttpLlmClient::new(config).unwrap(),
            tool_catalog: ToolCatalog::new(),
        };

        let output = orchestrator.parse_output(&json).unwrap();
        assert!(output.needs_user_input());
    }
}
