//! AnswerPlan and Shape Enforcement (Part B) - v0.0.437.
//!
//! Before rendering a final answer:
//! - Anna builds an AnswerPlan from QuestionIntent
//! - Any data not mapped to allowed_fields is DISCARDED
//! - Any specialist output violating constraints is IGNORED or TRUNCATED
//!
//! This rule is ABSOLUTE.

use super::intent::{IntentCategory, QuestionIntent, Scope, Units};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A field in the answer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerField {
    /// Field name.
    pub name: String,
    /// Field value.
    pub value: AnswerValue,
    /// Evidence ID that supports this field.
    pub evidence_id: Option<String>,
}

impl AnswerField {
    /// Create a new answer field.
    pub fn new(name: &str, value: AnswerValue) -> Self {
        Self {
            name: name.to_string(),
            value,
            evidence_id: None,
        }
    }

    /// Attach evidence to this field.
    pub fn with_evidence(mut self, evidence_id: &str) -> Self {
        self.evidence_id = Some(evidence_id.to_string());
        self
    }
}

/// Value types for answer fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnswerValue {
    /// String value.
    String(String),
    /// Numeric value.
    Number(f64),
    /// Boolean value.
    Boolean(bool),
    /// List of strings.
    StringList(Vec<String>),
    /// List of key-value pairs.
    ObjectList(Vec<HashMap<String, String>>),
}

impl AnswerValue {
    /// Format value with units.
    pub fn format_with_units(&self, units: Units) -> String {
        match (self, units) {
            (AnswerValue::Number(n), Units::Bytes) => format_bytes(*n as u64),
            (AnswerValue::Number(n), Units::Percent) => format!("{:.1}%", n),
            (AnswerValue::Number(n), Units::Seconds) => format_duration(*n),
            (AnswerValue::Number(n), Units::Human) => format!("{}", n),
            (AnswerValue::String(s), _) => s.clone(),
            (AnswerValue::Boolean(b), _) => if *b { "yes" } else { "no" }.to_string(),
            (AnswerValue::StringList(list), _) => list.join(", "),
            (AnswerValue::ObjectList(_), _) => "[complex data]".to_string(),
        }
    }
}

/// Shape of the expected answer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerShape {
    /// What kind of answer shape.
    pub shape_type: ShapeType,
    /// Allowed field names.
    pub allowed_fields: Vec<String>,
    /// Maximum items (for lists).
    pub max_items: Option<usize>,
    /// Whether extras are allowed.
    pub allow_extras: bool,
    /// Units for numeric values.
    pub units: Units,
}

/// Type of answer shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShapeType {
    /// Single value answer.
    SingleValue,
    /// Boolean yes/no.
    Boolean,
    /// List of items.
    List,
    /// Key-value pairs.
    KeyValue,
    /// Free-form (only for diagnosis/explanation with extras).
    FreeForm,
}

impl AnswerShape {
    /// Create shape from intent.
    pub fn from_intent(intent: &QuestionIntent) -> Self {
        let shape_type = match intent.scope {
            Scope::Single => ShapeType::SingleValue,
            Scope::Boolean => ShapeType::Boolean,
            Scope::List => ShapeType::List,
            Scope::Summary => ShapeType::KeyValue,
        };

        let (allowed_fields, max_items, allow_extras, units) = match &intent.answer_constraints {
            Some(c) => (
                c.allowed_fields.clone(),
                c.max_items,
                c.allow_extras,
                c.units,
            ),
            None => (Vec::new(), None, false, Units::Human),
        };

        Self {
            shape_type,
            allowed_fields,
            max_items,
            allow_extras,
            units,
        }
    }

    /// Check if a field is allowed.
    pub fn is_field_allowed(&self, field: &str) -> bool {
        self.allow_extras || self.allowed_fields.iter().any(|f| f == field)
    }
}

/// The answer plan - what will be rendered.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerPlan {
    /// Intent this plan is for.
    pub intent_id: String,
    /// Expected shape.
    pub shape: AnswerShape,
    /// Fields to include in answer.
    pub fields: Vec<AnswerField>,
    /// Items that were discarded (for debugging).
    pub discarded: Vec<DiscardedItem>,
}

impl AnswerPlan {
    /// Create a new answer plan from intent.
    pub fn new(intent: &QuestionIntent) -> Self {
        Self {
            intent_id: intent.intent_id.clone(),
            shape: AnswerShape::from_intent(intent),
            fields: Vec::new(),
            discarded: Vec::new(),
        }
    }

    /// Add a field to the plan (will be filtered).
    pub fn add_field(&mut self, field: AnswerField) {
        if self.shape.is_field_allowed(&field.name) {
            // Check max items for lists
            if let Some(max) = self.shape.max_items {
                if self.fields.len() >= max {
                    self.discarded.push(DiscardedItem {
                        field_name: field.name,
                        reason: DiscardReason::MaxItemsExceeded,
                    });
                    return;
                }
            }
            self.fields.push(field);
        } else {
            self.discarded.push(DiscardedItem {
                field_name: field.name,
                reason: DiscardReason::NotAllowed,
            });
        }
    }

    /// Check if the plan is complete (has required fields).
    pub fn is_complete(&self) -> bool {
        if self.shape.allowed_fields.is_empty() {
            !self.fields.is_empty()
        } else {
            // All required fields must be present
            self.shape
                .allowed_fields
                .iter()
                .all(|required| self.fields.iter().any(|f| &f.name == required))
        }
    }

    /// Get missing required fields.
    pub fn missing_fields(&self) -> Vec<String> {
        self.shape
            .allowed_fields
            .iter()
            .filter(|required| !self.fields.iter().any(|f| &f.name == *required))
            .cloned()
            .collect()
    }

    /// Render the answer as a string.
    pub fn render(&self) -> String {
        match self.shape.shape_type {
            ShapeType::SingleValue => self.render_single(),
            ShapeType::Boolean => self.render_boolean(),
            ShapeType::List => self.render_list(),
            ShapeType::KeyValue => self.render_key_value(),
            ShapeType::FreeForm => self.render_free_form(),
        }
    }

    fn render_single(&self) -> String {
        if let Some(field) = self.fields.first() {
            field.value.format_with_units(self.shape.units)
        } else {
            "No data available.".to_string()
        }
    }

    fn render_boolean(&self) -> String {
        if let Some(field) = self.fields.first() {
            match &field.value {
                AnswerValue::Boolean(b) => if *b { "Yes." } else { "No." }.to_string(),
                AnswerValue::String(s) => s.clone(),
                _ => "Unknown.".to_string(),
            }
        } else {
            "Unable to determine.".to_string()
        }
    }

    fn render_list(&self) -> String {
        if self.fields.is_empty() {
            return "None found.".to_string();
        }

        self.fields
            .iter()
            .map(|f| format!("- {}", f.value.format_with_units(self.shape.units)))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn render_key_value(&self) -> String {
        if self.fields.is_empty() {
            return "No data available.".to_string();
        }

        self.fields
            .iter()
            .map(|f| {
                format!(
                    "{}: {}",
                    f.name,
                    f.value.format_with_units(self.shape.units)
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn render_free_form(&self) -> String {
        // For diagnosis/explanation, join all fields
        self.fields
            .iter()
            .map(|f| f.value.format_with_units(self.shape.units))
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

/// Item that was discarded from the answer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscardedItem {
    /// Field name that was discarded.
    pub field_name: String,
    /// Why it was discarded.
    pub reason: DiscardReason,
}

/// Reason for discarding data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscardReason {
    /// Field not in allowed_fields.
    NotAllowed,
    /// Max items exceeded.
    MaxItemsExceeded,
    /// Wrong subject.
    WrongSubject,
    /// No evidence.
    NoEvidence,
}

/// Enforces answer shape strictly.
pub struct ShapeEnforcer;

impl ShapeEnforcer {
    /// Filter specialist output to match intent.
    pub fn enforce(intent: &QuestionIntent, raw_fields: Vec<AnswerField>) -> EnforcementResult {
        let mut plan = AnswerPlan::new(intent);

        for field in raw_fields {
            plan.add_field(field);
        }

        let is_valid = plan.is_complete() || !plan.fields.is_empty();

        EnforcementResult {
            plan,
            violations: Vec::new(),
            is_valid,
        }
    }

    /// Check if a raw answer violates constraints.
    pub fn check_violations(intent: &QuestionIntent, raw_text: &str) -> Vec<ShapeViolation> {
        let mut violations = Vec::new();

        // Check for tutorial leakage in fact/status questions
        if !intent.category.allows_tutorials() {
            if raw_text.contains("you can") || raw_text.contains("try running") {
                violations.push(ShapeViolation::TutorialLeakage);
            }
            if raw_text.contains("```") {
                violations.push(ShapeViolation::CommandLeakage);
            }
        }

        // Check for off-topic content
        let subject_keywords = get_subject_keywords(intent.subject);
        let has_subject_content = subject_keywords
            .iter()
            .any(|k| raw_text.to_lowercase().contains(k));

        if !has_subject_content && !intent.allows_extras() {
            violations.push(ShapeViolation::OffTopic);
        }

        violations
    }
}

/// Result of shape enforcement.
#[derive(Debug, Clone)]
pub struct EnforcementResult {
    /// The filtered answer plan.
    pub plan: AnswerPlan,
    /// Violations found.
    pub violations: Vec<ShapeViolation>,
    /// Whether the result is valid.
    pub is_valid: bool,
}

/// Type of shape violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeViolation {
    /// Tutorial text in fact/status answer.
    TutorialLeakage,
    /// Command suggestions in fact/status answer.
    CommandLeakage,
    /// Content about wrong subject.
    OffTopic,
    /// Too many items.
    TooManyItems,
    /// Missing required fields.
    MissingFields,
}

/// Get keywords for a subject.
fn get_subject_keywords(subject: super::intent::Subject) -> Vec<&'static str> {
    use super::intent::Subject;
    match subject {
        Subject::Memory => vec!["memory", "ram", "swap", "free", "available", "cached"],
        Subject::Cpu => vec!["cpu", "processor", "core", "thread", "frequency", "ghz"],
        Subject::Disk => vec![
            "disk",
            "storage",
            "filesystem",
            "mount",
            "partition",
            "gb",
            "tb",
        ],
        Subject::Service => vec!["service", "systemd", "unit", "running", "active", "failed"],
        Subject::Network => vec![
            "network",
            "interface",
            "ip",
            "ethernet",
            "wifi",
            "connection",
        ],
        Subject::Gpu => vec![
            "gpu", "graphics", "nvidia", "amd", "intel", "driver", "video",
        ],
        Subject::Boot => vec!["boot", "startup", "init", "kernel", "systemd-analyze"],
        Subject::Audio => vec!["audio", "sound", "pulseaudio", "pipewire", "alsa", "volume"],
        Subject::Packages => vec!["package", "installed", "pacman", "apt", "dnf", "version"],
        _ => vec![],
    }
}

/// Format bytes to human readable.
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Format duration in seconds to human readable.
fn format_duration(seconds: f64) -> String {
    if seconds >= 60.0 {
        let mins = (seconds / 60.0).floor();
        let secs = seconds % 60.0;
        format!("{}m {:.1}s", mins as u64, secs)
    } else {
        format!("{:.2}s", seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::question_contract::intent::{IntentBuilder, IntentCategory, Scope, Subject};

    #[test]
    fn test_answer_plan_filters_disallowed_fields() {
        let intent = IntentBuilder::new("int_001")
            .category(IntentCategory::Fact)
            .subject(Subject::Memory)
            .scope(Scope::Single)
            .allow_fields(vec!["free"])
            .build();

        let mut plan = AnswerPlan::new(&intent);

        // Add allowed field
        plan.add_field(AnswerField::new(
            "free",
            AnswerValue::String("4.2 GB".to_string()),
        ));

        // Try to add disallowed field
        plan.add_field(AnswerField::new(
            "total",
            AnswerValue::String("16 GB".to_string()),
        ));
        plan.add_field(AnswerField::new(
            "cached",
            AnswerValue::String("2 GB".to_string()),
        ));

        // Only allowed field should be present
        assert_eq!(plan.fields.len(), 1);
        assert_eq!(plan.fields[0].name, "free");

        // Disallowed should be tracked
        assert_eq!(plan.discarded.len(), 2);
    }

    #[test]
    fn test_max_items_enforced() {
        let intent = IntentBuilder::new("int_002")
            .scope(Scope::List)
            .constraints(super::super::intent::AnswerConstraints::list("service", 3))
            .build();

        let mut plan = AnswerPlan::new(&intent);

        // Add more than max items
        for i in 0..5 {
            plan.add_field(AnswerField::new(
                "service",
                AnswerValue::String(format!("service_{}", i)),
            ));
        }

        // Only 3 should be kept
        assert_eq!(plan.fields.len(), 3);
        assert_eq!(plan.discarded.len(), 2);
    }

    #[test]
    fn test_shape_enforcer_detects_tutorial_leakage() {
        let intent = IntentBuilder::new("int_003")
            .category(IntentCategory::Fact)
            .build();

        let raw_text = "You have 4GB free RAM. You can try running free -h for more details.";
        let violations = ShapeEnforcer::check_violations(&intent, raw_text);

        assert!(violations.contains(&ShapeViolation::TutorialLeakage));
    }

    #[test]
    fn test_diagnosis_allows_extras() {
        let intent = IntentBuilder::new("int_004")
            .category(IntentCategory::Diagnosis)
            .allow_extras()
            .build();

        let mut plan = AnswerPlan::new(&intent);

        // All fields should be allowed
        plan.add_field(AnswerField::new(
            "cause",
            AnswerValue::String("Slow disk".to_string()),
        ));
        plan.add_field(AnswerField::new(
            "evidence",
            AnswerValue::String("iostat".to_string()),
        ));
        plan.add_field(AnswerField::new(
            "suggestion",
            AnswerValue::String("Check SSD".to_string()),
        ));

        assert_eq!(plan.fields.len(), 3);
        assert!(plan.discarded.is_empty());
    }

    #[test]
    fn test_render_boolean() {
        let intent = IntentBuilder::new("int_005")
            .scope(Scope::Boolean)
            .allow_fields(vec!["result"])
            .build();

        let mut plan = AnswerPlan::new(&intent);
        plan.add_field(AnswerField::new("result", AnswerValue::Boolean(true)));

        assert_eq!(plan.render(), "Yes.");
    }

    #[test]
    fn test_render_list() {
        let intent = IntentBuilder::new("int_006")
            .scope(Scope::List)
            .constraints(super::super::intent::AnswerConstraints::list("item", 10))
            .build();

        let mut plan = AnswerPlan::new(&intent);
        plan.add_field(AnswerField::new(
            "item",
            AnswerValue::String("nginx.service".to_string()),
        ));
        plan.add_field(AnswerField::new(
            "item",
            AnswerValue::String("apache.service".to_string()),
        ));

        let rendered = plan.render();
        assert!(rendered.contains("nginx.service"));
        assert!(rendered.contains("apache.service"));
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(15.5), "15.50s");
        assert_eq!(format_duration(75.3), "1m 15.3s");
    }
}
