//! Tests for recipe executor.

use std::collections::HashMap;

use crate::recipe_v3::{RecipeDomain, RecipeMatcher, RecipeStep, RecipeV3};

use super::executor_core::RecipeExecutor;
use super::executor_helpers::create_execution_plan;
use super::executor_types::ExecutionResult;

fn test_recipe() -> RecipeV3 {
    RecipeV3::new("test-exec", "Test Execution")
        .with_matcher(RecipeMatcher::new(RecipeDomain::General))
        .with_step(RecipeStep::Explain {
            text: "This is a test".to_string(),
            citation: None,
        })
        .with_step(RecipeStep::RunProbe {
            probe: "echo 'hello'".to_string(),
            output_var: "greeting".to_string(),
            description: "Get greeting".to_string(),
        })
}

#[test]
fn test_dry_run() {
    let recipe = test_recipe();
    let executor = RecipeExecutor::new().dry_run();

    let result = executor.execute_recipe(&recipe, HashMap::new());
    assert!(result.success);
    assert!(result.message.contains("DRY RUN"));
}

#[test]
fn test_execution_with_variables() {
    let recipe = RecipeV3::new("var-test", "Variable Test").with_step(RecipeStep::Explain {
        text: "Hello ${name}!".to_string(),
        citation: None,
    });

    let executor = RecipeExecutor::new().with_var("name", "World");
    let result = executor.execute_recipe(&recipe, HashMap::new());

    assert!(result.success);
}

#[test]
fn test_execution_plan() {
    let recipe = test_recipe();
    let plan = create_execution_plan(&recipe, &HashMap::new());

    assert_eq!(plan.total_steps, 2);
    use crate::recipe_v3::RecipeRiskLevel;
    assert_eq!(plan.steps[0].risk_level, RecipeRiskLevel::None);
}

#[test]
fn test_step_execution_order() {
    let recipe = RecipeV3::new("order-test", "Order Test")
        .with_step(RecipeStep::RunProbe {
            probe: "echo first".to_string(),
            output_var: "v1".to_string(),
            description: "First".to_string(),
        })
        .with_step(RecipeStep::RunProbe {
            probe: "echo second".to_string(),
            output_var: "v2".to_string(),
            description: "Second".to_string(),
        });

    let executor = RecipeExecutor::new();
    let result = executor.execute_recipe(&recipe, HashMap::new());

    assert!(result.success);
    assert_eq!(result.steps_executed.len(), 2);
    assert!(result.variables.contains_key("v1"));
    assert!(result.variables.contains_key("v2"));
}
