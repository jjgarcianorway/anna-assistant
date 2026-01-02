//! Evidence formatting utilities for specialist prompts (v0.0.410).

use anna_shared::evidence_engine::EvidenceBundle;
use std::collections::HashMap;

/// Format evidence bundle for specialist prompt
/// This is the key integration point - creates the "evidence" section
pub fn format_evidence_for_prompt(bundle: &EvidenceBundle) -> String {
    let mut output = String::new();

    // Format probes section
    if !bundle.probes.is_empty() {
        output.push_str("## Probe Evidence\n\n");
        for probe in &bundle.probes {
            output.push_str(&format!(
                "### {}\n**Summary:** {}\n**Output:**\n```\n{}\n```\n\n",
                probe.id, probe.summary, probe.excerpt
            ));
        }
    }

    // Format docs section
    if !bundle.docs.is_empty() {
        output.push_str("## Documentation\n\n");
        for doc in &bundle.docs {
            output.push_str(&format!(
                "### {} ({})\n{}\n\n",
                doc.title, doc.source, doc.snippet
            ));
        }
    }

    // Format recipe matches
    if !bundle.recipes.is_empty() {
        output.push_str("## Relevant Recipes\n\n");
        for recipe in &bundle.recipes {
            output.push_str(&format!(
                "- **{}** ({}% confidence): {}\n",
                recipe.title, recipe.confidence, recipe.summary
            ));
        }
    }

    output
}

/// Build probe map for specialist input (backward compatible)
pub fn evidence_to_probe_map(bundle: &EvidenceBundle) -> HashMap<String, String> {
    let mut probes = HashMap::new();

    for evidence in &bundle.probes {
        // Use probe ID as key, excerpt as value
        let key = evidence.id.replace("probe:", "");
        probes.insert(key, evidence.excerpt.clone());
    }

    probes
}

/// Build enhanced input with evidence for specialist
pub fn build_enhanced_specialist_input(
    ticket_id: &str,
    domain: &str,
    intent: &str,
    question: &str,
    bundle: &EvidenceBundle,
    context_claims: Option<String>, // Added context_claims parameter
) -> String {
    let probes_json = evidence_to_probe_map(bundle);

    // Build docs array
    let docs: Vec<serde_json::Value> = bundle
        .docs
        .iter()
        .map(|d| {
            serde_json::json!({
                "source": d.source.to_string(),
                "title": d.title,
                "snippet": d.snippet
            })
        })
        .collect();

    // Build recipes array
    let recipes: Vec<serde_json::Value> = bundle
        .recipes
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "title": r.title,
                "confidence": r.confidence,
                "actions": r.actions
            })
        })
        .collect();

    let mut input = serde_json::json!({
        "ticket_id": ticket_id,
        "domain": domain,
        "intent": intent,
        "question": question,
        "probes": probes_json,
        "docs": docs,
        "recipes": recipes,
        "metadata": {
            "probes_run": bundle.metadata.probes_run,
            "docs_searched": bundle.metadata.docs_searched,
            "gather_time_ms": bundle.metadata.gather_time_ms
        }
    });

    if let Some(claims) = context_claims {
        input["context_claims"] = serde_json::Value::String(claims);
    }

    serde_json::to_string_pretty(&input).unwrap_or_else(|_| "{}".to_string())
}
