//! Seed recipes shipped with Anna (v0.0.420).
//!
//! These recipes prove the model and shape, providing built-in functionality
//! for common tasks without requiring learning.

use super::{
    FactRequirement, RecipeDomain, RecipeStepAction, RecipeStepV2, RecipeV2, TriggerPattern,
};

/// Get all seed recipes
pub fn get_seed_recipes() -> Vec<RecipeV2> {
    vec![
        vim_syntax_enable(),
        system_boot_check(),
        memory_show_free(),
        network_show_interfaces(),
        disk_usage_show(),
        swap_check(),
        services_failed_check(),
        uptime_show(),
    ]
}

/// All seed recipes as a static slice (for compile-time reference)
pub static SEED_RECIPES: &[&str] = &[
    "vim.syntax.enable",
    "system.boot.check",
    "memory.show_free",
    "network.show_interfaces",
    "disk.usage.show",
    "swap.check",
    "services.failed.check",
    "uptime.show",
];

// =============================================================================
// Recipe 1: vim.syntax.enable
// =============================================================================

fn vim_syntax_enable() -> RecipeV2 {
    RecipeV2::new("vim.syntax.enable", "Enable Vim Syntax Highlighting", RecipeDomain::Desktop)
        .with_trigger(
            TriggerPattern::new("enable_vim_syntax", vec!["vim", "syntax", "highlight", "colors"])
                .with_min_confidence(0.7),
        )
        .with_trigger(
            TriggerPattern::new("vim_syntax_highlighting", vec!["vim", "syntax", "enable"])
                .with_min_confidence(0.7),
        )
        .with_fact(FactRequirement::eq("editor", "vim"))
        .with_step(RecipeStepV2::safe_change(
            "Backup ~/.vimrc",
            RecipeStepAction::backup_file("~/.vimrc"),
        ))
        .with_step(RecipeStepV2::safe_change(
            "Add syntax enable to ~/.vimrc",
            RecipeStepAction::ensure_line("~/.vimrc", "syntax enable"),
        ))
        .with_step(RecipeStepV2::explain(
            "Explain the change",
            "Syntax highlighting is now enabled in Vim. The setting `syntax enable` has been added to your ~/.vimrc file. This will colorize code based on file type.",
        ))
        .with_citation("archwiki:Vim#Syntax_highlighting")
        .with_citation("man:vim(1)")
        .global()
}

// =============================================================================
// Recipe 2: system.boot.check
// =============================================================================

fn system_boot_check() -> RecipeV2 {
    RecipeV2::new("system.boot.check", "Check Boot Time", RecipeDomain::Performance)
        .with_trigger(
            TriggerPattern::new("check_boot_slow", vec!["boot", "slow", "startup", "time"])
                .with_min_confidence(0.7),
        )
        .with_trigger(
            TriggerPattern::new("boot_time", vec!["boot", "time", "analyze"])
                .with_min_confidence(0.7),
        )
        .with_step(RecipeStepV2::probe(
            "Check overall boot time",
            "systemd-analyze",
        ))
        .with_step(RecipeStepV2::probe(
            "List slow services",
            "systemd-analyze blame | head -20",
        ))
        .with_step(RecipeStepV2::explain_with_extract(
            "Show boot analysis",
            "Boot Analysis:\n\nTotal boot time: {boot_time}\n\nSlowest services:\n{slow_services}\n\nTo investigate a slow service, run: systemctl status <service>",
            "boot_time,slow_services",
        ))
        .with_citation("man:systemd-analyze(1)")
        .global()
}

// =============================================================================
// Recipe 3: memory.show_free
// =============================================================================

fn memory_show_free() -> RecipeV2 {
    RecipeV2::new("memory.show_free", "Show Free Memory", RecipeDomain::Performance)
        .with_trigger(
            TriggerPattern::new("show_free_memory", vec!["ram", "memory", "free", "usage"])
                .with_min_confidence(0.7),
        )
        .with_trigger(
            TriggerPattern::new("memory_usage", vec!["memory", "available", "used"])
                .with_min_confidence(0.7),
        )
        .with_step(RecipeStepV2::probe(
            "Get memory information",
            "free -h",
        ))
        .with_step(RecipeStepV2::explain_with_extract(
            "Show memory status",
            "Memory Status:\n\nAvailable: {available} (of {total} total)\nUsed: {used}\nBuffers/Cache: {buffers}\n\nNote: Linux uses free memory for disk caching, so 'available' is more relevant than 'free'.",
            "available,total,used,buffers",
        ))
        .with_citation("man:free(1)")
        .global()
}

// =============================================================================
// Recipe 4: network.show_interfaces
// =============================================================================

fn network_show_interfaces() -> RecipeV2 {
    RecipeV2::new("network.show_interfaces", "Show Network Interfaces", RecipeDomain::Network)
        .with_trigger(
            TriggerPattern::new("show_network_interfaces", vec!["network", "interfaces", "ip", "wifi"])
                .with_min_confidence(0.7),
        )
        .with_trigger(
            TriggerPattern::new("list_interfaces", vec!["interface", "ethernet", "connection"])
                .with_min_confidence(0.7),
        )
        .with_step(RecipeStepV2::probe(
            "List network interfaces",
            "ip -brief addr",
        ))
        .with_step(RecipeStepV2::explain_with_extract(
            "Show interfaces",
            "Network Interfaces:\n\n{interfaces}\n\nLegend: UP = active, DOWN = inactive, UNKNOWN = carrier detection unavailable",
            "interfaces",
        ))
        .with_citation("man:ip(8)")
        .global()
}

// =============================================================================
// Recipe 5: disk.usage.show
// =============================================================================

fn disk_usage_show() -> RecipeV2 {
    RecipeV2::new("disk.usage.show", "Show Disk Usage", RecipeDomain::Storage)
        .with_trigger(
            TriggerPattern::new("show_disk_usage", vec!["disk", "space", "usage", "storage"])
                .with_min_confidence(0.7),
        )
        .with_trigger(
            TriggerPattern::new("disk_space", vec!["disk", "full", "free", "space"])
                .with_min_confidence(0.7),
        )
        .with_step(RecipeStepV2::probe(
            "Get disk usage",
            "df -h --output=source,size,used,avail,pcent,target -x tmpfs -x devtmpfs | head -10",
        ))
        .with_step(RecipeStepV2::explain_with_extract(
            "Show disk usage",
            "Disk Usage:\n\n{disk_info}\n\nNote: tmpfs and devtmpfs are virtual filesystems and excluded.",
            "disk_info",
        ))
        .with_citation("man:df(1)")
        .global()
}

// =============================================================================
// Recipe 6: swap.check
// =============================================================================

fn swap_check() -> RecipeV2 {
    RecipeV2::new("swap.check", "Check Swap Status", RecipeDomain::Performance)
        .with_trigger(
            TriggerPattern::new("check_swap", vec!["swap", "memory", "virtual"])
                .with_min_confidence(0.7),
        )
        .with_trigger(
            TriggerPattern::new("swap_status", vec!["swap", "enabled", "active"])
                .with_min_confidence(0.7),
        )
        .with_step(RecipeStepV2::probe(
            "Get swap information",
            "swapon --show 2>/dev/null || echo 'No swap configured'",
        ))
        .with_step(RecipeStepV2::probe(
            "Get memory overview including swap",
            "free -h | grep -E '(Mem|Swap)'",
        ))
        .with_step(RecipeStepV2::explain_with_extract(
            "Show swap status",
            "Swap Status:\n\n{swap_info}\n\nSwap provides overflow memory when RAM is full. Modern systems with plenty of RAM may not need swap, but it's useful for hibernation and handling memory spikes.",
            "swap_info",
        ))
        .with_citation("man:swapon(8)")
        .with_citation("man:free(1)")
        .global()
}

// =============================================================================
// Recipe 7: services.failed.check
// =============================================================================

fn services_failed_check() -> RecipeV2 {
    RecipeV2::new("services.failed.check", "Check Failed Services", RecipeDomain::Services)
        .with_trigger(
            TriggerPattern::new("check_failed_services", vec!["failed", "service", "systemd", "error"])
                .with_min_confidence(0.7),
        )
        .with_trigger(
            TriggerPattern::new("service_errors", vec!["service", "failed", "broken"])
                .with_min_confidence(0.7),
        )
        .with_step(RecipeStepV2::probe(
            "List failed units",
            "systemctl --failed --no-pager",
        ))
        .with_step(RecipeStepV2::explain_with_extract(
            "Show failed services",
            "Service Status:\n\n{failed_services}\n\nTo investigate a failed service, run: systemctl status <service>\nTo see logs: journalctl -xeu <service>",
            "failed_services",
        ))
        .with_citation("man:systemctl(1)")
        .global()
}

// =============================================================================
// Recipe 8: uptime.show
// =============================================================================

fn uptime_show() -> RecipeV2 {
    RecipeV2::new("uptime.show", "Show System Uptime", RecipeDomain::Performance)
        .with_trigger(
            TriggerPattern::new("show_uptime", vec!["uptime", "running", "since"])
                .with_min_confidence(0.7),
        )
        .with_trigger(
            TriggerPattern::new("system_uptime", vec!["how", "long", "running", "uptime"])
                .with_min_confidence(0.7),
        )
        .with_step(RecipeStepV2::probe(
            "Get uptime",
            "uptime -p",
        ))
        .with_step(RecipeStepV2::probe(
            "Get load average",
            "uptime",
        ))
        .with_step(RecipeStepV2::explain_with_extract(
            "Show uptime",
            "System Uptime:\n\n{uptime}\n\nLoad average: {load_avg}\n\nLoad average shows system load over 1, 5, and 15 minutes. Values below the number of CPU cores indicate the system isn't overloaded.",
            "uptime,load_avg",
        ))
        .with_citation("man:uptime(1)")
        .global()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed_recipes_valid() {
        let recipes = get_seed_recipes();
        assert!(!recipes.is_empty());

        for recipe in &recipes {
            // All seed recipes should be global
            assert!(recipe.is_global);
            // All should have at least one trigger
            assert!(!recipe.trigger_patterns.is_empty());
            // All should have at least one step
            assert!(!recipe.steps.is_empty());
            // All should have at least one citation
            assert!(!recipe.citations.is_empty());
        }
    }

    #[test]
    fn test_vim_syntax_recipe() {
        let recipe = vim_syntax_enable();
        assert_eq!(recipe.id, "vim.syntax.enable");
        assert_eq!(recipe.domain, RecipeDomain::Desktop);
        assert!(recipe.needs_confirmation() == false); // SafeChange, not RiskyChange
    }

    #[test]
    fn test_memory_recipe() {
        let recipe = memory_show_free();
        assert_eq!(recipe.id, "memory.show_free");
        assert_eq!(recipe.domain, RecipeDomain::Performance);
        assert!(recipe.is_mutating() == false); // Probe only
    }
}
