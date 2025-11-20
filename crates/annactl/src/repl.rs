//! REPL - Conversational interface for Anna
//!
//! Phase 5.1: Conversational UX
//! Interactive Read-Eval-Print Loop for natural language interaction

use anna_common::context::db::{ContextDb, DbLocation};
use anna_common::display::{print_privacy_explanation, print_prompt, UI};
use anna_common::llm::ChatMessage;  // Beta.110: For conversation history
use anyhow::Result;
use std::collections::HashMap;
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
    let health_icon = match ctx.health.status {
        crate::health::HealthStatus::Healthy => fmt::success("✓"),
        crate::health::HealthStatus::Degraded => fmt::warning("⚠"),
        crate::health::HealthStatus::Broken => fmt::error("✗"),
    };

    // Simple one-line status bar
    format!(
        "{} {} {} {}",
        fmt::dimmed(&ctx.bar_prefix),
        health_icon,
        fmt::dimmed(&format!("LLM: {}", ctx.llm_mode)),
        fmt::dimmed(&format!("{}@{}", ctx.user, ctx.host))
    )
}

fn print_status_bar(ctx: &ReplUiContext) {
    println!("{}", status_bar(ctx));
    println!();  // Blank line after status
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
        Err(_e) => {
            // Silently fall back to English if database unavailable
            // This is expected on first run before installer completes
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
            ui.success("Auto-repair completed");  // ui.success() already adds ✓
        } else {
            ui.warning("Auto-repair partially completed");  // ui.warning() already adds ⚠
        }
        for action in &repair.actions {
            println!("  • {}", action);
        }
        println!();
    }

    // If health is still broken after repair, refuse to start REPL
    if health.status == crate::health::HealthStatus::Broken {
        print_status_bar(&ctx);
        ui.error("Anna cannot start: critical health issues remain");
        println!("Please run 'annactl status' for details");
        std::process::exit(health.exit_code());
    }

    // Beta.146: Temporarily disabled during refactoring
    // TODO: Re-enable LLM setup and brain upgrade notifications
    // Check if setup wizard needs to run
    // if let Some(db) = db_arc.as_ref() {
    //     let _ = crate::run_llm_setup_if_needed(&ui, db).await;
    // }

    // Check for brain upgrade notifications
    // if let Some(db) = db_arc.as_ref() {
    //     let _ = crate::check_brain_upgrade_notification(&ui, db).await;
    // }

    // Silence unused warnings
    let _ = (&ui, &db_arc);

    // Show status bar and welcome message
    print_status_bar(&ctx);

    // Format health status nicely (not debug output)
    let health_msg = match health.status {
        crate::health::HealthStatus::Healthy => "All systems operational",
        crate::health::HealthStatus::Degraded => "Some issues detected",
        crate::health::HealthStatus::Broken => "Critical issues present",
    };

    println!(
        "Hello {}, your Arch sysadmin is awake. {}.",
        ctx.user, health_msg
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

    // Beta.89: Launch full-screen TUI interface with streaming support
    // The TUI replaces the old line-by-line REPL
    println!();
    println!("Launching full-screen interface...");
    println!("Press Ctrl+C or Ctrl+Q to exit");
    println!();

    // Give user a moment to read the message
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // Launch TUI (this blocks until user exits)
    annactl::tui_v2::run().await
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
                // Beta.53: LLM integration for natural language queries
                // Beta.110: Pass conversation_history for multi-turn streaming
                // Conversation history is now updated inside handle_llm_query
                handle_llm_query(&original, db.as_ref(), &mut conversation_history).await;
            }
        }
    }
    Ok(())
}

// Beta.53: Old HistorianSnapshot removed - replaced by SystemSummary from anna_common::historian
// This function is stubbed out for now
#[allow(dead_code)]
fn load_historian_snapshot(_db: &ContextDb) {
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

    // Beta.89: Only suggest actually existing commands
    ui.info("Improvement suggestions will be based on 30-day Historian data once fully implemented.");
    println!();
    ui.info("For now, try:");
    println!("  • Run 'annactl status' for current system health");
    println!("  • Ask me specific questions about your system");
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

/// Handle LLM query for natural language understanding (Beta.53, Beta.110: streaming)
async fn handle_llm_query(
    user_message: &str,
    db: Option<&Arc<ContextDb>>,
    conversation_history: &mut Vec<ChatMessage>,
) {
    use anna_common::template_library::TemplateLibrary;
    use std::collections::HashMap;

    let ui = UI::auto();

    // Beta.112: Expanded template matching (55 new templates for consistency)
    let library = TemplateLibrary::default();
    let input_lower = user_message.to_lowercase();

    // Helper function to check for word boundaries (prevents "swapping" matching "swap")
    let contains_word = |text: &str, word: &str| -> bool {
        text.split_whitespace()
            .chain(text.split(&['-', '_', '/', '.', ',', '!', '?'][..]))
            .any(|w| w.eq_ignore_ascii_case(word))
    };

    // Pattern matching for template selection (Beta.112: MASSIVELY expanded - 68 templates)
    let template_match = if contains_word(&input_lower, "swap") {
        Some(("check_swap_status", HashMap::new()))
    } else if contains_word(&input_lower, "gpu") || contains_word(&input_lower, "vram") {
        Some(("check_gpu_memory", HashMap::new()))
    } else if input_lower.contains("wifi") || input_lower.contains("wireless") ||
       (input_lower.contains("network") && (input_lower.contains("slow") || input_lower.contains("issue") || input_lower.contains("problem"))) {
        Some(("wifi_diagnostics", HashMap::new()))
    } else if contains_word(&input_lower, "kernel") && (input_lower.contains("version") || input_lower.contains("what") || input_lower.contains("running")) {
        Some(("check_kernel_version", HashMap::new()))
    } else if input_lower.contains("disk") && input_lower.contains("space") {
        Some(("check_disk_space", HashMap::new()))
    } else if (contains_word(&input_lower, "memory") || contains_word(&input_lower, "ram")) && !input_lower.contains("gpu") {
        Some(("check_memory", HashMap::new()))
    } else if contains_word(&input_lower, "uptime") {
        Some(("check_uptime", HashMap::new()))
    } else if contains_word(&input_lower, "cpu") && (input_lower.contains("model") || input_lower.contains("what") || input_lower.contains("processor")) {
        Some(("check_cpu_model", HashMap::new()))
    } else if contains_word(&input_lower, "cpu") && (input_lower.contains("load") || input_lower.contains("usage")) {
        Some(("check_cpu_load", HashMap::new()))
    } else if contains_word(&input_lower, "distro") || (input_lower.contains("arch") && input_lower.contains("version")) {
        Some(("check_distro", HashMap::new()))
    } else if input_lower.contains("failed") && input_lower.contains("service") {
        Some(("check_failed_services", HashMap::new()))
    } else if input_lower.contains("journal") && input_lower.contains("error") {
        Some(("check_journal_errors", HashMap::new()))
    } else if input_lower.contains("weak") || (input_lower.contains("diagnostic") && input_lower.contains("system")) {
        Some(("system_weak_points_diagnostic", HashMap::new()))

    // Beta.112: PACKAGE MANAGEMENT (13 new templates)
    } else if input_lower.contains("orphan") || (input_lower.contains("unused") && input_lower.contains("package")) {
        Some(("list_orphaned_packages", HashMap::new()))
    } else if input_lower.contains("aur") {
        Some(("list_aur_packages", HashMap::new()))
    } else if input_lower.contains("pacman") && (input_lower.contains("cache") || input_lower.contains("size")) {
        Some(("check_pacman_cache_size", HashMap::new()))
    } else if input_lower.contains("mirror") {
        Some(("check_pacman_mirrors", HashMap::new()))
    } else if (input_lower.contains("update") || input_lower.contains("upgrade")) && !input_lower.contains("brain") {
        Some(("check_package_updates", HashMap::new()))
    } else if input_lower.contains("keyring") {
        Some(("check_archlinux_keyring", HashMap::new()))
    } else if input_lower.contains("package") && input_lower.contains("integrity") {
        Some(("check_package_integrity", HashMap::new()))
    } else if input_lower.contains("clean") && input_lower.contains("cache") {
        Some(("clean_package_cache", HashMap::new()))
    } else if input_lower.contains("explicit") && input_lower.contains("package") {
        Some(("list_explicit_packages", HashMap::new()))
    } else if input_lower.contains("pacman") && (input_lower.contains("status") || input_lower.contains("lock")) {
        Some(("check_pacman_status", HashMap::new()))
    } else if input_lower.contains("dependency") && input_lower.contains("conflict") {
        Some(("check_dependency_conflicts", HashMap::new()))
    } else if input_lower.contains("pending") && input_lower.contains("update") {
        Some(("check_pending_updates", HashMap::new()))
    } else if input_lower.contains("recent") && input_lower.contains("pacman") {
        Some(("show_recent_pacman_operations", HashMap::new()))

    // Beta.112: BOOT & SYSTEMD (8 new templates)
    } else if input_lower.contains("boot") && (input_lower.contains("time") || input_lower.contains("slow")) {
        Some(("analyze_boot_time", HashMap::new()))
    } else if input_lower.contains("boot") && input_lower.contains("error") {
        Some(("check_boot_errors", HashMap::new()))
    } else if input_lower.contains("boot") && input_lower.contains("log") {
        Some(("show_boot_log", HashMap::new()))
    } else if input_lower.contains("systemd") && input_lower.contains("timer") {
        Some(("check_systemd_timers", HashMap::new()))
    } else if input_lower.contains("journal") && input_lower.contains("size") {
        Some(("analyze_journal_size", HashMap::new()))
    } else if input_lower.contains("recent") && input_lower.contains("error") {
        Some(("show_recent_journal_errors", HashMap::new()))
    } else if input_lower.contains("kernel") && input_lower.contains("update") {
        Some(("check_recent_kernel_updates", HashMap::new()))
    } else if input_lower.contains("systemd") && input_lower.contains("version") {
        Some(("check_systemd_version", HashMap::new()))

    // Beta.112: CPU & PERFORMANCE (8 new templates)
    } else if input_lower.contains("cpu") && (input_lower.contains("freq") || input_lower.contains("speed")) {
        Some(("check_cpu_frequency", HashMap::new()))
    } else if input_lower.contains("cpu") && input_lower.contains("governor") {
        Some(("check_cpu_governor", HashMap::new()))
    } else if input_lower.contains("cpu") && input_lower.contains("usage") {
        Some(("analyze_cpu_usage", HashMap::new()))
    } else if (input_lower.contains("cpu") || input_lower.contains("temperature")) && input_lower.contains("temp") {
        Some(("check_cpu_temperature", HashMap::new()))
    } else if input_lower.contains("throttl") {
        Some(("detect_cpu_throttling", HashMap::new()))
    } else if input_lower.contains("top") && input_lower.contains("cpu") {
        Some(("show_top_cpu_processes", HashMap::new()))
    } else if input_lower.contains("load") && input_lower.contains("average") {
        Some(("check_load_average", HashMap::new()))
    } else if input_lower.contains("context") && input_lower.contains("switch") {
        Some(("analyze_context_switches", HashMap::new()))

    // Beta.112: MEMORY (6 new templates)
    } else if input_lower.contains("memory") && input_lower.contains("usage") {
        Some(("check_memory_usage", HashMap::new()))
    } else if input_lower.contains("swap") && input_lower.contains("usage") {
        Some(("check_swap_usage", HashMap::new()))
    } else if input_lower.contains("memory") && input_lower.contains("pressure") {
        Some(("analyze_memory_pressure", HashMap::new()))
    } else if input_lower.contains("top") && input_lower.contains("memory") {
        Some(("show_top_memory_processes", HashMap::new()))
    } else if input_lower.contains("oom") {
        Some(("check_oom_killer", HashMap::new()))
    } else if input_lower.contains("huge") && input_lower.contains("page") {
        Some(("check_huge_pages", HashMap::new()))

    // Beta.112: NETWORK (7 new templates)
    } else if input_lower.contains("dns") {
        Some(("check_dns_resolution", HashMap::new()))
    } else if input_lower.contains("network") && input_lower.contains("interface") {
        Some(("check_network_interfaces", HashMap::new()))
    } else if input_lower.contains("routing") || input_lower.contains("route") {
        Some(("check_routing_table", HashMap::new()))
    } else if input_lower.contains("firewall") {
        Some(("check_firewall_rules", HashMap::new()))
    } else if input_lower.contains("port") && input_lower.contains("listen") {
        Some(("check_listening_ports", HashMap::new()))
    } else if input_lower.contains("latency") || input_lower.contains("ping") {
        Some(("check_network_latency", HashMap::new()))
    } else if input_lower.contains("networkmanager") {
        Some(("check_networkmanager_status", HashMap::new()))

    // Beta.112: GPU & DISPLAY (9 new templates)
    } else if input_lower.contains("nvidia") && !input_lower.contains("install") {
        Some(("check_nvidia_status", HashMap::new()))
    } else if input_lower.contains("amd") && (input_lower.contains("gpu") || input_lower.contains("radeon")) {
        Some(("check_amd_gpu", HashMap::new()))
    } else if input_lower.contains("gpu") && input_lower.contains("driver") {
        Some(("check_gpu_drivers", HashMap::new()))
    } else if input_lower.contains("gpu") && input_lower.contains("process") {
        Some(("check_gpu_processes", HashMap::new()))
    } else if input_lower.contains("gpu") && input_lower.contains("temp") {
        Some(("check_gpu_temperature", HashMap::new()))
    } else if input_lower.contains("display") || input_lower.contains("xorg") || input_lower.contains("wayland") {
        Some(("check_display_server", HashMap::new()))
    } else if input_lower.contains("desktop") && input_lower.contains("environment") {
        Some(("check_desktop_environment", HashMap::new()))
    } else if input_lower.contains("xorg") && input_lower.contains("error") {
        Some(("analyze_xorg_errors", HashMap::new()))
    } else if input_lower.contains("wayland") && input_lower.contains("compositor") {
        Some(("check_wayland_compositor", HashMap::new()))

    // Beta.112: HARDWARE (4 new templates)
    } else if input_lower.contains("temperature") || input_lower.contains("temp") || input_lower.contains("heat") {
        Some(("check_temperature", HashMap::new()))
    } else if input_lower.contains("usb") {
        Some(("check_usb_devices", HashMap::new()))
    } else if input_lower.contains("pci") {
        Some(("check_pci_devices", HashMap::new()))
    } else if input_lower.contains("hostname") {
        Some(("check_hostname", HashMap::new()))
    } else {
        None
    };

    // If template matched, instantiate and run it
    if let Some((template_id, params)) = template_match {
        if let Some(template) = library.get(template_id) {
            match template.instantiate(&params) {
                Ok(recipe) => {
                    ui.info(&format!("Running {}...", template_id.replace('_', " ")));
                    println!();

                    match std::process::Command::new("sh")
                        .arg("-c")
                        .arg(&recipe.command)
                        .output()
                    {
                        Ok(output) => {
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            for line in stdout.lines() {
                                ui.info(line);
                            }
                            println!();
                            return;
                        }
                        Err(e) => {
                            ui.error(&format!("Command failed: {}", e));
                            // Fall through to LLM
                        }
                    }
                }
                Err(_) => {
                    // Fall through to LLM
                }
            }
        }
    }

    // Beta.110: Word-by-word streaming like one-shot mode
    use owo_colors::OwoColorize;
    use std::io::{self, Write};
    use anna_common::llm::{ChatMessage, LlmClient, LlmConfig, LlmPrompt};

    // Show thinking indicator
    print!("{} ", "anna (thinking):".bright_magenta().dimmed());
    io::stdout().flush().unwrap();

    // Load LLM config from database
    let llm_config = if let Some(db) = db {
        match db.load_llm_config().await {
            Ok(config) => config,
            Err(e) => {
                print!("\r{}", " ".repeat(50));  // Clear thinking line
                println!();
                ui.error(&format!("❌ Failed to load LLM config: {}", e));
                ui.info("💡 Try: 'annactl repair' to check LLM setup");
                println!();
                return;
            }
        }
    } else {
        LlmConfig::default()
    };

    // Create LLM client
    let client = match LlmClient::from_config(&llm_config) {
        Ok(client) => client,
        Err(e) => {
            print!("\r{}", " ".repeat(50));  // Clear thinking line
            println!();
            ui.error(&format!("❌ Failed to create LLM client: {}", e));
            ui.info("💡 Try: 'annactl repair' to check LLM setup");
            println!();
            return;
        }
    };

    // Build system context (similar to one-shot mode)
    use crate::system_query::query_system_telemetry;
    let system_context = if let Ok(telemetry) = query_system_telemetry() {
        format!(
            "System Information:\n\
             - CPU: {}\n\
             - RAM: {:.1} GB used / {:.1} GB total\n\
             - GPU: {}\n\
             - OS: Arch Linux",
            telemetry.hardware.cpu_model,
            telemetry.memory.used_mb as f64 / 1024.0,
            telemetry.memory.total_mb as f64 / 1024.0,
            telemetry.hardware.gpu_info.as_deref().unwrap_or("None"),
        )
    } else {
        "OS: Arch Linux".to_string()
    };

    // Create prompt with system context and conversation history
    let system_prompt = format!(
        "{}\n\n{}",
        LlmClient::anna_system_prompt(),
        system_context
    );

    // Build conversation history into the prompt format
    // Convert our Vec<ChatMessage> to Option<Vec<ChatMessage>> for the prompt
    let history_for_prompt = if !conversation_history.is_empty() {
        Some(conversation_history.clone())
    } else {
        None
    };

    let prompt = LlmPrompt {
        system: system_prompt,
        user: user_message.to_string(),
        conversation_history: history_for_prompt,
    };

    // Beta.110: Stream response word-by-word
    let mut response_started = false;
    let mut full_response = String::new();
    let mut callback = |chunk: &str| {
        if !response_started {
            // Clear the "thinking" indicator and start response
            print!("\r{}", " ".repeat(50)); // Clear line
            print!("{} ", "anna:".bright_magenta().bold());
            response_started = true;
        }

        // Print each word/chunk as it arrives
        print!("{}", chunk.white());
        io::stdout().flush().unwrap();
        full_response.push_str(chunk);
    };

    match client.chat_stream(&prompt, &mut callback) {
        Ok(_) => {
            // Response complete
            println!("\n");

            // Add to conversation history
            conversation_history.push(ChatMessage {
                role: "user".to_string(),
                content: user_message.to_string(),
            });
            conversation_history.push(ChatMessage {
                role: "assistant".to_string(),
                content: full_response,
            });
        }
        Err(e) => {
            if !response_started {
                print!("\r{}", " ".repeat(50)); // Clear thinking line
            }
            println!();
            ui.error(&format!("❌ LLM streaming failed: {}", e));
            ui.info("💡 Try: 'annactl repair' to check LLM setup");
            println!();
        }
    }
}

/// Display structured LLM output sections
fn display_structured_llm_output(response: &str) {
    
    

    let ui = UI::auto();

    // Extract structured sections from response
    let sections = parse_anna_sections(response);

    // Display [ANNA_TUI_HEADER] if present
    if let Some(header) = sections.get("ANNA_TUI_HEADER") {
        display_tui_header(header);
    }

    // Display [ANNA_SUMMARY] if present
    if let Some(summary) = sections.get("ANNA_SUMMARY") {
        ui.info(summary.trim());
        println!();
    }

    // Display [ANNA_HUMAN_OUTPUT] (main answer) in a nice box
    if let Some(human_output) = sections.get("ANNA_HUMAN_OUTPUT") {
        println!();
        println!("{}", fmt::dimmed("┌─── Anna's Response ───────────────────────────────────────────────────────────────┐"));
        for line in human_output.trim().lines() {
            println!("{} {}", fmt::dimmed("│"), line);
        }
        println!("{}", fmt::dimmed("└───────────────────────────────────────────────────────────────────────────────────┘"));
        println!();
    } else if sections.is_empty() {
        // Fallback: show raw response if no structure found in a box
        println!();
        println!("{}", fmt::dimmed("┌─── Anna's Response ───────────────────────────────────────────────────────────────┐"));
        for line in response.trim().lines() {
            println!("{} {}", fmt::dimmed("│"), line);
        }
        println!("{}", fmt::dimmed("└───────────────────────────────────────────────────────────────────────────────────┘"));
        println!();
    }

    // Display [ANNA_ACTION_PLAN] if present
    if let Some(action_plan) = sections.get("ANNA_ACTION_PLAN") {
        ui.section_header("📋", "Proposed Actions");
        println!("{}", action_plan.trim());
        println!();
    }
}

/// Parse Anna structured output sections
fn parse_anna_sections(response: &str) -> HashMap<String, String> {
    use std::collections::HashMap;

    let mut sections = HashMap::new();
    let lines: Vec<&str> = response.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Check if this line is a section header [ANNA_NAME]
        if line.starts_with("[ANNA_") && line.ends_with(']') {
            let section_name = line[1..line.len()-1].to_string(); // Remove [ and ]
            let mut content_lines = Vec::new();
            i += 1;

            // Collect lines until we hit another [ANNA_ or [/ANNA_ or end
            while i < lines.len() {
                let next_line = lines[i].trim();
                if next_line.starts_with("[ANNA_") || next_line.starts_with("[/ANNA_") {
                    break;
                }
                content_lines.push(lines[i]);
                i += 1;
            }

            let content = content_lines.join("\n").trim().to_string();
            sections.insert(section_name, content);
        } else {
            i += 1;
        }
    }

    sections
}

/// Display TUI header info
fn display_tui_header(header_content: &str) {
    let ui = UI::auto();

    // Parse header fields (simple key: value format)
    for line in header_content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();

            // NOTE: Don't print TUI header metadata (status, model_hint, etc.)
            // These are internal fields for future TUI implementation
            // Printing them confuses users and clutters output
            {}
        }
    }
    println!();
}
