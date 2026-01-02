//! Specialist consultation - handles queries that don't match existing recipes.

use std::collections::HashMap;
use tracing::warn;

use crate::ollama;
use crate::probes;
use super::types::{ParsedQuery, SpecialistSolution};

/// Gather evidence by running probes
pub async fn gather_evidence(probe_cmds: &[String]) -> HashMap<String, String> {
    let mut evidence = HashMap::new();

    for probe in probe_cmds {
        match probes::run_command(probe) {
            Ok(output) => {
                evidence.insert(probe.clone(), output);
            }
            Err(e) => {
                warn!("Evidence probe failed: {} - {}", probe, e);
            }
        }
    }

    evidence
}

/// Ask specialist to solve the problem
pub async fn ask_specialist(
    model: &str,
    query: &str,
    parsed: &ParsedQuery,
    evidence: &HashMap<String, String>,
    knowledge: &[anna_shared::evidence_engine::DocSnippet],
    specialist_name: &str,
) -> SpecialistSolution {
    let evidence_str = evidence
        .iter()
        .map(|(k, v)| format!("## {}\n```\n{}\n```", k, v))
        .collect::<Vec<_>>()
        .join("\n\n");

    // Format knowledge sources
    let knowledge_str = if knowledge.is_empty() {
        String::new()
    } else {
        let sections: Vec<String> = knowledge
            .iter()
            .map(|k| format!("## {} ({})\n{}", k.title, k.source, k.snippet))
            .collect();
        format!("\n\nKnowledge Sources:\n{}", sections.join("\n\n"))
    };

    let prompt = format!(
        r#"You are {}, a Linux {} specialist. Answer this query using the evidence and knowledge provided.

Query: "{}"

Evidence:
{}{}

IMPORTANT: If you use information from Knowledge Sources, cite them in your answer.
Example: "According to the Arch Wiki, ..." or "The man page states..."

Provide a clear, direct answer to the user's question.
Then respond with JSON containing your answer and confidence:
{{"answer": "your answer here", "confidence": 0.9, "explanation": "brief reasoning"}}"#,
        specialist_name, parsed.domain, query, evidence_str, knowledge_str
    );

    let response = match ollama::chat_with_timeout(model, &prompt, 30).await {
        Ok(r) => r,
        Err(e) => {
            return SpecialistSolution {
                answer: format!("Specialist error: {}", e),
                confidence: 0.0,
                explanation: "Failed to get specialist response".to_string(),
            };
        }
    };

    // Try to parse JSON from response
    let json: serde_json::Value = serde_json::from_str(&response)
        .or_else(|_| {
            if let Some(start) = response.find('{') {
                if let Some(end) = response.rfind('}') {
                    return serde_json::from_str(&response[start..=end]);
                }
            }
            Err(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "No JSON found",
            )))
        })
        .unwrap_or_else(|_| {
            // If JSON parsing fails, use the response as-is
            serde_json::json!({
                "answer": response,
                "confidence": 0.6,
                "explanation": "Raw response (JSON parsing failed)"
            })
        });

    SpecialistSolution {
        answer: json["answer"].as_str().unwrap_or(&response).to_string(),
        confidence: json["confidence"].as_f64().unwrap_or(0.6) as f32,
        explanation: json["explanation"].as_str().unwrap_or("").to_string(),
    }
}

/// v0.0.816: Check if domain produces dynamic (changing) results
/// These queries should NOT be learned as recipes because the answer changes
pub fn is_dynamic_domain(domain: &str) -> bool {
    matches!(
        domain.to_lowercase().as_str(),
        "storage" | "memory" | "performance" | "system" | "processes"
    )
}

/// v0.0.816: Check if probe produces dynamic (changing) results
pub fn is_dynamic_probe(probe: &str) -> bool {
    let dynamic_probes = [
        "largest_dirs", "largest_home", "disk_usage", "df",
        "free", "memory_info", "top_memory", "top_cpu",
        "ps", "uptime", "load_average", "who",
        "running_services", "failed_services",
        "network_stats", "listening_ports",
    ];

    dynamic_probes.iter().any(|p| probe.contains(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_dynamic_domain() {
        assert!(is_dynamic_domain("storage"));
        assert!(is_dynamic_domain("memory"));
        assert!(is_dynamic_domain("SYSTEM"));
        assert!(!is_dynamic_domain("network"));
        assert!(!is_dynamic_domain("desktop"));
    }

    #[test]
    fn test_is_dynamic_probe() {
        assert!(is_dynamic_probe("df -h"));
        assert!(is_dynamic_probe("free -m"));
        assert!(is_dynamic_probe("ps aux"));
        assert!(!is_dynamic_probe("uname -r"));
        assert!(!is_dynamic_probe("cat /etc/hostname"));
    }
}
