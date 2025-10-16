use anyhow::Result;
use clap::{Parser, Subcommand};
use std::{fs, path::PathBuf, process::Command};

#[derive(Parser)]
#[command(name = "annactl", about = "CLI for Anna")]
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
        /// Print compact JSON instead of pretty
        #[arg(long)]
        raw: bool,
    },
    /// Stub for plans
    Plans,
    /// Stub for showing a plan
    Show { id: String },
}

fn data_dir() -> PathBuf {
    let sys = PathBuf::from("/var/lib/anna");
    if sys.exists() {
        sys
    } else {
        dirs::data_local_dir()
            .unwrap_or(PathBuf::from("."))
            .join("Anna")
    }
}

fn status_cmd() -> Result<()> {
    let active = Command::new("systemctl")
        .args(["is-active", "--quiet", "annad"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    println!(
        "status: {}\n",
        if active {
            "\x1b[1;32mactive\x1b[0m"
        } else {
            "\x1b[1;31minactive\x1b[0m"
        }
    );

    println!("data_dir={}\n", data_dir().display());

    // logs since last start, in system’s native format
    let since_output = Command::new("bash")
        .arg("-c")
        .arg("systemctl show -p ActiveEnterTimestamp annad | cut -d= -f2")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_else(|| "1 hour ago".into());
    let since_trimmed = since_output.trim();

    let logs = Command::new("journalctl")
        .args(["-u", "annad", "--since", since_trimmed, "--no-pager"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_else(|| "No logs found.".to_string());

    println!("{}", logs.trim());
    Ok(())
}

fn sysinfo_cmd(raw: bool) -> Result<()> {
    let path = PathBuf::from("/var/lib/anna/system.json");
    let s = fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("system snapshot not found at {}: {e}", path.display()))?;
    if raw {
        println!("{}", s.trim());
        return Ok(());
    }
    match serde_json::from_str::<serde_json::Value>(&s) {
        Ok(v) => println!("{}", serde_json::to_string_pretty(&v)?),
        Err(_) => println!("{}", s.trim()),
    }
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Status => status_cmd()?,
        Cmd::Sysinfo { raw } => sysinfo_cmd(raw)?,
        Cmd::Plans => println!("(stub)"),
        Cmd::Show { id } => println!("Plan {}", id),
    }
    Ok(())
}
