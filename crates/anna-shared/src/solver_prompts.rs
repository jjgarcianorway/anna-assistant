//! Evidence-focused solver prompts (v0.0.408).
//!
//! Generates prompts that enforce research-first behavior:
//! - Evidence from probes and docs is ground truth
//! - No inventing commands or options
//! - Explicit "cannot answer" when uncertain
//! - All claims must reference evidence by ID

use crate::knowledge_item::KnowledgeItem;
use crate::rpc::ProbeResult;

/// Build a solver prompt that enforces evidence-based answers
pub fn build_solver_prompt(
    question: &str,
    domain: &str,
    intent: &str,
    probes: &[ProbeResult],
    knowledge: &[KnowledgeItem],
) -> String {
    let mut prompt = String::new();

    // System instructions
    prompt.push_str(EVIDENCE_INSTRUCTIONS);
    prompt.push_str("\n\n");

    // Domain/intent context
    prompt.push_str(&format!("Domain: {}\nIntent: {}\n\n", domain, intent));

    // User question
    prompt.push_str(&format!("User question: {}\n\n", question));

    // Probe outputs (numbered for reference)
    if !probes.is_empty() {
        prompt.push_str("=== PROBE OUTPUTS ===\n");
        for (i, probe) in probes.iter().enumerate() {
            let probe_id = format!("probe_{}", i);
            prompt.push_str(&format!(
                "--- {} (exit={}) ---\n$ {}\n{}\n\n",
                probe_id,
                probe.exit_code,
                probe.command,
                truncate(&probe.stdout, 1000)
            ));
        }
    }

    // Knowledge items (with IDs for reference)
    if !knowledge.is_empty() {
        prompt.push_str("=== DOCUMENTATION ===\n");
        for item in knowledge {
            prompt.push_str(&item.format_for_solver());
            prompt.push('\n');
        }
    }

    // Output format reminder
    prompt.push_str(OUTPUT_FORMAT);

    prompt
}

/// Instructions that enforce evidence-based answers
const EVIDENCE_INSTRUCTIONS: &str = r#"You are a system specialist answering technical questions.

CRITICAL RULES - FOLLOW EXACTLY:

1. EVIDENCE IS GROUND TRUTH
   - Use ONLY the probe outputs and documentation provided below
   - Do NOT invent commands, options, or configuration syntax
   - Every claim must be backed by evidence from the provided data

2. REFERENCE YOUR SOURCES
   - For every statement, cite the source by ID (e.g., "probe_0", "k1234...")
   - List all referenced IDs in the evidence_references array
   - If you cannot find evidence for a claim, do not make it

3. WHEN YOU CANNOT ANSWER
   - If probes show insufficient data: set can_answer=false
   - If no relevant documentation was found: set can_answer=false
   - Always suggest what the user can check manually

4. BE SPECIFIC AND TECHNICAL
   - Reference exact lines, values, and file paths from evidence
   - Use the terminology from the documentation
   - Do not rephrase or paraphrase when exact quotes are better

NEGATIVE EXAMPLES - DO NOT DO THIS:

X BAD: "unknown is installed"
   - NEVER use "unknown" as a package/tool name
   - If you cannot determine a name, say so explicitly

X BAD: Inventing probe output
   - Do NOT say "nano is at /usr/bin/nano" unless a probe shows that

X BAD: Empty evidence with can_answer=true
   - If can_answer=true, evidence array MUST have entries

X BAD: confidence > 0.7 with no documentation
   - High confidence requires strong evidence from probes AND docs

X BAD: Text outside the JSON object
   - Output ONLY { ... }, nothing before or after"#;

/// Output format specification
const OUTPUT_FORMAT: &str = r#"

=== OUTPUT FORMAT ===
Respond with ONLY a JSON object:
{
  "ticket_id": "<same as input>",
  "status": "ok" | "needs_more_data" | "cannot_answer" | "no_evidence",
  "can_answer": true | false,
  "answer": {
    "short": "Direct answer (1-2 sentences)",
    "detail": "Optional longer explanation"
  },
  "evidence": [
    {
      "probe": "probe_id or knowledge_id",
      "snippet": "Exact quote from the evidence",
      "interpretation": "What this means for the answer"
    }
  ],
  "evidence_references": ["probe_0", "k1234..."],
  "knowledge_used": ["man systemctl", "Arch Wiki: Systemd"],
  "confidence": 0.0-1.0,
  "next_steps": {
    "user_actions": [{"id": "...", "description": "..."}]
  }
}

If can_answer=false, answer.short should explain why and suggest manual steps."#;

/// Build a prompt for when no knowledge was found
pub fn build_no_knowledge_prompt(question: &str, domain: &str, probes: &[ProbeResult]) -> String {
    let mut prompt = String::new();

    prompt.push_str("You are a system specialist. No relevant documentation was found.\n\n");
    prompt.push_str(&format!("Domain: {}\n", domain));
    prompt.push_str(&format!("Question: {}\n\n", question));

    if !probes.is_empty() {
        prompt.push_str("=== PROBE OUTPUTS ===\n");
        for (i, probe) in probes.iter().enumerate() {
            prompt.push_str(&format!(
                "--- probe_{} ---\n$ {}\n{}\n\n",
                i,
                probe.command,
                truncate(&probe.stdout, 500)
            ));
        }
    }

    prompt.push_str(
        r#"
NO DOCUMENTATION FOUND.

You MUST:
1. Set can_answer=false
2. Set status="no_evidence"
3. Explain that you lack local documentation
4. Suggest specific commands the user can run to investigate
5. Optionally suggest relevant man pages or Arch Wiki links

Response format:
{
  "ticket_id": "...",
  "status": "no_evidence",
  "can_answer": false,
  "answer": {
    "short": "I cannot safely answer this from local data.",
    "detail": "No relevant documentation found for this query."
  },
  "evidence": [],
  "evidence_references": [],
  "knowledge_used": [],
  "confidence": 0.0,
  "next_steps": {
    "user_actions": [
      {"id": "check_man", "description": "Try: man <topic>"},
      {"id": "check_wiki", "description": "See: https://wiki.archlinux.org/title/..."}
    ]
  }
}"#,
    );

    prompt
}

/// Suggest manual commands based on domain
pub fn suggest_manual_commands(domain: &str, keywords: &[String]) -> Vec<String> {
    let mut suggestions = vec![];

    match domain.to_lowercase().as_str() {
        "services" | "systemd" => {
            suggestions.push("systemctl status <service>".to_string());
            suggestions.push("journalctl -u <service> -n 50".to_string());
            suggestions.push("man systemd.service".to_string());
        }
        "packages" => {
            suggestions.push("pacman -Qi <package>".to_string());
            suggestions.push("pacman -Ql <package>".to_string());
            suggestions.push("man pacman".to_string());
        }
        "storage" => {
            suggestions.push("df -h".to_string());
            suggestions.push("lsblk".to_string());
            suggestions.push("man fstab".to_string());
        }
        "network" => {
            suggestions.push("ip addr".to_string());
            suggestions.push("nmcli device status".to_string());
            suggestions.push("man networkmanager".to_string());
        }
        "audio" => {
            suggestions.push("pactl list sinks".to_string());
            suggestions.push("wpctl status".to_string());
            suggestions.push("man pipewire".to_string());
        }
        _ => {
            // Generic suggestions
            if let Some(kw) = keywords.first() {
                suggestions.push(format!("man {}", kw));
                suggestions.push(format!("{} --help", kw));
            }
        }
    }

    // Add Arch Wiki suggestion if we have keywords
    if let Some(kw) = keywords.first() {
        let slug = kw.replace(' ', "_");
        suggestions.push(format!(
            "Arch Wiki: https://wiki.archlinux.org/title/{}",
            slug
        ));
    }

    suggestions
}

/// Truncate text to max length
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...[truncated]", &s[..max.saturating_sub(15)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_solver_prompt() {
        let probes = vec![ProbeResult {
            command: "df -h".to_string(),
            stdout: "/dev/sda1 100G 75G 25G 75% /".to_string(),
            stderr: String::new(),
            exit_code: 0,
            timing_ms: 50,
        }];

        let knowledge = vec![KnowledgeItem::new(
            crate::knowledge_item::KnowledgeSourceType::ManPage,
            "man df",
            "Show disk space usage",
        )];

        let prompt = build_solver_prompt(
            "How much disk space do I have?",
            "storage",
            "query",
            &probes,
            &knowledge,
        );

        assert!(prompt.contains("EVIDENCE IS GROUND TRUTH"));
        assert!(prompt.contains("df -h"));
        assert!(prompt.contains("man df"));
    }

    #[test]
    fn test_suggest_manual_commands() {
        let suggestions = suggest_manual_commands("services", &["sshd".to_string()]);

        assert!(suggestions.iter().any(|s| s.contains("systemctl")));
        assert!(suggestions.iter().any(|s| s.contains("journalctl")));
    }

    #[test]
    fn test_no_knowledge_prompt() {
        let probes = vec![];
        let prompt = build_no_knowledge_prompt("test", "system", &probes);

        assert!(prompt.contains("NO DOCUMENTATION FOUND"));
        assert!(prompt.contains("can_answer=false"));
    }
}
