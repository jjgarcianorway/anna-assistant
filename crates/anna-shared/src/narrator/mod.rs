//! Team-aware dialog formatting for service desk narration (v0.0.218).
//!
//! Provides consistent display names and narrative formatting for team actions.
//! Uses roster.rs for humanized person profiles.
//!
//! v0.0.218: Modularized into domain-focused submodules.

mod it_dialog;
mod narration;
mod person;
mod roles;

#[cfg(test)]
mod tests;

// Re-export for backwards compatibility
pub use it_dialog::{it_confidence, it_domain_context, it_greeting};
pub use narration::{
    format_issues_list, narrate_escalation, narrate_review_result, narrate_team_action,
    narrate_ticket_assignment, status_indicator,
};
pub use person::{
    get_person, narrate_person_action, narrate_person_escalation, narrate_person_review,
};
pub use roles::{reviewer_badge, team_role_name, team_tag};
