// Adaptive Root Help Display
// Phase 3.8: Adaptive CLI Refinement
//
// Context-aware command display for progressive disclosure

use crate::context_detection::{should_use_color, ExecutionContext};
use anna_common::command_meta::{
    CommandCategory, CommandMetadata, CommandRegistry, DisplayContext,
};
use anna_common::display::UI;
use owo_colors::OwoColorize;

/// Display adaptive root help based on execution context
pub fn display_adaptive_root_help(show_all: bool, json_mode: bool) {
    let context = ExecutionContext::detect();

    if json_mode {
        display_json_help(&context, show_all);
        return;
    }

    if show_all {
        display_terminal_help(&context, show_all);
    } else {
        display_simple_help();
    }
}

/// Display simple, user-focused help (default)
fn display_simple_help() {
    use anna_common::terminal_format as fmt;
    let ui = UI::auto();

    println!();
    ui.section_header("🤖", "Anna Assistant");
    ui.info("Your local caretaker for this Arch Linux machine");
    println!();

    println!("Anna is a conversational system administrator. Just talk to her.");
    println!();

    ui.section_header("📋", "Commands");
    println!();
    println!(
        "  {}  - Start interactive conversation (REPL)",
        fmt::bold("annactl")
    );
    println!(
        "  {}  - Show comprehensive health report",
        fmt::bold("annactl status")
    );
    println!(
        "  {}    - Show this help message",
        fmt::bold("annactl help")
    );
    println!();

    ui.section_header("💬", "Natural Language Examples");
    println!();
    println!("Ask Anna anything about your system:");
    println!();
    println!("  annactl \"how are you?\"");
    println!("  annactl \"my computer feels slow\"");
    println!("  annactl \"what should I improve?\"");
    println!("  annactl \"fix yourself\"");
    println!();
    println!("Or start a conversation:");
    println!();
    println!("  annactl");
    println!();

    ui.section_header("📚", "More Information");
    println!();
    println!("Change language: annactl \"use Spanish\" / \"cambia al español\"");
    println!("Documentation: https://github.com/jjgarcianorway/anna-assistant");
    println!();
}

/// Display help in JSON format (machine-readable)
fn display_json_help(context: &ExecutionContext, show_all: bool) {
    let registry = CommandRegistry::new();
    let display_ctx = build_display_context(context);

    let commands: Vec<&CommandMetadata> = if show_all {
        registry.all().iter().collect()
    } else {
        registry.visible(&display_ctx)
    };

    let json_commands: Vec<serde_json::Value> = commands
        .iter()
        .map(|cmd| {
            serde_json::json!({
                "name": cmd.name,
                "category": format!("{:?}", cmd.category),
                "risk": format!("{:?}", cmd.risk_level),
                "description": cmd.description_short,
                "requires_root": cmd.requires_root,
                "requires_daemon": cmd.requires_daemon,
            })
        })
        .collect();

    let output = serde_json::json!({
        "context": format!("{:?}", context),
        "commands": json_commands,
        "total": json_commands.len(),
    });

    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

/// Display help in terminal format (human-readable)
fn display_terminal_help(context: &ExecutionContext, show_all: bool) {
    let use_color = should_use_color();
    let registry = CommandRegistry::new();
    let display_ctx = build_display_context(context);

    // Header
    if use_color {
        println!(
            "\n{}",
            "Anna Assistant - Adaptive Arch Linux Administration"
                .bold()
                .cyan()
        );
    } else {
        println!("\nAnna Assistant - Adaptive Arch Linux Administration");
    }
    println!("{}", "=".repeat(60));
    println!();

    // Context info
    print_context_info(context, use_color);

    // Get visible commands
    let commands: Vec<&CommandMetadata> = if show_all {
        registry.all().iter().collect()
    } else {
        registry.visible(&display_ctx)
    };

    // Separate by category
    let user_safe: Vec<_> = commands
        .iter()
        .filter(|c| c.category == CommandCategory::UserSafe)
        .copied()
        .collect();

    let advanced: Vec<_> = commands
        .iter()
        .filter(|c| c.category == CommandCategory::Advanced)
        .copied()
        .collect();

    let internal: Vec<_> = commands
        .iter()
        .filter(|c| c.category == CommandCategory::Internal)
        .copied()
        .collect();

    // Display UserSafe commands
    if !user_safe.is_empty() {
        print_category_header("Safe Commands", "🟢", "green", user_safe.len(), use_color);
        print_command_list(&user_safe, use_color);
        println!();
    }

    // Display Advanced commands (if visible)
    if !advanced.is_empty() {
        print_category_header(
            "Administrative Commands",
            "🟡",
            "yellow",
            advanced.len(),
            use_color,
        );
        print_command_list(&advanced, use_color);
        println!();
    }

    // Display Internal commands (only with --all)
    if !internal.is_empty() {
        print_category_header("Internal Commands", "🔴", "red", internal.len(), use_color);
        println!("  Developer and diagnostic tools");
        print_command_list(&internal, use_color);
        println!();
    }

    // Usage footer
    print_usage_footer(context, show_all, use_color);
}

/// Print context information
fn print_context_info(context: &ExecutionContext, use_color: bool) {
    let ctx_str = match context {
        ExecutionContext::User => "Normal User",
        ExecutionContext::Root => "Administrator (root)",
        ExecutionContext::Developer => "Developer Mode",
    };

    if use_color {
        println!("{}: {}", "Mode".bold(), ctx_str.cyan());
    } else {
        println!("Mode: {}", ctx_str);
    }
    println!();
}

/// Print category header with count
fn print_category_header(title: &str, emoji: &str, color: &str, count: usize, use_color: bool) {
    let header = if use_color {
        format!("{} {} ({} available)", emoji, title, count)
    } else {
        format!("{} ({} available)", title, count)
    };

    if use_color {
        match color {
            "green" => println!("{}", header.green().bold()),
            "yellow" => println!("{}", header.yellow().bold()),
            "red" => println!("{}", header.red().bold()),
            _ => println!("{}", header.bold()),
        }
    } else {
        println!("{}", header);
    }
}

/// Print command list
fn print_command_list(commands: &[&CommandMetadata], use_color: bool) {
    for cmd in commands {
        let name = format!("  {:<15}", cmd.name);
        let desc = cmd.description_short;

        if use_color && cmd.requires_root {
            println!("{} {}", name.yellow(), desc);
        } else if use_color {
            println!("{} {}", name, desc);
        } else {
            println!("{} {}", name, desc);
        }
    }
}

/// Print usage footer
fn print_usage_footer(context: &ExecutionContext, show_all: bool, use_color: bool) {
    if use_color {
        println!("{}", "Usage:".bold());
    } else {
        println!("Usage:");
    }

    println!("  annactl <command> [options]");
    println!("  annactl help <command>        Show detailed help for a command");

    if !show_all {
        println!("  annactl --help --all          Show all commands including internal");
    }

    println!();

    // Context-specific tips
    match context {
        ExecutionContext::User => {
            if use_color {
                println!("{}", "Tip:".bold());
            } else {
                println!("Tip:");
            }
            println!("  Run with sudo to access administrative commands:");
            println!("  sudo annactl update");
            println!();
            println!(
                "  Showing {} only - use 'annactl --help --all' to see more",
                if use_color {
                    "safe commands".green().to_string()
                } else {
                    "safe commands".to_string()
                }
            );
        }
        ExecutionContext::Root => {
            if use_color {
                println!("{}", "Note:".bold());
            } else {
                println!("Note:");
            }
            println!("  Running as root - administrative commands available");
            if !show_all {
                println!("  Use --help --all to see internal/developer commands");
            }
        }
        ExecutionContext::Developer => {
            if use_color {
                println!("{}", "Developer Mode:".bold());
            } else {
                println!("Developer Mode:");
            }
            println!("  All commands visible including internal diagnostics");
            println!("  Set via ANNA_DEV_MODE or ANNA_INTERNAL environment variables");
        }
    }

    println!();
    println!("For more help: https://docs.anna-assistant.org");
    println!();
}

/// Build DisplayContext from ExecutionContext
fn build_display_context(exec_ctx: &ExecutionContext) -> DisplayContext {
    DisplayContext {
        user_level: exec_ctx.to_user_level(),
        daemon_available: true, // Show all commands in help, even if daemon not running
        system_state: "healthy".to_string(), // Assume healthy for help display
        is_constrained: false,
        monitoring_mode: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_context_building() {
        let user_ctx = ExecutionContext::User;
        let display_ctx = build_display_context(&user_ctx);
        assert_eq!(
            display_ctx.user_level,
            anna_common::command_meta::UserLevel::Beginner
        );

        let root_ctx = ExecutionContext::Root;
        let display_ctx = build_display_context(&root_ctx);
        assert_eq!(
            display_ctx.user_level,
            anna_common::command_meta::UserLevel::Intermediate
        );
    }

    #[test]
    fn test_adaptive_help_no_panic() {
        // Just ensure it doesn't panic
        display_adaptive_root_help(false, true); // JSON mode
    }
}
