//! Instant answers: system identity, hardware, storage, users.

use anyhow::Result;
use super::super::instant_answers::{run_cmd, run_cmd_cached, run_shell, send_answer};
use crate::cache::InvalidationTag;
use crate::state::SharedState;

pub async fn try_system_answer(
    question: &str,
    writer: &mut tokio::net::unix::OwnedWriteHalf,
    state: &SharedState,
) -> Result<bool> {
    let q = question.to_lowercase();

    // Get cache reference
    let cache = {
        let state_guard = state.read().await;
        state_guard.cache.clone()
    };

    // --- USER IDENTITY ---
    if (q.contains("what user") || q.contains("which user") || q.contains("who am i"))
        && (q.contains("am i") || q.contains("is this"))
    {
        let user = crate::user_context::get_real_user().unwrap_or_else(|_| "unknown".to_string());
        send_answer(writer, format!("You are the user `{}`.", user)).await?;
        return Ok(true);
    }

    // --- HOSTNAME ---
    if q.contains("hostname") && (q.contains("what") || q.contains("my")) {
        let identity = crate::system_identity::get_system_identity();
        send_answer(writer, format!("Your hostname is `{}`.", identity.hostname)).await?;
        return Ok(true);
    }

    // --- LINUX DISTRIBUTION ---
    if (q.contains("linux") || q.contains("distribution") || q.contains("distro"))
        && (q.contains("version") || q.contains("running") || q.contains("what"))
    {
        let identity = crate::system_identity::get_system_identity();
        send_answer(writer, format!("You are running {}.", identity.distro_name)).await?;
        return Ok(true);
    }

    // --- KERNEL VERSION ---
    if q.contains("kernel") && q.contains("version") {
        let kernel = run_cmd_cached(&cache, "uname_kernel", "uname", &["-r"], 3600, &[InvalidationTag::Bootloader])?;
        send_answer(writer, format!("Kernel version: {}", kernel.trim())).await?;
        return Ok(true);
    }

    // --- SYSTEM ARCHITECTURE ---
    if q.contains("architecture") || (q.contains("system") && q.contains("using") && q.contains("x86")) {
        let arch = run_cmd("uname", &["-m"])?;
        send_answer(writer, format!("System architecture: {}", arch.trim())).await?;
        return Ok(true);
    }

    // --- IP ADDRESS ---
    if (q.contains("ip") || q.contains("ip address"))
        && (q.contains("current") || q.contains("my") || q.contains("what"))
        && !q.contains("static")
    {
        let output = run_cmd_cached(&cache, "ip_addr", "ip", &["addr", "show"], 30, &[InvalidationTag::Network])?;
        let ip = output
            .lines()
            .find(|l| l.contains("inet ") && !l.contains("127.0.0.1"))
            .and_then(|l| l.split_whitespace().nth(1))
            .unwrap_or("No IP found");
        send_answer(writer, format!("Your IP address: {}", ip)).await?;
        return Ok(true);
    }

    // --- RAM ---
    if (q.contains("ram") || q.contains("memory"))
        && (q.contains("how much") || q.contains("total") || q.contains("have"))
        && !q.contains("using") && !q.contains("consuming") && !q.contains("most")
    {
        let output = run_cmd_cached(&cache, "free_memory", "free", &["-h"], 15, &[InvalidationTag::Memory])?;
        let mem = output.lines().find(|l| l.starts_with("Mem:")).unwrap_or("");
        let f: Vec<&str> = mem.split_whitespace().collect();
        let total = f.get(1).unwrap_or(&"?");
        let used = f.get(2).unwrap_or(&"?");
        send_answer(writer, format!("RAM: {} total, {} used.", total, used)).await?;
        return Ok(true);
    }

    // --- TOP RAM CONSUMERS ---
    if (q.contains("ram") || q.contains("memory"))
        && (q.contains("using") || q.contains("consuming") || q.contains("most"))
    {
        let output = run_shell("ps aux --sort=-%mem | head -11")?;
        send_answer(writer, format!("Top memory consumers:\n```\n{}\n```", output.trim())).await?;
        return Ok(true);
    }

    // --- DISK SPACE ---
    if q.contains("disk space") || (q.contains("disk") && q.contains("free"))
        || (q.contains("how much") && q.contains("disk"))
    {
        let output = run_cmd_cached(&cache, "df_usage", "df", &["-h", "--output=source,size,used,avail,pcent,target"], 60, &[InvalidationTag::Fstab])?;
        send_answer(writer, format!("Disk space:\n```\n{}\n```", output.trim())).await?;
        return Ok(true);
    }

    // --- DISKS CONNECTED ---
    if (q.contains("disk") || q.contains("drive"))
        && (q.contains("connected") || q.contains("what") || q.contains("list"))
        && !q.contains("space") && !q.contains("free") && !q.contains("ssd") && !q.contains("hdd")
    {
        let output = run_cmd_cached(&cache, "lsblk_devices", "lsblk", &["-d", "-o", "NAME,SIZE,TYPE"], 300, &[InvalidationTag::BlockDevice])?;
        send_answer(writer, format!("Your disks:\n```\n{}\n```", output.trim())).await?;
        return Ok(true);
    }

    // --- ROOT FILESYSTEM DISK ---
    if q.contains("root") && (q.contains("filesystem") || q.contains("disk") || q.contains("on")) {
        let output = run_cmd("findmnt", &["/", "-o", "SOURCE,TARGET,FSTYPE,SIZE,USED"])?;
        send_answer(writer, format!("Root filesystem:\n```\n{}\n```", output.trim())).await?;
        return Ok(true);
    }

    // --- SSD OR HDD ---
    if q.contains("ssd") || q.contains("hdd") || q.contains("solid state")
        || (q.contains("disk") && q.contains("type"))
    {
        let output = run_cmd("lsblk", &["-d", "-o", "NAME,ROTA,SIZE"])?;
        let mut result = vec!["Disk types (ROTA=0: SSD, ROTA=1: HDD):".to_string()];
        for line in output.lines().skip(1) {
            let p: Vec<&str> = line.split_whitespace().collect();
            if p.len() >= 2 {
                let t = if p[1] == "0" { "SSD" } else { "HDD" };
                result.push(format!("{}: {} ({})", p[0], t, p.get(2).unwrap_or(&"")));
            }
        }
        send_answer(writer, result.join("\n")).await?;
        return Ok(true);
    }

    // --- PARTITIONS ---
    if q.contains("partition")
        && (q.contains("what") || q.contains("list") || q.contains("have") || q.contains("do i"))
    {
        let output = run_cmd("lsblk", &["-o", "NAME,SIZE,TYPE,MOUNTPOINT"])?;
        send_answer(writer, format!("Partitions:\n```\n{}\n```", output.trim())).await?;
        return Ok(true);
    }

    // --- MOUNTED FILESYSTEMS ---
    if (q.contains("filesystem") || q.contains("mount"))
        && (q.contains("what") || q.contains("mounted") || q.contains("list"))
        && !q.contains("root")
    {
        let output = run_cmd("df", &["-hT", "--output=source,fstype,size,used,avail,pcent,target"])?;
        send_answer(writer, format!("Mounted filesystems:\n```\n{}\n```", output.trim())).await?;
        return Ok(true);
    }

    // --- TRIM ---
    if q.contains("trim") {
        let active = run_cmd("systemctl", &["is-active", "fstrim.timer"]).unwrap_or_default();
        let enabled = run_cmd("systemctl", &["is-enabled", "fstrim.timer"]).unwrap_or_default();
        let status = if active.trim() == "active" { "active" } else { "not active" };
        send_answer(writer, format!("TRIM (fstrim.timer): {} — active: {}, enabled: {}", status, active.trim(), enabled.trim())).await?;
        return Ok(true);
    }

    // --- SMART HEALTH ---
    if q.contains("smart") || q.contains("health status") {
        let disks = run_cmd("lsblk", &["-d", "-o", "NAME", "--noheadings"])?;
        let mut results = vec!["SMART health:".to_string()];
        for disk in disks.lines().take(4) {
            let disk = disk.trim();
            if disk.is_empty() { continue; }
            let path = format!("/dev/{}", disk);
            let output = std::process::Command::new("smartctl")
                .args(&["-H", &path])
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
                .unwrap_or_else(|_| "smartctl not available".to_string());
            let health = output.lines()
                .find(|l| l.contains("SMART overall-health"))
                .and_then(|l| l.split(':').nth(1))
                .map(|s| s.trim())
                .unwrap_or("Unknown");
            results.push(format!("  {}: {}", path, health));
        }
        send_answer(writer, results.join("\n")).await?;
        return Ok(true);
    }

    // --- GPU ---
    if q.contains("gpu") || (q.contains("graphics") && q.contains("card")) {
        let output = run_cmd_cached(&cache, "lspci", "lspci", &[], 300, &[InvalidationTag::Hardware])?;
        let gpu = output
            .lines()
            .find(|l| l.to_lowercase().contains("vga") || l.to_lowercase().contains("3d"))
            .and_then(|l| l.split(':').nth(2))
            .map(|s| s.trim())
            .unwrap_or("No discrete GPU found");
        send_answer(writer, format!("GPU: {}", gpu)).await?;
        return Ok(true);
    }

    // --- AUDIO ---
    if q.contains("audio") || q.contains("sound") {
        let output = run_cmd("aplay", &["-l"]).unwrap_or_else(|_| "aplay not available".to_string());
        send_answer(writer, format!("Audio devices:\n```\n{}\n```", output.trim())).await?;
        return Ok(true);
    }

    // --- CPU MODEL ---
    if (q.contains("cpu") || q.contains("processor"))
        && (q.contains("what") || q.contains("which") || q.contains("do i have") || q.contains("model"))
        && !q.contains("core") && !q.contains("throttl") && !q.contains("profil")
        && !q.contains("temp") && !q.contains("process") && !q.contains("using")
        && !q.contains("consuming") && !q.contains("top")
    {
        let cpuinfo = std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
        let model = cpuinfo
            .lines()
            .find(|l| l.starts_with("model name"))
            .and_then(|l| l.split(':').nth(1))
            .map(|s| s.trim())
            .unwrap_or("Unknown CPU");
        send_answer(writer, format!("CPU: {}", model)).await?;
        return Ok(true);
    }

    // --- CPU CORES ---
    if q.contains("core") && (q.contains("how many") || q.contains("cpu") || q.contains("available")) {
        let cpuinfo = std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
        let cores = cpuinfo.lines().filter(|l| l.starts_with("processor")).count();
        send_answer(writer, format!("Your system has {} CPU cores.", cores)).await?;
        return Ok(true);
    }

    // --- CPU TEMPERATURE ---
    if q.contains("temp") && (q.contains("cpu") || q.contains("processor") || q.contains("system")) {
        let temps = run_shell("sensors 2>/dev/null | grep -E 'Core|Tdie|Tctl|Package' | head -8")
            .unwrap_or_default();
        let msg = if temps.trim().is_empty() {
            "Temperature sensors not available (lm_sensors not installed or not configured).".to_string()
        } else {
            format!("CPU temperatures:\n```\n{}\n```", temps.trim())
        };
        send_answer(writer, msg).await?;
        return Ok(true);
    }

    // --- CPU THROTTLING ---
    if (q.contains("cpu") || q.contains("processor")) && q.contains("throttl") {
        let gov = run_shell("cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null || echo unknown")?;
        let cur = run_shell("cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq 2>/dev/null || echo 0")?;
        let max = run_shell("cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_max_freq 2>/dev/null || echo 0")?;
        let cur_mhz: f64 = cur.trim().parse::<f64>().unwrap_or(0.0) / 1000.0;
        let max_mhz: f64 = max.trim().parse::<f64>().unwrap_or(0.0) / 1000.0;
        let status = if max_mhz > 0.0 && cur_mhz < max_mhz * 0.8 {
            format!("CPU throttling: {:.0}MHz of {:.0}MHz max ({:.0}%)", cur_mhz, max_mhz, cur_mhz / max_mhz * 100.0)
        } else {
            format!("CPU not throttling: {:.0}MHz / {:.0}MHz max", cur_mhz, max_mhz)
        };
        send_answer(writer, format!("{}\nGovernor: {}", status, gov.trim())).await?;
        return Ok(true);
    }

    // --- SYSTEM LOAD ---
    if q.contains("load average") || (q.contains("system load") && q.contains("current")) {
        let output = run_cmd("uptime", &[])?;
        send_answer(writer, output.trim().to_string()).await?;
        return Ok(true);
    }

    // --- UPTIME ---
    if q.contains("uptime") || (q.contains("how long") && q.contains("running")) {
        let output = run_cmd("uptime", &["-p"])?;
        send_answer(writer, format!("System uptime: {}", output.trim())).await?;
        return Ok(true);
    }

    // --- TOP CPU CONSUMERS ---
    if (q.contains("cpu") || q.contains("process"))
        && (q.contains("consuming") || q.contains("using most") || q.contains("top"))
        && !q.contains("throttl") && !q.contains("core")
    {
        let output = run_shell("ps aux --sort=-%cpu | head -11")?;
        send_answer(writer, format!("Top CPU consumers:\n```\n{}\n```", output.trim())).await?;
        return Ok(true);
    }

    // --- USER GROUPS ---
    if q.contains("group") && (q.contains("belong") || q.contains("my") || q.contains("what")) {
        let output = run_cmd_cached(&cache, "groups", "groups", &[], 300, &[])?;
        send_answer(writer, format!("Your groups: {}", output.trim())).await?;
        return Ok(true);
    }

    // --- SUDO PRIVILEGES ---
    if (q.contains("sudo") && q.contains("privilege")) || q.contains("do i have sudo") {
        let output = run_cmd("groups", &[])?;
        let has_sudo = output.contains("sudo") || output.contains("wheel");
        let answer = if has_sudo {
            "Yes, you have sudo privileges (member of sudo/wheel group).".to_string()
        } else {
            "No, you are not in the sudo/wheel group.".to_string()
        };
        send_answer(writer, answer).await?;
        return Ok(true);
    }

    // --- CREATE USER ---
    if q.contains("create") && q.contains("user") {
        send_answer(writer, "To create a user: `useradd -m <username>` (with home dir). Set password: `passwd <username>`. Add to sudo: `usermod -aG wheel <username>` (Arch) or `usermod -aG sudo <username>` (Debian).".to_string()).await?;
        return Ok(true);
    }

    // --- NETWORK INTERFACES ---
    if q.contains("network") && (q.contains("interface") || q.contains("available")) {
        let output = run_cmd_cached(&cache, "ip_link", "ip", &["link", "show"], 30, &[InvalidationTag::Network])?;
        send_answer(writer, format!("Network interfaces:\n```\n{}\n```", output.trim())).await?;
        return Ok(true);
    }

    Ok(false)
}
