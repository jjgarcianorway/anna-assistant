//! LLM-powered dialogue generation for natural specialist chatter (v0.0.254).
//!
//! Uses the translator model to generate contextual, varied dialogue that
//! reflects what staff members are actually doing.

use anna_shared::roster::PersonProfile;
use crate::ollama;
use tracing::debug;

/// Context for dialogue generation
pub struct DialogueContext<'a> {
    pub query: &'a str,
    pub case_id: &'a str,
    pub stage: DialogueStage,
    pub probe_count: Option<usize>,
    pub probe_success: Option<usize>,
    pub confidence: Option<u8>,
}

/// Which stage of the request we're generating dialogue for
#[derive(Debug, Clone, Copy)]
pub enum DialogueStage {
    Dispatch,
    Acknowledge,
    StartProbing,
    ProbesDone,
    Reviewing,
    Escalate,
    SeniorResponse,
    Done,
    AnnaReturning,
}

/// Generate Anna's dispatch message
pub async fn gen_dispatch(
    model: &str,
    junior: &PersonProfile,
    case_id: &str,
    query: &str,
) -> Option<String> {
    let prompt = format!(
        r#"You are Anna, an AI service desk coordinator. Generate a SINGLE short line (max 15 words) dispatching a case to {name}.

Context: Case {case_id}, user asked about: "{query}"
Team: {team:?}

Generate a friendly, natural dispatch message. Just the message, no quotes.
Example style: "Hey {name}! Quick one - user has a memory question. Case {short_id}"
"#,
        name = junior.display_name,
        case_id = case_id,
        query = truncate_query(query),
        team = junior.team,
        short_id = &case_id[..8.min(case_id.len())],
    );

    gen_dialogue_fast(model, &prompt).await
}

/// Generate junior's acknowledgment
pub async fn gen_junior_ack(
    model: &str,
    junior: &PersonProfile,
    _query: &str,
) -> Option<String> {
    let prompt = format!(
        r#"You are {name}, a {role} at an IT help desk. Generate a SINGLE short acknowledgment (max 10 words) that you're taking a case.

Be natural, casual, like real workplace chat. Just the message, no quotes.
Example style: "On it!" or "Got it, checking now" or "Yep, pulling it up"
"#,
        name = junior.display_name,
        role = junior.role_title,
    );

    gen_dialogue_fast(model, &prompt).await
}

/// Generate junior's probing message
pub async fn gen_junior_probing(
    model: &str,
    junior: &PersonProfile,
    probe_count: usize,
) -> Option<String> {
    let prompt = format!(
        r#"You are {name}, a {role}. Generate a SINGLE short line (max 12 words) about running {count} system check(s).

Be technical but brief. Just the message, no quotes.
Example: "Running {count} quick checks..." or "Pulling some diagnostics..."
"#,
        name = junior.display_name,
        role = junior.role_title,
        count = probe_count,
    );

    gen_dialogue_fast(model, &prompt).await
}

/// Generate junior's probes done message
pub async fn gen_junior_probes_done(
    model: &str,
    junior: &PersonProfile,
    success_count: usize,
    planned_count: usize,
) -> Option<String> {
    let status = if success_count == planned_count {
        "all succeeded"
    } else if success_count > 0 {
        "partial data"
    } else {
        "limited results"
    };

    let prompt = format!(
        r#"You are {name}, a {role}. Generate a SINGLE short line (max 10 words) about probe results ({status}).

Be brief and factual. Just the message, no quotes.
Example: "Got the data" or "All checks done" or "{count} of {total} worked"
"#,
        name = junior.display_name,
        role = junior.role_title,
        status = status,
        count = success_count,
        total = planned_count,
    );

    gen_dialogue_fast(model, &prompt).await
}

/// Generate junior's reviewing message
pub async fn gen_junior_reviewing(
    model: &str,
    junior: &PersonProfile,
) -> Option<String> {
    let prompt = format!(
        r#"You are {name}, a {role}. Generate a SINGLE short line (max 10 words) about checking/reviewing data.

Be natural, brief. Just the message, no quotes.
Example: "Looking at the numbers..." or "Checking this..."
"#,
        name = junior.display_name,
        role = junior.role_title,
    );

    gen_dialogue_fast(model, &prompt).await
}

/// Generate junior's done message
pub async fn gen_junior_done(
    model: &str,
    junior: &PersonProfile,
    confidence: u8,
) -> Option<String> {
    let conf_desc = if confidence >= 90 {
        "high confidence"
    } else if confidence >= 70 {
        "good confidence"
    } else {
        "moderate confidence"
    };

    let prompt = format!(
        r#"You are {name}, a {role}. Generate a SINGLE short line (max 12 words) about finishing with {conf}% ({desc}).

Be natural, mention the confidence level. Just the message, no quotes.
Example: "Done, {conf}% sure" or "Looks good - {conf}%"
"#,
        name = junior.display_name,
        role = junior.role_title,
        conf = confidence,
        desc = conf_desc,
    );

    gen_dialogue_fast(model, &prompt).await
}

/// Generate Anna's returning message
pub async fn gen_anna_returning(
    model: &str,
    junior: &PersonProfile,
) -> Option<String> {
    let prompt = format!(
        r#"You are Anna, an AI service desk coordinator. Generate a SINGLE short line (max 12 words) thanking {name} and taking the response back.

Be friendly and brief. Just the message, no quotes.
Example: "Thanks {name}!" or "Got it, sending to user"
"#,
        name = junior.display_name,
    );

    gen_dialogue_fast(model, &prompt).await
}

/// Fast dialogue generation with very short timeout
/// Returns None if generation fails or times out - fallback to static messages
async fn gen_dialogue_fast(model: &str, prompt: &str) -> Option<String> {
    // Very short timeout - dialogue should be instant or fall back
    match ollama::chat_with_timeout(model, prompt, 3).await {
        Ok(response) => {
            // Clean up the response - remove quotes, trim
            let cleaned = response
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .trim();

            // Validate length - should be short
            if cleaned.len() > 0 && cleaned.len() < 200 {
                debug!("Generated dialogue: {}", cleaned);
                Some(cleaned.to_string())
            } else {
                debug!("Dialogue too long or empty, using fallback");
                None
            }
        }
        Err(e) => {
            debug!("Dialogue generation failed: {}, using fallback", e);
            None
        }
    }
}

/// Truncate query for prompt context
fn truncate_query(query: &str) -> &str {
    if query.len() > 50 {
        &query[..50]
    } else {
        query
    }
}
