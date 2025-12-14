// v0.0.617: Settings Dispatcher (Phase 193)
// Dispatch settings operations to handlers

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Dispatch priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum DispatchPriority {
    /// Urgent
    Urgent,
    /// High
    High,
    /// Normal
    #[default]
    Normal,
    /// Low
    Low,
    /// Deferred
    Deferred,
}

impl std::fmt::Display for DispatchPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Urgent => write!(f, "urgent"),
            Self::High => write!(f, "high"),
            Self::Normal => write!(f, "normal"),
            Self::Low => write!(f, "low"),
            Self::Deferred => write!(f, "deferred"),
        }
    }
}

/// Dispatch status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DispatchStatus {
    /// Queued
    #[default]
    Queued,
    /// Dispatched
    Dispatched,
    /// Delivered
    Delivered,
    /// Failed
    Failed,
    /// Rejected
    Rejected,
}

impl std::fmt::Display for DispatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Queued => write!(f, "queued"),
            Self::Dispatched => write!(f, "dispatched"),
            Self::Delivered => write!(f, "delivered"),
            Self::Failed => write!(f, "failed"),
            Self::Rejected => write!(f, "rejected"),
        }
    }
}

/// Dispatch message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchMessage {
    /// Unique ID
    pub id: String,
    /// Target handler
    pub target: String,
    /// Operation
    pub operation: String,
    /// Payload
    pub payload: Option<String>,
    /// Priority
    pub priority: DispatchPriority,
    /// Status
    pub status: DispatchStatus,
    /// Created timestamp
    pub created_at: u64,
    /// Dispatched timestamp
    pub dispatched_at: Option<u64>,
}

impl DispatchMessage {
    /// Create new message
    pub fn new(id: impl Into<String>, target: impl Into<String>, operation: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            target: target.into(),
            operation: operation.into(),
            payload: None,
            priority: DispatchPriority::Normal,
            status: DispatchStatus::Queued,
            created_at: 0,
            dispatched_at: None,
        }
    }

    /// Set payload
    pub fn payload(mut self, payload: impl Into<String>) -> Self {
        self.payload = Some(payload.into());
        self
    }

    /// Set priority
    pub fn priority(mut self, priority: DispatchPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set created timestamp
    pub fn created_at(mut self, ts: u64) -> Self {
        self.created_at = ts;
        self
    }

    /// Mark dispatched
    pub fn mark_dispatched(&mut self, ts: u64) {
        self.status = DispatchStatus::Dispatched;
        self.dispatched_at = Some(ts);
    }

    /// Mark delivered
    pub fn mark_delivered(&mut self) {
        self.status = DispatchStatus::Delivered;
    }

    /// Mark failed
    pub fn mark_failed(&mut self) {
        self.status = DispatchStatus::Failed;
    }

    /// Mark rejected
    pub fn mark_rejected(&mut self) {
        self.status = DispatchStatus::Rejected;
    }
}

/// Dispatch result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchResult {
    /// Message ID
    pub message_id: String,
    /// Success
    pub success: bool,
    /// Response
    pub response: Option<String>,
    /// Error
    pub error: Option<String>,
    /// Duration ms
    pub duration_ms: u64,
}

impl DispatchResult {
    /// Create success result
    pub fn success(message_id: impl Into<String>) -> Self {
        Self {
            message_id: message_id.into(),
            success: true,
            response: None,
            error: None,
            duration_ms: 0,
        }
    }

    /// Create failure result
    pub fn failure(message_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            message_id: message_id.into(),
            success: false,
            response: None,
            error: Some(error.into()),
            duration_ms: 0,
        }
    }

    /// Set response
    pub fn with_response(mut self, response: impl Into<String>) -> Self {
        self.response = Some(response.into());
        self
    }

    /// Set duration
    pub fn with_duration(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }
}

/// Settings dispatcher
#[derive(Debug, Clone, Default)]
pub struct SettingsDispatcher {
    /// Message queue
    queue: VecDeque<DispatchMessage>,
    /// Results
    results: Vec<DispatchResult>,
    /// Max queue size
    max_queue: usize,
    /// Max results
    max_results: usize,
    /// Total dispatched
    total_dispatched: usize,
    /// Total delivered
    total_delivered: usize,
}

impl SettingsDispatcher {
    /// Create new dispatcher
    pub fn new() -> Self {
        Self {
            max_queue: 500,
            max_results: 200,
            ..Default::default()
        }
    }

    /// Enqueue message
    pub fn enqueue(&mut self, message: DispatchMessage) -> bool {
        if self.queue.len() >= self.max_queue {
            return false;
        }
        self.queue.push_back(message);
        self.sort_by_priority();
        true
    }

    /// Dequeue message
    pub fn dequeue(&mut self) -> Option<DispatchMessage> {
        self.queue.pop_front()
    }

    /// Sort by priority
    fn sort_by_priority(&mut self) {
        let mut items: Vec<_> = self.queue.drain(..).collect();
        items.sort_by_key(|m| m.priority);
        self.queue = items.into();
    }

    /// Complete dispatch
    pub fn complete(&mut self, result: DispatchResult) {
        self.total_dispatched += 1;
        if result.success {
            self.total_delivered += 1;
        }
        self.results.push(result);
        while self.results.len() > self.max_results {
            self.results.remove(0);
        }
    }

    /// Queue length
    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }

    /// Total dispatched
    pub fn total_dispatched(&self) -> usize {
        self.total_dispatched
    }

    /// Total delivered
    pub fn total_delivered(&self) -> usize {
        self.total_delivered
    }

    /// Delivery rate
    pub fn delivery_rate(&self) -> f64 {
        if self.total_dispatched == 0 {
            1.0
        } else {
            self.total_delivered as f64 / self.total_dispatched as f64
        }
    }

    /// Results
    pub fn results(&self) -> &[DispatchResult] {
        &self.results
    }
}

/// Format dispatcher
pub fn format_dispatcher(dispatcher: &SettingsDispatcher) -> String {
    let mut output = String::new();
    output.push_str("Settings Dispatcher:\n");
    output.push_str(&format!("  Queue: {}\n", dispatcher.queue_len()));
    output.push_str(&format!("  Dispatched: {}\n", dispatcher.total_dispatched()));
    output.push_str(&format!("  Delivered: {}\n", dispatcher.total_delivered()));
    output.push_str(&format!("  Delivery Rate: {:.1}%\n", dispatcher.delivery_rate() * 100.0));
    output
}

/// Check if query is about dispatcher
pub fn is_dispatcher_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("dispatcher")
        || lower.contains("dispatch")
        || lower.contains("message queue")
}

/// Fun fact about dispatcher
pub fn dispatcher_fun_fact() -> &'static str {
    "Anna's dispatcher routes settings operations to the right handlers based on priority!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_display() {
        assert_eq!(format!("{}", DispatchPriority::Urgent), "urgent");
        assert_eq!(format!("{}", DispatchPriority::Normal), "normal");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", DispatchStatus::Queued), "queued");
        assert_eq!(format!("{}", DispatchStatus::Delivered), "delivered");
    }

    #[test]
    fn test_message_new() {
        let m = DispatchMessage::new("m1", "handler1", "read");
        assert_eq!(m.status, DispatchStatus::Queued);
    }

    #[test]
    fn test_message_builder() {
        let m = DispatchMessage::new("m1", "handler1", "write")
            .payload("data")
            .priority(DispatchPriority::High);
        assert_eq!(m.priority, DispatchPriority::High);
    }

    #[test]
    fn test_message_lifecycle() {
        let mut m = DispatchMessage::new("m1", "h1", "op");
        m.mark_dispatched(100);
        assert_eq!(m.status, DispatchStatus::Dispatched);
        m.mark_delivered();
        assert_eq!(m.status, DispatchStatus::Delivered);
    }

    #[test]
    fn test_result_success() {
        let r = DispatchResult::success("m1");
        assert!(r.success);
    }

    #[test]
    fn test_result_failure() {
        let r = DispatchResult::failure("m1", "error");
        assert!(!r.success);
    }

    #[test]
    fn test_dispatcher_new() {
        let d = SettingsDispatcher::new();
        assert_eq!(d.queue_len(), 0);
    }

    #[test]
    fn test_dispatcher_enqueue() {
        let mut d = SettingsDispatcher::new();
        d.enqueue(DispatchMessage::new("m1", "h1", "op"));
        assert_eq!(d.queue_len(), 1);
    }

    #[test]
    fn test_dispatcher_priority_order() {
        let mut d = SettingsDispatcher::new();
        d.enqueue(DispatchMessage::new("m1", "h1", "op").priority(DispatchPriority::Low));
        d.enqueue(DispatchMessage::new("m2", "h1", "op").priority(DispatchPriority::Urgent));
        let m = d.dequeue().unwrap();
        assert_eq!(m.priority, DispatchPriority::Urgent);
    }

    #[test]
    fn test_is_dispatcher_query() {
        assert!(is_dispatcher_query("message dispatcher"));
        assert!(!is_dispatcher_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = dispatcher_fun_fact();
        assert!(fact.contains("dispatcher"));
    }
}
