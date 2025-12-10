//! Theatre-style REPL greeting for Service Desk experience (v0.0.345).
//!
//! v0.0.119: Clean, concise greetings.
//! v0.0.142: More conversational, personalized greetings.
//! v0.0.186: Modularized into domain-focused submodules.
//! v0.0.238: Added session-based "since last time" summary.
//! v0.0.275: LLM-generated greetings via translator for varied, natural text.
//! v0.0.284: Integrated telemetry-based health alerts.
//! v0.0.292: Added personality traits for tone-aware greetings.

mod personal;
mod status;
mod tests;
mod types;

use anna_shared::greeting_context::GreetingContext;
use anna_shared::health_alerts::{generate_alerts, AlertSeverity};
use anna_shared::snapshot::{self, SystemSnapshot};
use anna_shared::status::DaemonStatus;
use anna_shared::system_telemetry::TelemetryStore;
use anna_shared::ticket_tracker::TicketTracker;
use anna_shared::ui::print_hr;
use anna_shared::user_profile::UserProfile;

use status::{collect_failed_services, print_system_readiness};
use types::calculate_interaction_info;

// Re-export for external use
#[allow(unused_imports)]
pub use types::InteractionInfo;

/// Build greeting context from current system state
/// v0.0.287: Added maintenance prompt support
/// v0.0.289: Added interesting facts
/// v0.0.292: Added personality traits
fn build_greeting_context(
    username: &str,
    profile: &UserProfile,
    interaction_info: &types::InteractionInfo,
    health_issues: Vec<String>,
    llm_status: &str,
    snapshot: &snapshot::SystemSnapshot,
    telemetry: Option<&TelemetryStore>,
) -> GreetingContext {
    // Get open tickets count
    let tracker = TicketTracker::for_user();
    let open_tickets = tracker.open_tickets().map(|t| t.len() as u32).unwrap_or(0);

    // Get last session summary
    let last_session_summary = profile.since_last_time();

    // v0.0.314: Get time-of-day defaults
    let default_ctx = GreetingContext::default();

    // Build base context
    let ctx = GreetingContext {
        username: username.to_string(),
        hours_since_last: interaction_info.hours_since_last,
        days_since_last: interaction_info.days_since_last,
        is_first_time: interaction_info.is_first_time,
        streak_days: profile.streak_days,
        preferred_editor: profile.preferred_editor.clone(),
        top_topic: profile.top_topic().map(|s| s.to_string()),
        open_tickets,
        last_session_summary,
        health_issues,
        llm_status: llm_status.to_string(),
        maintenance_prompts: Vec::new(),
        interesting_facts: Vec::new(),
        personality: Default::default(),
        time_of_day: default_ctx.time_of_day,
        local_hour: default_ctx.local_hour,
    };

    // v0.0.287: Add maintenance prompts
    // v0.0.289: Add interesting facts
    // v0.0.292: Add personality traits
    ctx.with_maintenance(snapshot, telemetry)
        .with_interesting_facts(snapshot)
        .with_personality(profile)
}

/// Print the theatre-style REPL greeting
/// v0.0.275: Now uses LLM-generated greetings via translator
pub fn print_theatre_greeting(status: Option<&DaemonStatus>) {
    let username = std::env::var("USER").unwrap_or_else(|_| "user".to_string());

    // v0.0.106: Load user profile
    let mut profile = UserProfile::load();

    // Load last snapshot for comparison
    let last_snapshot = snapshot::load_last_snapshot();
    let interaction_info = calculate_interaction_info(&last_snapshot);

    // Collect current state
    let mut current_snapshot = SystemSnapshot::now();
    let failed_services = collect_failed_services(&mut current_snapshot);

    // v0.0.287: Load telemetry once for health alerts and maintenance prompts
    let telemetry = TelemetryStore::load_if_exists();

    // Collect health issues from multiple sources
    let mut health_issues = Vec::new();

    // 1. Failed services from snapshot
    if failed_services > 0 {
        health_issues.push(format!("{} failed services", failed_services));
    }

    // 2. v0.0.284: Telemetry-based health alerts
    if let Some(ref telemetry) = telemetry {
        let alerts = generate_alerts(telemetry);
        for alert in alerts.iter().filter(|a| !a.dismissed) {
            match alert.severity {
                AlertSeverity::Critical => {
                    health_issues.push(format!("[!] {}", alert.message));
                }
                AlertSeverity::Warning => {
                    health_issues.push(format!("[*] {}", alert.message));
                }
                AlertSeverity::Info => {
                    // Skip info-level in greeting to avoid clutter
                }
            }
        }

        // Add health score if low
        let score = telemetry.health_score();
        if score < 70 {
            health_issues.push(format!("Health score: {}%", score));
        }
    }

    // Get LLM status string
    let llm_status = status
        .map(|s| match s.llm.state {
            anna_shared::status::LlmState::Ready => "ready",
            anna_shared::status::LlmState::Bootstrapping => "starting",
            anna_shared::status::LlmState::PullingModels => "ready", // v0.0.310: functional while loading
            anna_shared::status::LlmState::Error => "error",
        })
        .unwrap_or("unknown");

    // Build greeting context (now includes maintenance prompts)
    let ctx = build_greeting_context(
        &username,
        &profile,
        &interaction_info,
        health_issues,
        llm_status,
        &current_snapshot,
        telemetry.as_ref(),
    );

    // v0.0.142: Clean header without redundant title
    println!();
    print_hr();

    // v0.0.275: Try LLM-generated greeting, fall back to deterministic
    let greeting_text = try_llm_greeting(&ctx);
    println!();
    println!("{}", greeting_text);

    // System readiness (LLM state)
    if let Some(st) = status {
        print_system_readiness(st);
    }

    println!();

    // v0.0.106: Update profile and save
    profile.record_session();
    // v0.0.238: Start a new session for tracking
    profile.start_session();
    let _ = profile.save();

    // Save snapshot for next time
    let _ = snapshot::save_snapshot(&current_snapshot);
}

/// v0.0.300: Deterministic greeting (fast, reliable)
/// LLM-generated greetings were causing hangs and empty output.
fn try_llm_greeting(ctx: &GreetingContext) -> String {
    use anna_shared::greeting_context::GreetingResponse;

    // Always use deterministic fallback - it's fast and reliable
    GreetingResponse::fallback(ctx).greeting
}
