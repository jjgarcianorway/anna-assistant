//! Streaming response handling for real-time answer display.

use anna_shared::rpc::{AskResult, RpcMethod, RpcRequest, StepType, StreamingResponse};
use anna_shared::socket_path;
use anyhow::{anyhow, Result};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::timeout;

use crate::display::*;
use crate::rpc::RPC_TIMEOUT_SECS;

/// Send a question with streaming response
/// Returns the AskResult so caller can check for needs_clarification
pub async fn ask_streaming(question: &str, session_id: &str) -> Result<AskResult> {
    let socket_file = socket_path();
    let socket_path = std::path::Path::new(&socket_file);

    if !socket_path.exists() {
        return Err(anyhow!(
            "Anna daemon not running.\n\
             The socket at {} does not exist.\n\n\
             Start the daemon with: sudo systemctl start annad",
            socket_file
        ));
    }

    let mut stream = UnixStream::connect(socket_path).await.map_err(|e| {
        anyhow!(
            "Cannot connect to Anna daemon: {}\n\n\
             Try: sudo systemctl restart annad",
            e
        )
    })?;

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

    loop {
        line.clear();
        match timeout(
            Duration::from_secs(RPC_TIMEOUT_SECS),
            reader.read_line(&mut line),
        )
        .await
        {
            Ok(Ok(0)) => break, // EOF
            Ok(Ok(_)) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                match serde_json::from_str::<StreamingResponse>(trimmed) {
                    Ok(StreamingResponse::Step { step }) => {
                        if in_answer {
                            // End the answer line
                            println!();
                            println!();
                            in_answer = false;
                        }
                        print_step(&step);
                        if matches!(step.step_type, StepType::FinalPrompt) {
                            // About to receive tokens
                            println!();
                            print_colored("ANSWER: ", GREEN);
                            flush_stdout();
                            in_answer = true;
                        }
                    }
                    Ok(StreamingResponse::Token { token }) => {
                        // Print token immediately
                        print_colored(&token, GREEN);
                        flush_stdout();
                    }
                    Ok(StreamingResponse::Validation { warning }) => {
                        // Display validation warning (v0.0.889)
                        // Only show high severity warnings to avoid noise
                        if warning.severity == "high" {
                            if in_answer {
                                println!();
                            }
                            print_colored("⚠ ", YELLOW);
                            println_colored(&warning.message, YELLOW);
                            flush_stdout();
                        }
                    }
                    Ok(StreamingResponse::Done { result }) => {
                        if in_answer {
                            println!();
                            println!();
                        }
                        final_result = Some(result);
                        break;
                    }
                    Ok(StreamingResponse::Error { message }) => {
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

    let result = final_result.ok_or_else(|| anyhow!("No result received from daemon"))?;

    // Only print iterations if not asking for clarification
    if !result.needs_clarification {
        println!();
        println_colored(&format!("({} iterations)", result.iterations), DIM);
    }

    Ok(result)
}
