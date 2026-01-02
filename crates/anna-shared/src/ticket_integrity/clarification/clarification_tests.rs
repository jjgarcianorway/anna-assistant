//! Clarification Tests - v0.0.442.

#[cfg(test)]
mod tests {
    use crate::ticket_integrity::clarification::{
        clarification_facts::{FactSource, KnownFacts},
        clarification_types::ClarificationRequiredIntent,
    };

    #[test]
    fn test_clarification_required_intents() {
        let intent = ClarificationRequiredIntent::EditorSyntaxStatus;
        let required = intent.required_facts();
        assert!(required.contains(&"editor.name"));
        assert!(required.contains(&"editor.config_path"));
    }

    #[test]
    fn test_clarification_question() {
        let intent = ClarificationRequiredIntent::EditorSyntaxStatus;
        let question = intent.clarification_for("editor.name").unwrap();
        assert!(question.question.contains("editor"));
        assert!(!question.options.is_empty());
    }

    #[test]
    fn test_known_facts() {
        let mut facts = KnownFacts::new();
        assert!(!facts.has("editor.name"));

        facts.add("editor.name", "vim", FactSource::User);
        assert!(facts.has("editor.name"));
        assert_eq!(facts.get("editor.name").unwrap().value, "vim");
    }
}
