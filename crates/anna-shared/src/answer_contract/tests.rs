//! Tests for answer contract module (v0.0.209).

#[cfg(test)]
mod tests {
    use crate::answer_contract::{
        trim_answer, validate_answer, AnswerContract, RequestedField, Verbosity,
    };

    #[test]
    fn test_contract_from_query_cores() {
        let contract = AnswerContract::from_query("how many cores does my cpu have?");
        assert!(contract
            .requested_fields
            .contains(&RequestedField::CpuCores));
        assert!(!contract
            .requested_fields
            .contains(&RequestedField::CpuModel));
    }

    #[test]
    fn test_contract_from_query_free_ram() {
        let contract = AnswerContract::from_query("how much free ram do I have?");
        assert!(contract.requested_fields.contains(&RequestedField::RamFree));
    }

    #[test]
    fn test_contract_minimal_verbosity() {
        let contract = AnswerContract::from_query("just tell me how many cores");
        assert_eq!(contract.verbosity, Verbosity::Minimal);
    }

    #[test]
    fn test_contract_teach_verbosity() {
        let contract = AnswerContract::from_query("explain how many cores I have");
        assert_eq!(contract.verbosity, Verbosity::Teach);
        assert!(contract.teaching_mode);
    }

    #[test]
    fn test_validate_answer_with_required_field() {
        let contract = AnswerContract::from_query("how many cores?");
        let validation = validate_answer("You have 8 cores and 16 threads.", &contract);
        assert!(validation.valid);
        assert!(validation.missing_fields.is_empty());
    }

    #[test]
    fn test_validate_answer_missing_field() {
        let contract = AnswerContract::from_query("what is my cpu temperature?");
        let validation = validate_answer("You have an Intel processor.", &contract);
        assert!(!validation.valid);
        assert!(!validation.missing_fields.is_empty());
    }

    #[test]
    fn test_trim_answer_cores() {
        let contract = AnswerContract {
            requested_fields: vec![RequestedField::CpuCores],
            verbosity: Verbosity::Minimal,
            teaching_mode: false,
            original_query: "just cores".to_string(),
        };
        let trimmed = trim_answer("Your CPU has 8 cores and is an Intel i7.", &contract);
        assert!(trimmed.is_some());
        assert!(trimmed.unwrap().contains("8"));
    }
}
