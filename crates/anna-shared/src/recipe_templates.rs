//! Generic Recipe Templates (v0.0.412).
//!
//! Pre-built parameterized recipes for common patterns.
//! These minimize hardcoding by using {{variable}} substitution.

use crate::recipe_engine::{
    EvidenceRequirement, Recipe, RecipeKind, RecipeParameter, RecipeStep, RiskLevel,
};

/// Build generic "Check systemd service status" recipe
pub fn service_status_recipe() -> Recipe {
    Recipe::new(
        "generic-service-status",
        "Check Systemd Service Status",
        RecipeKind::Inspect,
        "services",
    )
    .with_intent("check status of a systemd service")
    .with_tags(vec!["systemd", "service", "status", "unit", "failed"])
    .with_triggers(vec![
        "service status",
        "is service running",
        "why is service failing",
        "check service",
        "debug systemd",
        "service not starting",
    ])
    .with_evidence(vec![EvidenceRequirement::SystemdFailed])
    .with_params(vec![RecipeParameter {
        name: "service_name".to_string(),
        description: "Name of the systemd service (e.g., nginx, sshd)".to_string(),
        extraction_hint: "word before 'service' or known service name".to_string(),
        default: None,
        required: true,
    }])
    .with_steps(vec![
        RecipeStep::command(
            "s1",
            "systemctl status {{service_name}} --no-pager",
            "Get service status",
        ),
        RecipeStep::command(
            "s2",
            "journalctl -u {{service_name}} -n 30 --no-pager",
            "Get recent logs",
        )
        .depends("s1"),
        RecipeStep::render("s3", SERVICE_STATUS_TEMPLATE, "Render answer").depends("s2"),
    ])
    .with_docs(vec![
        "man:systemctl",
        "man:journalctl",
        "Arch Wiki: Systemd",
    ])
}

const SERVICE_STATUS_TEMPLATE: &str = r#"**Service Status: {{service_name}}**

{{s1.stdout}}

**Recent Logs:**
```
{{s2.stdout}}
```

If the service is failing, check the logs above for error messages.
Common fixes:
- Configuration errors: Check /etc/systemd/system/{{service_name}}.service
- Missing dependencies: `systemctl list-dependencies {{service_name}}`
- Restart attempt: `sudo systemctl restart {{service_name}}`
"#;

/// Build generic "Check disk usage" recipe
pub fn disk_usage_recipe() -> Recipe {
    Recipe::new(
        "generic-disk-usage",
        "Check Disk Usage",
        RecipeKind::Inspect,
        "storage",
    )
    .with_intent("check disk space and usage")
    .with_tags(vec!["disk", "space", "storage", "df", "usage", "full"])
    .with_triggers(vec![
        "disk usage",
        "disk space",
        "how much space",
        "disk full",
        "storage left",
        "out of space",
    ])
    .with_evidence(vec![EvidenceRequirement::DfRoot])
    .with_params(vec![RecipeParameter {
        name: "mount_path".to_string(),
        description: "Mount point to check (default: /)".to_string(),
        extraction_hint: "path mentioned, or / for root".to_string(),
        default: Some("/".to_string()),
        required: false,
    }])
    .with_steps(vec![
        RecipeStep::probe("s1", "disk_usage", "Get disk usage"),
        RecipeStep::command(
            "s2",
            "du -sh {{mount_path}}/* 2>/dev/null | sort -rh | head -10",
            "Top space users",
        )
        .depends("s1"),
        RecipeStep::render("s3", DISK_USAGE_TEMPLATE, "Render answer").depends("s2"),
    ])
    .with_docs(vec!["man:df", "man:du", "Arch Wiki: Disk"])
}

const DISK_USAGE_TEMPLATE: &str = r#"**Disk Usage**

{{s1.stdout}}

**Largest directories on {{mount_path}}:**
```
{{s2.stdout}}
```

If space is low:
- Clear package cache: `sudo pacman -Sc`
- Find large files: `find {{mount_path}} -size +100M -type f`
- Check journal size: `journalctl --disk-usage`
"#;

/// Build generic "Check memory usage" recipe
pub fn memory_usage_recipe() -> Recipe {
    Recipe::new(
        "generic-memory-usage",
        "Check Memory Usage",
        RecipeKind::Inspect,
        "system",
    )
    .with_intent("check RAM and swap usage")
    .with_tags(vec!["memory", "ram", "swap", "free", "usage"])
    .with_triggers(vec![
        "memory usage",
        "how much ram",
        "free memory",
        "memory left",
        "swap usage",
        "out of memory",
    ])
    .with_evidence(vec![
        EvidenceRequirement::Meminfo,
        EvidenceRequirement::Swaps,
    ])
    .with_steps(vec![
        RecipeStep::probe("s1", "memory_info", "Get memory info"),
        RecipeStep::command(
            "s2",
            "ps aux --sort=-%mem | head -10",
            "Top memory processes",
        )
        .depends("s1"),
        RecipeStep::render("s3", MEMORY_USAGE_TEMPLATE, "Render answer").depends("s2"),
    ])
    .with_docs(vec!["man:free", "man:ps", "Arch Wiki: Swap"])
}

const MEMORY_USAGE_TEMPLATE: &str = r#"**Memory Usage**

```
{{s1.stdout}}
```

**Top memory consumers:**
```
{{s2.stdout}}
```

If memory is low:
- Close unused applications
- Check for memory leaks in top consumers
- Consider adding swap if not present
"#;

/// Build generic "Check if package installed" recipe
pub fn package_check_recipe() -> Recipe {
    Recipe::new(
        "generic-package-check",
        "Check Package Installation",
        RecipeKind::Inspect,
        "packages",
    )
    .with_intent("check if a package is installed and its version")
    .with_tags(vec!["package", "installed", "version", "pacman"])
    .with_triggers(vec![
        "is installed",
        "package installed",
        "what version",
        "check package",
    ])
    .with_params(vec![RecipeParameter {
        name: "package_name".to_string(),
        description: "Name of the package to check".to_string(),
        extraction_hint: "package name from query".to_string(),
        default: None,
        required: true,
    }])
    .with_steps(vec![
        RecipeStep::command(
            "s1",
            "pacman -Q {{package_name}} 2>/dev/null || echo 'Not installed'",
            "Check package",
        ),
        RecipeStep::command(
            "s2",
            "pacman -Qi {{package_name}} 2>/dev/null | head -15",
            "Package info",
        )
        .depends("s1"),
        RecipeStep::render("s3", PACKAGE_CHECK_TEMPLATE, "Render answer").depends("s2"),
    ])
    .with_docs(vec!["man:pacman", "Arch Wiki: Pacman"])
}

const PACKAGE_CHECK_TEMPLATE: &str = r#"**Package: {{package_name}}**

{{s1.stdout}}

{{s2.stdout}}

To install: `sudo pacman -S {{package_name}}`
To remove: `sudo pacman -R {{package_name}}`
"#;

/// Build generic "Check failed systemd units" recipe
pub fn failed_units_recipe() -> Recipe {
    Recipe::new(
        "generic-failed-units",
        "Check Failed Systemd Units",
        RecipeKind::Diagnose,
        "services",
    )
    .with_intent("find and diagnose failed systemd services")
    .with_tags(vec!["systemd", "failed", "units", "services", "boot"])
    .with_triggers(vec![
        "failed services",
        "failed units",
        "systemd failed",
        "what failed",
        "boot errors",
    ])
    .with_evidence(vec![EvidenceRequirement::SystemdFailed])
    .with_steps(vec![
        RecipeStep::probe("s1", "systemd_failed", "Get failed units"),
        RecipeStep::render("s2", FAILED_UNITS_TEMPLATE, "Render answer").depends("s1"),
    ])
    .with_docs(vec!["man:systemctl", "Arch Wiki: Systemd"])
}

const FAILED_UNITS_TEMPLATE: &str = r#"**Failed Systemd Units**

```
{{s1.stdout}}
```

To investigate a failed unit:
- `systemctl status <unit-name>`
- `journalctl -u <unit-name>`

To reset failed state: `sudo systemctl reset-failed`
"#;

/// Build generic "Check network interfaces" recipe
pub fn network_interfaces_recipe() -> Recipe {
    Recipe::new(
        "generic-network-interfaces",
        "Check Network Interfaces",
        RecipeKind::Inspect,
        "network",
    )
    .with_intent("show network interfaces and their status")
    .with_tags(vec!["network", "interfaces", "ip", "ethernet", "wifi"])
    .with_triggers(vec![
        "network interfaces",
        "ip address",
        "network status",
        "ethernet",
        "wifi status",
    ])
    .with_evidence(vec![EvidenceRequirement::NetworkInterfaces])
    .with_steps(vec![
        RecipeStep::probe("s1", "network_interfaces", "Get interfaces"),
        RecipeStep::command("s2", "ip route | head -5", "Get routes").depends("s1"),
        RecipeStep::render("s3", NETWORK_TEMPLATE, "Render answer").depends("s2"),
    ])
    .with_docs(vec!["man:ip", "Arch Wiki: Network configuration"])
}

const NETWORK_TEMPLATE: &str = r#"**Network Interfaces**

```
{{s1.stdout}}
```

**Routing:**
```
{{s2.stdout}}
```

To bring interface up: `sudo ip link set <interface> up`
To get DHCP: `sudo dhclient <interface>`
"#;

/// Get all generic recipe templates
pub fn all_templates() -> Vec<Recipe> {
    vec![
        service_status_recipe(),
        disk_usage_recipe(),
        memory_usage_recipe(),
        package_check_recipe(),
        failed_units_recipe(),
        network_interfaces_recipe(),
    ]
}

/// Initialize recipe store with generic templates
pub fn initialize_store(store: &mut crate::recipe_store_v2::RecipeStoreV2) {
    for template in all_templates() {
        if store.get(&template.id).is_none() {
            store.add(template);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_status_recipe() {
        let recipe = service_status_recipe();
        assert_eq!(recipe.id, "generic-service-status");
        assert!(!recipe.steps.is_empty());
        assert!(recipe.required_params().contains(&"service_name"));
    }

    #[test]
    fn test_all_templates_valid() {
        for recipe in all_templates() {
            assert!(!recipe.id.is_empty());
            assert!(!recipe.steps.is_empty());
            assert!(!recipe.trigger_patterns.is_empty());
        }
    }
}
