// v0.0.611: Settings Queue (Phase 187)
// Queue management for settings operations

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use crate::unified_settings::SettingsCategory;

/// Queue priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum QueuePriority {
    /// Critical priority
    Critical,
    /// High priority
    High,
    /// Normal priority
    #[default]
    Normal,
    /// Low priority
    Low,
    /// Background priority
    Background,
}

impl std::fmt::Display for QueuePriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Critical => write!(f, "critical"),
            Self::High => write!(f, "high"),
            Self::Normal => write!(f, "normal"),
            Self::Low => write!(f, "low"),
            Self::Background => write!(f, "background"),
        }
    }
}

/// Queue item status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum QueueItemStatus {
    /// Queued
    #[default]
    Queued,
    /// Processing
    Processing,
    /// Completed
    Completed,
    /// Failed
    Failed,
    /// Retrying
    Retrying,
}

impl std::fmt::Display for QueueItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Queued => write!(f, "queued"),
            Self::Processing => write!(f, "processing"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::Retrying => write!(f, "retrying"),
        }
    }
}

/// Queue operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueueOperation {
    /// Read operation
    Read,
    /// Write operation
    Write,
    /// Update operation
    Update,
    /// Delete operation
    Delete,
    /// Sync operation
    Sync,
}

impl std::fmt::Display for QueueOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read => write!(f, "read"),
            Self::Write => write!(f, "write"),
            Self::Update => write!(f, "update"),
            Self::Delete => write!(f, "delete"),
            Self::Sync => write!(f, "sync"),
        }
    }
}

/// Queue item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueItem {
    /// Unique ID
    pub id: String,
    /// Operation
    pub operation: QueueOperation,
    /// Priority
    pub priority: QueuePriority,
    /// Status
    pub status: QueueItemStatus,
    /// Category
    pub category: Option<SettingsCategory>,
    /// Key
    pub key: Option<String>,
    /// Payload
    pub payload: Option<String>,
    /// Retry count
    pub retries: u32,
    /// Max retries
    pub max_retries: u32,
    /// Created timestamp
    pub created_at: u64,
}

impl QueueItem {
    /// Create new item
    pub fn new(id: impl Into<String>, operation: QueueOperation) -> Self {
        Self {
            id: id.into(),
            operation,
            priority: QueuePriority::Normal,
            status: QueueItemStatus::Queued,
            category: None,
            key: None,
            payload: None,
            retries: 0,
            max_retries: 3,
            created_at: 0,
        }
    }

    /// Set priority
    pub fn priority(mut self, priority: QueuePriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set key
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Set payload
    pub fn payload(mut self, payload: impl Into<String>) -> Self {
        self.payload = Some(payload.into());
        self
    }

    /// Set max retries
    pub fn max_retries(mut self, max: u32) -> Self {
        self.max_retries = max;
        self
    }

    /// Mark processing
    pub fn mark_processing(&mut self) {
        self.status = QueueItemStatus::Processing;
    }

    /// Mark completed
    pub fn mark_completed(&mut self) {
        self.status = QueueItemStatus::Completed;
    }

    /// Mark failed
    pub fn mark_failed(&mut self) {
        self.status = QueueItemStatus::Failed;
    }

    /// Retry
    pub fn retry(&mut self) -> bool {
        if self.retries < self.max_retries {
            self.retries += 1;
            self.status = QueueItemStatus::Retrying;
            true
        } else {
            false
        }
    }

    /// Can retry
    pub fn can_retry(&self) -> bool {
        self.retries < self.max_retries
    }
}

/// Settings queue
#[derive(Debug, Clone, Default)]
pub struct SettingsQueue {
    /// Items
    items: VecDeque<QueueItem>,
    /// Max size
    max_size: usize,
    /// Processed count
    processed: usize,
    /// Failed count
    failed: usize,
}

impl SettingsQueue {
    /// Create new queue
    pub fn new() -> Self {
        Self {
            max_size: 1000,
            ..Default::default()
        }
    }

    /// Enqueue item
    pub fn enqueue(&mut self, item: QueueItem) -> bool {
        if self.items.len() >= self.max_size {
            return false;
        }
        self.items.push_back(item);
        self.sort_by_priority();
        true
    }

    /// Dequeue item
    pub fn dequeue(&mut self) -> Option<QueueItem> {
        self.items.pop_front()
    }

    /// Peek front
    pub fn peek(&self) -> Option<&QueueItem> {
        self.items.front()
    }

    /// Sort by priority
    fn sort_by_priority(&mut self) {
        let mut items: Vec<_> = self.items.drain(..).collect();
        items.sort_by_key(|i| i.priority);
        self.items = items.into();
    }

    /// Length
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Processed count
    pub fn processed_count(&self) -> usize {
        self.processed
    }

    /// Failed count
    pub fn failed_count(&self) -> usize {
        self.failed
    }

    /// Mark processed
    pub fn mark_processed(&mut self) {
        self.processed += 1;
    }

    /// Mark failed
    pub fn mark_failed(&mut self) {
        self.failed += 1;
    }

    /// Clear
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// By priority
    pub fn by_priority(&self, priority: QueuePriority) -> Vec<&QueueItem> {
        self.items.iter().filter(|i| i.priority == priority).collect()
    }
}

/// Format queue
pub fn format_queue(queue: &SettingsQueue) -> String {
    let mut output = String::new();
    output.push_str("Settings Queue:\n");
    output.push_str(&format!("  Items: {}\n", queue.len()));
    output.push_str(&format!("  Processed: {}\n", queue.processed_count()));
    output.push_str(&format!("  Failed: {}\n", queue.failed_count()));
    output
}

/// Check if query is about queue
pub fn is_queue_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("queue")
        || lower.contains("pending operations")
        || lower.contains("job queue")
}

/// Fun fact about queue
pub fn queue_fun_fact() -> &'static str {
    "Anna uses priority queues to ensure critical settings changes are processed first!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_display() {
        assert_eq!(format!("{}", QueuePriority::Critical), "critical");
        assert_eq!(format!("{}", QueuePriority::Normal), "normal");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", QueueItemStatus::Queued), "queued");
        assert_eq!(format!("{}", QueueItemStatus::Processing), "processing");
    }

    #[test]
    fn test_operation_display() {
        assert_eq!(format!("{}", QueueOperation::Read), "read");
        assert_eq!(format!("{}", QueueOperation::Write), "write");
    }

    #[test]
    fn test_item_new() {
        let i = QueueItem::new("i1", QueueOperation::Write);
        assert_eq!(i.status, QueueItemStatus::Queued);
    }

    #[test]
    fn test_item_builder() {
        let i = QueueItem::new("i1", QueueOperation::Update)
            .priority(QueuePriority::High)
            .key("test_key")
            .max_retries(5);
        assert_eq!(i.max_retries, 5);
    }

    #[test]
    fn test_item_retry() {
        let mut i = QueueItem::new("i1", QueueOperation::Write).max_retries(2);
        assert!(i.retry());
        assert!(i.retry());
        assert!(!i.retry());
    }

    #[test]
    fn test_queue_new() {
        let q = SettingsQueue::new();
        assert!(q.is_empty());
    }

    #[test]
    fn test_queue_enqueue_dequeue() {
        let mut q = SettingsQueue::new();
        q.enqueue(QueueItem::new("i1", QueueOperation::Read));
        assert_eq!(q.len(), 1);
        let item = q.dequeue();
        assert!(item.is_some());
        assert!(q.is_empty());
    }

    #[test]
    fn test_queue_priority_order() {
        let mut q = SettingsQueue::new();
        q.enqueue(QueueItem::new("i1", QueueOperation::Read).priority(QueuePriority::Low));
        q.enqueue(QueueItem::new("i2", QueueOperation::Read).priority(QueuePriority::Critical));
        let item = q.dequeue().unwrap();
        assert_eq!(item.priority, QueuePriority::Critical);
    }

    #[test]
    fn test_is_queue_query() {
        assert!(is_queue_query("show queue"));
        assert!(!is_queue_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = queue_fun_fact();
        assert!(fact.contains("queue"));
    }
}
