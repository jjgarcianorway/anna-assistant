//! Output parsers for probe primitives.
//!
//! Functions for parsing raw command output into structured data.

use super::super::primitives::{ParserId, ProbePrimitive};
use super::output::{ParsedKind, ParsedOutput};
use std::collections::HashMap;

/// Parse output based on primitive's parser.
pub fn parse_output(primitive: &ProbePrimitive, output: &str) -> Option<ParsedOutput> {
    match primitive.parser {
        ParserId::TimeDuration => parse_boot_time(output),
        ParserId::Table => parse_service_list(output),
        ParserId::Numeric => parse_numeric(output),
        ParserId::Json => parse_json_output(output),
        ParserId::KeyValue => parse_key_value(output),
        ParserId::Raw => Some(ParsedOutput {
            kind: ParsedKind::Raw,
            summary: first_lines(output, 3),
            fields: HashMap::new(),
        }),
    }
}

/// Parse boot time from systemd-analyze.
fn parse_boot_time(output: &str) -> Option<ParsedOutput> {
    let mut fields = HashMap::new();

    // Parse "Startup finished in X (kernel) + Y (userspace) = Z"
    if let Some(total_start) = output.find("= ") {
        if let Some(total_end) = output[total_start..].find('\n') {
            let total = output[total_start + 2..total_start + total_end].trim();
            fields.insert("total".to_string(), total.to_string());
        }
    }

    Some(ParsedOutput {
        kind: ParsedKind::TimeMeasurement,
        summary: output.lines().next().unwrap_or("").to_string(),
        fields,
    })
}

/// Parse service list from systemctl.
fn parse_service_list(output: &str) -> Option<ParsedOutput> {
    let lines: Vec<&str> = output.lines().collect();
    let count = lines.len().saturating_sub(1); // Exclude header

    let mut fields = HashMap::new();
    fields.insert("count".to_string(), count.to_string());

    // Extract service names
    let services: Vec<String> = lines
        .iter()
        .skip(1)
        .filter_map(|line| line.split_whitespace().next())
        .map(|s| s.to_string())
        .take(5)
        .collect();

    fields.insert("services".to_string(), services.join(", "));

    Some(ParsedOutput {
        kind: ParsedKind::ItemList,
        summary: format!("{} items", count),
        fields,
    })
}

/// Parse numeric output (load average, etc.).
fn parse_numeric(output: &str) -> Option<ParsedOutput> {
    let mut fields = HashMap::new();

    // Extract numeric values from output
    let numbers: Vec<&str> = output
        .split_whitespace()
        .filter(|s| {
            s.chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
        })
        .take(5)
        .collect();

    for (i, num) in numbers.iter().enumerate() {
        fields.insert(format!("value_{}", i), num.to_string());
    }

    Some(ParsedOutput {
        kind: ParsedKind::KeyValue,
        summary: first_lines(output, 1),
        fields,
    })
}

/// Parse JSON output.
fn parse_json_output(output: &str) -> Option<ParsedOutput> {
    // Simple JSON key extraction for common patterns
    let mut fields = HashMap::new();

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.contains(':') && (trimmed.starts_with('"') || trimmed.starts_with('{')) {
            // Very basic JSON field extraction
            if let Some(key_start) = trimmed.find('"') {
                if let Some(key_end) = trimmed[key_start + 1..].find('"') {
                    let key = &trimmed[key_start + 1..key_start + 1 + key_end];
                    if let Some(val_start) = trimmed.find(':') {
                        let value = trimmed[val_start + 1..]
                            .trim()
                            .trim_matches(&['"', ','][..]);
                        fields.insert(key.to_string(), value.to_string());
                    }
                }
            }
        }
    }

    Some(ParsedOutput {
        kind: ParsedKind::KeyValue,
        summary: first_lines(output, 2),
        fields,
    })
}

/// Parse key=value output.
fn parse_key_value(output: &str) -> Option<ParsedOutput> {
    let mut fields = HashMap::new();

    for line in output.lines() {
        if let Some((key, value)) = line.split_once('=') {
            fields.insert(key.trim().to_string(), value.trim().to_string());
        }
    }

    Some(ParsedOutput {
        kind: ParsedKind::KeyValue,
        summary: first_lines(output, 2),
        fields,
    })
}

/// Get first N lines of output.
fn first_lines(output: &str, n: usize) -> String {
    output.lines().take(n).collect::<Vec<_>>().join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_boot_time() {
        let output = "Startup finished in 2.5s (kernel) + 5.3s (userspace) = 7.8s\n";
        let parsed = parse_boot_time(output).unwrap();

        assert!(matches!(parsed.kind, ParsedKind::TimeMeasurement));
        assert!(parsed.summary.contains("Startup finished"));
    }

    #[test]
    fn test_parse_key_value() {
        let output = "MemTotal:       16384000 kB\nMemFree:         8192000 kB\n";
        let parsed = parse_key_value(output).unwrap();

        assert!(matches!(parsed.kind, ParsedKind::KeyValue));
    }
}
