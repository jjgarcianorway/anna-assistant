//! Transcript storage (v0.0.431).
//!
//! Persists transcripts to disk with rotation and size limits.

use super::types::{HollywoodTranscript, TranscriptOutcome};
use super::{MAX_TRANSCRIPTS, MAX_TRANSCRIPT_SIZE, TRANSCRIPTS_DIR};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Transcript storage manager
pub struct TranscriptStorage {
    /// Base path for storage
    base_path: PathBuf,
    /// Maximum transcripts to keep
    max_transcripts: usize,
}

impl TranscriptStorage {
    /// Create new storage
    pub fn new(base_path: &str) -> Self {
        Self {
            base_path: PathBuf::from(base_path).join(TRANSCRIPTS_DIR),
            max_transcripts: MAX_TRANSCRIPTS,
        }
    }

    /// Ensure storage directory exists
    fn ensure_dir(&self) -> Result<(), StorageError> {
        fs::create_dir_all(&self.base_path)
            .map_err(|e| StorageError::IoError(e.to_string()))
    }

    /// Save a transcript
    pub fn save(&self, transcript: &HollywoodTranscript) -> Result<PathBuf, StorageError> {
        self.ensure_dir()?;

        let filename = format!("{}.json", transcript.inner.request_id);
        let path = self.base_path.join(&filename);

        let json = serde_json::to_string_pretty(transcript)
            .map_err(|e| StorageError::SerializeError(e.to_string()))?;

        // Check size limit
        if json.len() > MAX_TRANSCRIPT_SIZE {
            return Err(StorageError::SizeLimit(json.len()));
        }

        fs::write(&path, json)
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        // Rotate if needed
        self.rotate_if_needed()?;

        Ok(path)
    }

    /// Load a transcript by request ID
    pub fn load(&self, request_id: &str) -> Result<HollywoodTranscript, StorageError> {
        let filename = format!("{}.json", request_id);
        let path = self.base_path.join(&filename);

        if !path.exists() {
            return Err(StorageError::NotFound(request_id.to_string()));
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        serde_json::from_str(&content)
            .map_err(|e| StorageError::DeserializeError(e.to_string()))
    }

    /// List recent transcripts
    pub fn list_recent(&self, limit: usize) -> Result<Vec<TranscriptSummary>, StorageError> {
        self.ensure_dir()?;

        let mut entries: Vec<_> = fs::read_dir(&self.base_path)
            .map_err(|e| StorageError::IoError(e.to_string()))?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "json").unwrap_or(false))
            .collect();

        // Sort by modification time (newest first)
        entries.sort_by(|a, b| {
            let ma = a.metadata().and_then(|m| m.modified()).ok();
            let mb = b.metadata().and_then(|m| m.modified()).ok();
            mb.cmp(&ma)
        });

        let mut summaries = Vec::new();
        for entry in entries.into_iter().take(limit) {
            if let Ok(summary) = self.load_summary(&entry.path()) {
                summaries.push(summary);
            }
        }

        Ok(summaries)
    }

    /// Load just the summary from a file
    fn load_summary(&self, path: &Path) -> Result<TranscriptSummary, StorageError> {
        let content = fs::read_to_string(path)
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        // Parse minimal fields
        let value: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| StorageError::DeserializeError(e.to_string()))?;

        let request_id = value["request_id"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();
        let user_query = value["user_query"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let outcome: TranscriptOutcome = value["outcome"]
            .as_str()
            .and_then(|s| serde_json::from_str(&format!("\"{}\"", s)).ok())
            .unwrap_or_default();
        let processing_time_ms = value["processing_time_ms"]
            .as_u64()
            .unwrap_or(0);
        let started_at_ms = value["inner"]["started_at_ms"]
            .as_u64()
            .unwrap_or(0);

        Ok(TranscriptSummary {
            request_id,
            user_query,
            outcome,
            processing_time_ms,
            started_at_ms,
        })
    }

    /// Rotate old transcripts if over limit
    fn rotate_if_needed(&self) -> Result<(), StorageError> {
        let entries: Vec<_> = fs::read_dir(&self.base_path)
            .map_err(|e| StorageError::IoError(e.to_string()))?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "json").unwrap_or(false))
            .collect();

        if entries.len() <= self.max_transcripts {
            return Ok(());
        }

        // Get entries with modification times
        let mut with_times: Vec<_> = entries
            .into_iter()
            .filter_map(|e| {
                let mtime = e.metadata().ok()?.modified().ok()?;
                Some((e.path(), mtime))
            })
            .collect();

        // Sort oldest first
        with_times.sort_by(|a, b| a.1.cmp(&b.1));

        // Remove oldest until we're under limit
        let to_remove = with_times.len().saturating_sub(self.max_transcripts);
        for (path, _) in with_times.into_iter().take(to_remove) {
            let _ = fs::remove_file(path);
        }

        Ok(())
    }

    /// Get storage statistics
    pub fn stats(&self) -> Result<StorageStats, StorageError> {
        self.ensure_dir()?;

        let entries: Vec<_> = fs::read_dir(&self.base_path)
            .map_err(|e| StorageError::IoError(e.to_string()))?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "json").unwrap_or(false))
            .collect();

        let mut total_size = 0u64;
        let mut oldest: Option<SystemTime> = None;
        let mut newest: Option<SystemTime> = None;

        for entry in &entries {
            if let Ok(meta) = entry.metadata() {
                total_size += meta.len();
                if let Ok(mtime) = meta.modified() {
                    if oldest.map(|o| mtime < o).unwrap_or(true) {
                        oldest = Some(mtime);
                    }
                    if newest.map(|n| mtime > n).unwrap_or(true) {
                        newest = Some(mtime);
                    }
                }
            }
        }

        Ok(StorageStats {
            count: entries.len(),
            total_size_bytes: total_size,
            max_transcripts: self.max_transcripts,
            oldest_timestamp: oldest.and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
                .map(|d| d.as_secs()),
            newest_timestamp: newest.and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
                .map(|d| d.as_secs()),
        })
    }

    /// Delete a transcript
    pub fn delete(&self, request_id: &str) -> Result<(), StorageError> {
        let filename = format!("{}.json", request_id);
        let path = self.base_path.join(&filename);

        if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| StorageError::IoError(e.to_string()))?;
        }
        Ok(())
    }

    /// Clear all transcripts
    pub fn clear(&self) -> Result<usize, StorageError> {
        let entries: Vec<_> = fs::read_dir(&self.base_path)
            .map_err(|e| StorageError::IoError(e.to_string()))?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "json").unwrap_or(false))
            .collect();

        let count = entries.len();
        for entry in entries {
            let _ = fs::remove_file(entry.path());
        }

        Ok(count)
    }
}

/// Transcript summary (minimal info for listing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptSummary {
    pub request_id: String,
    pub user_query: String,
    pub outcome: TranscriptOutcome,
    pub processing_time_ms: u64,
    pub started_at_ms: u64,
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub count: usize,
    pub total_size_bytes: u64,
    pub max_transcripts: usize,
    pub oldest_timestamp: Option<u64>,
    pub newest_timestamp: Option<u64>,
}

/// Storage errors
#[derive(Debug, Clone)]
pub enum StorageError {
    IoError(String),
    SerializeError(String),
    DeserializeError(String),
    NotFound(String),
    SizeLimit(usize),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "I/O error: {}", e),
            Self::SerializeError(e) => write!(f, "Serialize error: {}", e),
            Self::DeserializeError(e) => write!(f, "Deserialize error: {}", e),
            Self::NotFound(id) => write!(f, "Transcript not found: {}", id),
            Self::SizeLimit(size) => write!(f, "Transcript too large: {} bytes", size),
        }
    }
}

impl std::error::Error for StorageError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_path() -> String {
        format!("/tmp/anna_transcript_test_{}", std::process::id())
    }

    fn cleanup(path: &str) {
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn test_save_and_load() {
        let path = test_path();
        let storage = TranscriptStorage::new(&path);

        let mut t = HollywoodTranscript::new("REQ-001", "how much ram?");
        t.set_answer("You have 16GB");
        t.set_handler("Sofia", "Desktop");
        t.set_confidence(0.95);

        storage.save(&t).unwrap();

        let loaded = storage.load("REQ-001").unwrap();
        assert_eq!(loaded.user_query, "how much ram?");
        assert!(loaded.final_answer.is_some());

        cleanup(&path);
    }

    #[test]
    fn test_list_recent() {
        let path = test_path();
        cleanup(&path); // Ensure clean start
        let storage = TranscriptStorage::new(&path);

        for i in 0..5 {
            let mut t = HollywoodTranscript::new(&format!("REQ-{:03}", i), "test query");
            t.finalize();
            storage.save(&t).unwrap();
        }

        let recent = storage.list_recent(3).unwrap();
        assert_eq!(recent.len(), 3);

        cleanup(&path);
    }
}
