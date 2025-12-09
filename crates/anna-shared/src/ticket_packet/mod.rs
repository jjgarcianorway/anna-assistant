//! Ticket packet for domain-relevant evidence collection (v0.0.216).
//!
//! A TicketPacket bundles all evidence relevant to a specific domain query,
//! providing structured access to probe results, parsed data, and context.
//!
//! v0.0.216: Modularized into domain-focused submodules.

mod builder;
mod domain;
mod packet;
mod policy;
pub mod types;

// Re-export for backwards compatibility
pub use builder::TicketPacketBuilder;
pub use domain::{evidence_kinds_for_domain, recommended_probes_for_domain};
pub use packet::TicketPacket;
pub use policy::{policy_for_team, PacketPolicy};
pub use types::{PacketBudget, MAX_PACKET_BYTES};
