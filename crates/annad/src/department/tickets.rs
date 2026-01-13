//! Ticket System - Case tracking for the IT department.
//! v0.0.999: Initial implementation
//!
//! Every user request becomes a ticket that flows through the department.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;

/// Ticket status
/// v0.3.29: Added Investigating and Experimenting for UX clarity
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TicketStatus {
    /// Just created, being analyzed
    New,
    /// Assigned to a specialist
    Assigned,
    /// v0.3.29: Actively investigating - running probes, gathering info
    Investigating,
    /// v0.3.29: Running experiments to test hypotheses
    Experimenting,
    /// Being worked on (generic in-progress)
    InProgress,
    /// Waiting for user response
    WaitingUser,
    /// Escalated to senior
    Escalated,
    /// Needs long research (will email)
    Researching,
    /// Successfully resolved
    Resolved,
    /// Could not be resolved
    Failed,
}

impl std::fmt::Display for TicketStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TicketStatus::New => write!(f, "New"),
            TicketStatus::Assigned => write!(f, "Assigned"),
            TicketStatus::Investigating => write!(f, "Investigating"),
            TicketStatus::Experimenting => write!(f, "Experimenting"),
            TicketStatus::InProgress => write!(f, "In Progress"),
            TicketStatus::WaitingUser => write!(f, "Waiting for User"),
            TicketStatus::Escalated => write!(f, "Escalated"),
            TicketStatus::Researching => write!(f, "Researching"),
            TicketStatus::Resolved => write!(f, "Resolved"),
            TicketStatus::Failed => write!(f, "Failed"),
        }
    }
}

impl TicketStatus {
    /// v0.3.29: Check if this is a terminal state (no more transitions)
    pub fn is_terminal(&self) -> bool {
        matches!(self, TicketStatus::Resolved | TicketStatus::Failed)
    }

    /// v0.3.29: Check if this is an active working state
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            TicketStatus::Investigating | TicketStatus::Experimenting | TicketStatus::InProgress
        )
    }
}

/// A conversation entry in the ticket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketMessage {
    pub from: String,
    pub to: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    /// If true, show to user (fly on wall)
    pub visible_to_user: bool,
}

/// A support ticket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticket {
    /// Case number format: CN-NNNN-DDMMYYYY
    pub case_number: String,
    /// Original user question
    pub question: String,
    /// Current status
    pub status: TicketStatus,
    /// Assigned department
    pub department: String,
    /// Assigned specialist ID
    pub assigned_to: Option<String>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Resolved timestamp (if resolved)
    pub resolved_at: Option<DateTime<Utc>>,
    /// Internal conversation log
    pub messages: Vec<TicketMessage>,
    /// Final answer given to user
    pub resolution: Option<String>,
    /// Commands executed
    pub commands_executed: Vec<String>,
    /// Was escalated to senior?
    pub was_escalated: bool,
    /// Was Anna able to use a recipe?
    pub used_recipe: bool,
    /// XP awarded for this ticket
    pub xp_awarded: u32,
}

impl Ticket {
    /// Generate a new case number
    fn generate_case_number() -> String {
        let now = Utc::now();
        let date = now.format("%d%m%Y");
        let seq = TICKET_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        format!("CN-{:04}-{}", seq, date)
    }

    /// Create a new ticket
    pub fn new(question: &str, department: &str) -> Self {
        let now = Utc::now();
        Self {
            case_number: Self::generate_case_number(),
            question: question.to_string(),
            status: TicketStatus::New,
            department: department.to_string(),
            assigned_to: None,
            created_at: now,
            updated_at: now,
            resolved_at: None,
            messages: vec![],
            resolution: None,
            commands_executed: vec![],
            was_escalated: false,
            used_recipe: false,
            xp_awarded: 0,
        }
    }

    /// Add a message to the conversation
    pub fn add_message(&mut self, from: &str, to: &str, message: &str, visible: bool) {
        self.messages.push(TicketMessage {
            from: from.to_string(),
            to: to.to_string(),
            message: message.to_string(),
            timestamp: Utc::now(),
            visible_to_user: visible,
        });
        self.updated_at = Utc::now();
    }

    /// Assign to a specialist
    pub fn assign(&mut self, specialist_id: &str) {
        self.assigned_to = Some(specialist_id.to_string());
        self.status = TicketStatus::Assigned;
        self.updated_at = Utc::now();
    }

    /// Escalate to senior
    pub fn escalate(&mut self, senior_id: &str) {
        self.assigned_to = Some(senior_id.to_string());
        self.status = TicketStatus::Escalated;
        self.was_escalated = true;
        self.updated_at = Utc::now();
    }

    /// Mark as resolved
    pub fn resolve(&mut self, resolution: &str, xp: u32) {
        self.status = TicketStatus::Resolved;
        self.resolution = Some(resolution.to_string());
        self.resolved_at = Some(Utc::now());
        self.xp_awarded = xp;
        self.updated_at = Utc::now();
    }

    /// v0.3.29: Transition to investigating state
    pub fn start_investigating(&mut self) {
        self.status = TicketStatus::Investigating;
        self.updated_at = Utc::now();
    }

    /// v0.3.29: Transition to experimenting state
    pub fn start_experimenting(&mut self) {
        self.status = TicketStatus::Experimenting;
        self.updated_at = Utc::now();
    }

    /// v0.3.29: Mark as failed
    pub fn fail(&mut self, reason: &str) {
        self.status = TicketStatus::Failed;
        self.resolution = Some(reason.to_string());
        self.resolved_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// v0.3.29: Get elapsed time since creation in seconds
    pub fn elapsed_secs(&self) -> i64 {
        (Utc::now() - self.created_at).num_seconds()
    }

    /// Get resolution time in seconds
    pub fn resolution_time_secs(&self) -> Option<i64> {
        self.resolved_at.map(|r| (r - self.created_at).num_seconds())
    }

    /// Get visible messages for fly-on-wall display
    pub fn visible_messages(&self) -> Vec<&TicketMessage> {
        self.messages.iter().filter(|m| m.visible_to_user).collect()
    }
}

/// Ticket store persistence
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TicketStore {
    pub tickets: Vec<Ticket>,
    pub total_resolved: u64,
    pub total_failed: u64,
    pub total_escalated: u64,
}

impl TicketStore {
    fn store_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("anna/tickets.json")
    }

    pub fn load() -> Self {
        let path = Self::store_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(store) = serde_json::from_str(&content) {
                    return store;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::store_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(path, content).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn add_ticket(&mut self, ticket: Ticket) {
        self.tickets.push(ticket);
        // Keep last 500 tickets
        if self.tickets.len() > 500 {
            self.tickets.remove(0);
        }
    }

    pub fn get_ticket(&self, case_number: &str) -> Option<&Ticket> {
        self.tickets.iter().find(|t| t.case_number == case_number)
    }

    pub fn get_ticket_mut(&mut self, case_number: &str) -> Option<&mut Ticket> {
        self.tickets.iter_mut().find(|t| t.case_number == case_number)
    }

    pub fn get_active_tickets(&self) -> Vec<&Ticket> {
        self.tickets.iter()
            .filter(|t| t.status != TicketStatus::Resolved && t.status != TicketStatus::Failed)
            .collect()
    }

    pub fn get_recent_resolved(&self, count: usize) -> Vec<&Ticket> {
        self.tickets.iter()
            .filter(|t| t.status == TicketStatus::Resolved)
            .rev()
            .take(count)
            .collect()
    }

    /// Update stats when ticket is resolved
    pub fn record_resolution(&mut self, ticket: &Ticket) {
        match ticket.status {
            TicketStatus::Resolved => self.total_resolved += 1,
            TicketStatus::Failed => self.total_failed += 1,
            _ => {}
        }
        if ticket.was_escalated {
            self.total_escalated += 1;
        }
    }

    /// Get tickets resolved by a specialist
    pub fn get_by_specialist(&self, specialist_id: &str) -> Vec<&Ticket> {
        self.tickets.iter()
            .filter(|t| t.assigned_to.as_deref() == Some(specialist_id))
            .collect()
    }

    /// Get average resolution time in seconds
    pub fn avg_resolution_time(&self) -> Option<f64> {
        let times: Vec<_> = self.tickets.iter()
            .filter_map(|t| t.resolution_time_secs())
            .collect();
        if times.is_empty() {
            return None;
        }
        Some(times.iter().sum::<i64>() as f64 / times.len() as f64)
    }

    /// Get stats per specialist
    pub fn specialist_stats(&self) -> std::collections::HashMap<String, (u64, u64)> {
        let mut stats: std::collections::HashMap<String, (u64, u64)> = std::collections::HashMap::new();
        for ticket in &self.tickets {
            if let Some(ref assigned) = ticket.assigned_to {
                let entry = stats.entry(assigned.clone()).or_insert((0, 0));
                entry.0 += 1; // total
                if ticket.status == TicketStatus::Resolved {
                    entry.1 += 1; // resolved
                }
            }
        }
        stats
    }

    /// v0.3.29: Get shortest resolution time in seconds
    pub fn min_resolution_time(&self) -> Option<i64> {
        self.tickets
            .iter()
            .filter_map(|t| t.resolution_time_secs())
            .min()
    }

    /// v0.3.29: Get longest resolution time in seconds
    pub fn max_resolution_time(&self) -> Option<i64> {
        self.tickets
            .iter()
            .filter_map(|t| t.resolution_time_secs())
            .max()
    }

    /// v0.3.29: Get ticket counts by final state
    pub fn stats_by_state(&self) -> TicketStatsByState {
        let mut stats = TicketStatsByState::default();
        for ticket in &self.tickets {
            match ticket.status {
                TicketStatus::Resolved => stats.resolved += 1,
                TicketStatus::Failed => stats.failed += 1,
                TicketStatus::Escalated => stats.escalated += 1,
                _ => stats.other += 1,
            }
        }
        stats
    }

    /// v0.3.29: Get current active ticket (most recent non-terminal)
    pub fn get_current_active(&self) -> Option<&Ticket> {
        self.tickets
            .iter()
            .rev()
            .find(|t| !t.status.is_terminal())
    }
}

/// v0.3.29: Ticket statistics by final state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TicketStatsByState {
    pub resolved: u64,
    pub failed: u64,
    pub escalated: u64,
    pub other: u64,
}

// Global ticket sequence for case numbers
static TICKET_SEQUENCE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);

// Global ticket store (thread-safe)
static STORE: RwLock<Option<TicketStore>> = RwLock::new(None);

fn get_store() -> TicketStore {
    let guard = STORE.read().unwrap();
    guard.clone().unwrap_or_else(|| {
        drop(guard);
        let store = TicketStore::load();
        let mut guard = STORE.write().unwrap();
        *guard = Some(store.clone());
        store
    })
}

fn save_store(store: &TicketStore) {
    let mut guard = STORE.write().unwrap();
    *guard = Some(store.clone());
    let _ = store.save();
}

/// Create a new ticket
pub fn create_ticket(question: &str, department: &str) -> Ticket {
    let ticket = Ticket::new(question, department);
    let mut store = get_store();
    store.add_ticket(ticket.clone());
    save_store(&store);
    ticket
}

/// Get a ticket by case number
pub fn get_ticket(case_number: &str) -> Option<Ticket> {
    let store = get_store();
    store.get_ticket(case_number).cloned()
}

/// Update a ticket
pub fn update_ticket(ticket: &Ticket) {
    let mut store = get_store();
    if let Some(existing) = store.get_ticket_mut(&ticket.case_number) {
        *existing = ticket.clone();
        // Update stats if resolved
        if ticket.status == TicketStatus::Resolved || ticket.status == TicketStatus::Failed {
            store.record_resolution(ticket);
        }
    }
    save_store(&store);
}

/// Get ticket store for stats
pub fn get_ticket_store() -> TicketStore {
    get_store()
}

/// Initialize sequence from stored tickets
pub fn init_ticket_sequence() {
    let store = get_store();
    if let Some(last) = store.tickets.last() {
        // Parse sequence from case number
        if let Some(seq_str) = last.case_number.split('-').nth(1) {
            if let Ok(seq) = seq_str.parse::<u32>() {
                TICKET_SEQUENCE.store(seq + 1, std::sync::atomic::Ordering::SeqCst);
            }
        }
    }
}

/// v0.3.23: Reset in-memory ticket store to match cleared files
/// Called after SafeReset to ensure daemon memory is consistent with disk
pub fn reset_ticket_store() {
    // Reset the sequence counter
    TICKET_SEQUENCE.store(1, std::sync::atomic::Ordering::SeqCst);

    // Clear the in-memory store - it will reload from disk (which is now empty)
    let mut guard = STORE.write().unwrap();
    *guard = Some(TicketStore::default());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// v0.3.29: Test ticket lifecycle state transitions
    #[test]
    fn test_ticket_lifecycle_states() {
        let mut ticket = Ticket::new("test question", "System Administration");

        // Initial state
        assert_eq!(ticket.status, TicketStatus::New);
        assert!(!ticket.status.is_terminal());
        assert!(!ticket.status.is_active());

        // Assign
        ticket.assign("specialist-1");
        assert_eq!(ticket.status, TicketStatus::Assigned);
        assert!(!ticket.status.is_terminal());

        // Start investigating
        ticket.start_investigating();
        assert_eq!(ticket.status, TicketStatus::Investigating);
        assert!(ticket.status.is_active());
        assert!(!ticket.status.is_terminal());

        // Start experimenting
        ticket.start_experimenting();
        assert_eq!(ticket.status, TicketStatus::Experimenting);
        assert!(ticket.status.is_active());
        assert!(!ticket.status.is_terminal());

        // Resolve
        ticket.resolve("Solution found", 10);
        assert_eq!(ticket.status, TicketStatus::Resolved);
        assert!(ticket.status.is_terminal());
        assert!(!ticket.status.is_active());
    }

    /// v0.3.29: Test ticket fail state
    #[test]
    fn test_ticket_fail_state() {
        let mut ticket = Ticket::new("failing question", "Network Operations");

        ticket.start_investigating();
        assert!(!ticket.status.is_terminal());

        ticket.fail("Could not determine cause");
        assert_eq!(ticket.status, TicketStatus::Failed);
        assert!(ticket.status.is_terminal());
        assert!(!ticket.status.is_active());
    }

    /// v0.3.29: Test terminal vs active states
    #[test]
    fn test_terminal_and_active_states() {
        // Terminal states
        assert!(TicketStatus::Resolved.is_terminal());
        assert!(TicketStatus::Failed.is_terminal());

        // Non-terminal states
        assert!(!TicketStatus::New.is_terminal());
        assert!(!TicketStatus::Assigned.is_terminal());
        assert!(!TicketStatus::Investigating.is_terminal());
        assert!(!TicketStatus::Experimenting.is_terminal());
        assert!(!TicketStatus::InProgress.is_terminal());
        assert!(!TicketStatus::WaitingUser.is_terminal());
        assert!(!TicketStatus::Escalated.is_terminal());
        assert!(!TicketStatus::Researching.is_terminal());

        // Active states
        assert!(TicketStatus::Investigating.is_active());
        assert!(TicketStatus::Experimenting.is_active());
        assert!(TicketStatus::InProgress.is_active());

        // Non-active states
        assert!(!TicketStatus::New.is_active());
        assert!(!TicketStatus::Assigned.is_active());
        assert!(!TicketStatus::Resolved.is_active());
        assert!(!TicketStatus::Failed.is_active());
    }

    /// v0.3.29: Test elapsed time calculation
    #[test]
    fn test_ticket_elapsed_time() {
        let ticket = Ticket::new("time test", "System Administration");

        // Elapsed time should be very small (just created)
        let elapsed = ticket.elapsed_secs();
        assert!(elapsed >= 0);
        assert!(elapsed < 5); // Should be less than 5 seconds
    }

    /// v0.3.29: Test ticket status display format
    #[test]
    fn test_ticket_status_display() {
        assert_eq!(format!("{}", TicketStatus::New), "New");
        assert_eq!(format!("{}", TicketStatus::Investigating), "Investigating");
        assert_eq!(format!("{}", TicketStatus::Experimenting), "Experimenting");
        assert_eq!(format!("{}", TicketStatus::InProgress), "In Progress");
        assert_eq!(format!("{}", TicketStatus::Resolved), "Resolved");
        assert_eq!(format!("{}", TicketStatus::Failed), "Failed");
    }

    /// v0.3.29: Test resolution time stats
    #[test]
    fn test_resolution_time_stats() {
        let mut store = TicketStore::default();

        // Empty store should have no resolution times
        assert!(store.min_resolution_time().is_none());
        assert!(store.max_resolution_time().is_none());
        assert!(store.avg_resolution_time().is_none());
    }
}
