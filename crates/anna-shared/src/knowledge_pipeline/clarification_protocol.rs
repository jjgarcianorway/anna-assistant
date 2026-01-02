//! Clarification protocol manager.

use super::clarification_request::ClarificationRequest;
use super::clarification_response::ClarificationResponse;

/// Clarification protocol manager.
pub struct ClarificationProtocol {
    /// Pending clarifications.
    pending: Vec<ClarificationRequest>,
    /// History of clarifications.
    history: Vec<(ClarificationRequest, ClarificationResponse)>,
}

impl ClarificationProtocol {
    /// Create a new protocol instance.
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            history: Vec::new(),
        }
    }

    /// Request clarification.
    pub fn request(&mut self, req: ClarificationRequest) {
        self.pending.push(req);
    }

    /// Check if there are pending clarifications.
    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    /// Get next pending clarification.
    pub fn next_pending(&self) -> Option<&ClarificationRequest> {
        self.pending.first()
    }

    /// Resolve the current pending clarification.
    pub fn resolve(&mut self, response: ClarificationResponse) -> Option<ClarificationRequest> {
        if let Some(req) = self.pending.first().cloned() {
            self.pending.remove(0);
            self.history.push((req.clone(), response));
            Some(req)
        } else {
            None
        }
    }

    /// Skip the current pending clarification (use default if available).
    pub fn skip(&mut self) -> Option<ClarificationResponse> {
        if let Some(req) = self.pending.first() {
            if let Some(default) = &req.default {
                let response = ClarificationResponse::default_value(default);
                self.resolve(response.clone());
                return Some(response);
            } else if !req.required {
                self.pending.remove(0);
                return Some(ClarificationResponse::default_value(""));
            }
        }
        None
    }

    /// Get clarification history.
    pub fn history(&self) -> &[(ClarificationRequest, ClarificationResponse)] {
        &self.history
    }

    /// Clear all pending clarifications.
    pub fn clear(&mut self) {
        self.pending.clear();
    }

    /// Get count of pending clarifications.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

impl Default for ClarificationProtocol {
    fn default() -> Self {
        Self::new()
    }
}
