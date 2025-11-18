//! Init Command - First-run wizard
//! Phase 3.9: Create /etc/anna, generate config, explain basics

use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::Command;

const CONFIG_DIR: &str = "/etc/anna";
const CONFIG_FILE: &str = "/etc/anna/config.toml";
const SENTINEL_FILE: &str = "/etc/anna/sentinel.toml";

const DEFAULT_CONFIG: &str = r#"# Anna Assistant Configuration
# Phase 3.9: Default configuration for new installations

# Monitoring mode (auto-detected, or: minimal, light, full)
monitoring_mode = "auto"

# Enable predictive intelligence
enable_predictions = true

# Enable self-healing (low-risk actions only)
enable_self_healing = false

# Action history retention (days)
history_retention_days = 90

# Log level (debug, info, warn, error)
log_level = "info"
"#;

const DEFAULT_SENTINEL: &str = r#"# Sentinel Framework Configuration
# Phase 1.0: Autonomous monitoring and action policies

# Probe check interval (seconds)
probe_interval = 60

# Health thresholds
[thresholds]
cpu_critical = 95
cpu_warning = 85
memory_critical = 95
memory_warning = 85
disk_critical = 95
disk_warning = 85

# Self-healing policies
[self_healing]
enabled = false
max_actions_per_hour = 3
allow_service_restart = false
allow_package_update = false
"#;

/// Execute init command - first-run wizard
pub async fn execute_init_command() -> Result<()> {
    println!("🚀 Anna Assistant - First Run Wizard");
    println!("════════════════════════════════════════════════════════════════");
    println!();

    // Check if already initialized
    if Path::new(CONFIG_FILE).exists() {
        println!("⚠️  Anna is already initialized!");
        println!();
        println!("Configuration directory: {}", CONFIG_DIR);
        println!("Config file: {}", CONFIG_FILE);
        println!("Sentinel config: {}", SENTINEL_FILE);
        println!();
        println!("To reinitialize, remove the config directory:");
        println!("  sudo rm -rf {}", CONFIG_DIR);
        println!();
        std::process::exit(2);
    }

    // Detect system constraints
    let system_info = detect_system_constraints();

    println!("System Detection:");
    println!("  Memory:         {} MB", system_info.total_memory_mb);
    println!("  Virtualization: {}", system_info.virtualization);
    println!(
        "  Constrained:    {}",
        if system_info.is_tiny {
            "Yes (tiny system)"
        } else {
            "No"
        }
    );
    println!();

    // Show recommended mode
    let recommended_mode = if system_info.is_tiny {
        "minimal"
    } else if system_info.total_memory_mb < 4096 {
        "light"
    } else {
        "full"
    };

    println!(
        "Recommended monitoring mode: {}",
        recommended_mode.to_uppercase()
    );
    println!();

    if system_info.is_tiny {
        println!("💡 Your system is resource-constrained (< 2GB RAM).");
        println!("   Anna will operate in MINIMAL mode for best performance.");
        println!();
    }

    // Create config directory
    println!("Creating configuration directory...");
    if let Err(e) = fs::create_dir_all(CONFIG_DIR) {
        eprintln!("❌ Failed to create {}: {}", CONFIG_DIR, e);
        eprintln!();
        eprintln!("This command requires root permissions.");
        eprintln!("Try: sudo annactl init");
        std::process::exit(1);
    }
    println!("  ✓ Created {}", CONFIG_DIR);

    // Write default config
    println!("Writing default configuration...");
    if let Err(e) = fs::write(CONFIG_FILE, DEFAULT_CONFIG) {
        eprintln!("❌ Failed to write {}: {}", CONFIG_FILE, e);
        eprintln!();
        eprintln!("This command requires root permissions.");
        eprintln!("Try: sudo annactl init");
        std::process::exit(1);
    }
    println!("  ✓ Created {}", CONFIG_FILE);

    // Write sentinel config
    if let Err(e) = fs::write(SENTINEL_FILE, DEFAULT_SENTINEL) {
        eprintln!("❌ Failed to write {}: {}", SENTINEL_FILE, e);
        eprintln!();
        eprintln!("This command requires root permissions.");
        eprintln!("Try: sudo annactl init");
        std::process::exit(1);
    }
    println!("  ✓ Created {}", SENTINEL_FILE);

    println!();
    println!("════════════════════════════════════════════════════════════════");
    println!("🎉 Initialization Complete!");
    println!("════════════════════════════════════════════════════════════════");
    println!();

    // Show the 3 safest commands
    println!("📖 Getting Started - The Safest Commands:");
    println!();
    println!("  [32mannactl help[39m     - Show all available commands");
    println!("  [32mannactl status[39m   - View system state and health");
    println!("  [32mannactl health[39m   - Check detailed health probes");
    println!("  [32mannactl profile[39m  - View system profile and recommendations");
    println!();
    println!("  These commands are read-only and safe to run anytime.");
    println!();

    // Show monitoring information
    println!("📊 Monitoring:");
    println!();
    match recommended_mode {
        "minimal" => {
            println!("  Internal monitoring is active (no external tools needed)");
            println!("  Use: annactl status, annactl health");
        }
        "light" => {
            println!("  Prometheus metrics available after installation:");
            println!("    http://localhost:9090");
            println!();
            println!("  Install: annactl monitor install");
        }
        "full" => {
            println!("  Full monitoring stack available after installation:");
            println!("    Grafana:    http://localhost:3000 (admin/admin)");
            println!("    Prometheus: http://localhost:9090");
            println!();
            println!("  Install: annactl monitor install");
        }
        _ => {}
    }
    println!();

    // Show predictive intelligence info
    println!("🔮 Predictive Intelligence:");
    println!();
    println!("  Anna learns from your system's behavior and can predict issues.");
    println!("  After running a few commands, try:");
    println!("    annactl learn    - Detect patterns in system behavior");
    println!("    annactl predict  - View predictions and recommendations");
    println!();

    // Show daemon start instructions
    println!("🔧 Next Steps:");
    println!();
    println!("  1. Enable and start the Anna daemon:");
    println!("     [33msudo systemctl enable --now annad[39m");
    println!();
    println!("  2. Check daemon status:");
    println!("     [33msudo systemctl status annad[39m");
    println!();
    println!("  3. Run your first status check:");
    println!("     [33mannactl status[39m");
    println!();

    // Security note
    println!("🔒 Security Notes:");
    println!();
    println!("  • Administrative commands require sudo (update, install, etc.)");
    println!("  • Self-healing is DISABLED by default (see /etc/anna/sentinel.toml)");
    println!("  • All actions are logged to /var/log/anna/");
    println!();

    println!("For detailed documentation, see:");
    println!("  https://docs.anna-assistant.org");
    println!();

    Ok(())
}

/// Detect system constraints
struct SystemInfo {
    total_memory_mb: u64,
    virtualization: String,
    is_tiny: bool,
}

fn detect_system_constraints() -> SystemInfo {
    let total_memory_mb = get_total_memory_mb();
    let virtualization = detect_virtualization();
    let is_tiny = total_memory_mb < 2048;

    SystemInfo {
        total_memory_mb,
        virtualization,
        is_tiny,
    }
}

/// Get total system memory in MB
fn get_total_memory_mb() -> u64 {
    // Read from /proc/meminfo
    if let Ok(contents) = fs::read_to_string("/proc/meminfo") {
        for line in contents.lines() {
            if line.starts_with("MemTotal:") {
                if let Some(kb_str) = line.split_whitespace().nth(1) {
                    if let Ok(kb) = kb_str.parse::<u64>() {
                        return kb / 1024; // Convert KB to MB
                    }
                }
            }
        }
    }

    // Fallback to 0 if we can't read
    0
}

/// Detect virtualization type
fn detect_virtualization() -> String {
    // Try systemd-detect-virt
    if let Ok(output) = Command::new("systemd-detect-virt").output() {
        if output.status.success() {
            if let Ok(virt) = String::from_utf8(output.stdout) {
                let virt = virt.trim();
                if virt != "none" {
                    return virt.to_string();
                }
            }
        }
    }

    "none (bare metal)".to_string()
}
