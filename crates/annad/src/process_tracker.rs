//! Process Tracker v8.0.0 - Track Process Executions
//!
//! Monitors processes and records execution events when they exit.
//!
//! Rules:
//! - One record per execution (not per sample)
//! - Records CPU and memory at exit time
//! - Tracks start time for duration calculation
//! - Only tracks processes that Anna knows about (on PATH)

use anna_common::{ExecutionRecord, ExecTelemetryWriter};
use chrono::Utc;
use sysinfo::{System, ProcessStatus};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Tracked process state
#[derive(Debug, Clone)]
struct TrackedProcess {
    /// Process name
    name: String,
    /// When we first saw this PID
    start_time: Instant,
    /// Last CPU reading
    last_cpu: f32,
    /// Last memory reading (bytes)
    last_mem: u64,
}

/// Process tracker for recording execution events
pub struct ProcessTracker {
    /// Currently tracked processes (PID -> state)
    tracked: HashMap<u32, TrackedProcess>,
    /// Telemetry writer
    writer: ExecTelemetryWriter,
    /// Set of known executables (from PATH)
    known_executables: Arc<RwLock<std::collections::HashSet<String>>>,
}

impl ProcessTracker {
    /// Create a new process tracker
    pub fn new(known_executables: Arc<RwLock<std::collections::HashSet<String>>>) -> Self {
        Self {
            tracked: HashMap::new(),
            writer: ExecTelemetryWriter::new(),
            known_executables,
        }
    }

    /// Update tracking state and record exits
    pub async fn update(&mut self, system: &System) {
        let now = Instant::now();
        let current_pids: std::collections::HashSet<u32> = system
            .processes()
            .keys()
            .map(|pid| pid.as_u32())
            .collect();

        // Find processes that have exited
        let exited_pids: Vec<u32> = self.tracked
            .keys()
            .filter(|pid| !current_pids.contains(pid))
            .copied()
            .collect();

        // Record exits
        for pid in exited_pids {
            if let Some(tracked) = self.tracked.remove(&pid) {
                let duration_ms = now.duration_since(tracked.start_time).as_millis() as u64;

                let record = ExecutionRecord {
                    timestamp: Utc::now().to_rfc3339(),
                    pid,
                    cpu_percent: if tracked.last_cpu > 0.0 { Some(tracked.last_cpu) } else { None },
                    mem_rss_kb: if tracked.last_mem > 0 { Some(tracked.last_mem / 1024) } else { None },
                    duration_ms: Some(duration_ms),
                    exit_code: None, // Cannot reliably get exit code from /proc after exit
                };

                if let Err(e) = self.writer.record(&tracked.name, &record) {
                    warn!("[!]  Failed to record execution for {}: {}", tracked.name, e);
                } else {
                    debug!("[EXEC] {} (pid={}) exited after {}ms", tracked.name, pid, duration_ms);
                }
            }
        }

        // Check for new processes to track
        let known = self.known_executables.read().await;
        for (pid, process) in system.processes() {
            let pid_u32 = pid.as_u32();

            // Update existing tracked process
            if let Some(tracked) = self.tracked.get_mut(&pid_u32) {
                tracked.last_cpu = process.cpu_usage();
                tracked.last_mem = process.memory();
                continue;
            }

            // Check if this is a new process we should track
            let sysinfo_name = process.name().to_string_lossy().to_string();
            // v7.29.0: Use proper process identity from /proc
            let name = crate::get_process_identity(pid_u32, &sysinfo_name);

            // Only track if it's a known executable
            if known.contains(&name) || known.contains(&sysinfo_name) {
                // Skip if it's a zombie or dead process
                if matches!(process.status(), ProcessStatus::Dead | ProcessStatus::Stop) {
                    continue;
                }

                self.tracked.insert(pid_u32, TrackedProcess {
                    name,
                    start_time: now,
                    last_cpu: process.cpu_usage(),
                    last_mem: process.memory(),
                });
            }
        }
    }

    /// Get count of currently tracked processes
    pub fn tracked_count(&self) -> usize {
        self.tracked.len()
    }

    /// Flush any remaining tracked processes (daemon shutdown)
    pub fn flush(&mut self) {
        let now = Instant::now();
        for (pid, tracked) in self.tracked.drain() {
            let duration_ms = now.duration_since(tracked.start_time).as_millis() as u64;

            let record = ExecutionRecord {
                timestamp: Utc::now().to_rfc3339(),
                pid,
                cpu_percent: if tracked.last_cpu > 0.0 { Some(tracked.last_cpu) } else { None },
                mem_rss_kb: if tracked.last_mem > 0 { Some(tracked.last_mem / 1024) } else { None },
                duration_ms: Some(duration_ms),
                exit_code: None,
            };

            let _ = self.writer.record(&tracked.name, &record);
        }
    }
}

/// Build set of known executables from PATH
pub fn build_known_executables() -> std::collections::HashSet<String> {
    let mut known = std::collections::HashSet::new();

    if let Ok(path_var) = std::env::var("PATH") {
        for dir in path_var.split(':') {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_file() || file_type.is_symlink() {
                            let name = entry.file_name().to_string_lossy().to_string();
                            known.insert(name);
                        }
                    }
                }
            }
        }
    }

    known
}
