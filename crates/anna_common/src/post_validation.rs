//! Post-Validation - LLM-based assessment after execution
//!
//! v6.50.0: Query LLM to assess satisfaction and detect issues

use crate::action_episodes::PostValidation;
use crate::llm_client::LlmClient;
use anyhow::{Context, Result};
use serde_json::json;

/// Query LLM to validate execution results
///
/// # Arguments
/// * `llm_client` - LLM client to query
/// * `user_question` - Original user request
/// * `plan_description` - What the plan intended to do
/// * `commands_executed` - List of commands that were run
/// * `execution_output` - Combined output from all commands
///
/// # Returns
/// PostValidation struct with satisfaction score and assessment
pub fn validate_execution(
    llm_client: &dyn LlmClient,
    user_question: &str,
    plan_description: &str,
    commands_executed: &[String],
    execution_output: &str,
) -> Result<PostValidation> {
    // Build validation prompt
    let commands_text = commands_executed
        .iter()
        .enumerate()
        .map(|(i, cmd)| format!("  {}. {}", i + 1, cmd))
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = format!(
        r#"You just executed a plan to address a user's request. Assess whether the execution satisfied their needs.

USER REQUEST: {}

PLAN INTENT: {}

COMMANDS EXECUTED:
{}

EXECUTION OUTPUT:
{}

Analyze the results and respond with JSON containing:
- satisfaction_score: A number from 0.0 to 1.0 indicating how well the request was satisfied (1.0 = fully satisfied)
- summary: A 1-3 sentence assessment of what was accomplished
- residual_concerns: Array of any remaining issues or concerns (empty array if none)
- suggested_checks: Array of up to 3 verification commands the user could run (empty array if none needed)

Be honest and precise. If something went wrong or is incomplete, reflect that in the score and concerns.

Example response:
{{
  "satisfaction_score": 0.95,
  "summary": "Successfully configured vim with 4-space tabs. The ~/.vimrc file has been updated and changes are active.",
  "residual_concerns": ["Old backup file remains in home directory"],
  "suggested_checks": ["vim --version", "cat ~/.vimrc | grep tabstop"]
}}

Respond ONLY with valid JSON.
"#,
        user_question, plan_description, commands_text, execution_output
    );

    // Define JSON schema for structured response
    let schema = json!({
        "type": "object",
        "properties": {
            "satisfaction_score": {
                "type": "number",
                "minimum": 0.0,
                "maximum": 1.0,
                "description": "How well the request was satisfied (0.0-1.0)"
            },
            "summary": {
                "type": "string",
                "description": "1-3 sentence assessment of what was accomplished"
            },
            "residual_concerns": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Any remaining issues or concerns"
            },
            "suggested_checks": {
                "type": "array",
                "items": {"type": "string"},
                "maxItems": 3,
                "description": "Up to 3 verification commands"
            }
        },
        "required": ["satisfaction_score", "summary", "residual_concerns", "suggested_checks"]
    });

    // Query LLM with schema
    let schema_str = serde_json::to_string_pretty(&schema)
        .context("Failed to serialize schema")?;

    let response = llm_client
        .call_json(
            "You are a helpful assistant that evaluates system execution results.",
            &prompt,
            &schema_str,
        )
        .map_err(|e| anyhow::anyhow!("LLM call failed: {:?}", e))?;

    // Parse response (response is already a Value, convert to PostValidation)
    let validation: PostValidation = serde_json::from_value(response)
        .context("Failed to parse post-validation response")?;

    // Validate satisfaction score is in range
    if !(0.0..=1.0).contains(&validation.satisfaction_score) {
        anyhow::bail!(
            "Invalid satisfaction score: {}",
            validation.satisfaction_score
        );
    }

    Ok(validation)
}

/// Build a concise execution summary for post-validation
///
/// This captures the key information needed for validation without
/// overwhelming the LLM with too much output.
pub fn summarize_execution_output(full_output: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = full_output.lines().collect();

    if lines.len() <= max_lines {
        return full_output.to_string();
    }

    // Take first few and last few lines
    let head_lines = max_lines / 2;
    let tail_lines = max_lines - head_lines;

    let mut summary = String::new();
    for line in lines.iter().take(head_lines) {
        summary.push_str(line);
        summary.push('\n');
    }

    summary.push_str(&format!(
        "\n... [{} lines omitted] ...\n\n",
        lines.len() - max_lines
    ));

    for line in lines.iter().skip(lines.len() - tail_lines) {
        summary.push_str(line);
        summary.push('\n');
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summarize_short_output() {
        let output = "Line 1\nLine 2\nLine 3";
        let summary = summarize_execution_output(output, 10);
        assert_eq!(summary, output);
    }

    #[test]
    fn test_summarize_long_output() {
        let output = (1..=100).map(|i| format!("Line {}", i)).collect::<Vec<_>>().join("\n");
        let summary = summarize_execution_output(&output, 10);

        assert!(summary.contains("Line 1"));
        assert!(summary.contains("Line 100"));
        assert!(summary.contains("omitted"));
    }

    #[test]
    fn test_satisfaction_score_validation() {
        // Valid score
        let valid = PostValidation {
            satisfaction_score: 0.85,
            summary: "Good".to_string(),
            residual_concerns: vec![],
            suggested_checks: vec![],
        };
        assert!(valid.satisfaction_score >= 0.0 && valid.satisfaction_score <= 1.0);

        // Would be invalid if we constructed it
        // (validation happens in validate_execution function)
    }
}
