//! Helper functions for recording telemetry events.

use chrono::Utc;

use super::telemetry::RecipeTelemetry;
use super::types::{LearningEvent, LearningEventType, ResolutionEvent, ResolutionSource};

/// Helper to record a resolution.
pub fn record_resolution(
    telemetry: &mut RecipeTelemetry,
    ticket_id: &str,
    source: ResolutionSource,
    recipe_id: Option<&str>,
    intent: Option<&str>,
    domain: Option<&str>,
    duration_ms: u64,
) {
    telemetry.record_resolution(ResolutionEvent {
        timestamp: Utc::now(),
        ticket_id: ticket_id.to_string(),
        source,
        recipe_id: recipe_id.map(String::from),
        intent: intent.map(String::from),
        domain: domain.map(String::from),
        duration_ms,
    });
}

/// Helper to record a learning event.
pub fn record_learning(
    telemetry: &mut RecipeTelemetry,
    event_type: LearningEventType,
    recipe_id: &str,
    from_ticket_id: Option<&str>,
    details: &str,
) {
    telemetry.record_learning(LearningEvent {
        timestamp: Utc::now(),
        event_type,
        recipe_id: recipe_id.to_string(),
        from_ticket_id: from_ticket_id.map(String::from),
        details: details.to_string(),
    });
}
