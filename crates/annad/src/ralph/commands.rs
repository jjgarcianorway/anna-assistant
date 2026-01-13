//! Command execution and answer generation for the Ralph loop.

use anyhow::Result;

use crate::ollama;
use super::criteria::{AnswerType, CompletionCriteria, IterationState, SelfEvaluation, quick_quality_check};

/// Get commands to run for answering the question
pub async fn get_commands(
    model: &str,
    question: &str,
    state: &IterationState,
) -> Result<Vec<String>> {
    let feedback_context = if let Some(ref feedback) = state.feedback {
        format!(
            "\n\nPrevious attempt feedback: {}\nAlready tried: {:?}",
            feedback, state.commands
        )
    } else {
        String::new()
    };

    let output_context = if !state.outputs.is_empty() {
        format!(
            "\n\nData collected so far:\n{}",
            state.outputs.join("\n---\n")
        )
    } else {
        String::new()
    };

    let prompt = format!(
        r#"System: Arch Linux with pacman

Question: "{}"{}{}

Return 1-3 bash commands to answer this question. Use these exact commands:

SYSTEM: uname -r, uptime -p, hostnamectl
HARDWARE: lscpu | head -20, free -h, lsusb, lspci | head -20
DESKTOP: echo $XDG_CURRENT_DESKTOP, echo $XDG_SESSION_TYPE
USER: id, groups, echo $SHELL, locale, timedatectl | grep "Time zone"
STORAGE: df -h, lsblk, findmnt / -o OPTIONS, swapon --show
NETWORK: ip -4 addr show, cat /etc/resolv.conf, ip route | grep default, ss -tlnp | head -15
SERVICES: systemctl --failed, systemctl list-units --type=service --state=running | head -20
PACKAGES: pacman -Q | wc -l, pacman -Qe | head -30, pacman -Qtdq
LOGS: journalctl -p err -b --no-pager | head -30

RULES:
- Output ONLY valid bash commands, one per line
- NO explanations, NO English text, NO comments
- If question already answered by data below, output: DONE
- If question needs no commands (how-to), output: NONE

Output commands now:"#,
        question, output_context, feedback_context
    );

    let response = ollama::chat_with_timeout(model, &prompt, 30).await?;
    let response = response.trim();

    // Check for special responses (case-insensitive)
    let response_upper = response.to_uppercase();
    if response_upper == "NONE" || response_upper == "DONE" || response.is_empty() {
        return Ok(Vec::new());
    }

    let commands: Vec<String> = response
        .lines()
        .map(|l| l.trim())
        .filter(|l| {
            if l.is_empty() || l.starts_with('#') {
                return false;
            }
            // Filter out DONE/NONE even if mixed with other commands
            let upper = l.to_uppercase();
            if upper == "DONE" || upper == "NONE" || upper.starts_with("DONE:") {
                return false;
            }
            true
        })
        .map(|l| l.to_string())
        .take(5) // Max 5 commands per iteration
        .collect();

    Ok(commands)
}

/// Generate an answer based on collected data
pub async fn generate_answer(
    model: &str,
    question: &str,
    state: &IterationState,
    criteria: &CompletionCriteria,
) -> Result<String> {
    let data_context = if state.outputs.is_empty() {
        "No command output available.".to_string()
    } else {
        state.outputs.join("\n---\n")
    };

    let grounding_instruction = if criteria.requires_grounding {
        "Base your answer ONLY on the data above. Do not make up information."
    } else {
        "You may provide general guidance based on your knowledge."
    };

    let prompt = format!(
        r#"You are Anna, an AI assistant for Arch Linux systems.
This is an Arch Linux system using pacman for packages.
Do NOT suggest apt, brew, or other package managers.

Question: {}

Data collected:
{}

{}

Provide a clear, helpful answer. Be concise but complete."#,
        question, data_context, grounding_instruction
    );

    let answer = ollama::chat_with_timeout(model, &prompt, 60).await?;
    Ok(answer.trim().to_string())
}

/// Self-evaluate the answer - is it good enough?
pub async fn self_evaluate(
    model: &str,
    question: &str,
    answer: &str,
    state: &IterationState,
    criteria: &CompletionCriteria,
) -> Result<SelfEvaluation> {
    // Quick heuristic checks first
    if answer.len() < 20 {
        return Ok(SelfEvaluation {
            is_complete: false,
            confidence: 0.2,
            missing: Some("Answer too short".to_string()),
            suggestions: Some("Provide more detail".to_string()),
        });
    }

    // Check quality heuristics
    if !quick_quality_check(answer) {
        return Ok(SelfEvaluation {
            is_complete: false,
            confidence: 0.3,
            missing: Some("Answer quality check failed".to_string()),
            suggestions: Some("Regenerate with better grounding".to_string()),
        });
    }

    // For simple/HowTo questions, skip LLM evaluation
    if matches!(criteria.answer_type, AnswerType::Simple | AnswerType::HowTo) {
        return Ok(SelfEvaluation {
            is_complete: true,
            confidence: 0.8,
            missing: None,
            suggestions: None,
        });
    }

    // LLM self-evaluation for complex questions
    let data_summary = if state.outputs.is_empty() {
        "No data collected".to_string()
    } else {
        format!("{} command outputs collected", state.outputs.len())
    };

    let prompt = format!(
        r#"Evaluate this answer:

Question: {}
Answer: {}
Data: {}

Rate on these criteria:
1. Does it directly answer the question? (YES/NO)
2. Is it grounded in the data collected? (YES/NO/NA)
3. Is anything important missing? (describe or NONE)

Format: COMPLETE/INCOMPLETE, CONFIDENCE (0-100), MISSING: <text>"#,
        question, answer, data_summary
    );

    let response = ollama::chat_with_timeout(model, &prompt, 20).await?;
    let response = response.to_uppercase();

    // Parse response
    let is_complete = response.contains("COMPLETE") && !response.contains("INCOMPLETE");

    let confidence = if let Some(conf_match) = response
        .split_whitespace()
        .find(|w| w.parse::<f32>().is_ok())
    {
        conf_match.parse::<f32>().unwrap_or(50.0) / 100.0
    } else if is_complete {
        0.8
    } else {
        0.4
    };

    let missing = if response.contains("MISSING:") {
        response
            .split("MISSING:")
            .nth(1)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && s != "NONE")
    } else {
        None
    };

    Ok(SelfEvaluation {
        is_complete,
        confidence: confidence.clamp(0.0, 1.0),
        missing: missing.clone(),
        suggestions: missing,
    })
}
