//! Service Desk Theatre integration for v0.0.106.
//!
//! Handles ticket creation, staff assignment, and case numbers for requests.
//! Named IT staff create the feeling of "an IT department inside the computer."
//!
//! v0.0.107: Added topic tracking for user profile personalization.

use anna_shared::roster::{person_for, PersonProfile, Tier};
use anna_shared::rpc::SpecialistDomain;
use anna_shared::staff_stats::StaffStats;
use anna_shared::teams::Team;
use anna_shared::ticket_tracker::{Ticket, TicketTracker};
use anna_shared::user_profile::UserProfile;

/// Theatre context for a single request
#[derive(Debug, Clone)]
pub struct TheatreContext {
    /// Case number for this request (e.g., "CN-0001-06122025")
    pub case_number: String,
    /// Assigned staff member
    pub staff: PersonProfile,
    /// Team handling this request
    pub team: Team,
    /// Internal ticket (for history tracking)
    pub ticket: Ticket,
}

impl TheatreContext {
    /// Create a new theatre context for a request
    pub fn new(query: &str, domain: SpecialistDomain) -> Self {
        let tracker = TicketTracker::new();
        let case_number = tracker.next_case_number();
        let team = domain_to_team(domain);
        let staff = person_for(team, Tier::Junior);

        let ticket = Ticket::new(
            case_number.clone(),
            query.to_string(),
            team.to_string(),
        );

        Self {
            case_number,
            staff,
            team,
            ticket,
        }
    }

    /// Get staff display name (e.g., "Sofia")
    pub fn staff_name(&self) -> &str {
        self.staff.display_name
    }

    /// Get staff full display (e.g., "Sofia (Desktop Administrator)")
    pub fn staff_display(&self) -> String {
        self.staff.display()
    }

    /// Assign to a specific staff member
    pub fn assign_to(&mut self, person_id: &str) {
        self.ticket.assign(person_id);
    }

    /// Start working on the ticket
    pub fn start_work(&mut self) {
        self.ticket.start_work();
    }

    /// Escalate to senior
    pub fn escalate(&mut self) {
        let senior = person_for(self.team, Tier::Senior);
        self.staff = senior.clone();
        self.ticket.escalate(senior.person_id);
    }

    /// Resolve the ticket
    pub fn resolve(&mut self, answer: String, reliability: u8, duration_ms: u64) {
        self.ticket.resolve(answer, reliability, duration_ms);
    }

    /// Save ticket to history
    pub fn save(&self) -> std::io::Result<()> {
        let tracker = TicketTracker::new();
        tracker.save_ticket(&self.ticket)
    }

    /// v0.0.107: Record topic to user profile for personalization
    pub fn record_topic_to_profile(&self) {
        let topic = self.team.to_string().to_lowercase();
        let mut profile = UserProfile::load();
        profile.record_topic(&topic);
        let _ = profile.save();
    }

    /// v0.0.107: Record staff performance metrics
    pub fn record_staff_stats(&self, reliability: u8, duration_ms: u64) {
        let resolved = self.ticket.status == anna_shared::ticket_tracker::TicketStatus::Resolved;
        let escalated = self.ticket.was_escalated;

        let mut stats = StaffStats::load();
        stats.record_ticket(
            self.staff.person_id,
            resolved,
            escalated,
            reliability,
            duration_ms,
        );
        let _ = stats.save();
    }
}

/// Map SpecialistDomain to Team
fn domain_to_team(domain: SpecialistDomain) -> Team {
    match domain {
        SpecialistDomain::System => Team::Desktop,
        SpecialistDomain::Network => Team::Network,
        SpecialistDomain::Storage => Team::Storage,
        SpecialistDomain::Security => Team::Security,
        SpecialistDomain::Packages => Team::Services,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anna_shared::ticket_tracker::TicketStatus;

    #[test]
    fn test_theatre_context_creation() {
        let ctx = TheatreContext::new("how much RAM?", SpecialistDomain::System);
        assert!(ctx.case_number.starts_with("CN-"));
        assert_eq!(ctx.team, Team::Desktop);
        assert!(!ctx.staff_name().is_empty());
    }

    #[test]
    fn test_theatre_context_escalation() {
        let mut ctx = TheatreContext::new("complex network issue", SpecialistDomain::Network);
        ctx.escalate();
        // Staff should change to senior
        assert!(ctx.ticket.was_escalated);
        assert_eq!(ctx.ticket.status, TicketStatus::Escalated);
    }

    #[test]
    fn test_domain_to_team_mapping() {
        assert_eq!(domain_to_team(SpecialistDomain::System), Team::Desktop);
        assert_eq!(domain_to_team(SpecialistDomain::Network), Team::Network);
        assert_eq!(domain_to_team(SpecialistDomain::Storage), Team::Storage);
        assert_eq!(domain_to_team(SpecialistDomain::Security), Team::Security);
        assert_eq!(domain_to_team(SpecialistDomain::Packages), Team::Services);
    }
}
