// v0.0.753: Settings Precinct Tests (Phase 329)
// Unit tests for precinct functionality

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_precinct_type_display() {
        assert_eq!(format!("{}", PrecinctType::Voting), "voting");
        assert_eq!(format!("{}", PrecinctType::Police), "police");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", PrecinctStatus::Designated), "designated");
        assert_eq!(format!("{}", PrecinctStatus::Active), "active");
    }

    #[test]
    fn test_config_new() {
        let c = PrecinctConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = PrecinctConfig::new("test")
            .precinct_type(PrecinctType::Fire)
            .status(PrecinctStatus::Consolidated);
        assert_eq!(c.precinct_type, PrecinctType::Fire);
        assert_eq!(c.status, PrecinctStatus::Consolidated);
    }

    #[test]
    fn test_ballot_new() {
        let b = PrecinctBallot::new("b1", "Title", "Content");
        assert_eq!(b.id, "b1");
    }

    #[test]
    fn test_ballot_builder() {
        let b = PrecinctBallot::new("b1", "Title", "Content")
            .district(1);
        assert_eq!(b.district, 1);
    }

    #[test]
    fn test_ballot_certified() {
        let mut b = PrecinctBallot::new("b1", "Title", "Content");
        b.make_contested();
        assert!(!b.certified);
        b.make_certified();
        assert!(b.certified);
    }

    #[test]
    fn test_captain_new() {
        let c = PrecinctCaptain::new("key", "name", "b1");
        assert_eq!(c.ballot_id, "b1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = PrecinctStats::default();
        let ballot = PrecinctBallot::new("b1", "Title", "Content");
        s.update(&[ballot], PrecinctType::Voting);
        assert_eq!(s.total_ballots, 1);
        assert_eq!(s.certified, 1);
    }

    #[test]
    fn test_precinct_new() {
        let p = SettingsPrecinct::new(PrecinctConfig::default());
        assert_eq!(p.ballot_count(), 0);
    }

    #[test]
    fn test_precinct_add_ballot() {
        let mut p = SettingsPrecinct::new(PrecinctConfig::default());
        p.add_ballot(PrecinctBallot::new("b1", "Title", "Content"));
        assert_eq!(p.ballot_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = PrecinctRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = PrecinctRegistry::new();
        r.register("p1", SettingsPrecinct::new(PrecinctConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_precinct_query() {
        assert!(is_precinct_query("settings precinct"));
        assert!(!is_precinct_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = precinct_fun_fact();
        assert!(fact.contains("precinct"));
    }
}
