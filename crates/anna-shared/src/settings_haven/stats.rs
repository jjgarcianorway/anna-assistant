// v0.0.784: Settings Haven (Phase 360)
// Safe haven for settings protection - Stats module

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::HavenType;
use super::guest::HavenGuest;

/// Haven stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HavenStats {
    /// Total guests
    pub total_guests: usize,
    /// Comfortable guests
    pub comfortable: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl HavenStats {
    /// Update from guests
    pub fn update(&mut self, guests: &[HavenGuest], haven_type: HavenType) {
        self.total_guests = guests.len();
        self.comfortable = guests.iter().filter(|g| g.comfortable).count();
        *self.by_type.entry(haven_type.to_string()).or_insert(0) += 1;
    }

    /// Comfort rate
    pub fn comfort_rate(&self) -> f64 {
        if self.total_guests == 0 { 0.0 } else { self.comfortable as f64 / self.total_guests as f64 * 100.0 }
    }
}
