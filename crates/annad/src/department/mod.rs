//! IT Department - Anna's team of specialists.
//! v0.0.999: Initial implementation
//!
//! Anna has a full IT department working inside the user's computer.
//! The user can watch the internal dialogue like a fly on the wall.

pub mod specialists;
pub mod tickets;
pub mod rpg;

pub use specialists::{Specialist, SpecialistRole, Department, get_department, get_specialist_for_topic, determine_department};
pub use tickets::{Ticket, TicketStatus, TicketStore, create_ticket, get_ticket, update_ticket, get_ticket_store};
pub use rpg::{AnnaXP, get_anna_xp, award_xp, get_title_for_level};
