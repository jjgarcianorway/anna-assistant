//! Tests for roster module (v0.0.182).

#[cfg(test)]
mod tests {
    use crate::roster::{all_persons, person_by_id, person_for, team_roster, Tier};
    use crate::teams::Team;

    #[test]
    fn test_person_for_deterministic() {
        let p1 = person_for(Team::Network, Tier::Junior);
        let p2 = person_for(Team::Network, Tier::Junior);
        assert_eq!(p1.person_id, p2.person_id);
        assert_eq!(p1.display_name, "Michael");
        assert_eq!(p1.role_title, "Network Engineer");
    }

    #[test]
    fn test_person_for_all_teams() {
        for team in [
            Team::Desktop,
            Team::Storage,
            Team::Network,
            Team::Performance,
            Team::Services,
            Team::Security,
            Team::Hardware,
            Team::Logs,
            Team::General,
        ] {
            let jr = person_for(team, Tier::Junior);
            let sr = person_for(team, Tier::Senior);
            assert_ne!(jr.person_id, sr.person_id);
            assert_ne!(jr.display_name, sr.display_name);
            assert_eq!(jr.team, team);
            assert_eq!(sr.team, team);
        }
    }

    #[test]
    fn test_person_display() {
        let p = person_for(Team::Storage, Tier::Senior);
        assert_eq!(p.display(), "Ines (Storage Architect)");
    }

    #[test]
    fn test_person_debug_tag() {
        let p = person_for(Team::Network, Tier::Junior);
        assert_eq!(p.debug_tag(), "michael/network");
    }

    #[test]
    fn test_person_by_id() {
        let p = person_by_id("security_sr").unwrap();
        assert_eq!(p.display_name, "Oskar");
        assert_eq!(p.team, Team::Security);
        assert_eq!(p.tier, Tier::Senior);

        assert!(person_by_id("nonexistent").is_none());
    }

    #[test]
    fn test_team_roster() {
        let roster = team_roster(Team::Desktop);
        assert_eq!(roster.len(), 2);
        assert!(roster.iter().any(|p| p.tier == Tier::Junior));
        assert!(roster.iter().any(|p| p.tier == Tier::Senior));
    }

    #[test]
    fn test_all_persons() {
        let all = all_persons();
        assert_eq!(all.len(), 18); // 9 teams * 2 tiers (v0.0.42: added Logs)
    }

    #[test]
    fn test_tier_display() {
        assert_eq!(Tier::Junior.to_string(), "junior");
        assert_eq!(Tier::Senior.to_string(), "senior");
    }

    #[test]
    fn test_tier_serialization() {
        let json = serde_json::to_string(&Tier::Senior).unwrap();
        assert_eq!(json, "\"senior\"");
        let parsed: Tier = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, Tier::Senior);
    }

    // v0.0.42: Golden tests for updated pinned names
    #[test]
    fn golden_network_junior_display() {
        let p = person_for(Team::Network, Tier::Junior);
        assert_eq!(p.display(), "Michael (Network Engineer)");
    }

    #[test]
    fn golden_storage_senior_display() {
        let p = person_for(Team::Storage, Tier::Senior);
        assert_eq!(p.display(), "Ines (Storage Architect)");
    }

    #[test]
    fn golden_performance_junior_display() {
        let p = person_for(Team::Performance, Tier::Junior);
        assert_eq!(p.display(), "Kari (Performance Analyst)");
    }

    #[test]
    fn golden_logs_team() {
        let jr = person_for(Team::Logs, Tier::Junior);
        assert_eq!(jr.display_name, "Daniel");
        assert_eq!(jr.role_title, "Logs Analyst");

        let sr = person_for(Team::Logs, Tier::Senior);
        assert_eq!(sr.display_name, "Lea");
        assert_eq!(sr.role_title, "Logs Engineer");
    }

    #[test]
    fn golden_all_pinned_names() {
        // v0.0.42: Verify all pinned names
        assert_eq!(
            person_for(Team::Network, Tier::Junior).display_name,
            "Michael"
        );
        assert_eq!(person_for(Team::Network, Tier::Senior).display_name, "Ana");
        assert_eq!(
            person_for(Team::Desktop, Tier::Junior).display_name,
            "Sofia"
        );
        assert_eq!(person_for(Team::Desktop, Tier::Senior).display_name, "Erik");
        assert_eq!(
            person_for(Team::Hardware, Tier::Junior).display_name,
            "Nora"
        );
        assert_eq!(person_for(Team::Hardware, Tier::Senior).display_name, "Jon");
        assert_eq!(person_for(Team::Storage, Tier::Junior).display_name, "Lars");
        assert_eq!(person_for(Team::Storage, Tier::Senior).display_name, "Ines");
        assert_eq!(
            person_for(Team::Performance, Tier::Junior).display_name,
            "Kari"
        );
        assert_eq!(
            person_for(Team::Performance, Tier::Senior).display_name,
            "Mateo"
        );
        assert_eq!(
            person_for(Team::Security, Tier::Junior).display_name,
            "Priya"
        );
        assert_eq!(
            person_for(Team::Security, Tier::Senior).display_name,
            "Oskar"
        );
        assert_eq!(
            person_for(Team::Services, Tier::Junior).display_name,
            "Hugo"
        );
        assert_eq!(
            person_for(Team::Services, Tier::Senior).display_name,
            "Mina"
        );
        assert_eq!(person_for(Team::Logs, Tier::Junior).display_name, "Daniel");
        assert_eq!(person_for(Team::Logs, Tier::Senior).display_name, "Lea");
        assert_eq!(
            person_for(Team::General, Tier::Junior).display_name,
            "Tomas"
        );
        assert_eq!(person_for(Team::General, Tier::Senior).display_name, "Sara");
    }
}
