//! Probe Plan - Dynamic Probe Composition (v0.0.435).
//!
//! Anna builds a ProbePlan at runtime by selecting primitives based on
//! ticket intent and domain keywords.

use super::primitives::{Domain, PrimitiveLibrary, ProbePrimitive};
use super::citations::{CitationStore, Citation, EvidenceId};
use super::sources::KnowledgeSource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

/// A probe plan built for a specific ticket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbePlan {
    /// Ticket ID this plan is for.
    pub ticket_id: String,
    /// Selected primitive IDs.
    pub selected_primitives: Vec<String>,
    /// Why each primitive was selected.
    pub selection_reasons: HashMap<String, String>,
    /// Maximum probes to run.
    pub max_probes: usize,
    /// When the plan was created.
    pub created_at: u64,
}

impl ProbePlan {
    /// Create an empty plan for a ticket.
    pub fn new(ticket_id: &str) -> Self {
        Self {
            ticket_id: ticket_id.to_string(),
            selected_primitives: Vec::new(),
            selection_reasons: HashMap::new(),
            max_probes: super::MAX_PROBES_PER_TICKET,
            created_at: timestamp_now(),
        }
    }

    /// Select primitives based on keywords from intent.
    pub fn select_from_keywords(&mut self, keywords: &[&str], library: &PrimitiveLibrary) {
        // Find all primitives matching any of the keywords
        for primitive in library.find_by_keywords(keywords) {
            if self.selected_primitives.len() >= self.max_probes {
                break;
            }
            if !self.selected_primitives.contains(&primitive.id.to_string()) {
                self.selected_primitives.push(primitive.id.to_string());
                self.selection_reasons.insert(
                    primitive.id.to_string(),
                    format!("matched keywords"),
                );
            }
        }
    }

    /// Select primitives for a specific domain.
    pub fn select_for_domain(&mut self, domain: Domain, library: &PrimitiveLibrary) {
        for primitive in library.for_domain(domain) {
            if self.selected_primitives.len() >= self.max_probes {
                break;
            }
            if !self.selected_primitives.contains(&primitive.id.to_string()) {
                self.selected_primitives.push(primitive.id.to_string());
                self.selection_reasons.insert(
                    primitive.id.to_string(),
                    format!("domain {:?}", domain),
                );
            }
        }
    }

    /// Add a specific primitive by ID.
    pub fn add_primitive(&mut self, id: &str, reason: &str) {
        if self.selected_primitives.len() < self.max_probes
            && !self.selected_primitives.contains(&id.to_string())
        {
            self.selected_primitives.push(id.to_string());
            self.selection_reasons.insert(id.to_string(), reason.to_string());
        }
    }

    /// Get number of selected probes.
    pub fn probe_count(&self) -> usize {
        self.selected_primitives.len()
    }

    /// Check if plan is empty.
    pub fn is_empty(&self) -> bool {
        self.selected_primitives.is_empty()
    }
}

/// Result of executing a probe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeOutput {
    /// Primitive ID.
    pub primitive_id: String,
    /// Raw command output.
    pub raw_output: String,
    /// Parsed/structured output (if parser succeeded).
    pub parsed: Option<ParsedOutput>,
    /// Exit code.
    pub exit_code: Option<i32>,
    /// Execution time in ms.
    pub execution_time_ms: u64,
    /// Any error message.
    pub error: Option<String>,
}

impl ProbeOutput {
    /// Check if probe succeeded.
    pub fn success(&self) -> bool {
        self.exit_code == Some(0) && self.error.is_none()
    }

    /// Get summary for citation.
    pub fn summary(&self, max_len: usize) -> String {
        if let Some(parsed) = &self.parsed {
            parsed.summary.clone()
        } else if self.raw_output.len() > max_len {
            format!("{}...", &self.raw_output[..max_len])
        } else {
            self.raw_output.clone()
        }
    }
}

/// Parsed output from a probe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedOutput {
    /// Type of parsed data.
    pub kind: ParsedKind,
    /// Human-readable summary.
    pub summary: String,
    /// Key-value pairs extracted.
    pub fields: HashMap<String, String>,
}

/// Kind of parsed output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParsedKind {
    /// Time measurement (e.g., boot time).
    TimeMeasurement,
    /// List of items (e.g., failed services).
    ItemList,
    /// Generic key-value.
    KeyValue,
    /// Unparsed raw text.
    Raw,
}

/// Selection of primitives with context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeSelection {
    /// Selected primitive.
    pub primitive_id: String,
    /// Why it was selected.
    pub reason: String,
    /// Priority (lower = run first).
    pub priority: u8,
    /// Parameters to substitute.
    pub parameters: HashMap<String, String>,
}

impl ProbeSelection {
    /// Create a new selection.
    pub fn new(primitive_id: &str, reason: &str) -> Self {
        Self {
            primitive_id: primitive_id.to_string(),
            reason: reason.to_string(),
            priority: 100,
            parameters: HashMap::new(),
        }
    }

    /// Set priority.
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Add parameter.
    pub fn with_param(mut self, key: &str, value: &str) -> Self {
        self.parameters.insert(key.to_string(), value.to_string());
        self
    }
}

/// Executes probe plans and collects evidence.
pub struct ProbeExecutor {
    /// Library of primitives.
    library: PrimitiveLibrary,
    /// Default timeout in ms.
    default_timeout_ms: u64,
}

impl ProbeExecutor {
    /// Create a new executor.
    pub fn new() -> Self {
        Self {
            library: PrimitiveLibrary::default(),
            default_timeout_ms: super::DEFAULT_PROBE_TIMEOUT_MS,
        }
    }

    /// Execute a probe plan and return outputs.
    pub fn execute_plan(&self, plan: &ProbePlan) -> Vec<ProbeOutput> {
        let mut outputs = Vec::new();

        for primitive_id in &plan.selected_primitives {
            if let Some(primitive) = self.library.get(primitive_id) {
                let output = self.execute_primitive(primitive, &HashMap::new());
                outputs.push(output);
            }
        }

        outputs
    }

    /// Execute a single primitive.
    pub fn execute_primitive(
        &self,
        primitive: &ProbePrimitive,
        params: &HashMap<String, String>,
    ) -> ProbeOutput {
        let start = std::time::Instant::now();

        // Substitute parameters in command template
        let command = substitute_params(primitive.command_template, params);

        // Execute with timeout
        let timeout_ms = if primitive.timeout_ms > 0 {
            primitive.timeout_ms
        } else {
            self.default_timeout_ms
        };

        let result = execute_command(&command, timeout_ms);

        let execution_time_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok((output, exit_code)) => {
                let parsed = parse_output(primitive, &output);
                ProbeOutput {
                    primitive_id: primitive.id.to_string(),
                    raw_output: output,
                    parsed,
                    exit_code: Some(exit_code),
                    execution_time_ms,
                    error: None,
                }
            }
            Err(e) => ProbeOutput {
                primitive_id: primitive.id.to_string(),
                raw_output: String::new(),
                parsed: None,
                exit_code: None,
                execution_time_ms,
                error: Some(e),
            },
        }
    }

    /// Execute plan and add evidence to citation store.
    pub fn execute_with_citations(
        &self,
        plan: &ProbePlan,
        store: &mut CitationStore,
    ) -> Vec<ProbeOutput> {
        let outputs = self.execute_plan(plan);

        for output in &outputs {
            // Add raw evidence
            let evidence_id = EvidenceId::probe(&output.primitive_id);
            store.add_evidence(
                evidence_id.clone(),
                KnowledgeSource::ProbeOutput(output.primitive_id.clone()),
                &output.raw_output,
            );

            // Add citation for the output
            if output.success() {
                let excerpt = output.summary(super::MAX_CITATION_EXCERPT_LEN);
                store.add_citation(Citation::new(
                    evidence_id,
                    &format!("probe:{}", output.primitive_id),
                    &excerpt,
                ));
            }
        }

        outputs
    }
}

impl Default for ProbeExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Substitute parameters in command template.
fn substitute_params(template: &str, params: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in params {
        result = result.replace(&format!("{{{}}}", key), value);
    }
    result
}

/// Execute a shell command with timeout.
fn execute_command(command: &str, timeout_ms: u64) -> Result<(String, i32), String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .map_err(|e| format!("Failed to execute: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let combined = if stdout.is_empty() {
        stderr.to_string()
    } else {
        stdout.to_string()
    };

    let exit_code = output.status.code().unwrap_or(-1);

    Ok((combined, exit_code))
}

/// Parse output based on primitive's parser.
fn parse_output(primitive: &ProbePrimitive, output: &str) -> Option<ParsedOutput> {
    use super::primitives::ParserId;

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
        .filter(|s| s.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false))
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
                        let value = trimmed[val_start + 1..].trim().trim_matches(&['"', ','][..]);
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

/// Get current timestamp.
fn timestamp_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_plan_creation() {
        let plan = ProbePlan::new("ticket-123");
        assert_eq!(plan.ticket_id, "ticket-123");
        assert!(plan.is_empty());
    }

    #[test]
    fn test_probe_plan_select_keywords() {
        let mut plan = ProbePlan::new("test");
        let library = PrimitiveLibrary::new();

        plan.select_from_keywords(&["boot", "slow"], &library);
        assert!(!plan.is_empty());
        assert!(plan.selected_primitives.contains(&"sys.boot.analyze".to_string()));
    }

    #[test]
    fn test_probe_plan_max_probes() {
        let mut plan = ProbePlan::new("test");
        plan.max_probes = 2;
        let library = PrimitiveLibrary::new();

        plan.select_for_domain(Domain::Boot, &library);
        plan.select_for_domain(Domain::Memory, &library);

        assert!(plan.probe_count() <= 2);
    }

    #[test]
    fn test_substitute_params() {
        let template = "journalctl -u {service} --since {since}";
        let mut params = HashMap::new();
        params.insert("service".to_string(), "nginx".to_string());
        params.insert("since".to_string(), "1h ago".to_string());

        let result = substitute_params(template, &params);
        assert_eq!(result, "journalctl -u nginx --since 1h ago");
    }

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

    #[test]
    fn test_probe_selection() {
        let selection = ProbeSelection::new("sys.boot.analyze", "slow boot complaint")
            .with_priority(1)
            .with_param("service", "nginx");

        assert_eq!(selection.primitive_id, "sys.boot.analyze");
        assert_eq!(selection.priority, 1);
        assert_eq!(selection.parameters.get("service"), Some(&"nginx".to_string()));
    }

    #[test]
    fn test_probe_output_success() {
        let output = ProbeOutput {
            primitive_id: "test".to_string(),
            raw_output: "output".to_string(),
            parsed: None,
            exit_code: Some(0),
            execution_time_ms: 100,
            error: None,
        };
        assert!(output.success());

        let failed = ProbeOutput {
            primitive_id: "test".to_string(),
            raw_output: "".to_string(),
            parsed: None,
            exit_code: Some(1),
            execution_time_ms: 100,
            error: Some("failed".to_string()),
        };
        assert!(!failed.success());
    }
}
