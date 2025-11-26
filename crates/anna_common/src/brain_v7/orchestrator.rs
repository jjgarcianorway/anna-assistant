//! Anna v7 Brain - Orchestrator
//!
//! The core execution engine with strict three-phase pipeline:
//! 1. PLAN (LLM) - Create execution plan
//! 2. EXECUTE (Rust) - Run tools, collect evidence
//! 3. INTERPRET (LLM) - Generate answer with reliability score
//!
//! Plus optional RETRY if reliability is too low.

use super::contracts::*;
use super::prompts::{
    interpreter_json_schema, interpreter_system_prompt, planner_json_schema,
    planner_system_prompt,
};
use super::tools::ToolCatalog;
use crate::llm_client::{HttpLlmClient, LlmClient, LlmConfig};
use chrono::Utc;

/// Minimum reliability score to accept without retry
const MIN_RELIABILITY: f32 = 0.8;

/// Maximum number of retry attempts
const MAX_RETRIES: usize = 1;

/// The v7 brain orchestrator
pub struct BrainOrchestrator {
    llm: HttpLlmClient,
    tools: ToolCatalog,
}

impl BrainOrchestrator {
    /// Create a new orchestrator with the given LLM config
    pub fn new(config: LlmConfig) -> Result<Self, anyhow::Error> {
        let llm = HttpLlmClient::new(config)?;
        let tools = ToolCatalog::new();
        Ok(Self { llm, tools })
    }

    /// Process a query through the full pipeline
    pub fn process(&self, query: &str) -> BrainResult {
        // Check for meta queries first
        if let Some(result) = self.handle_meta_query(query) {
            return result;
        }

        let mut retries = 0;
        let mut last_plan: Option<PlannerOutput> = None;
        let mut last_evidence: Option<EvidenceBundle> = None;

        loop {
            // Phase 1: PLAN
            let plan = match self.plan(query, last_plan.as_ref()) {
                Ok(p) => p,
                Err(e) => return BrainResult::error(format!("Planning failed: {}", e)),
            };

            // Validate plan has tools
            if plan.tool_calls.is_empty() {
                let msg = if !plan.limitations.unanswerable_parts.is_empty() {
                    format!(
                        "Cannot answer this query: {}",
                        plan.limitations.unanswerable_parts
                    )
                } else {
                    "No tools available to answer this query.".to_string()
                };
                return BrainResult::error(msg);
            }

            // Validate all tools exist
            let invalid: Vec<_> = plan
                .tool_calls
                .iter()
                .filter(|tc| !self.tools.has_tool(&tc.tool))
                .map(|tc| tc.tool.clone())
                .collect();

            if !invalid.is_empty() {
                return BrainResult::error(format!(
                    "Plan references unknown tools: {}. This is a planner error.",
                    invalid.join(", ")
                ));
            }

            // Phase 2: EXECUTE
            let evidence = self.execute(&plan);

            // Phase 3: INTERPRET
            let interpretation = match self.interpret(query, &plan, &evidence) {
                Ok(i) => i,
                Err(e) => {
                    // Fallback to raw evidence
                    return self.fallback_answer(query, &evidence, retries, &e.to_string());
                }
            };

            // Check reliability
            if interpretation.reliability.score >= MIN_RELIABILITY {
                return BrainResult::success(interpretation.answer, &interpretation.reliability);
            }

            // Low reliability - should we retry?
            retries += 1;
            if retries > MAX_RETRIES {
                return BrainResult::low_reliability(
                    interpretation.answer,
                    &interpretation.reliability,
                    retries,
                );
            }

            // Store context for retry
            last_plan = Some(plan);
            last_evidence = Some(evidence);
        }
    }

    /// Phase 1: Call the planner LLM
    fn plan(
        &self,
        query: &str,
        previous_plan: Option<&PlannerOutput>,
    ) -> Result<PlannerOutput, anyhow::Error> {
        let system = planner_system_prompt(&self.tools.get_descriptors());

        let user = if let Some(prev) = previous_plan {
            format!(
                "RETRY: Previous plan had low reliability.\n\
                 Previous intent: {}\n\
                 Previous tools: {}\n\n\
                 User query: \"{}\"",
                prev.intent,
                prev.tool_calls
                    .iter()
                    .map(|t| t.tool.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
                query
            )
        } else {
            format!("User query: \"{}\"", query)
        };

        let response = self.llm.call_json(&system, &user, planner_json_schema())?;

        // Parse the response into PlannerOutput
        parse_planner_output(&response)
    }

    /// Phase 2: Execute all planned tools
    fn execute(&self, plan: &PlannerOutput) -> EvidenceBundle {
        let mut runs = Vec::new();
        let mut all_succeeded = true;

        for tc in &plan.tool_calls {
            let run = self.tools.execute(&tc.tool, &tc.subtask_id, &tc.parameters);
            if !run.success() {
                all_succeeded = false;
            }
            runs.push(run);
        }

        EvidenceBundle {
            runs,
            all_succeeded,
            collected_at: Utc::now(),
        }
    }

    /// Phase 3: Call the interpreter LLM
    fn interpret(
        &self,
        query: &str,
        plan: &PlannerOutput,
        evidence: &EvidenceBundle,
    ) -> Result<InterpreterOutput, anyhow::Error> {
        let system = interpreter_system_prompt();

        // Build evidence text
        let evidence_text = evidence
            .runs
            .iter()
            .map(|r| {
                format!(
                    "=== Tool: {} (subtask: {}) ===\n\
                     Command: {}\n\
                     Exit code: {}\n\
                     Output:\n{}\n\
                     Stderr: {}",
                    r.tool,
                    r.subtask_id,
                    r.command_preview,
                    r.exit_code,
                    if r.stdout.is_empty() {
                        "(empty)"
                    } else {
                        &r.stdout
                    },
                    if r.stderr.is_empty() {
                        "(none)"
                    } else {
                        &r.stderr
                    }
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        let user = format!(
            "User query: \"{}\"\n\n\
             Plan intent: {}\n\n\
             Evidence bundle:\n{}",
            query, plan.intent, evidence_text
        );

        let response = self.llm.call_json(&system, &user, interpreter_json_schema())?;

        // Parse the response into InterpreterOutput
        parse_interpreter_output(&response)
    }

    /// Handle meta queries about Anna itself
    fn handle_meta_query(&self, query: &str) -> Option<BrainResult> {
        let q = query.to_lowercase();

        if q.contains("anna version") || q.contains("your version") {
            return Some(BrainResult::success(
                "Anna Assistant v7.0.0".to_string(),
                &Reliability::high("Version is hardcoded"),
            ));
        }

        if q.contains("who are you") || q.contains("about anna") {
            return Some(BrainResult::success(
                "I am Anna, a local Arch Linux system assistant. \
                 I answer queries by running real system tools and reporting only what I find. \
                 I never guess or invent information."
                    .to_string(),
                &Reliability::high("Identity is hardcoded"),
            ));
        }

        if q.contains("upgrade") && (q.contains("brain") || q.contains("llm") || q.contains("model"))
        {
            return Some(BrainResult::success(
                "To upgrade Anna's LLM backend:\n\
                 1. List available models: ollama list\n\
                 2. Pull a new model: ollama pull <model-name>\n\
                 3. Edit ~/.config/anna/config.toml and set model = \"<model-name>\"\n\
                 4. Restart Anna: systemctl --user restart annad\n\n\
                 Recommended models:\n\
                 - qwen2.5:14b (best reasoning, needs 16GB+ RAM)\n\
                 - llama3.2:3b (fast, good for simple queries)\n\
                 - mistral:7b (balanced)"
                    .to_string(),
                &Reliability::high("Upgrade instructions are hardcoded"),
            ));
        }

        None
    }

    /// Fallback when interpretation fails
    fn fallback_answer(
        &self,
        query: &str,
        evidence: &EvidenceBundle,
        retries: usize,
        error: &str,
    ) -> BrainResult {
        // Try to extract something useful from evidence
        let q = query.to_lowercase();

        for run in &evidence.runs {
            if !run.success() || run.stdout.is_empty() {
                continue;
            }

            // RAM queries
            if q.contains("ram") || q.contains("memory") {
                for line in run.stdout.lines() {
                    if line.starts_with("MemTotal:") {
                        if let Some(kb) = extract_kb_value(line) {
                            let gb = kb as f64 / 1024.0 / 1024.0;
                            return BrainResult {
                                answer: format!("Total RAM: {:.1} GB (from /proc/meminfo)", gb),
                                reliability_score: 0.7,
                                reliability_level: "MEDIUM".to_string(),
                                retries_used: retries,
                                success: true,
                                error: None,
                            };
                        }
                    }
                }
            }

            // CPU queries
            if q.contains("cpu") || q.contains("processor") {
                for line in run.stdout.lines() {
                    if line.contains("Model name:") {
                        if let Some(name) = line.split(':').nth(1) {
                            return BrainResult {
                                answer: format!("CPU: {}", name.trim()),
                                reliability_score: 0.7,
                                reliability_level: "MEDIUM".to_string(),
                                retries_used: retries,
                                success: true,
                                error: None,
                            };
                        }
                    }
                }
            }

            // GPU queries
            if q.contains("gpu") || q.contains("graphics") {
                let output = run.stdout.trim();
                if !output.is_empty() {
                    return BrainResult {
                        answer: format!("GPU: {}", output.lines().next().unwrap_or(output)),
                        reliability_score: 0.7,
                        reliability_level: "MEDIUM".to_string(),
                        retries_used: retries,
                        success: true,
                        error: None,
                    };
                }
            }
        }

        BrainResult::error(format!(
            "Could not interpret evidence. LLM error: {}",
            error
        ))
    }
}

/// Parse planner JSON response into PlannerOutput
fn parse_planner_output(json: &serde_json::Value) -> Result<PlannerOutput, anyhow::Error> {
    let intent = json
        .get("intent")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown intent")
        .to_string();

    let subtasks = json
        .get("subtasks")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    Some(Subtask {
                        id: item.get("id")?.as_str()?.to_string(),
                        description: item.get("description")?.as_str()?.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let tool_calls = json
        .get("tool_calls")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    Some(ToolCall {
                        subtask_id: item.get("subtask_id")?.as_str()?.to_string(),
                        tool: item.get("tool")?.as_str()?.to_string(),
                        parameters: item
                            .get("parameters")
                            .cloned()
                            .unwrap_or(serde_json::json!({})),
                        reason: item.get("reason")?.as_str()?.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let expected_evidence = json
        .get("expected_evidence")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    Some(ExpectedEvidence {
                        subtask_id: item.get("subtask_id")?.as_str()?.to_string(),
                        tool: item.get("tool")?.as_str()?.to_string(),
                        evidence_needed: item.get("evidence_needed")?.as_str()?.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let limitations = json.get("limitations").map_or_else(
        || Limitations {
            missing_tools: vec![],
            unanswerable_parts: String::new(),
        },
        |lim| Limitations {
            missing_tools: lim
                .get("missing_tools")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            unanswerable_parts: lim
                .get("unanswerable_parts")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        },
    );

    Ok(PlannerOutput {
        intent,
        subtasks,
        tool_calls,
        expected_evidence,
        limitations,
    })
}

/// Parse interpreter JSON response into InterpreterOutput
fn parse_interpreter_output(json: &serde_json::Value) -> Result<InterpreterOutput, anyhow::Error> {
    let answer = json
        .get("answer")
        .and_then(|v| v.as_str())
        .unwrap_or("Unable to determine answer")
        .to_string();

    let evidence_used = json
        .get("evidence_used")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    Some(EvidenceUsed {
                        tool: item.get("tool")?.as_str()?.to_string(),
                        summary: item.get("summary")?.as_str()?.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let reliability = json.get("reliability").map_or_else(
        || Reliability::medium(0.5, "Could not parse reliability"),
        |rel| Reliability {
            score: rel
                .get("score")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.5) as f32,
            level: rel
                .get("level")
                .and_then(|v| v.as_str())
                .unwrap_or("MEDIUM")
                .to_string(),
            reason: rel
                .get("reason")
                .and_then(|v| v.as_str())
                .unwrap_or("No reason provided")
                .to_string(),
        },
    );

    let uncertainty = json.get("uncertainty").map_or_else(
        || Uncertainty {
            has_unknowns: true,
            details: "Could not parse uncertainty".to_string(),
        },
        |unc| Uncertainty {
            has_unknowns: unc
                .get("has_unknowns")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            details: unc
                .get("details")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        },
    );

    Ok(InterpreterOutput {
        answer,
        evidence_used,
        reliability,
        uncertainty,
    })
}

/// Extract KB value from a /proc/meminfo line
fn extract_kb_value(line: &str) -> Option<u64> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 {
        parts[1].parse().ok()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_planner_output() {
        let json = serde_json::json!({
            "intent": "Check RAM",
            "subtasks": [{"id": "st1", "description": "Get memory"}],
            "tool_calls": [{
                "subtask_id": "st1",
                "tool": "mem_info",
                "parameters": {},
                "reason": "Need memory data"
            }],
            "expected_evidence": [{
                "subtask_id": "st1",
                "tool": "mem_info",
                "evidence_needed": "MemTotal"
            }],
            "limitations": {
                "missing_tools": [],
                "unanswerable_parts": ""
            }
        });

        let output = parse_planner_output(&json).unwrap();
        assert_eq!(output.intent, "Check RAM");
        assert_eq!(output.tool_calls.len(), 1);
        assert_eq!(output.tool_calls[0].tool, "mem_info");
    }

    #[test]
    fn test_parse_interpreter_output() {
        let json = serde_json::json!({
            "answer": "You have 32 GB RAM",
            "evidence_used": [{"tool": "mem_info", "summary": "MemTotal: 32GB"}],
            "reliability": {"score": 0.95, "level": "HIGH", "reason": "Direct evidence"},
            "uncertainty": {"has_unknowns": false, "details": ""}
        });

        let output = parse_interpreter_output(&json).unwrap();
        assert_eq!(output.answer, "You have 32 GB RAM");
        assert_eq!(output.reliability.level, "HIGH");
    }

    #[test]
    fn test_extract_kb_value() {
        assert_eq!(extract_kb_value("MemTotal:       32791612 kB"), Some(32791612));
        assert_eq!(extract_kb_value("Invalid line"), None);
    }
}
