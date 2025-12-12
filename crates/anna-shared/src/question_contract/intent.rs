//! QuestionIntent v1 (Part A) - v0.0.437.
//!
//! The translator model must output exactly one QuestionIntent before
//! anything else happens. This is the typed contract for understanding.

use serde::{Deserialize, Serialize};

/// The primary intent classification for a user question.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionIntent {
    /// Unique identifier for this intent.
    pub intent_id: String,
    /// What kind of answer is expected.
    pub category: IntentCategory,
    /// What domain/subject the question is about.
    pub subject: Subject,
    /// How many items expected in answer.
    pub scope: Scope,
    /// Time period the question refers to.
    pub timeframe: Timeframe,
    /// How precise the answer needs to be.
    pub precision: Precision,
    /// Constraints on what the answer can contain.
    pub answer_constraints: Option<AnswerConstraints>,
    /// Whether evidence is required (almost always true).
    pub requires_evidence: bool,
    /// Whether synthesis/reasoning is needed (diagnosis, explanation).
    pub requires_synthesis: bool,
    /// Whether user confirmation is needed before acting.
    pub requires_user_confirmation: bool,
    /// If set, execution STOPS and Anna asks this question.
    pub clarification_needed: Option<ClarificationRequest>,
}

impl QuestionIntent {
    /// Create a new intent with defaults.
    pub fn new(intent_id: &str, category: IntentCategory, subject: Subject) -> Self {
        Self {
            intent_id: intent_id.to_string(),
            category,
            subject,
            scope: Scope::Single,
            timeframe: Timeframe::Now,
            precision: Precision::Exact,
            answer_constraints: None,
            requires_evidence: true,
            requires_synthesis: false,
            requires_user_confirmation: false,
            clarification_needed: None,
        }
    }

    /// Check if clarification is needed (blocks all execution).
    pub fn needs_clarification(&self) -> bool {
        self.clarification_needed.is_some()
    }

    /// Check if this is a meta question (about Anna itself).
    pub fn is_meta_question(&self) -> bool {
        self.subject == Subject::Meta
    }

    /// Check if extras are allowed in the answer.
    pub fn allows_extras(&self) -> bool {
        self.answer_constraints
            .as_ref()
            .map(|c| c.allow_extras)
            .unwrap_or(false)
    }

    /// Get allowed fields for the answer.
    pub fn allowed_fields(&self) -> Vec<String> {
        self.answer_constraints
            .as_ref()
            .map(|c| c.allowed_fields.clone())
            .unwrap_or_default()
    }

    /// Check if a specific field is allowed in the answer.
    pub fn is_field_allowed(&self, field: &str) -> bool {
        match &self.answer_constraints {
            None => true, // No constraints means all allowed
            Some(c) => {
                c.allow_extras || c.allowed_fields.iter().any(|f| f == field)
            }
        }
    }
}

/// Category of question - determines answer shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum IntentCategory {
    /// Simple fact retrieval (RAM, CPU model, etc.).
    Fact,
    /// System status check (service running, enabled, etc.).
    Status,
    /// Problem diagnosis requiring reasoning.
    Diagnosis,
    /// Explanation of how/why something works.
    Explanation,
    /// Request to perform an action.
    ActionRequest,
    /// Unknown - requires clarification.
    #[default]
    Unknown,
}

impl IntentCategory {
    /// Label for display.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Fact => "fact",
            Self::Status => "status",
            Self::Diagnosis => "diagnosis",
            Self::Explanation => "explanation",
            Self::ActionRequest => "action_request",
            Self::Unknown => "unknown",
        }
    }

    /// Whether this category allows tutorial/help text.
    pub fn allows_tutorials(&self) -> bool {
        matches!(self, Self::Explanation | Self::ActionRequest)
    }

    /// Whether this category requires synthesis.
    pub fn requires_synthesis(&self) -> bool {
        matches!(self, Self::Diagnosis | Self::Explanation)
    }

    /// Whether this category needs a conclusion.
    pub fn needs_conclusion(&self) -> bool {
        matches!(self, Self::Diagnosis)
    }
}

/// Subject domain of the question.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Subject {
    Cpu,
    Memory,
    Disk,
    Service,
    Network,
    Audio,
    Gpu,
    Boot,
    Packages,
    Desktop,
    Security,
    Kernel,
    Drivers,
    Power,
    Time,
    Users,
    Processes,
    /// Questions about Anna itself.
    Meta,
    /// Multiple subjects.
    Multiple,
    /// Unknown - requires clarification.
    #[default]
    Unknown,
}

impl Subject {
    /// Label for display.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Cpu => "cpu",
            Self::Memory => "memory",
            Self::Disk => "disk",
            Self::Service => "service",
            Self::Network => "network",
            Self::Audio => "audio",
            Self::Gpu => "gpu",
            Self::Boot => "boot",
            Self::Packages => "packages",
            Self::Desktop => "desktop",
            Self::Security => "security",
            Self::Kernel => "kernel",
            Self::Drivers => "drivers",
            Self::Power => "power",
            Self::Time => "time",
            Self::Users => "users",
            Self::Processes => "processes",
            Self::Meta => "meta",
            Self::Multiple => "multiple",
            Self::Unknown => "unknown",
        }
    }

    /// Common fields for this subject.
    pub fn common_fields(&self) -> Vec<&'static str> {
        match self {
            Self::Memory => vec!["total", "used", "free", "available", "cached", "swap_total", "swap_used"],
            Self::Cpu => vec!["model", "cores", "threads", "frequency", "usage", "temperature"],
            Self::Disk => vec!["total", "used", "free", "filesystem", "mount_point", "usage_percent"],
            Self::Service => vec!["name", "status", "enabled", "active", "description"],
            Self::Network => vec!["interface", "ip", "mac", "status", "speed", "gateway", "dns"],
            Self::Gpu => vec!["model", "driver", "memory", "temperature", "usage"],
            Self::Boot => vec!["time", "services", "kernel_time", "userspace_time"],
            Self::Packages => vec!["name", "version", "installed", "repository"],
            Self::Audio => vec!["device", "driver", "volume", "muted", "default"],
            Self::Power => vec!["battery", "charging", "time_remaining", "power_profile"],
            _ => vec![],
        }
    }
}

/// Scope of the answer - how many items expected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    /// Single value expected (e.g., "how much free RAM").
    #[default]
    Single,
    /// List of items expected (e.g., "which services are failed").
    List,
    /// Summary/overview expected (e.g., "system health").
    Summary,
    /// Boolean yes/no expected (e.g., "is zram enabled").
    Boolean,
}

impl Scope {
    /// Label for display.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Single => "single",
            Self::List => "list",
            Self::Summary => "summary",
            Self::Boolean => "boolean",
        }
    }
}

/// Timeframe the question refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Timeframe {
    /// Current state (default for most questions).
    #[default]
    Now,
    /// Since last boot.
    LastBoot,
    /// Today only.
    Today,
    /// Since system was installed.
    SinceInstall,
    /// Historical (needs more context).
    Historical,
}

/// Precision level required.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Precision {
    /// Exact value required (3.14159).
    #[default]
    Exact,
    /// Approximate is acceptable (about 3).
    Approximate,
}

/// Constraints on what the answer can contain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerConstraints {
    /// Maximum items in list (None = unlimited).
    pub max_items: Option<usize>,
    /// Whether extra information is allowed beyond allowed_fields.
    /// DEFAULT IS FALSE - minimal answers by default.
    pub allow_extras: bool,
    /// Explicit list of fields allowed in the answer.
    pub allowed_fields: Vec<String>,
    /// Units for numeric values.
    pub units: Units,
}

impl Default for AnswerConstraints {
    fn default() -> Self {
        Self {
            max_items: Some(super::DEFAULT_MAX_ITEMS),
            allow_extras: false, // CRITICAL: default is NO extras
            allowed_fields: Vec::new(),
            units: Units::Human,
        }
    }
}

impl AnswerConstraints {
    /// Create constraints for a single-value fact.
    pub fn single_fact(field: &str) -> Self {
        Self {
            max_items: Some(1),
            allow_extras: false,
            allowed_fields: vec![field.to_string()],
            units: Units::Human,
        }
    }

    /// Create constraints for a boolean answer.
    pub fn boolean() -> Self {
        Self {
            max_items: Some(1),
            allow_extras: false,
            allowed_fields: vec!["result".to_string()],
            units: Units::Human,
        }
    }

    /// Create constraints for a list of items.
    pub fn list(field: &str, max: usize) -> Self {
        Self {
            max_items: Some(max),
            allow_extras: false,
            allowed_fields: vec![field.to_string()],
            units: Units::Human,
        }
    }

    /// Create constraints that allow extras (for diagnosis/explanation).
    pub fn with_extras(fields: Vec<String>) -> Self {
        Self {
            max_items: None,
            allow_extras: true,
            allowed_fields: fields,
            units: Units::Human,
        }
    }
}

/// Units for numeric values in answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Units {
    /// Raw bytes.
    Bytes,
    /// Percentage (0-100).
    Percent,
    /// Seconds.
    Seconds,
    /// Human-readable (auto-scale).
    #[default]
    Human,
}

/// Request for clarification - STOPS all execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarificationRequest {
    /// The question to ask the user.
    pub question: String,
    /// Choices for the user to pick from.
    pub choices: Vec<String>,
}

impl ClarificationRequest {
    /// Create a new clarification request.
    pub fn new(question: &str, choices: Vec<&str>) -> Self {
        Self {
            question: question.to_string(),
            choices: choices.into_iter().map(String::from).collect(),
        }
    }
}

/// Builder for constructing QuestionIntent.
pub struct IntentBuilder {
    intent: QuestionIntent,
}

impl IntentBuilder {
    /// Start building a new intent.
    pub fn new(intent_id: &str) -> Self {
        Self {
            intent: QuestionIntent::new(intent_id, IntentCategory::Unknown, Subject::Unknown),
        }
    }

    /// Set the category.
    pub fn category(mut self, category: IntentCategory) -> Self {
        self.intent.category = category;
        self.intent.requires_synthesis = category.requires_synthesis();
        self
    }

    /// Set the subject.
    pub fn subject(mut self, subject: Subject) -> Self {
        self.intent.subject = subject;
        self
    }

    /// Set the scope.
    pub fn scope(mut self, scope: Scope) -> Self {
        self.intent.scope = scope;
        self
    }

    /// Set the timeframe.
    pub fn timeframe(mut self, timeframe: Timeframe) -> Self {
        self.intent.timeframe = timeframe;
        self
    }

    /// Set answer constraints.
    pub fn constraints(mut self, constraints: AnswerConstraints) -> Self {
        self.intent.answer_constraints = Some(constraints);
        self
    }

    /// Allow only specific fields.
    pub fn allow_fields(mut self, fields: Vec<&str>) -> Self {
        let constraints = self.intent.answer_constraints.get_or_insert_with(Default::default);
        constraints.allowed_fields = fields.into_iter().map(String::from).collect();
        constraints.allow_extras = false;
        self
    }

    /// Set clarification needed (stops execution).
    pub fn needs_clarification(mut self, question: &str, choices: Vec<&str>) -> Self {
        self.intent.clarification_needed = Some(ClarificationRequest::new(question, choices));
        self
    }

    /// Enable extras (for diagnosis/explanation only).
    pub fn allow_extras(mut self) -> Self {
        let constraints = self.intent.answer_constraints.get_or_insert_with(Default::default);
        constraints.allow_extras = true;
        self
    }

    /// Set units.
    pub fn units(mut self, units: Units) -> Self {
        let constraints = self.intent.answer_constraints.get_or_insert_with(Default::default);
        constraints.units = units;
        self
    }

    /// Build the intent.
    pub fn build(self) -> QuestionIntent {
        self.intent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_builder() {
        let intent = IntentBuilder::new("int_001")
            .category(IntentCategory::Fact)
            .subject(Subject::Memory)
            .scope(Scope::Single)
            .allow_fields(vec!["free"])
            .build();

        assert_eq!(intent.category, IntentCategory::Fact);
        assert_eq!(intent.subject, Subject::Memory);
        assert!(!intent.allows_extras());
        assert!(intent.is_field_allowed("free"));
        assert!(!intent.is_field_allowed("total"));
    }

    #[test]
    fn test_clarification_stops_execution() {
        let intent = IntentBuilder::new("int_002")
            .needs_clarification("Which service?", vec!["nginx", "apache", "postgresql"])
            .build();

        assert!(intent.needs_clarification());
    }

    #[test]
    fn test_category_allows_tutorials() {
        assert!(!IntentCategory::Fact.allows_tutorials());
        assert!(!IntentCategory::Status.allows_tutorials());
        assert!(IntentCategory::Explanation.allows_tutorials());
        assert!(IntentCategory::ActionRequest.allows_tutorials());
    }

    #[test]
    fn test_constraints_default_no_extras() {
        let constraints = AnswerConstraints::default();
        assert!(!constraints.allow_extras);
    }

    #[test]
    fn test_single_fact_constraints() {
        let constraints = AnswerConstraints::single_fact("free_ram");
        assert_eq!(constraints.max_items, Some(1));
        assert!(!constraints.allow_extras);
        assert_eq!(constraints.allowed_fields, vec!["free_ram"]);
    }

    #[test]
    fn test_boolean_constraints() {
        let constraints = AnswerConstraints::boolean();
        assert_eq!(constraints.max_items, Some(1));
        assert!(!constraints.allow_extras);
    }

    #[test]
    fn test_field_allowed() {
        let intent = IntentBuilder::new("int_003")
            .allow_fields(vec!["free", "total"])
            .build();

        assert!(intent.is_field_allowed("free"));
        assert!(intent.is_field_allowed("total"));
        assert!(!intent.is_field_allowed("cached"));
    }

    #[test]
    fn test_extras_allowed() {
        let intent = IntentBuilder::new("int_004")
            .category(IntentCategory::Diagnosis)
            .allow_extras()
            .build();

        assert!(intent.allows_extras());
        assert!(intent.is_field_allowed("anything"));
    }
}
