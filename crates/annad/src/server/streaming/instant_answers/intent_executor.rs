//! LLM-based intent classification replacing keyword pattern matching.
//! Classifies the question in ≤8s, then runs direct system commands.

use anyhow::Result;
use tracing::debug;
use super::super::instant_answers::{run_cmd, run_cmd_cached, run_shell, send_answer};
use crate::cache::InvalidationTag;
use crate::state::SharedState;

// ── Intent variants ──────────────────────────────────────────────────────────

#[derive(Debug, PartialEq)]
enum ActionIntent {
    DiskUsage, DiskInfo, RamInfo, RamConsumers,
    CpuModel, CpuTemp, CpuUsage, CpuThrottle,
    SystemInfo, NetworkInfo, FirewallStatus, HealthCheck,
    ServiceStatus, BootInfo, PackageInfo, SystemUpdate,
    SecurityInfo, SystemLogs, KernelConfig, ProcessInfo,
    Unknown,
}

impl ActionIntent {
    fn parse(s: &str) -> Self {
        for word in s.split_whitespace() {
            let trimmed = word.trim_matches(|c: char| !c.is_alphanumeric());
            match trimmed {
                "DiskUsage"     => return Self::DiskUsage,
                "DiskInfo"      => return Self::DiskInfo,
                "RamInfo"       => return Self::RamInfo,
                "RamConsumers"  => return Self::RamConsumers,
                "CpuModel"      => return Self::CpuModel,
                "CpuTemp"       => return Self::CpuTemp,
                "CpuUsage"      => return Self::CpuUsage,
                "CpuThrottle"   => return Self::CpuThrottle,
                "SystemInfo"    => return Self::SystemInfo,
                "NetworkInfo"   => return Self::NetworkInfo,
                "FirewallStatus"=> return Self::FirewallStatus,
                "HealthCheck"   => return Self::HealthCheck,
                "ServiceStatus" => return Self::ServiceStatus,
                "BootInfo"      => return Self::BootInfo,
                "PackageInfo"   => return Self::PackageInfo,
                "SystemUpdate"  => return Self::SystemUpdate,
                "SecurityInfo"  => return Self::SecurityInfo,
                "SystemLogs"    => return Self::SystemLogs,
                "KernelConfig"  => return Self::KernelConfig,
                "ProcessInfo"   => return Self::ProcessInfo,
                _ => {}
            }
        }
        Self::Unknown
    }
}

// ── Classification prompt ────────────────────────────────────────────────────

const CLASSIFY_PROMPT: &str = r#"Classify this Linux system query. Reply with ONLY ONE WORD from the list.

DiskUsage     - disk space, storage usage, df, how much disk space
DiskInfo      - disk type SSD/HDD, partitions, SMART health, TRIM, filesystems mounted
RamInfo       - total RAM amount, how much memory installed
RamConsumers  - what's using RAM, top memory processes
CpuModel      - CPU model, processor name, cores count
CpuTemp       - CPU/system temperature, thermal sensors
CpuUsage      - top CPU processes, what's consuming CPU right now
CpuThrottle   - CPU throttling, frequency scaling, governor
SystemInfo     - hostname, distro, kernel version, OS, architecture, uptime, who am I, load average
NetworkInfo    - IP address, interfaces, routing table, open ports, DNS servers
FirewallStatus - firewall rules, iptables, nftables, ufw status
HealthCheck    - how is my system, system health, status overview, how am I doing, everything ok
ServiceStatus  - failed services, systemd units, service failures
BootInfo       - boot time analysis, boot logs, startup services, recent reboots
PackageInfo    - recently installed packages, orphaned packages, package errors
SystemUpdate   - update system, upgrade packages, arch-update, paru, yay, pending updates, available updates, check for updates
SecurityInfo   - login attempts, suspicious processes, file permissions, sensitive files
SystemLogs     - system errors in journal, kernel panic, recent error logs
KernelConfig   - kernel parameters, sysctl tuning, kernel settings
ProcessInfo    - process tree, running processes, parent process, started by
Unknown        - anything not in the list above

Query: "{question}"

Reply with exactly one word."#;

// ── Entry point ──────────────────────────────────────────────────────────────

pub async fn classify_and_execute(
    question: &str,
    writer: &mut tokio::net::unix::OwnedWriteHalf,
    state: &SharedState,
) -> Result<bool> {
    let model = {
        let guard = state.read().await;
        match guard.model.clone() {
            Some(m) => m,
            None => return Ok(false),
        }
    };

    let prompt = CLASSIFY_PROMPT.replace("{question}", question);
    let response = match crate::ollama::chat_with_timeout(&model, &prompt, 8).await {
        Ok(r) => r,
        Err(e) => {
            debug!("Intent classification failed: {}", e);
            return Ok(false);
        }
    };

    let intent = ActionIntent::parse(&response);
    debug!("Intent '{}' → {:?}", question, intent);

    if intent == ActionIntent::Unknown {
        return Ok(false);
    }

    let cache = {
        let guard = state.read().await;
        guard.cache.clone()
    };

    match intent {
        ActionIntent::DiskUsage     => exec_disk_usage(writer, &cache).await,
        ActionIntent::DiskInfo      => exec_disk_info(writer).await,
        ActionIntent::RamInfo       => exec_ram_info(writer, &cache).await,
        ActionIntent::RamConsumers  => exec_ram_consumers(writer).await,
        ActionIntent::CpuModel      => exec_cpu_model(writer).await,
        ActionIntent::CpuTemp       => exec_cpu_temp(writer).await,
        ActionIntent::CpuUsage      => exec_cpu_usage(writer).await,
        ActionIntent::CpuThrottle   => exec_cpu_throttle(writer).await,
        ActionIntent::SystemInfo    => exec_system_info(writer, &cache).await,
        ActionIntent::NetworkInfo   => exec_network_info(writer, &cache).await,
        ActionIntent::FirewallStatus=> exec_firewall(writer).await,
        ActionIntent::HealthCheck   => exec_health_check(writer, &cache).await,
        ActionIntent::ServiceStatus => exec_services(writer, &cache).await,
        ActionIntent::BootInfo      => exec_boot_info(writer).await,
        ActionIntent::PackageInfo   => exec_packages(writer).await,
        ActionIntent::SystemUpdate  => exec_update(writer, state).await,
        ActionIntent::SecurityInfo  => exec_security(writer).await,
        ActionIntent::SystemLogs    => exec_logs(writer).await,
        ActionIntent::KernelConfig  => exec_kernel_config(writer).await,
        ActionIntent::ProcessInfo   => exec_processes(writer).await,
        ActionIntent::Unknown       => Ok(false),
    }
}

// ── Executors ────────────────────────────────────────────────────────────────

async fn exec_disk_usage(writer: &mut tokio::net::unix::OwnedWriteHalf, cache: &crate::cache::SystemCache) -> Result<bool> {
    let out = run_cmd_cached(cache, "df_usage", "df", &["-h", "--output=source,size,used,avail,pcent,target"], 60, &[InvalidationTag::Fstab])?;
    send_answer(writer, format!("Disk space:\n```\n{}\n```", out.trim())).await?;
    Ok(true)
}

async fn exec_disk_info(writer: &mut tokio::net::unix::OwnedWriteHalf) -> Result<bool> {
    let lsblk = run_cmd("lsblk", &["-o", "NAME,SIZE,TYPE,ROTA,MOUNTPOINT"])?;
    let smart = run_shell(
        "for d in $(lsblk -d -o NAME --noheadings | head -4); do \
         smartctl -H /dev/$d 2>/dev/null | grep 'SMART overall' | sed \"s/^/$d: /\"; done"
    ).unwrap_or_default();
    let trim = run_cmd("systemctl", &["is-active", "fstrim.timer"]).unwrap_or_default();
    let mut out = format!("Disks (ROTA=0:SSD, ROTA=1:HDD):\n```\n{}\n```", lsblk.trim());
    if !smart.trim().is_empty() {
        out.push_str(&format!("\nSMART: {}", smart.trim()));
    }
    out.push_str(&format!("\nTRIM timer: {}", trim.trim()));
    send_answer(writer, out).await?;
    Ok(true)
}

async fn exec_ram_info(writer: &mut tokio::net::unix::OwnedWriteHalf, cache: &crate::cache::SystemCache) -> Result<bool> {
    let out = run_cmd_cached(cache, "free_memory", "free", &["-h"], 15, &[InvalidationTag::Memory])?;
    let mem = out.lines().find(|l| l.starts_with("Mem:")).unwrap_or("");
    let f: Vec<&str> = mem.split_whitespace().collect();
    send_answer(writer, format!("RAM: {} total, {} used.", f.get(1).unwrap_or(&"?"), f.get(2).unwrap_or(&"?"))).await?;
    Ok(true)
}

async fn exec_ram_consumers(writer: &mut tokio::net::unix::OwnedWriteHalf) -> Result<bool> {
    let out = run_shell("ps aux --sort=-%mem | head -11")?;
    send_answer(writer, format!("Top memory consumers:\n```\n{}\n```", out.trim())).await?;
    Ok(true)
}

async fn exec_cpu_model(writer: &mut tokio::net::unix::OwnedWriteHalf) -> Result<bool> {
    let cpuinfo = std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
    let model = cpuinfo.lines().find(|l| l.starts_with("model name"))
        .and_then(|l| l.split(':').nth(1)).map(|s| s.trim()).unwrap_or("Unknown CPU");
    let cores = cpuinfo.lines().filter(|l| l.starts_with("processor")).count();
    send_answer(writer, format!("CPU: {} ({} cores)", model, cores)).await?;
    Ok(true)
}

async fn exec_cpu_temp(writer: &mut tokio::net::unix::OwnedWriteHalf) -> Result<bool> {
    let temps = run_shell("sensors 2>/dev/null | grep -E 'Core|Tdie|Tctl|Package' | head -8").unwrap_or_default();
    let msg = if temps.trim().is_empty() {
        "Temperature sensors not available (lm_sensors not installed or not configured).".to_string()
    } else {
        format!("CPU temperatures:\n```\n{}\n```", temps.trim())
    };
    send_answer(writer, msg).await?;
    Ok(true)
}

async fn exec_cpu_usage(writer: &mut tokio::net::unix::OwnedWriteHalf) -> Result<bool> {
    let out = run_shell("ps aux --sort=-%cpu | head -11")?;
    send_answer(writer, format!("Top CPU consumers:\n```\n{}\n```", out.trim())).await?;
    Ok(true)
}

async fn exec_cpu_throttle(writer: &mut tokio::net::unix::OwnedWriteHalf) -> Result<bool> {
    let gov = run_shell("cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null || echo unknown")?;
    let cur = run_shell("cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq 2>/dev/null || echo 0")?;
    let max = run_shell("cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_max_freq 2>/dev/null || echo 0")?;
    let cur_mhz = cur.trim().parse::<f64>().unwrap_or(0.0) / 1000.0;
    let max_mhz = max.trim().parse::<f64>().unwrap_or(0.0) / 1000.0;
    let status = if max_mhz > 0.0 && cur_mhz < max_mhz * 0.8 {
        format!("CPU throttling: {:.0}MHz of {:.0}MHz ({:.0}%)", cur_mhz, max_mhz, cur_mhz / max_mhz * 100.0)
    } else {
        format!("CPU not throttling: {:.0}MHz / {:.0}MHz max", cur_mhz, max_mhz)
    };
    send_answer(writer, format!("{}\nGovernor: {}", status, gov.trim())).await?;
    Ok(true)
}

async fn exec_system_info(writer: &mut tokio::net::unix::OwnedWriteHalf, cache: &crate::cache::SystemCache) -> Result<bool> {
    let identity = crate::system_identity::get_system_identity();
    let kernel = run_cmd_cached(cache, "uname_kernel", "uname", &["-r"], 3600, &[InvalidationTag::Bootloader])?;
    let uptime = run_cmd("uptime", &["-p"]).unwrap_or_default();
    let user = crate::user_context::get_real_user().unwrap_or_else(|_| "unknown".to_string());
    let load = run_cmd("uptime", &[]).unwrap_or_default();
    send_answer(writer, format!(
        "System: {}\nHostname: {}\nKernel: {}\nUptime: {}\nUser: {}\nLoad: {}",
        identity.distro_name, identity.hostname, kernel.trim(),
        uptime.trim(), user, load.trim()
    )).await?;
    Ok(true)
}

async fn exec_network_info(writer: &mut tokio::net::unix::OwnedWriteHalf, cache: &crate::cache::SystemCache) -> Result<bool> {
    let ip_raw = run_cmd_cached(cache, "ip_addr", "ip", &["addr", "show"], 30, &[InvalidationTag::Network])?;
    let my_ip = ip_raw.lines().find(|l| l.contains("inet ") && !l.contains("127.0.0.1"))
        .and_then(|l| l.split_whitespace().nth(1)).unwrap_or("No IP found");
    let ports = run_cmd_cached(cache, "ss_ports", "ss", &["-tulpn"], 30, &[InvalidationTag::Network])?;
    let dns = run_shell("grep ^nameserver /etc/resolv.conf 2>/dev/null | head -3").unwrap_or_default();
    send_answer(writer, format!("IP: {}\nDNS: {}\nOpen ports:\n```\n{}\n```", my_ip, dns.trim(), ports.trim())).await?;
    Ok(true)
}

async fn exec_health_check(writer: &mut tokio::net::unix::OwnedWriteHalf, cache: &crate::cache::SystemCache) -> Result<bool> {
    let uptime = run_cmd("uptime", &[]).unwrap_or_default();
    let mem = run_cmd_cached(cache, "free_memory", "free", &["-h"], 15, &[InvalidationTag::Memory]).unwrap_or_default();
    let mem_line = mem.lines().find(|l| l.starts_with("Mem:")).unwrap_or("");
    let mf: Vec<&str> = mem_line.split_whitespace().collect();
    let disk = run_shell("df -h / 2>/dev/null | tail -1").unwrap_or_default();
    let failed = run_cmd("systemctl", &["--failed", "--no-pager"]).unwrap_or_default();
    let failed_count = failed.lines().filter(|l| l.contains("failed")).count();
    let updates: u32 = run_shell("checkupdates 2>/dev/null | wc -l || echo 0")
        .unwrap_or_default().trim().parse().unwrap_or(0);
    let update_msg = if updates == 0 { "up to date".to_string() } else { format!("{} updates available", updates) };
    let health = if failed_count == 0 { "healthy" } else { "issues detected" };
    send_answer(writer, format!(
        "System status: {}\n\nLoad: {}\nRAM: {} total, {} used\nDisk (/): {}\nFailed services: {}\nUpdates: {}",
        health, uptime.trim(),
        mf.get(1).unwrap_or(&"?"), mf.get(2).unwrap_or(&"?"),
        disk.trim(), failed_count, update_msg
    )).await?;
    Ok(true)
}

async fn exec_firewall(writer: &mut tokio::net::unix::OwnedWriteHalf) -> Result<bool> {
    let nft = run_shell("nft list ruleset 2>/dev/null | head -40").unwrap_or_default();
    let ipt = run_shell("iptables -nL --line-numbers 2>/dev/null | head -40").unwrap_or_default();
    let ufw = run_shell("ufw status 2>/dev/null").unwrap_or_default();
    let answer = if !nft.trim().is_empty() {
        format!("Firewall (nftables):\n```\n{}\n```", nft.trim())
    } else if !ipt.trim().is_empty() {
        format!("Firewall (iptables):\n```\n{}\n```", ipt.trim())
    } else if !ufw.trim().is_empty() {
        format!("Firewall (ufw):\n```\n{}\n```", ufw.trim())
    } else {
        "No active firewall detected.".to_string()
    };
    send_answer(writer, answer).await?;
    Ok(true)
}

async fn exec_services(writer: &mut tokio::net::unix::OwnedWriteHalf, cache: &crate::cache::SystemCache) -> Result<bool> {
    let out = run_cmd_cached(cache, "systemctl_failed", "systemctl", &["--failed", "--no-pager"], 30, &[InvalidationTag::Services])?;
    send_answer(writer, format!("Failed services:\n```\n{}\n```", out.trim())).await?;
    Ok(true)
}

async fn exec_boot_info(writer: &mut tokio::net::unix::OwnedWriteHalf) -> Result<bool> {
    let analyze = run_cmd("systemd-analyze", &[]).unwrap_or_default();
    let blame = run_cmd("systemd-analyze", &["blame"]).unwrap_or_default();
    let top = blame.lines().take(10).collect::<Vec<_>>().join("\n");
    let reboots = run_shell("last reboot | head -5").unwrap_or_default();
    send_answer(writer, format!(
        "{}\n\nTop boot contributors:\n{}\n\nRecent reboots:\n{}",
        analyze.trim(), top, reboots.trim()
    )).await?;
    Ok(true)
}

async fn exec_packages(writer: &mut tokio::net::unix::OwnedWriteHalf) -> Result<bool> {
    let recent = run_shell("tail -20 /var/log/pacman.log 2>/dev/null | grep -E 'installed|upgraded' || tail -20 /var/log/dpkg.log 2>/dev/null | grep 'install '").unwrap_or_default();
    let orphans = run_cmd("pacman", &["-Qdt"]).unwrap_or_default();
    let orphan_msg = if orphans.trim().is_empty() {
        "No orphaned packages.".to_string()
    } else {
        format!("Orphaned packages:\n```\n{}\n```", orphans.trim())
    };
    send_answer(writer, format!("Recent packages:\n```\n{}\n```\n\n{}", recent.trim(), orphan_msg)).await?;
    Ok(true)
}

async fn exec_update(writer: &mut tokio::net::unix::OwnedWriteHalf, state: &SharedState) -> Result<bool> {
    let username = crate::user_context::get_real_user().unwrap_or_else(|_| "root".to_string());
    let count: u32 = run_shell("checkupdates 2>/dev/null | wc -l || pacman -Qu 2>/dev/null | wc -l")
        .unwrap_or_default().trim().parse().unwrap_or(0);

    // No updates available
    if count == 0 {
        let aur = detect_aur_helper();
        let aur_msg = if let Some(ref h) = aur {
            let n: u32 = run_shell(&format!("runuser -l {} -c '{} -Qu 2>/dev/null | wc -l'", username, h))
                .unwrap_or_default().trim().parse().unwrap_or(0);
            if n == 0 { "No AUR updates.".to_string() } else { format!("{} AUR updates available.", n) }
        } else { String::new() };
        let msg = if aur_msg.is_empty() { "System is up to date.".to_string() }
                  else { format!("Official packages up to date. {}", aur_msg) };
        send_answer(writer, msg).await?;
        return Ok(true);
    }

    // arch-update if available
    if run_shell("which arch-update 2>/dev/null").map(|s| !s.trim().is_empty()).unwrap_or(false) {
        let out = run_shell(&format!("runuser -l {} -c 'arch-update 2>&1 | tail -30'", username)).unwrap_or_default();
        send_answer(writer, format!("arch-update:\n```\n{}\n```", out.trim())).await?;
        return Ok(true);
    }

    let _ = state; // state not needed beyond model which we already have
    let pacman = run_shell("pacman -Syu --noconfirm 2>&1 | tail -20").unwrap_or_default();
    let aur = detect_aur_helper();
    let aur_out = aur.as_ref().map(|h| {
        run_shell(&format!("runuser -l {} -c '{} -Syu --noconfirm 2>&1 | tail -20'", username, h)).unwrap_or_default()
    }).unwrap_or_default();

    let mut parts = vec![format!("pacman -Syu:\n```\n{}\n```", pacman.trim())];
    if !aur_out.trim().is_empty() {
        parts.push(format!("{} -Syu:\n```\n{}\n```", aur.as_deref().unwrap_or("AUR"), aur_out.trim()));
    }
    send_answer(writer, parts.join("\n\n")).await?;
    Ok(true)
}

async fn exec_security(writer: &mut tokio::net::unix::OwnedWriteHalf) -> Result<bool> {
    let attempts = run_cmd("lastb", &["-20"]).unwrap_or_default();
    let count = attempts.lines().filter(|l| !l.trim().is_empty() && !l.starts_with("btmp")).count();
    let login_msg = if count == 0 { "No recent failed login attempts.".to_string() }
                    else { format!("{} failed login attempts:\n```\n{}\n```", count, attempts.trim()) };
    let perms = run_shell("stat -c '%a %n' /etc/passwd /etc/shadow /etc/sudoers /root 2>/dev/null").unwrap_or_default();
    let suspicious = run_shell("ps aux | awk '$3 > 50 || $4 > 50 {print}' | head -5").unwrap_or_default();
    let susp_msg = if suspicious.trim().is_empty() { "No high-resource processes.".to_string() }
                   else { format!("High-resource processes:\n```\n{}\n```", suspicious.trim()) };
    send_answer(writer, format!("{}\n\nSensitive file permissions:\n{}\n\n{}", login_msg, perms.trim(), susp_msg)).await?;
    Ok(true)
}

async fn exec_logs(writer: &mut tokio::net::unix::OwnedWriteHalf) -> Result<bool> {
    let errors = run_cmd("journalctl", &["-p", "err", "-n", "20", "--no-pager"])?;
    let panic = run_shell("journalctl -b -1 --no-pager 2>/dev/null | grep -i 'panic\\|oops\\|segfault' | tail -5").unwrap_or_default();
    let extra = if panic.trim().is_empty() { String::new() } else { format!("\n\nKernel panic/oops:\n```\n{}\n```", panic.trim()) };
    send_answer(writer, format!("Recent system errors:\n```\n{}\n```{}", errors.trim(), extra)).await?;
    Ok(true)
}

async fn exec_kernel_config(writer: &mut tokio::net::unix::OwnedWriteHalf) -> Result<bool> {
    let params = run_shell(
        "sysctl -a 2>/dev/null | grep -E 'vm\\.(swappiness|dirty)|net\\.core\\.(rmem|wmem)|net\\.ipv4\\.(tcp_rmem|tcp_wmem|tcp_congestion)' | head -15"
    ).unwrap_or_default();
    send_answer(writer, format!("Performance-relevant kernel parameters:\n```\n{}\n```", params.trim())).await?;
    Ok(true)
}

async fn exec_processes(writer: &mut tokio::net::unix::OwnedWriteHalf) -> Result<bool> {
    let out = run_cmd("ps", &["auxf"])?;
    let top = out.lines().take(30).collect::<Vec<_>>().join("\n");
    send_answer(writer, format!("Process tree:\n```\n{}\n```", top)).await?;
    Ok(true)
}

fn detect_aur_helper() -> Option<String> {
    ["paru", "yay", "pikaur", "trizen", "aurman"].iter()
        .find(|&&h| run_shell(&format!("which {} 2>/dev/null", h))
            .map(|s| !s.trim().is_empty()).unwrap_or(false))
        .map(|&h| h.to_string())
}
