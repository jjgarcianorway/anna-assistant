//! Canary Tests - Memory (Part 1 of 4) - v0.0.437.
//!
//! Fixed tests that MUST pass for any release.
//! Any regression here BLOCKS release.
//!
//! Test cases:
//! - "how much free ram do I have" → single numeric answer only
//! - "is zram enabled" → boolean only

#[cfg(test)]
mod memory_canary_tests {
    use crate::question_contract::answer_plan::*;
    use crate::question_contract::filters::*;
    use crate::question_contract::intent::*;
    use crate::question_contract::*;

    // ============================================
    // CANARY 1: "how much free ram do I have"
    // Expected: single numeric answer only
    // ============================================

    #[test]
    fn canary_free_ram_intent() {
        let intent = IntentBuilder::new("canary_ram")
            .category(IntentCategory::Fact)
            .subject(Subject::Memory)
            .scope(Scope::Single)
            .allow_fields(vec!["free"])
            .build();

        // Must be a fact about memory
        assert_eq!(intent.category, IntentCategory::Fact);
        assert_eq!(intent.subject, Subject::Memory);
        assert_eq!(intent.scope, Scope::Single);

        // Must NOT allow extras
        assert!(!intent.allows_extras());

        // Only "free" field allowed
        assert!(intent.is_field_allowed("free"));
        assert!(!intent.is_field_allowed("total"));
        assert!(!intent.is_field_allowed("cached"));
        assert!(!intent.is_field_allowed("cpu_model"));
    }

    #[test]
    fn canary_free_ram_shape_enforcement() {
        let intent = IntentBuilder::new("canary_ram")
            .category(IntentCategory::Fact)
            .subject(Subject::Memory)
            .scope(Scope::Single)
            .allow_fields(vec!["free"])
            .build();

        let raw_fields = vec![
            AnswerField::new("free", AnswerValue::String("4.2 GB".to_string())),
            AnswerField::new("total", AnswerValue::String("16 GB".to_string())), // Should be discarded
            AnswerField::new("cached", AnswerValue::String("8 GB".to_string())), // Should be discarded
        ];

        let result = ShapeEnforcer::enforce(&intent, raw_fields);

        // Only free should remain
        assert_eq!(result.plan.fields.len(), 1);
        assert_eq!(result.plan.fields[0].name, "free");

        // Others should be discarded
        assert_eq!(result.plan.discarded.len(), 2);
    }

    #[test]
    fn canary_free_ram_no_tutorials() {
        let intent = IntentBuilder::new("canary_ram")
            .category(IntentCategory::Fact)
            .build();

        // This answer has leakage
        let bad_answer = "You have 4.2 GB free. You can try running free -h for more details.";
        let result = AnswerFilter::filter(&intent, bad_answer);

        assert!(result.has_leakage(), "Tutorial leakage must be detected");
        assert!(result
            .leakages
            .iter()
            .any(|l| l.leakage_type == LeakageType::Tutorial));

        // Good answer has no leakage
        let good_answer = "4.2 GB free.";
        let result = AnswerFilter::filter(&intent, good_answer);
        assert!(!result.has_leakage(), "Clean answer should pass");
    }

    // ============================================
    // CANARY 2: "is zram enabled"
    // Expected: boolean only
    // ============================================

    #[test]
    fn canary_zram_enabled_intent() {
        let intent = IntentBuilder::new("canary_zram")
            .category(IntentCategory::Status)
            .subject(Subject::Memory)
            .scope(Scope::Boolean)
            .constraints(AnswerConstraints::boolean())
            .build();

        assert_eq!(intent.category, IntentCategory::Status);
        assert_eq!(intent.scope, Scope::Boolean);
        assert!(!intent.allows_extras());
    }

    #[test]
    fn canary_zram_enabled_shape() {
        let intent = IntentBuilder::new("canary_zram")
            .scope(Scope::Boolean)
            .constraints(AnswerConstraints::boolean())
            .build();

        let mut plan = AnswerPlan::new(&intent);
        plan.add_field(AnswerField::new("result", AnswerValue::Boolean(true)));

        let rendered = plan.render();
        assert_eq!(rendered, "Yes.");

        // False case
        let mut plan = AnswerPlan::new(&intent);
        plan.add_field(AnswerField::new("result", AnswerValue::Boolean(false)));
        assert_eq!(plan.render(), "No.");
    }

    #[test]
    fn canary_zram_no_debug_steps() {
        let intent = IntentBuilder::new("canary_zram")
            .category(IntentCategory::Status)
            .build();

        // Bad: includes debug instructions
        let bad_answer = "Yes, zram is enabled. To diagnose performance, check /sys/block/zram0/";
        let result = AnswerFilter::filter(&intent, bad_answer);
        assert!(result.has_leakage());

        // Good: just the answer
        let good_answer = "Yes.";
        let result = AnswerFilter::filter(&intent, good_answer);
        assert!(!result.has_leakage());
    }
}
