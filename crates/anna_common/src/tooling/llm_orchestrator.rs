//! LLM Orchestrator - v6.62.0
//!
//! Hybrid architecture: Rust orchestrator controls the loop, LLM follows strict contract.
//!
//! Pipeline (with retry):
//! 1. Planner LLM: Query → Structured plan (subtasks, tool_calls, expected_evidence)
//! 2. Rust: Execute tools exactly as planned → EvidenceBundle
//! 3. Interpreter LLM: Query + Plan + Evidence → Structured answer with reliability score
//! 4. Rust: If reliability < 0.8 and retries remain → Loop back to planner with context
//! 5. Return final answer with explicit uncertainty when score stays low
//!
//! CONTRACT (enforced in both LLM prompts):
//! - Never hardcode system state
//! - Never invent tools not in catalog
//! - Never claim to have run commands that weren't executed
//! - If evidence is insufficient, say so explicitly

use super::catalog::{tool_catalog, ToolSpec, run_tool};
use crate::llm_client::{LlmClient, LlmConfig, HttpLlmClient, LlmError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Maximum retry passes for low-reliability answers
const MAX_RETRIES: usize = 2;

/// Minimum reliability score to accept without retry
const MIN_RELIABILITY: f32 = 0.8;

// ============================================================================
// Planner Output Types
// ============================================================================

/// Structured plan from the LLM planner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannerOutput {
    /// Subtasks identified from the query
    pub subtasks: Vec<Subtask>,
    /// Tools to call and why
    pub tool_calls: Vec<ToolCall>,
    /// What this plan cannot answer (if any)
    pub limitations: Option<String>,
}

/// A subtask identified by the planner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subtask {
    pub description: String,
    pub requires_tool: bool,
}

/// A tool call planned by the LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool_id: String,
    pub purpose: String,
    pub expected_evidence: String,
}

// ============================================================================
// Evidence Bundle Types
// ============================================================================

/// Complete evidence bundle from tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceBundle {
    pub tool_results: Vec<ToolEvidence>,
    pub timestamp: u64,
    pub all_tools_succeeded: bool,
}

/// Evidence from a single tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolEvidence {
    pub tool_id: String,
    pub purpose: String,
    pub expected_evidence: String,
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

// ============================================================================
// Interpreter Output Types
// ============================================================================

/// Structured answer from the LLM interpreter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterpreterOutput {
    pub answer: String,
    pub evidence_used: Vec<EvidenceSummary>,
    pub reliability: ReliabilityScore,
    pub uncertainty: UncertaintyInfo,
}

/// Summary of how a tool's evidence was used
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceSummary {
    pub tool: String,
    pub summary: String,
}

/// Reliability score with explanation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliabilityScore {
    pub score: f32,
    pub level: String,
    pub reason: String,
}

/// Information about unknowns and uncertainty
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncertaintyInfo {
    pub has_unknowns: bool,
    pub details: Option<String>,
}

// ============================================================================
// Orchestration Result
// ============================================================================

/// Final result returned to the user
#[derive(Debug, Clone)]
pub struct OrchestrationResult {
    pub answer: String,
    pub reliability_score: f32,
    pub reliability_level: String,
    pub evidence_used: Vec<EvidenceSummary>,
    pub uncertainty: Option<String>,
    pub retries_used: usize,
    pub success: bool,
}

// ============================================================================
// The Orchestrator
// ============================================================================

/// The LLM-driven orchestrator with reliability-based retry
pub struct LlmOrchestrator {
    llm_client: HttpLlmClient,
    catalog: HashMap<String, ToolSpec>,
    catalog_description: String,
}

impl LlmOrchestrator {
    /// Create a new orchestrator with the given LLM config
    pub fn new(config: LlmConfig) -> Result<Self, anyhow::Error> {
        let llm_client = HttpLlmClient::new(config)?;
        let catalog: HashMap<String, ToolSpec> = tool_catalog()
            .into_iter()
            .map(|spec| (spec.id.name().to_string(), spec))
            .collect();

        // Pre-build catalog description for prompts
        let catalog_description = build_catalog_description(&catalog);

        Ok(Self {
            llm_client,
            catalog,
            catalog_description,
        })
    }

    /// Process a user query through the full orchestration loop with retries
    pub fn process_query(&self, query: &str) -> OrchestrationResult {
        // Check for meta queries first (handled without tools)
        if is_meta_query(query) {
            return self.handle_meta_query(query);
        }

        let mut retries_used = 0;
        let mut previous_plan: Option<PlannerOutput> = None;
        let mut previous_evidence: Option<EvidenceBundle> = None;
        let mut previous_interpretation: Option<InterpreterOutput> = None;

        loop {
            // Step 1: Call the planner LLM
            let plan = match self.call_planner(
                query,
                previous_plan.as_ref(),
                previous_evidence.as_ref(),
                previous_interpretation.as_ref(),
            ) {
                Ok(p) => p,
                Err(e) => {
                    return OrchestrationResult {
                        answer: format!(
                            "I was unable to plan how to answer this query.\n\
                             Reason: {}\n\
                             Please try rephrasing your question.",
                            e
                        ),
                        reliability_score: 0.0,
                        reliability_level: "LOW".to_string(),
                        evidence_used: vec![],
                        uncertainty: Some("Planning failed".to_string()),
                        retries_used,
                        success: false,
                    };
                }
            };

            // Validate all tools exist before execution
            let invalid_tools: Vec<_> = plan
                .tool_calls
                .iter()
                .filter(|tc| !self.catalog.contains_key(&tc.tool_id))
                .map(|tc| tc.tool_id.clone())
                .collect();

            if !invalid_tools.is_empty() {
                return OrchestrationResult {
                    answer: format!(
                        "I cannot answer this query because the following tools \
                         are not available: {}.\n\
                         This may require additional system tools to be installed.",
                        invalid_tools.join(", ")
                    ),
                    reliability_score: 0.0,
                    reliability_level: "LOW".to_string(),
                    evidence_used: vec![],
                    uncertainty: Some(format!("Missing tools: {}", invalid_tools.join(", "))),
                    retries_used,
                    success: false,
                };
            }

            // Check if plan has no tools
            if plan.tool_calls.is_empty() {
                return OrchestrationResult {
                    answer: format!(
                        "I cannot answer this query with the available tools.\n\
                         {}",
                        plan.limitations.as_deref().unwrap_or("No tools applicable.")
                    ),
                    reliability_score: 0.2,
                    reliability_level: "LOW".to_string(),
                    evidence_used: vec![],
                    uncertainty: Some("No tools selected".to_string()),
                    retries_used,
                    success: false,
                };
            }

            // Step 2: Execute tools and build evidence bundle
            let evidence = self.execute_tools(&plan);

            // Step 3: Call the interpreter LLM
            let interpretation = match self.call_interpreter(query, &plan, &evidence) {
                Ok(i) => i,
                Err(e) => {
                    // Fallback to raw evidence if interpretation fails
                    return self.fallback_response(query, &evidence, retries_used, e);
                }
            };

            // Step 4: Check reliability and decide whether to retry
            if interpretation.reliability.score >= MIN_RELIABILITY {
                // Good enough - return this answer
                return OrchestrationResult {
                    answer: interpretation.answer,
                    reliability_score: interpretation.reliability.score,
                    reliability_level: interpretation.reliability.level,
                    evidence_used: interpretation.evidence_used,
                    uncertainty: if interpretation.uncertainty.has_unknowns {
                        interpretation.uncertainty.details
                    } else {
                        None
                    },
                    retries_used,
                    success: true,
                };
            }

            // Low reliability - should we retry?
            retries_used += 1;
            if retries_used >= MAX_RETRIES {
                // Max retries reached - return best effort with explicit uncertainty
                return OrchestrationResult {
                    answer: format!(
                        "{}\n\n⚠️  Note: This answer has low reliability ({:.0}%) due to: {}",
                        interpretation.answer,
                        interpretation.reliability.score * 100.0,
                        interpretation.reliability.reason
                    ),
                    reliability_score: interpretation.reliability.score,
                    reliability_level: interpretation.reliability.level,
                    evidence_used: interpretation.evidence_used,
                    uncertainty: interpretation.uncertainty.details,
                    retries_used,
                    success: false,
                };
            }

            // Store context for retry
            previous_plan = Some(plan);
            previous_evidence = Some(evidence);
            previous_interpretation = Some(interpretation);
        }
    }

    /// Call the planner LLM to create an execution plan
    fn call_planner(
        &self,
        query: &str,
        previous_plan: Option<&PlannerOutput>,
        previous_evidence: Option<&EvidenceBundle>,
        previous_interpretation: Option<&InterpreterOutput>,
    ) -> Result<PlannerOutput, LlmError> {
        let system_prompt = self.build_planner_system_prompt();

        let user_prompt = if let Some(prev_interp) = previous_interpretation {
            // Retry scenario - include previous context
            format!(
                "RETRY REQUEST\n\n\
                 Original query: \"{}\"\n\n\
                 Previous plan tools: {}\n\n\
                 Previous reliability score: {:.2} ({})\n\
                 Previous reliability reason: {}\n\n\
                 What was missing or unclear: {}\n\n\
                 Create a REFINED plan that addresses the gaps. \
                 Consider different tools or more targeted approaches.",
                query,
                previous_plan
                    .map(|p| p.tool_calls.iter().map(|t| t.tool_id.as_str()).collect::<Vec<_>>().join(", "))
                    .unwrap_or_default(),
                prev_interp.reliability.score,
                prev_interp.reliability.level,
                prev_interp.reliability.reason,
                prev_interp.uncertainty.details.as_deref().unwrap_or("Unknown")
            )
        } else {
            // First attempt
            format!(
                "User query: \"{}\"\n\n\
                 Create a plan to answer this query using ONLY tools from the catalog.",
                query
            )
        };

        let schema = r#"{
            "type": "object",
            "properties": {
                "subtasks": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "description": {"type": "string"},
                            "requires_tool": {"type": "boolean"}
                        },
                        "required": ["description", "requires_tool"]
                    }
                },
                "tool_calls": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "tool_id": {"type": "string"},
                            "purpose": {"type": "string"},
                            "expected_evidence": {"type": "string"}
                        },
                        "required": ["tool_id", "purpose", "expected_evidence"]
                    }
                },
                "limitations": {"type": "string"}
            },
            "required": ["subtasks", "tool_calls"]
        }"#;

        let response = self.llm_client.call_json(&system_prompt, &user_prompt, schema)?;

        // Parse response into PlannerOutput
        let subtasks = response
            .get("subtasks")
            .and_then(|s| s.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        Some(Subtask {
                            description: item.get("description")?.as_str()?.to_string(),
                            requires_tool: item.get("requires_tool")?.as_bool()?,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let tool_calls = response
            .get("tool_calls")
            .and_then(|t| t.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        Some(ToolCall {
                            tool_id: item.get("tool_id")?.as_str()?.to_string(),
                            purpose: item.get("purpose")?.as_str()?.to_string(),
                            expected_evidence: item
                                .get("expected_evidence")?
                                .as_str()?
                                .to_string(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let limitations = response
            .get("limitations")
            .and_then(|l| l.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        Ok(PlannerOutput {
            subtasks,
            tool_calls,
            limitations,
        })
    }

    /// Execute all planned tools and collect evidence
    fn execute_tools(&self, plan: &PlannerOutput) -> EvidenceBundle {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let mut tool_results = Vec::new();
        let mut all_succeeded = true;

        for tool_call in &plan.tool_calls {
            if let Some(spec) = self.catalog.get(&tool_call.tool_id) {
                let result = run_tool(spec);

                if !result.success {
                    all_succeeded = false;
                }

                tool_results.push(ToolEvidence {
                    tool_id: tool_call.tool_id.clone(),
                    purpose: tool_call.purpose.clone(),
                    expected_evidence: tool_call.expected_evidence.clone(),
                    success: result.success,
                    stdout: result.stdout,
                    stderr: result.stderr,
                    exit_code: result.exit_code,
                });
            } else {
                all_succeeded = false;
                tool_results.push(ToolEvidence {
                    tool_id: tool_call.tool_id.clone(),
                    purpose: tool_call.purpose.clone(),
                    expected_evidence: tool_call.expected_evidence.clone(),
                    success: false,
                    stdout: String::new(),
                    stderr: format!("Tool '{}' not found in catalog", tool_call.tool_id),
                    exit_code: -1,
                });
            }
        }

        EvidenceBundle {
            tool_results,
            timestamp,
            all_tools_succeeded: all_succeeded,
        }
    }

    /// Call the interpreter LLM to produce a final answer
    fn call_interpreter(
        &self,
        query: &str,
        plan: &PlannerOutput,
        evidence: &EvidenceBundle,
    ) -> Result<InterpreterOutput, LlmError> {
        let system_prompt = self.build_interpreter_system_prompt();

        // Build evidence text for the prompt
        let evidence_text = evidence
            .tool_results
            .iter()
            .map(|e| {
                format!(
                    "=== Tool: {} ===\n\
                     Purpose: {}\n\
                     Expected: {}\n\
                     Status: {}\n\
                     Output:\n{}\n",
                    e.tool_id,
                    e.purpose,
                    e.expected_evidence,
                    if e.success { "SUCCESS" } else { "FAILED" },
                    if e.success {
                        truncate_output(&e.stdout, 1500)
                    } else {
                        format!("Error: {}", e.stderr)
                    }
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let user_prompt = format!(
            "Original query: \"{}\"\n\n\
             Evidence bundle:\n{}\n\n\
             Based ONLY on the evidence above, answer the user's query.",
            query, evidence_text
        );

        let schema = r#"{
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
                    "required": ["has_unknowns"]
                }
            },
            "required": ["answer", "evidence_used", "reliability", "uncertainty"]
        }"#;

        let response = self.llm_client.call_json(&system_prompt, &user_prompt, schema)?;

        // Parse the response
        let answer = response
            .get("answer")
            .and_then(|a| a.as_str())
            .filter(|s| !s.is_empty() && s.len() > 3 && *s != "answer" && *s != "object")
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.extract_simple_answer(query, evidence));

        let evidence_used = response
            .get("evidence_used")
            .and_then(|e| e.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        Some(EvidenceSummary {
                            tool: item.get("tool")?.as_str()?.to_string(),
                            summary: item.get("summary")?.as_str()?.to_string(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let reliability = response
            .get("reliability")
            .map(|r| ReliabilityScore {
                score: r.get("score").and_then(|s| s.as_f64()).unwrap_or(0.5) as f32,
                level: r
                    .get("level")
                    .and_then(|l| l.as_str())
                    .unwrap_or("MEDIUM")
                    .to_string(),
                reason: r
                    .get("reason")
                    .and_then(|r| r.as_str())
                    .unwrap_or("No reason provided")
                    .to_string(),
            })
            .unwrap_or(ReliabilityScore {
                score: 0.5,
                level: "MEDIUM".to_string(),
                reason: "Could not parse reliability from LLM".to_string(),
            });

        let uncertainty = response
            .get("uncertainty")
            .map(|u| UncertaintyInfo {
                has_unknowns: u.get("has_unknowns").and_then(|h| h.as_bool()).unwrap_or(true),
                details: u.get("details").and_then(|d| d.as_str()).map(|s| s.to_string()),
            })
            .unwrap_or(UncertaintyInfo {
                has_unknowns: true,
                details: Some("Could not parse uncertainty".to_string()),
            });

        Ok(InterpreterOutput {
            answer,
            evidence_used,
            reliability,
            uncertainty,
        })
    }

    /// Build the planner system prompt
    fn build_planner_system_prompt(&self) -> String {
        format!(
            r#"You are Anna's planning brain. Your job is to create execution plans for system queries.

TOOL CATALOG (use ONLY these tool_id values):
{}

CRITICAL CONTRACT:
1. NEVER hardcode system facts (CPU model, RAM size, GPU, packages, etc.)
2. NEVER invent tools not in the catalog above
3. NEVER assume information from previous runs
4. If a query cannot be answered with available tools, set limitations explaining why

OUTPUT FORMAT (strict JSON):
{{
  "subtasks": [
    {{"description": "what needs to be determined", "requires_tool": true/false}}
  ],
  "tool_calls": [
    {{
      "tool_id": "exact tool name from catalog",
      "purpose": "why this tool is needed",
      "expected_evidence": "what data this tool should provide"
    }}
  ],
  "limitations": "what cannot be answered, if any (optional)"
}}

RULES:
- Use MINIMAL tools needed to answer the query
- Be specific about what evidence each tool should provide
- If retrying, focus on gaps identified in previous attempt"#,
            self.catalog_description
        )
    }

    /// Build the interpreter system prompt
    fn build_interpreter_system_prompt(&self) -> String {
        r#"You are Anna's interpretation brain. Your job is to answer user queries based ONLY on provided evidence.

CRITICAL CONTRACT:
1. Use ONLY information from the evidence_bundle provided
2. If something is not in the evidence, treat it as UNKNOWN
3. NEVER invent device names, package names, or system properties
4. NEVER make recommendations not supported by evidence
5. If a tool failed, explain what could not be determined
6. Always answer in English
7. No questions at the end of the answer

RELIABILITY SCORING:
- 0.9-1.0 (HIGH): Strong evidence from tools, all consistent
- 0.7-0.89 (MEDIUM): Reasonable evidence, minor gaps
- 0.4-0.69 (LOW): Weak evidence, several assumptions
- 0.0-0.39 (VERY LOW): Almost no evidence

OUTPUT FORMAT (strict JSON):
{
  "answer": "Direct answer in English, no trailing questions",
  "evidence_used": [
    {"tool": "tool_id", "summary": "what this tool output meant"}
  ],
  "reliability": {
    "score": 0.0-1.0,
    "level": "HIGH|MEDIUM|LOW",
    "reason": "why this score"
  },
  "uncertainty": {
    "has_unknowns": true/false,
    "details": "what is unknown or guessed (if any)"
  }
}

Be concise. Extract only facts present in the evidence."#
            .to_string()
    }

    /// Handle meta queries about Anna itself
    fn handle_meta_query(&self, query: &str) -> OrchestrationResult {
        let query_lower = query.to_lowercase();

        let answer = if query_lower.contains("upgrade")
            || query_lower.contains("brain")
            || query_lower.contains("model")
            || query_lower.contains("llm")
        {
            "To upgrade Anna's LLM backend:\n\
             1. List available models: ollama list\n\
             2. Pull a new model: ollama pull <model-name>\n\
             3. Edit ~/.config/anna/config.toml and set model = \"<model-name>\"\n\
             4. Restart Anna: systemctl --user restart annad\n\n\
             Recommended models:\n\
             - llama3.2:3b (fast, good for simple queries)\n\
             - llama3.1:8b (balanced)\n\
             - mistral:7b (good reasoning)"
                .to_string()
        } else if query_lower.contains("version") {
            "Anna Assistant v6.62.0".to_string()
        } else if query_lower.contains("who are you") || query_lower.contains("about anna") {
            "I am Anna, a local Arch Linux system assistant.\n\
             I answer queries by running real system tools and reporting only what I find.\n\
             I never guess or invent information - if I don't know, I'll say so."
                .to_string()
        } else {
            "I didn't understand that meta query. \
             Try asking about my version, how to upgrade my LLM, or who I am."
                .to_string()
        };

        OrchestrationResult {
            answer,
            reliability_score: 1.0,
            reliability_level: "HIGH".to_string(),
            evidence_used: vec![],
            uncertainty: None,
            retries_used: 0,
            success: true,
        }
    }

    /// Fallback response when interpreter fails
    fn fallback_response(
        &self,
        query: &str,
        evidence: &EvidenceBundle,
        retries: usize,
        error: LlmError,
    ) -> OrchestrationResult {
        // Try to extract something useful from raw evidence
        let answer = self.extract_simple_answer(query, evidence);

        OrchestrationResult {
            answer: format!(
                "{}\n\n(Note: LLM interpretation failed: {})",
                answer, error
            ),
            reliability_score: 0.4,
            reliability_level: "LOW".to_string(),
            evidence_used: evidence
                .tool_results
                .iter()
                .filter(|t| t.success)
                .map(|t| EvidenceSummary {
                    tool: t.tool_id.clone(),
                    summary: "Raw output used".to_string(),
                })
                .collect(),
            uncertainty: Some("LLM interpretation failed, showing raw evidence".to_string()),
            retries_used: retries,
            success: false,
        }
    }

    /// Simple evidence extraction when LLM fails
    fn extract_simple_answer(&self, query: &str, evidence: &EvidenceBundle) -> String {
        let query_lower = query.to_lowercase();

        // Try to extract based on query type
        for tool_result in &evidence.tool_results {
            if !tool_result.success || tool_result.stdout.trim().is_empty() {
                continue;
            }

            let output = &tool_result.stdout;

            // CPU queries
            if query_lower.contains("cpu") {
                for line in output.lines() {
                    if line.contains("Model name:") {
                        if let Some(name) = line.split(':').nth(1) {
                            return name.trim().to_string();
                        }
                    }
                }
            }

            // RAM queries
            if query_lower.contains("ram") || query_lower.contains("memory") {
                for line in output.lines() {
                    if line.starts_with("Mem:") {
                        let parts: Vec<_> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            return format!("Total RAM: {} MB", parts[1]);
                        }
                    }
                }
            }

            // GPU queries
            if query_lower.contains("gpu") {
                for line in output.lines() {
                    let line_lower = line.to_lowercase();
                    if line_lower.contains("vga") || line_lower.contains("3d controller") {
                        return line.trim().to_string();
                    }
                }
            }

            // IP queries
            if query_lower.contains("ip") {
                for line in output.lines() {
                    if line.contains("inet ") && !line.contains("127.0.0.1") {
                        let parts: Vec<_> = line.split_whitespace().collect();
                        if let Some(idx) = parts.iter().position(|&s| s == "inet") {
                            if idx + 1 < parts.len() {
                                return format!("IP: {}", parts[idx + 1]);
                            }
                        }
                    }
                }
            }
        }

        // Fallback: return first non-empty output
        for tool_result in &evidence.tool_results {
            if tool_result.success && !tool_result.stdout.trim().is_empty() {
                let truncated = truncate_output(&tool_result.stdout, 500);
                return truncated;
            }
        }

        // Handle empty results (e.g., no orphan packages)
        if evidence.tool_results.iter().all(|t| t.stdout.trim().is_empty() && t.stderr.is_empty())
        {
            if query_lower.contains("orphan") {
                return "No orphan packages found.".to_string();
            }
            if query_lower.contains("update") {
                return "No updates available.".to_string();
            }
            if query_lower.contains("failed") {
                return "No failed services.".to_string();
            }
        }

        "Unable to extract answer from evidence.".to_string()
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Check if query is a meta query about Anna itself
fn is_meta_query(query: &str) -> bool {
    let q = query.to_lowercase();
    q.contains("upgrade your brain")
        || q.contains("upgrade brain")
        || q.contains("llm upgrade")
        || q.contains("change model")
        || q.contains("anna version")
        || q.contains("about anna")
        || q.contains("who are you")
}

/// Build catalog description for prompts
fn build_catalog_description(catalog: &HashMap<String, ToolSpec>) -> String {
    let mut lines: Vec<_> = catalog
        .iter()
        .map(|(id, spec)| {
            format!(
                "- {}: {} (runs: {} {})",
                id,
                spec.description,
                spec.binary,
                spec.args.join(" ")
            )
        })
        .collect();
    lines.sort();
    lines.join("\n")
}

/// Truncate output to a maximum length
fn truncate_output(output: &str, max_len: usize) -> String {
    if output.len() <= max_len {
        output.to_string()
    } else {
        format!("{}...[truncated]", &output[..max_len])
    }
}

/// Build tool catalog description for external use
pub fn get_tool_catalog_for_llm() -> String {
    let catalog: HashMap<String, ToolSpec> = tool_catalog()
        .into_iter()
        .map(|spec| (spec.id.name().to_string(), spec))
        .collect();
    build_catalog_description(&catalog)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_meta_query() {
        assert!(is_meta_query("upgrade your brain"));
        assert!(is_meta_query("who are you"));
        assert!(is_meta_query("anna version"));
        assert!(!is_meta_query("how much RAM"));
        assert!(!is_meta_query("what GPU"));
    }

    #[test]
    fn test_truncate_output() {
        let short = "hello";
        assert_eq!(truncate_output(short, 100), "hello");

        let long = "a".repeat(200);
        let truncated = truncate_output(&long, 50);
        assert!(truncated.contains("...[truncated]"));
        assert!(truncated.len() < 100);
    }

    #[test]
    fn test_tool_catalog_for_llm() {
        let catalog = get_tool_catalog_for_llm();
        assert!(catalog.contains("free_mem"));
        assert!(catalog.contains("lscpu"));
        assert!(catalog.contains("df_human"));
    }
}
