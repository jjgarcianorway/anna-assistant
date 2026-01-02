//! Tests for stats and UI components.

#[cfg(test)]
mod tests {
    use crate::source_layer::stats_ui::{
        CleanStats, ConfirmDialog, DialogQuestion, DialogResult, OutputMode, ProgressIndicator,
    };
    use crate::ticket_integrity::outcome::TicketOutcome;

    #[test]
    fn test_clean_stats() {
        let mut stats = CleanStats::new("today");
        stats.record(TicketOutcome::Answered, 100);
        stats.record(TicketOutcome::Answered, 200);
        stats.record(TicketOutcome::ParseError, 50);

        assert_eq!(stats.total, 3);
        assert_eq!(stats.answered(), 2);
        assert!((stats.success_rate() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_dialog_question() {
        let dialog = DialogQuestion::new("Which editor?")
            .choice("vim", "vim")
            .choice("nano", "nano")
            .with_other();

        let display = dialog.display();
        assert!(display.contains("Which editor"));
        assert!(display.contains("1) vim"));
        assert!(display.contains("2) nano"));
        assert!(display.contains("9) Something else"));
        assert!(display.contains("0) Cancel"));
    }

    #[test]
    fn test_dialog_parse() {
        let dialog = DialogQuestion::new("Choose")
            .choice("a", "a")
            .choice("b", "b");

        assert_eq!(
            dialog.parse_input("1"),
            DialogResult::Selected("a".to_string())
        );
        assert_eq!(
            dialog.parse_input("2"),
            DialogResult::Selected("b".to_string())
        );
        assert_eq!(dialog.parse_input("0"), DialogResult::Cancelled);
        assert!(matches!(dialog.parse_input("x"), DialogResult::Invalid(_)));
    }

    #[test]
    fn test_confirm_dialog() {
        let confirm = ConfirmDialog::new("Proceed?").default_yes();
        assert!(confirm.parse_input(""));
        assert!(confirm.parse_input("y"));
        assert!(!confirm.parse_input("n"));
    }

    #[test]
    fn test_progress_indicator() {
        let mut progress = ProgressIndicator::new(4);
        progress.advance("Step 1");

        assert_eq!(progress.current, 1);
        let display = progress.display();
        assert!(display.contains("[1/4]"));
        assert!(display.contains("Step 1"));
    }

    #[test]
    fn test_output_modes() {
        assert_eq!(
            OutputMode::from_flags(false, false, false),
            OutputMode::Normal
        );
        assert_eq!(
            OutputMode::from_flags(true, false, false),
            OutputMode::Plain
        );
        assert_eq!(OutputMode::from_flags(false, true, false), OutputMode::Json);
        assert_eq!(OutputMode::from_flags(false, false, true), OutputMode::Fun);
    }
}
