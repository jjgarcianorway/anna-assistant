//! Command execution and retry logic.
//! v0.0.919: Added auto-tool installation when command not found

use anna_shared::config::AnnaConfig;
use anna_shared::deps::{command_exists, install_package};
use anna_shared::memory::Memory;
use anyhow::Result;
use std::process::Command;
use tracing::{debug, info, warn};

use super::cache::{cache_command, get_cached_command, get_perf_config, is_known_failed_command, record_command_failure_cache, clear_failure_cache};
use super::safety::is_dangerous_command;
use crate::ollama;

/// Map commands to their package names (command -> package)
/// v0.0.919: Used for auto-installation
/// v0.0.931: Expanded from 40 to 80+ tools
const COMMAND_TO_PACKAGE: &[(&str, &str)] = &[
    // Calculators and processors
    ("bc", "bc"),
    ("jq", "jq"),
    ("yq", "yq"),
    // System monitoring
    ("htop", "htop"),
    ("btop", "btop"),
    ("iotop", "iotop"),
    ("nethogs", "nethogs"),
    ("iftop", "iftop"),
    ("bmon", "bmon"),
    ("nload", "nload"),
    ("bandwhich", "bandwhich"),
    ("procs", "procs"),
    // Process/file tools
    ("lsof", "lsof"),
    ("strace", "strace"),
    ("ltrace", "ltrace"),
    ("fuser", "psmisc"),
    ("pstree", "psmisc"),
    ("killall", "psmisc"),
    // Disk tools
    ("smartctl", "smartmontools"),
    ("iostat", "sysstat"),
    ("mpstat", "sysstat"),
    ("sar", "sysstat"),
    ("dust", "dust"),
    ("duf", "duf"),
    ("ncdu", "ncdu"),
    ("gdu", "gdu"),
    // Network tools
    ("netstat", "net-tools"),
    ("ifconfig", "net-tools"),
    ("nslookup", "bind"),
    ("dig", "bind"),
    ("host", "bind"),
    ("traceroute", "traceroute"),
    ("mtr", "mtr"),
    ("tcpdump", "tcpdump"),
    ("nmap", "nmap"),
    ("ss", "iproute2"),
    ("curl", "curl"),
    ("wget", "wget"),
    ("httpie", "httpie"),
    ("socat", "socat"),
    ("netcat", "gnu-netcat"),
    // Hardware info
    ("lspci", "pciutils"),
    ("lsusb", "usbutils"),
    ("lshw", "lshw"),
    ("dmidecode", "dmidecode"),
    ("sensors", "lm_sensors"),
    ("inxi", "inxi"),
    ("hwinfo", "hwinfo"),
    // Archive tools
    ("unzip", "unzip"),
    ("unrar", "unrar"),
    ("7z", "p7zip"),
    ("tar", "tar"),
    ("gzip", "gzip"),
    ("bzip2", "bzip2"),
    ("xz", "xz"),
    ("zstd", "zstd"),
    // Modern CLI tools
    ("rg", "ripgrep"),
    ("ripgrep", "ripgrep"),
    ("fd", "fd"),
    ("bat", "bat"),
    ("eza", "eza"),
    ("exa", "eza"),
    ("fzf", "fzf"),
    ("delta", "git-delta"),
    ("sd", "sd"),
    ("hyperfine", "hyperfine"),
    ("tokei", "tokei"),
    // Development
    ("tree", "tree"),
    ("git", "git"),
    ("make", "make"),
    ("cmake", "cmake"),
    ("clang", "clang"),
    ("gcc", "gcc"),
    ("gdb", "gdb"),
    ("valgrind", "valgrind"),
    // Text processing
    ("awk", "gawk"),
    ("sed", "sed"),
    ("grep", "grep"),
    ("ag", "the_silver_searcher"),
    // GPU
    ("nvidia-smi", "nvidia-utils"),
    ("glxinfo", "mesa-utils"),
    ("vainfo", "libva-utils"),
    ("vdpauinfo", "vdpauinfo"),
    ("vulkaninfo", "vulkan-tools"),
    // Audio
    ("pactl", "libpulse"),
    ("wpctl", "wireplumber"),
    ("pw-top", "pipewire"),
    ("alsamixer", "alsa-utils"),
    // Bluetooth
    ("bluetoothctl", "bluez-utils"),
    // Power/battery
    ("acpi", "acpi"),
    ("upower", "upower"),
    ("powertop", "powertop"),
    ("tlp-stat", "tlp"),
];

/// Error categories for intelligent recovery
#[derive(Debug, Clone, PartialEq)]
pub enum CommandErrorType {
    CommandNotFound,
    PermissionDenied,
    PathNotFound,
    Timeout,
    SyntaxError,
    MissingDependency,
    EmptyOutput,
    Unknown,
}

/// Execute a shell command and return its output
/// v0.0.919: Added configurable timeout support
/// v0.0.921: Added negative learning (skip known-failed commands)
/// v0.0.925: Get command-specific timeout based on command type
fn get_command_timeout(cmd: &str) -> u64 {
    let base_timeout = get_perf_config().command_timeout_secs;
    let cmd_lower = cmd.to_lowercase();

    // Package managers need more time (downloads, installs)
    if cmd_lower.starts_with("pacman ")
        || cmd_lower.starts_with("yay ")
        || cmd_lower.starts_with("paru ")
        || cmd_lower.starts_with("apt ")
        || cmd_lower.starts_with("dnf ")
        || cmd_lower.starts_with("zypper ")
    {
        return 120.max(base_timeout);
    }

    // Recursive searches can take a while
    if cmd_lower.contains("find ") && (cmd_lower.contains(" /") || cmd_lower.contains(" ~"))
        || cmd_lower.contains("grep -r")
        || cmd_lower.contains("rg ")
    {
        return 60.max(base_timeout);
    }

    // System updates need even more time
    if cmd_lower.contains("-syu") || cmd_lower.contains("upgrade") || cmd_lower.contains("update") {
        return 180.max(base_timeout);
    }

    // Network commands with potential delays
    if cmd_lower.starts_with("ping ")
        || cmd_lower.starts_with("curl ")
        || cmd_lower.starts_with("wget ")
        || cmd_lower.starts_with("ssh ")
    {
        return 30.max(base_timeout);
    }

    // Quick read-only commands can use shorter timeout
    if cmd_lower.starts_with("cat ")
        || cmd_lower.starts_with("head ")
        || cmd_lower.starts_with("tail ")
        || cmd_lower.starts_with("echo ")
        || cmd_lower.starts_with("ls ")
        || cmd_lower.starts_with("stat ")
    {
        return 10.min(base_timeout);
    }

    base_timeout
}

pub fn execute_command(cmd: &str) -> Result<String> {
    // Check cache first
    if let Some(cached) = get_cached_command(cmd) {
        return Ok(cached);
    }

    // v0.0.921: Check if this command is known to fail
    if let Some(error_type) = is_known_failed_command(cmd) {
        debug!("Skipping known-failed command: {} ({})", cmd, error_type);
        return Ok(format!("[SKIPPED] Known failed command: {}", error_type));
    }

    // v0.0.925: Get command-specific timeout
    let timeout_secs = get_command_timeout(cmd);

    // Use timeout wrapper to prevent hanging commands
    let output = Command::new("timeout")
        .arg(format!("{}s", timeout_secs))
        .arg("sh")
        .arg("-c")
        .arg(cmd)
        .output()?;

    // Check if command timed out (exit code 124)
    if output.status.code() == Some(124) {
        return Ok(format!("[TIMEOUT] Command timed out after {}s: {}", timeout_secs, cmd));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let result = if stdout.trim().is_empty() && !stderr.trim().is_empty() {
        stderr
    } else {
        stdout
    };

    // v0.0.925: If empty output and no error, try alternative command
    if result.trim().is_empty() && output.status.success() {
        if let Some(alt_cmd) = get_alternative_command(cmd) {
            debug!("Empty output from '{}', trying alternative: {}", cmd, alt_cmd);
            let alt_output = Command::new("timeout")
                .arg(format!("{}s", timeout_secs))
                .arg("sh")
                .arg("-c")
                .arg(&alt_cmd)
                .output();

            if let Ok(alt_out) = alt_output {
                let alt_stdout = String::from_utf8_lossy(&alt_out.stdout).to_string();
                if !alt_stdout.trim().is_empty() {
                    let cleaned = strip_ansi_codes(&alt_stdout);
                    cache_command(cmd, &cleaned);
                    return Ok(cleaned);
                }
            }
        }
    }

    let cleaned = strip_ansi_codes(&result);
    cache_command(cmd, &cleaned);
    Ok(cleaned)
}

/// v0.0.925: Get alternative command when primary returns empty output
fn get_alternative_command(cmd: &str) -> Option<String> {
    let cmd_lower = cmd.to_lowercase();

    // systemctl alternatives
    if cmd_lower.contains("systemctl list-units") && cmd_lower.contains("--failed") {
        return Some("systemctl --failed 2>/dev/null || journalctl -p err -n 5".to_string());
    }

    // Process listing alternatives
    if cmd_lower.starts_with("pgrep ") {
        let pattern = cmd_lower.strip_prefix("pgrep ").unwrap_or("");
        return Some(format!("ps aux | grep -i '{}' | grep -v grep", pattern.trim()));
    }

    // Network alternatives
    if cmd_lower.starts_with("ss ") {
        return Some(cmd.replace("ss ", "netstat "));
    }
    if cmd_lower.starts_with("ip addr") {
        return Some("ifconfig 2>/dev/null || hostname -I".to_string());
    }

    // Disk alternatives
    if cmd_lower.starts_with("lsblk") && cmd_lower.contains("-f") {
        return Some("blkid 2>/dev/null || df -Th".to_string());
    }

    // Memory alternatives
    if cmd_lower.starts_with("free ") {
        return Some("cat /proc/meminfo | head -10".to_string());
    }

    // Log alternatives
    if cmd_lower.starts_with("journalctl") && cmd_lower.contains("-p err") {
        return Some("dmesg --level=err,warn 2>/dev/null | tail -20".to_string());
    }

    None
}

/// Strip ANSI escape codes from text
pub fn strip_ansi_codes(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                while let Some(&next) = chars.peek() {
                    chars.next();
                    if next.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// Classify error to enable intelligent recovery
/// v0.0.932: Expanded from 8 to 20+ error patterns
pub fn classify_command_error(output: &str, error: Option<&str>) -> (CommandErrorType, &'static str) {
    let combined = format!("{} {}", output, error.unwrap_or("")).to_lowercase();

    // Command not found patterns
    if combined.contains("command not found")
        || combined.contains("not found in path")
        || combined.contains("no such command")
        || combined.contains("not recognized as")
        || combined.contains(": not found")
    {
        return (CommandErrorType::CommandNotFound, "Install the package or use an alternative command");
    }

    // Permission denied patterns
    if combined.contains("permission denied")
        || combined.contains("operation not permitted")
        || combined.contains("access denied")
        || combined.contains("not permitted")
        || combined.contains("insufficient permissions")
        || combined.contains("must be root")
        || combined.contains("requires root")
        || combined.contains("need to be root")
    {
        return (CommandErrorType::PermissionDenied, "Try with sudo or check file permissions");
    }

    // Path not found patterns
    if combined.contains("no such file")
        || combined.contains("does not exist")
        || combined.contains("cannot find")
        || combined.contains("failed to open")
        || combined.contains("cannot access")
        || combined.contains("not a directory")
        || combined.contains("is a directory")
        || combined.contains("cannot stat")
    {
        return (CommandErrorType::PathNotFound, "Check if path exists or use correct location");
    }

    // Timeout patterns
    if combined.contains("timed out")
        || combined.contains("timeout")
        || combined.contains("connection timed out")
        || combined.contains("read timed out")
        || combined.contains("operation timed out")
    {
        return (CommandErrorType::Timeout, "Command took too long - try a simpler query");
    }

    // Syntax error patterns
    if combined.contains("syntax error")
        || combined.contains("invalid option")
        || combined.contains("unknown option")
        || combined.contains("unrecognized option")
        || combined.contains("illegal option")
        || combined.contains("bad flag")
        || combined.contains("missing argument")
        || combined.contains("requires an argument")
        || combined.contains("unexpected token")
        || combined.contains("parse error")
    {
        return (CommandErrorType::SyntaxError, "Fix command syntax or flags");
    }

    // Missing dependency patterns
    if combined.contains("dependency")
        || combined.contains("not installed")
        || combined.contains("package not found")
        || combined.contains("unable to locate package")
        || combined.contains("no package")
        || combined.contains("missing library")
        || combined.contains("cannot load")
        || combined.contains("shared object")
    {
        return (CommandErrorType::MissingDependency, "Install required dependency first");
    }

    // Empty output
    if output.trim().is_empty() {
        return (CommandErrorType::EmptyOutput, "Command produced no output");
    }

    // Additional common error patterns that map to existing types
    if combined.contains("connection refused")
        || combined.contains("network unreachable")
        || combined.contains("host unreachable")
        || combined.contains("name resolution")
    {
        return (CommandErrorType::Unknown, "Network error - check connectivity");
    }

    if combined.contains("disk full")
        || combined.contains("no space left")
        || combined.contains("out of memory")
        || combined.contains("cannot allocate")
    {
        return (CommandErrorType::Unknown, "Resource exhaustion - free up space/memory");
    }

    if combined.contains("device busy")
        || combined.contains("resource busy")
        || combined.contains("cannot unmount")
    {
        return (CommandErrorType::Unknown, "Resource is busy - try closing related apps");
    }

    (CommandErrorType::Unknown, "Unknown error - try alternative command")
}

/// Try to auto-install a missing command
/// v0.0.919: Returns true if installation succeeded and command now exists
pub fn try_auto_install(cmd: &str) -> bool {
    // Extract the base command (first word)
    let base_cmd = cmd.split_whitespace().next().unwrap_or(cmd);

    // Check if auto-install is enabled
    let config = AnnaConfig::load().unwrap_or_default();
    if !config.auto_install_helpers {
        debug!("Auto-install disabled in config");
        return false;
    }

    // Already installed?
    if command_exists(base_cmd) {
        return true;
    }

    // Find the package for this command
    let package = COMMAND_TO_PACKAGE
        .iter()
        .find(|(c, _)| *c == base_cmd)
        .map(|(_, p)| *p);

    let package = match package {
        Some(p) => p,
        None => {
            debug!("No package mapping for command: {}", base_cmd);
            // Try using the command name as package name (works for many tools)
            base_cmd
        }
    };

    info!("Auto-installing package '{}' for command '{}'", package, base_cmd);

    match install_package(package) {
        Ok(true) => {
            info!("Successfully installed package: {}", package);
            // v0.0.921: Clear failure cache since new command is available
            clear_failure_cache();
            true
        }
        Ok(false) => {
            // Already installed (shouldn't happen, but handle gracefully)
            true
        }
        Err(e) => {
            warn!("Failed to install package '{}': {}", package, e);
            false
        }
    }
}

/// Get recovery hint based on error type
pub fn get_recovery_prompt(error_type: &CommandErrorType, cmd: &str) -> String {
    let base_cmd = cmd.split_whitespace().next().unwrap_or(cmd);
    match error_type {
        CommandErrorType::CommandNotFound => format!(
            "Command '{}' not installed. Suggest the Arch package or alternative.",
            base_cmd
        ),
        CommandErrorType::PermissionDenied => format!(
            "Permission denied for '{}'. Suggest sudo or permission fix.",
            cmd
        ),
        CommandErrorType::PathNotFound => format!(
            "Path not found in '{}'. Suggest how to find correct path.",
            cmd
        ),
        CommandErrorType::Timeout => {
            let hint = match base_cmd {
                "find" => "Use 'locate' or add '-maxdepth 2'",
                "grep" | "rg" => "Add 'head -20' to limit output",
                "du" => "Use 'du -d1' or 'df'",
                "journalctl" => "Add '--since \"1 hour ago\"' or '--lines=50'",
                _ => "Try with 'timeout 5s' or limit output",
            };
            format!("Command '{}' timed out. {}.", cmd, hint)
        }
        CommandErrorType::SyntaxError => format!("Syntax error in '{}'. Fix flags/syntax.", cmd),
        CommandErrorType::MissingDependency => format!("Missing dependency for '{}'. Suggest install.", cmd),
        CommandErrorType::EmptyOutput => format!("No output from '{}'. Suggest alternative.", cmd),
        CommandErrorType::Unknown => format!("Command '{}' failed. Suggest alternative.", cmd),
    }
}

/// Record a command failure in memory for future avoidance
/// v0.0.921: Also records to session-level failure cache
pub fn record_command_failure(cmd: &str, error_type: &CommandErrorType) {
    // Record to session-level cache for immediate effect
    record_command_failure_cache(cmd, &format!("{:?}", error_type));

    // Also record to long-term memory
    if let Ok(mut memory) = Memory::load() {
        for exp in memory.experiences.iter_mut() {
            if exp.successful_commands.contains(&cmd.to_string()) {
                exp.context.record_failure(cmd, &format!("{:?}", error_type));
            }
        }
        let _ = memory.save();
        debug!("Recorded command failure: {} ({:?})", cmd, error_type);
    }
}

/// Get alternative commands when the first one fails
pub async fn get_alternative_commands_smart(
    model: &str,
    original_cmd: &str,
    error_output: &str,
    question: &str,
    recovery_hint: &str,
) -> Option<Vec<String>> {
    let fast_timeout = get_perf_config().fast_llm_timeout_secs;

    let prompt = format!(
        r#"Command failed: `{}`
Error: {}
Question: "{}"

DIAGNOSIS: {}

Suggest 1-2 alternative commands for Arch Linux.
Reply with ONLY the commands, one per line. No explanation.
If no alternative exists, reply with "NONE"."#,
        original_cmd,
        if error_output.len() > 200 { &error_output[..200] } else { error_output },
        question,
        recovery_hint
    );

    match ollama::chat_with_timeout(model, &prompt, fast_timeout).await {
        Ok(response) => {
            let response = response.trim();
            if response == "NONE" || response.is_empty() {
                return None;
            }
            let alternatives: Vec<String> = response
                .lines()
                .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
                .map(|l| l.trim().to_string())
                .take(2)
                .collect();
            if alternatives.is_empty() { None } else { Some(alternatives) }
        }
        Err(e) => {
            debug!("Failed to get alternative commands: {}", e);
            None
        }
    }
}

/// Execute a command with retry logic
/// v0.0.919: Added auto-installation for missing commands
pub async fn execute_command_with_retry(
    model: &str,
    cmd: &str,
    question: &str,
    alternatives_budget: &mut u32,
) -> (String, Vec<String>) {
    let mut all_commands = vec![cmd.to_string()];

    info!("Executing command: {}", cmd);
    match execute_command(cmd) {
        Ok(output) if !output.trim().is_empty()
            && !output.contains("command not found")
            && !output.contains("No such file") => {
            debug!("Command succeeded with {} bytes output", output.len());
            return (output, all_commands);
        }
        Ok(output) => {
            let (error_type, hint) = classify_command_error(&output, None);
            if error_type == CommandErrorType::Unknown && !output.trim().is_empty() {
                return (output, all_commands);
            }

            // v0.0.919: Try auto-installing missing command before asking LLM
            if error_type == CommandErrorType::CommandNotFound {
                if try_auto_install(cmd) {
                    // Retry the command after installation
                    if let Ok(retry_output) = execute_command(cmd) {
                        if !retry_output.trim().is_empty() && !retry_output.contains("command not found") {
                            info!("Command succeeded after auto-install");
                            return (retry_output, all_commands);
                        }
                    }
                }
            }
            record_command_failure(cmd, &error_type);

            if *alternatives_budget == 0 {
                return (output, all_commands);
            }
            *alternatives_budget = alternatives_budget.saturating_sub(1);

            let recovery_hint = get_recovery_prompt(&error_type, cmd);
            warn!("Command '{}' failed ({:?}): {}", cmd, error_type, hint);

            if let Some(alternatives) = get_alternative_commands_smart(model, cmd, &output, question, &recovery_hint).await {
                for alt_cmd in alternatives.iter() {
                    if is_dangerous_command(alt_cmd) {
                        continue;
                    }
                    all_commands.push(alt_cmd.clone());
                    if let Ok(alt_output) = execute_command(alt_cmd) {
                        if !alt_output.trim().is_empty() && !alt_output.contains("command not found") {
                            return (alt_output, all_commands);
                        }
                    }
                }
            }
            (output, all_commands)
        }
        Err(e) => {
            let error_msg = format!("Error: {}", e);
            let (error_type, _) = classify_command_error("", Some(&error_msg));
            record_command_failure(cmd, &error_type);

            if *alternatives_budget == 0 {
                return (error_msg, all_commands);
            }
            *alternatives_budget = alternatives_budget.saturating_sub(1);

            let recovery_hint = get_recovery_prompt(&error_type, cmd);
            if let Some(alternatives) = get_alternative_commands_smart(model, cmd, &error_msg, question, &recovery_hint).await {
                for alt_cmd in alternatives.iter() {
                    if is_dangerous_command(alt_cmd) {
                        continue;
                    }
                    all_commands.push(alt_cmd.clone());
                    if let Ok(alt_output) = execute_command(alt_cmd) {
                        if !alt_output.trim().is_empty() {
                            return (alt_output, all_commands);
                        }
                    }
                }
            }
            (error_msg, all_commands)
        }
    }
}

/// Clean prompt artifacts from LLM answers
pub fn clean_answer(answer: &str) -> String {
    let mut result = answer.to_string();
    let artifacts = ["RULES:", "RESPOND IN ENGLISH", "Answer:", "│", "┌", "└", "─"];
    for artifact in artifacts {
        result = result.replace(artifact, "");
    }
    result.lines()
        .filter(|line| {
            let t = line.trim();
            !t.starts_with("1. Answer") && !t.starts_with("2. ONLY")
                && !t.starts_with("3. Do NOT") && !t.starts_with("Question:")
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

/// Verify answer quality (quick check + optional LLM verification)
/// v0.0.929: Enhanced heuristics to reduce LLM verification calls
pub async fn verify_answer_quality(model: &str, question: &str, answer: &str) -> bool {
    let answer_trimmed = answer.trim();
    if answer_trimmed.is_empty() {
        return false;
    }

    let prompt_markers = ["RULES:", "RESPOND IN ENGLISH", "Question:", "│", "┌", "└"];
    for marker in prompt_markers {
        if answer_trimmed.contains(marker) {
            warn!("Answer validation: detected prompt leakage");
            return false;
        }
    }

    let answer_lower = answer_trimmed.to_lowercase();
    let error_markers = ["i cannot", "i don't have access", "as an ai", "as a language model"];
    for marker in error_markers {
        if answer_lower.contains(marker) {
            return false;
        }
    }

    let has_useful_content = answer_trimmed.len() > 10
        && !answer_lower.contains("not found")
        && !answer_lower.contains("command not found");

    // v0.0.929: Increased threshold and added success pattern detection
    if has_useful_content && answer_trimmed.len() < 800 {
        return true;
    }

    // v0.0.929: Heuristic success patterns - skip LLM for obvious good answers
    let question_lower = question.to_lowercase();

    // Factual questions with numeric data in answer
    let is_factual = question_lower.contains("how much")
        || question_lower.contains("how many")
        || question_lower.contains("what is")
        || question_lower.contains("what's")
        || question_lower.contains("disk")
        || question_lower.contains("memory")
        || question_lower.contains("cpu")
        || question_lower.contains("version");

    // Answer contains data patterns (numbers, paths, sizes)
    let has_data_patterns = answer_trimmed.chars().filter(|c| c.is_numeric()).count() > 3
        || answer_trimmed.contains("/dev/")
        || answer_trimmed.contains("/home/")
        || answer_trimmed.contains("/etc/")
        || answer_trimmed.contains(" GB")
        || answer_trimmed.contains(" MB")
        || answer_trimmed.contains(" KB")
        || answer_trimmed.contains("%");

    // Command output indicators (lines with consistent structure)
    let lines: Vec<&str> = answer_trimmed.lines().collect();
    let has_command_output = lines.len() > 2
        && lines.iter().filter(|l| l.contains(':') || l.contains('\t')).count() > lines.len() / 3;

    if is_factual && (has_data_patterns || has_command_output) {
        debug!("Heuristic validation: factual question with data patterns, skipping LLM");
        return true;
    }

    // v0.0.929: Skip LLM if answer has clear structure (lists, bullet points)
    let has_list_structure = answer_trimmed.contains("\n- ")
        || answer_trimmed.contains("\n* ")
        || answer_trimmed.contains("\n1. ")
        || answer_trimmed.contains("\n• ");

    if has_useful_content && has_list_structure {
        debug!("Heuristic validation: structured list answer, skipping LLM");
        return true;
    }

    // v0.0.936: Additional heuristic patterns
    // Service status questions with clear status indicators
    let is_service_question = question_lower.contains("service")
        || question_lower.contains("running")
        || question_lower.contains("status")
        || question_lower.contains("systemd");

    let has_service_indicators = answer_lower.contains("active (running)")
        || answer_lower.contains("inactive")
        || answer_lower.contains("enabled")
        || answer_lower.contains("disabled")
        || answer_lower.contains("loaded")
        || answer_lower.contains("● ");

    if is_service_question && has_service_indicators {
        debug!("Heuristic validation: service status answer, skipping LLM");
        return true;
    }

    // Package/install questions with version numbers
    let is_package_question = question_lower.contains("install")
        || question_lower.contains("package")
        || question_lower.contains("pacman")
        || question_lower.contains("version");

    // Version patterns like "1.2.3" or "v1.0"
    let version_regex_simple = answer_trimmed.contains(" v")
        || regex::Regex::new(r"\d+\.\d+(\.\d+)?").ok().map(|r| r.is_match(answer_trimmed)).unwrap_or(false);

    if is_package_question && version_regex_simple && has_useful_content {
        debug!("Heuristic validation: package/version answer, skipping LLM");
        return true;
    }

    // Network questions with IP/interface data
    let is_network_question = question_lower.contains("ip")
        || question_lower.contains("network")
        || question_lower.contains("interface")
        || question_lower.contains("connection");

    let has_network_data = answer_trimmed.contains("inet ")
        || answer_trimmed.contains("192.168.")
        || answer_trimmed.contains("10.0.")
        || answer_trimmed.contains("127.0.0.1")
        || answer_trimmed.contains("eth0")
        || answer_trimmed.contains("wlan")
        || answer_trimmed.contains("enp")
        || answer_trimmed.contains("wlp");

    if is_network_question && has_network_data {
        debug!("Heuristic validation: network info answer, skipping LLM");
        return true;
    }

    // Hardware questions with clear hw identifiers
    let is_hardware_question = question_lower.contains("gpu")
        || question_lower.contains("graphics")
        || question_lower.contains("cpu")
        || question_lower.contains("hardware")
        || question_lower.contains("pci");

    let has_hardware_data = answer_trimmed.contains("VGA")
        || answer_trimmed.contains("NVIDIA")
        || answer_trimmed.contains("AMD")
        || answer_trimmed.contains("Intel")
        || answer_trimmed.contains("Radeon")
        || answer_trimmed.contains("GeForce")
        || answer_trimmed.contains("model name")
        || answer_trimmed.contains("vendor_id");

    if is_hardware_question && has_hardware_data {
        debug!("Heuristic validation: hardware info answer, skipping LLM");
        return true;
    }

    // LLM verification for longer/questionable answers
    let prompt = format!(
        r#"Question: "{}"
Answer: "{}"

Is this answer helpful and relevant? Reply with only YES or NO."#,
        question,
        if answer_trimmed.len() > 300 { &answer_trimmed[..300] } else { answer_trimmed }
    );

    match ollama::chat_with_timeout(model, &prompt, 10).await {
        Ok(response) => response.trim().to_uppercase().contains("YES"),
        Err(_) => true,
    }
}

/// Execute multiple commands in parallel
pub fn execute_commands_parallel(commands: &[&str]) -> std::collections::HashMap<String, String> {
    use std::collections::HashMap;

    let results: Vec<_> = std::thread::scope(|s| {
        let handles: Vec<_> = commands
            .iter()
            .map(|cmd| {
                let cmd = *cmd;
                s.spawn(move || (cmd.to_string(), execute_command(cmd).ok()))
            })
            .collect();
        handles.into_iter().map(|h| h.join().ok()).collect()
    });

    let mut output = HashMap::new();
    for result in results.into_iter().flatten() {
        if let (cmd, Some(out)) = result {
            output.insert(cmd, out);
        }
    }
    output
}
