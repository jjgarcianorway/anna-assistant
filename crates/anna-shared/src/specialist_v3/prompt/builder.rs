//! Prompt building utilities.

/// Generate a specialist prompt for a specific query.
pub fn build_specialist_prompt(
    ticket_id: &str,
    question: &str,
    probe_data: &[(String, String)],
    knowledge_snippets: &[String],
    specialist_domain: &str,
) -> String {
    let mut prompt = String::with_capacity(4096);

    // Ticket context
    prompt.push_str(&format!("TICKET: {}\n", ticket_id));
    prompt.push_str(&format!("DOMAIN: {}\n", specialist_domain));
    prompt.push_str(&format!("QUESTION: {}\n\n", question));

    // Probe data section
    if !probe_data.is_empty() {
        prompt.push_str("PROBE DATA:\n");
        for (probe_id, output) in probe_data {
            prompt.push_str(&format!(
                "--- {} ---\n{}\n\n",
                probe_id,
                truncate_probe(output)
            ));
        }
    } else {
        prompt.push_str("PROBE DATA: (none available)\n\n");
    }

    // Knowledge section
    if !knowledge_snippets.is_empty() {
        prompt.push_str("KNOWLEDGE CITATIONS:\n");
        for snippet in knowledge_snippets {
            prompt.push_str(&format!("{}\n\n", snippet));
        }
    }

    // Final instruction
    prompt.push_str("Respond with ONLY the JSON object. No other text.");

    prompt
}

/// Truncate probe output to reasonable size.
fn truncate_probe(output: &str) -> &str {
    const MAX_PROBE_CHARS: usize = 2000;
    if output.len() <= MAX_PROBE_CHARS {
        output
    } else {
        &output[..MAX_PROBE_CHARS]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_prompt() {
        let prompt = build_specialist_prompt(
            "DSK-001",
            "How much memory is available?",
            &[(
                "probe:free".to_string(),
                "Mem: 25600 8400 17000".to_string(),
            )],
            &[],
            "desktop",
        );

        assert!(prompt.contains("DSK-001"));
        assert!(prompt.contains("memory"));
        assert!(prompt.contains("probe:free"));
    }
}
