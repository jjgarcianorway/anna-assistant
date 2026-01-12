//! LLM prompts for the pure intelligence core loop.
//!
//! These prompts are designed to be:
//! - Clear and unambiguous
//! - Grounded in actual system information
//! - Focused on investigation rather than assumptions

use super::InvestigationState;

/// System context that goes into every prompt
pub fn system_context() -> String {
    // Gather real system info
    let distro = std::fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("PRETTY_NAME="))
                .map(|l| l.trim_start_matches("PRETTY_NAME=").trim_matches('"').to_string())
        })
        .unwrap_or_else(|| "Arch Linux".to_string());

    let kernel = std::process::Command::new("uname")
        .arg("-r")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    format!(
        "You are Anna, an intelligent Linux system assistant.\n\
         System: {} (kernel {})\n\
         Package manager: pacman (Arch-based)\n\
         You help users by investigating their system and providing grounded answers.",
        distro, kernel
    )
}

/// Prompt to understand what the user is asking
pub fn understanding_prompt(question: &str) -> String {
    format!(
        r#"{context}

USER QUESTION: "{question}"

Analyze this question. Respond with:

INTENT: <one of: info, diagnostic, howto, fix, config>
NEED: <what information is needed to answer - one per line>

If this question is NOT about Linux/system administration, respond with:
OUT_OF_SCOPE: <brief explanation why>

Examples:
- "how much disk space" -> INTENT: info, NEED: disk usage statistics
- "why is my fan loud" -> INTENT: diagnostic, NEED: CPU usage, temperature, running processes
- "install docker" -> INTENT: howto, NEED: none (can answer from knowledge)
- "fix pacman lock" -> INTENT: fix, NEED: check if lock file exists, check running pacman

Respond now:"#,
        context = system_context(),
        question = question
    )
}

/// Prompt to decide what investigation steps to take next
pub fn next_step_prompt(question: &str, state: &InvestigationState) -> String {
    let findings_text = if state.findings.is_empty() {
        "No commands executed yet.".to_string()
    } else {
        state.findings.iter()
            .map(|f| {
                let status = if f.success { "OK" } else { "FAILED" };
                let output = if f.output.len() > 1000 {
                    format!("{}...(truncated)", &f.output[..1000])
                } else {
                    f.output.clone()
                };
                format!("$ {} [{}]\n{}", f.command, status, output)
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    };

    format!(
        r#"{context}

USER QUESTION: "{question}"

INVESTIGATION SO FAR (iteration {iteration}):
{findings}

Based on the findings above, decide what to do next.

IMPORTANT RULES:
1. DO NOT use interactive commands (top, htop, vim, nano, less, man)
2. Use non-interactive alternatives: ps aux, cat, head, tail
3. For CPU: ps aux --sort=-%cpu | head -10
4. For memory: free -h
5. For disk: df -h, du -sh /path
6. Maximum 3-5 commands per iteration
7. If you found a PROBLEM that can be fixed, suggest the fix

RESPOND WITH ONE OF:

Option 1 - Need more information:
COMMANDS:
command1
command2
command3

Option 2 - Have enough info to answer:
ANSWER

Option 3 - Found a fixable problem:
FIX: <the command to fix it>
PROBLEM: <what the problem is>
EXPLAIN: <why this fix works>

Option 4 - Question is out of scope:
OUT_OF_SCOPE: <reason>

Respond now:"#,
        context = system_context(),
        question = question,
        iteration = state.iteration,
        findings = findings_text
    )
}

/// Prompt to generate the final grounded answer
pub fn answer_prompt(question: &str, state: &InvestigationState) -> String {
    let findings_text = state.findings.iter()
        .map(|f| {
            let status = if f.success { "OK" } else { "FAILED" };
            format!("$ {} [{}]\n{}", f.command, status, f.output)
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    format!(
        r#"{context}

USER QUESTION: "{question}"

INVESTIGATION RESULTS:
{findings}

Based on the command outputs above, provide a helpful answer.

RULES:
1. Your answer MUST be grounded in the actual command output shown above
2. Do NOT invent information that isn't in the output
3. Be concise but complete
4. If the output shows "no results" or empty, that IS an answer (e.g., "no failing services")
5. If suggesting commands for the user to run, use pacman (not apt, brew, etc.)
6. Format numbers and paths clearly

Provide your answer now:"#,
        context = system_context(),
        question = question,
        findings = findings_text
    )
}
