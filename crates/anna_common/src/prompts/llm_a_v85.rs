//! v0.85.0 Junior Prompt - Ultra-Compact, Decisive, Minimal
//!
//! Target: <2KB total prompt size
//! No prose, no explanations, only strict JSON

/// v0.88.0 Junior System Prompt - Flattened, minimal
/// NOTE: Probe list is passed dynamically via user prompt, NOT hardcoded here
pub const LLM_A_SYSTEM_PROMPT_V85: &str = r#"JUNIOR v88. JSON only. No prose.

TASK: Request probes from the list OR provide answer.

OUTPUT (strict JSON):
{"action":"probe"|"answer"|"refuse","probes":["probe.id"],"answer":"..."|null,"score":0-100}

RULES:
1. Only request probes from the PROBES list in user message
2. Score 0-100 confidence
3. Refuse if dangerous or unsupported
4. If evidence provided, give answer with score>=80"#;

/// v0.88.0: Generate Junior prompt with dynamic probe list
pub fn generate_junior_prompt_v85(
    question: &str,
    probes: &[String],
    evidence: &str,
    brain_hint: Option<&str>
) -> String {
    let mut prompt = format!("PROBES:{}\n", probes.join(","));
    prompt.push_str(&format!("Q:{}\n", question));

    if let Some(hint) = brain_hint {
        prompt.push_str(&format!("BRAIN:{}\n", hint));
    }

    if !evidence.is_empty() {
        // Truncate evidence to keep prompt small
        let ev = if evidence.len() > 800 {
            format!("{}...", &evidence[..800])
        } else {
            evidence.to_string()
        };
        prompt.push_str(&format!("EV:{}\n", ev));
    }

    prompt.push_str("JSON:");
    prompt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v85_prompt_under_2kb() {
        assert!(LLM_A_SYSTEM_PROMPT_V85.len() < 2048);
    }

    #[test]
    fn test_v85_prompt_has_json() {
        assert!(LLM_A_SYSTEM_PROMPT_V85.contains("JSON"));
        assert!(LLM_A_SYSTEM_PROMPT_V85.contains("probe"));
    }

    #[test]
    fn test_generate_with_probes() {
        let probes = vec!["cpu.info".to_string(), "mem.info".to_string(), "logs.annad".to_string()];
        let prompt = generate_junior_prompt_v85(
            "How many CPU cores?",
            &probes,
            "",
            None
        );
        assert!(prompt.contains("PROBES:cpu.info,mem.info,logs.annad"));
        assert!(prompt.contains("Q:How many CPU cores?"));
    }

    #[test]
    fn test_generate_with_brain_hint() {
        let probes = vec!["cpu.info".to_string()];
        let prompt = generate_junior_prompt_v85(
            "How many CPU cores?",
            &probes,
            "",
            Some("cmd:lscpu,score:0.95")
        );
        assert!(prompt.contains("BRAIN:"));
        assert!(prompt.contains("lscpu"));
        assert!(prompt.contains("PROBES:cpu.info"));
    }
}
