//! Seed recipes for common operations (v0.0.423).
//!
//! Provides pre-built recipes for common system tasks.

use super::{ConfirmationPolicy, RecipeCondition, RecipeDomain, RecipeRiskLevel, RecipeStep, RecipeV3, RecipeBuilder};

/// Seed recipes for common operations
pub fn seed_recipes() -> Vec<RecipeV3> {
    vec![
        // Service restart
        RecipeBuilder::new("restart-service")
            .title("Restart Service")
            .description("Restart a systemd service")
            .domain(RecipeDomain::Systemd)
            .intent("restart")
            .keyword("restart")
            .keyword("service")
            .keyword("systemctl")
            .similarity_key("restart service")
            .parameter("service", "Name of the service to restart")
            .precondition(RecipeCondition::CommandExists {
                command: "systemctl".to_string(),
            })
            .step(RecipeStep::RunCommand {
                command: "sudo systemctl restart ${service}".to_string(),
                description: "Restart the service".to_string(),
                risk: Some(RecipeRiskLevel::Medium),
                capture_output: true,
                output_var: None,
            })
            .step(RecipeStep::RunProbe {
                probe: "systemctl is-active ${service}".to_string(),
                output_var: "status".to_string(),
                description: "Check service status".to_string(),
            })
            .postcondition(RecipeCondition::ServiceState {
                service: "${service}".to_string(),
                state: "running".to_string(),
            })
            .risk(RecipeRiskLevel::Medium)
            .confirmation(ConfirmationPolicy::Once)
            .citation("man systemctl")
            .tag("systemd")
            .build()
            .unwrap(),
        // Service status
        RecipeBuilder::new("check-service-status")
            .title("Check Service Status")
            .description("Check if a service is running")
            .domain(RecipeDomain::Systemd)
            .intent("status")
            .intent("check")
            .keyword("status")
            .keyword("running")
            .keyword("service")
            .similarity_key("check service status")
            .parameter("service", "Name of the service to check")
            .step(RecipeStep::RunProbe {
                probe: "systemctl status ${service} --no-pager".to_string(),
                output_var: "status".to_string(),
                description: "Get service status".to_string(),
            })
            .risk(RecipeRiskLevel::None)
            .confirmation(ConfirmationPolicy::Never)
            .citation("man systemctl")
            .tag("systemd")
            .build()
            .unwrap(),
        // Package install
        RecipeBuilder::new("install-package")
            .title("Install Package")
            .description("Install a package with pacman")
            .domain(RecipeDomain::Package)
            .intent("install")
            .keyword("install")
            .keyword("package")
            .keyword("pacman")
            .similarity_key("install package")
            .parameter("package", "Name of the package to install")
            .precondition(RecipeCondition::CommandExists {
                command: "pacman".to_string(),
            })
            .step(RecipeStep::RunCommand {
                command: "sudo pacman -S --noconfirm ${package}".to_string(),
                description: "Install the package".to_string(),
                risk: Some(RecipeRiskLevel::Low),
                capture_output: true,
                output_var: None,
            })
            .postcondition(RecipeCondition::PackageInstalled {
                package: "${package}".to_string(),
            })
            .risk(RecipeRiskLevel::Low)
            .confirmation(ConfirmationPolicy::Once)
            .citation("man pacman")
            .tag("pacman")
            .build()
            .unwrap(),
        // Disk usage
        RecipeBuilder::new("check-disk-usage")
            .title("Check Disk Usage")
            .description("Show disk space usage")
            .domain(RecipeDomain::Disk)
            .intent("check")
            .intent("status")
            .keyword("disk")
            .keyword("space")
            .keyword("usage")
            .keyword("storage")
            .similarity_key("check disk usage")
            .step(RecipeStep::RunProbe {
                probe: "df -h".to_string(),
                output_var: "disk_usage".to_string(),
                description: "Get disk usage".to_string(),
            })
            .risk(RecipeRiskLevel::None)
            .confirmation(ConfirmationPolicy::Never)
            .citation("man df")
            .tag("disk")
            .build()
            .unwrap(),
        // Memory usage
        RecipeBuilder::new("check-memory-usage")
            .title("Check Memory Usage")
            .description("Show memory and swap usage")
            .domain(RecipeDomain::Memory)
            .intent("check")
            .intent("status")
            .keyword("memory")
            .keyword("ram")
            .keyword("swap")
            .keyword("usage")
            .similarity_key("check memory usage")
            .step(RecipeStep::RunProbe {
                probe: "free -h".to_string(),
                output_var: "memory".to_string(),
                description: "Get memory usage".to_string(),
            })
            .risk(RecipeRiskLevel::None)
            .confirmation(ConfirmationPolicy::Never)
            .citation("man free")
            .tag("memory")
            .build()
            .unwrap(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed_recipes() {
        let seeds = seed_recipes();
        assert!(!seeds.is_empty());

        // All should be valid
        for recipe in &seeds {
            assert!(!recipe.id.is_empty());
            assert!(!recipe.title.is_empty());
            assert!(!recipe.steps.is_empty());
        }
    }
}
