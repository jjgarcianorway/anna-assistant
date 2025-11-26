//! Anna Brain v10.0.0 - Orchestrator
//!
//! The main loop: INPUT → LLM → TOOL REQUESTS → TOOL OUTPUT → LLM → ANSWER
//! Max 8 iterations. Evidence-based answers. Explicit reliability labels.

use crate::brain_v10::contracts::{
    BrainSession, BrainStep, ReliabilityLabel, SessionResult, StepType, ToolRequest,
};
use crate::brain_v10::prompt::{build_state_message, SYSTEM_PROMPT, OUTPUT_SCHEMA};
use crate::brain_v10::tools::{execute_tool, output_to_evidence, ToolCatalog, ToolSchema};
use crate::llm_client::{HttpLlmClient, LlmClient, LlmConfig};
use crate::telemetry::SystemTelemetry;
use anyhow::{anyhow, Result};

/// Maximum iterations of the think loop
const MAX_ITERATIONS: usize = 8;

/// Minimum reliability to accept an answer
const MIN_RELIABILITY: f32 = 0.4;

/// Result from the brain orchestrator
#[derive(Debug, Clone)]
pub enum BrainResult {
    /// Final answer with reliability
    Answer {
        text: String,
        reliability: f32,
        label: ReliabilityLabel,
    },
    /// Need user input to continue
    NeedsUserInput { question: String },
}

/// The brain orchestrator - implements the v10 spec loop
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
    pub fn process(
        &self,
        query: &str,
        telemetry: &SystemTelemetry,
        user_context: Option<&str>,
    ) -> Result<BrainResult> {
        let telemetry_json = self.telemetry_to_json(telemetry);

        // Create session with telemetry as E0 evidence
        let mut session = BrainSession::new(
            query,
            telemetry_json.clone(),
            self.tool_catalog.to_schema_list().to_vec(),
        );

        // Add telemetry as initial evidence E0
        session.add_evidence(
            "telemetry",
            "Pre-collected system snapshot",
            &serde_json::to_string_pretty(&telemetry_json).unwrap_or_default(),
            0,
        );

        // Add user context if provided
        if let Some(ctx) = user_context {
            session.add_evidence("user_input", "User-provided information", ctx, 0);
        }

        // Main loop
        loop {
            session.next_iteration();

            if session.iteration > MAX_ITERATIONS {
                // Return best effort answer
                return Ok(BrainResult::Answer {
                    text: format!(
                        "I could not answer confidently within {} iterations. \
                        The available evidence was insufficient.",
                        MAX_ITERATIONS
                    ),
                    reliability: 0.0,
                    label: ReliabilityLabel::VeryLow,
                });
            }

            // Build state message
            let state_msg = build_state_message(&session);

            // Call LLM
            let response = self.llm_client.call_json(SYSTEM_PROMPT, &state_msg, OUTPUT_SCHEMA)?;

            // Parse response
            let step = self.parse_step(&response)?;

            match step.step_type {
                StepType::FinalAnswer => {
                    let reliability = step.reliability;
                    let label = ReliabilityLabel::from_score(reliability);
                    let answer = step.answer.unwrap_or_else(|| {
                        "I could not determine this with confidence.".to_string()
                    });

                    // Format answer with evidence references
                    let formatted = self.format_answer_with_evidence(
                        &answer,
                        reliability,
                        &label,
                    );

                    return Ok(BrainResult::Answer {
                        text: formatted,
                        reliability,
                        label,
                    });
                }

                StepType::AskUser => {
                    let question = step
                        .user_question
                        .unwrap_or_else(|| "Please provide more information.".to_string());
                    return Ok(BrainResult::NeedsUserInput { question });
                }

                StepType::DecideTool => {
                    // Safety: If stuck in loop with no progress
                    if session.iteration >= 4 && step.reliability < MIN_RELIABILITY {
                        if let Some(fallback) = self.try_fallback_answer(query, &session) {
                            return Ok(fallback);
                        }
                    }

                    // Execute requested tool
                    if let Some(ref req) = step.tool_request {
                        let output = execute_tool(
                            &req.tool,
                            &req.arguments,
                            Some(&session.telemetry),
                        );

                        // Add result as evidence
                        let evidence_id = format!("E{}", session.evidence.len());
                        let description = req.why.clone();
                        session.add_evidence(
                            &req.tool,
                            &description,
                            if output.stdout.is_empty() && !output.stderr.is_empty() {
                                &output.stderr
                            } else {
                                &output.stdout
                            },
                            output.exit_code,
                        );
                    } else {
                        // LLM said decide_tool but no request - force answer
                        return Ok(BrainResult::Answer {
                            text: "I need more information but don't know which tool to use."
                                .to_string(),
                            reliability: 0.1,
                            label: ReliabilityLabel::VeryLow,
                        });
                    }
                }
            }
        }
    }

    /// Try to extract an obvious answer from evidence when LLM fails
    fn try_fallback_answer(&self, query: &str, session: &BrainSession) -> Option<BrainResult> {
        let query_lower = query.to_lowercase();

        // Package queries - extract the package name from query
        if query_lower.contains("installed") {
            // Extract package name from query
            let package_name = query_lower
                .split_whitespace()
                .find(|w| !["is", "installed", "installed?", "?", "do", "i", "have"].contains(w))
                .unwrap_or("");

            if !package_name.is_empty() {
                for evidence in &session.evidence {
                    if evidence.source == "run_shell" {
                        // Check if this evidence is about the package we're looking for
                        let is_relevant = evidence.description.to_lowercase().contains(package_name)
                            || evidence.content.to_lowercase().contains(package_name);

                        if !is_relevant {
                            continue;
                        }

                        // Positive: package found
                        if evidence.content.contains("local/") && evidence.is_success() {
                            let pkg_line = evidence.content.lines().next()?;
                            return Some(BrainResult::Answer {
                                text: format!(
                                    "Yes, {} is installed [{}].\nVersion: {}",
                                    package_name, evidence.id, pkg_line.trim()
                                ),
                                reliability: 0.85,
                                label: ReliabilityLabel::Medium,
                            });
                        }

                        // Negative: pacman query returned empty (exit 0 but no output)
                        if evidence.content.trim().is_empty() && evidence.is_success() {
                            return Some(BrainResult::Answer {
                                text: format!(
                                    "No, {} is not installed [{}]. \
                                    The package query returned no results.",
                                    package_name, evidence.id
                                ),
                                reliability: 0.85,
                                label: ReliabilityLabel::Medium,
                            });
                        }

                        // Negative: exit code 1 (package not found)
                        if !evidence.is_success() {
                            return Some(BrainResult::Answer {
                                text: format!(
                                    "No, {} is not installed [{}].",
                                    package_name, evidence.id
                                ),
                                reliability: 0.85,
                                label: ReliabilityLabel::Medium,
                            });
                        }
                    }
                }
            }
        }

        // RAM queries
        if query_lower.contains("ram") || query_lower.contains("memory") {
            for evidence in &session.evidence {
                if evidence.content.contains("Mem:") {
                    for line in evidence.content.lines() {
                        if line.starts_with("Mem:") {
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if parts.len() >= 2 {
                                return Some(BrainResult::Answer {
                                    text: format!(
                                        "You have {} MB of RAM [{}].",
                                        parts[1], evidence.id
                                    ),
                                    reliability: 0.9,
                                    label: ReliabilityLabel::High,
                                });
                            }
                        }
                    }
                }
            }
        }

        // CPU queries
        if query_lower.contains("cpu") || query_lower.contains("processor") {
            for evidence in &session.evidence {
                for line in evidence.content.lines() {
                    if line.contains("Model name:") {
                        if let Some(name) = line.split(':').nth(1) {
                            return Some(BrainResult::Answer {
                                text: format!("Your CPU is: {} [{}].", name.trim(), evidence.id),
                                reliability: 0.9,
                                label: ReliabilityLabel::High,
                            });
                        }
                    }
                }
            }
        }

        None
    }

    /// Format answer with evidence citations
    fn format_answer_with_evidence(
        &self,
        answer: &str,
        reliability: f32,
        label: &ReliabilityLabel,
    ) -> String {
        let mut output = answer.to_string();

        // Add reliability footer for non-HIGH confidence
        if *label != ReliabilityLabel::High {
            output.push_str(&format!(
                "\n\n{}  Confidence: {} ({:.0}%)",
                label.emoji(),
                label.display(),
                reliability * 100.0
            ));
        }

        output
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

    /// Parse the LLM response into BrainStep
    fn parse_step(&self, response: &serde_json::Value) -> Result<BrainStep> {
        // Parse step_type
        let step_type = match response.get("step_type").and_then(|s| s.as_str()) {
            Some("decide_tool") => StepType::DecideTool,
            Some("final_answer") => StepType::FinalAnswer,
            Some("ask_user") => StepType::AskUser,
            _ => StepType::FinalAnswer, // Default to final answer
        };

        // Parse tool_request
        let tool_request = response
            .get("tool_request")
            .and_then(|t| t.as_object())
            .map(|obj| {
                ToolRequest {
                    tool: obj
                        .get("tool")
                        .and_then(|t| t.as_str())
                        .unwrap_or("")
                        .to_string(),
                    arguments: obj
                        .get("arguments")
                        .and_then(|a| a.as_object())
                        .map(|args| {
                            args.iter()
                                .filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_string())))
                                .collect()
                        })
                        .unwrap_or_default(),
                    why: obj
                        .get("why")
                        .and_then(|w| w.as_str())
                        .unwrap_or("")
                        .to_string(),
                }
            });

        // Parse answer
        let answer = response
            .get("answer")
            .and_then(|a| a.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        // Parse evidence_refs
        let evidence_refs = response
            .get("evidence_refs")
            .and_then(|e| e.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

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

        // Parse user_question
        let user_question = response
            .get("user_question")
            .and_then(|q| q.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        Ok(BrainStep {
            step_type,
            tool_request,
            answer,
            evidence_refs,
            reliability,
            reasoning,
            user_question,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_decide_tool_step() {
        let json = serde_json::json!({
            "step_type": "decide_tool",
            "tool_request": {
                "tool": "run_shell",
                "arguments": {"command": "pacman -Qs steam"},
                "why": "Check if Steam is installed"
            },
            "answer": null,
            "evidence_refs": [],
            "reliability": 0.0,
            "reasoning": "Need to check package",
            "user_question": null
        });

        let config = LlmConfig::default();
        let orchestrator = BrainOrchestrator {
            llm_client: HttpLlmClient::new(config).unwrap(),
            tool_catalog: ToolCatalog::new(),
        };

        let step = orchestrator.parse_step(&json).unwrap();
        assert_eq!(step.step_type, StepType::DecideTool);
        assert!(step.tool_request.is_some());
        assert_eq!(step.tool_request.unwrap().tool, "run_shell");
    }

    #[test]
    fn test_parse_final_answer_step() {
        let json = serde_json::json!({
            "step_type": "final_answer",
            "tool_request": null,
            "answer": "Yes, Steam is installed [E1]. Version 1.0.0.85-1.",
            "evidence_refs": ["E1"],
            "reliability": 0.95,
            "reasoning": "pacman query confirmed installation",
            "user_question": null
        });

        let config = LlmConfig::default();
        let orchestrator = BrainOrchestrator {
            llm_client: HttpLlmClient::new(config).unwrap(),
            tool_catalog: ToolCatalog::new(),
        };

        let step = orchestrator.parse_step(&json).unwrap();
        assert_eq!(step.step_type, StepType::FinalAnswer);
        assert!(step.answer.is_some());
        assert!(step.answer.unwrap().contains("[E1]"));
        assert!(step.reliability > 0.9);
    }

    #[test]
    fn test_parse_ask_user_step() {
        let json = serde_json::json!({
            "step_type": "ask_user",
            "tool_request": null,
            "answer": null,
            "evidence_refs": [],
            "reliability": 0.1,
            "reasoning": "Need user preference",
            "user_question": "Which browser do you prefer to use?"
        });

        let config = LlmConfig::default();
        let orchestrator = BrainOrchestrator {
            llm_client: HttpLlmClient::new(config).unwrap(),
            tool_catalog: ToolCatalog::new(),
        };

        let step = orchestrator.parse_step(&json).unwrap();
        assert_eq!(step.step_type, StepType::AskUser);
        assert!(step.user_question.is_some());
    }
}
