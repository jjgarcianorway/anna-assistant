//! LLM-Only Core Loop - No pattern matching, pure intelligence.
//!
//! Architecture:
//! 1. UNDERSTAND - LLM parses intent and what info is needed
//! 2. INVESTIGATE - LLM decides commands, executes in stages
//! 3. ANALYZE - LLM correlates findings, identifies issues
//! 4. RESPOND - Grounded answer or smart fix suggestion
//!
//! Key principles:
//! - LLM always decides, no hardcoded patterns
//! - Multi-stage investigation (overview → deep dive)
//! - Fixes suggested based on actual findings, not keywords
//! - All answers grounded in command output

pub mod investigate;
pub mod prompts;

use anna_shared::rpc::{AskResult, DialogueStep, StepType, StreamingResponse};
use anna_shared::claim_gate::{ClaimGate, ClaimVerifier, EvidenceType};
use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info};

use crate::core_loop::command::execute_command;
use crate::ollama::chat_with_timeout;

/// Maximum investigation iterations (keep low to avoid timeouts)
const MAX_ITERATIONS: u8 = 3;
/// LLM timeout in seconds
const LLM_TIMEOUT_SECS: u64 = 60;

/// v0.3.25: Result of ClaimGate verification
pub struct VerificationResult {
    /// The answer text (may have [unverified] markers)
    pub answer: String,
    /// Evidence line to append (formatted)
    pub evidence_line: String,
    /// Whether investigation is needed (unverified FACT claims with low confidence)
    pub needs_investigation: bool,
    /// Suggested probes if investigation is needed
    pub suggested_probes: Vec<String>,
    /// Number of verified claims
    pub verified_count: usize,
    /// Number of unverified claims
    pub unverified_count: usize,
}

/// v0.3.27: Format evidence line for display with doc citations and failed probes
fn format_evidence_line(
    findings: &[Finding],
    doc_citations: &[String],
    failed_probes: &[String],
    debug_mode: bool,
) -> String {
    // v0.3.27: Always emit Evidence line if there are findings or citations
    // If all probes failed, say so explicitly
    if findings.is_empty() && doc_citations.is_empty() {
        return String::new();
    }

    let successful_findings: Vec<&Finding> = findings.iter().filter(|f| f.success).collect();
    let failed_findings: Vec<&Finding> = findings.iter().filter(|f| !f.success).collect();

    // If ALL probes failed, emit explicit failure notice
    if !findings.is_empty() && successful_findings.is_empty() && doc_citations.is_empty() {
        return format!(
            "Evidence: [ALL PROBES FAILED: {}]",
            failed_probes.join(", ")
        );
    }

    if debug_mode {
        // Verbose format with exit codes and doc citations
        let mut lines = vec!["Evidence:".to_string()];
        for f in findings {
            // v0.3.28: Phase 3 F15 - mark empty output on success
            let status = if f.success {
                if f.output.trim().is_empty() { "OK, empty" } else { "OK" }
            } else {
                "FAILED"
            };
            lines.push(format!(
                "  [Probe: `{}` ({})]",
                f.command, status
            ));
        }
        for cite in doc_citations {
            lines.push(format!("  {}", cite));
        }
        if !failed_probes.is_empty() {
            lines.push(format!("  [Failed probes: {}]", failed_probes.join(", ")));
        }
        lines.join("\n")
    } else {
        // Concise format - successful probes and doc citations
        // v0.3.27: Mark failed probes explicitly
        // v0.3.28: Phase 3 F15 - mark empty output probes
        let mut parts: Vec<String> = successful_findings
            .iter()
            .map(|f| {
                if f.output.trim().is_empty() {
                    format!("{}[empty]", f.command)
                } else {
                    f.command.clone()
                }
            })
            .collect();
        parts.extend(doc_citations.iter().cloned());

        if !failed_findings.is_empty() {
            let failed_cmds: Vec<String> = failed_findings.iter().map(|f| format!("{}[FAILED]", f.command)).collect();
            parts.extend(failed_cmds);
        }

        if parts.is_empty() {
            "Evidence: [no successful probes]".to_string()
        } else {
            format!("Evidence: {}", parts.join(", "))
        }
    }
}

/// v0.3.27: Verify answer through ClaimGate with question context for doc requirements
fn verify_answer(
    answer: &str,
    question: &str,
    findings: &[Finding],
    debug_mode: bool,
) -> VerificationResult {
    use anna_shared::claim_gate::ClaimVerifier;

    let gate = ClaimGate::new();

    // Build probe evidence from findings
    let mut evidence: Vec<EvidenceType> = findings
        .iter()
        .map(|f| {
            ClaimGate::evidence_from_probe(
                &f.command,
                &f.output,
                if f.success { 0 } else { 1 },
            )
        })
        .collect();

    // v0.3.26: If docs are required, search for relevant documentation
    if ClaimGate::claim_requires_docs(question) {
        // Search local docs
        let doc_citations = anna_shared::docs::search_docs(question);
        for cite in &doc_citations {
            evidence.push(ClaimGate::evidence_from_doc_citation(cite));
        }
        debug!("Found {} doc citations for question", doc_citations.len());
    }

    // Verify the response with question context
    let verified = gate.verify_response_with_context(answer, question, &evidence);

    let verified_count = verified.verified_claims.len();
    let unverified_count = verified.unverified_claims.len();

    // v0.3.27: Log blocking information
    if verified.claims_blocked {
        info!(
            "ClaimGate: BLOCKED {} unverified claims, {} verified, probes_failed={}",
            unverified_count, verified_count, verified.probes_failed
        );
        for reason in &verified.block_reasons {
            debug!("Block reason: {}", reason);
        }
    } else if !verified.unverified_claims.is_empty() {
        info!(
            "ClaimGate: {} claims verified, {} unverified, docs_required={}, docs_found={}",
            verified_count, unverified_count, verified.docs_required, verified.docs_found
        );
    }

    // v0.3.27: Format evidence line with failed probes info
    let evidence_line = format_evidence_line(
        findings,
        &verified.doc_citations,
        &verified.failed_probes,
        debug_mode,
    );

    VerificationResult {
        // v0.3.27: Use verified_text which has blocked claims replaced with uncertainty
        answer: verified.verified_text,
        evidence_line,
        needs_investigation: verified.needs_investigation,
        suggested_probes: verified.suggested_probes,
        verified_count,
        unverified_count,
    }
}

/// Investigation state tracks what we've learned
#[derive(Debug, Default)]
pub struct InvestigationState {
    /// Commands we've run and their outputs
    pub findings: Vec<Finding>,
    /// What we still need to find out
    pub open_questions: Vec<String>,
    /// Current iteration
    pub iteration: u8,
}

/// A single finding from command execution
#[derive(Debug, Clone)]
pub struct Finding {
    pub command: String,
    pub output: String,
    pub success: bool,
}

/// Result of the LLM deciding next steps
#[derive(Debug)]
pub enum NextStep {
    /// Run these commands to gather more info
    Investigate(Vec<String>),
    /// Have enough info, generate answer
    Answer,
    /// Found a problem, suggest a fix
    SuggestFix { problem: String, fix_command: String, explanation: String },
    /// Can't help with this
    OutOfScope(String),
}

/// Main entry point - execute a question using pure LLM intelligence
pub async fn execute_question_llm(model: &str, question: &str) -> Result<AskResult> {
    info!("LLM Core: Processing question: {}", question);

    let mut state = InvestigationState::default();
    let dialogue = Vec::new();

    // PHASE 1: UNDERSTAND - What is the user asking?
    let understanding = understand_question(model, question).await?;
    debug!("Understanding: {:?}", understanding);

    // Check if out of scope
    if let Some(reason) = &understanding.out_of_scope_reason {
        return Ok(AskResult {
            answer: format!("I'm Anna, your Linux assistant. {}", reason),
            success: true,
            iterations: 0,
            commands_executed: vec![],
            dialogue,
            needs_clarification: false,
            clarification_question: None,
            cached: false,
            citations: vec![],
        });
    }

    // PHASE 2: INVESTIGATE - Gather information
    loop {
        state.iteration += 1;
        if state.iteration > MAX_ITERATIONS {
            info!("Max iterations reached");
            break;
        }

        // Ask LLM what to do next
        let next = decide_next_step(model, question, &state).await?;

        match next {
            NextStep::Investigate(commands) => {
                info!("Iteration {}: Running {} commands", state.iteration, commands.len());

                for cmd in commands {
                    // Execute command (sync function, run in blocking task)
                    let cmd_clone = cmd.clone();
                    let result = tokio::task::spawn_blocking(move || {
                        execute_command(&cmd_clone)
                    }).await?;

                    let (output, success) = match result {
                        Ok(out) => (out, true),
                        Err(e) => (format!("Error: {}", e), false),
                    };

                    state.findings.push(Finding {
                        command: cmd,
                        output,
                        success,
                    });
                }
            }
            NextStep::Answer => {
                info!("LLM decided: enough info to answer");
                break;
            }
            NextStep::SuggestFix { problem, fix_command, explanation } => {
                info!("LLM found problem, suggesting fix");
                let answer = format!(
                    "I found the issue: {}\n\n\
                     I can fix this by running:\n  {}\n\n\
                     {}\n\n\
                     Would you like me to do this? (yes/no)",
                    problem, fix_command, explanation
                );
                return Ok(AskResult {
                    answer,
                    success: true,
                    iterations: state.iteration as u32,
                    commands_executed: state.findings.iter().map(|f| f.command.clone()).collect(),
                    dialogue,
                    needs_clarification: true,
                    clarification_question: Some("Confirm fix?".to_string()),
                    cached: false,
                    citations: vec![],
                });
            }
            NextStep::OutOfScope(reason) => {
                return Ok(AskResult {
                    answer: format!("I'm Anna, your Linux assistant. {}", reason),
                    success: true,
                    iterations: state.iteration as u32,
                    commands_executed: state.findings.iter().map(|f| f.command.clone()).collect(),
                    dialogue,
                    needs_clarification: false,
                    clarification_question: None,
                    cached: false,
                    citations: vec![],
                });
            }
        }
    }

    // PHASE 3: RESPOND - Generate grounded answer
    let raw_answer = generate_answer(model, question, &state).await?;

    // v0.3.25: Verify through ClaimGate and append evidence line
    let debug_mode = anna_shared::config::AnnaConfig::load()
        .map(|c| c.debug_mode)
        .unwrap_or(false);
    let verification = verify_answer(&raw_answer, question, &state.findings, debug_mode);

    // Append evidence line if we have any findings
    let answer = if verification.evidence_line.is_empty() {
        verification.answer
    } else {
        format!("{}\n\n{}", verification.answer, verification.evidence_line)
    };

    Ok(AskResult {
        answer,
        success: true,
        iterations: state.iteration as u32,
        commands_executed: state.findings.iter().map(|f| f.command.clone()).collect(),
        dialogue,
        needs_clarification: false,
        clarification_question: None,
        cached: false,
        citations: vec![],
    })
}

/// Streaming version of execute_question
/// session_context is accepted for API compatibility but not currently used
pub async fn execute_question_streaming_llm<W: AsyncWriteExt + Unpin>(
    model: &str,
    question: &str,
    _session_context: Option<&str>,
    writer: &mut W,
) -> Result<AskResult> {
    info!("LLM Core (streaming): Processing question: {}", question);

    let mut state = InvestigationState::default();
    let mut dialogue = Vec::new();

    // Helper to send streaming updates
    async fn send_step<W: AsyncWriteExt + Unpin>(
        writer: &mut W,
        step: DialogueStep,
        dialogue: &mut Vec<DialogueStep>,
    ) -> Result<()> {
        dialogue.push(step.clone());
        let response = StreamingResponse::Step { step };
        let json = serde_json::to_string(&response)?;
        writer.write_all(format!("{}\n", json).as_bytes()).await?;
        Ok(())
    }

    // PHASE 1: UNDERSTAND
    send_step(writer, DialogueStep {
        step_type: StepType::AnnaToLlm,
        content: "Understanding question...".to_string(),
    }, &mut dialogue).await?;

    let understanding = understand_question(model, question).await?;

    if let Some(reason) = &understanding.out_of_scope_reason {
        let result = AskResult {
            answer: format!("I'm Anna, your Linux assistant. {}", reason),
            success: true,
            iterations: 0,
            commands_executed: vec![],
            dialogue: dialogue.clone(),
            needs_clarification: false,
            clarification_question: None,
            cached: false,
            citations: vec![],
        };
        let response = StreamingResponse::Done { result: result.clone() };
        let json = serde_json::to_string(&response)?;
        writer.write_all(format!("{}\n", json).as_bytes()).await?;
        return Ok(result);
    }

    // PHASE 2: INVESTIGATE
    loop {
        state.iteration += 1;
        if state.iteration > MAX_ITERATIONS {
            break;
        }

        send_step(writer, DialogueStep {
            step_type: StepType::AnnaToLlm,
            content: format!("Investigation iteration {}...", state.iteration),
        }, &mut dialogue).await?;

        let next = decide_next_step(model, question, &state).await?;

        match next {
            NextStep::Investigate(commands) => {
                for cmd in commands {
                    // Show command being executed
                    send_step(writer, DialogueStep {
                        step_type: StepType::CommandExec,
                        content: cmd.clone(),
                    }, &mut dialogue).await?;

                    // Execute command
                    let cmd_clone = cmd.clone();
                    let result = tokio::task::spawn_blocking(move || {
                        execute_command(&cmd_clone)
                    }).await?;

                    let (output, success) = match result {
                        Ok(out) => (out, true),
                        Err(e) => (format!("Error: {}", e), false),
                    };

                    // Show truncated output
                    let display_output = if output.len() > 500 {
                        format!("{}...(truncated)", &output[..500])
                    } else {
                        output.clone()
                    };

                    send_step(writer, DialogueStep {
                        step_type: StepType::CommandOutput,
                        content: display_output,
                    }, &mut dialogue).await?;

                    state.findings.push(Finding { command: cmd, output, success });
                }
            }
            NextStep::Answer => break,
            NextStep::SuggestFix { problem, fix_command, explanation } => {
                let raw_answer = format!(
                    "I found the issue: {}\n\n\
                     I can fix this by running:\n  {}\n\n\
                     {}\n\n\
                     Would you like me to do this? (yes/no)",
                    problem, fix_command, explanation
                );

                // v0.3.25: Verify answer through ClaimGate with evidence line
                let debug_mode = anna_shared::config::AnnaConfig::load()
                    .map(|c| c.debug_mode)
                    .unwrap_or(false);
                let verification = verify_answer(&raw_answer, question, &state.findings, debug_mode);

                // Append evidence line
                let final_answer = if verification.evidence_line.is_empty() {
                    verification.answer.clone()
                } else {
                    format!("{}\n\n{}", verification.answer, verification.evidence_line)
                };

                // v0.3.22: Truth-first rendering - send complete answer, no fake streaming
                send_step(writer, DialogueStep {
                    step_type: StepType::FinalAnswer,
                    content: final_answer.clone(),
                }, &mut dialogue).await?;

                let result = AskResult {
                    answer: final_answer,
                    success: true,
                    iterations: state.iteration as u32,
                    commands_executed: state.findings.iter().map(|f| f.command.clone()).collect(),
                    dialogue: dialogue.clone(),
                    needs_clarification: true,
                    clarification_question: Some("Confirm fix?".to_string()),
                    cached: false,
                    citations: vec![],
                };
                let response = StreamingResponse::Done { result: result.clone() };
                let json = serde_json::to_string(&response)?;
                writer.write_all(format!("{}\n", json).as_bytes()).await?;
                return Ok(result);
            }
            NextStep::OutOfScope(reason) => {
                let result = AskResult {
                    answer: format!("I'm Anna, your Linux assistant. {}", reason),
                    success: true,
                    iterations: state.iteration as u32,
                    commands_executed: state.findings.iter().map(|f| f.command.clone()).collect(),
                    dialogue: dialogue.clone(),
                    needs_clarification: false,
                    clarification_question: None,
                    cached: false,
                    citations: vec![],
                };
                let response = StreamingResponse::Done { result: result.clone() };
                let json = serde_json::to_string(&response)?;
                writer.write_all(format!("{}\n", json).as_bytes()).await?;
                return Ok(result);
            }
        }
    }

    // PHASE 3: GENERATE ANSWER
    send_step(writer, DialogueStep {
        step_type: StepType::AnnaToLlm,
        content: "Generating answer...".to_string(),
    }, &mut dialogue).await?;

    let raw_answer = generate_answer(model, question, &state).await?;

    // v0.3.25: Verify answer through ClaimGate with evidence line
    let debug_mode = anna_shared::config::AnnaConfig::load()
        .map(|c| c.debug_mode)
        .unwrap_or(false);
    let verification = verify_answer(&raw_answer, question, &state.findings, debug_mode);

    // Append evidence line if we have any findings
    let final_answer = if verification.evidence_line.is_empty() {
        verification.answer.clone()
    } else {
        format!("{}\n\n{}", verification.answer, verification.evidence_line)
    };

    // v0.3.22: Truth-first rendering - send complete answer, no fake streaming
    send_step(writer, DialogueStep {
        step_type: StepType::FinalAnswer,
        content: final_answer.clone(),
    }, &mut dialogue).await?;

    let result = AskResult {
        answer: final_answer,
        success: true,
        iterations: state.iteration as u32,
        commands_executed: state.findings.iter().map(|f| f.command.clone()).collect(),
        dialogue,
        needs_clarification: false,
        clarification_question: None,
        cached: false,
        citations: vec![],
    };

    let response = StreamingResponse::Done { result: result.clone() };
    let json = serde_json::to_string(&response)?;
    writer.write_all(format!("{}\n", json).as_bytes()).await?;

    Ok(result)
}

/// Understanding result from LLM
#[derive(Debug)]
struct Understanding {
    /// What type of question is this
    #[allow(dead_code)]
    intent: String,
    /// What information do we need to answer
    #[allow(dead_code)]
    info_needed: Vec<String>,
    /// Is this out of scope?
    out_of_scope_reason: Option<String>,
}

/// Use LLM to understand the question
async fn understand_question(model: &str, question: &str) -> Result<Understanding> {
    let prompt = prompts::understanding_prompt(question);

    let response = chat_with_timeout(model, &prompt, LLM_TIMEOUT_SECS).await?;

    // Parse LLM response
    let mut intent = "factual".to_string();
    let mut info_needed = Vec::new();
    let mut out_of_scope_reason = None;

    for line in response.lines() {
        let line = line.trim();
        if line.starts_with("INTENT:") {
            intent = line.trim_start_matches("INTENT:").trim().to_lowercase();
        } else if line.starts_with("NEED:") {
            info_needed.push(line.trim_start_matches("NEED:").trim().to_string());
        } else if line.starts_with("OUT_OF_SCOPE:") {
            out_of_scope_reason = Some(line.trim_start_matches("OUT_OF_SCOPE:").trim().to_string());
        }
    }

    Ok(Understanding { intent, info_needed, out_of_scope_reason })
}

/// Use LLM to decide what to do next
async fn decide_next_step(model: &str, question: &str, state: &InvestigationState) -> Result<NextStep> {
    let prompt = prompts::next_step_prompt(question, state);

    let response = chat_with_timeout(model, &prompt, LLM_TIMEOUT_SECS).await?;
    let response = response.trim();

    // Parse LLM decision
    if response.starts_with("COMMANDS:") {
        let commands: Vec<String> = response
            .lines()
            .skip(1)
            .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
            .map(|l| l.trim().to_string())
            .filter(|cmd| is_valid_command(cmd))
            .take(3)
            .collect();

        if commands.is_empty() {
            return Ok(NextStep::Answer);
        }
        return Ok(NextStep::Investigate(commands));
    }

    if response.starts_with("ANSWER") {
        return Ok(NextStep::Answer);
    }

    if response.starts_with("FIX:") {
        let mut problem = String::new();
        let mut fix_command = String::new();
        let mut explanation = String::new();

        for line in response.lines() {
            if line.starts_with("FIX:") {
                fix_command = line.trim_start_matches("FIX:").trim().to_string();
            } else if line.starts_with("PROBLEM:") {
                problem = line.trim_start_matches("PROBLEM:").trim().to_string();
            } else if line.starts_with("EXPLAIN:") {
                explanation = line.trim_start_matches("EXPLAIN:").trim().to_string();
            }
        }

        if !fix_command.is_empty() && !problem.is_empty() {
            return Ok(NextStep::SuggestFix { problem, fix_command, explanation });
        }
    }

    if response.starts_with("OUT_OF_SCOPE:") {
        let reason = response.trim_start_matches("OUT_OF_SCOPE:").trim().to_string();
        return Ok(NextStep::OutOfScope(reason));
    }

    // Default: try to extract commands from response
    let commands: Vec<String> = response
        .lines()
        .filter(|l| {
            let l = l.trim();
            !l.is_empty() && !l.starts_with('#') && !l.contains(':')
        })
        .map(|l| l.trim().to_string())
        .filter(|cmd| is_valid_command(cmd))
        .take(2)
        .collect();

    if commands.is_empty() {
        Ok(NextStep::Answer)
    } else {
        Ok(NextStep::Investigate(commands))
    }
}

/// Validate that a string looks like a valid bash command, not garbage
fn is_valid_command(cmd: &str) -> bool {
    let cmd = cmd.trim();

    // Too short or too long
    if cmd.len() < 2 || cmd.len() > 300 {
        return false;
    }

    // Contains LLM prompt tokens or non-ASCII
    if cmd.contains("<|") || cmd.contains("|>") {
        return false;
    }

    // Reject commands with non-ASCII characters (Chinese, etc)
    if !cmd.chars().all(|c| c.is_ascii() || c == '/' || c == '-' || c == '_') {
        return false;
    }

    // Starts with common English words (not commands)
    let english_starts = [
        "Please", "Could", "Would", "Can", "The", "This", "That", "It", "If",
        "To", "For", "With", "From", "I ", "You", "We", "They", "What", "How",
        "Why", "When", "Where", "Is", "Are", "Was", "Were", "Been", "Being",
        "Have", "Has", "Had", "Do", "Does", "Did", "Will", "Shall", "May",
        "Might", "Must", "Should", "A ", "An ", "Based", "Here", "Let",
    ];
    for word in english_starts {
        if cmd.starts_with(word) {
            return false;
        }
    }

    // First word should look like a command
    let first_word = cmd.split_whitespace().next().unwrap_or("");
    if first_word.is_empty() {
        return false;
    }

    // Valid command patterns: starts with letter, or ./ or /
    let first_char = first_word.chars().next().unwrap_or(' ');
    if !first_char.is_ascii_alphabetic() && first_char != '.' && first_char != '/' {
        return false;
    }

    // EXACT valid commands (not prefixes - prevents systemd-analyzeblade)
    // v0.2.6: Expanded command list
    let valid_commands = [
        // Core utils
        "ls", "cat", "head", "tail", "grep", "awk", "sed", "find", "df", "du",
        "wc", "sort", "uniq", "cut", "tr", "tee", "xargs", "basename", "dirname",
        "cp", "mv", "touch", "mkdir", "rm", "ln", "readlink",
        // System info
        "free", "ps", "uptime", "uname", "lscpu", "lspci", "lsblk", "lsusb", "lsof",
        "hostnamectl", "timedatectl", "localectl", "locale", "hwinfo",
        // Storage
        "mount", "umount", "findmnt", "swapon", "swapoff", "mkswap",
        "fdisk", "gdisk", "parted", "blkid", "smartctl", "hdparm",
        "zpool", "zfs", "btrfs", "cryptsetup", "lvm", "mdadm", "lvs", "vgs", "pvs",
        // Systemd
        "systemctl", "journalctl", "systemd-analyze", "loginctl", "coredumpctl",
        // Network
        "ip", "ss", "ping", "curl", "wget", "traceroute", "dig", "nslookup", "host",
        "nmcli", "iwctl", "rfkill", "iw", "ethtool", "netstat", "arp",
        "nft", "iptables", "firewall-cmd", "ufw",
        // Packages
        "pacman", "yay", "paru", "makepkg", "pkgfile", "pacsearch",
        // Hardware
        "nvidia-smi", "glxinfo", "vulkaninfo", "vainfo", "vdpauinfo",
        "lsmod", "modinfo", "modprobe", "dmesg", "sensors", "acpi", "dmidecode",
        "upower", "powertop", "tlp-stat", "cpupower", "turbostat",
        // Audio
        "pactl", "pipewire", "pw-cli", "pw-dump", "wpctl", "aplay", "arecord", "amixer",
        // Display
        "xrandr", "wlr-randr", "swaymsg", "hyprctl", "xdpyinfo", "xwininfo",
        // Users/Auth
        "id", "whoami", "groups", "passwd", "chown", "chmod", "chsh",
        "getent", "last", "lastlog", "w", "who", "users",
        // Environment
        "printenv", "env", "echo", "printf", "test", "true", "false", "set", "export",
        // Other system
        "sudo", "which", "whereis", "file", "type", "stat", "date", "cal",
        "logger", "xdg-open", "fwupdmgr", "bluetoothctl",
        // Printing
        "cupsd", "lpstat", "lpq", "lp", "cancel",
        // Monitoring (interactive but useful output)
        "top", "htop", "btop", "iotop", "nethogs", "iftop",
    ];

    // Check if first word exactly matches a valid command
    let base_cmd = first_word.split('/').last().unwrap_or(first_word);
    for valid in valid_commands {
        if base_cmd == valid {
            return true;
        }
    }

    // Also allow absolute paths to common locations
    if first_word.starts_with("/usr/bin/") ||
       first_word.starts_with("/bin/") ||
       first_word.starts_with("/sbin/") ||
       first_word.starts_with("./") {
        return true;
    }

    false
}

/// Use LLM to generate final answer based on findings
async fn generate_answer(model: &str, question: &str, state: &InvestigationState) -> Result<String> {
    let prompt = prompts::answer_prompt(question, state);

    let response = chat_with_timeout(model, &prompt, LLM_TIMEOUT_SECS).await?;

    Ok(response.trim().to_string())
}
