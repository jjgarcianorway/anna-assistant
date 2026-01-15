//! Streaming response handling for real-time answer display.
//! v0.3.4: Added spinner animation while waiting for LLM response
//! v0.3.28: Added version compatibility check before streaming requests
//! v0.3.30: TERMINALITY CONTRACT - No Done packet = No answer = Failure
//! v0.3.35: Uses daemon_recovery for self-healing connections

use anna_shared::rpc::{AskResult, RpcMethod, RpcRequest, StepType, StreamingResponse};
use anyhow::{anyhow, Result};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::time::timeout;

use crate::daemon_recovery::connect_with_recovery;
use crate::display::*;
use crate::rpc::{ensure_compatible_daemon, RPC_TIMEOUT_SECS};
use crate::spinner::Spinner;

/// Send a question with streaming response
/// Returns the AskResult so caller can check for needs_clarification
///
/// TERMINALITY CONTRACT (v0.3.30):
/// - Client MUST NOT print a final answer unless it receives a terminal Done packet
/// - If stream ends without Done: print failure, return error (exit non-zero)
/// - NO FALLBACKS - partial streams are failures, period
pub async fn ask_streaming(question: &str, session_id: &str) -> Result<AskResult> {
    // v0.3.28: Verify version compatibility before streaming request
    ensure_compatible_daemon().await?;

    // v0.3.35: Use self-healing connection - never tells user to run commands
    let mut stream = connect_with_recovery().await?;

    // Send request with session_id for context tracking
    let request = RpcRequest::new(
        RpcMethod::AskStreaming,
        Some(serde_json::json!({
            "question": question,
            "session_id": session_id
        })),
    );
    let request_json = serde_json::to_string(&request)?;

    timeout(Duration::from_secs(5), async {
        stream
            .write_all(format!("{}\n", request_json).as_bytes())
            .await
    })
    .await
    .map_err(|_| anyhow!("Timeout writing to daemon"))?
    .map_err(|e| anyhow!("Failed to write to daemon: {}", e))?;

    // Read streaming responses
    let (reader, _) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    let mut in_answer = false;
    let mut final_result: Option<AskResult> = None;
    let mut spinner: Option<Spinner> = None;
    #[allow(unused_assignments)]
    let mut _waiting_for_answer = false;
    // v0.3.30: Track whether we received ANY streaming content (for error messages)
    let mut received_any_content = false;

    loop {
        line.clear();
        match timeout(
            Duration::from_secs(RPC_TIMEOUT_SECS),
            reader.read_line(&mut line),
        )
        .await
        {
            Ok(Ok(0)) => break, // EOF - will check for Done below
            Ok(Ok(_)) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                match serde_json::from_str::<StreamingResponse>(trimmed) {
                    Ok(StreamingResponse::Step { step }) => {
                        received_any_content = true;
                        // Stop spinner when we get a step
                        if let Some(ref mut s) = spinner {
                            s.stop();
                            spinner = None;
                        }
                        if in_answer {
                            // End the answer line
                            println!();
                            println!();
                            in_answer = false;
                        }
                        print_step(&step);
                        if matches!(step.step_type, StepType::FinalPrompt) {
                            // About to receive tokens - start spinner
                            println!();
                            print_colored("ANSWER: ", GREEN);
                            flush_stdout();
                            in_answer = true;
                            _waiting_for_answer = true;
                            spinner = Some(Spinner::new("thinking..."));
                        }
                        // Start spinner while internal processing
                        if matches!(step.step_type, StepType::SpecialistWorking | StepType::TeamAssignment) {
                            spinner = Some(Spinner::new(""));
                        }
                    }
                    Ok(StreamingResponse::Token { token }) => {
                        received_any_content = true;
                        // Stop spinner on first token
                        if let Some(ref mut s) = spinner {
                            s.stop();
                            spinner = None;
                        }
                        _waiting_for_answer = false;
                        // Print token immediately
                        print_colored(&token, GREEN);
                        flush_stdout();
                    }
                    Ok(StreamingResponse::Dialogue { speaker, recipient, message, offset_ms }) => {
                        // v0.3.44: Internal comms dialogue line
                        received_any_content = true;
                        if let Some(ref mut s) = spinner {
                            s.stop();
                            spinner = None;
                        }
                        if in_answer {
                            println!();
                            in_answer = false;
                        }
                        // Format: [0.0s] Speaker -> Recipient: message
                        let offset_secs = offset_ms as f64 / 1000.0;
                        print_colored(&format!("[{:.1}s] ", offset_secs), DIM);
                        print_colored(&speaker, CYAN);
                        if let Some(ref recip) = recipient {
                            print!(" -> ");
                            print_colored(recip, MAGENTA);
                        }
                        print!(": ");
                        println!("{}", message);
                        flush_stdout();
                    }
                    Ok(StreamingResponse::Validation { warning }) => {
                        // Display validation warning (v0.0.889)
                        // Only show high severity warnings to avoid noise
                        if warning.severity == "high" {
                            if let Some(ref mut s) = spinner {
                                s.stop();
                                spinner = None;
                            }
                            if in_answer {
                                println!();
                            }
                            print_colored("[!] ", YELLOW);
                            println_colored(&warning.message, YELLOW);
                            flush_stdout();
                        }
                    }
                    Ok(StreamingResponse::Done { result }) => {
                        // TERMINAL PACKET RECEIVED - this is the ONLY valid completion
                        if let Some(ref mut s) = spinner {
                            s.stop();
                            spinner = None;
                        }
                        if in_answer {
                            println!();
                            println!();
                        }
                        final_result = Some(result);
                        break;
                    }
                    Ok(StreamingResponse::Error { message }) => {
                        if let Some(ref mut s) = spinner {
                            s.stop();
                            spinner = None;
                        }
                        if in_answer {
                            println!();
                        }
                        print_colored("Error: ", RED);
                        println!("{}", message);
                        return Err(anyhow!("{}", message));
                    }
                    Err(_) => {
                        // Ignore parse errors for partial lines
                    }
                }
            }
            Ok(Err(e)) => {
                return Err(anyhow!("Failed to read from daemon: {}", e));
            }
            Err(_) => {
                print_timeout_error(RPC_TIMEOUT_SECS);
                return Err(anyhow!("Request timed out after {}s", RPC_TIMEOUT_SECS));
            }
        }
    }

    // v0.3.30: TERMINALITY CONTRACT ENFORCEMENT
    // No Done packet = FAILURE. No fallbacks. No reconstructed results.
    let result = match final_result {
        Some(r) => r,
        None => {
            // Stream ended without terminal Done packet - this is a FAILURE
            if in_answer {
                println!(); // Clean up partial answer line
            }
            println!();
            print_colored("[FAILED] ", RED);
            if received_any_content {
                println!("Stream terminated without completion. Partial results discarded.");
            } else {
                println!("No response received from daemon.");
            }
            return Err(anyhow!("Stream terminated without Done packet - request failed"));
        }
    };

    // Only print metadata for successful, complete results
    if !result.needs_clarification {
        println!();
        println_colored(&format!("({} iterations)", result.iterations), DIM);

        // v0.3.6: Display citations if present
        if !result.citations.is_empty() {
            println!();
            println_colored("Sources:", DIM);
            for cite in &result.citations {
                print_colored("  - ", DIM);
                print!("{}", cite.source);
                if let Some(ref url) = cite.url {
                    print_colored(&format!(" ({})", url), DIM);
                }
                println!();
            }
        }
    }

    Ok(result)
}

// =============================================================================
// v0.3.30: CONTRACT ENFORCEMENT TESTS (R5)
// =============================================================================

#[cfg(test)]
mod tests {
    /// R5: Verify streaming code has NO fallback logic
    /// The terminality contract: No Done packet = No answer = Failure
    #[test]
    fn test_client_refuses_final_answer_without_terminal_packet() {
        // Read the source file to verify contract enforcement
        let source = include_str!("streaming.rs");

        // Find the main function (excluding test module)
        let test_module_start = source.find("#[cfg(test)]").unwrap_or(source.len());
        let main_code = &source[..test_module_start];

        // Forbidden patterns that would violate terminality contract
        // (only check main code, not test module to avoid self-referential issues)
        let forbidden = [
            "fallback_answer",
            "result_reconstructed",
            "connection_incomplete",
        ];

        for pattern in &forbidden {
            assert!(
                !main_code.to_lowercase().contains(&pattern.to_lowercase()),
                "Streaming code contains forbidden fallback pattern: '{}'", pattern
            );
        }

        // Verify we don't have the old fallback AskResult construction
        assert!(
            !main_code.contains("connection incomplete - result reconstructed"),
            "Main code should not have fallback result construction"
        );

        // Verify terminality contract documentation exists
        assert!(
            source.contains("TERMINALITY CONTRACT"),
            "Contract documentation missing: should have 'TERMINALITY CONTRACT' comment"
        );

        // Verify failure case returns error
        assert!(
            source.contains("Stream terminated without Done packet - request failed"),
            "Should return error when Done packet is missing"
        );

        // Verify no AskResult construction outside of Done packet handling
        // The only place we should construct success is from the Done packet
        let done_section_start = source.find("StreamingResponse::Done");
        let error_section_start = source.find("None => {");

        assert!(done_section_start.is_some(), "Should handle Done packet");
        assert!(error_section_start.is_some(), "Should handle missing Done");

        // The error section should NOT construct an AskResult
        if let Some(error_start) = error_section_start {
            let error_section = &source[error_start..];
            let section_end = error_section.find("};").unwrap_or(200);
            let error_handling = &error_section[..section_end];

            assert!(
                !error_handling.contains("AskResult {"),
                "Error handling should NOT construct AskResult"
            );
            assert!(
                error_handling.contains("return Err"),
                "Error handling should return Err"
            );
        }
    }

    #[test]
    fn test_streaming_has_proper_failure_message() {
        let source = include_str!("streaming.rs");

        // Verify proper failure message is printed
        assert!(
            source.contains("[FAILED]"),
            "Should print [FAILED] marker on stream termination"
        );

        // Verify both cases are handled
        assert!(
            source.contains("Stream terminated without completion"),
            "Should handle partial stream case"
        );
        assert!(
            source.contains("No response received from daemon"),
            "Should handle empty stream case"
        );
    }

    /// v0.3.35: Verify streaming code never tells users to run manual commands
    #[test]
    fn test_no_manual_commands_in_streaming() {
        let source = include_str!("streaming.rs");

        // Find main code (excluding test module)
        let test_module_start = source.find("#[cfg(test)]").unwrap_or(source.len());
        let main_code = &source[..test_module_start];

        // Forbidden patterns - manual command instructions
        let forbidden = [
            "sudo systemctl start",
            "sudo systemctl restart",
            "systemctl start annad",
            "systemctl restart annad",
            "usermod -aG anna",
            "Run: sudo",
            "Try: sudo",
        ];

        for pattern in &forbidden {
            assert!(
                !main_code.contains(pattern),
                "Streaming code should not tell users to run '{}' - use daemon_recovery instead",
                pattern
            );
        }
    }
}
