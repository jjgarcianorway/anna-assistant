//! Confirmation prompt generation (v0.0.214).

use super::recipe::ServiceRecipe;
use super::types::{ServiceAction, ServiceRisk};

/// Generate confirmation prompt for service action
pub fn confirmation_prompt(recipe: &ServiceRecipe, action: ServiceAction) -> String {
    let mut prompt = format!(
        "{} {}?\n\
         Service: {}\n\
         Description: {}\n",
        action
            .display_name()
            .chars()
            .next()
            .unwrap()
            .to_uppercase()
            .collect::<String>()
            + &action.display_name()[1..],
        recipe.display_name,
        recipe.name,
        recipe.description
    );

    // Add risk warning if needed
    match recipe.risk {
        ServiceRisk::High => {
            prompt.push_str("\nWARNING: This is a critical service. ");
            prompt.push_str("Modifying it may affect system stability.\n");
        }
        ServiceRisk::Protected => {
            prompt.push_str("\nERROR: This service is protected and cannot be modified.\n");
            return prompt;
        }
        _ => {}
    }

    // Add rollback info
    if let Some(rollback) = recipe.rollback_command(action) {
        prompt.push_str(&format!("To undo: sudo {}\n", rollback));
    }

    prompt.push_str("\nProceed? [y/N]");
    prompt
}
