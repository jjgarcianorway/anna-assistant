// v0.0.528: Team Specialist Roster - Roster Management
// Manages the full IT department roster

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::specialist::Specialist;
use super::types::{AvailabilityStatus, Department, SeniorityLevel};

/// Team specialist roster
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeamSpecialistRoster {
    specialists: HashMap<String, Specialist>,
}

impl TeamSpecialistRoster {
    /// Create a new roster
    pub fn new() -> Self {
        Self {
            specialists: HashMap::new(),
        }
    }

    /// Add specialist to roster
    pub fn add(&mut self, specialist: Specialist) {
        self.specialists.insert(specialist.id.clone(), specialist);
    }

    /// Get specialist by ID
    pub fn get(&self, id: &str) -> Option<&Specialist> {
        self.specialists.get(id)
    }

    /// Get mutable specialist
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Specialist> {
        self.specialists.get_mut(id)
    }

    /// Get all specialists in a department
    pub fn by_department(&self, dept: &Department) -> Vec<&Specialist> {
        self.specialists
            .values()
            .filter(|s| &s.department == dept)
            .collect()
    }

    /// Get available specialists
    pub fn available(&self) -> Vec<&Specialist> {
        self.specialists
            .values()
            .filter(|s| s.status == AvailabilityStatus::Available)
            .collect()
    }

    /// Get available specialist for department (prefer junior)
    pub fn find_available(&self, dept: &Department) -> Option<&Specialist> {
        // First try junior
        let junior = self
            .specialists
            .values()
            .find(|s| {
                &s.department == dept
                    && s.seniority == SeniorityLevel::Junior
                    && s.status == AvailabilityStatus::Available
            });

        if junior.is_some() {
            return junior;
        }

        // Fall back to senior
        self.specialists.values().find(|s| {
            &s.department == dept
                && s.seniority == SeniorityLevel::Senior
                && s.status == AvailabilityStatus::Available
        })
    }

    /// Get senior specialist for escalation
    pub fn find_senior(&self, dept: &Department) -> Option<&Specialist> {
        self.specialists.values().find(|s| {
            &s.department == dept
                && s.seniority == SeniorityLevel::Senior
                && s.status == AvailabilityStatus::Available
        })
    }

    /// Get top performers by tickets closed
    pub fn top_performers(&self, n: usize) -> Vec<&Specialist> {
        let mut list: Vec<_> = self.specialists.values().collect();
        list.sort_by(|a, b| b.tickets_closed.cmp(&a.tickets_closed));
        list.into_iter().take(n).collect()
    }

    /// Get department stats
    pub fn department_stats(&self) -> HashMap<Department, (u32, u32)> {
        let mut stats = HashMap::new();
        for s in self.specialists.values() {
            let entry = stats.entry(s.department.clone()).or_insert((0, 0));
            entry.0 += 1; // count
            entry.1 += s.tickets_closed; // total tickets
        }
        stats
    }

    /// Total specialists
    pub fn total_count(&self) -> usize {
        self.specialists.len()
    }

    /// Total tickets closed
    pub fn total_tickets(&self) -> u32 {
        self.specialists.values().map(|s| s.tickets_closed).sum()
    }

    /// List all specialists
    pub fn all(&self) -> Vec<&Specialist> {
        self.specialists.values().collect()
    }
}
