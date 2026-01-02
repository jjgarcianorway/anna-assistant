//! Core types for greeting insights (v0.0.335).

use crate::teams::Team;

/// A greeting insight that can be added to Anna's welcome
#[derive(Debug, Clone)]
pub struct GreetingInsight {
    /// The staff member delivering this insight
    pub staff_name: &'static str,
    /// Team the insight is from
    pub team: Team,
    /// The insight message
    pub message: String,
    /// How urgent is this (affects display order)
    pub priority: u8,
    /// Is this good news or concerning?
    pub positive: bool,
}
