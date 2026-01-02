//! Tests for dialogue renderer - Phase 89

#[cfg(test)]
mod tests {
    use crate::dialogue_renderer::*;

    #[test]
    fn test_speaker() {
        assert_eq!(Speaker::Anna.name(), "Anna");
        assert!(!Speaker::User.color_code().is_empty());
    }

    #[test]
    fn test_dialogue_mood() {
        assert_eq!(DialogueMood::Neutral.prefix(), "");
        assert!(!DialogueMood::Confident.prefix().is_empty());
    }

    #[test]
    fn test_dialogue_new() {
        let dialogue = Dialogue::new();
        assert_eq!(dialogue.turn_count(), 0);
    }

    #[test]
    fn test_anna_says() {
        let mut dialogue = Dialogue::new();
        dialogue.anna_says("Hello!", DialogueMood::Helpful, 1234567890);

        assert_eq!(dialogue.turn_count(), 1);
        assert_eq!(dialogue.turns[0].speaker, Speaker::Anna);
    }

    #[test]
    fn test_user_says() {
        let mut dialogue = Dialogue::new();
        dialogue.user_says("I need help", 1234567890);

        assert_eq!(dialogue.turn_count(), 1);
        assert_eq!(dialogue.turns[0].speaker, Speaker::User);
    }

    #[test]
    fn test_specialist_says() {
        let mut dialogue = Dialogue::new();
        dialogue.specialist_says(
            Speaker::Junior,
            "Maya",
            "Desktop",
            "I can help with that",
            1234567890,
        );

        assert_eq!(dialogue.turn_count(), 1);
        assert!(dialogue.turns[0].internal);
        assert_eq!(dialogue.turns[0].speaker_name, Some("Maya".to_string()));
    }

    #[test]
    fn test_internal_external_turns() {
        let mut dialogue = Dialogue::new();
        dialogue.anna_says("Hello", DialogueMood::Neutral, 1);
        dialogue.user_says("Hi", 2);
        dialogue.specialist_says(Speaker::Junior, "Maya", "Desktop", "Internal", 3);

        assert_eq!(dialogue.external_turns().len(), 2);
        assert_eq!(dialogue.internal_turns().len(), 1);
    }

    #[test]
    fn test_render_dialogue() {
        let mut dialogue = Dialogue::new();
        dialogue.subject = Some("Test".to_string());
        dialogue.anna_says("Hello!", DialogueMood::Helpful, 1);

        let output = render_dialogue(&dialogue, false);
        assert!(output.contains("Test"));
        assert!(output.contains("Anna"));
    }

    #[test]
    fn test_render_dialogue_plain() {
        let mut dialogue = Dialogue::new();
        dialogue.anna_says("Hello!", DialogueMood::Neutral, 1);

        let output = render_dialogue_plain(&dialogue, false);
        assert!(output.contains("Anna"));
        assert!(output.contains("Hello!"));
    }

    #[test]
    fn test_render_dialogue_compact() {
        let mut dialogue = Dialogue::new();
        dialogue.anna_says("Hello!", DialogueMood::Neutral, 1);
        dialogue.user_says("Hi there", 2);

        let output = render_dialogue_compact(&dialogue);
        assert!(output.contains("[Anna]"));
        assert!(output.contains("[User]"));
    }

    #[test]
    fn test_is_dialogue_query() {
        assert!(is_dialogue_query("show conversation"));
        assert!(is_dialogue_query("fly on the wall view"));
        assert!(is_dialogue_query("internal communication"));
        assert!(!is_dialogue_query("what is the weather?"));
    }

    #[test]
    fn test_dialogue_fun_fact() {
        let mut dialogue = Dialogue::new();
        dialogue.anna_says("Hello", DialogueMood::Neutral, 1);

        let fact = dialogue_fun_fact(&dialogue);
        assert!(!fact.is_empty());
    }

    #[test]
    fn test_truncate() {
        // Note: truncate is a private function in renderers module
        // This test verifies the behavior through render_dialogue_compact
        let mut dialogue = Dialogue::new();
        dialogue.anna_says("this is a very long string that should be truncated because it exceeds the max length", DialogueMood::Neutral, 1);

        let output = render_dialogue_compact(&dialogue);
        assert!(output.contains("..."));
    }
}
