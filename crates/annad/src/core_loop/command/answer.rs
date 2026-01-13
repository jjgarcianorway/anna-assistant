//! Answer cleaning and quality verification.

use tracing::{debug, warn};

use crate::core_loop::cache::get_perf_config;
use crate::ollama;

/// Clean prompt artifacts from LLM answers
pub fn clean_answer(answer: &str) -> String {
    let mut result = answer.to_string();
    let artifacts = [
        "RULES:",
        "RESPOND IN ENGLISH",
        "Answer:",
        "│",
        "┌",
        "└",
        "─",
    ];
    for artifact in artifacts {
        result = result.replace(artifact, "");
    }
    result
        .lines()
        .filter(|line| {
            let t = line.trim();
            !t.starts_with("1. Answer")
                && !t.starts_with("2. ONLY")
                && !t.starts_with("3. Do NOT")
                && !t.starts_with("Question:")
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

    let prompt_markers = [
        "RULES:",
        "RESPOND IN ENGLISH",
        "Question:",
        "│",
        "┌",
        "└",
    ];
    for marker in prompt_markers {
        if answer_trimmed.contains(marker) {
            warn!("Answer validation: detected prompt leakage");
            return false;
        }
    }

    let answer_lower = answer_trimmed.to_lowercase();
    let error_markers = [
        "i cannot",
        "i don't have access",
        "as an ai",
        "as a language model",
    ];
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
        && lines
            .iter()
            .filter(|l| l.contains(':') || l.contains('\t'))
            .count()
            > lines.len() / 3;

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
        || regex::Regex::new(r"\d+\.\d+(\.\d+)?")
            .ok()
            .map(|r| r.is_match(answer_trimmed))
            .unwrap_or(false);

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
        if answer_trimmed.len() > 300 {
            &answer_trimmed[..300]
        } else {
            answer_trimmed
        }
    );

    let fast_timeout = get_perf_config().fast_llm_timeout_secs;
    match ollama::chat_with_timeout(model, &prompt, fast_timeout).await {
        Ok(response) => response.trim().to_uppercase().contains("YES"),
        Err(_) => true,
    }
}
