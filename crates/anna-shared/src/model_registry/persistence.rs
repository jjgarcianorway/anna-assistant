//! Model registry persistence (v0.0.201).

use std::path::PathBuf;

use super::registry::ModelRegistry;
use super::types::ModelState;

/// Path to model registry file
pub fn model_registry_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    PathBuf::from(home)
        .join(".anna")
        .join("model_registry.json")
}

/// Load model registry from disk
pub fn load_model_registry() -> ModelRegistry {
    let path = model_registry_path();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(registry) = serde_json::from_str(&content) {
                return registry;
            }
        }
    }
    ModelRegistry::new()
}

/// Save model registry to disk
pub fn save_model_registry(registry: &ModelRegistry) -> std::io::Result<()> {
    let path = model_registry_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(registry).map_err(|e| std::io::Error::other(e))?;
    std::fs::write(&path, content)
}

/// Parse ollama list output to extract model states
pub fn parse_ollama_list(output: &str) -> Vec<(String, ModelState)> {
    let mut states = Vec::new();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Skip header line
    for line in output.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let name = parts[0].to_string();
            // Try to parse size (e.g., "2.0 GB" or "1.5GB")
            let size_bytes = if parts.len() >= 3 {
                parse_size_string(parts[2])
            } else {
                None
            };

            states.push((
                name,
                ModelState {
                    present: true,
                    digest: None,
                    last_seen_ts: Some(now),
                    size_bytes,
                },
            ));
        }
    }

    states
}

/// Parse size string like "2.0 GB" or "1.5GB" to bytes
fn parse_size_string(s: &str) -> Option<u64> {
    let s = s.trim().to_uppercase();
    let (num_str, multiplier) = if s.ends_with("GB") {
        (s.trim_end_matches("GB").trim(), 1024 * 1024 * 1024)
    } else if s.ends_with("MB") {
        (s.trim_end_matches("MB").trim(), 1024 * 1024)
    } else if s.ends_with("KB") {
        (s.trim_end_matches("KB").trim(), 1024)
    } else {
        return None;
    };

    num_str
        .parse::<f64>()
        .ok()
        .map(|n| (n * multiplier as f64) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ollama_list() {
        let output = "NAME              ID              SIZE      MODIFIED\n\
                      llama3.2:3b       abc123def456    2.0 GB    2 days ago\n\
                      qwen2.5:1.5b      789xyz012345    1.0 GB    1 week ago";

        let states = parse_ollama_list(output);
        assert_eq!(states.len(), 2);
        assert_eq!(states[0].0, "llama3.2:3b");
        assert!(states[0].1.present);
        assert_eq!(states[1].0, "qwen2.5:1.5b");
    }

    #[test]
    fn test_parse_size_string() {
        assert_eq!(parse_size_string("2.0 GB"), Some(2 * 1024 * 1024 * 1024));
        assert_eq!(
            parse_size_string("1.5GB"),
            Some((1.5 * 1024.0 * 1024.0 * 1024.0) as u64)
        );
        assert_eq!(parse_size_string("512 MB"), Some(512 * 1024 * 1024));
        assert_eq!(parse_size_string("invalid"), None);
    }
}
