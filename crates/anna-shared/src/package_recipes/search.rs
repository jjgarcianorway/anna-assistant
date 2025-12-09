//! Package recipe search functions (v0.0.230).

use super::catalog::common_packages;
use super::types::{PackageManager, PackageRecipe};

/// Find a package recipe by name
pub fn find_recipe(name: &str) -> Option<PackageRecipe> {
    let name_lower = name.to_lowercase();
    common_packages()
        .into_iter()
        .find(|p| p.name == name_lower || p.display_name.to_lowercase() == name_lower)
}

/// Generate confirmation prompt for package install
pub fn confirmation_prompt(recipe: &PackageRecipe, manager: &PackageManager) -> String {
    let cmd = recipe.install_command(manager).unwrap_or_default();
    format!(
        "Install {}?\n\
         Description: {}\n\
         Command: sudo {}\n\
         \n\
         Proceed? [y/N]",
        recipe.display_name, recipe.description, cmd
    )
}
