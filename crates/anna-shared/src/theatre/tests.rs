//! Theatre tests (v0.0.226).

#[cfg(test)]
mod tests {
    use crate::roster::Tier;
    use crate::teams::Team;
    use crate::theatre::{
        describe_check, format_case_id, NarrativeBuilder, NarrativeSegment, Speaker,
    };

    #[test]
    fn test_speaker_display() {
        assert_eq!(Speaker::Anna.display_name(), "Anna");
        assert_eq!(Speaker::You.display_name(), "you");

        let member = Speaker::from_team(Team::Network, Tier::Junior);
        assert_eq!(member.display_name(), "Michael");
    }

    #[test]
    fn test_speaker_display_with_role() {
        let member = Speaker::from_team(Team::Storage, Tier::Senior);
        assert_eq!(member.display_with_role(), "Ines (Storage Architect)");
    }

    #[test]
    fn test_narrative_segment_anna() {
        let seg = NarrativeSegment::anna("Hello!");
        assert_eq!(seg.speaker, Speaker::Anna);
        assert!(!seg.internal);
    }

    #[test]
    fn test_narrative_segment_team() {
        let seg = NarrativeSegment::team_member(Team::Desktop, Tier::Junior, "On it!");
        if let Speaker::TeamMember { name, .. } = &seg.speaker {
            assert_eq!(name, "Sofia");
        } else {
            panic!("Expected TeamMember speaker");
        }
        assert!(seg.internal);
    }

    #[test]
    fn test_narrative_builder() {
        let mut builder = NarrativeBuilder::new().with_internal_comms();
        builder.add_greeting("storage");
        builder.add_checking("disk space");
        builder.add_dispatch(Team::Storage, "abc12345");

        let narrative = builder.build();
        assert_eq!(narrative.len(), 3);
    }

    #[test]
    fn test_describe_check() {
        let probes = vec!["df".to_string(), "free".to_string()];
        let desc = describe_check(&probes);
        assert!(desc.contains("disk"));
        assert!(desc.contains("memory"));
    }

    #[test]
    fn test_format_case_id() {
        let case = format_case_id("abc123456789");
        assert_eq!(case, "CN-ABC12345");
    }

    #[test]
    fn test_empty_describe_check() {
        let probes: Vec<String> = vec![];
        let desc = describe_check(&probes);
        assert_eq!(desc, "system data");
    }
}
