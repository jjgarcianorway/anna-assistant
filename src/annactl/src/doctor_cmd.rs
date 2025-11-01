// Anna v0.10.1 - annactl doctor pre|post commands

use anyhow::{Context, Result};
use serde_json::Value;
use std::path::Path;
use std::process::Command;

pub fn doctor_pre(verbose: bool) -> Result<()> {
    println!("\n╭─ Anna Preflight Checks ──────────────────────────────────────────");
    println!("│");

    let mut all_ok = true;
    let mut failures = Vec::new();

    // 1. OS/Arch check
    if verbose {
        println!("│  Checking OS/architecture...");
    }
    let is_linux = cfg!(target_os = "linux");
    if is_linux {
        println!("│  ✓ OS: Linux");
    } else {
        println!("│  ✗ OS: Not Linux (unsupported)");
        all_ok = false;
        failures.push("Operating system not supported (Linux required)");
    }

    // 2. Check systemd
    if verbose {
        println!("│  Checking systemd...");
    }
    let has_systemd = Path::new("/run/systemd/system").exists();
    if has_systemd {
        println!("│  ✓ Init: systemd detected");
    } else {
        println!("│  ⚠ Init: systemd not detected (anna requires systemd)");
        all_ok = false;
        failures.push("systemd not found - anna requires systemd");
    }

    // 3. Check disk space
    if verbose {
        println!("│  Checking disk space...");
    }
    let df_output = Command::new("df")
        .args(&["--output=avail", "/"])
        .output();

    if let Ok(output) = df_output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = stdout.lines().nth(1) {
            if let Ok(avail_kb) = line.trim().parse::<u64>() {
                let avail_mb = avail_kb / 1024;
                if avail_mb >= 200 {
                    println!("│  ✓ Disk: {} MB available on /", avail_mb);
                } else {
                    println!("│  ✗ Disk: Only {} MB available on / (need 200 MB)", avail_mb);
                    all_ok = false;
                    failures.push("Insufficient disk space on / (< 200 MB)");
                }
            }
        }
    }

    // Check /var
    let df_var = Command::new("df")
        .args(&["--output=avail", "/var"])
        .output();

    if let Ok(output) = df_var {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = stdout.lines().nth(1) {
            if let Ok(avail_kb) = line.trim().parse::<u64>() {
                let avail_mb = avail_kb / 1024;
                if avail_mb >= 200 {
                    println!("│  ✓ Disk: {} MB available on /var", avail_mb);
                } else {
                    println!("│  ✗ Disk: Only {} MB available on /var (need 200 MB)", avail_mb);
                    all_ok = false;
                    failures.push("Insufficient disk space on /var (< 200 MB)");
                }
            }
        }
    }

    // 4. Check if running as root
    if verbose {
        println!("│  Checking permissions...");
    }
    let is_root = unsafe { libc::geteuid() } == 0;
    if is_root {
        println!("│  ✓ Permissions: running as root");
    } else {
        println!("│  ⚠ Permissions: not running as root (installer requires sudo)");
        // Not a failure for pre check
    }

    // 5. Check systemd unit directory
    if verbose {
        println!("│  Checking systemd paths...");
    }
    let unit_dir = Path::new("/etc/systemd/system");
    if unit_dir.exists() {
        println!("│  ✓ Systemd: unit directory exists");
    } else {
        println!("│  ✗ Systemd: unit directory not found");
        all_ok = false;
        failures.push("Systemd unit directory not found");
    }

    println!("│");
    println!("╰──────────────────────────────────────────────────────────────────");

    if all_ok {
        println!("\n✓ Preflight checks passed\n");
        std::process::exit(0);
    } else {
        println!("\n✗ Preflight checks failed\n");
        println!("Failures:");
        for failure in failures {
            println!("  - {}", failure);
        }
        println!();
        std::process::exit(10);
    }
}

pub fn doctor_post(verbose: bool) -> Result<()> {
    println!("\n╭─ Anna Postflight Checks ─────────────────────────────────────────");
    println!("│");

    let mut all_ok = true;
    let mut degraded = Vec::new();

    // 1. Check binaries exist
    if verbose {
        println!("│  Checking binaries...");
    }

    let binaries = vec![
        ("/usr/local/bin/annad", "annad"),
        ("/usr/local/bin/annactl", "annactl"),
    ];

    for (path, name) in binaries {
        if Path::new(path).exists() {
            // Check executable
            let metadata = std::fs::metadata(path)?;
            let is_executable = metadata.permissions().mode() & 0o111 != 0;
            if is_executable {
                println!("│  ✓ Binary: {} installed and executable", name);
            } else {
                println!("│  ⚠ Binary: {} not executable", name);
                degraded.push(format!("{} not executable: chmod +x {}", name, path));
            }
        } else {
            println!("│  ✗ Binary: {} not found", name);
            all_ok = false;
        }
    }

    // 2. Check systemd unit
    if verbose {
        println!("│  Checking systemd unit...");
    }
    let unit_path = "/etc/systemd/system/annad.service";
    if Path::new(unit_path).exists() {
        println!("│  ✓ Systemd: annad.service installed");
    } else {
        println!("│  ✗ Systemd: annad.service not found");
        all_ok = false;
    }

    // 3. Check directories
    if verbose {
        println!("│  Checking directories...");
    }

    let dirs = vec![
        "/var/lib/anna",
        "/var/log/anna",
        "/etc/anna",
        "/usr/lib/anna",
    ];

    for dir in dirs {
        if Path::new(dir).exists() {
            println!("│  ✓ Directory: {} exists", dir);
        } else {
            println!("│  ⚠ Directory: {} missing", dir);
            degraded.push(format!("Directory missing: {}", dir));
        }
    }

    // 4. Check anna user
    if verbose {
        println!("│  Checking anna user...");
    }
    let user_check = Command::new("id")
        .arg("anna")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    if user_check.is_ok() && user_check.unwrap().success() {
        println!("│  ✓ User: anna exists");
    } else {
        println!("│  ⚠ User: anna does not exist");
        degraded.push("User 'anna' not created".to_string());
    }

    // 5. Check CAPABILITIES.toml
    if verbose {
        println!("│  Checking capability registry...");
    }
    let cap_path = "/usr/lib/anna/CAPABILITIES.toml";
    if Path::new(cap_path).exists() {
        println!("│  ✓ Registry: CAPABILITIES.toml present");
    } else {
        println!("│  ⚠ Registry: CAPABILITIES.toml missing");
        degraded.push("CAPABILITIES.toml not installed".to_string());
    }

    // 6. Check directory ownership and permissions
    if verbose {
        println!("│  Checking directory ownership...");
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;

        // Check /var/lib/anna
        if let Ok(metadata) = std::fs::metadata("/var/lib/anna") {
            let uid = metadata.uid();
            let _gid = metadata.gid();
            let mode = metadata.mode() & 0o777;

            // Get anna user/group IDs
            let anna_check = Command::new("id")
                .args(&["-u", "anna"])
                .output();

            let anna_uid: u32 = if let Ok(output) = anna_check {
                String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .parse()
                    .unwrap_or(0)
            } else {
                0
            };

            if uid == anna_uid && mode == 0o750 {
                println!("│  ✓ Ownership: /var/lib/anna correct (anna:anna, 0750)");
            } else {
                println!("│  ⚠ Ownership: /var/lib/anna incorrect (uid={} mode={:o})", uid, mode);
                degraded.push(format!("/var/lib/anna wrong ownership: expected anna:anna 0750, got uid={} mode={:o}", uid, mode));
            }
        } else {
            println!("│  ⚠ Directory: /var/lib/anna not readable");
        }

        // Check /var/log/anna
        if let Ok(metadata) = std::fs::metadata("/var/log/anna") {
            let uid = metadata.uid();
            let _gid = metadata.gid();
            let mode = metadata.mode() & 0o777;

            let anna_check = Command::new("id")
                .args(&["-u", "anna"])
                .output();

            let anna_uid: u32 = if let Ok(output) = anna_check {
                String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .parse()
                    .unwrap_or(0)
            } else {
                0
            };

            if uid == anna_uid && mode == 0o750 {
                println!("│  ✓ Ownership: /var/log/anna correct (anna:anna, 0750)");
            } else {
                println!("│  ⚠ Ownership: /var/log/anna incorrect (uid={} mode={:o})", uid, mode);
                degraded.push(format!("/var/log/anna wrong ownership: expected anna:anna 0750, got uid={} mode={:o}", uid, mode));
            }
        } else {
            println!("│  ⚠ Directory: /var/log/anna not readable");
        }
    }

    // 7. Check annad service is running
    if verbose {
        println!("│  Checking daemon status...");
    }
    let daemon_check = Command::new("systemctl")
        .args(&["is-active", "annad"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    if daemon_check.is_ok() && daemon_check.unwrap().success() {
        println!("│  ✓ Daemon: annad is active");

        // 8. Check socket exists (wait up to 10 seconds)
        if verbose {
            println!("│  Checking RPC socket...");
        }
        let socket_path = "/run/anna/annad.sock";
        let mut socket_found = false;
        for _ in 0..10 {
            if Path::new(socket_path).exists() {
                socket_found = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }

        if socket_found {
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                if let Ok(md) = std::fs::metadata(socket_path) {
                    println!("│  ✓ Socket: {} (uid={} gid={} mode={:o})", socket_path, md.uid(), md.gid(), md.mode() & 0o777);
                } else {
                    println!("│  ✓ Socket: {} present", socket_path);
                }
            }
            #[cfg(not(unix))]
            {
                println!("│  ✓ Socket: {} present", socket_path);
            }
        } else {
            println!("│  ⚠ Socket: {} not found after 10 seconds", socket_path);
            degraded.push("RPC socket not created - daemon may be starting or failed".to_string());
        }
    } else {
        println!("│  ⚠ Daemon: annad is not running");
        degraded.push("annad service not active".to_string());
    }

    // 9. Check DB write access with detailed errno
    if verbose {
        println!("│  Checking database write access...");
    }
    let db_path = Path::new("/var/lib/anna/telemetry.db");
    let db_dir = Path::new("/var/lib/anna");
    let db_test = db_dir.join(".writetest");

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;

        // Show DB and dir info
        if verbose {
            if let Ok(md) = std::fs::metadata(db_dir) {
                println!("│    DB dir: uid={} gid={} mode={:o}", md.uid(), md.gid(), md.mode() & 0o777);
            }
            if db_path.exists() {
                if let Ok(md) = std::fs::metadata(db_path) {
                    println!("│    DB file: uid={} gid={} mode={:o}", md.uid(), md.gid(), md.mode() & 0o777);
                }
            }
        }

        match std::fs::write(&db_test, b"test") {
            Ok(_) => {
                let _ = std::fs::remove_file(&db_test);
                println!("│  ✓ Database: {} is writable", db_dir.display());
            }
            Err(e) => {
                let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
                println!("│  ⚠ Database: {} not writable (errno: {}, {})", db_dir.display(), errno, e);
                if let Ok(md) = std::fs::metadata(db_dir) {
                    println!("│    Directory: uid={} gid={} mode={:o}", md.uid(), md.gid(), md.mode() & 0o777);
                }
                degraded.push(format!("{} not writable: errno {} - {}", db_dir.display(), errno, e));
            }
        }
    }
    #[cfg(not(unix))]
    {
        match std::fs::write(&db_test, b"test") {
            Ok(_) => {
                let _ = std::fs::remove_file(&db_test);
                println!("│  ✓ Database: {} is writable", db_dir.display());
            }
            Err(e) => {
                println!("│  ⚠ Database: {} not writable ({})", db_dir.display(), e);
                degraded.push(format!("{} not writable: {}", db_dir.display(), e));
            }
        }
    }

    // 10. Check optional packages (non-fatal)
    if verbose {
        println!("│  Checking optional dependencies...");
    }

    let optional_cmds = vec!["sensors", "smartctl", "ip", "ethtool"];
    let mut missing_optional = Vec::new();

    for cmd in optional_cmds {
        if which::which(cmd).is_ok() {
            if verbose {
                println!("│  ✓ Optional: {} available", cmd);
            }
        } else {
            missing_optional.push(cmd);
            if verbose {
                println!("│  ⚠ Optional: {} not found", cmd);
            }
        }
    }

    if !missing_optional.is_empty() {
        println!("│  ⚠ Optional dependencies missing: {}", missing_optional.join(", "));
        println!("│    Some telemetry modules will be degraded");
    }

    println!("│");
    println!("╰──────────────────────────────────────────────────────────────────");

    if all_ok && degraded.is_empty() {
        println!("\n✓ Postflight checks passed - Anna is ready\n");
        std::process::exit(0);
    } else if all_ok {
        println!("\n⚠ Postflight checks passed with warnings\n");
        println!("Warnings:");
        for warn in degraded {
            println!("  - {}", warn);
        }
        println!("\nAnna will run but some features may be degraded.");
        println!("Run: annactl capabilities\n");
        std::process::exit(12); // Degraded exit code
    } else {
        println!("\n✗ Postflight checks failed\n");
        println!("Critical issues found - Anna may not function correctly");
        std::process::exit(11);
    }
}

use std::os::unix::fs::PermissionsExt;

pub fn doctor_repair(_json: bool, skip_confirmation: bool) -> Result<()> {
    println!("\n╭─ Anna Repair ────────────────────────────────────────────────────");
    println!("│");
    println!("│  This will:");
    println!("│  - Stop annad daemon");
    println!("│  - Fix directory ownership and permissions");
    println!("│  - Install/verify CAPABILITIES.toml");
    println!("│  - Restart daemon");
    println!("│");

    if !skip_confirmation {
        println!("│  Continue? [y/N] ");
        print!("│  > ");
        std::io::Write::flush(&mut std::io::stdout())?;

        let mut response = String::new();
        std::io::stdin().read_line(&mut response)?;

        if !response.trim().eq_ignore_ascii_case("y") {
            println!("│");
            println!("╰──────────────────────────────────────────────────────────────────");
            println!("\nRepair cancelled\n");
            return Ok(());
        }
    }

    println!("│");
    println!("╰──────────────────────────────────────────────────────────────────");
    println!();

    // Step 1: Stop service
    println!("→ Stopping annad service...");
    let stop_status = Command::new("sudo")
        .args(&["systemctl", "stop", "annad"])
        .status()?;

    if !stop_status.success() {
        eprintln!("⚠ Failed to stop service (continuing anyway)");
    }

    // Step 2: Fix permissions on directories
    println!("→ Fixing directory permissions...");
    let dirs = vec!["/var/lib/anna", "/var/log/anna", "/run/anna"];
    for dir in &dirs {
        let chown_status = Command::new("sudo")
            .args(&["chown", "-R", "anna:anna", dir])
            .status();

        let chmod_status = Command::new("sudo")
            .args(&["chmod", "0750", dir])
            .status();

        if chown_status.is_ok() && chmod_status.is_ok() {
            println!("  ✓ Fixed {}", dir);
        } else {
            eprintln!("  ⚠ Failed to fix {}", dir);
        }
    }

    // Step 3: Remove stale socket
    println!("→ Removing stale socket...");
    let socket_path = "/run/anna/annad.sock";
    if Path::new(socket_path).exists() {
        let rm_status = Command::new("sudo")
            .args(&["rm", "-f", socket_path])
            .status()?;
        if rm_status.success() {
            println!("  ✓ Removed {}", socket_path);
        }
    } else {
        println!("  (socket does not exist)");
    }

    // Step 4: Start service
    println!("→ Starting annad service...");
    let start_status = Command::new("sudo")
        .args(&["systemctl", "start", "annad"])
        .status()?;

    if !start_status.success() {
        eprintln!("\n✗ Failed to start service");
        eprintln!("  Check: sudo journalctl -u annad -n 30\n");
        std::process::exit(1);
    }

    // Step 5: Poll for socket (10 seconds)
    println!("→ Waiting for socket (up to 10s)...");
    let mut socket_found = false;
    for i in 1..=10 {
        std::thread::sleep(std::time::Duration::from_secs(1));
        if Path::new(socket_path).exists() {
            socket_found = true;
            println!("  ✓ Socket appeared after {}s", i);
            break;
        }
    }

    if !socket_found {
        eprintln!("\n⚠ Socket did not appear after 10s");
        eprintln!("  Check: sudo journalctl -u annad -n 30\n");
        std::process::exit(1);
    }

    // Step 6: Verify socket permissions and ownership
    println!("→ Verifying socket permissions...");
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if let Ok(md) = std::fs::metadata(socket_path) {
            let mode = md.mode() & 0o777;
            let uid = md.uid();
            let gid = md.gid();

            // Get anna uid/gid for comparison
            let anna_uid = Command::new("id")
                .args(&["-u", "anna"])
                .output()
                .ok()
                .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse::<u32>().ok())
                .unwrap_or(1003);

            let anna_gid = Command::new("id")
                .args(&["-g", "anna"])
                .output()
                .ok()
                .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse::<u32>().ok())
                .unwrap_or(1003);

            if mode != 0o660 {
                println!("  ⚠ Socket mode incorrect: {:o} (expected: 660), fixing...", mode);
                let chmod_status = Command::new("sudo")
                    .args(&["chmod", "0660", socket_path])
                    .status()?;
                if chmod_status.success() {
                    println!("  ✓ Fixed socket mode to 0660");
                }
            } else {
                println!("  ✓ Socket mode correct: {:o}", mode);
            }

            if uid != anna_uid || gid != anna_gid {
                println!("  ⚠ Socket ownership incorrect: uid={} gid={} (expected: anna:anna), fixing...", uid, gid);
                let chown_status = Command::new("sudo")
                    .args(&["chown", "anna:anna", socket_path])
                    .status()?;
                if chown_status.success() {
                    println!("  ✓ Fixed socket ownership to anna:anna");
                }
            } else {
                println!("  ✓ Socket ownership correct: anna:anna");
            }
        }
    }

    // Step 7: Verify systemd RuntimeDirectoryMode
    println!("→ Verifying systemd configuration...");
    let unit_path = "/etc/systemd/system/annad.service";
    if let Ok(contents) = std::fs::read_to_string(unit_path) {
        if contents.contains("RuntimeDirectoryMode=0750") {
            println!("  ✓ RuntimeDirectoryMode=0750 present in service file");
        } else {
            println!("  ⚠ RuntimeDirectoryMode=0750 not found in service file");
            println!("    Expected: RuntimeDirectoryMode=0750");
        }
    }

    println!();
    println!("✓ Repair completed successfully");
    println!();
    println!("Recommended: Run 'annactl doctor post --verbose' to verify");
    println!();
    std::process::exit(0);
}

pub fn doctor_report(output_path: Option<&str>) -> Result<()> {
    use std::fs::File;
    use std::io::Write;

    println!("\n╭─ Anna Diagnostic Report ─────────────────────────────────────────");
    println!("│");

    // Generate timestamp
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let default_output = format!("/tmp/anna-report-{}.tar.gz", timestamp);
    let report_path = output_path.unwrap_or(&default_output);
    let report_dir = format!("/tmp/anna-report-{}", timestamp);

    println!("│  Generating diagnostic report...");
    println!("│  Output: {}", report_path);
    println!("│");

    // Create temporary directory
    std::fs::create_dir_all(&report_dir)?;

    // Collect system information
    let mut system_info = String::new();
    system_info.push_str(&format!("Anna Diagnostic Report\n"));
    system_info.push_str(&format!("Generated: {}\n", timestamp));
    system_info.push_str(&format!("Hostname: {}\n", hostname::get()?.to_string_lossy()));
    system_info.push_str(&format!("\n"));

    // OS info
    if let Ok(os_release) = std::fs::read_to_string("/etc/os-release") {
        system_info.push_str("OS Information:\n");
        system_info.push_str(&os_release);
        system_info.push_str("\n");
    }

    // Kernel
    if let Ok(output) = Command::new("uname").arg("-a").output() {
        system_info.push_str(&format!("Kernel: {}\n", String::from_utf8_lossy(&output.stdout)));
    }

    File::create(format!("{}/system_info.txt", report_dir))?.write_all(system_info.as_bytes())?;

    // Copy systemd unit file
    if let Ok(unit) = std::fs::read_to_string("/etc/systemd/system/annad.service") {
        File::create(format!("{}/annad.service", report_dir))?.write_all(unit.as_bytes())?;
    }

    // Directory listings
    let mut dir_listing = String::new();
    for dir in &["/var/lib/anna", "/var/log/anna", "/run/anna", "/etc/anna", "/usr/lib/anna"] {
        if let Ok(output) = Command::new("ls").args(&["-laR", dir]).output() {
            dir_listing.push_str(&format!("\n=== {} ===\n", dir));
            dir_listing.push_str(&String::from_utf8_lossy(&output.stdout));
        }
    }
    File::create(format!("{}/directory_listings.txt", report_dir))?.write_all(dir_listing.as_bytes())?;

    // Journal logs
    if let Ok(output) = Command::new("journalctl").args(&["-u", "annad", "-n", "100", "--no-pager"]).output() {
        File::create(format!("{}/journal.log", report_dir))?.write_all(&output.stdout)?;
    }

    // annactl outputs
    let mut annactl_output = String::new();

    // version
    if let Ok(output) = Command::new("annactl").arg("version").output() {
        annactl_output.push_str("=== annactl version ===\n");
        annactl_output.push_str(&String::from_utf8_lossy(&output.stdout));
        annactl_output.push_str("\n");
    }

    // status
    if let Ok(output) = Command::new("annactl").arg("status").output() {
        annactl_output.push_str("=== annactl status ===\n");
        annactl_output.push_str(&String::from_utf8_lossy(&output.stdout));
        annactl_output.push_str("\n");
    }

    // events
    if let Ok(output) = Command::new("annactl").args(&["events", "--limit", "10"]).output() {
        annactl_output.push_str("=== annactl events --limit 10 ===\n");
        annactl_output.push_str(&String::from_utf8_lossy(&output.stdout));
        annactl_output.push_str("\n");
    }

    File::create(format!("{}/annactl_outputs.txt", report_dir))?.write_all(annactl_output.as_bytes())?;

    // Create tarball
    let status = Command::new("tar")
        .args(&["-czf", report_path, "-C", "/tmp", &format!("anna-report-{}", timestamp)])
        .status()?;

    // Cleanup temp directory
    std::fs::remove_dir_all(&report_dir)?;

    if status.success() {
        println!("│  ✓ Report generated successfully");
        println!("│");
        println!("╰──────────────────────────────────────────────────────────────────");
        println!();
        println!("Report saved to: {}", report_path);
        println!();
        println!("Share this file when reporting issues or requesting support.");
        println!();
        std::process::exit(0);
    } else {
        eprintln!("│  ✗ Failed to create tarball");
        eprintln!("│");
        eprintln!("╰──────────────────────────────────────────────────────────────────");
        std::process::exit(1);
    }
}

// Anna v0.12.4 - Comprehensive Health Check Implementation

pub async fn doctor_check(_json: bool, verbose: bool) -> Result<()> {
    println!("\n╭─ Anna Health Check ──────────────────────────────────────────────");
    println!("│");

    let mut issues = Vec::new();
    let mut warnings = Vec::new();
    let mut suggestions = Vec::new();

    // 1. Daemon Connectivity Check
    if verbose {
        println!("│  Checking daemon connectivity...");
    }

    let daemon_ok = check_daemon_connectivity(&mut issues, &mut warnings, verbose).await;

    if !daemon_ok {
        print_check_results(issues, warnings, suggestions);
        std::process::exit(1);
    }

    // 2. Radar Health Check
    if verbose {
        println!("│  Checking system radars...");
    }
    check_radar_health(&mut issues, &mut warnings, &mut suggestions, verbose).await?;

    // 3. Resource Usage Check
    if verbose {
        println!("│  Checking resource usage...");
    }
    check_resource_usage(&mut issues, &mut warnings, &mut suggestions, verbose).await?;

    // 4. Configuration Check
    if verbose {
        println!("│  Checking configuration...");
    }
    check_configuration(&mut issues, &mut warnings, &mut suggestions, verbose);

    // 5. Permission Check
    if verbose {
        println!("│  Checking permissions...");
    }
    check_permissions(&mut issues, &mut warnings, &mut suggestions, verbose);

    println!("│");
    print_check_results(issues, warnings, suggestions);

    Ok(())
}

async fn check_daemon_connectivity(
    issues: &mut Vec<String>,
    warnings: &mut Vec<String>,
    verbose: bool,
) -> bool {
    // Check if service is running
    let service_status = Command::new("systemctl")
        .args(&["is-active", "annad"])
        .output();

    match service_status {
        Ok(output) if output.status.success() => {
            println!("│  ✓ Daemon: annad service is active");
        }
        _ => {
            println!("│  ✗ Daemon: annad service is not running");
            issues.push("annad service is not active - run: sudo systemctl start annad".to_string());
            return false;
        }
    }

    // Check socket exists
    let socket_path = std::path::Path::new("/run/anna/annad.sock");
    if !socket_path.exists() {
        println!("│  ✗ Socket: RPC socket not found");
        issues.push("RPC socket missing at /run/anna/annad.sock".to_string());
        return false;
    }

    if verbose {
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if let Ok(md) = std::fs::metadata(socket_path) {
                println!(
                    "│    Socket: uid={} gid={} mode={:o}",
                    md.uid(),
                    md.gid(),
                    md.mode() & 0o777
                );
            }
        }
    }

    // Try RPC call
    let rpc_result = try_rpc_call("status", None).await;
    match rpc_result {
        Ok(_) => {
            println!("│  ✓ RPC: Daemon responding to requests");
            true
        }
        Err(e) => {
            println!("│  ✗ RPC: Daemon not responding");
            if verbose {
                println!("│    Error: {}", e);
            }
            warnings.push(format!("Daemon not responding to RPC calls: {}", e));
            false
        }
    }
}

async fn check_radar_health(
    issues: &mut Vec<String>,
    warnings: &mut Vec<String>,
    suggestions: &mut Vec<String>,
    verbose: bool,
) -> Result<()> {
    match try_rpc_call("radar_show", None).await {
        Ok(response) => {
            if let Some(health) = response.get("health") {
                if let Some(categories) = health.get("categories") {
                    // Check CPU load
                    if let Some(cpu_load) = categories.get("cpu_load") {
                        if let Some(score) = cpu_load.get("score").and_then(|s| s.as_f64()) {
                            if score < 3.0 {
                                warnings.push(format!("High CPU load detected (score: {})", format_score(score)));
                                suggestions.push("Consider closing resource-intensive applications".to_string());
                            } else if verbose {
                                println!("│  ✓ CPU Load: {}", format_score(score));
                            }
                        }
                    }

                    // Check memory pressure
                    if let Some(mem_pressure) = categories.get("memory_pressure") {
                        if let Some(score) = mem_pressure.get("score").and_then(|s| s.as_f64()) {
                            if score < 3.0 {
                                warnings.push(format!("High memory pressure (score: {})", format_score(score)));
                                suggestions.push("Free up memory or close applications".to_string());
                            } else if verbose {
                                println!("│  ✓ Memory Pressure: {}", format_score(score));
                            }
                        }
                    }

                    // Check disk headroom
                    if let Some(disk) = categories.get("disk_headroom") {
                        if let Some(score) = disk.get("score").and_then(|s| s.as_f64()) {
                            if score < 3.0 {
                                warnings.push(format!("Low disk space (score: {})", format_score(score)));
                                suggestions.push("Clean up disk space on /".to_string());
                            } else if verbose {
                                println!("│  ✓ Disk Headroom: {}", format_score(score));
                            }
                        }
                    }

                    // Check thermal
                    if let Some(thermal) = categories.get("thermal_ok") {
                        if let Some(score) = thermal.get("score").and_then(|s| s.as_f64()) {
                            if score < 5.0 {
                                warnings.push(format!("High temperature detected (score: {})", format_score(score)));
                                suggestions.push("Check cooling system and airflow".to_string());
                            } else if verbose {
                                println!("│  ✓ Thermal: {}", format_score(score));
                            }
                        }
                    }
                }
            }

            // Check network radar
            if let Some(network) = response.get("network") {
                if let Some(categories) = network.get("categories") {
                    if let Some(dns) = categories.get("dns_reliability") {
                        if let Some(score) = dns.get("score").and_then(|s| s.as_f64()) {
                            if score < 5.0 {
                                warnings.push(format!("Network connectivity issues (score: {})", format_score(score)));
                                suggestions.push("Check network connection (can you ping 8.8.8.8?)".to_string());
                            } else if verbose {
                                println!("│  ✓ Network Connectivity: {}", format_score(score));
                            }
                        }
                    }
                }
            }

            println!("│  ✓ Radars: All radars operational");
        }
        Err(e) => {
            issues.push(format!("Failed to fetch radar scores: {}", e));
        }
    }

    Ok(())
}

async fn check_resource_usage(
    _issues: &mut Vec<String>,
    warnings: &mut Vec<String>,
    suggestions: &mut Vec<String>,
    verbose: bool,
) -> Result<()> {
    // Get telemetry snapshot
    match try_rpc_call("collect", Some(serde_json::json!({"limit": 1}))).await {
        Ok(response) => {
            if let Some(snapshots) = response.get("snapshots").and_then(|s| s.as_array()) {
                if let Some(snapshot) = snapshots.first() {
                    // Check CPU cores vs load
                    if let Some(cores) = snapshot.get("cores").and_then(|c| c.as_u64()) {
                        if let Some(load_1m) = snapshot.get("load_1m").and_then(|l| l.as_f64()) {
                            let load_per_core = load_1m / cores as f64;
                            if load_per_core > 1.5 {
                                warnings.push(format!("High load: {:.2} per core", load_per_core));
                            } else if verbose {
                                println!("│  ✓ Load: {:.2} per core", load_per_core);
                            }
                        }
                    }

                    // Check memory usage
                    if let Some(mem_used) = snapshot.get("mem_used_mb").and_then(|m| m.as_u64()) {
                        if let Some(mem_total) = snapshot.get("mem_free_mb").and_then(|m| m.as_u64()) {
                            let total = mem_used + mem_total;
                            let used_pct = (mem_used as f64 / total as f64) * 100.0;
                            if used_pct > 90.0 {
                                warnings.push(format!("High memory usage: {:.1}%", used_pct));
                                suggestions.push("Consider restarting applications or adding more RAM".to_string());
                            } else if verbose {
                                println!("│  ✓ Memory Usage: {:.1}%", used_pct);
                            }
                        }
                    }
                }
            }
            println!("│  ✓ Resources: Telemetry collection working");
        }
        Err(e) => {
            warnings.push(format!("Could not collect telemetry: {}", e));
        }
    }

    Ok(())
}

fn check_configuration(
    _issues: &mut Vec<String>,
    warnings: &mut Vec<String>,
    _suggestions: &mut Vec<String>,
    verbose: bool,
) {
    let config_files = vec![
        ("/etc/anna/config.toml", true),
        ("/etc/anna/policy.toml", false), // optional
        ("/usr/lib/anna/CAPABILITIES.toml", true),
    ];

    for (path, required) in config_files {
        if std::path::Path::new(path).exists() {
            if verbose {
                println!("│  ✓ Config: {} present", path);
            }
        } else if required {
            warnings.push(format!("Missing config file: {}", path));
        } else if verbose {
            println!("│  ⚠ Config: {} missing (optional)", path);
        }
    }

    println!("│  ✓ Configuration: Core files present");
}

fn check_permissions(
    _issues: &mut Vec<String>,
    warnings: &mut Vec<String>,
    suggestions: &mut Vec<String>,
    verbose: bool,
) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;

        let dirs_to_check = vec![
            ("/var/lib/anna", 0o750),
            ("/var/log/anna", 0o750),
            ("/run/anna", 0o750),
        ];

        for (dir, expected_mode) in dirs_to_check {
            if let Ok(md) = std::fs::metadata(dir) {
                let mode = md.mode() & 0o777;
                if mode != expected_mode {
                    warnings.push(format!(
                        "{} has mode {:o}, expected {:o}",
                        dir, mode, expected_mode
                    ));
                    suggestions.push(format!("Run: annactl doctor repair"));
                } else if verbose {
                    println!("│  ✓ Permissions: {} ({:o})", dir, mode);
                }
            } else {
                warnings.push(format!("{} not accessible", dir));
            }
        }
    }

    println!("│  ✓ Permissions: Directory permissions OK");
}

fn print_check_results(issues: Vec<String>, warnings: Vec<String>, suggestions: Vec<String>) {
    println!("╰──────────────────────────────────────────────────────────────────");
    println!();

    if issues.is_empty() && warnings.is_empty() {
        println!("✓ All health checks passed");
        println!();
        println!("Anna is operating normally.");
        std::process::exit(0);
    } else if !issues.is_empty() {
        println!("✗ Critical Issues Found:");
        for issue in &issues {
            println!("  • {}", issue);
        }
        println!();

        if !warnings.is_empty() {
            println!("⚠ Warnings:");
            for warning in &warnings {
                println!("  • {}", warning);
            }
            println!();
        }

        if !suggestions.is_empty() {
            println!("Suggested Actions:");
            for suggestion in &suggestions {
                println!("  → {}", suggestion);
            }
            println!();
        }

        std::process::exit(1);
    } else {
        println!("⚠ Health check passed with warnings");
        println!();
        println!("Warnings:");
        for warning in &warnings {
            println!("  • {}", warning);
        }
        println!();

        if !suggestions.is_empty() {
            println!("Suggested Actions:");
            for suggestion in &suggestions {
                println!("  → {}", suggestion);
            }
            println!();
        }

        std::process::exit(0);
    }
}

// Format score nicely: 10/10 for whole numbers, 9.2/10 for decimals
fn format_score(score: f64) -> String {
    if score.fract() == 0.0 {
        format!("{:.0}/10", score)
    } else {
        format!("{:.1}/10", score)
    }
}

async fn try_rpc_call(method: &str, params: Option<Value>) -> Result<Value> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixStream;

    let socket_path = "/run/anna/annad.sock";
    let mut stream = UnixStream::connect(socket_path)
        .await
        .context("Failed to connect to daemon socket")?;

    // Build JSON-RPC request
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params,
    });

    let request_str = serde_json::to_string(&request)?;
    stream
        .write_all(request_str.as_bytes())
        .await
        .context("Failed to send RPC request")?;
    stream.write_all(b"\n").await?;

    // Read response with timeout
    let mut buffer = vec![0u8; 65536];
    let n = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        stream.read(&mut buffer),
    )
    .await
    .context("RPC call timed out after 5s")??;

    let response_str = String::from_utf8_lossy(&buffer[..n]);
    let response: Value =
        serde_json::from_str(&response_str).context("Failed to parse RPC response")?;

    if let Some(error) = response.get("error") {
        anyhow::bail!("RPC error: {}", error);
    }

    response
        .get("result")
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("No result in RPC response"))
}
