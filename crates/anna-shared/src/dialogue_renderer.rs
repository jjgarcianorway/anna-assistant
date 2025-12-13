//! Dialogue Renderer - Phase 89
//!
//! Renders specialist conversations in natural language for fly-on-the-wall display.
//! VISION.md: "Show natural language dialog between players"
//! "User reads the whole communication like a fly on the wall"

use serde::{Deserialize, Serialize};

/// Speaker in a dialogue
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Speaker {
    #[default]
    Anna,
    User,
    Junior,
    Senior,
    Lead,
    System,
}

impl Speaker {
    pub fn name(&self) -> &'static str {
        match self {
            Speaker::Anna => "Anna",
            Speaker::User => "User",
            Speaker::Junior => "Junior",
            Speaker::Senior => "Senior",
            Speaker::Lead => "Lead",
            Speaker::System => "System",
        }
    }

    pub fn color_code(&self) -> &'static str {
        match self {
            Speaker::Anna => "\x1b[36m",      // Cyan
            Speaker::User => "\x1b[32m",      // Green
            Speaker::Junior => "\x1b[33m",    // Yellow
            Speaker::Senior => "\x1b[35m",    // Magenta
            Speaker::Lead => "\x1b[34m",      // Blue
            Speaker::System => "\x1b[90m",    // Gray
        }
    }
}

/// Dialogue mood/tone
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DialogueMood {
    #[default]
    Neutral,
    Confident,
    Uncertain,
    Apologetic,
    Helpful,
    Thinking,
}

impl DialogueMood {
    pub fn prefix(&self) -> &'static str {
        match self {
            DialogueMood::Neutral => "",
            DialogueMood::Confident => "I know this! ",
            DialogueMood::Uncertain => "I'm not entirely sure, but ",
            DialogueMood::Apologetic => "I apologize, ",
            DialogueMood::Helpful => "Let me help you. ",
            DialogueMood::Thinking => "Let me think... ",
        }
    }
}

/// A single dialogue turn
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueTurn {
    /// Who is speaking
    pub speaker: Speaker,
    /// Speaker's name (human name)
    pub speaker_name: Option<String>,
    /// Department (if specialist)
    pub department: Option<String>,
    /// The message content
    pub content: String,
    /// Mood/tone
    pub mood: DialogueMood,
    /// Timestamp
    pub timestamp: u64,
    /// Is internal communication
    pub internal: bool,
}

/// A complete dialogue
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Dialogue {
    /// Ticket ID this dialogue belongs to
    pub ticket_id: Option<String>,
    /// All turns in order
    pub turns: Vec<DialogueTurn>,
    /// Subject/topic
    pub subject: Option<String>,
}

impl Dialogue {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a turn
    pub fn add_turn(&mut self, turn: DialogueTurn) {
        self.turns.push(turn);
    }

    /// Add Anna speaking
    pub fn anna_says(&mut self, content: &str, mood: DialogueMood, timestamp: u64) {
        self.turns.push(DialogueTurn {
            speaker: Speaker::Anna,
            speaker_name: Some("Anna".to_string()),
            department: None,
            content: content.to_string(),
            mood,
            timestamp,
            internal: false,
        });
    }

    /// Add user speaking
    pub fn user_says(&mut self, content: &str, timestamp: u64) {
        self.turns.push(DialogueTurn {
            speaker: Speaker::User,
            speaker_name: None,
            department: None,
            content: content.to_string(),
            mood: DialogueMood::Neutral,
            timestamp,
            internal: false,
        });
    }

    /// Add specialist speaking (internal)
    pub fn specialist_says(
        &mut self,
        speaker: Speaker,
        name: &str,
        department: &str,
        content: &str,
        timestamp: u64,
    ) {
        self.turns.push(DialogueTurn {
            speaker,
            speaker_name: Some(name.to_string()),
            department: Some(department.to_string()),
            content: content.to_string(),
            mood: DialogueMood::Neutral,
            timestamp,
            internal: true,
        });
    }

    /// Get turn count
    pub fn turn_count(&self) -> usize {
        self.turns.len()
    }

    /// Get internal turns only
    pub fn internal_turns(&self) -> Vec<&DialogueTurn> {
        self.turns.iter().filter(|t| t.internal).collect()
    }

    /// Get external (user-facing) turns only
    pub fn external_turns(&self) -> Vec<&DialogueTurn> {
        self.turns.iter().filter(|t| !t.internal).collect()
    }
}

/// Render a dialogue for display
pub fn render_dialogue(dialogue: &Dialogue, show_internal: bool) -> String {
    let mut lines = Vec::new();

    // Header
    if let Some(subject) = &dialogue.subject {
        lines.push(format!("--- {} ---", subject));
        lines.push(String::new());
    }

    let reset = "\x1b[0m";

    for turn in &dialogue.turns {
        // Skip internal if not showing
        if turn.internal && !show_internal {
            continue;
        }

        // Internal marker
        if turn.internal && show_internal {
            lines.push("--- Internal communication ---".to_string());
        }

        // Speaker line
        let speaker_display = if let Some(name) = &turn.speaker_name {
            if let Some(dept) = &turn.department {
                format!("{} ({})", name, dept)
            } else {
                name.clone()
            }
        } else {
            turn.speaker.name().to_string()
        };

        let color = turn.speaker.color_code();
        let prefix = turn.mood.prefix();

        lines.push(format!(
            "{}{}{}: {}{}",
            color, speaker_display, reset, prefix, turn.content
        ));
        lines.push(String::new());
    }

    lines.join("\n")
}

/// Render dialogue without colors
pub fn render_dialogue_plain(dialogue: &Dialogue, show_internal: bool) -> String {
    let mut lines = Vec::new();

    if let Some(subject) = &dialogue.subject {
        lines.push(format!("--- {} ---", subject));
        lines.push(String::new());
    }

    for turn in &dialogue.turns {
        if turn.internal && !show_internal {
            continue;
        }

        if turn.internal && show_internal {
            lines.push("--- Internal communication ---".to_string());
        }

        let speaker_display = if let Some(name) = &turn.speaker_name {
            if let Some(dept) = &turn.department {
                format!("{} ({})", name, dept)
            } else {
                name.clone()
            }
        } else {
            turn.speaker.name().to_string()
        };

        let prefix = turn.mood.prefix();
        lines.push(format!("{}: {}{}", speaker_display, prefix, turn.content));
        lines.push(String::new());
    }

    lines.join("\n")
}

/// Render dialogue compact (one line per turn)
pub fn render_dialogue_compact(dialogue: &Dialogue) -> String {
    dialogue
        .external_turns()
        .iter()
        .map(|t| {
            let name = t.speaker_name.as_deref().unwrap_or(t.speaker.name());
            format!("[{}] {}", name, truncate(&t.content, 50))
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

/// Truncate string
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Check if query is about dialogue
pub fn is_dialogue_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "show dialogue",
        "show conversation",
        "internal communication",
        "what did they say",
        "fly on the wall",
        "show discussion",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Generate dialogue fun fact
pub fn dialogue_fun_fact(dialogue: &Dialogue) -> String {
    if dialogue.turns.is_empty() {
        return "No dialogue recorded yet.".to_string();
    }

    let facts = [
        format!("This dialogue has {} turns.", dialogue.turn_count()),
        format!(
            "{} internal exchanges occurred.",
            dialogue.internal_turns().len()
        ),
        format!(
            "{} messages to/from the user.",
            dialogue.external_turns().len()
        ),
    ];

    facts[dialogue.turn_count() % facts.len()].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a long string", 10), "this is...");
    }
}
