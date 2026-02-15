//! Parallel investigation for multi-domain questions.
//!
//! When a user asks about multiple domains (e.g., "check wifi and disk space"),
//! this module runs parallel investigations and synthesizes results.

use anna_shared::agent::{detect_domains, AgentDomain};
use anna_shared::config::AnnaConfig;
use anna_shared::rpc::AskResult;
use anyhow::Result;
use std::collections::HashMap;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

use super::ralph_loop;

/// Result from a parallel domain investigation.
#[derive(Debug)]
pub struct DomainResult {
    pub domain: AgentDomain,
    pub result: AskResult,
}

/// Check if a question should use parallel investigation.
pub fn should_parallelize(question: &str, config: &AnnaConfig) -> Option<Vec<AgentDomain>> {
    if !config.agents.multi_agent_mode || !config.agents.parallel_investigation {
        return None;
    }

    let domains = detect_domains(question);
    if domains.len() > 1 {
        info!("Multi-domain question detected: {:?}", domains);
        Some(domains)
    } else {
        None
    }
}

/// Extract the sub-question for a specific domain from a multi-domain question.
fn extract_domain_question(question: &str, domain: AgentDomain) -> String {
    let _q_lower = question.to_lowercase();

    // Common patterns for multi-domain questions
    let domain_keywords = match domain {
        AgentDomain::Network => vec!["wifi", "network", "internet", "connection", "ethernet", "ip"],
        AgentDomain::Storage => vec!["disk", "storage", "space", "partition", "mount", "filesystem"],
        AgentDomain::System => vec!["cpu", "memory", "ram", "process", "load", "uptime"],
        AgentDomain::Desktop => vec!["screen", "display", "resolution", "monitor", "brightness"],
        AgentDomain::Packages => vec!["package", "update", "install", "pacman", "apt", "yay"],
        AgentDomain::Hardware => vec!["hardware", "device", "usb", "bluetooth", "battery"],
        AgentDomain::Audio => vec!["audio", "sound", "volume", "speaker", "microphone"],
        AgentDomain::Security => vec!["firewall", "security", "permission", "user", "sudo"],
        AgentDomain::General => vec!["status", "check", "info", "help"],
    };

    // Find relevant parts of the question for this domain
    let words: Vec<&str> = question.split(|c: char| c.is_whitespace() || c == ',' || c == ';')
        .filter(|s| !s.is_empty())
        .collect();

    // Find domain-relevant keywords in the question
    let mut relevant_parts = Vec::new();
    let mut found_keyword = false;

    for (i, word) in words.iter().enumerate() {
        let word_lower = word.to_lowercase();
        if domain_keywords.iter().any(|k| word_lower.contains(k)) {
            found_keyword = true;
            // Include context words around the keyword
            let start = i.saturating_sub(2);
            let end = (i + 3).min(words.len());
            for w in &words[start..end] {
                if !relevant_parts.contains(w) && !["and", "or", "also", "check", "my"].contains(&w.to_lowercase().as_str()) {
                    relevant_parts.push(*w);
                }
            }
        }
    }

    if found_keyword && !relevant_parts.is_empty() {
        // Build a focused question for this domain
        format!("check {}", relevant_parts.join(" "))
    } else {
        // Fallback: use domain name in question
        format!("check {} status", domain.as_str())
    }
}

/// Run parallel investigations for multiple domains.
pub async fn run_parallel_investigation(
    model: &str,
    question: &str,
    domains: Vec<AgentDomain>,
    max_parallel: usize,
) -> Result<Vec<DomainResult>> {
    info!("Starting parallel investigation for {} domains", domains.len());

    let mut handles: Vec<(AgentDomain, JoinHandle<Result<AskResult>>)> = Vec::new();

    // Limit parallel tasks
    let domains_to_process: Vec<_> = domains.into_iter().take(max_parallel).collect();

    for domain in domains_to_process {
        let domain_question = extract_domain_question(question, domain);
        let model = model.to_string();

        debug!("Spawning investigation for {:?}: {}", domain, domain_question);

        let handle = tokio::spawn(async move {
            ralph_loop(&model, &domain_question).await
        });

        handles.push((domain, handle));
    }

    // Collect results
    let mut results = Vec::new();
    for (domain, handle) in handles {
        match handle.await {
            Ok(Ok(result)) => {
                debug!("Got result from {:?} domain (success={})", domain, result.success);
                results.push(DomainResult { domain, result });
            }
            Ok(Err(e)) => {
                warn!("Investigation failed for {:?}: {}", domain, e);
            }
            Err(e) => {
                warn!("Task panicked for {:?}: {}", domain, e);
            }
        }
    }

    Ok(results)
}

/// Synthesize results from multiple domain investigations into a unified answer.
pub fn synthesize_parallel_results(
    _original_question: &str,
    results: Vec<DomainResult>,
) -> AskResult {
    if results.is_empty() {
        return AskResult {
            answer: "I couldn't gather information from any domain.".to_string(),
            success: false,
            iterations: 0,
            commands_executed: vec![],
            dialogue: vec![],
            needs_clarification: true,
            clarification_question: Some("Could you ask about one thing at a time?".to_string()),
            cached: false,
            citations: vec![],
            abstained: false,
            final_confidence: Some(0.0),
        };
    }

    // Build combined answer
    let mut answer_parts = Vec::new();
    let mut all_commands = Vec::new();
    let mut all_dialogue = Vec::new();
    let mut total_iterations = 0;
    let mut total_confidence = 0.0;
    let mut success_count = 0;

    for dr in &results {
        let answer = dr.result.answer.trim();
        if !answer.is_empty() {
            answer_parts.push(answer.to_string());
        }

        // Collect commands and dialogue
        all_commands.extend(dr.result.commands_executed.clone());
        all_dialogue.extend(dr.result.dialogue.clone());
        total_iterations += dr.result.iterations;

        if dr.result.success {
            success_count += 1;
        }

        if let Some(conf) = dr.result.final_confidence {
            total_confidence += conf;
        }
    }

    let combined_answer = answer_parts.join("\n\n");
    let avg_confidence = if !results.is_empty() {
        total_confidence / results.len() as f32
    } else {
        0.0
    };

    AskResult {
        answer: combined_answer,
        success: success_count == results.len(),
        iterations: total_iterations,
        commands_executed: all_commands,
        dialogue: all_dialogue,
        needs_clarification: false,
        clarification_question: None,
        cached: false,
        citations: vec![],
        abstained: false,
        final_confidence: Some(avg_confidence),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain_question_network() {
        let q = extract_domain_question("check my wifi and disk space", AgentDomain::Network);
        assert!(q.contains("wifi") || q.contains("network"));
    }

    #[test]
    fn test_extract_domain_question_storage() {
        let q = extract_domain_question("check my wifi and disk space", AgentDomain::Storage);
        assert!(q.contains("disk") || q.contains("storage") || q.contains("space"));
    }

    #[test]
    fn test_should_parallelize_disabled() {
        let mut config = AnnaConfig::default();
        config.agents.multi_agent_mode = false;

        let result = should_parallelize("check wifi and disk", &config);
        assert!(result.is_none());
    }
}
