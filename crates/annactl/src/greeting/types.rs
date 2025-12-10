//! Greeting types (v0.0.369).
//!
//! v0.0.275: bullet() now unused (LLM generates greetings), kept for fallback.
//! v0.0.369: Removed bullet() - use symbols::BULLET from centralized UI instead.

#![allow(dead_code)]

use anna_shared::snapshot::SystemSnapshot;

/// Information about user's interaction history
pub struct InteractionInfo {
    pub hours_since_last: Option<u64>,
    pub days_since_last: Option<u64>,
    pub is_first_time: bool,
}

pub fn calculate_interaction_info(last_snapshot: &Option<SystemSnapshot>) -> InteractionInfo {
    match last_snapshot {
        Some(s) if s.captured_at > 0 => {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let hours = now.saturating_sub(s.captured_at) / 3600;
            let days = hours / 24;
            InteractionInfo {
                hours_since_last: Some(hours),
                days_since_last: if days > 0 { Some(days) } else { None },
                is_first_time: false,
            }
        }
        _ => InteractionInfo {
            hours_since_last: None,
            days_since_last: None,
            is_first_time: true,
        },
    }
}
