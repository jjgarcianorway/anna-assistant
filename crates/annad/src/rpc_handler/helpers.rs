//! RPC handler helpers (v0.0.298).
//! v0.0.298: Use `validated` field for outcome determination.

use anna_shared::event_log::{EventLog, EventRecord};

use crate::progress_tracker::ProgressTracker;
use crate::state::SharedState;
use crate::theatre::TheatreContext;

/// Save progress events to state for polling
pub async fn save_progress(state: &SharedState, progress: &ProgressTracker) {
    state.write().await.progress_events = progress.events().to_vec();
}

/// v0.0.169: Record event to event log for gamification stats persistence
/// v0.0.298: Use `validated` field instead of reliability_score >= 60
pub fn record_event_log(
    request_id: &str,
    result: &anna_shared::rpc::ServiceDeskResult,
    theatre: &TheatreContext,
    duration_ms: u64,
) {
    let event_log = EventLog::new(EventLog::default_path(), 10000);

    // v0.0.298: Determine outcome based on `validated` field (set by ticket verification loop)
    // This is more authoritative than simple reliability_score thresholds
    let outcome = if result.needs_clarification {
        "clarification"
    } else if result.validated {
        "verified"
    } else if result.reliability_score > 0 {
        "failed"
    } else {
        "timeout"
    };

    // Build event record
    let mut record = EventRecord::new(request_id, &result.domain.to_string());
    record.outcome = outcome.to_string();
    record.reliability = result.reliability_score;
    record.team = theatre.team.to_string();
    record.escalated = theatre.ticket.was_escalated;
    record.escalation_tier = if theatre.ticket.was_escalated { 2 } else { 0 };
    record.duration_ms = duration_ms;
    record.interactions = if result.needs_clarification { 1 } else { 0 };

    // Save to event log (ignore errors - stats are not critical)
    let _ = event_log.append(&record);
}
