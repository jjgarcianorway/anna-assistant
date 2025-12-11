//! Recipe executor for learning engine (v0.0.427).
//!
//! Executes recipes without calling LLM:
//! - Runs probes
//! - Fills answer templates
//! - Tracks success/failure

use super::{LearnedRecipe, RecipeMatch, RecipeProbe};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result of recipe execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Recipe ID that was executed
    pub recipe_id: String,
    /// Whether execution was successful
    pub success: bool,
    /// Short answer
    pub short_answer: String,
    /// Detailed answer
    pub detailed_answer: String,
    /// Probe results
    pub probe_results: HashMap<String, ProbeResult>,
    /// Variables extracted
    pub variables: HashMap<String, String>,
    /// Execution time in milliseconds
    pub execution_ms: u64,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Whether this was a recipe-based resolution (no LLM)
    pub recipe_based: bool,
}

impl ExecutionResult {
    /// Create a successful result
    pub fn success(recipe_id: &str, short: &str, detailed: &str) -> Self {
        Self {
            recipe_id: recipe_id.to_string(),
            success: true,
            short_answer: short.to_string(),
            detailed_answer: detailed.to_string(),
            probe_results: HashMap::new(),
            variables: HashMap::new(),
            execution_ms: 0,
            error: None,
            recipe_based: true,
        }
    }

    /// Create a failed result
    pub fn failure(recipe_id: &str, error: &str) -> Self {
        Self {
            recipe_id: recipe_id.to_string(),
            success: false,
            short_answer: String::new(),
            detailed_answer: String::new(),
            probe_results: HashMap::new(),
            variables: HashMap::new(),
            execution_ms: 0,
            error: Some(error.to_string()),
            recipe_based: true,
        }
    }

    /// Add a probe result
    pub fn with_probe(mut self, id: &str, result: ProbeResult) -> Self {
        self.probe_results.insert(id.to_string(), result);
        self
    }

    /// Set execution time
    pub fn with_time(mut self, ms: u64) -> Self {
        self.execution_ms = ms;
        self
    }
}

/// Result of a single probe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResult {
    /// Probe ID
    pub probe_id: String,
    /// Whether probe succeeded
    pub success: bool,
    /// Output from probe
    pub output: String,
    /// Execution time in ms
    pub duration_ms: u64,
    /// Error if failed
    pub error: Option<String>,
}

impl ProbeResult {
    /// Create a successful probe result
    pub fn ok(probe_id: &str, output: &str, duration_ms: u64) -> Self {
        Self {
            probe_id: probe_id.to_string(),
            success: true,
            output: output.to_string(),
            duration_ms,
            error: None,
        }
    }

    /// Create a failed probe result
    pub fn failed(probe_id: &str, error: &str) -> Self {
        Self {
            probe_id: probe_id.to_string(),
            success: false,
            output: String::new(),
            duration_ms: 0,
            error: Some(error.to_string()),
        }
    }
}

/// Probe executor trait (for testing and different implementations)
pub trait ProbeExecutor {
    /// Execute a probe and return result
    fn execute(&self, probe: &RecipeProbe, params: &HashMap<String, String>) -> ProbeResult;
}

/// Default probe executor that runs shell commands
pub struct ShellProbeExecutor;

impl ProbeExecutor for ShellProbeExecutor {
    fn execute(&self, probe: &RecipeProbe, params: &HashMap<String, String>) -> ProbeResult {
        let start = std::time::Instant::now();

        // Build command from probe tool
        let command = build_probe_command(&probe.tool, &probe.params, params);

        // Execute with timeout
        let result = std::process::Command::new("sh")
            .arg("-c")
            .arg(&command)
            .output();

        let duration = start.elapsed().as_millis() as u64;

        match result {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    ProbeResult::ok(&probe.id, &stdout, duration)
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    ProbeResult::failed(&probe.id, &stderr)
                }
            }
            Err(e) => ProbeResult::failed(&probe.id, &e.to_string()),
        }
    }
}

/// Build probe command from tool and params
fn build_probe_command(
    tool: &str,
    probe_params: &[String],
    values: &HashMap<String, String>,
) -> String {
    // Map tool names to actual commands
    let base_command = match tool.trim_start_matches("probe.") {
        "free" => "free -h",
        "df" => "df -h",
        "systemctl_status" => "systemctl status",
        "systemctl" => "systemctl --no-pager",
        "journalctl" => "journalctl --no-pager -n 20",
        "ps" => "ps aux",
        "uptime" => "uptime",
        "vmstat" => "vmstat 1 1",
        "ip_addr" => "ip addr",
        "ss" => "ss -tuln",
        "lsblk" => "lsblk",
        "mount" => "mount",
        _ => tool,
    };

    // Substitute parameters
    let mut command = base_command.to_string();
    for param in probe_params {
        let value = if param.starts_with("{{") && param.ends_with("}}") {
            let key = &param[2..param.len() - 2];
            values.get(key).map(|s| s.as_str()).unwrap_or("")
        } else {
            values.get(param).map(|s| s.as_str()).unwrap_or(param)
        };

        if !value.is_empty() {
            command.push(' ');
            command.push_str(value);
        }
    }

    command
}

/// Execute a recipe
pub fn execute_recipe<E: ProbeExecutor>(
    recipe: &LearnedRecipe,
    match_result: &RecipeMatch,
    executor: &E,
) -> ExecutionResult {
    let start = std::time::Instant::now();

    // Merge params from match with any defaults
    let mut params = match_result.params.clone();

    // Execute required probes
    let mut probe_results = HashMap::new();
    let mut all_outputs = HashMap::new();

    for probe in &recipe.probes {
        let result = executor.execute(probe, &params);

        if !result.success && !probe.optional {
            // Required probe failed
            let elapsed = start.elapsed().as_millis() as u64;
            return ExecutionResult::failure(
                &recipe.id,
                &format!("Required probe {} failed: {:?}", probe.id, result.error),
            )
            .with_time(elapsed);
        }

        if result.success {
            // Extract variables from probe output
            let extracted = extract_variables_from_output(&probe.id, &result.output);
            all_outputs.extend(extracted);
        }

        probe_results.insert(probe.id.clone(), result);
    }

    // Merge extracted values with params
    params.extend(all_outputs.clone());

    // Fill answer templates
    let short = recipe.answer_template.render_short(&params);
    let detailed = recipe.answer_template.render_detailed(&params);

    let elapsed = start.elapsed().as_millis() as u64;

    let mut result = ExecutionResult::success(&recipe.id, &short, &detailed);
    result.probe_results = probe_results;
    result.variables = params;
    result.execution_ms = elapsed;

    result
}

/// Extract variables from probe output
fn extract_variables_from_output(probe_id: &str, output: &str) -> HashMap<String, String> {
    let mut vars = HashMap::new();

    match probe_id.trim_start_matches("probe:") {
        "free" => {
            // Parse free output
            for line in output.lines() {
                if line.starts_with("Mem:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 7 {
                        vars.insert("total_mem".to_string(), parts[1].to_string());
                        vars.insert("used_mem".to_string(), parts[2].to_string());
                        vars.insert("free_mem".to_string(), parts[3].to_string());
                        vars.insert("available_mem".to_string(), parts.get(6).unwrap_or(&"").to_string());
                    }
                }
            }
        }
        "df" => {
            // Parse df output for root filesystem
            for line in output.lines() {
                if line.contains(" /") && !line.contains(" /boot") && !line.contains(" /home") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 5 {
                        vars.insert("disk_used".to_string(), parts[2].to_string());
                        vars.insert("disk_available".to_string(), parts[3].to_string());
                        vars.insert("disk_percent".to_string(), parts[4].to_string());
                    }
                    break;
                }
            }
        }
        "uptime" => {
            // Extract uptime
            if let Some(up_idx) = output.find("up ") {
                let rest = &output[up_idx + 3..];
                if let Some(end) = rest.find(',') {
                    vars.insert("uptime".to_string(), rest[..end].trim().to_string());
                }
            }
        }
        _ => {
            // Generic: just store raw output
            vars.insert(format!("{}_output", probe_id), output.trim().to_string());
        }
    }

    vars
}

/// Check if all recipe requirements are met
pub fn can_execute(recipe: &LearnedRecipe, match_result: &RecipeMatch) -> bool {
    // Check if strong match
    if !match_result.is_strong() {
        return false;
    }

    // Check if all required params are available
    for param_name in recipe.inputs.required_params() {
        if !match_result.params.contains_key(&param_name) {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::learning_engine::{AnswerTemplate, RecipePattern};

    struct MockExecutor;

    impl ProbeExecutor for MockExecutor {
        fn execute(&self, probe: &RecipeProbe, _params: &HashMap<String, String>) -> ProbeResult {
            match probe.id.as_str() {
                "probe:free" => ProbeResult::ok(
                    "probe:free",
                    "              total        used        free      shared  buff/cache   available\nMem:           16Gi       8.0Gi       4.0Gi       1.0Gi       4.0Gi       7.0Gi",
                    50,
                ),
                _ => ProbeResult::ok(&probe.id, "mock output", 10),
            }
        }
    }

    fn make_recipe() -> LearnedRecipe {
        LearnedRecipe::new("test-ram", "performance.memory")
            .with_pattern(RecipePattern::new("check_free_ram"))
            .with_probe(RecipeProbe::new("probe:free", "probe.free"))
            .with_answer(
                "Available RAM: {{available_mem}}",
                "Memory Status:\n  Total: {{total_mem}}\n  Used: {{used_mem}}\n  Available: {{available_mem}}",
            )
    }

    fn make_match(recipe_id: &str, score: f32) -> RecipeMatch {
        RecipeMatch {
            recipe_id: recipe_id.to_string(),
            score,
            breakdown: Default::default(),
            params: HashMap::new(),
            missing_signals: vec![],
        }
    }

    #[test]
    fn test_execute_recipe() {
        let recipe = make_recipe();
        let match_result = make_match("test-ram", 0.9);
        let executor = MockExecutor;

        let result = execute_recipe(&recipe, &match_result, &executor);

        assert!(result.success);
        assert!(result.short_answer.contains("7.0Gi"));
        assert!(result.recipe_based);
    }

    #[test]
    fn test_probe_command_building() {
        let mut params = HashMap::new();
        params.insert("service_name".to_string(), "nginx".to_string());

        let cmd = build_probe_command("probe.systemctl_status", &["{{service_name}}".to_string()], &params);
        assert!(cmd.contains("nginx"));
    }

    #[test]
    fn test_variable_extraction() {
        let free_output = "              total        used        free      shared  buff/cache   available\nMem:           16Gi       8.0Gi       4.0Gi       1.0Gi       4.0Gi       7.0Gi";
        let vars = extract_variables_from_output("free", free_output);

        assert_eq!(vars.get("total_mem"), Some(&"16Gi".to_string()));
        assert_eq!(vars.get("available_mem"), Some(&"7.0Gi".to_string()));
    }

    #[test]
    fn test_can_execute() {
        let recipe = make_recipe();
        let strong_match = make_match("test-ram", 0.9);
        let weak_match = make_match("test-ram", 0.5);

        assert!(can_execute(&recipe, &strong_match));
        assert!(!can_execute(&recipe, &weak_match));
    }
}
