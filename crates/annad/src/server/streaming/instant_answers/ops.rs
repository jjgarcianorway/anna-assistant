//! Instant answers: processes, services, packages, logs, networking, security.

use anyhow::Result;
use super::super::instant_answers::{run_cmd, run_shell, send_answer};

pub async fn try_ops_answer(
    question: &str,
    writer: &mut tokio::net::unix::OwnedWriteHalf,
) -> Result<bool> {
    let q = question.to_lowercase();

    // --- PROCESS: STARTER / PARENT ---
    if (q.contains("started") && q.contains("process")) || (q.contains("parent") && q.contains("process")) {
        let output = run_cmd("ps", &["auxf"])?;
        let top = output.lines().take(30).collect::<Vec<_>>().join("\n");
        send_answer(writer, format!("Process tree:\n```\n{}\n```", top)).await?;
        return Ok(true);
    }

    // --- KILL PROCESS ---
    if q.contains("kill") && (q.contains("process") || q.contains("stuck")) {
        send_answer(writer, "Kill a process: `kill <PID>` (graceful) or `kill -9 <PID>` (force). Find PID: `pgrep <name>` or `ps aux | grep <name>`.".to_string()).await?;
        return Ok(true);
    }

    // --- BOOT SERVICES ---
    if q.contains("boot") && q.contains("service")
        || (q.contains("start") && q.contains("boot") && q.contains("service"))
    {
        let output = run_cmd("systemctl", &["list-unit-files", "--state=enabled", "--no-pager"])?;
        send_answer(writer, format!("Services enabled at boot:\n```\n{}\n```", output.trim())).await?;
        return Ok(true);
    }

    // --- BOOT TIME ---
    if q.contains("boot") && (q.contains("slow") || q.contains("time") || q.contains("long")) {
        let output = run_cmd("systemd-analyze", &[])?;
        let blame = run_cmd("systemd-analyze", &["blame"]).unwrap_or_default();
        let top = blame.lines().take(10).collect::<Vec<_>>().join("\n");
        send_answer(writer, format!("{}\n\nTop boot time contributors:\n{}", output.trim(), top)).await?;
        return Ok(true);
    }

    // --- BOOT LOGS ---
    if q.contains("boot log") || (q.contains("analyze") && q.contains("boot")) {
        let output = run_cmd("journalctl", &["-b", "--no-pager", "-p", "err", "--lines=20"])?;
        send_answer(writer, format!("Boot errors:\n```\n{}\n```", output.trim())).await?;
        return Ok(true);
    }

    // --- SERVICE FAILURE ---
    if q.contains("service") && (q.contains("fail") || q.contains("why")) && !q.contains("boot") {
        let output = run_cmd("systemctl", &["--failed", "--no-pager"])?;
        send_answer(writer, format!("Failed services:\n```\n{}\n```", output.trim())).await?;
        return Ok(true);
    }

    // --- ENABLE SERVICE AT BOOT ---
    if q.contains("enable") && q.contains("service") && q.contains("boot") {
        send_answer(writer, "Enable a service at boot: `systemctl enable <service>`. Enable and start now: `systemctl enable --now <service>`.".to_string()).await?;
        return Ok(true);
    }

    // --- SERVICE STATUS ---
    if q.contains("status") && (q.contains("service") || q.contains("systemd") || q.contains("unit")) {
        let output = run_cmd("systemctl", &["list-units", "--state=failed", "--no-pager"])?;
        send_answer(writer, format!("Failed systemd units:\n```\n{}\n```\nUse `systemctl status <name>` for a specific service.", output.trim())).await?;
        return Ok(true);
    }

    // --- ACTIVE TIMERS ---
    if q.contains("timer") && (q.contains("active") || q.contains("what")) {
        let output = run_cmd("systemctl", &["list-timers", "--no-pager"])?;
        send_answer(writer, format!("Active systemd timers:\n```\n{}\n```", output.trim())).await?;
        return Ok(true);
    }

    // --- CREATE SYSTEMD TIMER ---
    if q.contains("timer") && (q.contains("create") || q.contains("schedule") || q.contains("systemd")) {
        send_answer(writer, "Create a systemd timer: write a `.service` and `.timer` file in `/etc/systemd/system/`, then `systemctl enable --now <name>.timer`. Use `OnCalendar=` for scheduled times.".to_string()).await?;
        return Ok(true);
    }

    // --- RECENTLY INSTALLED PACKAGES ---
    if q.contains("package") && q.contains("recent")
        || (q.contains("install") && q.contains("recent"))
    {
        let output = run_shell("tail -20 /var/log/pacman.log 2>/dev/null | grep -E 'installed|upgraded' || tail -20 /var/log/dpkg.log 2>/dev/null | grep 'install '").unwrap_or_default();
        send_answer(writer, format!("Recently installed packages:\n```\n{}\n```", output.trim())).await?;
        return Ok(true);
    }

    // --- PACKAGE FAILURE ---
    if q.contains("package") && (q.contains("fail") || q.contains("error")) && q.contains("install") {
        let output = run_shell("tail -50 /var/log/pacman.log 2>/dev/null | grep -E 'error|warning|fail' | tail -10").unwrap_or_default();
        send_answer(writer, format!("Recent package errors:\n```\n{}\n```", output.trim())).await?;
        return Ok(true);
    }

    // --- ORPHANED / UNUSED PACKAGES ---
    if q.contains("orphan") || (q.contains("unused") && q.contains("package")) {
        let output = run_cmd("pacman", &["-Qdt"])
            .unwrap_or_else(|_| "".to_string());
        let msg = if output.trim().is_empty() {
            "No orphaned packages found.".to_string()
        } else {
            format!("Orphaned packages (remove with `pacman -Rns`):\n```\n{}\n```", output.trim())
        };
        send_answer(writer, msg).await?;
        return Ok(true);
    }

    // --- SAFE SYSTEM UPDATE (Q42 timeout fix) ---
    if q.contains("update") && (q.contains("system") || q.contains("safely") || q.contains("safe")) {
        let available = run_shell("checkupdates 2>/dev/null | head -20 || pacman -Qu 2>/dev/null | head -20").unwrap_or_default();
        let answer = if available.trim().is_empty() {
            "System is up to date. No updates available.".to_string()
        } else {
            format!("Available updates:\n```\n{}\n```\nUpdate command: `sudo pacman -Syu` (Arch) or `sudo apt upgrade` (Debian/Ubuntu). Review changes before proceeding.", available.trim())
        };
        send_answer(writer, answer).await?;
        return Ok(true);
    }

    // --- SYSTEM ERRORS IN LOGS ---
    if q.contains("log") && (q.contains("error") || q.contains("check")) {
        let output = run_cmd("journalctl", &["-p", "err", "-n", "20", "--no-pager"])?;
        send_answer(writer, format!("Recent system errors:\n```\n{}\n```", output.trim())).await?;
        return Ok(true);
    }

    // --- KERNEL PANIC ---
    if q.contains("kernel panic") || (q.contains("panic") && q.contains("log")) {
        let output = run_shell("journalctl -b -1 --no-pager 2>/dev/null | grep -i 'panic\\|oops\\|segfault' | tail -10 || dmesg | grep -i panic | tail -10").unwrap_or_default();
        let msg = if output.trim().is_empty() {
            "No kernel panic found in recent logs.".to_string()
        } else {
            format!("Kernel panic/oops entries:\n```\n{}\n```", output.trim())
        };
        send_answer(writer, msg).await?;
        return Ok(true);
    }

    // --- FILE PERMISSIONS EXPLANATION (Q29 timeout fix) ---
    if q.contains("permission")
        && (q.contains("explain") || q.contains("what") || q.contains("mean"))
        && !q.contains("exposing") && !q.contains("sensitive")
    {
        let home_files = run_shell("ls -la ~ | head -10").unwrap_or_default();
        let answer = format!(
            "Linux permissions: rwx = read(4)+write(2)+execute(1). Format: [type][owner][group][others].\n\
            Example: -rwxr-xr-- = file, owner=rwx, group=rx, others=r.\n\
            Common: 755=rwxr-xr-x (executable), 644=rw-r--r-- (file), 700=rwx------ (private).\n\
            \nYour home directory:\n```\n{}\n```\n\
            Use `ls -la <path>` to inspect a specific file.",
            home_files.trim()
        );
        send_answer(writer, answer).await?;
        return Ok(true);
    }

    // --- SENSITIVE FILE PERMISSIONS (Q89 timeout fix) ---
    if q.contains("permission") && (q.contains("sensitive") || q.contains("exposing")) {
        let mut results = vec!["Sensitive file permission check:".to_string()];
        for (path, expected) in &[("/etc/passwd", "644"), ("/etc/shadow", "640"), ("/etc/sudoers", "440"), ("/root", "700")] {
            let perms = run_shell(&format!("stat -c '%a %n' {} 2>/dev/null || echo 'not found'", path))
                .unwrap_or_default();
            results.push(format!("  {} (expected {}): {}", path, expected, perms.trim()));
        }
        let world_writable = run_shell("find /etc -maxdepth 2 -perm -002 -type f 2>/dev/null | head -5")
            .unwrap_or_default();
        if world_writable.trim().is_empty() {
            results.push("\nNo world-writable files found in /etc.".to_string());
        } else {
            results.push(format!("\nWorld-writable files in /etc (review urgently):\n{}", world_writable.trim()));
        }
        send_answer(writer, results.join("\n")).await?;
        return Ok(true);
    }

    // --- DNS CHECK ---
    if q.contains("dns") && (q.contains("working") || q.contains("correct")) {
        let result = run_cmd("dig", &["+short", "google.com"])
            .or_else(|_| run_cmd("nslookup", &["google.com"]))
            .unwrap_or_else(|_| "".to_string());
        let ok = !result.trim().is_empty() && !result.contains("failed");
        let answer = if ok {
            format!("DNS is working. google.com resolved to: {}", result.trim())
        } else {
            "DNS resolution failed. Check /etc/resolv.conf and network connectivity.".to_string()
        };
        send_answer(writer, answer).await?;
        return Ok(true);
    }

    // --- DNS SERVER ---
    if q.contains("dns") && (q.contains("server") || q.contains("which") || q.contains("using")) {
        let output = run_shell("grep ^nameserver /etc/resolv.conf 2>/dev/null || resolvectl status 2>/dev/null | grep 'DNS Servers' | head -3").unwrap_or_default();
        send_answer(writer, format!("DNS servers:\n{}", output.trim())).await?;
        return Ok(true);
    }

    // --- OPEN PORTS ---
    if q.contains("port") && (q.contains("open") || q.contains("listening")) {
        let output = run_cmd("ss", &["-tulpn"])?;
        send_answer(writer, format!("Open ports:\n```\n{}\n```", output.trim())).await?;
        return Ok(true);
    }

    // --- ROUTING TABLE ---
    if q.contains("routing") || (q.contains("route") && q.contains("table")) {
        let output = run_cmd("ip", &["route", "show"])?;
        send_answer(writer, format!("Routing table:\n```\n{}\n```", output.trim())).await?;
        return Ok(true);
    }

    // --- UNAUTHORIZED LOGIN ATTEMPTS ---
    if (q.contains("unauthorized") || q.contains("failed")) && q.contains("login") {
        let output = run_cmd("lastb", &["-20"]).unwrap_or_default();
        let count = output.lines().filter(|l| !l.trim().is_empty() && !l.starts_with("btmp")).count();
        let answer = if count == 0 {
            "No recent unauthorized login attempts found.".to_string()
        } else {
            format!("Found {} failed login attempts:\n```\n{}\n```", count, output.trim())
        };
        send_answer(writer, answer).await?;
        return Ok(true);
    }

    // --- SUSPICIOUS PROCESSES ---
    if q.contains("suspicious") && q.contains("process") {
        let output = run_shell("ps aux | awk '$3 > 50 || $4 > 50 {print}' | head -10")?;
        let msg = if output.trim().is_empty() {
            "No suspicious processes detected (none using >50% CPU or RAM).".to_string()
        } else {
            format!("High-resource processes:\n```\n{}\n```", output.trim())
        };
        send_answer(writer, msg).await?;
        return Ok(true);
    }

    // --- SYSTEM CALL TRACING ---
    if q.contains("system call") || q.contains("syscall") || q.contains("strace") {
        send_answer(writer, "Trace system calls: `strace -p <PID>` (attach to process) or `strace <command>`. Summary: `strace -c <command>`. Install: `pacman -S strace`.".to_string()).await?;
        return Ok(true);
    }

    // --- CPU PROFILING ---
    if (q.contains("profile") && q.contains("cpu")) || (q.contains("per thread") && q.contains("cpu")) {
        send_answer(writer, "CPU profiling: `perf top` (live), `perf record -g <cmd>` + `perf report` (detailed). Per-thread: `perf top -t <TID>`. Install: `pacman -S perf`.".to_string()).await?;
        return Ok(true);
    }

    // --- RECENT REBOOTS ---
    if q.contains("reboot") && (q.contains("recent") || q.contains("last") || q.contains("when")) {
        let output = run_shell("last reboot | head -10")?;
        send_answer(writer, format!("Recent reboots:\n```\n{}\n```", output.trim())).await?;
        return Ok(true);
    }

    // --- FIREWALL STATUS (Q76/Q87 timeout fix) ---
    // Daemon runs as root so iptables/nft work directly. Use -n to avoid DNS lookup hangs.
    if q.contains("firewall") && (q.contains("configured") || q.contains("proper") || q.contains("blocking") || q.contains("critical") || q.contains("status")) {
        // Try nftables first (modern), then iptables with -n (no DNS)
        let nft = run_shell("nft list ruleset 2>/dev/null | head -40").unwrap_or_default();
        let ipt = run_shell("iptables -nL --line-numbers 2>/dev/null | head -40").unwrap_or_default();
        let ufw = run_shell("ufw status 2>/dev/null").unwrap_or_default();

        let answer = if !nft.trim().is_empty() {
            format!("Firewall (nftables):\n```\n{}\n```", nft.trim())
        } else if !ipt.trim().is_empty() {
            format!("Firewall (iptables, -n flag to avoid DNS hangs):\n```\n{}\n```", ipt.trim())
        } else if !ufw.trim().is_empty() {
            format!("Firewall (ufw):\n```\n{}\n```", ufw.trim())
        } else {
            "No active firewall detected (nftables, iptables, ufw not configured).".to_string()
        };
        send_answer(writer, answer).await?;
        return Ok(true);
    }

    // --- CHANGE OWNERSHIP (Q24 timeout fix) ---
    // Daemon runs as root, so chown works. But question lacks specific path - show current dir.
    if q.contains("ownership") && (q.contains("change") || q.contains("chown")) {
        let user = crate::user_context::get_real_user().unwrap_or_else(|_| "user".to_string());
        let answer = format!(
            "To change ownership: `chown {} <path>` (file/dir) or `chown -R {} <path>` (recursive).\n\
            Since the daemon runs as root, ownership changes execute directly.\n\
            Specify which path you want to transfer — e.g. ask: \"change ownership of /srv/data to me\"",
            user, user
        );
        send_answer(writer, answer).await?;
        return Ok(true);
    }

    // --- KERNEL PARAMETER TUNING (Q91 timeout fix) ---
    // Daemon runs as root so sysctl -w works without sudo.
    if (q.contains("kernel") && q.contains("parameter")) || q.contains("sysctl") || (q.contains("tune") && q.contains("kernel")) {
        let current = run_shell("sysctl -a 2>/dev/null | grep -E 'vm\\.(swappiness|dirty)|net\\.core\\.(rmem|wmem)|net\\.ipv4\\.(tcp_rmem|tcp_wmem|tcp_congestion)' | head -15")
            .unwrap_or_default();
        let answer = format!(
            "Current performance-relevant kernel parameters:\n```\n{}\n```\n\
            Common tunings (daemon applies as root, changes persist after reboot if added to /etc/sysctl.d/):\n\
            • vm.swappiness=10          — reduce swap usage (default 60)\n\
            • vm.dirty_ratio=15         — flush dirty pages earlier\n\
            • net.core.rmem_max=16MB    — increase network buffer\n\
            • net.ipv4.tcp_congestion_control=bbr — modern TCP (if available)\n\
            \nTo apply: ask \"set vm.swappiness to 10\" and I'll run it directly.",
            current.trim()
        );
        send_answer(writer, answer).await?;
        return Ok(true);
    }

    Ok(false)
}
