//! Service Desk Theatre integration (v0.0.290).
//!
//! Handles ticket creation, staff assignment, and case numbers for requests.
//! Named IT staff create the feeling of "an IT department inside the computer."
//!
//! v0.0.107: Added topic tracking for user profile personalization.
//! v0.0.290: Added email notifications for long-running tickets.

use anna_shared::email::{send_notification, EmailNotification};
use anna_shared::roster::{person_for, PersonProfile, Tier};
use anna_shared::rpc::SpecialistDomain;
use anna_shared::staff_stats::StaffStats;
use anna_shared::teams::Team;
use anna_shared::ticket_tracker::{Ticket, TicketDomain, TicketTracker};
use anna_shared::user_profile::UserProfile;
use tracing::debug;

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
    /// v0.0.251: Now generates domain-prefixed case numbers (e.g., NET-0042)
    pub fn new(query: &str, domain: SpecialistDomain) -> Self {
        let tracker = TicketTracker::new();
        let team = domain_to_team(domain);
        let ticket_domain = team_to_ticket_domain(team);
        let case_number = tracker.next_case_number_for_domain(ticket_domain);
        let staff = person_for(team, Tier::Junior);

        let ticket = Ticket::new(case_number.clone(), query.to_string(), team.to_string());

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
    /// v0.0.108: Also records tools mentioned in the query
    pub fn record_topic_to_profile(&self) {
        let topic = self.team.to_string().to_lowercase();
        let mut profile = UserProfile::load();
        profile.record_topic(&topic);

        // v0.0.108: Extract tools from query
        profile.record_tools_from_query(&self.ticket.query);

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

    /// v0.0.290: Send email notification for ticket creation (long-running queries)
    /// Called when a query takes too long or requires follow-up
    pub fn notify_ticket_created(&self) {
        if let Err(e) = send_notification(EmailNotification::TicketCreated(&self.ticket)) {
            debug!("Email notification skipped: {}", e);
        }
    }

    /// v0.0.290: Send email notification for ticket resolution
    pub fn notify_ticket_resolved(&self) {
        if let Err(e) = send_notification(EmailNotification::TicketResolved(&self.ticket)) {
            debug!("Email notification skipped: {}", e);
        }
    }

    /// v0.0.290: Send email notification when clarification is needed
    pub fn notify_needs_clarification(&self) {
        if let Err(e) = send_notification(EmailNotification::NeedsClarification(&self.ticket)) {
            debug!("Email notification skipped: {}", e);
        }
    }

    /// v0.0.290: Check if request took long enough to warrant ticket notification
    /// Threshold is 10 seconds - fast queries don't need email notification
    pub fn should_notify(&self, duration_ms: u64) -> bool {
        const NOTIFICATION_THRESHOLD_MS: u64 = 10_000; // 10 seconds
        duration_ms >= NOTIFICATION_THRESHOLD_MS
    }
}

/// Map SpecialistDomain to Team
/// v0.0.405: Expanded for all domains
fn domain_to_team(domain: SpecialistDomain) -> Team {
    match domain {
        SpecialistDomain::System => Team::Performance,
        SpecialistDomain::Boot => Team::Services,
        SpecialistDomain::Services => Team::Services,
        SpecialistDomain::Network => Team::Network,
        SpecialistDomain::Storage => Team::Storage,
        SpecialistDomain::Packages => Team::Services,
        SpecialistDomain::Audio => Team::Hardware,
        SpecialistDomain::Display => Team::Hardware,
        SpecialistDomain::Desktop => Team::Desktop,
        SpecialistDomain::Security => Team::Security,
    }
}

/// v0.0.251: Map Team to TicketDomain for case number prefixes
fn team_to_ticket_domain(team: Team) -> TicketDomain {
    match team {
        Team::Desktop | Team::Hardware | Team::Performance => TicketDomain::Desktop,
        Team::Network => TicketDomain::Network,
        Team::Storage => TicketDomain::Storage,
        Team::Security => TicketDomain::Security,
        Team::Services | Team::Logs | Team::General => TicketDomain::Services,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anna_shared::ticket_tracker::TicketStatus;

    #[test]
    fn test_theatre_context_creation() {
        let ctx = TheatreContext::new("how much RAM?", SpecialistDomain::System);
        // v0.0.405: System → Performance team, but DSK ticket domain prefix
        assert!(ctx.case_number.starts_with("DSK-"));
        assert_eq!(ctx.team, Team::Performance);
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
        // v0.0.405: Updated mappings for all domains
        assert_eq!(domain_to_team(SpecialistDomain::System), Team::Performance);
        assert_eq!(domain_to_team(SpecialistDomain::Boot), Team::Services);
        assert_eq!(domain_to_team(SpecialistDomain::Services), Team::Services);
        assert_eq!(domain_to_team(SpecialistDomain::Network), Team::Network);
        assert_eq!(domain_to_team(SpecialistDomain::Storage), Team::Storage);
        assert_eq!(domain_to_team(SpecialistDomain::Packages), Team::Services);
        assert_eq!(domain_to_team(SpecialistDomain::Audio), Team::Hardware);
        assert_eq!(domain_to_team(SpecialistDomain::Display), Team::Hardware);
        assert_eq!(domain_to_team(SpecialistDomain::Desktop), Team::Desktop);
        assert_eq!(domain_to_team(SpecialistDomain::Security), Team::Security);
    }
}
