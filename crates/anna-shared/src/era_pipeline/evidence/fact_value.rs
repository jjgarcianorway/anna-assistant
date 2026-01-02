//! FactValue - Typed fact values for evidence collection.

use serde::{Deserialize, Serialize};

/// Typed fact value.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum FactValue {
    /// Numeric value (integer or float).
    Number(f64),
    /// String value.
    String(String),
    /// Boolean value.
    Bool(bool),
    /// List of strings.
    List(Vec<String>),
    /// Null/missing.
    Null,
}

impl FactValue {
    /// Create a numeric fact.
    pub fn number(n: f64) -> Self {
        Self::Number(n)
    }

    /// Create a string fact.
    pub fn string(s: &str) -> Self {
        Self::String(s.to_string())
    }

    /// Create a boolean fact.
    pub fn bool(b: bool) -> Self {
        Self::Bool(b)
    }

    /// Create a list fact.
    pub fn list(items: Vec<String>) -> Self {
        Self::List(items)
    }

    /// Check if null.
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Get as number.
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Self::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Get as string.
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get as bool.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Get as list.
    pub fn as_list(&self) -> Option<&[String]> {
        match self {
            Self::List(l) => Some(l),
            _ => None,
        }
    }

    /// Format for display.
    pub fn display(&self) -> String {
        match self {
            Self::Number(n) => {
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    format!("{:.1}", n)
                }
            }
            Self::String(s) => s.clone(),
            Self::Bool(b) => if *b { "Yes" } else { "No" }.to_string(),
            Self::List(l) => l.join(", "),
            Self::Null => "N/A".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fact_value_types() {
        let num = FactValue::number(17.5);
        assert_eq!(num.as_number(), Some(17.5));
        assert_eq!(num.display(), "17.5");

        let s = FactValue::string("hello");
        assert_eq!(s.as_string(), Some("hello"));

        let b = FactValue::bool(true);
        assert_eq!(b.as_bool(), Some(true));
        assert_eq!(b.display(), "Yes");

        let list = FactValue::list(vec!["a".to_string(), "b".to_string()]);
        assert_eq!(
            list.as_list(),
            Some(vec!["a".to_string(), "b".to_string()].as_slice())
        );
    }
}
