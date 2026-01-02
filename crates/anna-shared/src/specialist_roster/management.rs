//! Specialist roster management
//!
//! Provides SpecialistRoster for managing specialists and their operations.

use super::types::{Department, SpecialistLevel, SpecialistProfile};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Specialist roster
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpecialistRoster {
    /// All specialists
    pub specialists: Vec<SpecialistProfile>,
    /// Count by department
    pub by_department: HashMap<String, u64>,
    /// Count by level
    pub by_level: HashMap<String, u64>,
    /// Total tickets resolved
    pub total_tickets: u64,
}

impl SpecialistRoster {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a specialist
    pub fn add(&mut self, specialist: SpecialistProfile) {
        *self.by_department.entry(specialist.department.name().to_string()).or_insert(0) += 1;
        *self.by_level.entry(specialist.level.name().to_string()).or_insert(0) += 1;
        self.specialists.push(specialist);
    }

    /// Get specialist by ID
    pub fn get(&self, id: &str) -> Option<&SpecialistProfile> {
        self.specialists.iter().find(|s| s.id == id)
    }

    /// Get specialist by name
    pub fn get_by_name(&self, name: &str) -> Option<&SpecialistProfile> {
        self.specialists.iter().find(|s| s.name == name)
    }

    /// Record ticket resolution
    pub fn record_resolution(&mut self, id: &str) -> bool {
        let found = self.specialists.iter().position(|s| s.id == id);
        if let Some(idx) = found {
            self.specialists[idx].tickets_resolved += 1;
            self.total_tickets += 1;
            true
        } else {
            false
        }
    }

    /// Set availability
    pub fn set_available(&mut self, id: &str, available: bool) -> bool {
        let found = self.specialists.iter().position(|s| s.id == id);
        if let Some(idx) = found {
            self.specialists[idx].available = available;
            true
        } else {
            false
        }
    }

    /// Get available specialists
    pub fn available(&self) -> Vec<&SpecialistProfile> {
        self.specialists.iter().filter(|s| s.available).collect()
    }

    /// Get specialists by department
    pub fn by_dept(&self, dept: Department) -> Vec<&SpecialistProfile> {
        self.specialists.iter().filter(|s| s.department == dept).collect()
    }

    /// Get specialists by level
    pub fn by_lvl(&self, level: SpecialistLevel) -> Vec<&SpecialistProfile> {
        self.specialists.iter().filter(|s| s.level == level).collect()
    }

    /// Get juniors
    pub fn juniors(&self) -> Vec<&SpecialistProfile> {
        self.by_lvl(SpecialistLevel::Junior)
    }

    /// Get seniors
    pub fn seniors(&self) -> Vec<&SpecialistProfile> {
        self.by_lvl(SpecialistLevel::Senior)
    }

    /// Total specialist count
    pub fn total_count(&self) -> usize {
        self.specialists.len()
    }

    /// Available count
    pub fn available_count(&self) -> usize {
        self.specialists.iter().filter(|s| s.available).count()
    }

    /// Top performer
    pub fn top_performer(&self) -> Option<&SpecialistProfile> {
        self.specialists.iter().max_by_key(|s| s.tickets_resolved)
    }

    /// Most active department
    pub fn most_active_department(&self) -> Option<(&str, u64)> {
        self.by_department
            .iter()
            .max_by_key(|(_, v)| *v)
            .map(|(k, v)| (k.as_str(), *v))
    }
}
