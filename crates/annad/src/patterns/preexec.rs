//! Pattern pre-execution for grounded answers.

use anna_shared::rpc::DeepUnderstanding;
use tracing::debug;

use super::match_common_pattern;
use crate::core_loop::execute_command;

/// Result of pattern pre-execution.
pub struct PatternPreExecResult {
    pub understanding: DeepUnderstanding,
    pub command_outputs: Vec<(String, String)>,
}

/// Match pattern and pre-execute suggested commands for grounded answers.
/// This provides fresh command output to the LLM without needing an extra round-trip.
pub fn match_and_preexec(question: &str) -> Option<PatternPreExecResult> {
    let understanding = match_common_pattern(question)?;

    // Only pre-execute for high-confidence factual queries
    if understanding.confidence < 0.85 || understanding.suggested_commands.is_empty() {
        return Some(PatternPreExecResult {
            understanding,
            command_outputs: vec![],
        });
    }

    // Execute up to 3 suggested commands
    let mut outputs = Vec::new();
    for cmd in understanding.suggested_commands.iter().take(3) {
        // Skip dangerous commands (pre-execution should be read-only)
        let cmd_lower = cmd.to_lowercase();
        if cmd_lower.contains("rm ")
            || cmd_lower.contains("dd ")
            || cmd_lower.contains("mkfs")
            || cmd_lower.contains("> /")
            || cmd_lower.contains("sudo ")
        {
            debug!("Pattern pre-exec: skipping potentially dangerous command: {}", cmd);
            continue;
        }

        match execute_command(cmd) {
            Ok(output) if !output.trim().is_empty() => {
                debug!("Pattern pre-exec: got output for '{}'", cmd);
                outputs.push((cmd.clone(), output));
            }
            Ok(_) => {
                debug!("Pattern pre-exec: empty output for '{}'", cmd);
            }
            Err(e) => {
                debug!("Pattern pre-exec: failed '{}': {}", cmd, e);
            }
        }
    }

    Some(PatternPreExecResult {
        understanding,
        command_outputs: outputs,
    })
}
