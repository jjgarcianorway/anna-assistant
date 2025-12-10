//! LLM-powered dialogue generation for natural specialist chatter (v0.0.319).
//!
//! v0.0.255: Added personality quirks for unique character voices.
//! v0.0.265: Disabled - small models produced nonsense.
//! v0.0.319: Re-enabled with better prompts for context-aware dialogue.
//!           Dialogues explain the truth but look different each time.
//!           Style inspired by README: natural, conversational, shows thinking.

use anna_shared::roster::PersonProfile;
use crate::ollama;
use tracing::{debug, warn};

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

/// Short timeout for dialogue generation (don't block the request)
const DIALOGUE_TIMEOUT_SECS: u64 = 3;

/// Generate Anna's dispatch message to the junior
/// v0.0.319: Context-aware - mentions what the query is about
pub async fn gen_dispatch(
    model: &str,
    junior: &PersonProfile,
    case_id: &str,
    query: &str,
) -> Option<String> {
    let short_id = &case_id[..8.min(case_id.len())];
    let query_summary = summarize_query(query);

    let prompt = format!(
        r#"You are Anna, a friendly service desk manager assigning a ticket.
Write ONE casual line (max 12 words) to {} about this task.

User asked about: {}
Case: {}

Style: Natural workplace chat. Like talking to a colleague.

Examples of good responses:
- "{}, disk space question for you."
- "Got a network thing, {}."
- "{}, someone's asking about memory usage."
- "Hey {}, process question coming your way."

Your line (no quotes):"#,
        junior.display_name, query_summary, short_id,
        junior.display_name, junior.display_name, junior.display_name, junior.display_name
    );

    generate_dialogue(model, &prompt, 3, 18).await
}

/// Generate junior's acknowledgment - shows they understand the task
pub async fn gen_junior_ack(
    model: &str,
    junior: &PersonProfile,
    query: &str,
) -> Option<String> {
    let query_summary = summarize_query(query);

    let prompt = format!(
        r#"You are {}, an IT tech acknowledging a ticket about: {}

Write ONE casual line (max 10 words) showing you understand and will handle it.

Style: Like a quick reply to a coworker. Natural, not robotic.

Examples of good responses:
- "I'll check the usual suspects."
- "On it. Let me pull some numbers."
- "Storage thing? Checking now."
- "Gotcha. Running diagnostics."

Your line (no quotes):"#,
        junior.display_name, query_summary
    );

    generate_dialogue(model, &prompt, 2, 14).await
}

/// Generate junior's probing message - shows they're gathering data
pub async fn gen_junior_probing(
    model: &str,
    junior: &PersonProfile,
    probe_count: usize,
) -> Option<String> {
    let prompt = format!(
        r#"You are {}, running {} system command{} to gather data.

Write ONE casual status line (max 8 words).

Style: Quick update to the team. Technical but natural.

Examples of good responses:
- "Running {} checks..."
- "Pulling the numbers now."
- "Let me grab some data."
- "Checking {} things..."

Your line (no quotes):"#,
        junior.display_name, probe_count, if probe_count == 1 { "" } else { "s" },
        probe_count, probe_count
    );

    generate_dialogue(model, &prompt, 2, 12).await
}

/// Generate junior's probes done message - reports what they found
pub async fn gen_junior_probes_done(
    model: &str,
    junior: &PersonProfile,
    success_count: usize,
    planned_count: usize,
) -> Option<String> {
    let outcome = if success_count == planned_count && planned_count > 0 {
        format!("all {} succeeded", planned_count)
    } else if success_count > 0 {
        format!("{} of {} worked", success_count, planned_count)
    } else {
        "limited data".to_string()
    };

    let prompt = format!(
        r#"You are {}. System checks done: {}.

Write ONE casual line (max 10 words) about what you got.

Style: Quick status update. Natural IT speak.

Examples of good responses:
- "Got everything. Good data."
- "Partial data but should be enough."
- "All checks passed."
- "Mixed results, but I can work with this."

Your line (no quotes):"#,
        junior.display_name, outcome
    );

    generate_dialogue(model, &prompt, 2, 14).await
}

/// Generate junior's reviewing message - shows quality check
pub async fn gen_junior_reviewing(
    model: &str,
    junior: &PersonProfile,
) -> Option<String> {
    let prompt = format!(
        r#"You are {}, reviewing an answer before sending it back.

Write ONE casual line (max 7 words) about checking quality.

Style: Thinking out loud. Natural self-check.

Examples of good responses:
- "Let me double-check this..."
- "Looks right. Verifying..."
- "Running sanity check."
- "Making sure this is accurate."

Your line (no quotes):"#,
        junior.display_name
    );

    generate_dialogue(model, &prompt, 2, 10).await
}

/// Generate junior's done message - reports confidence
pub async fn gen_junior_done(
    model: &str,
    junior: &PersonProfile,
    confidence: u8,
) -> Option<String> {
    let quality_word = if confidence >= 90 {
        "solid"
    } else if confidence >= 70 {
        "good"
    } else if confidence >= 50 {
        "okay"
    } else {
        "uncertain"
    };

    let prompt = format!(
        r#"You are {}. Answer is ready, {}% confidence ({} quality).

Write ONE casual completion line (max 10 words) including the percentage.

Style: Quick handoff. Honest about confidence level.

Examples of good responses:
- "Done. {}% - looks solid."
- "Finished. {}%, should be good."
- "Ready. Only {}% though."
- "That's {}% on my end."

Your line (no quotes):"#,
        junior.display_name, confidence, quality_word,
        confidence, confidence, confidence, confidence
    );

    generate_dialogue(model, &prompt, 2, 14).await
}

/// Generate Anna's returning message - thanks the staff
pub async fn gen_anna_returning(
    model: &str,
    junior: &PersonProfile,
) -> Option<String> {
    let prompt = format!(
        r#"You are Anna, thanking {} for handling a ticket.

Write ONE casual thanks (max 8 words) before sending answer to user.

Style: Quick appreciation. Workplace friendly.

Examples of good responses:
- "Thanks {}! Got it."
- "Perfect, {}. I'll take it."
- "Nice work, {}."
- "Got it. Thanks {}."

Your line (no quotes):"#,
        junior.display_name, junior.display_name, junior.display_name, junior.display_name, junior.display_name
    );

    generate_dialogue(model, &prompt, 2, 12).await
}

/// Helper: Generate dialogue with validation
async fn generate_dialogue(
    model: &str,
    prompt: &str,
    min_words: usize,
    max_words: usize,
) -> Option<String> {
    match ollama::chat_with_timeout(model, prompt, DIALOGUE_TIMEOUT_SECS).await {
        Ok(response) => {
            let cleaned = clean_dialogue_response(&response);
            if is_valid_dialogue(&cleaned, min_words, max_words) {
                debug!("Generated dialogue: {}", cleaned);
                Some(cleaned)
            } else {
                debug!("Invalid dialogue ({}w), using fallback: {}",
                    cleaned.split_whitespace().count(), cleaned);
                None
            }
        }
        Err(e) => {
            warn!("Dialogue gen error: {}", e);
            None
        }
    }
}

/// Summarize query to key topic (for more focused prompts)
fn summarize_query(query: &str) -> String {
    let q = query.to_lowercase();

    // Detect category from keywords
    if q.contains("disk") || q.contains("storage") || q.contains("space") {
        "disk/storage".to_string()
    } else if q.contains("memory") || q.contains("ram") || q.contains("swap") {
        "memory usage".to_string()
    } else if q.contains("network") || q.contains("internet") || q.contains("wifi") || q.contains("ip") {
        "network".to_string()
    } else if q.contains("process") || q.contains("cpu") || q.contains("slow") {
        "performance".to_string()
    } else if q.contains("service") || q.contains("systemd") || q.contains("daemon") {
        "services".to_string()
    } else if q.contains("package") || q.contains("install") || q.contains("update") {
        "packages".to_string()
    } else if q.contains("config") || q.contains("vim") || q.contains("bash") || q.contains("editor") {
        "configuration".to_string()
    } else if q.contains("health") || q.contains("status") || q.contains("how is") {
        "system health".to_string()
    } else if query.len() > 40 {
        format!("{}...", &query[..37])
    } else {
        query.to_string()
    }
}

/// Clean up LLM response - remove quotes, prefixes, artifacts
fn clean_dialogue_response(response: &str) -> String {
    let mut s = response.trim();

    // Remove markdown quotes
    if s.starts_with('"') && s.ends_with('"') && s.len() > 2 {
        s = &s[1..s.len()-1];
    }
    if s.starts_with('\'') && s.ends_with('\'') && s.len() > 2 {
        s = &s[1..s.len()-1];
    }

    // Remove common prefixes the LLM might add
    for prefix in &[
        "Message:", "Your message:", "Response:", "Answer:", "Line:",
        "Your line:", "My line:", "Reply:", "Output:",
    ] {
        if let Some(rest) = s.strip_prefix(prefix) {
            s = rest.trim();
        }
    }

    // Take only first line if multiple
    if let Some(first) = s.lines().next() {
        s = first;
    }

    // Remove leading dash/bullet if present
    let s = s.strip_prefix("- ").unwrap_or(s);
    let s = s.strip_prefix("* ").unwrap_or(s);

    s.trim().to_string()
}

/// Validate dialogue - check reasonable length and no garbage
fn is_valid_dialogue(s: &str, min_words: usize, max_words: usize) -> bool {
    if s.is_empty() {
        return false;
    }

    let word_count = s.split_whitespace().count();
    if word_count < min_words || word_count > max_words {
        return false;
    }

    // Reject if mostly special chars (likely garbage)
    let alnum_count = s.chars().filter(|c| c.is_alphanumeric()).count();
    if alnum_count < s.len() / 2 {
        return false;
    }

    // Reject common garbage patterns
    let s_lower = s.to_lowercase();
    if s_lower.contains("as an ai") || s_lower.contains("i cannot") {
        return false;
    }

    true
}
