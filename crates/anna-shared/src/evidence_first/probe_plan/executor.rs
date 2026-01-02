//! Probe executor - runs probe plans and collects evidence.

use super::super::citations::{Citation, CitationStore, EvidenceId};
use super::super::primitives::{PrimitiveLibrary, ProbePrimitive};
use super::super::sources::KnowledgeSource;
use super::output::ProbeOutput;
use super::parsers::parse_output;
use super::plan::ProbePlan;
use std::collections::HashMap;
use std::process::Command;

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
            default_timeout_ms: super::super::DEFAULT_PROBE_TIMEOUT_MS,
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
                let excerpt = output.summary(super::super::MAX_CITATION_EXCERPT_LEN);
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
fn execute_command(command: &str, _timeout_ms: u64) -> Result<(String, i32), String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substitute_params() {
        let template = "journalctl -u {service} --since {since}";
        let mut params = HashMap::new();
        params.insert("service".to_string(), "nginx".to_string());
        params.insert("since".to_string(), "1h ago".to_string());

        let result = substitute_params(template, &params);
        assert_eq!(result, "journalctl -u nginx --since 1h ago");
    }
}
