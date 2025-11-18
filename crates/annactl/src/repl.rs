//! REPL - Conversational interface for Anna
//!
//! Phase 5.1: Conversational UX
//! Interactive Read-Eval-Print Loop for natural language interaction

use anna_common::context::db::{ContextDb, DbLocation};
use anna_common::display::{print_privacy_explanation, print_prompt, UI};
use anyhow::Result;
use chrono::Local;
use std::env;
use std::io::{self, BufRead};
use std::sync::Arc;

use crate::intent_router::{self, Intent};
use crate::version_banner;
use anna_common::terminal_format as fmt;

#[derive(Clone)]
struct ReplUiContext {
    bar_prefix: String,
    user: String,
    host: String,
    llm_mode: String,
    health: crate::health::HealthReport,
    auto_update: String,
}

fn current_user() -> String {
    env::var("USER").unwrap_or_else(|_| "unknown".to_string())
}

fn current_host() -> String {
    std::process::Command::new("hostname")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "localhost".to_string())
}

fn status_bar(ctx: &ReplUiContext) -> String {
    let time = Local::now().format("%Y-%m-%d %H:%M").to_string();
    let health_tag = match ctx.health.status {
        crate::health::HealthStatus::Healthy => fmt::success("[OK]"),
        crate::health::HealthStatus::Degraded => fmt::warning("[DEGRADED]"),
        crate::health::HealthStatus::Broken => fmt::error("[BROKEN]"),
    };
    format!(
        "{} {} | {} | {} | User: {}@{} | Context: {} | Time: {}",
        ctx.bar_prefix,
        fmt::bold(&health_tag),
        fmt::dimmed(&format!("LLM: {}", ctx.llm_mode)),
        fmt::dimmed(&format!("Mode: {}", ctx.auto_update)),
        ctx.user,
        ctx.host,
        fmt::dimmed("local"),
        time
    )
}

fn print_status_bar(ctx: &ReplUiContext) {
    println!("{}", status_bar(ctx));
}

/// Start the conversational REPL
pub async fn start_repl() -> Result<()> {
    let db_location = DbLocation::auto_detect();

    // Open database and load language config
    let (db_arc, lang_config) = match ContextDb::open(db_location).await {
        Ok(db) => {
            // Load saved language config from database
            let config = db.load_language_config().await.unwrap_or_default();
            (Some(Arc::new(db)), config)
        }
        Err(e) => {
            eprintln!("Warning: Failed to open context database: {}", e);
            eprintln!("Continuing without database features...");
            // Use default config (English) if database not available
            let config = anna_common::language::LanguageConfig::new();
            (None, config)
        }
    };

    // Create UI with loaded language config
    let ui = UI::new(&lang_config);

    // Display version banner with mode and update status
    if let Some(db) = db_arc.as_ref() {
        version_banner::display_startup_banner(db).await;
    }

    // Check health and auto-repair before starting REPL
    let health = match crate::health::HealthReport::check(true).await {
        Ok(report) => report,
        Err(e) => {
            ui.error(&format!("Failed to check Anna's health: {}", e));
            return Err(e);
        }
    };

    let llm_mode = if let Some(db) = db_arc.as_ref() {
        db.load_llm_config()
            .await
            .map(|c| version_banner::format_llm_mode(&c))
            .unwrap_or_else(|_| "Rules + Arch Wiki (LLM not configured)".to_string())
    } else {
        "Rules + Arch Wiki (LLM not configured)".to_string()
    };

    let ctx = ReplUiContext {
        bar_prefix: format!("Anna v{}", env!("CARGO_PKG_VERSION")),
        user: current_user(),
        host: current_host(),
        llm_mode,
        health: health.clone(),
        auto_update: "Auto-update: On".to_string(),
    };

    // If auto-repair happened, show what was fixed
    if let Some(repair) = &health.last_repair {
        println!();
        print_status_bar(&ctx);
        if repair.success {
            ui.success("✓ Auto-repair completed");
        } else {
            ui.warning("⚠ Auto-repair partially completed");
        }
        for action in &repair.actions {
            println!("  • {}", action);
        }
        println!();
    }

    // If health is still broken after repair, refuse to start REPL
    if health.status == crate::health::HealthStatus::Broken {
        print_status_bar(&ctx);
        ui.error("✗ Anna cannot start: critical health issues remain");
        println!("Please run 'annactl status' for details");
        std::process::exit(health.exit_code());
    }

    // Check if setup wizard needs to run
    if let Some(db) = db_arc.as_ref() {
        if let Err(e) = crate::run_llm_setup_if_needed(&ui, db).await {
            // Don't show SQL errors to user
            if !e.to_string().contains("updated_at") {
                eprintln!("Warning: LLM setup check failed: {}", e);
            }
        }
    }

    // Check for brain upgrade notifications
    if let Some(db) = db_arc.as_ref() {
        if let Err(e) = crate::check_brain_upgrade_notification(&ui, db).await {
            eprintln!("Warning: Brain upgrade check failed: {}", e);
        }
    }

    // Show status bar and welcome message in user's language
    print_status_bar(&ctx);
    println!(
        "Hello {}, your Arch sysadmin is awake. LLM mode: {}. System health: {:?}.",
        ctx.user, ctx.llm_mode, health.status
    );
    if health.last_repair.is_some() {
        println!("Repairs since last run:");
        if let Some(repair) = &health.last_repair {
            for action in &repair.actions {
                println!("  • {}", action);
            }
        }
        println!();
    }

    // Proactive startup summary - inform user of any issues
    display_startup_summary(&ui).await;

    run_repl_loop(ctx, db_arc).await
}

/// Display proactive startup summary with Historian data (Beta.53)
async fn display_startup_summary(_ui: &UI) {
    use crate::rpc_client::RpcClient;
    use anna_common::ipc::Method;

    // Fetch system facts from daemon
    let facts = match RpcClient::connect().await {
        Ok(mut rpc) => match rpc.call(Method::GetFacts).await {
            Ok(anna_common::ipc::ResponseData::Facts(f)) => Some(f),
            _ => None,
        },
        Err(_) => None,
    };

    let facts = match facts {
        Some(f) => f,
        None => return, // Can't connect to daemon, skip summary
    };

    // Fetch Historian 30-day summary from daemon
    let historian = match RpcClient::connect().await {
        Ok(mut rpc) => match rpc.call(Method::GetHistorianSummary).await {
            Ok(anna_common::ipc::ResponseData::HistorianSummary(h)) => Some(h),
            _ => None,
        },
        Err(_) => None,
    };

    // Get current LLM model from context DB
    let current_model = if let Some(db) = anna_common::context::db() {
        db.execute(|conn| {
            let mut stmt = conn.prepare("SELECT model FROM llm_config ORDER BY updated_at DESC LIMIT 1")?;
            let mut rows = stmt.query([])?;
            if let Some(row) = rows.next()? {
                let model: String = row.get(0)?;
                Ok(model)
            } else {
                Ok("unknown".to_string())
            }
        })
        .await
        .unwrap_or_else(|_| "unknown".to_string())
    } else {
        "unknown".to_string()
    };

    // Display comprehensive startup summary with Historian data
    crate::startup_summary::display_startup_summary(&facts, historian.as_ref(), &current_model);
}

/// Main REPL loop (factored out for clarity)
async fn run_repl_loop(ctx: ReplUiContext, db: Option<std::sync::Arc<ContextDb>>) -> Result<()> {
    use anna_common::llm::ChatMessage;

    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    let mut conversation_history: Vec<ChatMessage> = Vec::new();

    loop {
        print_status_bar(&ctx);
        print_prompt();
        let input = match lines.next() {
            Some(Ok(line)) => line.trim().to_string(),
            Some(Err(e)) => {
                let ui = UI::auto();
                ui.error(&format!("Error reading input: {}", e));
                continue;
            }
            None => break,
        };
        if input.is_empty() {
            continue;
        }
        let intent = intent_router::route_intent(&input);
        match intent {
            Intent::Exit => {
                let ui = UI::auto();
                ui.info("Goodbye! I'll keep watching your system in the background.");
                ui.info("Run 'annactl' anytime you need me.");
                println!();
                break;
            }
            Intent::AnnaStatus | Intent::SystemStatus => {
                handle_status_intent(&ctx, db.as_ref()).await;
            }
            Intent::Privacy => {
                let ui = UI::auto();
                ui.thinking();
                ui.section_header("🔒", "Privacy & Data Handling");
                print_privacy_explanation();
            }
            Intent::Report => {
                let ui = UI::auto();
                ui.thinking();
                crate::report_display::generate_professional_report().await;
            }
            Intent::Suggest | Intent::Improve => {
                handle_improve_intent(db.as_ref()).await;
            }
            Intent::AnnaSelfRepair => {
                handle_self_repair_intent().await;
            }
            Intent::Repair { .. } => {
                let ui = UI::auto();
                ui.thinking();
                ui.info("I can help you fix issues.");
                println!();
                ui.info("[TODO: Call repair handler]");
                println!();
                ui.info("First, let me show you what needs fixing.");
                ui.info("Ask: 'What should I improve?' to see suggestions.");
                println!();
            }
            Intent::Discard { .. } => {
                let ui = UI::auto();
                ui.thinking();
                ui.info("I can hide suggestions you don't want to see.");
                println!();
                ui.info("[TODO: Call discard handler]");
                println!();
                ui.info("Which suggestion would you like to ignore?");
                println!();
            }
            Intent::Autonomy { .. } => {
                let ui = UI::auto();
                ui.thinking();
                ui.info("Autonomy controls how much I can do automatically.");
                println!();
                ui.info("[TODO: Show/set autonomy level]");
                println!();
            }
            Intent::Apply { .. } => {
                let ui = UI::auto();
                ui.thinking();
                ui.info("I can apply changes for you once you confirm.");
                println!();
                ui.info("[TODO: Implement apply handler]");
                println!();
            }
            Intent::Personality { .. } => {
                let ui = UI::auto();
                ui.info("Personality adjustment coming in Beta.53!");
                println!();
            }
            Intent::Language { .. } => {
                let ui = UI::auto();
                ui.info("Language selection coming soon!");
                println!();
            }
            Intent::SetupBrain => {
                let ui = UI::auto();
                ui.info("LLM brain is already configured. Use 'annactl upgrade model' to change models.");
                println!();
            }
            Intent::Help => {
                let ui = UI::auto();
                ui.section_header("📖", "Anna Assistant Help");
                ui.info("Available commands:");
                ui.bullet_list(&[
                    "status - Show system status",
                    "report - Generate system report",
                    "suggest - Get improvement suggestions",
                    "exit - Exit Anna",
                ]);
                println!();
            }
            Intent::HistorianSummary => {
                handle_historian_summary(db.as_ref()).await;
            }
            Intent::OffTopic => {
                let ui = UI::auto();
                ui.info("I'm focused on Arch and your system. Ask me about that!");
                println!();
            }
            Intent::Unclear(original) => {
                conversation_history.push(ChatMessage {
                    role: "user".to_string(),
                    content: original.clone(),
                });
                // TODO: Implement LLM response with Beta.53 integration
                let ui = UI::auto();
                ui.info("I don't understand that yet. Try asking about system status, reports, or suggestions.");
                println!();
            }
        }
    }
    Ok(())
}

// Beta.53: Old HistorianSnapshot removed - replaced by SystemSummary from anna_common::historian
// This function is stubbed out for now
#[allow(dead_code)]
fn load_historian_snapshot(_db: &ContextDb) -> () {
    // TODO: Implement using Historian IPC to get SystemSummary from annad
}

async fn handle_status_intent(ctx: &ReplUiContext, _db: Option<&Arc<ContextDb>>) {
    let ui = UI::auto();
    ui.section_header("💻", "System Health");
    ui.info(&format!("Health: {:?}", ctx.health.status));
    ui.info(&format!("LLM: {}", ctx.llm_mode));

    // TODO Beta.53: Implement Historian summary via IPC to annad
    ui.info("(30-day Historian trends will be shown here once IPC is implemented)");
    println!();
}

async fn handle_improve_intent(_db: Option<&Arc<ContextDb>>) {
    let ui = UI::auto();
    ui.section_header("🛠", "What you can improve");

    // TODO Beta.53: Implement intelligent suggestions based on Historian trends
    ui.info("Improvement suggestions will be based on 30-day Historian data once IPC is implemented.");
    println!();
    ui.info("For now, try:");
    println!("  • Run 'annactl status' for current system health");
    println!("  • Run 'annactl report' for detailed analysis");
    println!();
}

async fn handle_self_repair_intent() {
    let ui = UI::auto();
    ui.section_header("🔧", "Anna Self-Repair");
    println!();

    match crate::health::HealthReport::check(true).await {
        Ok(report) => {
            ui.info(&format!("Health after repair: {:?}", report.status));
            if let Some(repair) = report.last_repair {
                if repair.actions.is_empty() {
                    ui.info("No changes were necessary.");
                } else {
                    ui.success("Actions taken:");
                    for action in repair.actions {
                        println!("  • {}", action);
                    }
                }
            }
            if report.status == crate::health::HealthStatus::Broken {
                ui.error("Some issues remain. Check network or try 'annactl status' for details.");
            }
        }
        Err(e) => {
            ui.error(&format!("Failed to run self-repair: {}", e));
        }
    }
}

async fn handle_historian_summary(_db: Option<&Arc<ContextDb>>) {
    let ui = UI::auto();
    ui.section_header("🗄", "Historian 30-Day Summary");

    // TODO Beta.53: Implement full Historian summary via IPC to annad
    ui.info("Historian summary will show:");
    println!("  • Boot time trends (average, fastest, slowest)");
    println!("  • CPU utilization trends");
    println!("  • Memory usage patterns");
    println!("  • Error trends and recurring issues");
    println!("  • Recent repair actions");
    println!();
    ui.info("This feature requires Historian IPC implementation in annad.");
    println!();
}
