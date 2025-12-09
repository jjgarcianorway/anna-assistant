//! Tests for theatre_render module (v0.0.202).

#[cfg(test)]
mod tests {
    use crate::theatre_render::helpers::{probe_id_from_command, reliability_color, team_from_domain};
    use anna_shared::teams::Team;
    use anna_shared::ui::colors;

    #[test]
    fn test_team_from_domain() {
        assert_eq!(team_from_domain("storage"), Team::Storage);
        assert_eq!(team_from_domain("network"), Team::Network);
        assert_eq!(team_from_domain("unknown"), Team::General);
    }

    #[test]
    fn test_probe_id_from_command() {
        assert_eq!(probe_id_from_command("df -h"), "df");
        assert_eq!(probe_id_from_command("free -h"), "free");
        assert_eq!(
            probe_id_from_command("lspci | grep -i audio"),
            "lspci_audio"
        );
    }

    #[test]
    fn test_reliability_color() {
        assert_eq!(reliability_color(90), colors::OK);
        assert_eq!(reliability_color(60), colors::WARN);
        assert_eq!(reliability_color(30), colors::ERR);
    }
}
