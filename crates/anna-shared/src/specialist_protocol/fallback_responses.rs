//! Response generation logic for fallback handler (v0.0.428).

use super::fallback_types::{truncate, ExtractedFact, FallbackContext, FallbackReason};
use super::{ProbeEvidence, ResponseMeta, StrictResponse};

/// Generate partial response from extracted facts
pub fn generate_partial_response(
    ctx: &FallbackContext,
    facts: Vec<ExtractedFact>,
) -> StrictResponse {
    // Build summary from facts
    let summary = if facts.len() == 1 {
        facts[0].summary.clone()
    } else {
        format!(
            "I found {} pieces of information: {}",
            facts.len(),
            facts
                .iter()
                .map(|f| f.summary.as_str())
                .collect::<Vec<_>>()
                .join("; ")
        )
    };

    // Add explanation about the limitation
    let reason_text = match &ctx.reason {
        FallbackReason::Timeout => "My detailed analysis timed out.",
        FallbackReason::ParseError(_) => "I encountered an internal error during analysis.",
        FallbackReason::ValidationFailed(_) => "Some analysis results were inconsistent.",
        FallbackReason::LlmError(_) => "I couldn't complete the full analysis.",
        FallbackReason::NoSpecialist => "No specialist was available for this query.",
        FallbackReason::RetryExhausted => {
            "I couldn't get a complete analysis after multiple attempts."
        }
    };

    let key_facts: Vec<String> = facts.iter().map(|f| f.summary.clone()).collect();
    let probes: Vec<ProbeEvidence> = facts
        .iter()
        .map(|f| ProbeEvidence {
            id: f.probe_id.clone(),
            summary: f.summary.clone(),
            raw_reference: Some(truncate(&f.raw_snippet, 100)),
        })
        .collect();

    let meta = ResponseMeta {
        handled_by: "Fallback Handler".to_string(),
        ticket_id: ctx.ticket_id.clone(),
        version: 1,
    };

    StrictResponse::partial(
        &ctx.domain,
        &ctx.intent,
        &summary,
        key_facts,
        reason_text,
        probes,
        meta,
    )
    .with_latency(ctx.elapsed_ms)
}

/// Generate failure response when no useful data available
pub fn generate_failure_response(ctx: &FallbackContext) -> StrictResponse {
    let summary = match &ctx.reason {
        FallbackReason::Timeout => {
            "I couldn't complete my analysis in time. Please try again with a simpler question."
        }
        FallbackReason::ParseError(_) => {
            "I encountered an internal error. Please try rephrasing your question."
        }
        FallbackReason::ValidationFailed(_) => {
            "I couldn't produce a reliable answer. Please try a different approach."
        }
        FallbackReason::LlmError(_) => "I'm having trouble analyzing this. Please try again later.",
        FallbackReason::NoSpecialist => {
            "I don't have a specialist available for this type of question."
        }
        FallbackReason::RetryExhausted => {
            "I couldn't get a valid response after multiple attempts."
        }
    };

    let meta = ResponseMeta {
        handled_by: "Fallback Handler".to_string(),
        ticket_id: ctx.ticket_id.clone(),
        version: 1,
    };

    StrictResponse::failure(&ctx.domain, &ctx.intent, summary, meta).with_latency(ctx.elapsed_ms)
}
