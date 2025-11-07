//! Anna Control - CLI client for Anna Assistant
//!
//! Provides user interface to interact with the Anna daemon.

mod commands;
mod tui;
mod rpc_client;

use anyhow::Result;
use clap::{Parser, Subcommand};

// Version is embedded at build time
const VERSION: &str = env!("ANNA_VERSION");

#[derive(Parser)]
#[command(name = "annactl")]
#[command(about = "Anna Assistant - Autonomous system administrator", long_about = None)]
#[command(version = VERSION)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show system status and daemon health
    Status,

    /// Get system recommendations (optionally filter by category)
    ///
    /// Examples:
    ///   annactl advise              # Show all recommendations
    ///   annactl advise security     # Show only security recommendations
    ///   annactl advise packages     # Show only package recommendations
    Advise {
        /// Category to filter by (security, packages, performance, hardware, system, network)
        category: Option<String>,

        /// Display mode: smart (default), critical, recommended, all
        #[arg(short, long, default_value = "smart")]
        mode: String,

        /// Maximum number of recommendations to show (0 = no limit)
        #[arg(short, long, default_value = "25")]
        limit: usize,
    },

    /// Apply recommendations by number, ID, range, or bundle
    ///
    /// Examples:
    ///   annactl apply 1                    # Apply recommendation #1
    ///   annactl apply 1-5                  # Apply recommendations 1 through 5
    ///   annactl apply 1,3,5                # Apply recommendations 1, 3, and 5
    ///   annactl apply --id amd-microcode   # Apply by recommendation ID
    ///   annactl apply --bundle hyprland    # Apply Hyprland setup bundle
    ///   annactl apply --auto               # Auto-apply all safe recommendations
    Apply {
        /// Recommendation number(s) to apply (e.g., "1", "1-5", "1,3,5-7")
        numbers: Option<String>,

        /// Apply a specific recommendation by ID
        #[arg(short, long)]
        id: Option<String>,

        /// Apply all recommendations in a workflow bundle
        #[arg(short, long)]
        bundle: Option<String>,

        /// Auto-apply all allowed actions without confirmation
        #[arg(short, long)]
        auto: bool,

        /// Dry run (show what would be done without applying)
        #[arg(short = 'n', long)]
        dry_run: bool,
    },

    /// List available workflow bundles
    Bundles,

    /// Rollback actions or bundles (Beta.91+)
    ///
    /// Examples:
    ///   annactl rollback list                 # List rollbackable actions
    ///   annactl rollback action mangohud      # Rollback specific action
    ///   annactl rollback last                 # Rollback last action
    ///   annactl rollback last 3               # Rollback last 3 actions
    ///   annactl rollback bundle hyprland      # Rollback bundle
    Rollback {
        #[command(subcommand)]
        action: RollbackAction,
    },

    /// Generate system health report (optionally filter by category)
    ///
    /// Examples:
    ///   annactl report           # Full system report
    ///   annactl report security  # Security-focused report
    Report {
        /// Category to focus on (security, performance, packages, etc.)
        category: Option<String>,
    },

    /// Run system diagnostics and optionally fix issues
    ///
    /// Examples:
    ///   annactl doctor              # Run diagnostics only
    ///   annactl doctor --fix        # Fix issues with confirmation
    ///   annactl doctor --fix --auto # Fix all issues automatically
    Doctor {
        /// Automatically fix detected issues
        #[arg(short, long)]
        fix: bool,

        /// Show what would be fixed without actually fixing
        #[arg(short = 'n', long)]
        dry_run: bool,

        /// Fix all issues without confirmation
        #[arg(short, long)]
        auto: bool,
    },

    /// Configure Anna settings interactively or get/set values
    ///
    /// Examples:
    ///   annactl config                        # Open interactive TUI
    ///   annactl config get autonomy_tier      # Get a value
    ///   annactl config set autonomy_tier 1    # Set a value
    Config {
        /// Action: get, set, or none for TUI
        action: Option<String>,

        /// Key to get/set
        key: Option<String>,

        /// Value to set (only for 'set' action)
        value: Option<String>,
    },

    /// View autonomous actions log
    Autonomy {
        /// Number of recent actions to show
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },

    /// Update Arch Wiki cache
    WikiCache {
        /// Force update even if cache is fresh
        #[arg(short, long)]
        force: bool,
    },

    /// Show system health score and trends
    Health,

    /// Dismiss a recommendation by number
    ///
    /// Examples:
    ///   annactl dismiss 1    # Dismiss recommendation #1
    Dismiss {
        /// Recommendation number to dismiss
        number: usize,
    },

    /// Show dismissed recommendations
    ///
    /// Examples:
    ///   annactl dismissed            # Show all dismissed items
    ///   annactl dismissed --undismiss 1  # Un-dismiss an item by number
    Dismissed {
        /// Un-dismiss a recommendation by its list number
        #[arg(short, long)]
        undismiss: Option<usize>,
    },

    /// View application history and analytics
    History {
        /// Number of days to show (default: 30)
        #[arg(short, long, default_value = "30")]
        days: i64,

        /// Show detailed entries
        #[arg(short = 'v', long)]
        detailed: bool,
    },

    /// Check for updates and optionally install them
    ///
    /// Examples:
    ///   annactl update              # Check for updates
    ///   annactl update --install    # Install available updates
    Update {
        /// Automatically install updates without confirmation
        #[arg(short, long)]
        install: bool,

        /// Check only (don't show full release notes)
        #[arg(short, long)]
        check: bool,
    },

    /// Manage ignore filters (categories and priorities)
    ///
    /// Examples:
    ///   annactl ignore show                      # Show current filters
    ///   annactl ignore category "Cosmetic"       # Ignore category
    ///   annactl ignore priority Optional         # Ignore priority
    ///   annactl ignore unignore category Desktop # Remove filter
    ///   annactl ignore reset                     # Clear all filters
    Ignore {
        #[command(subcommand)]
        action: IgnoreAction,
    },

    // TUI temporarily disabled for 1.0 release - will be re-enabled in 2.0 with better UX
    // /// Open interactive TUI for browsing and applying recommendations
    // Tui,

    /// Generate shell completion scripts
    ///
    /// Examples:
    ///   annactl completions bash > /usr/share/bash-completion/completions/annactl
    ///   annactl completions zsh > /usr/share/zsh/site-functions/_annactl
    ///   annactl completions fish > ~/.config/fish/completions/annactl.fish
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

#[derive(Debug, Subcommand)]
pub enum RollbackAction {
    /// List all rollbackable actions
    List,

    /// Rollback a specific action by advice ID
    Action {
        /// Advice ID to rollback
        advice_id: String,

        /// Dry run (show what would be done without executing)
        #[arg(short = 'n', long)]
        dry_run: bool,
    },

    /// Rollback last N actions (default: 1)
    Last {
        /// Number of actions to rollback (default: 1)
        #[arg(default_value = "1")]
        count: usize,

        /// Dry run (show what would be done without executing)
        #[arg(short = 'n', long)]
        dry_run: bool,
    },

    /// Rollback a workflow bundle
    Bundle {
        /// Bundle name to rollback
        name: String,

        /// Dry run (show what would be removed without removing)
        #[arg(short = 'n', long)]
        dry_run: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum IgnoreAction {
    /// Show current ignore filters
    Show,

    /// List recommendations currently hidden by ignore filters
    ListHidden,

    /// Ignore a category
    Category {
        /// Category name to ignore
        name: String,
    },

    /// Ignore a priority level (Mandatory, Recommended, Optional, Cosmetic)
    Priority {
        /// Priority level to ignore
        level: String,
    },

    /// Remove a filter
    Unignore {
        /// Type: 'category' or 'priority'
        filter_type: String,

        /// Value to un-ignore
        value: String,
    },

    /// Reset all ignore filters
    Reset,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Status => commands::status().await,
        Commands::Advise { category, mode, limit } => {
            commands::advise(None, mode, category, limit).await
        }
        Commands::Apply { numbers, id, bundle, auto, dry_run } => {
            commands::apply(id, numbers, bundle, auto, dry_run).await
        }
        Commands::Bundles => commands::bundles().await,
        Commands::Rollback { action } => match action {
            RollbackAction::List => commands::rollback_list().await,
            RollbackAction::Action { advice_id, dry_run } => {
                commands::rollback_action(&advice_id, dry_run).await
            }
            RollbackAction::Last { count, dry_run } => {
                commands::rollback_last(count, dry_run).await
            }
            RollbackAction::Bundle { name, dry_run } => {
                commands::rollback_bundle(&name, dry_run).await
            }
        },
        Commands::Report { category } => commands::report(category).await,
        Commands::Doctor { fix, dry_run, auto } => commands::doctor(fix, dry_run, auto).await,
        Commands::Config { action, key, value } => commands::config_new(action, key, value).await,
        Commands::Autonomy { limit } => commands::autonomy(limit).await,
        Commands::WikiCache { force } => commands::wiki_cache(force).await,
        Commands::Health => commands::health().await,
        Commands::Dismiss { number } => commands::dismiss(None, Some(number)).await,
        Commands::Dismissed { undismiss } => commands::dismissed(undismiss).await,
        Commands::History { days, detailed } => commands::history(days, detailed).await,
        Commands::Update { install, check } => commands::update(install, check).await,
        Commands::Ignore { action } => commands::ignore(action).await,
        // TUI disabled for 1.0 release
        // Commands::Tui => tui::run().await,
        Commands::Completions { shell } => {
            generate_completions(shell);
            Ok(())
        }
    }
}

/// Generate shell completion scripts
fn generate_completions(shell: clap_complete::Shell) {
    use clap::CommandFactory;
    use clap_complete::generate;

    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();

    generate(shell, &mut cmd, bin_name, &mut std::io::stdout());
}
