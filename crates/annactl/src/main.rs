//! Anna CLI - simple REPL interface to ask questions about Arch Linux.

use anna_shared::rpc::{AskResult, RpcMethod, RpcRequest, RpcResponse, StepType, StreamingResponse};
use anna_shared::socket_path;
use anna_shared::status::DaemonStatus;
use anyhow::{anyhow, Result};
use std::io::{self, Write};
use std::path::Path;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::timeout;

const RPC_TIMEOUT_SECS: u64 = 120; // 2 minutes for LLM operations

/// Connect to the daemon
async fn connect() -> Result<UnixStream> {
    let socket_file = socket_path();
    let socket_path = Path::new(&socket_file);

    if !socket_path.exists() {
        return Err(anyhow!(
            "Anna daemon not running.\n\
             The socket at {} does not exist.\n\n\
             Start the daemon with: sudo systemctl start annad",
            socket_file
        ));
    }

    UnixStream::connect(socket_path).await.map_err(|e| {
        anyhow!(
            "Cannot connect to Anna daemon: {}\n\n\
             Try: sudo systemctl restart annad",
            e
        )
    })
}

/// Send an RPC request and get the response
async fn call(method: RpcMethod, params: Option<serde_json::Value>) -> Result<RpcResponse> {
    let mut stream = connect().await?;
    let request = RpcRequest::new(method, params);
    let request_json = serde_json::to_string(&request)?;

    // Send request
    timeout(Duration::from_secs(5), async {
        stream
            .write_all(format!("{}\n", request_json).as_bytes())
            .await
    })
    .await
    .map_err(|_| anyhow!("Timeout writing to daemon"))?
    .map_err(|e| anyhow!("Failed to write to daemon: {}", e))?;

    // Read response
    let (reader, _) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    timeout(Duration::from_secs(RPC_TIMEOUT_SECS), reader.read_line(&mut line))
        .await
        .map_err(|_| anyhow!("Request timed out after {}s", RPC_TIMEOUT_SECS))?
        .map_err(|e| anyhow!("Failed to read from daemon: {}", e))?;

    serde_json::from_str(&line).map_err(|e| anyhow!("Invalid response: {}", e))
}

/// Get daemon status
async fn get_status() -> Result<DaemonStatus> {
    let response = call(RpcMethod::Status, None).await?;
    if let Some(error) = response.error {
        return Err(anyhow!("Status error: {}", error.message));
    }
    let result = response.result.ok_or_else(|| anyhow!("No result"))?;
    serde_json::from_value(result).map_err(|e| anyhow!("Parse error: {}", e))
}

/// Send a question and get the answer (non-streaming)
async fn ask(question: &str) -> Result<AskResult> {
    let params = serde_json::json!({ "question": question });
    let response = call(RpcMethod::Ask, Some(params)).await?;

    if let Some(error) = response.error {
        return Err(anyhow!("{}", error.message));
    }

    let result = response.result.ok_or_else(|| anyhow!("No result"))?;
    serde_json::from_value(result).map_err(|e| anyhow!("Parse error: {}", e))
}

/// Send a question with streaming response
/// Returns the AskResult so caller can check for needs_clarification
async fn ask_streaming(question: &str, session_id: &str) -> Result<AskResult> {
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
    let request = RpcRequest::new(RpcMethod::AskStreaming, Some(serde_json::json!({
        "question": question,
        "session_id": session_id
    })));
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
        match timeout(Duration::from_secs(RPC_TIMEOUT_SECS), reader.read_line(&mut line)).await {
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
                            println_colored("═══════════════════════════════════════", DIM);
                            in_answer = false;
                        }
                        print_step(&step);
                        if matches!(step.step_type, StepType::FinalPrompt) {
                            // About to receive tokens
                            println_colored("═══════════════════════════════════════", DIM);
                            print_colored("ANSWER: ", GREEN);
                            io::stdout().flush().ok();
                            in_answer = true;
                        }
                    }
                    Ok(StreamingResponse::Token { token }) => {
                        // Print token immediately
                        print_colored(&token, GREEN);
                        io::stdout().flush().ok();
                    }
                    Ok(StreamingResponse::Done { result }) => {
                        if in_answer {
                            println!();
                            println_colored("═══════════════════════════════════════", DIM);
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
                println!();
                println_colored("╔══════════════════════════════════════════════╗", RED);
                println_colored("║           REQUEST TIMED OUT                  ║", RED);
                println_colored("╚══════════════════════════════════════════════╝", RED);
                println!();
                println!("  The request took longer than {}s to complete.", RPC_TIMEOUT_SECS);
                println!();
                println_colored("  Possible causes:", YELLOW);
                println!("  • Ollama model is loading (first query is slow)");
                println!("  • Complex question requiring many iterations");
                println!("  • LLM server is overloaded");
                println!();
                println_colored("  Suggestions:", GREEN);
                println!("  • Try again - model may be loaded now");
                println!("  • Check: annactl status");
                println!("  • Increase timeout in ~/.anna/config.toml:");
                println!("    [performance]");
                println!("    llm_timeout_secs = 180");
                println!();
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

/// Print a single dialogue step
fn print_step(step: &anna_shared::rpc::DialogueStep) {
    match step.step_type {
        StepType::UserQuestion => {
            print_colored("USER: ", CYAN);
            println!("{}", step.content);
            println!();
        }
        StepType::AnnaToLlm => {
            print_colored("ANNA → LLM: ", YELLOW);
            println!("(command selection prompt)");
            println_colored("┌─────────────────────────────────────────", DIM);
            for line in step.content.lines() {
                println_colored(&format!("│ {}", line), DIM);
            }
            println_colored("└─────────────────────────────────────────", DIM);
            println!();
        }
        StepType::LlmCommands => {
            print_colored("LLM → ANNA: ", YELLOW);
            if step.content == "NONE" || step.content == "DONE" {
                println_colored(&step.content, DIM);
            } else {
                println!("commands to run:");
                for line in step.content.lines() {
                    let line = line.trim();
                    if !line.is_empty() {
                        print_colored("  $ ", DIM);
                        println_colored(line, CYAN);
                    }
                }
            }
            println!();
        }
        StepType::CommandExec => {
            print_colored("EXEC: ", GREEN);
            println!("{}", step.content);
        }
        StepType::CommandOutput => {
            print_colored("OUTPUT: ", DIM);
            println!("{}", step.content);
            println!();
        }
        StepType::ValidationPrompt => {
            print_colored("ANNA → LLM: ", YELLOW);
            println!("(validation prompt)");
            println_colored("┌─────────────────────────────────────────", DIM);
            for line in step.content.lines() {
                println_colored(&format!("│ {}", line), DIM);
            }
            println_colored("└─────────────────────────────────────────", DIM);
            println!();
        }
        StepType::ValidationResponse => {
            print_colored("LLM → ANNA: ", YELLOW);
            println!("{}", step.content);
            println!();
        }
        StepType::FinalPrompt => {
            print_colored("ANNA → LLM: ", YELLOW);
            println!("(final answer prompt)");
            println_colored("┌─────────────────────────────────────────", DIM);
            for line in step.content.lines() {
                println_colored(&format!("│ {}", line), DIM);
            }
            println_colored("└─────────────────────────────────────────", DIM);
            println!();
        }
        StepType::FinalAnswer => {
            // This step comes after streaming, so we don't print it again
        }
        StepType::WikiSearch => {
            print_colored("ANNA → WIKI: ", MAGENTA);
            println!("searching Arch Wiki...");
            println_colored(&format!("  query: {}", step.content), DIM);
            println!();
        }
        StepType::WikiResults => {
            print_colored("WIKI → ANNA: ", MAGENTA);
            println!("found articles:");
            for line in step.content.lines() {
                println_colored(&format!("  • {}", line), DIM);
            }
            println!();
        }
        StepType::WikiCommands => {
            print_colored("WIKI: ", MAGENTA);
            println!("extracted commands:");
            for line in step.content.lines() {
                let line = line.trim();
                if !line.is_empty() {
                    print_colored("  $ ", DIM);
                    println_colored(line, CYAN);
                }
            }
            println!();
        }
        StepType::ClarificationQuestion => {
            print_colored("ANNA → USER: ", YELLOW);
            println!("{}", step.content);
            println!();
        }
        StepType::ClarificationResponse => {
            print_colored("USER → ANNA: ", CYAN);
            println!("{}", step.content);
            println!();
        }
        StepType::IntentClassifying => {
            print_colored("ANNA: ", BLUE);
            println!("understanding question...");
        }
        StepType::IntentResult => {
            print_colored("  intent: ", DIM);
            println!("{}", step.content);
        }
        StepType::SubQuestion => {
            println!();
            print_colored("─── ", DIM);
            print_colored(&step.content, YELLOW);
            println!();
        }
        StepType::SubQuestionResult => {
            print_colored("  → ", GREEN);
            println!("{}", step.content);
        }
        StepType::UnderstandingCheck => {
            print_colored("ANNA: ", CYAN);
            println!("{}", step.content);
        }
        StepType::ConfirmationRequest => {
            println!();
            print_colored("ANNA → USER: ", YELLOW);
            println!("Please confirm:");
            for line in step.content.lines() {
                println!("  {}", line);
            }
            println!();
        }
        StepType::MissingInfo => {
            print_colored("ANNA: ", RED);
            println!("Missing information:");
            for line in step.content.lines() {
                println!("  - {}", line);
            }
        }
        StepType::SystemAlert => {
            println!();
            println_colored("╔══════════════════════════════════════════════╗", YELLOW);
            println_colored("║           SYSTEM ALERT                       ║", YELLOW);
            println_colored("╚══════════════════════════════════════════════╝", YELLOW);
            for line in step.content.lines() {
                print_colored("  ", YELLOW);
                println!("{}", line);
            }
            println!();
        }
    }
}

/// Print colored text
fn print_colored(text: &str, color: &str) {
    print!("{}{}\x1b[0m", color, text);
}

fn println_colored(text: &str, color: &str) {
    println!("{}{}\x1b[0m", color, text);
}

const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";
const BLUE: &str = "\x1b[34m";
const CYAN: &str = "\x1b[36m";
const MAGENTA: &str = "\x1b[35m";
const DIM: &str = "\x1b[2m";
const BOLD: &str = "\x1b[1m";

/// Print the greeting
fn print_greeting() {
    println!();
    println_colored("Anna - Arch Linux Assistant", BOLD);
    println_colored("Ask questions about your system in plain English.", DIM);
    println_colored("Type 'quit' or Ctrl-D to exit.", DIM);
    println!();
}

/// Print status
async fn print_status() {
    match get_status().await {
        Ok(status) => {
            let state_color = match status.state {
                anna_shared::status::DaemonState::Ready => GREEN,
                anna_shared::status::DaemonState::Starting => YELLOW,
                anna_shared::status::DaemonState::Error => RED,
            };
            print!("Status: ");
            println_colored(&status.state.to_string(), state_color);
            println!("Version: {}", status.version);
            print!("Ollama: ");
            if status.ollama_running {
                println_colored("running", GREEN);
            } else {
                println_colored("not running", RED);
            }
            if let Some(model) = &status.model {
                println!("Model: {}", model);
            }
            if let Some(gpu) = &status.gpu {
                print!("GPU: ");
                println_colored(gpu, CYAN);
                if let Some(vram) = status.vram_mb {
                    println!("VRAM: {} MB", vram);
                }
            }
        }
        Err(e) => {
            print_colored("Error: ", RED);
            println!("{}", e);
        }
    }
}

/// Handle a question with clarification loop
async fn handle_question(question: &str) {
    // Generate a one-time session ID for command-line mode
    let session_id = uuid::Uuid::new_v4().to_string();
    handle_question_with_clarification(question, false, &session_id).await;
}

/// Handle a question, with optional clarification support
/// When in_repl is true, can prompt user for clarification
async fn handle_question_with_clarification(question: &str, in_repl: bool, session_id: &str) {
    // Clear line and start streaming
    println!();

    let mut current_question = question.to_string();
    let max_clarifications = 3; // Prevent infinite loops
    let mut clarification_count = 0;

    loop {
        match ask_streaming(&current_question, session_id).await {
            Ok(result) => {
                if result.needs_clarification && in_repl && clarification_count < max_clarifications {
                    // Display clarification question and prompt user
                    println!();
                    if let Some(ref clarification_q) = result.clarification_question {
                        print_colored("ANNA needs clarification: ", YELLOW);
                        println!("{}", clarification_q);
                    }
                    print_colored("> ", CYAN);
                    io::stdout().flush().ok();

                    // Read user's clarification response
                    let mut response = String::new();
                    if io::stdin().read_line(&mut response).is_ok() {
                        let response = response.trim();
                        if !response.is_empty() && response.to_lowercase() != "cancel" {
                            // Append clarification to original question
                            current_question = format!("{} (Context: {})", question, response);
                            clarification_count += 1;
                            println!();
                            continue; // Re-submit with clarification
                        }
                    }
                    // User cancelled or empty response
                    println_colored("Clarification cancelled.", DIM);
                } else if result.needs_clarification && !in_repl {
                    // Non-REPL mode: just show the clarification question
                    println!();
                    print_colored("Note: ", YELLOW);
                    println!("This question may need more context. Try running in interactive mode (annactl without arguments).");
                }
                // Done
                break;
            }
            Err(e) => {
                print_colored("Error: ", RED);
                println!("{}", e);
                break;
            }
        }
    }
}

/// Print the full dialogue for transparency
fn print_dialogue(result: &AskResult) {
    for step in &result.dialogue {
        match step.step_type {
            StepType::UserQuestion => {
                print_colored("USER: ", CYAN);
                println!("{}", step.content);
                println!();
            }
            StepType::AnnaToLlm => {
                print_colored("ANNA → LLM: ", YELLOW);
                println!("(command selection prompt)");
                println_colored("┌─────────────────────────────────────────", DIM);
                for line in step.content.lines() {
                    println_colored(&format!("│ {}", line), DIM);
                }
                println_colored("└─────────────────────────────────────────", DIM);
                println!();
            }
            StepType::LlmCommands => {
                print_colored("LLM → ANNA: ", YELLOW);
                if step.content == "NONE" || step.content == "DONE" {
                    println_colored(&step.content, DIM);
                } else {
                    println!("commands to run:");
                    for line in step.content.lines() {
                        let line = line.trim();
                        if !line.is_empty() {
                            print_colored("  $ ", DIM);
                            println_colored(line, CYAN);
                        }
                    }
                }
                println!();
            }
            StepType::CommandExec => {
                print_colored("EXEC: ", GREEN);
                println!("{}", step.content);
            }
            StepType::CommandOutput => {
                print_colored("OUTPUT: ", DIM);
                println!("{}", step.content);
                println!();
            }
            StepType::ValidationPrompt => {
                print_colored("ANNA → LLM: ", YELLOW);
                println!("(validation prompt)");
                println_colored("┌─────────────────────────────────────────", DIM);
                for line in step.content.lines() {
                    println_colored(&format!("│ {}", line), DIM);
                }
                println_colored("└─────────────────────────────────────────", DIM);
                println!();
            }
            StepType::ValidationResponse => {
                print_colored("LLM → ANNA: ", YELLOW);
                println!("{}", step.content);
                println!();
            }
            StepType::FinalPrompt => {
                print_colored("ANNA → LLM: ", YELLOW);
                println!("(final answer prompt)");
                println_colored("┌─────────────────────────────────────────", DIM);
                for line in step.content.lines() {
                    println_colored(&format!("│ {}", line), DIM);
                }
                println_colored("└─────────────────────────────────────────", DIM);
                println!();
            }
            StepType::FinalAnswer => {
                println_colored("═══════════════════════════════════════", DIM);
                print_colored("ANSWER: ", GREEN);
                println!();
                println_colored(&step.content, GREEN);
                println_colored("═══════════════════════════════════════", DIM);
            }
            StepType::WikiSearch => {
                print_colored("ANNA → WIKI: ", MAGENTA);
                println!("searching Arch Wiki...");
                println_colored(&format!("  query: {}", step.content), DIM);
                println!();
            }
            StepType::WikiResults => {
                print_colored("WIKI → ANNA: ", MAGENTA);
                println!("found articles:");
                for line in step.content.lines() {
                    println_colored(&format!("  • {}", line), DIM);
                }
                println!();
            }
            StepType::WikiCommands => {
                print_colored("WIKI: ", MAGENTA);
                println!("extracted commands:");
                for line in step.content.lines() {
                    let line = line.trim();
                    if !line.is_empty() {
                        print_colored("  $ ", DIM);
                        println_colored(line, CYAN);
                    }
                }
                println!();
            }
            StepType::ClarificationQuestion => {
                print_colored("ANNA → USER: ", YELLOW);
                println!("{}", step.content);
                println!();
            }
            StepType::ClarificationResponse => {
                print_colored("USER → ANNA: ", CYAN);
                println!("{}", step.content);
                println!();
            }
            StepType::IntentClassifying => {
                print_colored("ANNA: ", BLUE);
                println!("understanding question...");
            }
            StepType::IntentResult => {
                print_colored("  intent: ", DIM);
                println!("{}", step.content);
            }
            StepType::SubQuestion => {
                println!();
                print_colored("─── ", DIM);
                print_colored(&step.content, YELLOW);
                println!();
            }
            StepType::SubQuestionResult => {
                print_colored("  → ", GREEN);
                println!("{}", step.content);
            }
            StepType::UnderstandingCheck => {
                print_colored("ANNA: ", CYAN);
                println!("{}", step.content);
            }
            StepType::ConfirmationRequest => {
                println!();
                print_colored("ANNA → USER: ", YELLOW);
                println!("Please confirm:");
                for line in step.content.lines() {
                    println!("  {}", line);
                }
                println!();
            }
            StepType::MissingInfo => {
                print_colored("ANNA: ", RED);
                println!("Missing information:");
                for line in step.content.lines() {
                    println!("  - {}", line);
                }
            }
            StepType::SystemAlert => {
                println!();
                println_colored("╔══════════════════════════════════════════════╗", YELLOW);
                println_colored("║           SYSTEM ALERT                       ║", YELLOW);
                println_colored("╚══════════════════════════════════════════════╝", YELLOW);
                for line in step.content.lines() {
                    print_colored("  ", YELLOW);
                    println!("{}", line);
                }
                println!();
            }
        }
    }
}

/// Run the REPL
async fn run_repl() -> Result<()> {
    print_greeting();
    print_status().await;
    println!();

    // Generate a session_id that persists for this REPL session
    // This enables context tracking across questions ("it", "that service", etc.)
    let session_id = uuid::Uuid::new_v4().to_string();

    let username = std::env::var("USER").unwrap_or_else(|_| "you".to_string());
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);

    loop {
        print_colored(&format!("{}: ", username), CYAN);
        io::stdout().flush()?;

        let mut input = String::new();
        match reader.read_line(&mut input).await {
            Ok(0) => {
                // EOF (Ctrl-D)
                println!();
                println_colored("Goodbye!", DIM);
                break;
            }
            Ok(_) => {
                let input = input.trim();
                if input.is_empty() {
                    continue;
                }

                match input.to_lowercase().as_str() {
                    "quit" | "exit" | "q" | ":q" => {
                        println_colored("Goodbye!", DIM);
                        break;
                    }
                    "status" => {
                        print_status().await;
                    }
                    "help" => {
                        println!("Just ask questions about your Arch Linux system!");
                        println!("Examples:");
                        println!("  What's my disk usage?");
                        println!("  How do I install neovim?");
                        println!("  Show failed services");
                        println!();
                        println!("Commands: status, help, quit");
                    }
                    _ => {
                        handle_question_with_clarification(input, true, &session_id).await;
                    }
                }
                println!();
            }
            Err(e) => {
                print_colored("Input error: ", RED);
                println!("{}", e);
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        let cmd = args[1..].join(" ");

        // Handle built-in commands
        match cmd.to_lowercase().as_str() {
            "status" => {
                print_status().await;
            }
            "help" | "--help" | "-h" => {
                println!("Anna - Arch Linux Assistant");
                println!();
                println!("Usage:");
                println!("  annactl                  Start interactive REPL");
                println!("  annactl status           Show daemon status");
                println!("  annactl <question>       Ask a question");
                println!();
                println!("Examples:");
                println!("  annactl \"what's my disk usage?\"");
                println!("  annactl how do I install neovim");
            }
            "--version" | "-v" => {
                println!("annactl {}", anna_shared::VERSION);
            }
            _ => {
                // It's a question
                handle_question(&cmd).await;
            }
        }
    } else {
        // REPL mode
        run_repl().await?;
    }

    Ok(())
}
