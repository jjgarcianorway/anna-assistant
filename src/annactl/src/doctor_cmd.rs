// Anna v0.10.1 - annactl doctor pre|post commands

use anyhow::Result;
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

    // 6. Check optional packages (non-fatal)
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
