//! Per-staff performance metrics
//!
//! Tracks individual staff member statistics including tickets handled,
//! success rates, XP, and levels.

use serde::{Deserialize, Serialize};

use super::levels::xp_to_level;

/// Per-staff performance metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StaffMetrics {
    /// Total tickets handled
    pub tickets_handled: u32,
    /// Successfully resolved tickets
    pub tickets_resolved: u32,
    /// Escalated tickets (for juniors)
    pub tickets_escalated: u32,
    /// Average reliability score
    pub avg_reliability: f32,
    /// Total handling time (ms)
    pub total_time_ms: u64,
    /// Last activity timestamp
    pub last_active: i64,
    /// v0.0.301: Staff XP (experience points)
    #[serde(default)]
    pub xp: u64,
    /// v0.0.301: Staff level (1-6)
    #[serde(default = "default_level")]
    pub level: u8,
}

fn default_level() -> u8 {
    1
}

impl StaffMetrics {
    /// Record a ticket completion
    pub fn record_ticket(
        &mut self,
        resolved: bool,
        escalated: bool,
        reliability: u8,
        duration_ms: u64,
    ) {
        let old_total = self.tickets_handled;
        self.tickets_handled += 1;

        if resolved {
            self.tickets_resolved += 1;
        }
        if escalated {
            self.tickets_escalated += 1;
        }

        // Update rolling average
        if old_total == 0 {
            self.avg_reliability = reliability as f32;
        } else {
            self.avg_reliability = (self.avg_reliability * old_total as f32 + reliability as f32)
                / self.tickets_handled as f32;
        }

        self.total_time_ms += duration_ms;
        self.last_active = chrono::Utc::now().timestamp();

        // v0.0.301: Compute XP and level
        self.update_xp(resolved, reliability);
    }

    /// v0.0.315: Update XP based on ticket outcome (with penalties)
    fn update_xp(&mut self, resolved: bool, reliability: u8) {
        // v0.0.315: XP can increase OR decrease based on performance
        let xp_change: i64;

        if resolved {
            // Base XP for resolved: +10
            let mut bonus: i64 = 10;

            // Bonus for high reliability: +2 per point above 60
            if reliability > 60 {
                bonus += (reliability - 60) as i64 * 2;
            }

            // Extra bonus for excellent work (90+): +15
            if reliability >= 90 {
                bonus += 15;
            }

            xp_change = bonus;
        } else {
            // PENALTY for unresolved tickets
            // Low reliability = bigger penalty (staff should improve)
            if reliability < 40 {
                xp_change = -15; // Significant penalty for poor work
            } else if reliability < 60 {
                xp_change = -5; // Minor penalty
            } else {
                xp_change = 0; // No penalty if reliability was ok but still unresolved
            }
        }

        // Apply XP change (floor at 0)
        if xp_change >= 0 {
            self.xp += xp_change as u64;
        } else {
            let penalty = (-xp_change) as u64;
            self.xp = self.xp.saturating_sub(penalty);
        }

        self.level = xp_to_level(self.xp);
    }

    /// Get success rate as percentage
    pub fn success_rate(&self) -> f32 {
        if self.tickets_handled == 0 {
            0.0
        } else {
            (self.tickets_resolved as f32 / self.tickets_handled as f32) * 100.0
        }
    }

    /// Get average handling time in ms
    pub fn avg_time_ms(&self) -> u64 {
        if self.tickets_handled == 0 {
            0
        } else {
            self.total_time_ms / self.tickets_handled as u64
        }
    }
}
