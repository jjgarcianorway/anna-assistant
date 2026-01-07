mod advice;
mod doctor;
mod paths;
mod persona;
mod quickscan;
mod rpc;
mod status;
mod ui;
mod uninstall;
mod update;

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::PathBuf;

use ui::{detect_style, load_ui_cfg, Style, UiCfg};

#[derive(Deserialize, Default, Clone)]
struct PartitionDetail {
    name: String,
    #[serde(default)]
    size: String,
    #[serde(default)]
    fstype: Option<String>,
    #[serde(default)]
    mountpoint: Option<String>,
    #[serde(default)]
    label: Option<String>,
}

#[derive(Deserialize)]
struct SystemSnapshot {
    os: String,
    kernel: String,
    uptime_secs: u64,
    cpu_model: String,
    cpu_cores_logical: usize,
    total_memory_mb: u64,
    available_memory_mb: u64,
    total_swap_mb: u64,
    free_swap_mb: u64,
    #[serde(default)]
    partitions: Vec<String>,
    #[serde(default)]
    partition_table: Vec<PartitionDetail>,
    #[serde(default)]
    gpu_model: Option<String>,
    #[serde(default)]
    audio_server: Option<String>,
    #[serde(default)]
    network_managers: Vec<String>,
    #[serde(default)]
    display_servers_running: Vec<String>,
    #[serde(default)]
    display_servers_available: Vec<String>,
    #[serde(default)]
    bootloaders: Vec<String>,
    #[serde(default)]
    dual_boot: bool,
    #[serde(default)]
    other_os_partitions: Vec<String>,
}

#[derive(Parser)]
#[command(name = "annactl", about = "CLI for Anna", version)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Show service status and recent logs (since last start)
    Status,
    /// Show the last collected system snapshot (from /var/lib/anna/system.json)
    Sysinfo {
        /// Print compact JSON instead of pretty output
        #[arg(long)]
        raw: bool,
    },
    /// Stub for plans
    Plans,
    /// Stub for showing a plan
    Show { id: String },
    /// Manage persona overrides
    Persona {
        #[command(subcommand)]
        cmd: PersonaCmd,
    },
    /// Inspect and preview persona-driven advice
    Advice {
        #[command(subcommand)]
        cmd: AdviceCmd,
    },
    /// Run a quick health check
    Quickscan {
        #[arg(long)]
        raw: bool,
        /// Run only if last scan was >24h ago
        #[arg(long)]
        auto: bool,
    },
    /// System diagnostics and health checks
    Doctor {
        #[command(subcommand)]
        cmd: DoctorCmd,
    },
    /// Uninstall Anna from the system
    Uninstall {
        /// Remove all data without creating a backup
        #[arg(long)]
        complete: bool,
        /// Keep data directories, only unregister the service
        #[arg(long)]
        keep: bool,
    },
    /// Manage Anna updates and version
    Update {
        #[command(subcommand)]
        cmd: UpdateCmd,
    },
    /// UI preferences and preview
    Ui {
        #[command(subcommand)]
        cmd: UiCmd,
    },
}

#[derive(Subcommand)]
enum UpdateCmd {
    /// Check for available updates
    Check,
    /// Apply available updates
    Apply {
        #[arg(long)]
        force: bool,
    },
    /// Rollback to previous version
    Rollback,
    /// Manage auto-update policy
    Policy {
        /// Set update policy: auto, manual, or notify
        mode: Option<String>,
    },
}

#[derive(Subcommand)]
enum DoctorCmd {
    /// Audit permissions for Anna directories and polkit rules
    Perms,
    /// Repair installation (create dirs, restart service, verify socket)
    Repair,
    /// Dump environment configuration (paths, mode, XDG vars)
    Env,
}

#[derive(Subcommand)]
enum UiCmd {
    /// Show styled output samples with current UI settings
    Preview,
}

#[derive(Subcommand)]
enum PersonaCmd {
    /// Show the current persona and metadata
    Show {
        #[arg(long)]
        raw: bool,
    },
    /// Set a manual persona override
    Set { name: String },
    /// Remove persona override
    Unset,
    /// Show persona samples for a date
    Samples {
        #[arg(long)]
        date: String,
        #[arg(long, default_value_t = 50)]
        tail: usize,
    },
    /// Show aggregated persona rollup for a date
    Rollup {
        #[arg(long)]
        date: String,
    },
    /// Show persona explanations without JSON
    Explain {
        #[arg(long)]
        raw: bool,
    },
    /// Show recent persona trigger information
    Triggers {
        #[arg(long)]
        raw: bool,
    },
    /// Show internal persona debug information (traits, confidence)
    Debug {
        #[arg(long)]
        raw: bool,
    },
}

#[derive(Subcommand)]
enum AdviceCmd {
    /// List available advice entries
    List {
        #[arg(long)]
        raw: bool,
    },
    /// Show a specific advice entry
    Show {
        id: String,
        #[arg(long)]
        raw: bool,
    },
    /// Apply an advice entry or all entries
    Apply {
        /// Advice ID to apply (omit to use --all)
        id: Option<String>,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        raw: bool,
        /// Apply all eligible advice entries
        #[arg(long)]
        all: bool,
        /// Include risky items when using --all
        #[arg(long)]
        force: bool,
    },
}

fn main() -> Result<()> {
    let ui_cfg = load_ui_cfg();
    let style = detect_style(&ui_cfg);
    let cli = Cli::parse();

    match cli.cmd {
        Cmd::Status => status::run(&style, &ui_cfg)?,
        Cmd::Sysinfo { raw } => sysinfo_cmd(raw, &ui_cfg, &style)?,
        Cmd::Plans => println!("(stub)"),
        Cmd::Show { id } => println!("Plan {}", id),
        Cmd::Persona { cmd } => handle_persona(cmd, &ui_cfg, &style),
        Cmd::Advice { cmd } => handle_advice(cmd, &ui_cfg, &style),
        Cmd::Quickscan { raw, auto } => {
            quickscan::run(quickscan::QuickscanArgs { raw, auto }, &ui_cfg, &style)?
        }
        Cmd::Doctor { cmd } => handle_doctor(cmd, &ui_cfg, &style),
        Cmd::Uninstall { complete, keep } => {
            uninstall::run(uninstall::UninstallArgs { complete, keep }, &ui_cfg, &style)?
        }
        Cmd::Update { cmd } => handle_update(cmd, &ui_cfg, &style),
        Cmd::Ui { cmd } => handle_ui(cmd, &ui_cfg, &style),
    }

    Ok(())
}

fn handle_doctor(cmd: DoctorCmd, cfg: &UiCfg, style: &Style) {
    let result = match cmd {
        DoctorCmd::Perms => doctor::run(doctor::DoctorArgs {
            command: doctor::DoctorCommand::Perms,
            _cfg: cfg,
            style,
        }),
        DoctorCmd::Repair => doctor::run(doctor::DoctorArgs {
            command: doctor::DoctorCommand::Repair,
            _cfg: cfg,
            style,
        }),
        DoctorCmd::Env => doctor::run(doctor::DoctorArgs {
            command: doctor::DoctorCommand::Env,
            _cfg: cfg,
            style,
        }),
    };

    if let Err(err) = result {
        eprintln!("{}", err);
        std::process::exit(err.code());
    }
}

fn handle_update(cmd: UpdateCmd, cfg: &UiCfg, style: &Style) {
    let result = match cmd {
        UpdateCmd::Check => update::run(
            update::UpdateArgs {
                command: update::UpdateCommand::Check,
                force: false,
                mode: None,
            },
            cfg,
            style,
        ),
        UpdateCmd::Apply { force } => update::run(
            update::UpdateArgs {
                command: update::UpdateCommand::Apply,
                force,
                mode: None,
            },
            cfg,
            style,
        ),
        UpdateCmd::Rollback => update::run(
            update::UpdateArgs {
                command: update::UpdateCommand::Rollback,
                force: false,
                mode: None,
            },
            cfg,
            style,
        ),
        UpdateCmd::Policy { mode } => update::run(
            update::UpdateArgs {
                command: update::UpdateCommand::Policy,
                force: false,
                mode: mode.as_deref(),
            },
            cfg,
            style,
        ),
    };

    if let Err(err) = result {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

fn handle_persona(cmd: PersonaCmd, cfg: &UiCfg, style: &Style) {
    let result = match cmd {
        PersonaCmd::Show { raw } => persona::run(
            persona::PersonaArgs {
                command: persona::PersonaCommand::Show,
                name: None,
                date: None,
                tail: None,
                raw,
            },
            cfg,
            style,
        ),
        PersonaCmd::Set { name } => persona::run(
            persona::PersonaArgs {
                command: persona::PersonaCommand::Set,
                name: Some(&name),
                date: None,
                tail: None,
                raw: false,
            },
            cfg,
            style,
        ),
        PersonaCmd::Unset => persona::run(
            persona::PersonaArgs {
                command: persona::PersonaCommand::Unset,
                name: None,
                date: None,
                tail: None,
                raw: false,
            },
            cfg,
            style,
        ),
        PersonaCmd::Samples { date, tail } => persona::run(
            persona::PersonaArgs {
                command: persona::PersonaCommand::Samples,
                name: None,
                date: Some(&date),
                tail: Some(tail),
                raw: false,
            },
            cfg,
            style,
        ),
        PersonaCmd::Rollup { date } => persona::run(
            persona::PersonaArgs {
                command: persona::PersonaCommand::Rollup,
                name: None,
                date: Some(&date),
                tail: None,
                raw: false,
            },
            cfg,
            style,
        ),
        PersonaCmd::Explain { raw } => persona::run(
            persona::PersonaArgs {
                command: persona::PersonaCommand::Explain,
                name: None,
                date: None,
                tail: None,
                raw,
            },
            cfg,
            style,
        ),
        PersonaCmd::Triggers { raw } => persona::run(
            persona::PersonaArgs {
                command: persona::PersonaCommand::Triggers,
                name: None,
                date: None,
                tail: None,
                raw,
            },
            cfg,
            style,
        ),
        PersonaCmd::Debug { raw } => persona::run(
            persona::PersonaArgs {
                command: persona::PersonaCommand::Debug,
                name: None,
                date: None,
                tail: None,
                raw,
            },
            cfg,
            style,
        ),
    };

    if let Err(err) = result {
        eprintln!("{}", err);
        std::process::exit(err.code());
    }
}

fn handle_advice(cmd: AdviceCmd, cfg: &UiCfg, style: &Style) {
    let result = match cmd {
        AdviceCmd::List { raw } => advice::run(
            advice::AdviceArgs {
                command: advice::AdviceCommand::List,
                id: None,
                dry_run: false,
                raw,
                all: false,
                force: false,
            },
            cfg,
            style,
        ),
        AdviceCmd::Show { id, raw } => advice::run(
            advice::AdviceArgs {
                command: advice::AdviceCommand::Show,
                id: Some(&id),
                dry_run: false,
                raw,
                all: false,
                force: false,
            },
            cfg,
            style,
        ),
        AdviceCmd::Apply {
            id,
            dry_run,
            raw,
            all,
            force,
        } => advice::run(
            advice::AdviceArgs {
                command: advice::AdviceCommand::Apply,
                id: id.as_deref(),
                dry_run,
                raw,
                all,
                force,
            },
            cfg,
            style,
        ),
    };

    if let Err(err) = result {
        eprintln!("{}", err);
        std::process::exit(err.code());
    }
}

fn handle_ui(cmd: UiCmd, cfg: &UiCfg, style: &Style) {
    match cmd {
        UiCmd::Preview => ui_preview(cfg, style),
    }
}

fn ui_preview(cfg: &UiCfg, style: &Style) {
    ui::banner(style, "UI Preview");

    println!("{}", ui::head(style, "Current Settings"));
    println!("{}", ui::kv(style, "Colors", &format!("{}", cfg.colors)));
    println!("{}", ui::kv(style, "Emojis", &format!("{}", cfg.emojis)));
    println!(
        "{}",
        ui::kv(
            style,
            "Theme",
            match cfg.theme {
                ui::Theme::Dark => "dark",
                ui::Theme::Light => "light",
            }
        )
    );
    println!(
        "{}",
        ui::kv(
            style,
            "Effective",
            &format!(
                "color={} emoji={} bold={}",
                style.color, style.emoji, style.bold
            )
        )
    );

    println!("\n{}", ui::head(style, "Style Samples"));
    println!("{}", ui::ok(style, "Success message"));
    println!("{}", ui::warn(style, "Warning message"));
    println!("{}", ui::err(style, "Error message"));
    println!("{}", ui::info(style, "Info message"));
    println!("{}", ui::note(style, "Note message"));
    println!("{}", ui::step(style, "Step message"));
    println!("{}", ui::bullet(style, "Bullet point"));

    println!("\n{}", ui::head(style, "Configuration"));
    let paths = paths::AnnaPaths::detect();
    let config_path = paths.config_dir.join("config.toml");
    println!(
        "{}",
        ui::kv(style, "Config file", &config_path.display().to_string())
    );

    if !cfg.colors || !cfg.emojis {
        println!("\n{}", ui::note(style, "To enable colors and emojis:"));
        println!(
            "{}",
            ui::bullet(style, &format!("Edit {} and add:", config_path.display()))
        );
        println!("  [ui]");
        println!("  colors = true");
        println!("  emojis = true");
    }

    if !ui::supports_emoji() {
        println!(
            "\n{}",
            ui::warn(
                style,
                "Terminal does not support UTF-8. Install a color emoji font like Noto Color Emoji."
            )
        );
    }
}

fn sysinfo_cmd(raw: bool, _ui_cfg: &UiCfg, style: &Style) -> Result<()> {
    let (contents, path) = read_snapshot()?;
    if raw {
        println!("{}", contents.trim());
        return Ok(());
    }

    match serde_json::from_str::<SystemSnapshot>(&contents) {
        Ok(snapshot) => {
            println!("{}", ui::head(style, "System Snapshot"));
            println!("{}", render_snapshot(&snapshot, style));
        }
        Err(_) => {
            println!(
                "{}",
                ui::err(
                    style,
                    &format!("system snapshot at {} is not valid JSON", path.display())
                )
            );
        }
    }
    Ok(())
}

fn read_snapshot() -> Result<(String, PathBuf)> {
    for path in snapshot_locations() {
        match fs::read_to_string(&path) {
            Ok(data) => return Ok((data, path)),
            Err(err) => {
                if err.kind() == std::io::ErrorKind::NotFound {
                    continue;
                }
            }
        }
    }
    Err(anyhow!("system snapshot not found"))
}

fn snapshot_locations() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(env_path) = env::var_os("ANNA_SYSTEM_JSON") {
        paths.push(PathBuf::from(env_path));
    }
    paths.push(PathBuf::from("/var/lib/anna/system.json"));
    if let Some(local) = dirs::data_local_dir() {
        paths.push(local.join("Anna/system.json"));
        paths.push(local.join("anna/system.json"));
    }
    paths
}

fn render_snapshot(snapshot: &SystemSnapshot, style: &Style) -> String {
    let mut out = String::new();
    out.push_str(&format!("{}\n", ui::kv(style, "OS", &snapshot.os)));
    out.push_str(&format!("{}\n", ui::kv(style, "Kernel", &snapshot.kernel)));
    out.push_str(&format!(
        "{}\n",
        ui::kv(style, "Uptime", &format_uptime(snapshot.uptime_secs))
    ));
    out.push_str(&format!(
        "{}\n",
        ui::kv(
            style,
            "CPU",
            &format!(
                "{} ({} logical core{})",
                snapshot.cpu_model,
                snapshot.cpu_cores_logical,
                if snapshot.cpu_cores_logical == 1 {
                    ""
                } else {
                    "s"
                }
            )
        )
    ));
    out.push_str(&format!(
        "{}\n",
        ui::kv(
            style,
            "Memory",
            &format_capacity(snapshot.total_memory_mb, snapshot.available_memory_mb)
        )
    ));
    let swap_text = if snapshot.total_swap_mb == 0 {
        "disabled".to_string()
    } else {
        format_capacity(snapshot.total_swap_mb, snapshot.free_swap_mb)
    };
    out.push_str(&format!("{}\n", ui::kv(style, "Swap", &swap_text)));
    if let Some(gpu) = snapshot.gpu_model.as_deref() {
        out.push_str(&format!("{}\n", ui::kv(style, "GPU", gpu)));
    }
    if let Some(audio) = snapshot.audio_server.as_deref() {
        out.push_str(&format!("{}\n", ui::kv(style, "Audio", audio)));
    }
    out.push_str(&format!(
        "{}\n",
        ui::kv(style, "Networking", &fmt_list(&snapshot.network_managers))
    ));
    let display_info = format!(
        "running: {} | available: {}",
        fmt_list(&snapshot.display_servers_running),
        fmt_list(&snapshot.display_servers_available)
    );
    out.push_str(&format!("{}\n", ui::kv(style, "Display", &display_info)));
    let boot_info = if snapshot.bootloaders.is_empty() {
        "unknown".to_string()
    } else {
        fmt_list(&snapshot.bootloaders)
    };
    out.push_str(&format!("{}\n", ui::kv(style, "Boot", &boot_info)));
    let dual_info = if snapshot.dual_boot {
        if snapshot.other_os_partitions.is_empty() {
            "yes".to_string()
        } else {
            format!("yes ({})", snapshot.other_os_partitions.join(", "))
        }
    } else {
        "no".to_string()
    };
    out.push_str(&format!("{}\n", ui::kv(style, "Dual boot", &dual_info)));

    if !snapshot.partition_table.is_empty() {
        out.push_str(&format!("\n{}\n", ui::head(style, "Partitions")));
        for part in &snapshot.partition_table {
            let mut line = format!("{} {}", part.name, part.size);
            if let Some(fs) = part.fstype.as_deref() {
                line.push_str(&format!(" [{fs}]"));
            }
            if let Some(label) = part.label.as_deref() {
                if !label.is_empty() {
                    line.push_str(&format!(" {label}"));
                }
            }
            if let Some(mount) = part.mountpoint.as_deref() {
                if !mount.is_empty() {
                    line.push_str(&format!(" → {}", mount));
                }
            }
            out.push_str(&format!("{}\n", ui::bullet(style, &line)));
        }
    } else if !snapshot.partitions.is_empty() {
        out.push_str(&format!("\n{}\n", ui::head(style, "Partitions")));
        for part in &snapshot.partitions {
            out.push_str(&format!("{}\n", ui::bullet(style, part)));
        }
    }

    out.trim_end().to_string()
}

fn format_uptime(secs: u64) -> String {
    let days = secs / 86_400;
    let hours = (secs % 86_400) / 3_600;
    let minutes = (secs % 3_600) / 60;
    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{} day{}", days, if days == 1 { "" } else { "s" }));
    }
    if hours > 0 {
        parts.push(format!(
            "{} hour{}",
            hours,
            if hours == 1 { "" } else { "s" }
        ));
    }
    if minutes > 0 {
        parts.push(format!(
            "{} minute{}",
            minutes,
            if minutes == 1 { "" } else { "s" }
        ));
    }
    if parts.is_empty() {
        parts.push("less than a minute".to_string());
    }
    parts.join(", ")
}

fn format_capacity(total: u64, available: u64) -> String {
    format!("{} MB total ({} MB available)", total, available)
}

fn fmt_list(items: &[String]) -> String {
    if items.is_empty() {
        "none".to_string()
    } else {
        items.join(", ")
    }
}
