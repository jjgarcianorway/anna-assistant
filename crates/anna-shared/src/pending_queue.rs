//! Pending ticket retry queue (v0.0.258).
//!
//! Manages tickets that need to be retried during idle time.
//! These are tickets where Anna couldn't give a confident answer but
//! may be able to improve with more time or different approach.
//!
//! Use cases:
//! - Low reliability score (< 70) but not a failure
//! - Timeout during processing
//! - Needs additional probes or data collection
//! - Learning opportunity (can improve recipe)

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;

/// Minimum reliability score to consider retry
pub const RETRY_THRESHOLD: u8 = 70;

/// Maximum retry attempts before giving up
pub const MAX_RETRY_ATTEMPTS: u8 = 3;

/// Minimum seconds between retry attempts
pub const MIN_RETRY_INTERVAL_SECS: u64 = 300; // 5 minutes

/// A ticket pending retry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingTicket {
    /// Case ID (e.g., "NET-0042")
    pub case_id: String,
    /// Original query from user
    pub query: String,
    /// Last reliability score achieved
    pub last_score: u8,
    /// Number of retry attempts so far
    pub retry_count: u8,
    /// Unix timestamp when created
    pub created_at: u64,
    /// Unix timestamp of last retry attempt
    pub last_retry_at: Option<u64>,
    /// Reason for pending status
    pub reason: PendingReason,
    /// Domain/team for routing
    pub domain: String,
}

/// Reason a ticket is pending retry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PendingReason {
    /// Reliability score was too low
    LowReliability,
    /// Processing timed out
    Timeout,
    /// Evidence was insufficient
    InsufficientEvidence,
    /// Needs human clarification
    NeedsClarification,
    /// LLM was unavailable
    LlmUnavailable,
}

impl std::fmt::Display for PendingReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LowReliability => write!(f, "Low reliability"),
            Self::Timeout => write!(f, "Timeout"),
            Self::InsufficientEvidence => write!(f, "Insufficient evidence"),
            Self::NeedsClarification => write!(f, "Needs clarification"),
            Self::LlmUnavailable => write!(f, "LLM unavailable"),
        }
    }
}

/// Queue of pending tickets
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PendingQueue {
    tickets: VecDeque<PendingTicket>,
}

impl PendingQueue {
    /// Create a new empty queue
    pub fn new() -> Self {
        Self {
            tickets: VecDeque::new(),
        }
    }

    /// Add a ticket to the retry queue
    pub fn enqueue(&mut self, ticket: PendingTicket) {
        // Don't add duplicates
        if !self.tickets.iter().any(|t| t.case_id == ticket.case_id) {
            self.tickets.push_back(ticket);
        }
    }

    /// Get next ticket eligible for retry
    pub fn next_ready(&self) -> Option<&PendingTicket> {
        let now = current_timestamp();
        self.tickets.iter().find(|t| {
            t.retry_count < MAX_RETRY_ATTEMPTS
                && t.last_retry_at
                    .map(|last| now - last >= MIN_RETRY_INTERVAL_SECS)
                    .unwrap_or(true)
        })
    }

    /// Mark ticket as retried (increment count, update timestamp)
    pub fn mark_retried(&mut self, case_id: &str) {
        if let Some(ticket) = self.tickets.iter_mut().find(|t| t.case_id == case_id) {
            ticket.retry_count += 1;
            ticket.last_retry_at = Some(current_timestamp());
        }
    }

    /// Update score after retry
    pub fn update_score(&mut self, case_id: &str, new_score: u8) {
        if let Some(ticket) = self.tickets.iter_mut().find(|t| t.case_id == case_id) {
            ticket.last_score = new_score;
        }
    }

    /// Remove ticket from queue (resolved or gave up)
    pub fn remove(&mut self, case_id: &str) -> Option<PendingTicket> {
        if let Some(pos) = self.tickets.iter().position(|t| t.case_id == case_id) {
            self.tickets.remove(pos)
        } else {
            None
        }
    }

    /// Get all pending tickets
    pub fn all(&self) -> impl Iterator<Item = &PendingTicket> {
        self.tickets.iter()
    }

    /// Count of pending tickets
    pub fn len(&self) -> usize {
        self.tickets.len()
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.tickets.is_empty()
    }

    /// Count of tickets eligible for retry now
    pub fn ready_count(&self) -> usize {
        let now = current_timestamp();
        self.tickets
            .iter()
            .filter(|t| {
                t.retry_count < MAX_RETRY_ATTEMPTS
                    && t.last_retry_at
                        .map(|last| now - last >= MIN_RETRY_INTERVAL_SECS)
                        .unwrap_or(true)
            })
            .count()
    }

    /// Remove tickets that have exceeded max retries
    pub fn prune_exhausted(&mut self) -> Vec<PendingTicket> {
        let mut exhausted = Vec::new();
        self.tickets.retain(|t| {
            if t.retry_count >= MAX_RETRY_ATTEMPTS {
                exhausted.push(t.clone());
                false
            } else {
                true
            }
        });
        exhausted
    }

    /// Load queue from disk
    pub fn load() -> Self {
        let path = queue_path();
        if path.exists() {
            if let Ok(data) = std::fs::read_to_string(&path) {
                if let Ok(queue) = serde_json::from_str(&data) {
                    return queue;
                }
            }
        }
        Self::new()
    }

    /// Save queue to disk
    pub fn save(&self) -> std::io::Result<()> {
        let path = queue_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(self)?;
        std::fs::write(path, data)
    }
}

/// Get path to pending queue file
fn queue_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".anna").join("pending_queue.json")
}

/// Get current Unix timestamp
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Helper to create a pending ticket
pub fn create_pending_ticket(
    case_id: &str,
    query: &str,
    score: u8,
    reason: PendingReason,
    domain: &str,
) -> PendingTicket {
    PendingTicket {
        case_id: case_id.to_string(),
        query: query.to_string(),
        last_score: score,
        retry_count: 0,
        created_at: current_timestamp(),
        last_retry_at: None,
        reason,
        domain: domain.to_string(),
    }
}

/// Check if a result should be queued for retry
pub fn should_queue_for_retry(score: u8, is_timeout: bool, evidence_count: usize) -> bool {
    // Queue if: low score but not terrible, or timeout with some evidence
    (score < RETRY_THRESHOLD && score >= 40) || (is_timeout && evidence_count > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pending_queue_operations() {
        let mut queue = PendingQueue::new();
        assert!(queue.is_empty());

        let ticket = create_pending_ticket(
            "NET-0001",
            "check network",
            55,
            PendingReason::LowReliability,
            "network",
        );
        queue.enqueue(ticket);

        assert_eq!(queue.len(), 1);
        assert!(!queue.is_empty());

        // Should be ready immediately
        assert!(queue.next_ready().is_some());
        assert_eq!(queue.ready_count(), 1);
    }

    #[test]
    fn test_no_duplicates() {
        let mut queue = PendingQueue::new();

        let ticket1 = create_pending_ticket("NET-0001", "q1", 55, PendingReason::Timeout, "net");
        let ticket2 = create_pending_ticket("NET-0001", "q2", 60, PendingReason::Timeout, "net");

        queue.enqueue(ticket1);
        queue.enqueue(ticket2);

        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn test_mark_retried() {
        let mut queue = PendingQueue::new();
        let ticket =
            create_pending_ticket("NET-0001", "query", 55, PendingReason::LowReliability, "net");
        queue.enqueue(ticket);

        queue.mark_retried("NET-0001");

        let t = queue.tickets.front().unwrap();
        assert_eq!(t.retry_count, 1);
        assert!(t.last_retry_at.is_some());
    }

    #[test]
    fn test_should_queue_for_retry() {
        // Low score in acceptable range
        assert!(should_queue_for_retry(55, false, 2));
        // Score too low
        assert!(!should_queue_for_retry(30, false, 2));
        // Score good enough
        assert!(!should_queue_for_retry(75, false, 2));
        // Timeout with evidence
        assert!(should_queue_for_retry(0, true, 1));
        // Timeout without evidence
        assert!(!should_queue_for_retry(0, true, 0));
    }

    #[test]
    fn test_prune_exhausted() {
        let mut queue = PendingQueue::new();
        let mut ticket =
            create_pending_ticket("NET-0001", "query", 55, PendingReason::LowReliability, "net");
        ticket.retry_count = MAX_RETRY_ATTEMPTS;
        queue.tickets.push_back(ticket);

        let exhausted = queue.prune_exhausted();
        assert_eq!(exhausted.len(), 1);
        assert!(queue.is_empty());
    }
}
