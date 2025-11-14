// Adaptive Help System
// Phase 3.1: Contextual Autonomy and Adaptive Simplicity
//
// Context-aware help that adapts to:
// - User experience level (beginner/intermediate/expert)
// - System state (healthy/degraded/iso_live)
// - Daemon availability
// - Resource constraints

use anyhow::Result;
use anna_common::command_meta::{
    CommandCategory, CommandMetadata, CommandRegistry, DisplayContext, UserLevel,
};
use owo_colors::OwoColorize;

/// Display adaptive help based on context
pub async fn display_help(
    command: Option<String>,
    show_all: bool,
    context: DisplayContext,
) -> Result<()> {
    let registry = CommandRegistry::new();

    if let Some(cmd_name) = command {
        // Show help for specific command
        display_command_help(&registry, &cmd_name)?;
    } else {
        // Show general help with context-aware filtering
        display_general_help(&registry, context, show_all)?;
    }

    Ok(())
}

/// Display help for a specific command
fn display_command_help(registry: &CommandRegistry, command: &str) -> Result<()> {
    let metadata = registry
        .get(command)
        .ok_or_else(|| anyhow::anyhow!("Unknown command: {}", command))?;

    println!();
    println!("{}", format!("annactl {}", metadata.name).bold().cyan());
    println!("{}", "=".repeat(60));
    println!();

    // Short description
    println!("{}", metadata.description_short);
    println!();

    // Long description
    println!("{}", "Description:".bold());
    println!("{}", metadata.description_long);
    println!();

    // Metadata
    println!("{}", "Details:".bold());
    println!("  Category:     {}", format_category(metadata.category));
    println!("  Risk Level:   {}", format_risk(metadata.risk_level));

    if metadata.requires_root {
        println!("  Requires:     {}", "root/sudo access".yellow());
    }

    if metadata.requires_daemon {
        println!("  Requires:     {}", "daemon running".cyan());
    }

    if !metadata.available_states.is_empty() {
        println!("  Available in: {}", metadata.available_states.join(", "));
    }
    println!();

    // Examples
    if !metadata.examples.is_empty() {
        println!("{}", "Examples:".bold());
        for example in metadata.examples {
            println!("  $ {}", example.green());
        }
        println!();
    }

    // Related commands
    if !metadata.see_also.is_empty() {
        println!("{}", "See also:".bold());
        println!("  {}", metadata.see_also.join(", "));
        println!();
    }

    Ok(())
}

/// Display general help with adaptive filtering
fn display_general_help(
    registry: &CommandRegistry,
    context: DisplayContext,
    show_all: bool,
) -> Result<()> {
    println!();
    println!("{}", "Anna Assistant - Arch Linux System Administration".bold().cyan());
    println!("{}", "=".repeat(60));
    println!();

    // Show context information
    display_context_info(&context)?;

    let commands = if show_all {
        // Show all commands regardless of context
        let mut all: Vec<_> = registry.all().iter().collect();
        all.sort_by_key(|cmd| (cmd.category as u8, cmd.name));
        all
    } else {
        // Show only visible commands for context
        registry.visible(&context)
    };

    if commands.is_empty() {
        println!("No commands available in current context.");
        println!();
        println!("Try:");
        println!("  annactl help --all    Show all commands");
        println!();
        return Ok(());
    }

    // Group commands by category
    let user_safe: Vec<_> = commands
        .iter()
        .filter(|cmd| cmd.category == CommandCategory::UserSafe)
        .collect();

    let advanced: Vec<_> = commands
        .iter()
        .filter(|cmd| cmd.category == CommandCategory::Advanced)
        .collect();

    let internal: Vec<_> = commands
        .iter()
        .filter(|cmd| cmd.category == CommandCategory::Internal)
        .collect();

    // Display categories
    if !user_safe.is_empty() {
        println!("{}", "🟢 User-Safe Commands".bold().green());
        println!("  {}", "Safe for everyday use by all users".dimmed());
        println!();
        display_command_list(&user_safe, &context)?;
        println!();
    }

    if !advanced.is_empty() {
        println!("{}", "🟡 Advanced Commands".bold().yellow());
        println!("  {}", "System administration - requires knowledge".dimmed());
        println!();
        display_command_list(&advanced, &context)?;
        println!();
    }

    if !internal.is_empty() {
        println!("{}", "🔴 Internal Commands".bold().red());
        println!("  {}", "Developer/diagnostic tools for experts".dimmed());
        println!();
        display_command_list(&internal, &context)?;
        println!();
    }

    // Footer with tips
    display_footer(&context, show_all)?;

    Ok(())
}

/// Display context information
fn display_context_info(context: &DisplayContext) -> Result<()> {
    println!("{}", "Current Context:".bold());

    println!("  User Level:   {}", format_user_level(context.user_level));

    if context.daemon_available {
        println!("  Daemon:       {}", "available".green());
    } else {
        println!("  Daemon:       {}", "unavailable".red());
        println!("  {}", "  (Some commands require daemon to be running)".dimmed());
    }

    println!("  System State: {}", format_system_state(&context.system_state));

    if context.is_constrained {
        println!("  Resources:    {}", "constrained".yellow());
    }

    if let Some(mode) = &context.monitoring_mode {
        println!("  Monitoring:   {}", mode);
    }

    println!();

    Ok(())
}

/// Display list of commands
fn display_command_list(commands: &[&&CommandMetadata], context: &DisplayContext) -> Result<()> {
    for cmd in commands {
        let name = format!("  {:<20}", cmd.name);

        // Highlight if relevant to current state
        if cmd.is_highlighted(context) {
            println!("{} {}", format!("{} ⭐", name).bold().yellow(), cmd.description_short);
        } else {
            println!("{} {}", name, cmd.description_short);
        }
    }

    Ok(())
}

/// Display footer with helpful tips
fn display_footer(context: &DisplayContext, show_all: bool) -> Result<()> {
    println!("{}", "Usage:".bold());
    println!("  annactl <command> [options]");
    println!("  annactl help <command>           Show detailed help for a command");

    if !show_all {
        println!("  annactl help --all               Show all commands (including internal)");
    }

    println!();
    println!("{}", "Tips:".bold());

    // Context-specific tips
    match context.system_state.as_str() {
        "degraded" => {
            println!("  ⭐ Your system is degraded. Try:");
            println!("     annactl            - Interactive REPL (ask me to fix issues)");
            println!("     annactl status     - Check Anna's health");
        }
        "iso_live" => {
            println!("  ⭐ You're in ISO live mode. Try:");
            println!("     annactl install    - Install Arch Linux");
            println!("     annactl rescue     - System rescue tools");
        }
        _ => {
            if !context.daemon_available {
                println!("  ⚠️  Daemon not running. Start it with:");
                println!("     sudo systemctl start annad");
            } else {
                println!("  💡 Commonly used commands:");
                println!("     annactl status     - Check system status");
                println!("     annactl health     - Run health probes");
                println!("     sudo annactl update - Update system packages");
            }
        }
    }

    println!();
    println!("  For more information: https://docs.anna-assistant.org");
    println!();

    Ok(())
}

/// Format category with color
fn format_category(category: CommandCategory) -> String {
    match category {
        CommandCategory::UserSafe => "User-Safe".green().to_string(),
        CommandCategory::Advanced => "Advanced".yellow().to_string(),
        CommandCategory::Internal => "Internal".red().to_string(),
    }
}

/// Format risk level with color
fn format_risk(risk: anna_common::command_meta::RiskLevel) -> String {
    use anna_common::command_meta::RiskLevel;

    match risk {
        RiskLevel::None => "None".green().to_string(),
        RiskLevel::Low => "Low".cyan().to_string(),
        RiskLevel::Medium => "Medium".yellow().to_string(),
        RiskLevel::High => "High".red().to_string(),
        RiskLevel::Critical => "Critical".red().bold().to_string(),
    }
}

/// Format user level with color
fn format_user_level(level: UserLevel) -> String {
    match level {
        UserLevel::Beginner => "Beginner".cyan().to_string(),
        UserLevel::Intermediate => "Intermediate".yellow().to_string(),
        UserLevel::Expert => "Expert".green().to_string(),
    }
}

/// Format system state with color
fn format_system_state(state: &str) -> String {
    match state {
        "healthy" => "Healthy".green().to_string(),
        "degraded" => "Degraded".yellow().to_string(),
        "critical" => "Critical".red().to_string(),
        "iso_live" => "ISO Live".cyan().to_string(),
        _ => state.to_string(),
    }
}

/// Detect user level from command usage history (Phase 3.2)
pub async fn detect_user_level() -> UserLevel {
    // For now, default to intermediate
    // TODO: Query persistent context for command usage history
    // - Count successful advanced commands
    // - Check if user has used internal commands
    // - Promote based on experience

    UserLevel::Intermediate
}

/// Detect system state by querying daemon (with fast timeout)
pub async fn detect_system_state(socket_path: Option<&str>) -> String {
    use crate::rpc_client::RpcClient;

    // Try to connect to daemon with quick timeout
    let mut client = match RpcClient::connect_quick(socket_path).await {
        Ok(c) => c,
        Err(_) => return "unknown".to_string(),
    };

    // Query status
    match client.sentinel_status().await {
        Ok(anna_common::ipc::ResponseData::SentinelStatus(status)) => {
            status.system_state
        }
        _ => "unknown".to_string(),
    }
}

/// Check if daemon is available (with fast timeout)
pub async fn check_daemon_available(socket_path: Option<&str>) -> bool {
    use crate::rpc_client::RpcClient;

    RpcClient::connect_quick(socket_path).await.is_ok()
}

/// Build display context from current system state
pub async fn build_context(socket_path: Option<&str>) -> DisplayContext {
    let user_level = detect_user_level().await;
    let daemon_available = check_daemon_available(socket_path).await;
    let system_state = if daemon_available {
        detect_system_state(socket_path).await
    } else {
        "unknown".to_string()
    };

    // Try to get profile for resource constraints
    // TODO: Re-enable once daemon is updated to support get_profile
    let (is_constrained, monitoring_mode) = (false, None);

    /* Disabled for now - old daemon doesn't support this
    let (is_constrained, monitoring_mode) = if daemon_available {
        use crate::rpc_client::RpcClient;

        if let Ok(mut client) = RpcClient::connect_quick(socket_path).await {
            if let Ok(anna_common::ipc::ResponseData::Profile(profile)) = client.get_profile().await {
                (profile.is_constrained, Some(profile.recommended_monitoring_mode))
            } else {
                (false, None)
            }
        } else {
            (false, None)
        }
    } else {
        (false, None)
    };
    */

    DisplayContext {
        user_level,
        daemon_available,
        system_state,
        is_constrained,
        monitoring_mode,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_category() {
        let user_safe = format_category(CommandCategory::UserSafe);
        assert!(user_safe.contains("User-Safe"));

        let advanced = format_category(CommandCategory::Advanced);
        assert!(advanced.contains("Advanced"));

        let internal = format_category(CommandCategory::Internal);
        assert!(internal.contains("Internal"));
    }

    #[test]
    fn test_format_user_level() {
        let beginner = format_user_level(UserLevel::Beginner);
        assert!(beginner.contains("Beginner"));

        let intermediate = format_user_level(UserLevel::Intermediate);
        assert!(intermediate.contains("Intermediate"));

        let expert = format_user_level(UserLevel::Expert);
        assert!(expert.contains("Expert"));
    }

    #[test]
    fn test_format_system_state() {
        assert!(format_system_state("healthy").contains("Healthy"));
        assert!(format_system_state("degraded").contains("Degraded"));
        assert!(format_system_state("critical").contains("Critical"));
        assert!(format_system_state("iso_live").contains("ISO Live"));
    }

    #[tokio::test]
    async fn test_detect_user_level() {
        let level = detect_user_level().await;
        // Should return intermediate by default
        assert_eq!(level, UserLevel::Intermediate);
    }
}
