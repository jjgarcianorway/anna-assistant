//! Full system report — comprehensive multi-section report from live commands.
//! Called when user asks for "full report", "complete report", "detailed status".
//! Runs real commands; does NOT use pre-cached LiveState.

use anyhow::Result;
use std::process::Command;

fn run(cmd: &str) -> String {
    Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

fn section(title: &str, content: &str) -> String {
    if content.is_empty() {
        return String::new();
    }
    format!("\n── {} ──\n{}\n", title, content)
}

/// Generate a comprehensive system report from live command output.
pub fn generate_full_report() -> Result<String> {
    let mut report = String::from("FULL SYSTEM REPORT\n");

    // --- Overview ---
    let hostname = run("hostname");
    let kernel = run("uname -r");
    let os = run("grep PRETTY_NAME /etc/os-release | cut -d'\"' -f2");
    let uptime = run("uptime -p");
    let boot = run("who -b | awk '{print $3, $4}'");
    let overview = format!(
        "Host:    {}\nOS:      {}\nKernel:  {}\nUptime:  {}\nBooted:  {}",
        hostname, os, kernel, uptime, boot
    );
    report.push_str(&section("OVERVIEW", &overview));

    // --- CPU ---
    let cpu_model = run("grep -m1 'model name' /proc/cpuinfo | cut -d':' -f2 | xargs");
    let cpu_cores = run("nproc");
    let load = run("cat /proc/loadavg | awk '{print $1, $2, $3}'");
    let cpu_freq = run("grep 'cpu MHz' /proc/cpuinfo | awk '{sum+=$4; n++} END {printf \"%.0f MHz avg\", sum/n}'");
    let cpu_info = format!(
        "Model:  {}\nCores:  {}\nLoad:   {} (1/5/15 min)\nFreq:   {}",
        cpu_model, cpu_cores, load, cpu_freq
    );
    report.push_str(&section("CPU", &cpu_info));

    // --- Memory ---
    let mem = run("free -h | awk 'NR==1{print} NR==2{print} NR==3{print}'");
    report.push_str(&section("MEMORY", &mem));

    // --- Disk ---
    let disk = run("df -hT | grep -v tmpfs | grep -v devtmpfs | grep -v udev");
    report.push_str(&section("DISK", &disk));

    // --- Services ---
    let failed = run("systemctl --failed --no-legend --no-pager 2>/dev/null");
    let failed_section = if failed.is_empty() {
        "All services running normally.".to_string()
    } else {
        format!("FAILED SERVICES:\n{}", failed)
    };
    report.push_str(&section("SERVICES", &failed_section));

    // --- Top Processes ---
    // Truncate command to 50 chars to prevent multi-hundred-char lines
    let top_cpu = run(r#"ps aux --sort=-%cpu | awk 'NR==1{printf "%-10s %5s %5s  %-50s\n","USER","%CPU","%MEM","COMMAND"} NR>1 && NR<=8{cmd=substr($11,1,50); for(i=12;i<=NF&&length(cmd)<50;i++) cmd=cmd" "$i; printf "%-10s %5s %5s  %-50s\n",$1,$3,$4,substr(cmd,1,50)}'"#);
    report.push_str(&section("TOP PROCESSES (CPU)", &top_cpu));

    let top_mem = run(r#"ps aux --sort=-%mem | awk 'NR>1 && NR<=6{cmd=substr($11,1,40); printf "%-10s %5s%%  %s\n",$1,$4,cmd}'"#);
    report.push_str(&section("TOP PROCESSES (MEM)", &top_mem));

    // --- Network ---
    let ifaces = run("ip -4 addr show | grep -E '^[0-9]+:|inet ' | sed 's/^/  /'");
    let gateway = run("ip route show default | awk '{print $3}'");
    let dns = run("grep '^nameserver' /etc/resolv.conf | awk '{print $2}' | head -3 | tr '\\n' ' '");
    let net = format!("Interfaces:\n{}\nGateway: {}\nDNS:     {}", ifaces, gateway, dns);
    report.push_str(&section("NETWORK", &net));

    // --- Temperature ---
    // Only current-reading lines (start with non-space, contain °C). Strip parenthetical
    // threshold annotations and the ALARM label — those are sensor chip flags, not a crisis.
    let temps = run(r#"sensors 2>/dev/null | grep '°C' | grep -v '^ ' | sed 's/  *(.*)//' | head -15"#);
    if !temps.is_empty() {
        report.push_str(&section("TEMPERATURE", &temps));
    }

    // --- Battery (if present) ---
    // xargs -r skips upower -i if no battery device found (avoids empty-arg error)
    let battery = run("upower -e 2>/dev/null | grep battery | xargs -r upower -i 2>/dev/null | grep -E 'state:|percentage:|time to' | sed 's/^[ \\t]*//'");
    if !battery.is_empty() {
        report.push_str(&section("BATTERY", &battery));
    }

    // --- Recent Errors (deduplicated) ---
    // Fetch 100, deduplicate by message text (fields 5+), keep first occurrence of each.
    // This collapses repeated entries like 10x identical flatpak timeout lines into one.
    let errors = run(r#"journalctl -p err -b --no-pager -n 100 --output=short 2>/dev/null | awk 'NR==1{next}{msg=""; for(i=5;i<=NF;i++) msg=msg" "$i; if(!seen[msg]++){print}}' | head -10"#);
    let error_section = if errors.trim().is_empty() || errors.contains("-- No entries --") {
        "No errors in current boot.".to_string()
    } else {
        errors
    };
    report.push_str(&section("RECENT ERRORS (this boot, deduplicated)", &error_section));

    // --- Packages ---
    let pkg_count = run("pacman -Q 2>/dev/null | wc -l");
    let upgradable = run("checkupdates 2>/dev/null | wc -l");
    let last_update = run("grep 'upgraded' /var/log/pacman.log 2>/dev/null | tail -1 | cut -d'[' -f2 | cut -d']' -f1");
    if !pkg_count.is_empty() {
        let pkg = format!(
            "Installed:  {} packages\nUpgradable: {}\nLast update: {}",
            pkg_count,
            if upgradable.is_empty() { "unknown (checkupdates not available)".to_string() } else { upgradable },
            if last_update.is_empty() { "unknown".to_string() } else { last_update }
        );
        report.push_str(&section("PACKAGES", &pkg));
    }

    // --- Anna Registry ---
    let registry = crate::artifact_registry::ArtifactRegistry::load();
    let reg_summary = registry.summary_for_briefing();
    if !reg_summary.is_empty() {
        report.push_str(&section("ANNA MANAGED", &reg_summary));
    }

    Ok(report)
}

/// Detect if the question is asking for a full/detailed/complete system report.
pub fn is_full_report_request(question: &str) -> bool {
    let q = question.to_lowercase();
    let report_words = ["report", "status", "overview", "summary"];
    let has_report = report_words.iter().any(|w| q.contains(w));

    // "full" and "complete" alone are specific enough — "full report", "complete status"
    let has_simple_full = ["full", "complete"].iter().any(|w| {
        let padded = format!(" {} ", q);
        padded.contains(&format!(" {} ", w)) || q.starts_with(&format!("{} ", w))
    });

    // "detailed", "comprehensive", "everything" require "system" nearby to avoid false positives
    // e.g. "detailed summary of neovim" must NOT trigger a system report
    let has_qualified_full = ["detailed", "comprehensive", "everything"]
        .iter()
        .any(|w| q.contains(w))
        && q.contains("system");

    has_report && (has_simple_full || has_qualified_full)
}
