//! Anna CLI (annactl) - User interface wrapper
//!
//! Talks to LLM-A only. Provides clean output.

mod client;
mod llm_client;
mod orchestrator;
mod output;

use anyhow::Result;
use clap::{Parser, Subcommand};
use owo_colors::OwoColorize;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "annactl")]
#[command(author = "Anna Assistant Team")]
#[command(version)]
#[command(about = "Anna - Your intelligent Linux assistant", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Ask Anna a question directly
    #[arg(trailing_var_arg = true)]
    question: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Ask Anna a question
    Ask {
        /// The question to ask
        #[arg(trailing_var_arg = true)]
        question: Vec<String>,
    },
    /// Check Anna daemon status
    Status,
    /// List available probes
    Probes,
    /// Run a specific probe
    Probe {
        /// Probe ID to run
        id: String,
    },
    /// Show configuration
    Config,
    /// Initialize Anna (first run)
    Init,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "annactl=warn".into()),
        ))
        .with(tracing_subscriber::fmt::layer().without_time())
        .init();

    let cli = Cli::parse();

    // Handle commands
    match cli.command {
        Some(Commands::Ask { question }) => {
            let q = question.join(" ");
            if q.is_empty() {
                eprintln!("{}  Please provide a question", "".red());
                std::process::exit(1);
            }
            run_ask(&q).await?;
        }
        Some(Commands::Status) => {
            run_status().await?;
        }
        Some(Commands::Probes) => {
            run_list_probes().await?;
        }
        Some(Commands::Probe { id }) => {
            run_probe(&id).await?;
        }
        Some(Commands::Config) => {
            run_config().await?;
        }
        Some(Commands::Init) => {
            run_init().await?;
        }
        None => {
            // Direct question mode
            if !cli.question.is_empty() {
                let q = cli.question.join(" ");
                run_ask(&q).await?;
            } else {
                print_banner();
                println!("\nUsage: {} <question>", "annactl".cyan());
                println!("       {} ask <question>", "annactl".cyan());
                println!("       {} status", "annactl".cyan());
                println!("\nRun {} for more options", "annactl --help".cyan());
            }
        }
    }

    Ok(())
}

fn print_banner() {
    println!(
        "\n{}  {}",
        "".bright_magenta(),
        format!("Anna v{}", env!("CARGO_PKG_VERSION")).bright_white()
    );
    println!("   Your intelligent Linux assistant\n");
}

async fn run_ask(question: &str) -> Result<()> {
    let daemon = client::DaemonClient::new();

    // Check daemon health
    if !daemon.is_healthy().await {
        eprintln!("{}  Anna daemon is not running", "".red());
        eprintln!("   Run: {} to start", "sudo systemctl start annad".cyan());
        std::process::exit(1);
    }

    // Run orchestrator
    let result = orchestrator::process_question(question, &daemon).await?;

    // Output result
    output::display_response(&result);

    Ok(())
}

async fn run_status() -> Result<()> {
    let daemon = client::DaemonClient::new();

    match daemon.health().await {
        Ok(health) => {
            println!(
                "{}  Anna Daemon {}",
                "".green(),
                "running".bright_green()
            );
            println!("   Version: {}", health.version.cyan());
            println!("   Uptime: {}s", health.uptime_seconds);
            println!("   Probes: {}", health.probes_available);
        }
        Err(_) => {
            println!("{}  Anna Daemon {}", "".red(), "stopped".bright_red());
            println!(
                "   Start with: {}",
                "sudo systemctl start annad".cyan()
            );
        }
    }

    Ok(())
}

async fn run_list_probes() -> Result<()> {
    let daemon = client::DaemonClient::new();
    let probes = daemon.list_probes().await?;

    println!("{}  Available Probes\n", "".cyan());
    for probe in probes.probes {
        println!(
            "   {}  {}  ({})",
            "".bright_blue(),
            probe.id.bright_white(),
            probe.cache_policy.dimmed()
        );
    }

    Ok(())
}

async fn run_probe(id: &str) -> Result<()> {
    let daemon = client::DaemonClient::new();
    let result = daemon.run_probe(id, false).await?;

    if result.success {
        println!("{}  Probe: {}\n", "".green(), id.cyan());
        let formatted = serde_json::to_string_pretty(&result.data)?;
        println!("{}", formatted);
    } else {
        println!("{}  Probe failed: {}", "".red(), id);
        if let Some(err) = result.error {
            println!("   {}", err.red());
        }
    }

    Ok(())
}

async fn run_config() -> Result<()> {
    let config = anna_common::AnnaConfig::default();
    println!("{}  Anna Configuration\n", "".cyan());
    println!("   Version: {}", config.version.cyan());
    println!("   Orchestrator: {}", config.models.orchestrator.yellow());
    println!("   Expert: {}", config.models.expert.yellow());
    println!("   Daemon URL: {}", config.daemon_socket.dimmed());
    println!("   Ollama URL: {}", config.ollama_url.dimmed());

    Ok(())
}

async fn run_init() -> Result<()> {
    println!("{}  Initializing Anna...\n", "".cyan());

    // Detect hardware
    println!("   {}  Detecting hardware...", "".bright_blue());
    let hw = detect_hardware();
    println!("      RAM: {} GB", hw.ram_gb);
    println!("      CPU: {} cores", hw.cpu_cores);
    println!(
        "      GPU: {}",
        if hw.has_gpu { "detected" } else { "none" }
    );

    // Select models
    let models = hw.select_models();
    println!("\n   {}  Selected models:", "".bright_blue());
    println!("      Orchestrator (LLM-A): {}", models.orchestrator.yellow());
    println!("      Expert (LLM-B): {}", models.expert.yellow());

    println!(
        "\n{}  Initialization complete",
        "".green()
    );
    println!("   Run {} to check status", "annactl status".cyan());

    Ok(())
}

fn detect_hardware() -> anna_common::HardwareInfo {
    use sysinfo::System;

    let mut sys = System::new_all();
    sys.refresh_all();

    let ram_gb = sys.total_memory() / 1024 / 1024 / 1024;
    let cpu_cores = sys.cpus().len();

    // Simple GPU detection (check for nvidia/amd devices)
    let has_gpu = std::path::Path::new("/dev/nvidia0").exists()
        || std::path::Path::new("/dev/dri/card0").exists();

    anna_common::HardwareInfo {
        ram_gb,
        cpu_cores,
        has_gpu,
        vram_gb: None, // Would need nvidia-smi for accurate detection
    }
}
