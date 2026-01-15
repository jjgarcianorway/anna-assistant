//! Fake daemon mode for golden transcript testing.
//! When ANNA_FAKE_DAEMON=1, reads fixtures instead of connecting to real daemon.
//!
//! Fixture format (line-based JSON):
//! - Each line is a StreamingResponse JSON object
//! - Fixture path set via ANNA_FAKE_FIXTURE env var

use anna_shared::rpc::{AskResult, Citation, DialogueStep, StepType, StreamingResponse};
use anyhow::{anyhow, Result};
use std::env;
use std::fs;

/// Check if fake daemon mode is enabled
pub fn is_fake_daemon() -> bool {
    env::var("ANNA_FAKE_DAEMON").map(|v| v == "1").unwrap_or(false)
}

/// Get fixture path from env
fn get_fixture_path() -> Result<String> {
    env::var("ANNA_FAKE_FIXTURE")
        .map_err(|_| anyhow!("ANNA_FAKE_FIXTURE not set"))
}

/// Load fixture file and return streaming responses
pub fn load_fixture() -> Result<Vec<StreamingResponse>> {
    let path = get_fixture_path()?;
    let content = fs::read_to_string(&path)
        .map_err(|e| anyhow!("Failed to read fixture {}: {}", path, e))?;

    let mut responses = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let response: StreamingResponse = serde_json::from_str(line)
            .map_err(|e| anyhow!("Failed to parse fixture line: {} - {}", line, e))?;
        responses.push(response);
    }
    Ok(responses)
}

/// Create a simple factual answer fixture (T1)
pub fn fixture_simple_answer() -> Vec<StreamingResponse> {
    vec![
        StreamingResponse::Step {
            step: DialogueStep {
                step_type: StepType::TeamAssignment,
                content: "Kevin (Storage) is handling this.".to_string(),
            },
        },
        StreamingResponse::Step {
            step: DialogueStep {
                step_type: StepType::FinalPrompt,
                content: "".to_string(),
            },
        },
        StreamingResponse::Token { token: "Your".to_string() },
        StreamingResponse::Token { token: " disk".to_string() },
        StreamingResponse::Token { token: " has".to_string() },
        StreamingResponse::Token { token: " 234GB".to_string() },
        StreamingResponse::Token { token: " free".to_string() },
        StreamingResponse::Token { token: ".".to_string() },
        StreamingResponse::Done {
            result: AskResult {
                answer: "Your disk has 234GB free.".to_string(),
                success: true,
                iterations: 1,
                commands_executed: vec!["df -h".to_string()],
                dialogue: vec![],
                needs_clarification: false,
                clarification_question: None,
                cached: false,
                citations: vec![Citation {
                    source: "df -h".to_string(),
                    url: None,
                    section: None,
                }],
            },
        },
    ]
}

/// Create confirmation flow fixture (T2)
pub fn fixture_confirmation_flow() -> Vec<StreamingResponse> {
    vec![
        StreamingResponse::Step {
            step: DialogueStep {
                step_type: StepType::TeamAssignment,
                content: "Marcus (Desktop) is handling this.".to_string(),
            },
        },
        StreamingResponse::Step {
            step: DialogueStep {
                step_type: StepType::ConfirmationRequest,
                content: "I will configure GDM resolution.\n\nSteps:\n  1. Create config file\n  2. Set resolution\n  3. Restart GDM".to_string(),
            },
        },
        StreamingResponse::Done {
            result: AskResult {
                answer: "I will configure GDM resolution.".to_string(),
                success: true,
                iterations: 0,
                commands_executed: vec![],
                dialogue: vec![],
                needs_clarification: true,
                clarification_question: Some("Proceed? (yes/no)".to_string()),
                cached: false,
                citations: vec![],
            },
        },
    ]
}

/// Create failure fixture (T3)
pub fn fixture_timeout() -> Vec<StreamingResponse> {
    vec![
        StreamingResponse::Step {
            step: DialogueStep {
                step_type: StepType::TeamAssignment,
                content: "Marcus (Performance) is handling this.".to_string(),
            },
        },
        // Stream ends without Done packet - simulates timeout/interruption
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_simple_answer_has_done() {
        let responses = fixture_simple_answer();
        assert!(matches!(responses.last(), Some(StreamingResponse::Done { .. })));
    }

    #[test]
    fn test_fixture_confirmation_has_clarification() {
        let responses = fixture_confirmation_flow();
        if let Some(StreamingResponse::Done { result }) = responses.last() {
            assert!(result.needs_clarification);
            assert_eq!(result.clarification_question, Some("Proceed? (yes/no)".to_string()));
        } else {
            panic!("Expected Done response");
        }
    }

    #[test]
    fn test_fixture_timeout_no_done() {
        let responses = fixture_timeout();
        assert!(!matches!(responses.last(), Some(StreamingResponse::Done { .. })));
    }
}
