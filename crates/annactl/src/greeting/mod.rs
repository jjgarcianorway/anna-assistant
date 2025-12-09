//! Theatre-style REPL greeting for Service Desk experience (v0.0.186).
//!
//! v0.0.119: Clean, concise greetings.
//! v0.0.142: More conversational, personalized greetings.
//! v0.0.186: Modularized into domain-focused submodules.

mod personal;
mod status;
mod tests;
mod types;

use anna_shared::snapshot::{self, SystemSnapshot};
use anna_shared::status::DaemonStatus;
use anna_shared::telemetry::TelemetrySnapshot;
use anna_shared::ui::{colors, HR};
use anna_shared::user_profile::UserProfile;

use personal::{print_open_tickets, print_personalized_greeting, print_user_patterns};
use status::{collect_failed_services, print_since_last_time, print_system_readiness};
use types::calculate_interaction_info;

// Re-export for tests
pub use types::{bullet, InteractionInfo};

/// Print the theatre-style REPL greeting
/// Shows: personalized greeting, time since last visit, health deltas, patterns
/// v0.0.106: Loads user profile for personalized patterns
/// v0.0.142: More conversational style
pub fn print_theatre_greeting(status: Option<&DaemonStatus>) {
    let username = std::env::var("USER").unwrap_or_else(|_| "user".to_string());

    // v0.0.106: Load user profile
    let mut profile = UserProfile::load();

    // Load last snapshot for comparison
    let last_snapshot = snapshot::load_last_snapshot();
    let interaction_info = calculate_interaction_info(&last_snapshot);

    // Collect current state
    let telemetry = TelemetrySnapshot::collect();
    let mut current_snapshot = SystemSnapshot::now();
    let failed_services = collect_failed_services(&mut current_snapshot);

    // Calculate health delta if we have a previous snapshot
    let health_deltas = if let Some(ref prev) = last_snapshot {
        snapshot::diff_snapshots(prev, &current_snapshot)
    } else {
        Vec::new()
    };

    // v0.0.142: Clean header without redundant title
    println!();
    println!("{}{}{}", colors::DIM, HR, colors::RESET);

    // Personalized greeting based on interaction history
    print_personalized_greeting(&username, &interaction_info);

    // "Since last time" section if we have meaningful changes
    if last_snapshot.is_some() {
        print_since_last_time(
            &telemetry,
            &health_deltas,
            failed_services,
            &interaction_info,
        );
    }

    // v0.0.106: Show personalized patterns if we have history
    print_user_patterns(&profile);

    // v0.0.116: Show open tickets if any
    print_open_tickets();

    // System readiness (LLM state)
    if let Some(st) = status {
        print_system_readiness(st);
    }

    // v0.0.142: More conversational closing
    println!();
    println!(
        "{}But I believe you want to ask me something, isn't it?{}",
        colors::DIM,
        colors::RESET
    );
    println!();

    // v0.0.106: Update profile and save
    profile.record_session();
    let _ = profile.save();

    // Save snapshot for next time
    let _ = snapshot::save_snapshot(&current_snapshot);
}
