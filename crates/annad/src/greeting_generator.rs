//! LLM-based greeting generator (v0.0.292).
//!
//! Uses the translator model to generate personalized, varied greetings
//! while maintaining consistent content structure.
//!
//! v0.0.292: Added personality-aware greeting generation.

use anna_shared::greeting_context::{GreetingContext, GreetingResponse};
use tracing::{info, warn};

use crate::ollama;

/// Build the greeting generation prompt from context
fn build_greeting_prompt(ctx: &GreetingContext) -> String {
    let mut facts = Vec::new();

    // User info
    facts.push(format!("Username: {}", ctx.username));

    // Time since last session
    if ctx.is_first_time {
        facts.push("This is the user's FIRST TIME using the system".to_string());
    } else if let Some(days) = ctx.days_since_last {
        facts.push(format!("Days since last session: {}", days));
    } else if let Some(hours) = ctx.hours_since_last {
        facts.push(format!("Hours since last session: {}", hours));
    }

    // Streak
    if ctx.streak_days > 1 {
        facts.push(format!("Current streak: {} consecutive days", ctx.streak_days));
    }

    // Preferences
    if let Some(ref editor) = ctx.preferred_editor {
        facts.push(format!("Preferred editor: {}", editor));
    }

    if let Some(ref topic) = ctx.top_topic {
        facts.push(format!("Most frequent topic: {}", topic));
    }

    // Open tickets
    if ctx.open_tickets > 0 {
        facts.push(format!("Open tickets: {}", ctx.open_tickets));
    }

    // Last session summary
    if let Some(ref summary) = ctx.last_session_summary {
        facts.push(format!("Last session: {}", summary));
    }

    // Health issues
    if !ctx.health_issues.is_empty() {
        let issues = ctx.health_issues.iter().take(3).cloned().collect::<Vec<_>>().join(", ");
        facts.push(format!("System issues: {}", issues));
    }

    // LLM status
    if ctx.llm_status != "ready" {
        facts.push(format!("LLM status: {}", ctx.llm_status));
    }

    // v0.0.292: Personality-specific tone rules
    let personality_rules = build_personality_rules(ctx);

    format!(
        r#"Generate a friendly IT service desk greeting for a user.

USER FACTS:
{}

TONE & STYLE:
{}

CONTENT RULES:
- Start with "Hello" or similar greeting using their username
- If first time: Welcome them, introduce yourself as Anna (their local IT department)
- If returning after days: Acknowledge the time gap warmly
- If open tickets: Mention them briefly
- If health issues: Note them with concern
- If they have a streak: Acknowledge it positively
- End with an invitation to ask questions (varied phrasing each time)
- Keep it concise (3-6 lines)
- NO markdown, NO bullet points, plain text only
- Vary your phrasing - don't be repetitive between greetings

Output ONLY the greeting text, nothing else."#,
        facts.join("\n"),
        personality_rules
    )
}

/// v0.0.292: Build personality-specific tone rules
fn build_personality_rules(ctx: &GreetingContext) -> String {
    let mut rules = Vec::new();

    // Formality
    if !ctx.personality.formality.is_empty() {
        rules.push(format!("Formality: {}", ctx.personality.formality));
    } else {
        rules.push("Formality: balanced - friendly but professional".to_string());
    }

    // Humor
    if !ctx.personality.humor.is_empty() {
        rules.push(format!("Humor: {}", ctx.personality.humor));
    } else {
        rules.push("Humor: subtle - occasional light touch".to_string());
    }

    // Technical depth
    if !ctx.personality.technical_depth.is_empty() {
        rules.push(format!("Technical style: {}", ctx.personality.technical_depth));
    }

    rules.join("\n")
}

/// Generate a personalized greeting using the translator LLM
pub async fn generate_greeting(
    model: &str,
    ctx: &GreetingContext,
    timeout_secs: u64,
) -> GreetingResponse {
    let prompt = build_greeting_prompt(ctx);

    info!("Generating greeting for {} (payload {} bytes)", ctx.username, prompt.len());

    match ollama::chat_with_timeout(model, &prompt, timeout_secs).await {
        Ok(response) => {
            let greeting = response.trim().to_string();

            // Validate the response has some content
            if greeting.len() > 10 && greeting.len() < 2000 {
                info!("LLM greeting generated ({} chars)", greeting.len());
                GreetingResponse {
                    greeting,
                    is_llm_generated: true,
                }
            } else {
                warn!("LLM greeting invalid length ({}), using fallback", greeting.len());
                GreetingResponse::fallback(ctx)
            }
        }
        Err(e) => {
            warn!("LLM greeting failed: {}, using fallback", e);
            GreetingResponse::fallback(ctx)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greeting_prompt_first_time() {
        let ctx = GreetingContext {
            username: "alice".to_string(),
            is_first_time: true,
            ..Default::default()
        };

        let prompt = build_greeting_prompt(&ctx);
        assert!(prompt.contains("alice"));
        assert!(prompt.contains("FIRST TIME"));
    }

    #[test]
    fn test_greeting_prompt_returning_user() {
        let ctx = GreetingContext {
            username: "bob".to_string(),
            is_first_time: false,
            days_since_last: Some(5),
            streak_days: 3,
            open_tickets: 2,
            ..Default::default()
        };

        let prompt = build_greeting_prompt(&ctx);
        assert!(prompt.contains("bob"));
        assert!(prompt.contains("Days since last session: 5"));
        assert!(prompt.contains("streak: 3"));
        assert!(prompt.contains("Open tickets: 2"));
    }

    #[test]
    fn test_greeting_prompt_with_health_issues() {
        let ctx = GreetingContext {
            username: "charlie".to_string(),
            health_issues: vec!["High CPU usage".to_string(), "Low disk space".to_string()],
            ..Default::default()
        };

        let prompt = build_greeting_prompt(&ctx);
        assert!(prompt.contains("System issues:"));
        assert!(prompt.contains("High CPU usage"));
    }
}
