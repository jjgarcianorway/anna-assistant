//! In-flight request deduplication.

use std::collections::HashMap;
use std::time::Instant;
use tracing::debug;

use super::answer_cache::normalize_question;
use super::types::{InflightRequest, INFLIGHT_REQUESTS};

/// Check if a request is already in-flight.
pub fn is_request_inflight(question: &str) -> bool {
    let key = normalize_question(question);

    if let Ok(guard) = INFLIGHT_REQUESTS.read() {
        if let Some(ref requests) = *guard {
            if let Some(req) = requests.get(&key) {
                if req.started_at.elapsed().as_secs() < 60 {
                    debug!("Request already in-flight: {}", &question[..question.len().min(40)]);
                    return true;
                }
            }
        }
    }
    false
}

/// Register a request as in-flight.
pub fn register_inflight_request(question: &str) {
    let key = normalize_question(question);

    if let Ok(mut guard) = INFLIGHT_REQUESTS.write() {
        let requests = guard.get_or_insert_with(HashMap::new);

        requests.insert(key, InflightRequest { started_at: Instant::now() });

        if requests.len() > 20 {
            requests.retain(|_, v| v.started_at.elapsed().as_secs() < 60);
        }
    }
}

/// Remove a request from in-flight tracking.
pub fn complete_inflight_request(question: &str) {
    let key = normalize_question(question);

    if let Ok(mut guard) = INFLIGHT_REQUESTS.write() {
        if let Some(ref mut requests) = *guard {
            requests.remove(&key);
        }
    }
}
