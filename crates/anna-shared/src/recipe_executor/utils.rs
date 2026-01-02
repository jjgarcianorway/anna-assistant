//! Utility functions for recipe execution.

use crate::recipe_engine::RecipeStep;
use std::collections::HashMap;
use std::process::Command;

use super::types::StepOutput;

/// Map probe ID to actual command
pub fn probe_id_to_command(probe_id: &str) -> String {
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
pub fn substitute_params(template: &str, params: &HashMap<String, String>) -> String {
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
pub fn run_shell_command(command: &str, step_id: &str) -> Result<StepOutput, String> {
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
pub fn topological_sort(steps: &[RecipeStep]) -> Result<Vec<String>, String> {
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
