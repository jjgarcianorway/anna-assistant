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
async fn ask_streaming(question: &str) -> Result<()> {
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

    // Send request
    let request = RpcRequest::new(RpcMethod::AskStreaming, Some(serde_json::json!({ "question": question })));
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
    let mut iterations = 0;

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
                        iterations = result.iterations;
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
                return Err(anyhow!("Request timed out after {}s", RPC_TIMEOUT_SECS));
            }
        }
    }

    println!();
    println_colored(&format!("({} iterations)", iterations), DIM);

    Ok(())
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
            println_colored("(asking for commands)", DIM);
            let lines: Vec<&str> = step.content.lines().collect();
            if lines.len() > 3 {
                println_colored(&format!("  {}", lines[0]), DIM);
                println_colored("  ...", DIM);
            }
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
            println_colored("(validating output)", DIM);
            println!();
        }
        StepType::ValidationResponse => {
            print_colored("LLM → ANNA: ", YELLOW);
            println!("{}", step.content);
            println!();
        }
        StepType::FinalPrompt => {
            print_colored("ANNA → LLM: ", YELLOW);
            println_colored("(generating final answer)", DIM);
            println!();
        }
        StepType::FinalAnswer => {
            // This step comes after streaming, so we don't print it again
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
const CYAN: &str = "\x1b[36m";
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

/// Handle a question
async fn handle_question(question: &str) {
    // Clear line and start streaming
    println!();

    match ask_streaming(question).await {
        Ok(()) => {
            // Done - iterations printed by ask_streaming
        }
        Err(e) => {
            print_colored("Error: ", RED);
            println!("{}", e);
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
                println_colored("(asking for commands)", DIM);
                // Show abbreviated prompt
                let lines: Vec<&str> = step.content.lines().collect();
                if lines.len() > 3 {
                    println_colored(&format!("  {}", lines[0]), DIM);
                    println_colored("  ...", DIM);
                }
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
                println_colored("(validating output)", DIM);
                println!();
            }
            StepType::ValidationResponse => {
                print_colored("LLM → ANNA: ", YELLOW);
                println!("{}", step.content);
                println!();
            }
            StepType::FinalPrompt => {
                print_colored("ANNA → LLM: ", YELLOW);
                println_colored("(generating final answer)", DIM);
                println!();
            }
            StepType::FinalAnswer => {
                println_colored("═══════════════════════════════════════", DIM);
                print_colored("ANSWER: ", GREEN);
                println!();
                println_colored(&step.content, GREEN);
                println_colored("═══════════════════════════════════════", DIM);
            }
        }
    }
}

/// Run the REPL
async fn run_repl() -> Result<()> {
    print_greeting();
    print_status().await;
    println!();

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
                        handle_question(input).await;
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
