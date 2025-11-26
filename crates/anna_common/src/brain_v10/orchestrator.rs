//! Anna Brain v10.0.2 - Orchestrator
//!
//! The main loop: INPUT → LLM → TOOL REQUESTS → TOOL OUTPUT → LLM → ANSWER
//! Max 8 iterations. Evidence-based answers. Explicit reliability labels.
//!
//! v10.0.2: Focus on proper LLM dialog - let the LLM work through iterations

use crate::brain_v10::contracts::{
    BrainSession, BrainStep, ReliabilityLabel, StepType, ToolRequest,
};
use crate::brain_v10::fallback::try_fallback_answer;
use crate::brain_v10::prompt::{build_state_message, suggest_command_for_query, SYSTEM_PROMPT, OUTPUT_SCHEMA};
use crate::brain_v10::tools::{execute_tool, ToolCatalog};
use crate::llm_client::{HttpLlmClient, LlmClient, LlmConfig};
use crate::telemetry::SystemTelemetry;
use anyhow::Result;

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

        // v10.0.1: Pre-run fallback command if we have one for this query type
        // This ensures we have evidence even if LLM fails to request the right tool
        if let Some(fallback_cmd) = suggest_command_for_query(query) {
            let output = execute_tool(
                "run_shell",
                &std::collections::HashMap::from([("command".to_string(), fallback_cmd.to_string())]),
                Some(&session.telemetry),
            );
            session.add_evidence(
                "run_shell",
                &format!("Pre-fetched: {}", fallback_cmd),
                if output.stdout.is_empty() && !output.stderr.is_empty() {
                    &output.stderr
                } else {
                    &output.stdout
                },
                output.exit_code,
            );
        }

        // Main loop
        loop {
            session.next_iteration();

            if session.iteration > MAX_ITERATIONS {
                // Before giving up, try fallback answer from evidence
                if let Some(fallback) = try_fallback_answer(query, &session) {
                    return Ok(fallback);
                }
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

            // Call LLM - if it fails, try fallback
            let response = match self.llm_client.call_json(SYSTEM_PROMPT, &state_msg, OUTPUT_SCHEMA) {
                Ok(r) => r,
                Err(_) => {
                    // LLM failed - try fallback answer
                    if let Some(fallback) = try_fallback_answer(query, &session) {
                        return Ok(fallback);
                    }
                    continue; // Try again
                }
            };

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
                    // v10.0.1: If LLM asks user on iteration 1-2, it's confused
                    // Try fallback instead
                    if session.iteration <= 2 {
                        if let Some(fallback) = try_fallback_answer(query, &session) {
                            return Ok(fallback);
                        }
                    }
                    let question = step
                        .user_question
                        .unwrap_or_else(|| "Please provide more information.".to_string());
                    return Ok(BrainResult::NeedsUserInput { question });
                }

                StepType::DecideTool => {
                    // v10.0.2: Let LLM iterate through evidence naturally
                    // Only fallback if truly stuck (iteration >= 5 AND we have evidence)
                    if session.iteration >= 5 && session.evidence.len() >= 2 {
                        if let Some(fallback) = try_fallback_answer(query, &session) {
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
                        // LLM said decide_tool but no request - use fallback
                        if let Some(fallback) = try_fallback_answer(query, &session) {
                            return Ok(fallback);
                        }
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

    fn create_orchestrator() -> BrainOrchestrator {
        BrainOrchestrator {
            llm_client: HttpLlmClient::new(LlmConfig::default()).unwrap(),
            tool_catalog: ToolCatalog::new(),
        }
    }

    #[test]
    fn test_parse_decide_tool_step() {
        let json = serde_json::json!({
            "step_type": "decide_tool",
            "tool_request": {"tool": "run_shell", "arguments": {"command": "pacman -Qs steam"}, "why": "Check Steam"},
            "reliability": 0.0, "reasoning": "Need to check"
        });
        let step = create_orchestrator().parse_step(&json).unwrap();
        assert_eq!(step.step_type, StepType::DecideTool);
        assert!(step.tool_request.is_some());
    }

    #[test]
    fn test_parse_final_answer_step() {
        let json = serde_json::json!({
            "step_type": "final_answer", "answer": "Yes, Steam is installed [E1].",
            "evidence_refs": ["E1"], "reliability": 0.95, "reasoning": "confirmed"
        });
        let step = create_orchestrator().parse_step(&json).unwrap();
        assert_eq!(step.step_type, StepType::FinalAnswer);
        assert!(step.answer.is_some());
    }
}
