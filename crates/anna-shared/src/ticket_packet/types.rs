//! Ticket packet type definitions (v0.0.216).

use serde::{Deserialize, Serialize};

/// Maximum packet size in bytes (8KB) - v0.0.40
pub const MAX_PACKET_BYTES: usize = 8 * 1024;

/// Budget tracking for a packet
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PacketBudget {
    /// Number of probes planned
    pub probes_planned: usize,
    /// Number of probes executed
    pub probes_executed: usize,
    /// Number of probes that succeeded
    pub probes_succeeded: usize,
    /// Total bytes collected
    pub bytes_collected: usize,
    /// Whether budget was exceeded
    pub budget_exceeded: bool,
}
