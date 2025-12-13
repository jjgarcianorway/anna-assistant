// v0.0.535: Greeting Generator (Phase 111)
// Generates personalized greetings with insights per VISION.md

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Time of day for greeting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimeOfDay {
    Morning,
    Afternoon,
    Evening,
    Night,
}

impl Default for TimeOfDay {
    fn default() -> Self {
        Self::Morning
    }
}

impl std::fmt::Display for TimeOfDay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Morning => write!(f, "Morning"),
            Self::Afternoon => write!(f, "Afternoon"),
            Self::Evening => write!(f, "Evening"),
            Self::Night => write!(f, "Night"),
        }
    }
}

impl TimeOfDay {
    /// Get time of day from hour (0-23)
    pub fn from_hour(hour: u8) -> Self {
        match hour {
            5..=11 => Self::Morning,
            12..=17 => Self::Afternoon,
            18..=21 => Self::Evening,
            _ => Self::Night,
        }
    }

    /// Get greeting prefix
    pub fn greeting_prefix(&self) -> &'static str {
        match self {
            Self::Morning => "Good morning",
            Self::Afternoon => "Good afternoon",
            Self::Evening => "Good evening",
            Self::Night => "Hello",
        }
    }
}

/// Type of insight to include
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InsightType {
    TimeSinceLastVisit,
    BootTimeChange,
    UsagePattern,
    PendingTickets,
    SystemHealth,
    NewFeature,
    Tip,
    Warning,
    Error,
}

impl std::fmt::Display for InsightType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TimeSinceLastVisit => write!(f, "Time Since Last Visit"),
            Self::BootTimeChange => write!(f, "Boot Time Change"),
            Self::UsagePattern => write!(f, "Usage Pattern"),
            Self::PendingTickets => write!(f, "Pending Tickets"),
            Self::SystemHealth => write!(f, "System Health"),
            Self::NewFeature => write!(f, "New Feature"),
            Self::Tip => write!(f, "Tip"),
            Self::Warning => write!(f, "Warning"),
            Self::Error => write!(f, "Error"),
        }
    }
}

/// Individual insight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreetingInsight {
    pub insight_type: InsightType,
    pub message: String,
    pub priority: u8,
    pub actionable: bool,
}

impl GreetingInsight {
    /// Create new insight
    pub fn new(insight_type: InsightType, message: &str, priority: u8) -> Self {
        Self {
            insight_type,
            message: message.to_string(),
            priority,
            actionable: false,
        }
    }

    /// Mark as actionable
    pub fn actionable(mut self) -> Self {
        self.actionable = true;
        self
    }
}

/// Greeting context
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GreetingContext {
    pub username: String,
    pub time_of_day: TimeOfDay,
    pub last_visit_ago: Option<String>,
    pub insights: Vec<GreetingInsight>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl GreetingContext {
    /// Create new context
    pub fn new(username: &str, hour: u8) -> Self {
        Self {
            username: username.to_string(),
            time_of_day: TimeOfDay::from_hour(hour),
            last_visit_ago: None,
            insights: Vec::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Set last visit
    pub fn set_last_visit(&mut self, ago: &str) {
        self.last_visit_ago = Some(ago.to_string());
    }

    /// Add insight
    pub fn add_insight(&mut self, insight: GreetingInsight) {
        self.insights.push(insight);
    }

    /// Add error
    pub fn add_error(&mut self, error: &str) {
        self.errors.push(error.to_string());
    }

    /// Add warning
    pub fn add_warning(&mut self, warning: &str) {
        self.warnings.push(warning.to_string());
    }

    /// Has errors?
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Has warnings?
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Get sorted insights (by priority)
    pub fn sorted_insights(&self) -> Vec<&GreetingInsight> {
        let mut sorted: Vec<_> = self.insights.iter().collect();
        sorted.sort_by(|a, b| b.priority.cmp(&a.priority));
        sorted
    }
}

/// Greeting generator
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GreetingGenerator {
    templates: HashMap<TimeOfDay, Vec<String>>,
    insight_count_limit: usize,
}

impl GreetingGenerator {
    /// Create new generator
    pub fn new() -> Self {
        let mut gen = Self {
            templates: HashMap::new(),
            insight_count_limit: 3,
        };
        gen.load_default_templates();
        gen
    }

    /// Load default templates
    fn load_default_templates(&mut self) {
        self.templates.insert(
            TimeOfDay::Morning,
            vec![
                "Rise and shine, {}!".to_string(),
                "Good morning, {}!".to_string(),
                "Hello {}, ready for a productive day?".to_string(),
            ],
        );
        self.templates.insert(
            TimeOfDay::Afternoon,
            vec![
                "Good afternoon, {}!".to_string(),
                "Hello {}, how's your day going?".to_string(),
                "Hey {}, hope you're having a great day!".to_string(),
            ],
        );
        self.templates.insert(
            TimeOfDay::Evening,
            vec![
                "Good evening, {}!".to_string(),
                "Hello {}, winding down?".to_string(),
                "Evening, {}!".to_string(),
            ],
        );
        self.templates.insert(
            TimeOfDay::Night,
            vec![
                "Hello, {}!".to_string(),
                "Hey {}, burning the midnight oil?".to_string(),
                "Greetings, {}!".to_string(),
            ],
        );
    }

    /// Set insight limit
    pub fn set_insight_limit(&mut self, limit: usize) {
        self.insight_count_limit = limit;
    }

    /// Generate greeting
    pub fn generate(&self, context: &GreetingContext) -> String {
        let mut output = String::new();

        // Main greeting
        let templates = self.templates.get(&context.time_of_day).unwrap();
        let template = &templates[0]; // In real impl, randomize
        output.push_str(&template.replace("{}", &context.username));
        output.push('\n');

        // Last visit info
        if let Some(ago) = &context.last_visit_ago {
            output.push_str(&format!("\nIt's been {} since you last reached out!\n", ago));
        }

        // Errors (high priority)
        if context.has_errors() {
            output.push_str("\n⚠ There are some issues that need attention:\n");
            for error in &context.errors {
                output.push_str(&format!("  • {}\n", error));
            }
        }

        // Warnings
        if context.has_warnings() {
            output.push_str("\nHeads up:\n");
            for warning in &context.warnings {
                output.push_str(&format!("  • {}\n", warning));
            }
        }

        // Insights
        let insights = context.sorted_insights();
        if !insights.is_empty() {
            output.push_str("\nHere are a few updates:\n");
            for insight in insights.iter().take(self.insight_count_limit) {
                output.push_str(&format!("\n• {}\n", insight.message));
            }
        }

        // Closing
        if !context.has_errors() && !context.has_warnings() {
            output.push_str("\nNo warnings or errors detected. Everything looks good!\n");
        }

        output.push_str("\nBut I think you're asking me something specific, aren't you?\n");

        output
    }
}

/// Format insight for display
pub fn format_insight(insight: &GreetingInsight) -> String {
    format!(
        "[{}] {} (priority: {}){}",
        insight.insight_type,
        insight.message,
        insight.priority,
        if insight.actionable { " [Action needed]" } else { "" }
    )
}

/// Format context summary
pub fn format_context_summary(context: &GreetingContext) -> String {
    format!(
        "User: {} | Time: {} | Insights: {} | Errors: {} | Warnings: {}",
        context.username,
        context.time_of_day,
        context.insights.len(),
        context.errors.len(),
        context.warnings.len()
    )
}

/// Check if query is greeting-related
pub fn is_greeting_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("hello")
        || lower.contains("hi ")
        || lower.contains("hey")
        || lower.contains("greet")
        || lower.contains("good morning")
        || lower.contains("good evening")
}

/// Fun fact about greetings
pub fn greeting_fun_fact() -> &'static str {
    "Anna remembers when you last visited and tracks changes to your system - like a friend who actually pays attention!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_of_day_from_hour() {
        assert_eq!(TimeOfDay::from_hour(8), TimeOfDay::Morning);
        assert_eq!(TimeOfDay::from_hour(14), TimeOfDay::Afternoon);
        assert_eq!(TimeOfDay::from_hour(20), TimeOfDay::Evening);
        assert_eq!(TimeOfDay::from_hour(2), TimeOfDay::Night);
    }

    #[test]
    fn test_greeting_prefix() {
        assert_eq!(TimeOfDay::Morning.greeting_prefix(), "Good morning");
        assert_eq!(TimeOfDay::Night.greeting_prefix(), "Hello");
    }

    #[test]
    fn test_insight_creation() {
        let insight = GreetingInsight::new(InsightType::BootTimeChange, "Boot time increased", 5);
        assert_eq!(insight.priority, 5);
        assert!(!insight.actionable);
    }

    #[test]
    fn test_insight_actionable() {
        let insight = GreetingInsight::new(InsightType::Error, "Service failed", 10).actionable();
        assert!(insight.actionable);
    }

    #[test]
    fn test_context_creation() {
        let context = GreetingContext::new("lhoqvso", 10);
        assert_eq!(context.username, "lhoqvso");
        assert_eq!(context.time_of_day, TimeOfDay::Morning);
    }

    #[test]
    fn test_context_errors() {
        let mut context = GreetingContext::new("user", 12);
        assert!(!context.has_errors());
        context.add_error("Service failed");
        assert!(context.has_errors());
    }

    #[test]
    fn test_sorted_insights() {
        let mut context = GreetingContext::new("user", 12);
        context.add_insight(GreetingInsight::new(InsightType::Tip, "Low priority", 1));
        context.add_insight(GreetingInsight::new(InsightType::Error, "High priority", 10));
        let sorted = context.sorted_insights();
        assert_eq!(sorted[0].priority, 10);
    }

    #[test]
    fn test_generator_generate() {
        let gen = GreetingGenerator::new();
        let context = GreetingContext::new("testuser", 10);
        let greeting = gen.generate(&context);
        assert!(greeting.contains("testuser"));
    }

    #[test]
    fn test_is_greeting_query() {
        assert!(is_greeting_query("Hello Anna"));
        assert!(is_greeting_query("Hey there"));
        assert!(is_greeting_query("Good morning"));
        assert!(!is_greeting_query("Install vim"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = greeting_fun_fact();
        assert!(fact.contains("remember") || fact.contains("friend"));
    }
}
