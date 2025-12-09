//! Snapshot history (v0.0.225).

use crate::snapshot::SystemSnapshot;

use super::summary::HealthSummary;
use super::types::HealthDelta;

/// Maximum snapshots to keep in history
const MAX_HISTORY_SIZE: usize = 5;

/// In-memory snapshot history - stores last N snapshots, rotated on refresh.
#[derive(Debug, Clone, Default)]
pub struct SnapshotHistory {
    snapshots: Vec<SystemSnapshot>,
}

impl SnapshotHistory {
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
        }
    }

    pub fn push(&mut self, snapshot: SystemSnapshot) {
        self.snapshots.push(snapshot);
        while self.snapshots.len() > MAX_HISTORY_SIZE {
            self.snapshots.remove(0);
        }
    }

    pub fn latest(&self) -> Option<&SystemSnapshot> {
        self.snapshots.last()
    }

    pub fn previous(&self) -> Option<&SystemSnapshot> {
        (self.snapshots.len() >= 2).then(|| &self.snapshots[self.snapshots.len() - 2])
    }

    pub fn get_back(&self, n: usize) -> Option<&SystemSnapshot> {
        (n < self.snapshots.len()).then(|| &self.snapshots[self.snapshots.len() - 1 - n])
    }

    pub fn all(&self) -> &[SystemSnapshot] {
        &self.snapshots
    }

    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }

    pub fn latest_delta(&self) -> Option<HealthDelta> {
        Some(HealthDelta::from_snapshots(
            self.previous()?,
            self.latest()?,
        ))
    }

    pub fn health_summary(&self) -> HealthSummary {
        HealthSummary {
            snapshot: self.latest().cloned(),
            delta: self.latest_delta(),
            history_count: self.len(),
        }
    }
}
