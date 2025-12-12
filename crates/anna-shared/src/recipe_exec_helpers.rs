//! Recipe Executor Helpers (v0.0.412).
//!
//! Helper functions for recipe execution:
//! - Parameter substitution
//! - Shell command execution
//! - Condition evaluation
//! - Topological sort
//! - File operations

use std::collections::HashMap;
use std::process::Command;

/// Output from a single step
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StepOutput {
    pub step_id: String,
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration_ms: u64,
}

impl Default for StepOutput {
    fn default() -> Self {
        Self {
            step_id: String::new(),
            success: false,
            stdout: String::new(),
            stderr: String::new(),
            exit_code: -1,
            duration_ms: 0,
        }
    }
}

/// Substitute {{param}} placeholders in text
pub fn substitute_params(text: &str, params: &HashMap<String, String>) -> String {
    let mut result = text.to_string();
    for (key, value) in params {
        result = result.replace(&format!("{{{{{}}}}}", key), value);
    }
    // Handle step output references like {{s1.stdout}}
    // These should be substituted by the caller with actual step outputs
    result
}

/// Substitute step outputs in template ({{s1.stdout}}, {{s2.stderr}}, etc.)
pub fn substitute_step_outputs(text: &str, outputs: &HashMap<String, StepOutput>) -> String {
    let mut result = text.to_string();
    for (step_id, output) in outputs {
        result = result.replace(&format!("{{{{{}.stdout}}}}", step_id), &output.stdout);
        result = result.replace(&format!("{{{{{}.stderr}}}}", step_id), &output.stderr);
        result = result.replace(
            &format!("{{{{{}.exit_code}}}}", step_id),
            &output.exit_code.to_string(),
        );
    }
    result
}

/// Expand home directory in paths
pub fn expand_home(text: &str) -> String {
    let mut result = text.to_string();
    if result.starts_with("~/") || result.contains(" ~/") {
        if let Some(home) = dirs::home_dir() {
            result = result.replace("~/", &format!("{}/", home.display()));
        }
    }
    result
}

/// Run a shell command and capture output
pub fn run_shell_command(command: &str, step_id: &str) -> Result<StepOutput, String> {
    let start = std::time::Instant::now();
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
        duration_ms: start.elapsed().as_millis() as u64,
    })
}

/// Evaluate a simple condition
pub fn evaluate_condition(condition: &str, text: &str) -> bool {
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
pub fn topological_sort(
    step_ids: &[String],
    deps: &HashMap<String, Vec<String>>,
) -> Result<Vec<String>, String> {
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();

    // Initialize
    for step_id in step_ids {
        in_degree.entry(step_id.as_str()).or_insert(0);
        graph.entry(step_id.as_str()).or_default();
    }

    // Build dependency graph
    for (step_id, step_deps) in deps {
        for dep in step_deps {
            graph
                .entry(dep.as_str())
                .or_default()
                .push(step_id.as_str());
            *in_degree.entry(step_id.as_str()).or_insert(0) += 1;
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

    if result.len() != step_ids.len() {
        return Err("Circular dependency detected".to_string());
    }

    Ok(result)
}

/// Create backup of a file
pub fn create_backup(path: &str) -> Result<String, String> {
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
pub fn append_to_file(path: &str, content: &str) -> Result<(), String> {
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
pub fn prepend_to_file(path: &str, content: &str) -> Result<(), String> {
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
        let ids = vec!["s1".to_string(), "s2".to_string(), "s3".to_string()];
        let mut deps = HashMap::new();
        deps.insert("s2".to_string(), vec!["s1".to_string()]);
        deps.insert("s3".to_string(), vec!["s2".to_string()]);

        let order = topological_sort(&ids, &deps).unwrap();
        assert_eq!(order, vec!["s1", "s2", "s3"]);
    }
}
