//! Ticket store and processing operations.

use super::{Ticket, TicketStatsByState, TicketStatus, TICKET_SEQUENCE};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;

/// Ticket store persistence
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TicketStore {
    pub tickets: Vec<Ticket>,
    pub total_resolved: u64,
    pub total_failed: u64,
    pub total_escalated: u64,
}

impl TicketStore {
    /// Store path (system-wide)
    fn store_path() -> PathBuf {
        anna_shared::paths::paths().tickets_file()
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
            .filter(|t| !t.status.is_terminal())
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
        if times.is_empty() { return None; }
        Some(times.iter().sum::<i64>() as f64 / times.len() as f64)
    }

    /// Get stats per specialist
    pub fn specialist_stats(&self) -> std::collections::HashMap<String, (u64, u64)> {
        let mut stats = std::collections::HashMap::new();
        for ticket in &self.tickets {
            if let Some(ref assigned) = ticket.assigned_to {
                let entry = stats.entry(assigned.clone()).or_insert((0, 0));
                entry.0 += 1;
                if ticket.status == TicketStatus::Resolved {
                    entry.1 += 1;
                }
            }
        }
        stats
    }

    /// v0.3.29: Get shortest resolution time in seconds
    pub fn min_resolution_time(&self) -> Option<i64> {
        self.tickets.iter().filter_map(|t| t.resolution_time_secs()).min()
    }

    /// v0.3.29: Get longest resolution time in seconds
    pub fn max_resolution_time(&self) -> Option<i64> {
        self.tickets.iter().filter_map(|t| t.resolution_time_secs()).max()
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
        self.tickets.iter().rev().find(|t| !t.status.is_terminal())
    }
}

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
        if let Some(seq_str) = last.case_number.split('-').nth(1) {
            if let Ok(seq) = seq_str.parse::<u32>() {
                TICKET_SEQUENCE.store(seq + 1, std::sync::atomic::Ordering::SeqCst);
            }
        }
    }
}

/// v0.3.23: Reset in-memory ticket store to match cleared files
pub fn reset_ticket_store() {
    TICKET_SEQUENCE.store(1, std::sync::atomic::Ordering::SeqCst);
    let mut guard = STORE.write().unwrap();
    *guard = Some(TicketStore::default());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolution_time_stats() {
        let store = TicketStore::default();
        assert!(store.min_resolution_time().is_none());
        assert!(store.max_resolution_time().is_none());
        assert!(store.avg_resolution_time().is_none());
    }
}
