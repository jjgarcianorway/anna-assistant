//! Future Planning - Anna schedules data collection and delivers results later.
//!
//! Tracks promises Anna makes and ensures delivery.

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{info, warn};

use crate::opportunity_detector::Proposal;

const DELIVERABLES_DB: &str = "/var/lib/anna/deliverables.json";

/// A deliverable Anna promised to create.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deliverable {
    /// Unique ID
    pub id: String,
    /// Original proposal
    pub proposal: Proposal,
    /// When this was scheduled
    pub scheduled_at: DateTime<Utc>,
    /// When this should be delivered
    pub deliver_at: DateTime<Utc>,
    /// Current status
    pub status: DeliverableStatus,
    /// User who requested this
    pub requested_by: Option<String>,
    /// Additional context
    pub context: String,
}

/// Status of a deliverable.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeliverableStatus {
    /// Collecting data
    Collecting,
    /// Ready to generate
    ReadyToGenerate,
    /// Generating report
    Generating,
    /// Delivered to user
    Delivered,
    /// Cancelled by user
    Cancelled,
}

/// Database of deliverables.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeliverableDatabase {
    pub deliverables: Vec<Deliverable>,
}

impl DeliverableDatabase {
    /// Load from disk.
    pub fn load() -> Self {
        let path = PathBuf::from(DELIVERABLES_DB);
        if !path.exists() {
            return Self::default();
        }
        
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }
    
    /// Save to disk.
    pub fn save(&self) -> Result<()> {
        let path = PathBuf::from(DELIVERABLES_DB);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        Ok(())
    }
    
    /// Add a new deliverable.
    pub fn add(&mut self, deliverable: Deliverable) {
        info!("Scheduled deliverable: {} (ready {})", 
            deliverable.id, deliverable.deliver_at.format("%Y-%m-%d"));
        self.deliverables.push(deliverable);
    }
    
    /// Get deliverables ready for delivery.
    pub fn ready_for_delivery(&self) -> Vec<&Deliverable> {
        let now = Utc::now();
        self.deliverables
            .iter()
            .filter(|d| d.status == DeliverableStatus::ReadyToGenerate && d.deliver_at <= now)
            .collect()
    }
    
    /// Mark deliverable as delivered.
    pub fn mark_delivered(&mut self, id: &str) {
        if let Some(deliverable) = self.deliverables.iter_mut().find(|d| d.id == id) {
            deliverable.status = DeliverableStatus::Delivered;
            info!("Marked deliverable {} as delivered", id);
        }
    }
    
    /// Cancel a deliverable.
    pub fn cancel(&mut self, id: &str) -> Result<()> {
        if let Some(deliverable) = self.deliverables.iter_mut().find(|d| d.id == id) {
            deliverable.status = DeliverableStatus::Cancelled;
            info!("Cancelled deliverable {}", id);
            Ok(())
        } else {
            Err(anyhow!("Deliverable {} not found", id))
        }
    }
    
    /// Get active deliverables (not delivered or cancelled).
    pub fn active(&self) -> Vec<&Deliverable> {
        self.deliverables
            .iter()
            .filter(|d| d.status != DeliverableStatus::Delivered 
                     && d.status != DeliverableStatus::Cancelled)
            .collect()
    }
}

/// Schedule a proposal as a future deliverable.
pub fn schedule_proposal(
    proposal: Proposal,
    username: Option<String>,
    context: String,
) -> Result<()> {
    let mut db = DeliverableDatabase::load();
    
    let deliver_at = proposal.available_at.unwrap_or_else(|| Utc::now());
    
    let deliverable = Deliverable {
        id: proposal.id.clone(),
        proposal,
        scheduled_at: Utc::now(),
        deliver_at,
        status: DeliverableStatus::Collecting,
        requested_by: username,
        context,
    };
    
    db.add(deliverable);
    db.save()?;
    
    Ok(())
}

/// Generate and deliver ready deliverables.
pub async fn process_ready_deliverables() -> Result<Vec<String>> {
    let mut db = DeliverableDatabase::load();
    let ready: Vec<Deliverable> = db.ready_for_delivery().iter().map(|d| (*d).clone()).collect();

    let mut delivered = Vec::new();

    for deliverable in ready {
        info!("Processing deliverable: {}", deliverable.id);

        match generate_deliverable(&deliverable).await {
            Ok(report) => {
                // Send via Telegram
                crate::telegram::notifier::push_notification(&report);

                db.mark_delivered(&deliverable.id);
                delivered.push(deliverable.id.clone());
            }
            Err(e) => {
                warn!("Failed to generate deliverable {}: {}", deliverable.id, e);
            }
        }
    }

    if !delivered.is_empty() {
        db.save()?;
    }

    Ok(delivered)
}

/// Generate a deliverable report.
async fn generate_deliverable(deliverable: &Deliverable) -> Result<String> {
    let proposal = &deliverable.proposal;
    
    // Determine what type of report to generate based on proposal ID
    if proposal.id.contains("boot") {
        generate_boot_report(deliverable).await
    } else if proposal.id.contains("memory") {
        generate_memory_report(deliverable).await
    } else if proposal.id.contains("disk") {
        generate_disk_report(deliverable).await
    } else if proposal.id.contains("cpu") || proposal.id.contains("load") {
        generate_cpu_report(deliverable).await
    } else {
        generate_generic_report(deliverable).await
    }
}

/// Generate boot performance report.
async fn generate_boot_report(deliverable: &Deliverable) -> Result<String> {
    let history = anna_shared::monitor::LongTermHistory::load();
    
    if history.daily_snapshots.is_empty() {
        return Err(anyhow!("No boot data available"));
    }
    
    let days = history.daily_snapshots.len();
    let boot_times: Vec<f32> = history.daily_snapshots.iter()
        .map(|s| s.avg_boot_time)
        .collect();
    
    let avg_boot = boot_times.iter().sum::<f32>() / boot_times.len() as f32;
    let min_boot = boot_times.iter().fold(f32::INFINITY, |a, &b| a.min(b));
    let max_boot = boot_times.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
    
    // Simple trend analysis
    let first_week = boot_times.iter().take(7).sum::<f32>() / 7.0;
    let last_week = boot_times.iter().rev().take(7).sum::<f32>() / 7.0;
    let trend = if last_week > first_week + 1.0 {
        format!("increasing ({:.1}s slower)", last_week - first_week)
    } else if first_week > last_week + 1.0 {
        format!("improving ({:.1}s faster)", first_week - last_week)
    } else {
        "stable".to_string()
    };
    
    let report = format!(
        "Boot Performance Report ({} days)\n\n\
        Average: {:.1}s\n\
        Best: {:.1}s\n\
        Worst: {:.1}s\n\
        Trend: {}\n\n\
        This report was scheduled on {} as requested.",
        days, avg_boot, min_boot, max_boot, trend,
        deliverable.scheduled_at.format("%B %d")
    );
    
    Ok(report)
}

/// Generate memory usage report.
async fn generate_memory_report(deliverable: &Deliverable) -> Result<String> {
    let history = anna_shared::monitor::LongTermHistory::load();
    
    if history.daily_snapshots.is_empty() {
        return Err(anyhow!("No memory data available"));
    }
    
    let days = history.daily_snapshots.len();
    let memory_usage: Vec<f32> = history.daily_snapshots.iter()
        .map(|s| s.avg_memory_pct)
        .collect();
    
    let avg_mem = memory_usage.iter().sum::<f32>() / memory_usage.len() as f32;
    let min_mem = memory_usage.iter().fold(f32::INFINITY, |a, &b| a.min(b));
    let max_mem = memory_usage.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
    
    let report = format!(
        "Memory Usage Report ({} days)\n\n\
        Average: {:.1}%\n\
        Minimum: {:.1}%\n\
        Maximum: {:.1}%\n\
        Range: {:.1}%\n\n\
        Scheduled on {} as requested.",
        days, avg_mem, min_mem, max_mem, max_mem - min_mem,
        deliverable.scheduled_at.format("%B %d")
    );
    
    Ok(report)
}

/// Generate disk usage report.
async fn generate_disk_report(deliverable: &Deliverable) -> Result<String> {
    let history = anna_shared::monitor::LongTermHistory::load();
    
    if history.daily_snapshots.is_empty() {
        return Err(anyhow!("No disk data available"));
    }
    
    let days = history.daily_snapshots.len();
    let disk_usage: Vec<f32> = history.daily_snapshots.iter()
        .map(|s| s.disk_used_gb)
        .collect();
    
    let first = disk_usage.first().unwrap();
    let last = disk_usage.last().unwrap();
    let growth = last - first;
    let growth_per_day = growth / days as f32;
    
    let report = format!(
        "Disk Usage Report ({} days)\n\n\
        Start: {:.1} GB\n\
        Current: {:.1} GB\n\
        Growth: {:.1} GB total ({:.2} GB/day)\n\n\
        At this rate, you'll use +{:.1} GB in the next 30 days.\n\n\
        Scheduled on {} as requested.",
        days, first, last, growth, growth_per_day, growth_per_day * 30.0,
        deliverable.scheduled_at.format("%B %d")
    );
    
    Ok(report)
}

/// Generate CPU/load report.
async fn generate_cpu_report(deliverable: &Deliverable) -> Result<String> {
    let history = anna_shared::monitor::LongTermHistory::load();
    
    if history.daily_snapshots.is_empty() {
        return Err(anyhow!("No CPU data available"));
    }
    
    let days = history.daily_snapshots.len();
    let load_avg: Vec<f32> = history.daily_snapshots.iter()
        .map(|s| s.avg_load)
        .collect();
    
    let avg_load = load_avg.iter().sum::<f32>() / load_avg.len() as f32;
    let max_load = load_avg.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
    
    let report = format!(
        "CPU Load Report ({} days)\n\n\
        Average load: {:.2}\n\
        Peak load: {:.2}\n\n\
        Scheduled on {} as requested.",
        days, avg_load, max_load,
        deliverable.scheduled_at.format("%B %d")
    );
    
    Ok(report)
}

/// Generate generic report.
async fn generate_generic_report(deliverable: &Deliverable) -> Result<String> {
    Ok(format!(
        "Deliverable: {}\n\n\
        Status: Ready\n\
        Scheduled: {}\n\
        Context: {}\n\n\
        The data collection period is complete.",
        deliverable.proposal.deliverable,
        deliverable.scheduled_at.format("%B %d at %H:%M"),
        deliverable.context
    ))
}

/// List active deliverables for user.
pub fn list_active_deliverables() -> String {
    let db = DeliverableDatabase::load();
    let active = db.active();
    
    if active.is_empty() {
        return "No scheduled deliverables.".to_string();
    }
    
    let mut response = format!("Active Deliverables ({}):\n\n", active.len());
    
    for (i, deliverable) in active.iter().enumerate() {
        let status_str = match deliverable.status {
            DeliverableStatus::Collecting => "Collecting data",
            DeliverableStatus::ReadyToGenerate => "Ready to generate",
            DeliverableStatus::Generating => "Generating",
            _ => "Unknown",
        };
        
        response.push_str(&format!(
            "{}. {} ({})\n   Ready: {}\n   Status: {}\n\n",
            i + 1,
            deliverable.proposal.deliverable,
            deliverable.id,
            deliverable.deliver_at.format("%B %d"),
            status_str
        ));
    }
    
    response
}
