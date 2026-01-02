//! Variable extraction from probe outputs.

use std::collections::HashMap;

/// Extract variables from probe output
pub fn extract_variables_from_output(probe_id: &str, output: &str) -> HashMap<String, String> {
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
                        vars.insert(
                            "available_mem".to_string(),
                            parts.get(6).unwrap_or(&"").to_string(),
                        );
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_extraction() {
        let free_output = "              total        used        free      shared  buff/cache   available\nMem:           16Gi       8.0Gi       4.0Gi       1.0Gi       4.0Gi       7.0Gi";
        let vars = extract_variables_from_output("free", free_output);

        assert_eq!(vars.get("total_mem"), Some(&"16Gi".to_string()));
        assert_eq!(vars.get("available_mem"), Some(&"7.0Gi".to_string()));
    }
}
