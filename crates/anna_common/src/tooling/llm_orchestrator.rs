//! LLM Orchestrator - v6.60.0
//!
//! Pure orchestration layer that delegates ALL decision-making to the LLM.
//! The orchestrator NEVER decides which tools to run or how to interpret results.
//!
//! Flow:
//! 1. User query + tool catalog -> LLM Planner -> Command plan
//! 2. Command plan -> Executor -> Raw output
//! 3. Raw output + original query -> LLM Interpreter -> Human answer
//!
//! The orchestrator only enforces:
//! - Tool catalog (allowed commands)
//! - Execution sandbox
//! - Result forwarding

use super::catalog::{tool_catalog, ToolSpec, ToolId, run_tool};
use crate::llm_client::{LlmClient, LlmConfig, HttpLlmClient, LlmError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Command plan returned by the LLM planner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPlan {
    /// List of commands to execute
    pub commands: Vec<PlannedCommand>,
    /// Brief explanation of the plan (for transparency)
    pub reasoning: String,
}

/// A single planned command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedCommand {
    /// Tool ID from the catalog
    pub tool_id: String,
    /// Additional arguments (if any)
    pub extra_args: Vec<String>,
    /// Why this command is needed
    pub purpose: String,
}

/// Execution result for a single command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandOutput {
    pub tool_id: String,
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

/// Final orchestration result
#[derive(Debug, Clone)]
pub struct OrchestrationResult {
    pub answer: String,
    pub plan: Option<CommandPlan>,
    pub outputs: Vec<CommandOutput>,
    pub success: bool,
}

/// The LLM-driven orchestrator
pub struct LlmOrchestrator {
    llm_client: HttpLlmClient,
    catalog: HashMap<String, ToolSpec>,
}

impl LlmOrchestrator {
    /// Create a new orchestrator with the given LLM config
    pub fn new(config: LlmConfig) -> Result<Self, anyhow::Error> {
        let llm_client = HttpLlmClient::new(config)?;
        let catalog: HashMap<String, ToolSpec> = tool_catalog()
            .into_iter()
            .map(|spec| (spec.id.name().to_string(), spec))
            .collect();

        Ok(Self { llm_client, catalog })
    }

    /// Process a user query through the full orchestration loop
    pub fn process_query(&self, query: &str) -> OrchestrationResult {
        // Step 1: Ask LLM to plan which commands to run
        let plan = match self.plan_commands(query) {
            Ok(p) => p,
            Err(e) => {
                // Planning failed - return error with helpful message
                return OrchestrationResult {
                    answer: format!(
                        "Failed to plan commands: {}\n\n\
                         This may indicate an issue with the LLM connection or model.\n\
                         Query: \"{}\"",
                        e, query
                    ),
                    plan: None,
                    outputs: vec![],
                    success: false,
                };
            }
        };

        // Check if we have any commands to run
        if plan.commands.is_empty() {
            return OrchestrationResult {
                answer: format!(
                    "LLM reasoning: {}\n\n\
                     No system commands needed to answer this query.\n\
                     Query: \"{}\"",
                    plan.reasoning, query
                ),
                plan: Some(plan),
                outputs: vec![],
                success: true,
            };
        }

        // Step 2: Execute the planned commands
        let outputs = self.execute_plan(&plan);

        // Check if any commands actually produced output
        let has_output = outputs.iter().any(|o| o.success && !o.stdout.trim().is_empty());

        // Step 3: Ask LLM to summarize the results
        let answer = if has_output {
            match self.summarize_results(query, &plan, &outputs) {
                Ok(a) if !a.is_empty() && a != "Unable to summarize results." => a,
                _ => {
                    // Fallback: return raw output if LLM summarization fails
                    self.format_raw_output(query, &outputs)
                }
            }
        } else {
            // No successful output - explain what happened
            let failed_tools: Vec<_> = outputs.iter()
                .filter(|o| !o.success)
                .map(|o| format!("{}: {}", o.tool_id, o.stderr.lines().next().unwrap_or("unknown error")))
                .collect();

            if failed_tools.is_empty() {
                format!(
                    "Commands ran but produced no output.\n\n\
                     LLM reasoning: {}\n\
                     Query: \"{}\"",
                    plan.reasoning, query
                )
            } else {
                format!(
                    "Some commands failed:\n{}\n\n\
                     LLM reasoning: {}\n\
                     Query: \"{}\"",
                    failed_tools.join("\n"), plan.reasoning, query
                )
            }
        };

        OrchestrationResult {
            answer,
            plan: Some(plan),
            outputs,
            success: has_output,
        }
    }

    /// Ask the LLM to plan which commands to run
    fn plan_commands(&self, query: &str) -> Result<CommandPlan, LlmError> {
        let system_prompt = self.build_planner_system_prompt();
        let user_prompt = format!(
            "User query: \"{}\"\n\n\
             Based on the available tools, create a plan to answer this query.\n\
             Return a JSON object with:\n\
             - \"commands\": array of {{\"tool_id\": \"...\", \"extra_args\": [], \"purpose\": \"...\"}}\n\
             - \"reasoning\": brief explanation of why these commands\n\n\
             Only use tools from the catalog. If no tools are needed, return empty commands array.",
            query
        );

        let schema = r#"{
            "type": "object",
            "properties": {
                "commands": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "tool_id": {"type": "string"},
                            "extra_args": {"type": "array", "items": {"type": "string"}},
                            "purpose": {"type": "string"}
                        },
                        "required": ["tool_id", "purpose"]
                    }
                },
                "reasoning": {"type": "string"}
            },
            "required": ["commands", "reasoning"]
        }"#;

        let response = self.llm_client.call_json(&system_prompt, &user_prompt, schema)?;

        // Parse the response into CommandPlan
        let commands: Vec<PlannedCommand> = response
            .get("commands")
            .and_then(|c| c.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|cmd| {
                        let tool_id = cmd.get("tool_id")?.as_str()?.to_string();
                        let extra_args = cmd
                            .get("extra_args")
                            .and_then(|a| a.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(String::from))
                                    .collect()
                            })
                            .unwrap_or_default();
                        let purpose = cmd
                            .get("purpose")
                            .and_then(|p| p.as_str())
                            .unwrap_or("")
                            .to_string();

                        Some(PlannedCommand {
                            tool_id,
                            extra_args,
                            purpose,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let reasoning = response
            .get("reasoning")
            .and_then(|r| r.as_str())
            .unwrap_or("")
            .to_string();

        Ok(CommandPlan { commands, reasoning })
    }

    /// Execute the planned commands
    fn execute_plan(&self, plan: &CommandPlan) -> Vec<CommandOutput> {
        let mut outputs = Vec::new();

        for cmd in &plan.commands {
            // Validate tool is in catalog
            if let Some(spec) = self.catalog.get(&cmd.tool_id) {
                let result = run_tool(spec);
                outputs.push(CommandOutput {
                    tool_id: cmd.tool_id.clone(),
                    success: result.success,
                    stdout: result.stdout,
                    stderr: result.stderr,
                    exit_code: result.exit_code,
                });
            } else {
                outputs.push(CommandOutput {
                    tool_id: cmd.tool_id.clone(),
                    success: false,
                    stdout: String::new(),
                    stderr: format!("Tool '{}' not in catalog", cmd.tool_id),
                    exit_code: -1,
                });
            }
        }

        outputs
    }

    /// Ask the LLM to summarize the command outputs
    fn summarize_results(
        &self,
        query: &str,
        plan: &CommandPlan,
        outputs: &[CommandOutput],
    ) -> Result<String, LlmError> {
        let system_prompt = "You are a helpful Linux system assistant. \
            Your task is to answer the user's question based on command output. \
            Be concise and direct. Answer in 1-3 sentences. \
            You MUST respond with valid JSON: {\"answer\": \"your answer here\"}";

        // Build output summary - truncate long outputs
        let mut output_text = String::new();
        for output in outputs {
            let stdout = if output.stdout.len() > 1500 {
                format!("{}...[truncated]", &output.stdout[..1500])
            } else {
                output.stdout.clone()
            };

            output_text.push_str(&format!(
                "Command: {}\nOutput:\n{}\n\n",
                output.tool_id,
                if output.success { &stdout } else { &output.stderr }
            ));
        }

        let user_prompt = format!(
            "User question: \"{}\"\n\n\
             Command output:\n{}\n\n\
             Answer the user's question in 1-3 sentences. \
             Respond ONLY with JSON: {{\"answer\": \"your answer\"}}",
            query, output_text
        );

        let schema = r#"{"type": "object", "properties": {"answer": {"type": "string"}}, "required": ["answer"]}"#;

        let response = self.llm_client.call_json(system_prompt, &user_prompt, schema)?;

        // Helper to check if string looks like a real answer vs schema artifact
        let is_valid_answer = |s: &str| -> bool {
            !s.is_empty()
                && s.len() > 3  // Real answers are longer than "answer" or "object"
                && s != "object"
                && s != "string"
                && s != "answer"
                && !s.starts_with("{")
                && !s.starts_with("type")
        };

        // Try to extract answer from various response formats
        let answer = response
            .get("answer")
            .and_then(|a| a.as_str())
            .filter(|s| is_valid_answer(s))
            .map(|s| s.to_string())
            .or_else(|| {
                // Try as direct string response
                response.as_str()
                    .filter(|s| is_valid_answer(s))
                    .map(|s| s.to_string())
            })
            .or_else(|| {
                // Last resort: try to find any meaningful string in the response
                if let Some(obj) = response.as_object() {
                    for (key, v) in obj {
                        // Skip schema-like keys
                        if key == "type" || key == "properties" || key == "required" {
                            continue;
                        }
                        if let Some(s) = v.as_str() {
                            if is_valid_answer(s) {
                                return Some(s.to_string());
                            }
                        }
                    }
                }
                None
            });

        // Return answer or signal failure to trigger fallback
        match answer {
            Some(a) => Ok(a),
            None => Err(LlmError::InvalidJson("No valid answer in response".to_string())),
        }
    }

    /// Build the system prompt that includes the tool catalog
    fn build_planner_system_prompt(&self) -> String {
        let mut prompt = String::from(
            "You are an Arch Linux system assistant. \
             You have access to these system tools:\n\n",
        );

        for (id, spec) in &self.catalog {
            prompt.push_str(&format!(
                "- {}: {} (command: {} {})\n",
                id, spec.description, spec.binary,
                spec.args.join(" ")
            ));
        }

        prompt.push_str("\n\
            Rules:\n\
            1. Only use tools from the list above.\n\
            2. Select the minimal set of tools needed to answer the query.\n\
            3. If no tools are needed, return an empty commands array.\n\
            4. Explain your reasoning briefly.\n\
            5. For hardware info: use lscpu, free_mem, lspci_gpu, etc.\n\
            6. For packages: use pacman_query, pacman_orphans, etc.\n\
            7. For network: use ip_addr, nmcli_device, resolv_conf, etc.\n\
            8. For services: use systemctl_failed, journalctl_errors, etc.\n\
            9. For disk: use df_human, du_home_top, du_var_top, lsblk, etc.\n");

        prompt
    }

    /// Format raw output when LLM summarization fails
    fn format_raw_output(&self, query: &str, outputs: &[CommandOutput]) -> String {
        let mut result = format!("📋  Query: {}\n\n📊  Results:\n", query);

        for output in outputs {
            if output.success && !output.stdout.is_empty() {
                result.push_str(&output.stdout);
                result.push('\n');
            }
        }

        if result.ends_with("Results:\n") {
            result.push_str("No output from commands.");
        }

        result
    }
}

/// Build tool catalog description for LLM
pub fn get_tool_catalog_for_llm() -> String {
    let mut catalog_text = String::new();

    for spec in tool_catalog() {
        catalog_text.push_str(&format!(
            "- {}: {} (command: `{} {}`)\n",
            spec.id.name(),
            spec.description,
            spec.binary,
            spec.args.join(" ")
        ));
    }

    catalog_text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_catalog_for_llm() {
        let catalog = get_tool_catalog_for_llm();
        assert!(catalog.contains("free_mem"));
        assert!(catalog.contains("lscpu"));
        assert!(catalog.contains("df_human"));
    }
}
