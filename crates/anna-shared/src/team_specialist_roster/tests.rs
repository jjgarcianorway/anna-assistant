// v0.0.528: Team Specialist Roster - Tests
// Test suite for the specialist roster system

#[cfg(test)]
mod tests {
    use super::super::formatting::{is_roster_query, roster_fun_fact};
    use super::super::roster::TeamSpecialistRoster;
    use super::super::specialist::Specialist;
    use super::super::types::{AvailabilityStatus, Department, SeniorityLevel};

    #[test]
    fn test_specialist_creation() {
        let spec = Specialist::new(
            "desktop-jr-1",
            "Sofia Chen",
            Department::Desktop,
            SeniorityLevel::Junior,
            "qwen2.5:3b",
        );
        assert_eq!(spec.name, "Sofia Chen");
        assert_eq!(spec.seniority, SeniorityLevel::Junior);
        assert_eq!(spec.status, AvailabilityStatus::Available);
    }

    #[test]
    fn test_ticket_assignment() {
        let mut spec = Specialist::new(
            "net-jr-1",
            "Marcus Rivera",
            Department::Network,
            SeniorityLevel::Junior,
            "qwen2.5:3b",
        );
        spec.assign_ticket("CN-001");
        assert_eq!(spec.status, AvailabilityStatus::OnTicket);
        assert_eq!(spec.current_ticket, Some("CN-001".to_string()));
    }

    #[test]
    fn test_ticket_completion() {
        let mut spec = Specialist::new(
            "sec-jr-1",
            "Aisha Patel",
            Department::Security,
            SeniorityLevel::Junior,
            "qwen2.5:3b",
        );
        spec.assign_ticket("CN-002");
        spec.complete_ticket(true, 5000);
        assert_eq!(spec.status, AvailabilityStatus::Available);
        assert_eq!(spec.tickets_closed, 1);
        assert_eq!(spec.avg_resolution_ms, 5000);
    }

    #[test]
    fn test_roster_add_and_get() {
        let mut roster = TeamSpecialistRoster::new();
        let spec = Specialist::new(
            "sys-sr-1",
            "David Kim",
            Department::System,
            SeniorityLevel::Senior,
            "qwen2.5:14b",
        );
        roster.add(spec);
        assert_eq!(roster.total_count(), 1);
        assert!(roster.get("sys-sr-1").is_some());
    }

    #[test]
    fn test_by_department() {
        let mut roster = TeamSpecialistRoster::new();
        roster.add(Specialist::new(
            "net-1",
            "A",
            Department::Network,
            SeniorityLevel::Junior,
            "m",
        ));
        roster.add(Specialist::new(
            "net-2",
            "B",
            Department::Network,
            SeniorityLevel::Senior,
            "m",
        ));
        roster.add(Specialist::new(
            "sys-1",
            "C",
            Department::System,
            SeniorityLevel::Junior,
            "m",
        ));
        assert_eq!(roster.by_department(&Department::Network).len(), 2);
    }

    #[test]
    fn test_find_available_prefers_junior() {
        let mut roster = TeamSpecialistRoster::new();
        roster.add(Specialist::new(
            "desk-sr",
            "Senior",
            Department::Desktop,
            SeniorityLevel::Senior,
            "m",
        ));
        roster.add(Specialist::new(
            "desk-jr",
            "Junior",
            Department::Desktop,
            SeniorityLevel::Junior,
            "m",
        ));
        let found = roster.find_available(&Department::Desktop).unwrap();
        assert_eq!(found.seniority, SeniorityLevel::Junior);
    }

    #[test]
    fn test_find_senior() {
        let mut roster = TeamSpecialistRoster::new();
        roster.add(Specialist::new(
            "audio-sr",
            "Senior Audio",
            Department::Audio,
            SeniorityLevel::Senior,
            "m",
        ));
        let senior = roster.find_senior(&Department::Audio);
        assert!(senior.is_some());
        assert_eq!(senior.unwrap().seniority, SeniorityLevel::Senior);
    }

    #[test]
    fn test_top_performers() {
        let mut roster = TeamSpecialistRoster::new();
        let mut spec1 = Specialist::new("a", "A", Department::System, SeniorityLevel::Junior, "m");
        spec1.tickets_closed = 50;
        let mut spec2 = Specialist::new("b", "B", Department::System, SeniorityLevel::Senior, "m");
        spec2.tickets_closed = 100;
        roster.add(spec1);
        roster.add(spec2);
        let top = roster.top_performers(1);
        assert_eq!(top[0].name, "B");
    }

    #[test]
    fn test_can_escalate() {
        let junior = Specialist::new("j", "J", Department::System, SeniorityLevel::Junior, "m");
        let senior = Specialist::new("s", "S", Department::System, SeniorityLevel::Senior, "m");
        assert!(junior.can_escalate());
        assert!(!senior.can_escalate());
    }

    #[test]
    fn test_is_roster_query() {
        assert!(is_roster_query("Who is on the team?"));
        assert!(is_roster_query("Show available specialists"));
        assert!(is_roster_query("Which departments are there?"));
        assert!(!is_roster_query("Install vim"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = roster_fun_fact();
        assert!(fact.contains("junior") && fact.contains("senior"));
    }
}
