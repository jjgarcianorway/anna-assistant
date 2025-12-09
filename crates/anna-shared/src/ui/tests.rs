//! Tests for UI module (v0.0.213).

#[cfg(test)]
mod tests {
    use crate::ui::formatting::{format_bytes, format_duration, progress_bar};
    use crate::ui::spinner::Spinner;
    use crate::ui::stage::StageProgress;
    use crate::ui::symbols;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024), "1.0 KiB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MiB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GiB");
    }

    #[test]
    fn test_progress_bar() {
        assert_eq!(progress_bar(0.5, 10), "[█████░░░░░]");
        assert_eq!(progress_bar(1.0, 10), "[██████████]");
        assert_eq!(progress_bar(0.0, 10), "[░░░░░░░░░░]");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(5), "00:00:05");
        assert_eq!(format_duration(65), "01:05");
        assert_eq!(format_duration(3665), "01:01:05");
    }

    #[test]
    fn test_spinner_new() {
        let spinner = Spinner::new("Loading...");
        assert!(spinner.is_running());
        assert_eq!(spinner.frame_char(), symbols::SPINNER[0]);
    }

    #[test]
    fn test_spinner_tick() {
        let mut spinner = Spinner::new("Loading...");
        spinner.tick();
        assert_eq!(spinner.frame_char(), symbols::SPINNER[1]);
        spinner.tick();
        assert_eq!(spinner.frame_char(), symbols::SPINNER[2]);
    }

    #[test]
    fn test_stage_progress() {
        let mut progress = StageProgress::new(&["translator", "probes", "specialist"]);
        progress.start("translator");
        progress.complete(100);
        progress.start("probes");
        progress.complete(200);
        progress.skip("specialist");

        assert!(progress.summary().contains("2/3"));
    }

    #[test]
    fn test_stage_status_render() {
        let mut progress = StageProgress::new(&["a", "b"]);
        progress.start("a");
        let line = progress.render_line();
        assert!(line.contains("◉")); // Running indicator
    }
}
