//! Utility functions for knowledge fetching (v0.0.432).

use std::process::Command;

/// Run a shell command and return output.
pub fn run_command(cmd: &str) -> Result<String, String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .map_err(|e| e.to_string())?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !stdout.is_empty() {
        Ok(stdout)
    } else if !stderr.is_empty() {
        Ok(stderr)
    } else {
        Err("No output".to_string())
    }
}

/// Compute relevance of content to query (simple keyword matching).
pub fn compute_relevance(content: &str, query: &str) -> f32 {
    let content_lower = content.to_lowercase();
    let query_lower = query.to_lowercase();
    let query_words: Vec<&str> = query_lower
        .split_whitespace()
        .filter(|w| w.len() > 2)
        .collect();

    if query_words.is_empty() {
        return 0.5;
    }

    let matches = query_words
        .iter()
        .filter(|w| content_lower.contains(*w))
        .count();

    (matches as f32 / query_words.len() as f32).min(1.0)
}

/// Check if a word looks like a command.
pub fn is_likely_command(word: &str) -> bool {
    word.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        && word.len() >= 2
        && word
            .chars()
            .next()
            .map(|c| c.is_ascii_lowercase())
            .unwrap_or(false)
}

/// Sanitize a string for use as a filename.
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relevance_computation() {
        let content = "MemTotal: 32000000 kB\nMemFree: 16000000 kB\nAvailable memory: plenty";
        let relevance = compute_relevance(content, "memory free available");
        // "memory", "free", "available" are all > 2 chars and present in content
        assert!(relevance > 0.5, "relevance was {}", relevance);
    }
}
