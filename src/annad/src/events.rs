// Anna v0.11.0 - Event Engine
//
// Centralized event system with debouncing, coalescing, and domain-based routing.
// Converts system changes into semantic triggers for the doctor/repair pipeline.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// Event domains (subsystems that can trigger doctor checks)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventDomain {
    Packages, // Package manager changes
    Config,   // /etc configuration drift
    Devices,  // USB, block, net, bluetooth hotplug
    Network,  // Link state, IP address changes
    Storage,  // Mount/unmount, filesystem changes
    Kernel,   // Kernel/initramfs updates
}

impl EventDomain {
    pub fn as_str(&self) -> &str {
        match self {
            EventDomain::Packages => "packages",
            EventDomain::Config => "config",
            EventDomain::Devices => "devices",
            EventDomain::Network => "network",
            EventDomain::Storage => "storage",
            EventDomain::Kernel => "kernel",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "packages" => Some(EventDomain::Packages),
            "config" => Some(EventDomain::Config),
            "devices" => Some(EventDomain::Devices),
            "network" => Some(EventDomain::Network),
            "storage" => Some(EventDomain::Storage),
            "kernel" => Some(EventDomain::Kernel),
            _ => None,
        }
    }
}

/// A system event that triggers doctor checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEvent {
    pub domain: EventDomain,
    pub cause: String,                     // What triggered this event
    pub timestamp: i64,                    // Unix timestamp
    pub metadata: HashMap<String, String>, // Additional context
}

/// Result of processing an event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventResult {
    pub event: SystemEvent,
    pub doctor_result: DoctorResult,
    pub repair_result: Option<RepairResult>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorResult {
    pub alerts_found: usize,
    pub degraded_modules: Vec<String>,
    pub action_taken: String, // "auto_repair", "alert_only", "no_action"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairResult {
    pub success: bool,
    pub message: String,
    pub alerts_cleared: usize,
}

/// Event queue with coalescing and cooldown
pub struct EventQueue {
    pending: Arc<Mutex<HashMap<EventDomain, PendingEvent>>>,
    cooldowns: Arc<Mutex<HashMap<EventDomain, Instant>>>,
    debounce_ms: u64,
    cooldown_secs: u64,
}

struct PendingEvent {
    events: Vec<SystemEvent>,
    first_seen: Instant,
}

impl EventQueue {
    pub fn new(debounce_ms: u64, cooldown_secs: u64) -> Self {
        Self {
            pending: Arc::new(Mutex::new(HashMap::new())),
            cooldowns: Arc::new(Mutex::new(HashMap::new())),
            debounce_ms,
            cooldown_secs,
        }
    }

    /// Enqueue an event (may coalesce with pending events)
    pub fn enqueue(&self, event: SystemEvent) {
        let mut pending = self.pending.lock().unwrap();
        let domain = event.domain.clone();

        // Check if in cooldown
        {
            let cooldowns = self.cooldowns.lock().unwrap();
            if let Some(last_run) = cooldowns.get(&domain) {
                if last_run.elapsed() < Duration::from_secs(self.cooldown_secs) {
                    debug!(
                        "Domain {:?} in cooldown, dropping event: {}",
                        domain, event.cause
                    );
                    return;
                }
            }
        }

        // Add to pending
        pending
            .entry(domain)
            .or_insert_with(|| PendingEvent {
                events: Vec::new(),
                first_seen: Instant::now(),
            })
            .events
            .push(event);
    }

    /// Drain events that have passed the debounce window
    pub fn drain_ready(&self) -> Vec<(EventDomain, Vec<SystemEvent>)> {
        let mut pending = self.pending.lock().unwrap();
        let mut ready = Vec::new();
        let now = Instant::now();

        let debounce_duration = Duration::from_millis(self.debounce_ms);

        pending.retain(|domain, pending_event| {
            if now.duration_since(pending_event.first_seen) >= debounce_duration {
                // Ready to process
                ready.push((domain.clone(), pending_event.events.clone()));
                false // Remove from pending
            } else {
                true // Keep in pending
            }
        });

        // Set cooldown for drained domains
        if !ready.is_empty() {
            let mut cooldowns = self.cooldowns.lock().unwrap();
            for (domain, _) in &ready {
                cooldowns.insert(domain.clone(), Instant::now());
            }
        }

        ready
    }

    /// Get current pending count (for monitoring)
    pub fn pending_count(&self) -> usize {
        self.pending.lock().unwrap().len()
    }
}

/// Shared state that can be accessed by RPC server
pub struct EventEngineState {
    history: Arc<Mutex<VecDeque<EventResult>>>,
    queue: Arc<EventQueue>,
}

impl EventEngineState {
    pub fn get_history(&self, limit: usize) -> Vec<EventResult> {
        let hist = self.history.lock().unwrap();
        hist.iter().rev().take(limit).cloned().collect()
    }

    pub fn pending_count(&self) -> usize {
        self.queue.pending.lock().unwrap().len()
    }

    /// Calculate event processing rate (events/sec) based on recent history
    pub fn event_rate_per_sec(&self) -> f64 {
        use std::time::{SystemTime, UNIX_EPOCH};

        let hist = self.history.lock().unwrap();
        if hist.is_empty() {
            return 0.0;
        }

        // Get current time as Unix timestamp
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Count events in last 60 seconds
        let window_secs = 60;
        let cutoff = now - window_secs;

        let recent_count = hist
            .iter()
            .filter(|result| result.event.timestamp >= cutoff)
            .count();

        // Calculate rate
        if recent_count > 0 {
            recent_count as f64 / window_secs as f64
        } else {
            0.0
        }
    }

    /// Get age of oldest pending event in seconds
    pub fn oldest_pending_event_sec(&self) -> u64 {
        use std::time::Instant;

        let pending = self.queue.pending.lock().unwrap();
        if pending.is_empty() {
            return 0;
        }

        // Find oldest first_seen timestamp
        let oldest = pending
            .values()
            .map(|p| p.first_seen)
            .min()
            .unwrap_or_else(Instant::now);

        oldest.elapsed().as_secs()
    }
}

/// Event engine coordinator
pub struct EventEngine {
    queue: Arc<EventQueue>,
    tx: mpsc::UnboundedSender<SystemEvent>,
    rx: Option<mpsc::UnboundedReceiver<SystemEvent>>,
    history: Arc<Mutex<VecDeque<EventResult>>>,
    max_history: usize,
}

impl EventEngine {
    pub fn new(debounce_ms: u64, cooldown_secs: u64, max_history: usize) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        Self {
            queue: Arc::new(EventQueue::new(debounce_ms, cooldown_secs)),
            tx,
            rx: Some(rx),
            history: Arc::new(Mutex::new(VecDeque::with_capacity(max_history))),
            max_history,
        }
    }

    /// Get a sender for submitting events
    pub fn sender(&self) -> mpsc::UnboundedSender<SystemEvent> {
        self.tx.clone()
    }

    /// Get a shared state handle (for RPC server)
    pub fn shared_state(&self) -> EventEngineState {
        EventEngineState {
            history: Arc::clone(&self.history),
            queue: Arc::clone(&self.queue),
        }
    }

    /// Start the event processing loop
    pub async fn run(mut self, doctor_handler: Arc<dyn DoctorHandler + Send + Sync>) -> Result<()> {
        info!(
            "Event engine starting (debounce: {}ms, cooldown: {}s)",
            self.queue.debounce_ms, self.queue.cooldown_secs
        );

        let queue = Arc::clone(&self.queue);
        let history = Arc::clone(&self.history);
        let mut rx = self.rx.take().expect("Event receiver already taken");

        // Event ingestion task
        let queue_clone = Arc::clone(&queue);
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                debug!("Received event: {:?} - {}", event.domain, event.cause);
                queue_clone.enqueue(event);
            }
        });

        // Event processing loop
        loop {
            // Sleep and check for ready events
            tokio::time::sleep(Duration::from_millis(100)).await;

            let ready = queue.drain_ready();
            if ready.is_empty() {
                continue;
            }

            for (domain, events) in ready {
                info!("Processing {} events for domain {:?}", events.len(), domain);

                // Merge causes
                let causes: Vec<String> = events.iter().map(|e| e.cause.clone()).collect();
                let cause_str = causes.join(", ");

                // Create composite event
                let composite = SystemEvent {
                    domain: domain.clone(),
                    cause: cause_str,
                    timestamp: chrono::Utc::now().timestamp(),
                    metadata: HashMap::new(),
                };

                // Run doctor
                let start = Instant::now();
                match doctor_handler.handle_event(&composite).await {
                    Ok(result) => {
                        let duration_ms = start.elapsed().as_millis() as u64;

                        info!(
                            "Domain {:?} processed in {}ms: {} alerts, action: {}",
                            domain,
                            duration_ms,
                            result.doctor_result.alerts_found,
                            result.doctor_result.action_taken
                        );

                        // Store in history
                        let mut hist = history.lock().unwrap();
                        hist.push_back(result);
                        if hist.len() > self.max_history {
                            hist.pop_front();
                        }
                    }
                    Err(e) => {
                        warn!("Failed to handle event for {:?}: {}", domain, e);
                    }
                }
            }
        }
    }

    /// Get recent event history
    pub fn get_history(&self, limit: usize) -> Vec<EventResult> {
        let hist = self.history.lock().unwrap();
        hist.iter().rev().take(limit).cloned().collect()
    }

    /// Get pending event count
    pub fn pending_count(&self) -> usize {
        self.queue.pending_count()
    }
}

/// Trait for handling doctor/repair logic
#[async_trait::async_trait]
pub trait DoctorHandler {
    async fn handle_event(&self, event: &SystemEvent) -> Result<EventResult>;
}

/// Create a system event
pub fn create_event(domain: EventDomain, cause: impl Into<String>) -> SystemEvent {
    SystemEvent {
        domain,
        cause: cause.into(),
        timestamp: chrono::Utc::now().timestamp(),
        metadata: HashMap::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_queue_coalescing() {
        let queue = EventQueue::new(100, 30);

        // Enqueue multiple events for same domain
        let event1 = create_event(EventDomain::Packages, "installed foo");
        let event2 = create_event(EventDomain::Packages, "installed bar");

        queue.enqueue(event1);
        queue.enqueue(event2);

        // Should be pending
        assert_eq!(queue.pending_count(), 1);

        // Wait for debounce
        std::thread::sleep(Duration::from_millis(150));

        // Drain should coalesce both events
        let ready = queue.drain_ready();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].1.len(), 2);

        // Should be in cooldown now
        assert_eq!(queue.pending_count(), 0);
    }

    #[test]
    fn test_cooldown_enforcement() {
        let queue = EventQueue::new(100, 1); // 1 second cooldown

        let event1 = create_event(EventDomain::Packages, "event 1");
        queue.enqueue(event1);

        std::thread::sleep(Duration::from_millis(150));
        let ready1 = queue.drain_ready();
        assert_eq!(ready1.len(), 1);

        // Immediate second event should be dropped (cooldown)
        let event2 = create_event(EventDomain::Packages, "event 2");
        queue.enqueue(event2);
        assert_eq!(queue.pending_count(), 0); // Dropped

        // After cooldown, should accept
        std::thread::sleep(Duration::from_secs(1));
        let event3 = create_event(EventDomain::Packages, "event 3");
        queue.enqueue(event3);
        assert_eq!(queue.pending_count(), 1); // Accepted
    }
}
