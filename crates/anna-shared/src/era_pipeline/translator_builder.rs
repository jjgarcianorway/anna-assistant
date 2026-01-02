//! Direct answer builder for deterministic cases (Part D3) - v0.0.441.

use super::evidence::EvidenceBundle;
use super::pipeline::AnswerType;
use super::translator_helpers::infer_unit;
use super::translator_types::MAX_LIST_ITEMS;

/// Direct answer builder for deterministic cases.
pub struct DirectAnswerBuilder;

impl DirectAnswerBuilder {
    /// Build answer directly from evidence (no reasoning needed).
    pub fn build(
        fact_name: &str,
        evidence: &EvidenceBundle,
        answer_type: AnswerType,
    ) -> Option<String> {
        let value = evidence.get(fact_name)?;

        match answer_type {
            AnswerType::Numeric => {
                let n = value.as_number()?;
                let unit = infer_unit(fact_name);
                if n.fract() == 0.0 {
                    Some(format!("{}{}", n as i64, unit))
                } else {
                    Some(format!("{:.1}{}", n, unit))
                }
            }
            AnswerType::Boolean => {
                let b = value.as_bool()?;
                Some(if b {
                    "Yes.".to_string()
                } else {
                    "No.".to_string()
                })
            }
            AnswerType::List => {
                let list = value.as_list()?;
                Some(
                    list.iter()
                        .take(MAX_LIST_ITEMS)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", "),
                )
            }
            AnswerType::Entity => Some(value.as_string()?.to_string()),
            AnswerType::Brief => Some(value.display()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::era_pipeline::evidence::EvidenceBundleBuilder;

    #[test]
    fn test_direct_answer_builder() {
        let evidence = EvidenceBundleBuilder::new("DSK-0127")
            .fact_number("memory.free_gib", 17.5)
            .build();

        let answer = DirectAnswerBuilder::build("memory.free_gib", &evidence, AnswerType::Numeric);
        assert_eq!(answer, Some("17.5 GiB".to_string()));
    }
}
