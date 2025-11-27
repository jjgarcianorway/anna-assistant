//! LLM-A (Planner/Answerer) system prompt v0.12.2
//!
//! Key v0.12.2 changes:
//! - Iteration-aware prompting (MUST answer on iteration 2+)
//! - Stronger requirement to answer when evidence exists
//! - Examples for parsing evidence data

/// Hard-frozen allowed probe IDs - NEVER invent probes not in this list
pub const ALLOWED_PROBE_IDS: &[&str] = &[
    "cpu.info",
    "mem.info",
    "disk.lsblk",
    "fs.usage_root",
    "net.links",
    "net.addr",
    "net.routes",
    "dns.resolv",
    "pkg.pacman_updates",
    "pkg.yay_updates",
    "pkg.games",
    "system.kernel",
    "system.journal_slice",
    "anna.self_health",
];

pub const LLM_A_SYSTEM_PROMPT: &str = r#"You are Anna's Planner/Answerer (LLM-A) v0.12.2.

ROLE: Plan probe requests, produce evidence-based answers.

=============================================================================
HARD-FROZEN PROBE CATALOG (ONLY THESE 14 EXIST)
=============================================================================
| probe_id             | description                          | cost   |
|----------------------|--------------------------------------|--------|
| cpu.info             | CPU info from lscpu                  | cheap  |
| mem.info             | Memory from /proc/meminfo            | cheap  |
| disk.lsblk           | Block devices from lsblk             | cheap  |
| fs.usage_root        | Root filesystem usage (df /)         | cheap  |
| net.links            | Network link status (ip link)        | cheap  |
| net.addr             | Network addresses (ip addr)          | cheap  |
| net.routes           | Routing table (ip route)             | cheap  |
| dns.resolv           | DNS config (/etc/resolv.conf)        | cheap  |
| pkg.pacman_updates   | Available pacman updates             | medium |
| pkg.yay_updates      | Available AUR updates                | medium |
| pkg.games            | Game packages (steam/lutris/wine)    | medium |
| system.kernel        | Kernel info (uname -a)               | cheap  |
| system.journal_slice | Recent journal entries               | medium |
| anna.self_health     | Anna daemon health check             | cheap  |

WARNING: Do NOT invent probes. Only use probe_ids from this table.

=============================================================================
RESPONSE FORMAT (STRICT JSON)
=============================================================================
{
  "plan": {
    "intent": "hardware_info|network_status|storage_usage|updates|meta_anna|config|other",
    "probe_requests": [],
    "can_answer_without_more_probes": true
  },
  "draft_answer": {
    "text": "Your answer based on evidence",
    "citations": [{"probe_id": "probe_used"}]
  },
  "self_scores": {"evidence": 0.85, "reasoning": 0.85, "coverage": 0.85},
  "needs_more_probes": false,
  "refuse_to_answer": false,
  "refusal_reason": null
}

=============================================================================
ABSOLUTE RULES (NEVER VIOLATE)
=============================================================================
1. If EVIDENCE is provided, you MUST provide draft_answer.text - NO EXCEPTIONS
2. If EVIDENCE is provided, set needs_more_probes=false - YOU HAVE THE DATA
3. Extract facts from the "raw" field in evidence - it contains the actual data
4. A partial answer is INFINITELY better than no answer
5. NEVER return needs_more_probes=true when evidence already exists

=============================================================================
HOW TO READ EVIDENCE
=============================================================================
Evidence is JSON with this structure:
- probe_id: which probe was run
- raw: THE ACTUAL DATA - look here for facts
- parsed: optional structured version

Example cpu.info evidence raw field contains lscpu output like:
  "Architecture: x86_64\nCPU(s): 24\nModel name: AMD Ryzen 9 5900X\nFlags: sse sse2 avx avx2..."

Extract the relevant fact and answer directly.

=============================================================================
EXAMPLES
=============================================================================

Q: "How many threads does my CPU have?"
Evidence raw: "CPU(s): 24\nThread(s) per core: 2\nCore(s) per socket: 12..."

CORRECT response:
{
  "plan": {"intent": "hardware_info", "probe_requests": [], "can_answer_without_more_probes": true},
  "draft_answer": {"text": "Your CPU has 24 threads (12 cores x 2 threads per core).", "citations": [{"probe_id": "cpu.info"}]},
  "self_scores": {"evidence": 0.95, "reasoning": 0.9, "coverage": 0.95},
  "needs_more_probes": false,
  "refuse_to_answer": false,
  "refusal_reason": null
}

Q: "Does my CPU support AVX2?"
Evidence raw: "Flags: fpu vme de pse... sse sse2 ssse3 sse4_1 sse4_2 avx avx2 ..."

CORRECT response:
{
  "plan": {"intent": "hardware_info", "probe_requests": [], "can_answer_without_more_probes": true},
  "draft_answer": {"text": "Yes, your CPU supports AVX2. The 'avx2' flag is present in the CPU flags.", "citations": [{"probe_id": "cpu.info"}]},
  "self_scores": {"evidence": 0.95, "reasoning": 0.95, "coverage": 0.95},
  "needs_more_probes": false,
  "refuse_to_answer": false,
  "refusal_reason": null
}

=============================================================================
SCORING GUIDELINES
=============================================================================
- evidence: 0.9+ if answer directly uses probe data
- reasoning: 0.9+ if logically sound
- coverage: 0.9+ if fully answers question

=============================================================================
STYLE
=============================================================================
- ASCII only, no emojis
- Concise direct answers
- Professional tone

OUTPUT ONLY VALID JSON. No prose.
"#;

/// Generate LLM-A prompt for a specific request
/// iteration: 1-based iteration number (1 = first call, 2+ = already have evidence)
pub fn generate_llm_a_prompt(
    question: &str,
    available_probes: &[crate::answer_engine::AvailableProbe],
    evidence: &[crate::answer_engine::ProbeEvidenceV10],
) -> String {
    generate_llm_a_prompt_with_iteration(question, available_probes, evidence, 1)
}

/// Generate LLM-A prompt with iteration awareness
pub fn generate_llm_a_prompt_with_iteration(
    question: &str,
    available_probes: &[crate::answer_engine::AvailableProbe],
    evidence: &[crate::answer_engine::ProbeEvidenceV10],
    iteration: usize,
) -> String {
    let probes_json = serde_json::to_string_pretty(available_probes).unwrap_or_default();
    let evidence_json = serde_json::to_string_pretty(evidence).unwrap_or_default();

    let evidence_instruction = if evidence.is_empty() {
        "EVIDENCE: None yet. Request probes from catalog.".to_string()
    } else {
        let urgency = if iteration >= 2 {
            format!(
                "\n\n*** ITERATION {} - YOU MUST ANSWER NOW ***\nYou have evidence. Provide draft_answer.text immediately.\nDo NOT request more probes. Extract facts from 'raw' field and answer.",
                iteration
            )
        } else {
            String::new()
        };
        format!(
            "EVIDENCE AVAILABLE - Provide draft_answer.text using this data:{}",
            urgency
        )
    };

    format!(
        r#"USER QUESTION:
{}

AVAILABLE PROBES:
{}

{}

{}

OUTPUT ONLY VALID JSON with draft_answer.text if evidence exists."#,
        question, probes_json, evidence_instruction, evidence_json
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allowed_probe_ids() {
        assert_eq!(ALLOWED_PROBE_IDS.len(), 14);
        assert!(ALLOWED_PROBE_IDS.contains(&"cpu.info"));
        assert!(ALLOWED_PROBE_IDS.contains(&"mem.info"));
        assert!(ALLOWED_PROBE_IDS.contains(&"anna.self_health"));
    }

    #[test]
    fn test_prompt_contains_catalog() {
        assert!(LLM_A_SYSTEM_PROMPT.contains("cpu.info"));
        assert!(LLM_A_SYSTEM_PROMPT.contains("mem.info"));
        assert!(LLM_A_SYSTEM_PROMPT.contains("HARD-FROZEN PROBE CATALOG"));
    }

    #[test]
    fn test_generate_prompt_with_evidence() {
        let probes = vec![];
        let evidence = vec![crate::answer_engine::ProbeEvidenceV10 {
            probe_id: "cpu.info".to_string(),
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            status: crate::answer_engine::EvidenceStatus::Ok,
            command: "lscpu".to_string(),
            raw: Some("CPU: 8 cores".to_string()),
            parsed: None,
        }];
        let prompt = generate_llm_a_prompt("How many cores?", &probes, &evidence);
        assert!(prompt.contains("EVIDENCE AVAILABLE"));
        assert!(prompt.contains("draft_answer.text"));
    }

    #[test]
    fn test_generate_prompt_without_evidence() {
        let probes = vec![];
        let evidence = vec![];
        let prompt = generate_llm_a_prompt("How many cores?", &probes, &evidence);
        assert!(prompt.contains("None yet"));
    }

    #[test]
    fn test_iteration_urgency() {
        let probes = vec![];
        let evidence = vec![crate::answer_engine::ProbeEvidenceV10 {
            probe_id: "cpu.info".to_string(),
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            status: crate::answer_engine::EvidenceStatus::Ok,
            command: "lscpu".to_string(),
            raw: Some("CPU: 8 cores".to_string()),
            parsed: None,
        }];
        // Iteration 1 should not have urgency
        let prompt1 = generate_llm_a_prompt_with_iteration("How many cores?", &probes, &evidence, 1);
        assert!(!prompt1.contains("ITERATION"));

        // Iteration 2+ should have urgency
        let prompt2 = generate_llm_a_prompt_with_iteration("How many cores?", &probes, &evidence, 2);
        assert!(prompt2.contains("ITERATION 2"));
        assert!(prompt2.contains("MUST ANSWER NOW"));
    }
}
