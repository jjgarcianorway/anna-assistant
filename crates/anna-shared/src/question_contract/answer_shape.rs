//! Answer shape definitions - v0.0.437.
//!
//! Defines the expected structure and constraints for answers.

use super::intent::{QuestionIntent, Scope, Units};
use serde::{Deserialize, Serialize};

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
