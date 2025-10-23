use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;

use crate::ui::{self, Style, UiCfg};

pub fn run(style: &Style, _cfg: &UiCfg) -> Result<()> {
    println!("{}", ui::head(style, "Anna status"));
    println!(
        "{}",
        ui::kv(
            style,
            "status",
            if is_active()? { "active" } else { "inactive" }
        )
    );
    let dir = data_dir();
    println!("{}", ui::kv(style, "data_dir", &dir.display().to_string()));

    println!("\n{}", ui::head(style, "Last 5 journal entries"));
    let logs = fetch_journal()?;
    if logs.trim().is_empty() {
        println!("(no recent logs)");
    } else {
        println!("{}", logs.trim_end());
    }
    Ok(())
}

fn is_active() -> Result<bool> {
    let active = Command::new("systemctl")
        .args(["is-active", "--quiet", "annad"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    Ok(active)
}

fn data_dir() -> PathBuf {
    let sys = PathBuf::from("/var/lib/anna");
    if sys.exists() {
        return sys;
    }
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Anna")
}

fn fetch_journal() -> Result<String> {
    let since = Command::new("bash")
        .arg("-c")
        .arg("systemctl show -p ActiveEnterTimestamp annad | cut -d= -f2")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "1 hour ago".to_string());

    let output = Command::new("journalctl")
        .args(["-u", "annad", "--since", &since, "-n", "5", "--no-pager"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();

    Ok(output)
}
