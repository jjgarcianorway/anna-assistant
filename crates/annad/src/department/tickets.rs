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
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TicketStatus {
    /// Just created, being analyzed
    New,
    /// Assigned to a specialist
    Assigned,
    /// Being worked on
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
            TicketStatus::InProgress => write!(f, "In Progress"),
            TicketStatus::WaitingUser => write!(f, "Waiting for User"),
            TicketStatus::Escalated => write!(f, "Escalated"),
            TicketStatus::Researching => write!(f, "Researching"),
            TicketStatus::Resolved => write!(f, "Resolved"),
            TicketStatus::Failed => write!(f, "Failed"),
        }
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
