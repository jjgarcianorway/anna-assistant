//! Recipe Executor - Safe execution of learned recipes (v0.0.412).
//!
//! Executes recipe steps with:
//! - Safety checks and user confirmation
//! - Dependency resolution
//! - Output collection for rendering
//! - Audit trail logging

use crate::doc_snippet::DocSnippet;
use crate::recipe_engine::{Recipe, RecipeStep, RecipeStepType, RiskLevel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use tracing::{debug, info, warn};

/// Result of executing a recipe
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Whether execution succeeded
    pub success: bool,
    /// Final rendered answer
    pub answer: String,
    /// Step outputs collected
    pub step_outputs: HashMap<String, StepOutput>,
    /// Any errors encountered
    pub errors: Vec<String>,
    /// Audit trail
    pub audit: Vec<AuditEntry>,
    /// Doc sources used
    pub sources: Vec<DocSnippet>,
}

/// Output from a single step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepOutput {
    pub step_id: String,
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration_ms: u64,
}

/// Audit trail entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: u64,
    pub step_id: String,
    pub action: String,
    pub details: String,
    pub backup_path: Option<String>,
}

/// Execution context with parameters and outputs
#[derive(Debug, Clone, Default)]
pub struct ExecutionContext {
    /// Recipe parameters (filled in from query)
    pub params: HashMap<String, String>,
    /// Step outputs so far
    pub outputs: HashMap<String, StepOutput>,
    /// Whether to skip confirmations (auto mode)
    pub auto_confirm: bool,
    /// Ticket ID for audit
    pub ticket_id: Option<String>,
    /// Recipe ID being executed
    pub recipe_id: Option<String>,
}

/// Recipe executor
pub struct RecipeExecutor {
    /// Confirmation callback (returns true if user confirms)
    confirm_fn: Option<Box<dyn Fn(&str) -> bool + Send + Sync>>,
}

impl Default for RecipeExecutor {
    fn default() -> Self {
        Self { confirm_fn: None }
    }
}

impl RecipeExecutor {
    /// Create new executor
    pub fn new() -> Self {
        Self::default()
    }

    /// Set confirmation callback
    pub fn with_confirm<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> bool + Send + Sync + 'static,
    {
        self.confirm_fn = Some(Box::new(f));
        self
    }

    /// Execute a recipe
    pub fn execute(&self, recipe: &Recipe, ctx: &mut ExecutionContext) -> ExecutionResult {
        let mut result = ExecutionResult {
            success: true,
            answer: String::new(),
            step_outputs: HashMap::new(),
            errors: vec![],
            audit: vec![],
            sources: vec![],
        };

        ctx.recipe_id = Some(recipe.id.clone());
        info!("Executing recipe: {} ({})", recipe.name, recipe.id);

        // Build dependency graph and execution order
        let order = match topological_sort(&recipe.steps) {
            Ok(o) => o,
            Err(e) => {
                result.success = false;
                result.errors.push(format!("Dependency error: {}", e));
                return result;
            }
        };

        // Execute steps in order
        for step_id in order {
            let step = match recipe.steps.iter().find(|s| s.id == step_id) {
                Some(s) => s,
                None => continue,
            };

            // Check dependencies succeeded
            if !step
                .depends_on
                .iter()
                .all(|dep| ctx.outputs.get(dep).map(|o| o.success).unwrap_or(false))
            {
                debug!("Skipping step {} - dependency not met", step_id);
                continue;
            }

            // Execute step
            match self.execute_step(step, recipe, ctx) {
                Ok(output) => {
                    let success = output.success;
                    ctx.outputs.insert(step_id.clone(), output.clone());
                    result.step_outputs.insert(step_id.clone(), output);

                    if !success && step.kind != RecipeStepType::CheckCondition {
                        result.success = false;
                        result.errors.push(format!("Step {} failed", step_id));
                        break;
                    }
                }
                Err(e) => {
                    result.success = false;
                    result.errors.push(format!("Step {} error: {}", step_id, e));
                    break;
                }
            }
        }

        // Find render_answer step output for final answer
        for step in &recipe.steps {
            if step.kind == RecipeStepType::RenderAnswer {
                if let Some(output) = result.step_outputs.get(&step.id) {
                    result.answer = output.stdout.clone();
                }
            }
        }

        // Add doc sources
        for doc_ref in &recipe.doc_sources {
            result.sources.push(DocSnippet::new(
                crate::doc_snippet::DocSourceKind::Builtin,
                doc_ref,
                "",
            ));
        }

        result
    }

    /// Execute a single step
    fn execute_step(
        &self,
        step: &RecipeStep,
        recipe: &Recipe,
        ctx: &mut ExecutionContext,
    ) -> Result<StepOutput, String> {
        let start = std::time::Instant::now();

        match step.kind {
            RecipeStepType::RunProbe => self.run_probe(step, ctx),
            RecipeStepType::RunCommand => self.run_command(step, recipe, ctx),
            RecipeStepType::CheckCondition => self.check_condition(step, ctx),
            RecipeStepType::EditFile => self.edit_file(step, recipe, ctx),
            RecipeStepType::RenderAnswer => self.render_answer(step, recipe, ctx),
            RecipeStepType::Subrecipe => Err("Subrecipe execution not yet implemented".to_string()),
        }
        .map(|mut output| {
            output.duration_ms = start.elapsed().as_millis() as u64;
            output
        })
    }

    /// Run a probe
    fn run_probe(&self, step: &RecipeStep, ctx: &ExecutionContext) -> Result<StepOutput, String> {
        let probe_id = step.params.get("probe_id").ok_or("Missing probe_id")?;
        let command = probe_id_to_command(probe_id);
        let command = substitute_params(&command, &ctx.params);

        debug!("Running probe: {}", command);
        run_shell_command(&command, &step.id)
    }

    /// Run a shell command with safety checks
    fn run_command(
        &self,
        step: &RecipeStep,
        recipe: &Recipe,
        ctx: &mut ExecutionContext,
    ) -> Result<StepOutput, String> {
        let command = step.params.get("command").ok_or("Missing command")?;
        let command = substitute_params(command, &ctx.params);

        // Check if confirmation needed
        if step.requires_confirmation() && !ctx.auto_confirm {
            let msg = format!(
                "Recipe '{}' wants to run:\n  {}\n\nAllow this command?",
                recipe.name, command
            );

            if let Some(ref confirm_fn) = self.confirm_fn {
                if !confirm_fn(&msg) {
                    return Ok(StepOutput {
                        step_id: step.id.clone(),
                        success: false,
                        stdout: String::new(),
                        stderr: "User declined".to_string(),
                        exit_code: -1,
                        duration_ms: 0,
                    });
                }
            }
        }

        debug!("Running command: {}", command);
        run_shell_command(&command, &step.id)
    }

    /// Check a condition
    fn check_condition(
        &self,
        step: &RecipeStep,
        ctx: &ExecutionContext,
    ) -> Result<StepOutput, String> {
        let condition = step.params.get("condition").ok_or("Missing condition")?;
        let target_step = step.params.get("target_step");

        // Get output from target step if specified
        let check_text = if let Some(target) = target_step {
            ctx.outputs
                .get(target)
                .map(|o| o.stdout.as_str())
                .unwrap_or("")
        } else {
            ""
        };

        // Simple condition checks
        let success = evaluate_condition(condition, check_text);

        Ok(StepOutput {
            step_id: step.id.clone(),
            success,
            stdout: if success {
                "condition met"
            } else {
                "condition not met"
            }
            .to_string(),
            stderr: String::new(),
            exit_code: if success { 0 } else { 1 },
            duration_ms: 0,
        })
    }

    /// Edit a file with safety
    fn edit_file(
        &self,
        step: &RecipeStep,
        recipe: &Recipe,
        ctx: &mut ExecutionContext,
    ) -> Result<StepOutput, String> {
        let file_path = step.params.get("file").ok_or("Missing file path")?;
        let file_path = substitute_params(file_path, &ctx.params);
        let content = step.params.get("content").ok_or("Missing content")?;
        let content = substitute_params(content, &ctx.params);
        let mode = step
            .params
            .get("mode")
            .map(|s| s.as_str())
            .unwrap_or("append");

        // Always ask for confirmation on file edits
        if !ctx.auto_confirm {
            let msg =
                format!(
                "Recipe '{}' wants to {} file:\n  {}\nWith content:\n  {}\n\nAllow this change?",
                recipe.name, mode, file_path, content.chars().take(100).collect::<String>()
            );

            if let Some(ref confirm_fn) = self.confirm_fn {
                if !confirm_fn(&msg) {
                    return Ok(StepOutput {
                        step_id: step.id.clone(),
                        success: false,
                        stdout: String::new(),
                        stderr: "User declined file edit".to_string(),
                        exit_code: -1,
                        duration_ms: 0,
                    });
                }
            }
        }

        // Create backup
        let backup_path = create_backup(&file_path)?;

        // Perform edit
        let result = match mode {
            "append" => append_to_file(&file_path, &content),
            "prepend" => prepend_to_file(&file_path, &content),
            "replace" => std::fs::write(&file_path, &content).map_err(|e| e.to_string()),
            _ => Err(format!("Unknown edit mode: {}", mode)),
        };

        match result {
            Ok(()) => Ok(StepOutput {
                step_id: step.id.clone(),
                success: true,
                stdout: format!("Edited {} (backup: {})", file_path, backup_path),
                stderr: String::new(),
                exit_code: 0,
                duration_ms: 0,
            }),
            Err(e) => Ok(StepOutput {
                step_id: step.id.clone(),
                success: false,
                stdout: String::new(),
                stderr: e,
                exit_code: 1,
                duration_ms: 0,
            }),
        }
    }

    /// Render the final answer
    fn render_answer(
        &self,
        step: &RecipeStep,
        recipe: &Recipe,
        ctx: &ExecutionContext,
    ) -> Result<StepOutput, String> {
        let template = step.params.get("template").ok_or("Missing template")?;

        // Substitute parameters
        let mut answer = substitute_params(template, &ctx.params);

        // Substitute step outputs ({{step_id.stdout}}, {{step_id.exit_code}})
        for (step_id, output) in &ctx.outputs {
            answer = answer.replace(&format!("{{{{{}.stdout}}}}", step_id), &output.stdout);
            answer = answer.replace(&format!("{{{{{}.stderr}}}}", step_id), &output.stderr);
            answer = answer.replace(
                &format!("{{{{{}.exit_code}}}}", step_id),
                &output.exit_code.to_string(),
            );
        }

        Ok(StepOutput {
            step_id: step.id.clone(),
            success: true,
            stdout: answer,
            stderr: String::new(),
            exit_code: 0,
            duration_ms: 0,
        })
    }
}

/// Map probe ID to actual command
fn probe_id_to_command(probe_id: &str) -> String {
    match probe_id {
        "memory_info" | "meminfo" => "free -h".to_string(),
        "disk_usage" | "df_root" => "df -h /".to_string(),
        "systemd_failed" => "systemctl --failed --no-pager".to_string(),
        "systemd_services" => "systemctl list-units --type=service --no-pager".to_string(),
        "pacman_list" => "pacman -Q".to_string(),
        "journal_errors" => "journalctl -p err -n 50 --no-pager".to_string(),
        "network_interfaces" => "ip addr".to_string(),
        "gpu_info" => "lspci | grep -i vga".to_string(),
        "audio_devices" => "pactl list sinks short 2>/dev/null || aplay -l".to_string(),
        _ => probe_id.to_string(), // Treat as raw command
    }
}

/// Substitute parameters in a string
fn substitute_params(template: &str, params: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in params {
        result = result.replace(&format!("{{{{{}}}}}", key), value);
    }
    // Expand ~ to home directory
    if result.starts_with("~/") || result.contains(" ~/") {
        if let Some(home) = dirs::home_dir() {
            result = result.replace("~/", &format!("{}/", home.display()));
        }
    }
    result
}

/// Run a shell command and capture output
fn run_shell_command(command: &str, step_id: &str) -> Result<StepOutput, String> {
    let output = Command::new("sh")
        .args(["-c", command])
        .output()
        .map_err(|e| format!("Failed to run command: {}", e))?;

    Ok(StepOutput {
        step_id: step_id.to_string(),
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
        duration_ms: 0,
    })
}

/// Evaluate a simple condition
fn evaluate_condition(condition: &str, text: &str) -> bool {
    // Simple condition language:
    // "contains:word" - text contains word
    // "not_empty" - text is not empty
    // "exit_code:0" - exit code is 0
    // "greater_than:N" - first number in text > N

    if condition.starts_with("contains:") {
        let word = &condition[9..];
        return text.to_lowercase().contains(&word.to_lowercase());
    }
    if condition == "not_empty" {
        return !text.trim().is_empty();
    }
    if condition.starts_with("exit_code:") {
        // This is checked via step success
        return true;
    }
    if condition.starts_with("greater_than:") {
        if let Ok(threshold) = condition[13..].parse::<i64>() {
            // Find first number in text
            for word in text.split_whitespace() {
                if let Ok(n) = word.parse::<i64>() {
                    return n > threshold;
                }
            }
        }
        return false;
    }

    // Default: non-empty is true
    !text.trim().is_empty()
}

/// Topological sort for step dependencies
fn topological_sort(steps: &[RecipeStep]) -> Result<Vec<String>, String> {
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();

    // Initialize
    for step in steps {
        in_degree.entry(&step.id).or_insert(0);
        graph.entry(&step.id).or_default();
        for dep in &step.depends_on {
            graph.entry(dep.as_str()).or_default().push(&step.id);
            *in_degree.entry(&step.id).or_insert(0) += 1;
        }
    }

    // Kahn's algorithm
    let mut queue: Vec<&str> = in_degree
        .iter()
        .filter(|(_, &d)| d == 0)
        .map(|(&id, _)| id)
        .collect();
    let mut result = vec![];

    while let Some(node) = queue.pop() {
        result.push(node.to_string());
        if let Some(neighbors) = graph.get(node) {
            for &neighbor in neighbors {
                if let Some(deg) = in_degree.get_mut(neighbor) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push(neighbor);
                    }
                }
            }
        }
    }

    if result.len() != steps.len() {
        return Err("Circular dependency detected".to_string());
    }

    Ok(result)
}

/// Create backup of a file
fn create_backup(path: &str) -> Result<String, String> {
    let backup_dir = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
        .join(".anna")
        .join("backups");

    std::fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let filename = std::path::Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let backup_path = backup_dir.join(format!("{}_{}", filename, timestamp));

    if std::path::Path::new(path).exists() {
        std::fs::copy(path, &backup_path).map_err(|e| e.to_string())?;
    }

    Ok(backup_path.to_string_lossy().to_string())
}

/// Append content to file
fn append_to_file(path: &str, content: &str) -> Result<(), String> {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    file.write_all(content.as_bytes())
        .map_err(|e| e.to_string())
}

/// Prepend content to file
fn prepend_to_file(path: &str, content: &str) -> Result<(), String> {
    let existing = std::fs::read_to_string(path).unwrap_or_default();
    let new_content = format!("{}{}", content, existing);
    std::fs::write(path, new_content).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substitute_params() {
        let mut params = HashMap::new();
        params.insert("service".to_string(), "nginx".to_string());

        let result = substitute_params("systemctl status {{service}}", &params);
        assert_eq!(result, "systemctl status nginx");
    }

    #[test]
    fn test_evaluate_condition() {
        assert!(evaluate_condition("contains:error", "There was an error"));
        assert!(!evaluate_condition("contains:error", "All is well"));
        assert!(evaluate_condition("not_empty", "some text"));
        assert!(!evaluate_condition("not_empty", "   "));
    }

    #[test]
    fn test_topological_sort() {
        let steps = vec![
            RecipeStep::probe("s1", "meminfo", "Get memory"),
            RecipeStep::probe("s2", "df", "Get disk").depends("s1"),
            RecipeStep::render("s3", "Done", "Final").depends("s2"),
        ];

        let order = topological_sort(&steps).unwrap();
        assert_eq!(order, vec!["s1", "s2", "s3"]);
    }
}
