//! Greeting context and generation (v0.0.217).

/// Context for generating personalized greetings
#[derive(Debug, Clone)]
pub struct GreetingContext {
    pub username: String,
    pub days_away: i64,
    pub streak_days: u32,
    pub preferred_editor: Option<String>,
    pub top_topic: Option<String>,
    pub is_new_user: bool,
}

impl GreetingContext {
    /// Generate greeting message based on context
    pub fn generate_greeting(&self) -> String {
        if self.is_new_user {
            return format!(
                "Hello {}! Welcome to Anna. I'm your personal IT department. Ask me anything about your system!",
                self.username
            );
        }

        let time_part = match self.days_away {
            0 => "Good to see you again!".to_string(),
            1 => "Back again today! That's great.".to_string(),
            2..=6 => format!(
                "It's been {} days. I hope everything is running smoothly!",
                self.days_away
            ),
            _ => format!(
                "It's been a while ({} days)! Let me check if anything happened.",
                self.days_away
            ),
        };

        let streak_part = if self.streak_days > 1 {
            format!(" You're on a {} day streak!", self.streak_days)
        } else {
            String::new()
        };

        format!("Hello {}! {}{}", self.username, time_part, streak_part)
    }
}
