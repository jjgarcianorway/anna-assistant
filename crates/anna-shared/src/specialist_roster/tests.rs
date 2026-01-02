//! Tests for specialist roster

#[cfg(test)]
mod tests {
    use crate::specialist_roster::{
        formatting::{format_specialist_roster, is_specialist_roster_query, roster_fun_fact},
        names::get_specialist_name,
        management::SpecialistRoster,
        types::{Department, SpecialistLevel, SpecialistProfile},
    };

    fn make_specialist(name: &str, dept: Department, level: SpecialistLevel) -> SpecialistProfile {
        SpecialistProfile {
            id: format!("SPEC-{}", name),
            name: name.to_string(),
            department: dept,
            level,
            tickets_resolved: 0,
            available: true,
            model: Some("llama3".to_string()),
            skills: vec!["Linux".to_string()],
            joined_at: 1234567890,
        }
    }

    #[test]
    fn test_specialist_level() {
        assert_eq!(SpecialistLevel::Junior.name(), "Junior");
        assert_eq!(SpecialistLevel::Senior.symbol(), "S");
    }

    #[test]
    fn test_department() {
        assert_eq!(Department::Desktop.name(), "Desktop");
        assert_eq!(Department::Network.name(), "Network");
    }

    #[test]
    fn test_add_specialist() {
        let mut roster = SpecialistRoster::new();
        roster.add(make_specialist("Maya", Department::Desktop, SpecialistLevel::Junior));

        assert_eq!(roster.total_count(), 1);
        assert!(roster.get("SPEC-Maya").is_some());
    }

    #[test]
    fn test_get_by_name() {
        let mut roster = SpecialistRoster::new();
        roster.add(make_specialist("Maya", Department::Desktop, SpecialistLevel::Junior));

        assert!(roster.get_by_name("Maya").is_some());
        assert!(roster.get_by_name("Unknown").is_none());
    }

    #[test]
    fn test_record_resolution() {
        let mut roster = SpecialistRoster::new();
        roster.add(make_specialist("Maya", Department::Desktop, SpecialistLevel::Junior));

        assert!(roster.record_resolution("SPEC-Maya"));
        assert_eq!(roster.get("SPEC-Maya").unwrap().tickets_resolved, 1);
        assert_eq!(roster.total_tickets, 1);
    }

    #[test]
    fn test_set_available() {
        let mut roster = SpecialistRoster::new();
        roster.add(make_specialist("Maya", Department::Desktop, SpecialistLevel::Junior));

        assert!(roster.set_available("SPEC-Maya", false));
        assert!(!roster.get("SPEC-Maya").unwrap().available);
        assert_eq!(roster.available_count(), 0);
    }

    #[test]
    fn test_by_department() {
        let mut roster = SpecialistRoster::new();
        roster.add(make_specialist("Maya", Department::Desktop, SpecialistLevel::Junior));
        roster.add(make_specialist("Kenji", Department::Network, SpecialistLevel::Junior));

        assert_eq!(roster.by_dept(Department::Desktop).len(), 1);
        assert_eq!(roster.by_dept(Department::Network).len(), 1);
    }

    #[test]
    fn test_juniors_seniors() {
        let mut roster = SpecialistRoster::new();
        roster.add(make_specialist("Maya", Department::Desktop, SpecialistLevel::Junior));
        roster.add(make_specialist("Kenji", Department::Network, SpecialistLevel::Senior));

        assert_eq!(roster.juniors().len(), 1);
        assert_eq!(roster.seniors().len(), 1);
    }

    #[test]
    fn test_top_performer() {
        let mut roster = SpecialistRoster::new();
        roster.add(make_specialist("Maya", Department::Desktop, SpecialistLevel::Junior));
        roster.add(make_specialist("Kenji", Department::Network, SpecialistLevel::Senior));

        roster.record_resolution("SPEC-Maya");
        roster.record_resolution("SPEC-Maya");
        roster.record_resolution("SPEC-Kenji");

        let top = roster.top_performer().unwrap();
        assert_eq!(top.name, "Maya");
    }

    #[test]
    fn test_format_roster() {
        let mut roster = SpecialistRoster::new();
        roster.add(make_specialist("Maya", Department::Desktop, SpecialistLevel::Junior));

        let output = format_specialist_roster(&roster);
        assert!(output.contains("Specialist Roster"));
        assert!(output.contains("Maya"));
    }

    #[test]
    fn test_is_specialist_roster_query() {
        assert!(is_specialist_roster_query("show team roster"));
        assert!(is_specialist_roster_query("who is available?"));
        assert!(is_specialist_roster_query("list specialists"));
        assert!(!is_specialist_roster_query("what is the weather?"));
    }

    #[test]
    fn test_roster_fun_fact() {
        let mut roster = SpecialistRoster::new();
        roster.add(make_specialist("Maya", Department::Desktop, SpecialistLevel::Junior));

        let fact = roster_fun_fact(&roster);
        assert!(!fact.is_empty());
    }

    #[test]
    fn test_get_specialist_name() {
        let name = get_specialist_name(Department::Desktop, SpecialistLevel::Junior);
        assert_eq!(name, "Maya");
    }
}
