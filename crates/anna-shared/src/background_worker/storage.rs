//! Job persistence storage (v0.0.430).

use super::job::BackgroundJob;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Storage for background jobs
#[derive(Debug)]
pub struct JobStorage {
    /// Base storage path
    path: PathBuf,
}

impl JobStorage {
    /// Create new storage at path
    pub fn new(base_path: &str) -> Self {
        let path = PathBuf::from(base_path);
        Self { path }
    }

    /// Get jobs file path
    fn jobs_file(&self) -> PathBuf {
        self.path.join(super::JOBS_FILE)
    }

    /// Load jobs from disk
    pub fn load(&self) -> Result<HashMap<String, BackgroundJob>, StorageError> {
        let file_path = self.jobs_file();

        if !file_path.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(&file_path)
            .map_err(|e| StorageError::ReadError(e.to_string()))?;

        let jobs: HashMap<String, BackgroundJob> = serde_json::from_str(&content)
            .map_err(|e| StorageError::ParseError(e.to_string()))?;

        Ok(jobs)
    }

    /// Save jobs to disk
    pub fn save(&self, jobs: &HashMap<String, BackgroundJob>) -> Result<(), std::io::Error> {
        // Ensure directory exists
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::create_dir_all(&self.path)?;

        let content = serde_json::to_string_pretty(jobs)?;
        fs::write(self.jobs_file(), content)
    }

    /// Check if storage exists
    pub fn exists(&self) -> bool {
        self.jobs_file().exists()
    }
}

/// Pending message storage for user notifications
#[derive(Debug)]
pub struct PendingMessageStorage {
    path: PathBuf,
}

impl PendingMessageStorage {
    /// Create new storage
    pub fn new(base_path: &str) -> Self {
        Self {
            path: PathBuf::from(base_path),
        }
    }

    /// Get messages file path
    fn messages_file(&self) -> PathBuf {
        self.path.join(super::PENDING_MESSAGES_FILE)
    }

    /// Load pending messages
    pub fn load(&self) -> Result<Vec<PendingMessage>, StorageError> {
        let file_path = self.messages_file();

        if !file_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&file_path)
            .map_err(|e| StorageError::ReadError(e.to_string()))?;

        let messages: Vec<PendingMessage> = serde_json::from_str(&content)
            .map_err(|e| StorageError::ParseError(e.to_string()))?;

        Ok(messages)
    }

    /// Save pending messages
    pub fn save(&self, messages: &[PendingMessage]) -> Result<(), std::io::Error> {
        fs::create_dir_all(&self.path)?;
        let content = serde_json::to_string_pretty(messages)?;
        fs::write(self.messages_file(), content)
    }

    /// Add a message
    pub fn add(&self, message: PendingMessage) -> Result<(), StorageError> {
        let mut messages = self.load()?;
        messages.push(message);
        self.save(&messages)
            .map_err(|e| StorageError::WriteError(e.to_string()))
    }

    /// Take all messages (clears the queue)
    pub fn take_all(&self) -> Result<Vec<PendingMessage>, StorageError> {
        let messages = self.load()?;
        if !messages.is_empty() {
            self.save(&[])
                .map_err(|e| StorageError::WriteError(e.to_string()))?;
        }
        Ok(messages)
    }

    /// Count pending messages
    pub fn count(&self) -> usize {
        self.load().map(|m| m.len()).unwrap_or(0)
    }
}

/// A pending message for the user
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PendingMessage {
    /// Message ID
    pub id: String,
    /// Subject/title
    pub subject: String,
    /// Message body
    pub body: String,
    /// When created (unix timestamp)
    pub created_at: u64,
    /// Source (e.g., "long_ticket:TKT-123", "monitor:disk_full")
    pub source: String,
    /// Priority
    pub priority: MessagePriority,
    /// Whether this has been shown
    #[serde(default)]
    pub shown: bool,
}

/// Message priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessagePriority {
    Low,
    Normal,
    High,
}

impl PendingMessage {
    /// Create a new message
    pub fn new(subject: &str, body: &str, source: &str) -> Self {
        Self {
            id: format!(
                "MSG-{}",
                uuid::Uuid::new_v4().to_string()[..8].to_uppercase()
            ),
            subject: subject.to_string(),
            body: body.to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            source: source.to_string(),
            priority: MessagePriority::Normal,
            shown: false,
        }
    }

    /// Set priority
    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }

    /// From long ticket completion
    pub fn from_long_ticket(ticket_id: &str, summary: &str) -> Self {
        Self::new(
            &format!("Analysis Complete: {}", ticket_id),
            summary,
            &format!("long_ticket:{}", ticket_id),
        )
    }

    /// From monitor alert
    pub fn from_monitor(monitor_id: &str, alert: &str) -> Self {
        Self::new(
            &format!("Alert: {}", monitor_id),
            alert,
            &format!("monitor:{}", monitor_id),
        )
        .with_priority(MessagePriority::High)
    }
}

/// Storage errors
#[derive(Debug, Clone)]
pub enum StorageError {
    ReadError(String),
    WriteError(String),
    ParseError(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadError(e) => write!(f, "Read error: {}", e),
            Self::WriteError(e) => write!(f, "Write error: {}", e),
            Self::ParseError(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for StorageError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn test_path() -> String {
        format!("/tmp/anna_storage_test_{}", std::process::id())
    }

    fn cleanup(path: &str) {
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn test_job_storage_roundtrip() {
        let path = test_path();
        let storage = JobStorage::new(&path);

        let mut jobs = HashMap::new();
        jobs.insert(
            "JOB-1".to_string(),
            BackgroundJob::doc_refresh(),
        );

        storage.save(&jobs).unwrap();
        let loaded = storage.load().unwrap();

        assert_eq!(loaded.len(), 1);
        assert!(loaded.contains_key("JOB-1"));

        cleanup(&path);
    }

    #[test]
    fn test_pending_messages() {
        let path = test_path();
        let storage = PendingMessageStorage::new(&path);

        let msg = PendingMessage::new("Test", "Body", "test");
        storage.add(msg).unwrap();

        assert_eq!(storage.count(), 1);

        let messages = storage.take_all().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(storage.count(), 0);

        cleanup(&path);
    }
}
