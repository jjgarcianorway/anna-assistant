//! AnswerPlan and Shape Enforcement (Part B) - v0.0.437.
//!
//! Before rendering a final answer:
//! - Anna builds an AnswerPlan from QuestionIntent
//! - Any data not mapped to allowed_fields is DISCARDED
//! - Any specialist output violating constraints is IGNORED or TRUNCATED
//!
//! This rule is ABSOLUTE.

use super::answer_field::{AnswerField, AnswerValue};
use super::answer_shape::{AnswerShape, ShapeType};
use super::intent::QuestionIntent;
use serde::{Deserialize, Serialize};

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
}
