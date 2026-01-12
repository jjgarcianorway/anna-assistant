//! 500 Questions Test - Comprehensive pattern coverage analysis
//! Tests Anna's pattern matching against 500 diverse Linux questions

use super::match_common_pattern;

/// All 500 test questions organized by category
const TEST_QUESTIONS: &[(&str, &str)] = &[
    // === PACMAN (1-15) ===
    ("pacman database is locked", "pacman"),
    ("how do I update my system", "pacman"),
    ("what packages are installed", "pacman"),
    ("pacman says file exists in filesystem", "pacman"),
    ("how to clean pacman cache", "pacman"),
    ("remove orphan packages", "pacman"),
    ("pacman key issues", "pacman"),
    ("downgrade a package", "pacman"),
    ("list explicitly installed packages", "pacman"),
    ("find which package owns a file", "pacman"),
    ("pacman mirror issues", "pacman"),
    ("how to reinstall all packages", "pacman"),
    ("pacman partial upgrade warning", "pacman"),
    ("check for broken packages", "pacman"),
    ("pacman hook failed", "pacman"),

    // === ERRORS (16-30) ===
    ("failed to start service", "errors"),
    ("permission denied when running sudo", "errors"),
    ("command not found", "errors"),
    ("no space left on device", "errors"),
    ("cannot allocate memory", "errors"),
    ("segmentation fault", "errors"),
    ("dependency resolution failed", "errors"),
    ("kernel panic not syncing", "errors"),
    ("unable to find expected entry", "errors"),
    ("grub error unknown filesystem", "errors"),
    ("nvidia driver not loading", "errors"),
    ("pulseaudio connection refused", "errors"),
    ("dbus connection failed", "errors"),
    ("glibc version mismatch", "errors"),
    ("libstdc++ not found", "errors"),

    // === RECOVERY (31-45) ===
    ("I deleted /usr/bin by accident", "recovery"),
    ("system won't boot", "recovery"),
    ("corrupted pacman database", "recovery"),
    ("how to enter single user mode", "recovery"),
    ("boot into rescue mode", "recovery"),
    ("fix broken grub", "recovery"),
    ("recover from failed update", "recovery"),
    ("chroot into broken system", "recovery"),
    ("reinstall bootloader", "recovery"),
    ("fix fstab mistake", "recovery"),
    ("recover deleted files", "recovery"),
    ("system hangs on boot", "recovery"),
    ("fix broken initramfs", "recovery"),
    ("restore from snapshot", "recovery"),
    ("emergency shell at boot", "recovery"),

    // === PERFORMANCE (46-60) ===
    ("system is slow", "performance"),
    ("high cpu usage", "performance"),
    ("high memory usage", "performance"),
    ("disk io is slow", "performance"),
    ("why is my fan spinning", "performance"),
    ("system feels laggy", "performance"),
    ("reduce boot time", "performance"),
    ("optimize for gaming", "performance"),
    ("improve battery life", "performance"),
    ("reduce swap usage", "performance"),
    ("tune for ssd", "performance"),
    ("profile application performance", "performance"),
    ("find memory leaks", "performance"),
    ("reduce latency", "performance"),
    ("optimize network performance", "performance"),

    // === FACTUAL (61-85) ===
    ("what is my disk usage", "factual"),
    ("how much ram do I have", "factual"),
    ("what gpu do I have", "factual"),
    ("what cpu do I have", "factual"),
    ("what kernel am I running", "factual"),
    ("show system uptime", "factual"),
    ("list running services", "factual"),
    ("what is my ip address", "factual"),
    ("what distro am I running", "factual"),
    ("show available disk space", "factual"),
    ("list installed packages count", "factual"),
    ("what is my hostname", "factual"),
    ("show cpu temperature", "factual"),
    ("what is my shell", "factual"),
    ("show environment variables", "factual"),
    ("list all users", "factual"),
    ("what groups am I in", "factual"),
    ("show open files limit", "factual"),
    ("what is my default gateway", "factual"),
    ("show dns servers", "factual"),
    ("list mounted filesystems", "factual"),
    ("what is my timezone", "factual"),
    ("show current date and time", "factual"),
    ("what is my locale", "factual"),
    ("show kernel parameters", "factual"),

    // === DEVELOPMENT (86-105) ===
    ("git status", "development"),
    ("git log", "development"),
    ("docker containers", "development"),
    ("docker images", "development"),
    ("list running containers", "development"),
    ("show docker networks", "development"),
    ("npm version", "development"),
    ("node version", "development"),
    ("python version", "development"),
    ("cargo version", "development"),
    ("rustc version", "development"),
    ("gcc version", "development"),
    ("show git remotes", "development"),
    ("list git branches", "development"),
    ("check go version", "development"),
    ("java version", "development"),
    ("ruby version", "development"),
    ("pip list", "development"),
    ("show virtualenvs", "development"),
    ("list systemd services", "development"),

    // === SECURITY (106-125) ===
    ("firewall status", "security"),
    ("list open ports", "security"),
    ("who has sudo access", "security"),
    ("show ssh keys", "security"),
    ("check failed login attempts", "security"),
    ("list firewall rules", "security"),
    ("show active connections", "security"),
    ("audit sudo usage", "security"),
    ("check for rootkits", "security"),
    ("show selinux status", "security"),
    ("list authorized ssh keys", "security"),
    ("show passwd permissions", "security"),
    ("check suid files", "security"),
    ("show iptables rules", "security"),
    ("list listening services", "security"),
    ("ssh brute force attempts", "security"),
    ("check file integrity", "security"),
    ("security audit", "security"),
    ("intrusion detection", "security"),
    ("malware scan", "security"),

    // === DESKTOP (126-145) ===
    ("wayland or x11", "desktop"),
    ("which desktop environment", "desktop"),
    ("gnome version", "desktop"),
    ("kde version", "desktop"),
    ("list installed themes", "desktop"),
    ("show gtk theme", "desktop"),
    ("show icon theme", "desktop"),
    ("screen resolution", "desktop"),
    ("list connected monitors", "desktop"),
    ("show display manager", "desktop"),
    ("check compositor status", "desktop"),
    ("show desktop notifications", "desktop"),
    ("window manager info", "desktop"),
    ("show cursor theme", "desktop"),
    ("list available fonts", "desktop"),
    ("plasma settings", "desktop"),
    ("gnome extensions", "desktop"),
    ("desktop shortcuts", "desktop"),
    ("taskbar configuration", "desktop"),
    ("desktop icons", "desktop"),

    // === HOWTO (146-170) ===
    ("how to install a package", "howto"),
    ("how to update system", "howto"),
    ("how to enable a service", "howto"),
    ("how to add a user", "howto"),
    ("how to change permissions", "howto"),
    ("how to mount a drive", "howto"),
    ("how to create a partition", "howto"),
    ("how to set up ssh", "howto"),
    ("how to configure network", "howto"),
    ("how to change hostname", "howto"),
    ("how to install from aur", "howto"),
    ("how to create systemd service", "howto"),
    ("how to set up cron job", "howto"),
    ("how to configure firewall", "howto"),
    ("how to backup system", "howto"),
    ("how to resize partition", "howto"),
    ("how to encrypt disk", "howto"),
    ("how to configure dns", "howto"),
    ("how to set static ip", "howto"),
    ("how to enable auto login", "howto"),
    ("how to disable root login", "howto"),
    ("how to set up vnc", "howto"),
    ("how to configure samba", "howto"),
    ("how to set up nfs", "howto"),
    ("how to install nvidia drivers", "howto"),

    // === NETWORK (171-195) ===
    ("am i connected to internet", "network"),
    ("wifi status", "network"),
    ("network interfaces", "network"),
    ("what is my public ip", "network"),
    ("show default gateway", "network"),
    ("list dns servers", "network"),
    ("check vpn status", "network"),
    ("show open ports", "network"),
    ("active network connections", "network"),
    ("test dns resolution", "network"),
    ("ping statistics", "network"),
    ("check network speed", "network"),
    ("show routing table", "network"),
    ("list wifi networks", "network"),
    ("show network traffic", "network"),
    ("bandwidth usage", "network"),
    ("network latency", "network"),
    ("check firewall ports", "network"),
    ("show mac address", "network"),
    ("traceroute to google", "network"),
    ("list network services", "network"),
    ("show arp table", "network"),
    ("check for packet loss", "network"),
    ("network interface statistics", "network"),
    ("show netstat output", "network"),

    // === HARDWARE (196-220) ===
    ("cpu temperature", "hardware"),
    ("gpu temperature", "hardware"),
    ("list usb devices", "hardware"),
    ("list pci devices", "hardware"),
    ("show battery status", "hardware"),
    ("disk health", "hardware"),
    ("memory info", "hardware"),
    ("cpu info", "hardware"),
    ("show hardware sensors", "hardware"),
    ("list block devices", "hardware"),
    ("show motherboard info", "hardware"),
    ("check power supply", "hardware"),
    ("list input devices", "hardware"),
    ("show graphics card", "hardware"),
    ("check disk smart", "hardware"),
    ("show cpu frequency", "hardware"),
    ("list nvme devices", "hardware"),
    ("check fan speed", "hardware"),
    ("show dimm info", "hardware"),
    ("bios version", "hardware"),
    ("firmware info", "hardware"),
    ("list serial devices", "hardware"),
    ("show thermal zones", "hardware"),
    ("check acpi info", "hardware"),
    ("list sata devices", "hardware"),

    // === GAMING (221-245) ===
    ("steam games list", "gaming"),
    ("wine version", "gaming"),
    ("proton version", "gaming"),
    ("check vulkan support", "gaming"),
    ("opengl version", "gaming"),
    ("show gamepad devices", "gaming"),
    ("lutris games", "gaming"),
    ("check steam runtime", "gaming"),
    ("show dxvk version", "gaming"),
    ("gaming mouse setup", "gaming"),
    ("controller configuration", "gaming"),
    ("show mangohud stats", "gaming"),
    ("check vkbasalt", "gaming"),
    ("gaming performance", "gaming"),
    ("show gpu usage while gaming", "gaming"),
    ("steam proton logs", "gaming"),
    ("wine prefix info", "gaming"),
    ("check gamemode status", "gaming"),
    ("esync vs fsync", "gaming"),
    ("show shader cache", "gaming"),
    ("nvidia prime status", "gaming"),
    ("amd gpu gaming setup", "gaming"),
    ("steam library location", "gaming"),
    ("protondb compatibility", "gaming"),
    ("gaming latency check", "gaming"),

    // === BOOT (246-270) ===
    ("grub configuration", "boot"),
    ("boot entries", "boot"),
    ("kernel parameters", "boot"),
    ("initramfs info", "boot"),
    ("show boot time", "boot"),
    ("boot order", "boot"),
    ("efi variables", "boot"),
    ("secure boot status", "boot"),
    ("grub theme", "boot"),
    ("boot menu timeout", "boot"),
    ("list installed kernels", "boot"),
    ("show boot device", "boot"),
    ("check uefi mode", "boot"),
    ("boot logs", "boot"),
    ("systemd boot analysis", "boot"),
    ("grub rescue info", "boot"),
    ("boot partition info", "boot"),
    ("show initrd contents", "boot"),
    ("check boot loader", "boot"),
    ("plymouth status", "boot"),
    ("silent boot setup", "boot"),
    ("boot splash config", "boot"),
    ("grub password setup", "boot"),
    ("dual boot config", "boot"),
    ("boot repair info", "boot"),

    // === CONTAINER (271-295) ===
    ("list docker containers", "container"),
    ("docker images list", "container"),
    ("podman containers", "container"),
    ("docker compose status", "container"),
    ("container logs", "container"),
    ("docker networks list", "container"),
    ("docker volumes", "container"),
    ("container stats", "container"),
    ("docker system info", "container"),
    ("podman pods", "container"),
    ("running containers list", "container"),
    ("container resource usage", "container"),
    ("docker registry info", "container"),
    ("container ports", "container"),
    ("docker build cache", "container"),
    ("podman images", "container"),
    ("container health check", "container"),
    ("docker swarm status", "container"),
    ("kubernetes pods", "container"),
    ("container runtime info", "container"),
    ("docker daemon status", "container"),
    ("container filesystem", "container"),
    ("docker inspect output", "container"),
    ("container environment vars", "container"),
    ("docker prune info", "container"),

    // === LOGS (296-320) ===
    ("recent system logs", "logs"),
    ("boot logs", "logs"),
    ("kernel messages", "logs"),
    ("service logs", "logs"),
    ("error logs", "logs"),
    ("authentication logs", "logs"),
    ("sudo logs", "logs"),
    ("crash logs", "logs"),
    ("dmesg output", "logs"),
    ("journalctl errors", "logs"),
    ("application logs", "logs"),
    ("security logs", "logs"),
    ("audit logs", "logs"),
    ("network logs", "logs"),
    ("hardware logs", "logs"),
    ("xorg logs", "logs"),
    ("systemd logs", "logs"),
    ("failed services log", "logs"),
    ("login history", "logs"),
    ("cron logs", "logs"),
    ("syslog contents", "logs"),
    ("daemon logs", "logs"),
    ("kernel ring buffer", "logs"),
    ("journal size", "logs"),
    ("rotate logs", "logs"),

    // === AUDIO (321-345) ===
    ("no sound output", "audio"),
    ("audio devices", "audio"),
    ("volume level", "audio"),
    ("pipewire status", "audio"),
    ("pulseaudio info", "audio"),
    ("alsa devices", "audio"),
    ("bluetooth audio", "audio"),
    ("audio sinks", "audio"),
    ("microphone input", "audio"),
    ("sound card info", "audio"),
    ("audio latency", "audio"),
    ("sample rate", "audio"),
    ("audio routing", "audio"),
    ("speaker test", "audio"),
    ("headphone detection", "audio"),
    ("midi devices", "audio"),
    ("audio mixing", "audio"),
    ("jack audio status", "audio"),
    ("sound server info", "audio"),
    ("audio codecs", "audio"),
    ("equalizer settings", "audio"),
    ("audio profiles", "audio"),
    ("default audio device", "audio"),
    ("audio troubleshoot", "audio"),
    ("music player status", "audio"),

    // === POWER (346-365) ===
    ("battery status", "power"),
    ("battery health", "power"),
    ("charging status", "power"),
    ("power consumption", "power"),
    ("suspend settings", "power"),
    ("hibernate config", "power"),
    ("screen brightness", "power"),
    ("power profiles", "power"),
    ("cpu governor", "power"),
    ("fan control", "power"),
    ("thermal throttling", "power"),
    ("power button action", "power"),
    ("lid switch action", "power"),
    ("wake on lan", "power"),
    ("power saving tips", "power"),
    ("tlp status", "power"),
    ("auto suspend", "power"),
    ("battery calibration", "power"),
    ("power statistics", "power"),
    ("acpi events", "power"),

    // === SYSTEMD (366-390) ===
    ("failed services", "systemd"),
    ("running services", "systemd"),
    ("list all services", "systemd"),
    ("service dependencies", "systemd"),
    ("systemd timers", "systemd"),
    ("default target", "systemd"),
    ("service status", "systemd"),
    ("unit file location", "systemd"),
    ("enable disable service", "systemd"),
    ("service logs", "systemd"),
    ("systemd analyze blame", "systemd"),
    ("list socket units", "systemd"),
    ("service restart policy", "systemd"),
    ("show service config", "systemd"),
    ("mask unmask service", "systemd"),
    ("systemd user services", "systemd"),
    ("service environment", "systemd"),
    ("service resource limits", "systemd"),
    ("service security", "systemd"),
    ("transient services", "systemd"),
    ("boot target info", "systemd"),
    ("service ordering", "systemd"),
    ("slice units", "systemd"),
    ("scope units", "systemd"),
    ("path units", "systemd"),

    // === FILESYSTEM (391-415) ===
    ("disk usage", "filesystem"),
    ("list mounts", "filesystem"),
    ("fstab contents", "filesystem"),
    ("partition table", "filesystem"),
    ("uuid of drives", "filesystem"),
    ("inode usage", "filesystem"),
    ("largest files", "filesystem"),
    ("directory sizes", "filesystem"),
    ("filesystem type", "filesystem"),
    ("mount options", "filesystem"),
    ("resize filesystem", "filesystem"),
    ("check filesystem", "filesystem"),
    ("disk quotas", "filesystem"),
    ("filesystem labels", "filesystem"),
    ("block size", "filesystem"),
    ("journal status", "filesystem"),
    ("extended attributes", "filesystem"),
    ("acl permissions", "filesystem"),
    ("sparse files", "filesystem"),
    ("deduplication", "filesystem"),
    ("compression ratio", "filesystem"),
    ("snapshot info", "filesystem"),
    ("trim status", "filesystem"),
    ("filesystem stats", "filesystem"),
    ("mount points", "filesystem"),

    // === PROCESS (416-440) ===
    ("running processes", "process"),
    ("process tree", "process"),
    ("top cpu processes", "process"),
    ("top memory processes", "process"),
    ("zombie processes", "process"),
    ("background jobs", "process"),
    ("process priority", "process"),
    ("kill process howto", "process"),
    ("process threads", "process"),
    ("process io", "process"),
    ("process open files", "process"),
    ("process environment", "process"),
    ("process limits", "process"),
    ("process status", "process"),
    ("process cpu time", "process"),
    ("process memory map", "process"),
    ("process signals", "process"),
    ("process user", "process"),
    ("process parent", "process"),
    ("process children", "process"),
    ("defunct processes", "process"),
    ("sleeping processes", "process"),
    ("process groups", "process"),
    ("session leaders", "process"),
    ("foreground processes", "process"),

    // === CRON (441-455) ===
    ("my crontab", "cron"),
    ("system cron jobs", "cron"),
    ("cron daily jobs", "cron"),
    ("cron weekly jobs", "cron"),
    ("atq", "cron"),
    ("scheduled tasks", "cron"),
    ("cron syntax help", "cron"),
    ("anacron status", "cron"),
    ("cron logs", "cron"),
    ("timer units", "cron"),
    ("cron environment", "cron"),
    ("cron mail", "cron"),
    ("cron permissions", "cron"),
    ("cron alternatives", "cron"),
    ("task scheduler status", "cron"),

    // === USERS (456-470) ===
    ("list all users", "users"),
    ("logged in users", "users"),
    ("user groups", "users"),
    ("last logins", "users"),
    ("failed logins", "users"),
    ("user home directory", "users"),
    ("user shell", "users"),
    ("password expiry", "users"),
    ("user permissions", "users"),
    ("add user howto", "users"),
    ("delete user howto", "users"),
    ("change password howto", "users"),
    ("user quotas", "users"),
    ("locked users", "users"),
    ("root login status", "users"),

    // === TIME (471-480) ===
    ("current time", "time"),
    ("system uptime", "time"),
    ("timezone info", "time"),
    ("ntp status", "time"),
    ("time sync", "time"),
    ("hardware clock", "time"),
    ("list timezones", "time"),
    ("time drift", "time"),
    ("chrony status", "time"),
    ("rtc info", "time"),

    // === PRINTING (481-488) ===
    ("list printers", "printing"),
    ("printer status", "printing"),
    ("print queue", "printing"),
    ("cups status", "printing"),
    ("default printer", "printing"),
    ("print jobs", "printing"),
    ("printer drivers", "printing"),
    ("cups logs", "printing"),

    // === BACKUP (489-496) ===
    ("rsync version", "backup"),
    ("borg backups", "backup"),
    ("restic snapshots", "backup"),
    ("tar archives", "backup"),
    ("backup schedule", "backup"),
    ("incremental backup", "backup"),
    ("restore backup howto", "backup"),
    ("backup verification", "backup"),

    // === LOCALE (497-500) ===
    ("keyboard layout", "locale"),
    ("available locales", "locale"),
    ("current locale", "locale"),
    ("font rendering", "locale"),
];

#[test]
fn test_500_questions_coverage() {
    let mut total = 0;
    let mut matched = 0;
    let mut unmatched = Vec::new();
    let mut category_stats: std::collections::HashMap<&str, (u32, u32)> = std::collections::HashMap::new();

    for (question, category) in TEST_QUESTIONS {
        total += 1;
        let entry = category_stats.entry(category).or_insert((0, 0));
        entry.1 += 1; // total for category

        if match_common_pattern(question).is_some() {
            matched += 1;
            entry.0 += 1; // matched for category
        } else {
            unmatched.push((question, category));
        }
    }

    let coverage = (matched as f64 / total as f64) * 100.0;

    println!("\n========================================");
    println!("  500 QUESTIONS TEST RESULTS");
    println!("========================================\n");
    println!("Total Questions: {}", total);
    println!("Pattern Matches: {} ({:.1}%)", matched, coverage);
    println!("Unmatched: {}", total - matched);
    println!("\n--- Category Breakdown ---\n");

    let mut categories: Vec<_> = category_stats.iter().collect();
    categories.sort_by_key(|(name, _)| *name);

    for (category, (cat_matched, cat_total)) in &categories {
        let cat_pct = (*cat_matched as f64 / *cat_total as f64) * 100.0;
        let status = if cat_pct >= 80.0 { "OK" } else if cat_pct >= 50.0 { "FAIR" } else { "LOW" };
        println!("{:15} {:2}/{:2} ({:5.1}%) [{}]", category, cat_matched, cat_total, cat_pct, status);
    }

    if !unmatched.is_empty() && unmatched.len() <= 50 {
        println!("\n--- Unmatched Questions (first 50) ---\n");
        for (q, cat) in unmatched.iter().take(50) {
            println!("[{}] {}", cat, q);
        }
    }

    println!("\n========================================");
    println!("  COVERAGE: {:.1}%", coverage);
    println!("========================================\n");

    // Assert we have good coverage (at least 50%)
    assert!(coverage >= 50.0, "Pattern coverage too low: {:.1}%", coverage);
}

#[test]
fn test_pattern_categories_exist() {
    // Verify we have questions for all major categories
    let categories: std::collections::HashSet<&str> = TEST_QUESTIONS.iter().map(|(_, cat)| *cat).collect();

    let expected = vec![
        "pacman", "errors", "recovery", "performance", "factual",
        "development", "security", "desktop", "howto", "network",
        "hardware", "gaming", "boot", "container", "logs",
        "audio", "power", "systemd", "filesystem", "process",
        "cron", "users", "time", "printing", "backup", "locale"
    ];

    for cat in expected {
        assert!(categories.contains(cat), "Missing category: {}", cat);
    }
}
