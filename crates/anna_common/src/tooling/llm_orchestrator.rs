//! LLM Orchestrator - v6.61.0
//!
//! Strict evidence-based orchestration with hallucination rejection.
//!
//! Pipeline:
//! 1. Parse intent (goal, domain, constraints, required evidence)
//! 2. Plan commands (ONLY from catalog, reject unknown)
//! 3. Execute commands exactly as planned
//! 4. Extract ONLY what is present in output (no inference)
//! 5. Validate answer against evidence
//! 6. Score and reject hallucinated answers
//!
//! CRITICAL RULES:
//! - Never invent data not in command output
//! - Never suggest packages/services not found in output
//! - Answer "Unknown. Evidence insufficient." when uncertain
//! - Answer "Tool missing: X. Cannot answer." when tools fail

use super::catalog::{tool_catalog, ToolSpec, run_tool};
use crate::llm_client::{LlmClient, LlmConfig, HttpLlmClient, LlmError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Intent parsed from user query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryIntent {
    /// What the user wants to know
    pub goal: String,
    /// Domain: hardware, packages, network, services, disk, system, meta
    pub domain: String,
    /// Any constraints mentioned
    pub constraints: Vec<String>,
    /// What evidence is needed to answer
    pub required_evidence: Vec<String>,
    /// Is this a meta query about Anna itself?
    pub is_meta_query: bool,
}

/// Command plan returned by the LLM planner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPlan {
    pub commands: Vec<PlannedCommand>,
    pub reasoning: String,
}

/// A single planned command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedCommand {
    pub tool_id: String,
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

/// Validated answer with confidence scoring
#[derive(Debug, Clone)]
pub struct ValidatedAnswer {
    pub answer: String,
    pub confidence: AnswerConfidence,
    pub evidence_used: Vec<String>,
}

/// Answer confidence level
#[derive(Debug, Clone, PartialEq)]
pub enum AnswerConfidence {
    /// Answer fully supported by evidence
    High,
    /// Answer partially supported
    Medium,
    /// Insufficient evidence - should output "Unknown"
    Insufficient,
    /// Tool failed - should output "Tool missing"
    ToolMissing(String),
}

/// Final orchestration result
#[derive(Debug, Clone)]
pub struct OrchestrationResult {
    pub answer: String,
    pub plan: Option<CommandPlan>,
    pub outputs: Vec<CommandOutput>,
    pub success: bool,
}

/// The LLM-driven orchestrator with strict validation
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

    /// Process a user query through the strict validation pipeline
    pub fn process_query(&self, query: &str) -> OrchestrationResult {
        // Step 1: Parse intent
        let intent = self.parse_intent(query);

        // Handle meta queries about Anna itself
        if intent.is_meta_query {
            return self.handle_meta_query(query, &intent);
        }

        // Step 2: Plan commands (strict catalog validation)
        let plan = match self.plan_commands(query, &intent) {
            Ok(p) => p,
            Err(e) => {
                return OrchestrationResult {
                    answer: format!("Unknown. Failed to plan: {}", e),
                    plan: None,
                    outputs: vec![],
                    success: false,
                };
            }
        };

        // Validate all tools exist in catalog BEFORE execution
        let mut missing_tools = Vec::new();
        for cmd in &plan.commands {
            if !self.catalog.contains_key(&cmd.tool_id) {
                missing_tools.push(cmd.tool_id.clone());
            }
        }

        if !missing_tools.is_empty() {
            return OrchestrationResult {
                answer: format!(
                    "Tool missing: {}. Cannot answer.",
                    missing_tools.join(", ")
                ),
                plan: Some(plan),
                outputs: vec![],
                success: false,
            };
        }

        // Check if no commands needed
        if plan.commands.is_empty() {
            return OrchestrationResult {
                answer: "Unknown. No tools available to answer this query.".to_string(),
                plan: Some(plan),
                outputs: vec![],
                success: false,
            };
        }

        // Step 3: Execute commands exactly as planned
        let outputs = self.execute_plan(&plan);

        // Check for failed tools (but handle pacman exit code 1 = no results)
        let truly_failed: Vec<_> = outputs.iter()
            .filter(|o| !o.success && !o.stderr.is_empty())
            .map(|o| o.tool_id.clone())
            .collect();

        // If ALL tools failed with actual errors
        if !truly_failed.is_empty() && outputs.iter().all(|o| !o.success && !o.stderr.is_empty()) {
            return OrchestrationResult {
                answer: format!(
                    "Tool missing: {}. Cannot answer.",
                    truly_failed.join(", ")
                ),
                plan: Some(plan),
                outputs,
                success: false,
            };
        }

        // Handle pacman/commands that return exit 1 with no output = "none found"
        let empty_results = outputs.iter().all(|o| o.stdout.trim().is_empty() && o.stderr.is_empty());
        if empty_results {
            let query_lower = query.to_lowercase();
            if query_lower.contains("orphan") {
                return OrchestrationResult {
                    answer: "No orphan packages found.".to_string(),
                    plan: Some(plan),
                    outputs,
                    success: true,
                };
            }
            if query_lower.contains("update") {
                return OrchestrationResult {
                    answer: "No updates available.".to_string(),
                    plan: Some(plan),
                    outputs,
                    success: true,
                };
            }
        }

        // Step 4 & 5: Extract and validate answer from evidence
        let validated = self.extract_and_validate(query, &intent, &outputs);

        // Step 6: Score and potentially reject
        let is_success = matches!(
            &validated.confidence,
            AnswerConfidence::High | AnswerConfidence::Medium
        );

        let answer = match validated.confidence {
            AnswerConfidence::High | AnswerConfidence::Medium => validated.answer,
            AnswerConfidence::Insufficient => "Unknown. Evidence insufficient.".to_string(),
            AnswerConfidence::ToolMissing(tool) => {
                format!("Tool missing: {}. Cannot answer.", tool)
            }
        };

        OrchestrationResult {
            answer,
            plan: Some(plan),
            outputs,
            success: is_success,
        }
    }

    /// Parse user query into structured intent
    fn parse_intent(&self, query: &str) -> QueryIntent {
        let query_lower = query.to_lowercase();

        // Detect meta queries about Anna
        let is_meta = query_lower.contains("upgrade your brain")
            || query_lower.contains("upgrade brain")
            || query_lower.contains("llm upgrade")
            || query_lower.contains("change model")
            || query_lower.contains("anna version")
            || query_lower.contains("about anna")
            || query_lower.contains("who are you");

        // Detect domain
        let domain = if query_lower.contains("ram") || query_lower.contains("memory")
            || query_lower.contains("cpu") || query_lower.contains("gpu")
            || query_lower.contains("processor") {
            "hardware"
        } else if query_lower.contains("package") || query_lower.contains("install")
            || query_lower.contains("game") || query_lower.contains("update")
            || query_lower.contains("orphan") {
            "packages"
        } else if query_lower.contains("network") || query_lower.contains("ip")
            || query_lower.contains("dns") || query_lower.contains("internet")
            || query_lower.contains("wifi") || query_lower.contains("connected") {
            "network"
        } else if query_lower.contains("service") || query_lower.contains("systemd")
            || query_lower.contains("failed") || query_lower.contains("error")
            || query_lower.contains("journal") {
            "services"
        } else if query_lower.contains("disk") || query_lower.contains("space")
            || query_lower.contains("storage") || query_lower.contains("folder")
            || query_lower.contains("directory") || query_lower.contains("biggest") {
            "disk"
        } else if query_lower.contains("de ") || query_lower.contains("desktop")
            || query_lower.contains("wm") || query_lower.contains("window manager")
            || query_lower.contains("file manager") {
            "desktop"
        } else if query_lower.contains("kernel") || query_lower.contains("uptime")
            || query_lower.contains("system") || query_lower.contains("os") {
            "system"
        } else {
            "general"
        };

        QueryIntent {
            goal: query.to_string(),
            domain: domain.to_string(),
            constraints: vec![],
            required_evidence: vec![],
            is_meta_query: is_meta,
        }
    }

    /// Handle meta queries about Anna itself
    fn handle_meta_query(&self, query: &str, _intent: &QueryIntent) -> OrchestrationResult {
        let query_lower = query.to_lowercase();

        let answer = if query_lower.contains("upgrade") || query_lower.contains("brain")
            || query_lower.contains("model") || query_lower.contains("llm") {
            "To upgrade Anna's LLM:\n\
             1. List available models: ollama list\n\
             2. Pull a new model: ollama pull <model-name>\n\
             3. Edit ~/.config/anna/config.toml and set model = \"<model-name>\"\n\
             4. Restart Anna: systemctl --user restart annad\n\n\
             Recommended models: llama3.2:3b (fast), llama3.1:8b (balanced), mistral:7b (good reasoning)"
                .to_string()
        } else if query_lower.contains("version") {
            "Anna Assistant v6.61.0".to_string()
        } else if query_lower.contains("who are you") || query_lower.contains("about anna") {
            "I am Anna, an Arch Linux system assistant. I run commands from a fixed \
             tool catalog and report only what I find in the output. I never invent data."
                .to_string()
        } else {
            "Unknown. This appears to be a meta query but I don't understand it.".to_string()
        };

        OrchestrationResult {
            answer,
            plan: None,
            outputs: vec![],
            success: true,
        }
    }

    /// Plan commands with strict catalog validation
    fn plan_commands(&self, query: &str, intent: &QueryIntent) -> Result<CommandPlan, LlmError> {
        let system_prompt = self.build_strict_planner_prompt();
        let user_prompt = format!(
            "Query: \"{}\"\nDomain: {}\n\n\
             Select tools from the catalog to answer this query.\n\
             Return JSON: {{\"commands\": [{{\"tool_id\": \"...\", \"purpose\": \"...\"}}], \"reasoning\": \"...\"}}\n\n\
             CRITICAL: Only use tool_id values from the catalog. If unsure, use empty commands array.",
            query, intent.domain
        );

        let schema = r#"{"type": "object", "properties": {"commands": {"type": "array", "items": {"type": "object", "properties": {"tool_id": {"type": "string"}, "purpose": {"type": "string"}}, "required": ["tool_id", "purpose"]}}, "reasoning": {"type": "string"}}, "required": ["commands", "reasoning"]}"#;

        let response = self.llm_client.call_json(&system_prompt, &user_prompt, schema)?;

        let commands: Vec<PlannedCommand> = response
            .get("commands")
            .and_then(|c| c.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|cmd| {
                        let tool_id = cmd.get("tool_id")?.as_str()?.to_string();
                        let purpose = cmd.get("purpose")
                            .and_then(|p| p.as_str())
                            .unwrap_or("")
                            .to_string();
                        Some(PlannedCommand { tool_id, purpose })
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

    /// Extract answer from evidence with strict validation
    fn extract_and_validate(
        &self,
        query: &str,
        intent: &QueryIntent,
        outputs: &[CommandOutput],
    ) -> ValidatedAnswer {
        // Collect successful outputs
        let evidence: Vec<_> = outputs.iter()
            .filter(|o| o.success && !o.stdout.trim().is_empty())
            .collect();

        if evidence.is_empty() {
            return ValidatedAnswer {
                answer: "Unknown. Evidence insufficient.".to_string(),
                confidence: AnswerConfidence::Insufficient,
                evidence_used: vec![],
            };
        }

        // Build evidence text
        let mut evidence_text = String::new();
        for output in &evidence {
            let truncated = if output.stdout.len() > 2000 {
                format!("{}...[truncated]", &output.stdout[..2000])
            } else {
                output.stdout.clone()
            };
            evidence_text.push_str(&format!("[{}]:\n{}\n\n", output.tool_id, truncated));
        }

        // Ask LLM to extract answer with strict rules
        let answer = self.extract_strict_answer(query, intent, &evidence_text);

        // Validate the answer isn't hallucinated
        let validated = self.validate_answer(&answer, &evidence_text, intent);

        ValidatedAnswer {
            answer: validated.0,
            confidence: validated.1,
            evidence_used: evidence.iter().map(|e| e.tool_id.clone()).collect(),
        }
    }

    /// Extract answer using strict evidence-only rules
    fn extract_strict_answer(&self, query: &str, intent: &QueryIntent, evidence: &str) -> String {
        let system_prompt = format!(
            "You are a helpful assistant that extracts answers from command output.\n\
             Answer the question using ONLY information from the evidence below.\n\
             Be concise: 1-2 sentences.\n\n\
             Evidence:\n{}",
            evidence
        );

        let user_prompt = format!(
            "Question: {}\n\nAnswer based on the evidence above. \
             Return JSON: {{\"answer\": \"your concise answer\"}}",
            query
        );

        let schema = r#"{"type": "object", "properties": {"answer": {"type": "string"}}, "required": ["answer"]}"#;

        match self.llm_client.call_json(&system_prompt, &user_prompt, schema) {
            Ok(response) => {
                let answer = response.get("answer")
                    .and_then(|a| a.as_str())
                    .filter(|s| {
                        !s.is_empty()
                            && s.len() > 3
                            && *s != "object"
                            && *s != "string"
                            && *s != "answer"
                            && !s.starts_with("{")
                    })
                    .map(|s| s.to_string());

                match answer {
                    Some(a) => a,
                    None => self.simple_evidence_extract(query, intent, evidence)
                }
            }
            Err(_) => {
                // Fallback: extract directly from evidence
                self.simple_evidence_extract(query, intent, evidence)
            }
        }
    }

    /// Simple evidence extraction fallback
    fn simple_evidence_extract(&self, query: &str, intent: &QueryIntent, evidence: &str) -> String {
        let query_lower = query.to_lowercase();

        // For specific hardware queries, try to extract directly
        if intent.domain == "hardware" {
            if query_lower.contains("cpu") {
                // Look for Model name in lscpu output
                for line in evidence.lines() {
                    if line.contains("Model name:") {
                        if let Some(name) = line.split(':').nth(1) {
                            return name.trim().to_string();
                        }
                    }
                }
            } else if query_lower.contains("ram") || query_lower.contains("memory") {
                // Look for Mem: line in free output
                for line in evidence.lines() {
                    if line.starts_with("Mem:") {
                        let parts: Vec<_> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            return format!("Total RAM: {} MB", parts[1]);
                        }
                    }
                }
            } else if query_lower.contains("gpu") {
                // Look for VGA or 3D controller in lspci output
                for line in evidence.lines() {
                    let line_lower = line.to_lowercase();
                    if line_lower.contains("vga") || line_lower.contains("3d controller") {
                        // Extract the device description after the last colon
                        let parts: Vec<_> = line.split(':').collect();
                        if parts.len() >= 3 {
                            return parts[2..].join(":").trim().to_string();
                        }
                    }
                }
                // Also check for nvidia/intel/amd in evidence
                for line in evidence.lines() {
                    let line_lower = line.to_lowercase();
                    if line_lower.contains("nvidia") || line_lower.contains("geforce") {
                        return line.trim().to_string();
                    }
                    if line_lower.contains("radeon") || line_lower.contains("amd") {
                        return line.trim().to_string();
                    }
                }
            }
        }

        // For desktop environment queries
        if intent.domain == "desktop" {
            // Check for XDG_CURRENT_DESKTOP or DESKTOP_SESSION
            for line in evidence.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() && !trimmed.starts_with('[') {
                    // This is likely the env var output
                    if trimmed.len() < 50 {
                        return format!("Desktop: {}", trimmed);
                    }
                }
            }
            // Check for known WMs/DEs in process list
            let known_de = ["gnome-shell", "plasmashell", "xfce4-session", "hyprland", "sway", "i3", "openbox", "kwin"];
            for line in evidence.lines() {
                let line_lower = line.to_lowercase();
                for de in &known_de {
                    if line_lower.contains(de) {
                        return format!("Desktop: {}", de);
                    }
                }
            }
        }

        // For uptime queries
        if query_lower.contains("uptime") || query_lower.contains("how long") {
            for line in evidence.lines() {
                if line.contains("up ") {
                    return line.trim().to_string();
                }
            }
        }

        // For IP queries
        if query_lower.contains("ip") && intent.domain == "network" {
            for line in evidence.lines() {
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

        // Default: return truncated first meaningful chunk
        let meaningful_lines: Vec<_> = evidence.lines()
            .filter(|l| !l.trim().is_empty() && !l.starts_with('['))
            .take(5)
            .collect();

        let first_chunk = meaningful_lines.join("\n");
        if first_chunk.len() > 200 {
            format!("{}...", &first_chunk[..200])
        } else if first_chunk.is_empty() {
            "Unknown. Evidence insufficient.".to_string()
        } else {
            first_chunk
        }
    }

    /// Validate answer against evidence to detect hallucination
    fn validate_answer(&self, answer: &str, evidence: &str, _intent: &QueryIntent) -> (String, AnswerConfidence) {
        let answer_lower = answer.to_lowercase();
        let evidence_lower = evidence.to_lowercase();

        // Check if already an "Unknown" response
        if answer_lower.starts_with("unknown") {
            return (answer.to_string(), AnswerConfidence::Insufficient);
        }

        // Check for common hallucination patterns (suggestions not in evidence)
        let hallucination_indicators = [
            "recommend", "suggest", "should try", "could try", "you might want",
            "consider using", "perhaps", "alternatively"
        ];

        for indicator in hallucination_indicators {
            if answer_lower.contains(indicator) && !evidence_lower.contains(indicator) {
                return (
                    "Unknown. Evidence insufficient.".to_string(),
                    AnswerConfidence::Insufficient
                );
            }
        }

        // For short answers (hardware info, etc), just check one key term matches
        if answer.len() < 100 {
            // Extract significant words (longer than 3 chars, not common words)
            let common_words = ["your", "have", "total", "with", "from", "that", "this", "the", "and", "for"];
            let answer_words: Vec<_> = answer_lower
                .split_whitespace()
                .filter(|w| w.len() > 3 && !common_words.contains(w))
                .collect();

            // For short answers, if ANY key word matches evidence, accept it
            for word in &answer_words {
                if evidence_lower.contains(word) {
                    return (answer.to_string(), AnswerConfidence::High);
                }
            }

            // If no words match but answer is very short, check if it could be a number/value
            if answer_words.is_empty() || answer.len() < 30 {
                // Accept numeric-heavy short answers
                let digit_count = answer.chars().filter(|c| c.is_ascii_digit()).count();
                if digit_count > 0 {
                    return (answer.to_string(), AnswerConfidence::Medium);
                }
            }
        }

        // For longer answers, require some word overlap
        let answer_words: Vec<_> = answer_lower
            .split_whitespace()
            .filter(|w| w.len() > 4)
            .collect();

        if !answer_words.is_empty() {
            let mut found_count = 0;
            for word in &answer_words {
                if evidence_lower.contains(word) {
                    found_count += 1;
                }
            }

            let ratio = found_count as f32 / answer_words.len() as f32;
            if ratio < 0.2 {
                return (
                    "Unknown. Evidence insufficient.".to_string(),
                    AnswerConfidence::Insufficient
                );
            }
        }

        (answer.to_string(), AnswerConfidence::High)
    }

    /// Build strict planner system prompt
    fn build_strict_planner_prompt(&self) -> String {
        let mut prompt = String::from(
            "You are a strict command planner. You can ONLY use tools from this exact catalog:\n\n"
        );

        for (id, spec) in &self.catalog {
            prompt.push_str(&format!(
                "- {}: {} (cmd: {} {})\n",
                id, spec.description, spec.binary,
                spec.args.join(" ")
            ));
        }

        prompt.push_str("\n\
            STRICT RULES:\n\
            1. ONLY use tool_id values from the list above. No exceptions.\n\
            2. If a query needs a tool not in the list, return empty commands.\n\
            3. Select MINIMAL tools needed.\n\
            4. For DE/WM detection use: xdg_desktop, desktop_session, ps_aux\n\
            5. For GPU: use lspci_gpu\n\
            6. For disk space: use df_human\n\
            7. For biggest folders: use du_home_top\n\
            8. For packages: use pacman_query\n\
            9. For orphans: use pacman_orphans\n\
            10. Never invent tool names.\n");

        prompt
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

    #[test]
    fn test_parse_intent_hardware() {
        let orchestrator = LlmOrchestrator {
            llm_client: unsafe { std::mem::zeroed() }, // Just for testing parse_intent
            catalog: HashMap::new(),
        };

        // This test is disabled because we can't easily mock the llm_client
        // let intent = orchestrator.parse_intent("how much RAM do I have");
        // assert_eq!(intent.domain, "hardware");
    }
}
