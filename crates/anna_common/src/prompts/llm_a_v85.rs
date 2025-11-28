//! v0.85.0 Junior Prompt - Ultra-Compact, Decisive, Minimal
//!
//! Target: <2KB total prompt size
//! No prose, no explanations, only strict JSON

/// v0.85.0 Junior System Prompt - Flattened, minimal
pub const LLM_A_SYSTEM_PROMPT_V85: &str = r#"JUNIOR v85. JSON only. No prose.

TASK: Propose ONE safe command OR provide answer.

OUTPUT (strict JSON):
{"action":"command"|"answer"|"refuse","command":"..."|null,"answer":"..."|null,"score":0-100,"probes":["id1"]}

RULES:
1. Safe commands only: ls,cat,head,lscpu,free,df,ip,uname,systemctl status
2. No rm,mv,chmod,kill,reboot,shutdown
3. Score 0-100 confidence
4. Refuse if dangerous or unsupported

PROBES: cpu.info,mem.info,disk.lsblk,net.ip,sys.uname,service.status,fs.ls,log.journalctl"#;

/// Generate v0.85.0 Junior prompt
pub fn generate_junior_prompt_v85(question: &str, evidence: &str, brain_hint: Option<&str>) -> String {
    let mut prompt = format!("Q:{}\n", question);

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
        assert!(LLM_A_SYSTEM_PROMPT_V85.contains("command"));
    }

    #[test]
    fn test_generate_with_brain_hint() {
        let prompt = generate_junior_prompt_v85(
            "How many CPU cores?",
            "",
            Some("cmd:lscpu,score:0.95")
        );
        assert!(prompt.contains("BRAIN:"));
        assert!(prompt.contains("lscpu"));
    }
}
