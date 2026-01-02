// v0.0.539: Team Consultation Tracker (Phase 115)
// Tracks "most consulted team" and specialist interactions per VISION.md

mod types;
mod record;
mod tracker;
mod formatting;

// Re-export all public items to preserve the original API
pub use types::{TeamDepartment, ConsultationOutcome, SeniorityConsulted};
pub use record::ConsultationRecord;
pub use tracker::TeamConsultationTracker;
pub use formatting::{format_consultation, format_tracker_summary, is_team_query, team_consultation_fun_fact};
