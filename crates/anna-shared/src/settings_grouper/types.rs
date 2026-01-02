// v0.0.676: Settings Grouper - Types (Phase 252)
// Core types and enums

use serde::{Deserialize, Serialize};

/// Group by field
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum GroupByField {
    /// Group by key prefix
    #[default]
    KeyPrefix,
    /// Group by key suffix
    KeySuffix,
    /// Group by value
    Value,
    /// Group by value type
    ValueType,
}

impl std::fmt::Display for GroupByField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::KeyPrefix => write!(f, "key_prefix"),
            Self::KeySuffix => write!(f, "key_suffix"),
            Self::Value => write!(f, "value"),
            Self::ValueType => write!(f, "value_type"),
        }
    }
}

/// Value type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ValueTypeClass {
    /// String type
    #[default]
    String,
    /// Number type
    Number,
    /// Boolean type
    Boolean,
    /// Empty type
    Empty,
}

impl std::fmt::Display for ValueTypeClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String => write!(f, "string"),
            Self::Number => write!(f, "number"),
            Self::Boolean => write!(f, "boolean"),
            Self::Empty => write!(f, "empty"),
        }
    }
}

/// Classify value type
pub fn classify_value(value: &str) -> ValueTypeClass {
    if value.is_empty() {
        ValueTypeClass::Empty
    } else if value == "true" || value == "false" {
        ValueTypeClass::Boolean
    } else if value.parse::<f64>().is_ok() {
        ValueTypeClass::Number
    } else {
        ValueTypeClass::String
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_by_field_display() {
        assert_eq!(format!("{}", GroupByField::KeyPrefix), "key_prefix");
        assert_eq!(format!("{}", GroupByField::Value), "value");
    }

    #[test]
    fn test_value_type_class_display() {
        assert_eq!(format!("{}", ValueTypeClass::String), "string");
        assert_eq!(format!("{}", ValueTypeClass::Number), "number");
    }

    #[test]
    fn test_classify_value() {
        assert_eq!(classify_value("hello"), ValueTypeClass::String);
        assert_eq!(classify_value("123"), ValueTypeClass::Number);
        assert_eq!(classify_value("true"), ValueTypeClass::Boolean);
        assert_eq!(classify_value(""), ValueTypeClass::Empty);
    }
}
