//! Misc recommendations

use anna_common::{Advice, Alternative, Priority, RiskLevel, SystemFacts};
use std::path::Path;
use std::process::Command;

pub(crate) fn check_command_usage(commands: &[&str]) -> usize {
    let mut count = 0;

    // Try to read bash history
    if let Ok(home) = std::env::var("HOME") {
        let bash_history = Path::new(&home).join(".bash_history");
        if let Ok(contents) = std::fs::read_to_string(bash_history) {
            for cmd in commands {
                count += contents.lines().filter(|line| line.contains(cmd)).count();
            }
        }

        // Also try zsh history
        let zsh_history = Path::new(&home).join(".zsh_history");
        if let Ok(contents) = std::fs::read_to_string(zsh_history) {
            for cmd in commands {
                count += contents.lines().filter(|line| line.contains(cmd)).count();
            }
        }
    }

    count
}

pub(crate) fn check_system_monitors() -> Vec<Advice> {
    let mut result = Vec::new();

    // btop - modern system monitor
    if !Command::new("which")
        .arg("btop")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        result.push(Advice {
            id: "install-btop".to_string(),
            title: "Install btop - beautiful resource monitor".to_string(),
            reason: "btop is a gorgeous, modern alternative to htop/top with mouse support, themes, and graphs. Shows CPU, memory, disks, network, and processes in a beautiful TUI!".to_string(),
            action: "Install btop".to_string(),
            command: Some("sudo pacman -S --noconfirm btop".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "System Utilities".to_string(),
            alternatives: vec![
                Alternative {
                    name: "htop".to_string(),
                    description: "Classic interactive process viewer".to_string(),
                    install_command: "sudo pacman -S --noconfirm htop".to_string(),
                },
            ],
            wiki_refs: vec!["https://wiki.archlinux.org/title/System_monitor".to_string()],
            depends_on: Vec::new(),
            related_to: Vec::new(),
            bundle: None,
            satisfies: Vec::new(),
            popularity: 85,
            requires: Vec::new(),
        });
    }

    result
}

pub(crate) fn check_arch_specific_tools() -> Vec<Advice> {
    let mut result = Vec::new();

    // arch-audit - security audit tool
    if !Command::new("which")
        .arg("arch-audit")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        result.push(Advice {
            id: "install-arch-audit".to_string(),
            title: "Install arch-audit - check for security vulnerabilities".to_string(),
            reason: "arch-audit scans your installed packages for known security vulnerabilities (CVEs). Essential for keeping your system secure!".to_string(),
            action: "Install arch-audit".to_string(),
            command: Some("sudo pacman -S --noconfirm arch-audit".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "Security & Privacy".to_string(),
            alternatives: Vec::new(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Security#Vulnerability_scanning".to_string()],
            depends_on: Vec::new(),
            related_to: Vec::new(),
            bundle: Some("security-hardening".to_string()),
            satisfies: Vec::new(),
            popularity: 70,
            requires: Vec::new(),
        });
    }

    // pkgfile - command-not-found handler
    if !Command::new("which")
        .arg("pkgfile")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        result.push(Advice {
            id: "install-pkgfile".to_string(),
            title: "Install pkgfile - find which package owns a file".to_string(),
            reason: "pkgfile tells you which package contains a command or file. Type a missing command and it suggests which package to install! Also enables 'command-not-found' handler.".to_string(),
            action: "Install pkgfile and update database".to_string(),
            command: Some("sudo pacman -S --noconfirm pkgfile && sudo pkgfile --update".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "Package Management".to_string(),
            alternatives: Vec::new(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Pkgfile".to_string()],
            depends_on: Vec::new(),
            related_to: Vec::new(),
            bundle: None,
            satisfies: Vec::new(),
            popularity: 75,
            requires: Vec::new(),
        });
    }

    // pacman-contrib - additional pacman tools
    if !Command::new("which")
        .arg("paccache")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        result.push(Advice {
            id: "install-pacman-contrib".to_string(),
            title: "Install pacman-contrib - extra pacman utilities".to_string(),
            reason: "pacman-contrib provides useful tools like paccache (clean package cache), checkupdates (check updates without -Sy), pacdiff (merge .pacnew files), and more!".to_string(),
            action: "Install pacman-contrib".to_string(),
            command: Some("sudo pacman -S --noconfirm pacman-contrib".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "Package Management".to_string(),
            alternatives: Vec::new(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Pacman#Utilities".to_string()],
            depends_on: Vec::new(),
            related_to: Vec::new(),
            bundle: None,
            satisfies: Vec::new(),
            popularity: 80,
            requires: Vec::new(),
        });
    }

    result
}

pub(crate) fn check_config_files() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for .vimrc
    let vimrc_path =
        std::path::Path::new(&std::env::var("HOME").unwrap_or_default()).join(".vimrc");
    if vimrc_path.exists() {
        let vimrc_size = std::fs::metadata(&vimrc_path).map(|m| m.len()).unwrap_or(0);

        // If vimrc is very small (< 500 bytes), suggest improvements
        if vimrc_size < 500 {
            result.push(Advice {
                id: "improve-vimrc".to_string(),
                title: "Enhance your .vimrc configuration".to_string(),
                reason: "Your .vimrc is pretty minimal. A well-configured vim makes editing so much better! Add line numbers, syntax highlighting, smart indentation, and better search. These are quality-of-life improvements every vim user should have.".to_string(),
                action: "Add essential vim configurations".to_string(),
                command: Some("cat ~/.vimrc 2>/dev/null || echo 'No .vimrc found'".to_string()), // Show current vimrc
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Development Tools".to_string(),
                alternatives: vec![
                    Alternative {
                        name: "neovim".to_string(),
                        description: "Modern vim with better defaults".to_string(),
                        install_command: "pacman -S neovim".to_string(),
                    },
                ],
                wiki_refs: vec!["https://wiki.archlinux.org/title/Vim".to_string()],
                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
                popularity: 60,
            requires: Vec::new(),
            });
        }
    }

    // Check for .bashrc/.zshrc
    let shell = std::env::var("SHELL").unwrap_or_default();
    let rcfile = if shell.contains("zsh") {
        ".zshrc"
    } else {
        ".bashrc"
    };

    let rc_path = std::path::Path::new(&std::env::var("HOME").unwrap_or_default()).join(rcfile);
    if rc_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&rc_path) {
            // Check for useful aliases
            let has_aliases = content.contains("alias");

            if !has_aliases || content.lines().filter(|l| l.contains("alias")).count() < 3 {
                result.push(Advice {
                    id: format!("improve-{}", rcfile),
                    title: format!("Add useful aliases to your {}", rcfile),
                    reason: format!("Your {} could use some helpful aliases! Common ones like 'll' for 'ls -lah', 'update' for system updates, or 'grep' with colors save tons of typing. Every power user has their favorite aliases.", rcfile),
                    action: format!("Consider adding useful aliases to {}", rcfile),
                    command: Some(format!("grep -n '^[[:space:]]*alias' ~/{} 2>/dev/null || echo 'No aliases found'", rcfile)), // Show current aliases
                    risk: RiskLevel::Low,
                    priority: Priority::Optional,
                    category: "Shell & Terminal".to_string(),
                    alternatives: Vec::new(),
                    wiki_refs: vec![
                        "https://wiki.archlinux.org/title/Bash#Aliases".to_string(),
                        "https://wiki.archlinux.org/title/Zsh#Aliases".to_string(),
                    ],
                    depends_on: Vec::new(),
                    related_to: Vec::new(),
                    bundle: None,
            satisfies: Vec::new(),
                    popularity: 70,
            requires: Vec::new(),
                });
            }
        }
    }

    result
}

pub(crate) fn check_power_management() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if system is a laptop (has battery)
    let is_laptop = std::path::Path::new("/sys/class/power_supply/BAT0").exists()
        || std::path::Path::new("/sys/class/power_supply/BAT1").exists();

    if is_laptop {
        // Check for TLP
        let has_tlp = Command::new("pacman")
            .args(&["-Q", "tlp"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_tlp {
            result.push(Advice {
                id: "laptop-tlp".to_string(),
                title: "Install TLP for better laptop battery life".to_string(),
                reason: "TLP automatically optimizes your laptop's power settings. It can significantly extend your battery life by managing CPU frequencies, disk write behavior, USB power, and more. Just install it and forget it - it works automatically!".to_string(),
                action: "Install TLP for power management".to_string(),
                command: Some("pacman -S --noconfirm tlp && systemctl enable tlp.service".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Power Management".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/TLP".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check for powertop
        let has_powertop = Command::new("pacman")
            .args(&["-Q", "powertop"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_powertop {
            result.push(Advice {
                id: "laptop-powertop".to_string(),
                title: "Install powertop to analyze power consumption".to_string(),
                reason: "powertop shows you exactly what's draining your battery. It lists processes, devices, and can even suggest tuning options. Great for diagnosing battery issues and seeing the impact of your power settings!".to_string(),
                action: "Install powertop".to_string(),
                command: Some("pacman -S --noconfirm powertop".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Power Management".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Powertop".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_usb_automount() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for udisks2 (required for automounting)
    let has_udisks2 = Command::new("pacman")
        .args(&["-Q", "udisks2"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_udisks2 {
        result.push(Advice {
            id: "udisks2".to_string(),
            title: "Install udisks2 for USB drive management".to_string(),
            reason: "udisks2 handles mounting and unmounting USB drives, external hard drives, and SD cards. Most file managers depend on it for automatic mounting. Without it, you'll have to mount drives manually with command line!".to_string(),
            action: "Install udisks2".to_string(),
            command: Some("pacman -S --noconfirm udisks2".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "Desktop Environment".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Udisks".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    } else {
        // If they have udisks2, suggest udiskie for automatic mounting
        let has_udiskie = Command::new("pacman")
            .args(&["-Q", "udiskie"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_udiskie {
            result.push(Advice {
                id: "udiskie".to_string(),
                title: "Install udiskie for automatic USB mounting".to_string(),
                reason: "udiskie automatically mounts USB drives when you plug them in and unmounts when you unplug. No more clicking 'mount' every time! Just plug and play. It's especially great for minimal window managers without built-in automounting.".to_string(),
                action: "Install udiskie".to_string(),
                command: Some("pacman -S --noconfirm udiskie".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Desktop Environment".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Udisks#Udiskie".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_printer_support() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if CUPS is installed
    let has_cups = Command::new("pacman")
        .args(&["-Q", "cups"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // Check for USB printers (lsusb)
    let has_printer = Command::new("lsusb")
        .output()
        .map(|o| {
            let output = String::from_utf8_lossy(&o.stdout).to_lowercase();
            output.contains("printer")
                || output.contains("canon")
                || output.contains("hp")
                || output.contains("epson")
                || output.contains("brother")
        })
        .unwrap_or(false);

    if !has_cups && has_printer {
        result.push(Advice {
            id: "cups-install".to_string(),
            title: "Install CUPS for printer support".to_string(),
            reason: "A printer was detected via USB, but CUPS isn't installed! CUPS (Common Unix Printing System) is what Linux uses to manage printers. Without it, you can't print anything. It provides a web interface at http://localhost:631 for easy printer setup.".to_string(),
            action: "Install CUPS printing system".to_string(),
            command: Some("pacman -S --noconfirm cups".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "Hardware Support".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/CUPS".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    if has_cups {
        // Check if CUPS service is enabled
        let cups_enabled = Command::new("systemctl")
            .args(&["is-enabled", "cups"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !cups_enabled {
            result.push(Advice {
                id: "cups-enable-service".to_string(),
                title: "Enable CUPS service for printing".to_string(),
                reason: "CUPS is installed but not enabled! The CUPS service needs to be running for printers to work. Enable it so it starts automatically on boot, then you can access the web interface at http://localhost:631 to add printers.".to_string(),
                action: "Enable and start CUPS".to_string(),
                command: Some("systemctl enable --now cups".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Hardware Support".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/CUPS#Installation".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Suggest printer drivers
        let has_gutenprint = Command::new("pacman")
            .args(&["-Q", "gutenprint"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_gutenprint {
            result.push(Advice {
                id: "printer-drivers".to_string(),
                title: "Install Gutenprint for wide printer support".to_string(),
                reason: "You have CUPS but no printer drivers! Gutenprint provides drivers for hundreds of printer models (Canon, Epson, HP, etc.). Without proper drivers, your printer might not work or print at low quality. Think of it as the 'universal driver pack' for printers.".to_string(),
                action: "Install Gutenprint drivers".to_string(),
                command: Some("pacman -S --noconfirm gutenprint".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Hardware Support".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/CUPS#Printer_drivers".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_monitoring_tools() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for htop (better than top)
    let has_htop = Command::new("pacman")
        .args(&["-Q", "htop"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_htop {
        result.push(Advice {
            id: "monitoring-htop".to_string(),
            title: "Install htop for interactive process monitoring".to_string(),
            reason: "'top' is okay, but htop is WAY better! It's colorful, interactive, shows CPU cores individually, makes it easy to kill processes, and generally makes system monitoring actually pleasant. You can sort by memory, CPU, or any column with a keystroke. Every Linux user should have this!".to_string(),
            action: "Install htop".to_string(),
            command: Some("pacman -S --noconfirm htop".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "Utilities".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Htop".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    // Check for btop (even better than htop)
    let has_btop = Command::new("pacman")
        .args(&["-Q", "btop"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_btop && has_htop {
        result.push(Advice {
            id: "monitoring-btop".to_string(),
            title: "Consider btop for gorgeous system monitoring".to_string(),
            reason: "You have htop, but btop is the next evolution! It's like htop on steroids - beautiful graphs, detailed stats, disk I/O, network monitoring, GPU stats, all in a stunning TUI. It's eye candy AND functional. If you like htop, you'll love btop!".to_string(),
            action: "Install btop".to_string(),
            command: Some("pacman -S --noconfirm btop".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Utilities".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Btop".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    // Check for iotop (disk I/O monitoring)
    let has_iotop = Command::new("pacman")
        .args(&["-Q", "iotop"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_iotop {
        result.push(Advice {
            id: "monitoring-iotop".to_string(),
            title: "Install iotop to monitor disk I/O".to_string(),
            reason: "When your system is slow and disk activity is high, iotop tells you exactly which process is hammering your disk! It's like 'top' but for disk I/O. Essential for debugging slow systems or figuring out what's writing to your SSD constantly.".to_string(),
            action: "Install iotop".to_string(),
            command: Some("pacman -S --noconfirm iotop".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Utilities".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Iotop".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    result
}

pub(crate) fn check_laptop_optimizations(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if this is a laptop (has battery)
    let has_battery = std::path::Path::new("/sys/class/power_supply/BAT0").exists()
        || std::path::Path::new("/sys/class/power_supply/BAT1").exists();

    if !has_battery {
        return result; // Not a laptop
    }

    // Check for powertop
    let has_powertop = Command::new("pacman")
        .args(&["-Q", "powertop"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_powertop {
        result.push(Advice {
            id: "laptop-powertop".to_string(),
            title: "Install powertop for battery optimization".to_string(),
            reason: "You're on a laptop but don't have powertop! Powertop shows detailed power consumption and can auto-tune your system for better battery life. It can easily add 1-2 hours of battery by optimizing USB power, CPU C-states, and more. Run 'powertop --auto-tune' for automatic optimizations!".to_string(),
            action: "Install powertop".to_string(),
            command: Some("pacman -S --noconfirm powertop".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "Performance & Optimization".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Powertop".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    // Check for touchpad drivers (libinput)
    let has_libinput = Command::new("pacman")
        .args(&["-Q", "xf86-input-libinput"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_libinput {
        result.push(Advice {
            id: "laptop-touchpad".to_string(),
            title: "Install libinput for modern touchpad support".to_string(),
            reason: "You're on a laptop and need good touchpad drivers! libinput is the modern input driver that handles touchpads, trackpoints, and gestures. It provides smooth scrolling, multi-finger gestures, palm detection, and tap-to-click. Essential for a good laptop experience!".to_string(),
            action: "Install xf86-input-libinput".to_string(),
            command: Some("pacman -S --noconfirm xf86-input-libinput".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "Hardware Support".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Libinput".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    // Check for backlight control
    let has_backlight_control = Command::new("which")
        .arg("brightnessctl")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
        || Command::new("which")
            .arg("light")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

    if !has_backlight_control {
        result.push(Advice {
            id: "laptop-backlight".to_string(),
            title: "Install brightnessctl for screen brightness control".to_string(),
            reason: "You can't easily control your laptop screen brightness! brightnessctl lets you adjust brightness from the command line or bind it to keyboard shortcuts. Essential for laptops - you'll want to dim the screen to save battery or brighten it in sunlight.".to_string(),
            action: "Install brightnessctl".to_string(),
            command: Some("pacman -S --noconfirm brightnessctl".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "Hardware Support".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Backlight".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    // Check for laptop-mode-tools (advanced power management)
    let has_laptop_mode = Command::new("pacman")
        .args(&["-Q", "laptop-mode-tools"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_laptop_mode && !facts.dev_tools_detected.iter().any(|t| t == "tlp") {
        result.push(Advice {
            id: "laptop-mode-tools".to_string(),
            title: "Consider laptop-mode-tools for advanced power saving".to_string(),
            reason: "Want even more battery life? laptop-mode-tools provides aggressive power management: spins down HDDs, manages CPU frequency, controls device power states, and more. It's more configurable than TLP but requires more setup. Great for squeezing every minute out of your battery!".to_string(),
            action: "Install laptop-mode-tools".to_string(),
            command: Some("pacman -S --noconfirm laptop-mode-tools".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Performance & Optimization".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Laptop_Mode_Tools".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    result
}

pub(crate) fn check_webcam_support() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if webcam exists
    let has_webcam = std::path::Path::new("/dev/video0").exists()
        || std::path::Path::new("/dev/video1").exists();

    if !has_webcam {
        return result; // No webcam detected
    }

    // Check for v4l-utils
    let has_v4l = Command::new("pacman")
        .args(&["-Q", "v4l-utils"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_v4l {
        result.push(Advice {
            id: "webcam-v4l-utils".to_string(),
            title: "Install v4l-utils for webcam control".to_string(),
            reason: "You have a webcam but no tools to control it! v4l-utils provides v4l2-ctl for adjusting brightness, contrast, focus, and other camera settings. Super useful for video calls - you can tweak your camera to look good in any lighting!".to_string(),
            action: "Install v4l-utils".to_string(),
            command: Some("pacman -S --noconfirm v4l-utils".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Hardware Support".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Webcam_setup".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    // Suggest cheese for testing
    let has_cheese = Command::new("pacman")
        .args(&["-Q", "cheese"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_cheese {
        result.push(Advice {
            id: "webcam-cheese".to_string(),
            title: "Install Cheese for webcam testing".to_string(),
            reason: "Want to test your webcam? Cheese is a simple, fast webcam viewer. Perfect for checking if your camera works, adjusting position, or just making sure you look good before a video call! It can also take photos and videos.".to_string(),
            action: "Install Cheese webcam viewer".to_string(),
            command: Some("pacman -S --noconfirm cheese".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Utilities".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Webcam_setup#Cheese".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    result
}

pub(crate) fn check_android_integration() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for KDE Connect
    let has_kdeconnect = Command::new("pacman")
        .args(&["-Q", "kdeconnect"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_kdeconnect {
        result.push(Advice {
            id: "mobile-kdeconnect".to_string(),
            title: "Install KDE Connect for phone integration".to_string(),
            reason: "KDE Connect is AMAZING for phone integration! Get phone notifications on your PC, send/receive texts, share clipboard, transfer files, use phone as remote control, and more. Works with Android (and iOS via third-party). Makes your phone and PC work together seamlessly!".to_string(),
            action: "Install KDE Connect".to_string(),
            command: Some("pacman -S --noconfirm kdeconnect".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Utilities".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/KDE_Connect".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    // Check for scrcpy (screen mirroring)
    let has_scrcpy = Command::new("pacman")
        .args(&["-Q", "scrcpy"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_scrcpy {
        result.push(Advice {
            id: "mobile-scrcpy".to_string(),
            title: "Install scrcpy for Android screen mirroring".to_string(),
            reason: "scrcpy mirrors your Android screen to your PC with low latency! Control your phone from your computer - great for demos, testing apps, or just using your phone on a big screen. Works over USB or WiFi. Super smooth and fast!".to_string(),
            action: "Install scrcpy".to_string(),
            command: Some("pacman -S --noconfirm scrcpy".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Utilities".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Android#Screen_mirroring".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    result
}

pub(crate) fn check_file_sharing() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for Samba (Windows file sharing)
    let has_samba = Command::new("pacman")
        .args(&["-Q", "samba"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_samba {
        result.push(Advice {
            id: "fileshare-samba".to_string(),
            title: "Install Samba for Windows file sharing".to_string(),
            reason: "Samba lets you share files with Windows computers on your network! Super useful for mixed Windows/Linux environments. Share folders, access Windows shares, print to network printers. It's how Linux speaks 'Windows file sharing' - essential for home networks or offices with Windows machines!".to_string(),
            action: "Install Samba".to_string(),
            command: Some("pacman -S --noconfirm samba".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Network Configuration".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Samba".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    // Check for NFS utilities
    let has_nfs = Command::new("pacman")
        .args(&["-Q", "nfs-utils"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_nfs {
        result.push(Advice {
            id: "fileshare-nfs".to_string(),
            title: "Install NFS for Unix/Linux file sharing".to_string(),
            reason: "NFS (Network File System) is the native Linux/Unix way to share files across networks! Much faster than Samba for Linux-to-Linux sharing. Great for home servers, NAS devices, or accessing files from multiple Linux machines. Works seamlessly with proper permissions!".to_string(),
            action: "Install NFS utilities".to_string(),
            command: Some("pacman -S --noconfirm nfs-utils".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Network Configuration".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/NFS".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    result
}

pub(crate) fn check_cloud_storage() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for rclone (universal cloud sync)
    let has_rclone = Command::new("pacman")
        .args(&["-Q", "rclone"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_rclone {
        result.push(Advice {
            id: "cloud-rclone".to_string(),
            title: "Install rclone for cloud storage sync".to_string(),
            reason: "rclone is like rsync for cloud storage! It supports 40+ cloud providers (Google Drive, Dropbox, OneDrive, S3, Backblaze, etc.). Sync, copy, mount cloud storage as local drive, encrypt files, compare directories. Think of it as the Swiss Army knife for cloud storage. One tool to rule them all!".to_string(),
            action: "Install rclone".to_string(),
            command: Some("pacman -S --noconfirm rclone".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Utilities".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Rclone".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    result
}

pub(crate) fn check_databases() -> Vec<Advice> {
    let mut result = Vec::new();

    let db_usage = check_command_usage(&["psql", "mysql", "mongod", "redis-cli", "sqlite3"]);

    if db_usage > 5 {
        // Check for PostgreSQL
        let has_postgresql = Command::new("pacman")
            .args(&["-Q", "postgresql"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_postgresql {
            result.push(Advice {
                id: "database-postgresql".to_string(),
                title: "Install PostgreSQL database".to_string(),
                reason: "PostgreSQL is the world's most advanced open-source database! ACID-compliant, supports JSON, full-text search, geospatial data, and advanced indexing. Great for web apps, data analytics, anything needing a robust relational database. The database developers love!".to_string(),
                action: "Install PostgreSQL".to_string(),
                command: Some("pacman -S --noconfirm postgresql".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Development Tools".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/PostgreSQL".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: Some("web-dev".to_string()),
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_web_servers() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if user is doing web development
    let has_html_files = Command::new("find")
        .args(&[
            &std::env::var("HOME").unwrap_or_default(),
            "-name",
            "*.html",
            "-type",
            "f",
        ])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    let web_usage = check_command_usage(&["nginx", "apache", "httpd"]);

    if has_html_files || web_usage > 3 {
        // Check for nginx
        let has_nginx = Command::new("pacman")
            .args(&["-Q", "nginx"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_nginx {
            result.push(Advice {
                id: "webserver-nginx".to_string(),
                title: "Install nginx web server".to_string(),
                reason: "nginx is a fast, lightweight web server and reverse proxy! Perfect for serving static sites, acting as a load balancer, or proxying to Node.js/Python apps. Used by 40% of the busiest websites. Easy to configure, incredibly fast, and rock-solid stable!".to_string(),
                action: "Install nginx".to_string(),
                command: Some("pacman -S --noconfirm nginx".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Development Tools".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Nginx".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_remote_desktop() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for SSH but suggest VNC for GUI access
    let has_tigervnc = Command::new("pacman")
        .args(&["-Q", "tigervnc"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_tigervnc {
        result.push(Advice {
            id: "remote-vnc".to_string(),
            title: "Install TigerVNC for remote desktop access".to_string(),
            reason: "Need to access your desktop remotely? TigerVNC lets you control your Linux desktop from anywhere! Great for remote work, helping family/friends with tech support, or accessing your home PC from laptop. SSH is for terminal, VNC is for full desktop. Works cross-platform!".to_string(),
            action: "Install TigerVNC".to_string(),
            command: Some("pacman -S --noconfirm tigervnc".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Network Configuration".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/TigerVNC".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    result
}

pub(crate) fn check_monitor_tools() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if using X11
    let is_x11 = std::env::var("XDG_SESSION_TYPE").unwrap_or_default() == "x11";

    if is_x11 {
        // Check for arandr (GUI for xrandr)
        let has_arandr = Command::new("pacman")
            .args(&["-Q", "arandr"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_arandr {
            result.push(Advice {
                id: "monitor-arandr".to_string(),
                title: "Install arandr for easy monitor configuration".to_string(),
                reason: "arandr is a visual GUI for xrandr! Drag and drop monitors to arrange them, change resolutions, adjust refresh rates. Way easier than typing xrandr commands. Great for laptops with external monitors or multi-monitor desktops!".to_string(),
                action: "Install arandr".to_string(),
                command: Some("pacman -S --noconfirm arandr".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Desktop Environment".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Xrandr".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_dual_boot() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for GRUB
    let has_grub = Command::new("which")
        .arg("grub-mkconfig")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if has_grub {
        // Check for os-prober
        let has_os_prober = Command::new("pacman")
            .args(&["-Q", "os-prober"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_os_prober {
            result.push(Advice {
                id: "dualboot-osprober".to_string(),
                title: "Install os-prober for dual boot detection".to_string(),
                reason: "You have GRUB! os-prober automatically detects other operating systems (Windows, other Linux distros) and adds them to GRUB menu. Essential for dual boot setups. After installing, run 'sudo grub-mkconfig -o /boot/grub/grub.cfg' to regenerate config!".to_string(),
                action: "Install os-prober".to_string(),
                command: Some("pacman -S --noconfirm os-prober".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "System Configuration".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/GRUB#Detecting_other_operating_systems".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_additional_databases() -> Vec<Advice> {
    let mut result = Vec::new();

    let db_usage = check_command_usage(&["mysql", "mongod", "redis-cli"]);

    if db_usage > 3 {
        // Check for MySQL/MariaDB
        let has_mysql = Command::new("pacman")
            .args(&["-Q", "mariadb"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_mysql && db_usage > 5 {
            result.push(Advice {
                id: "database-mariadb".to_string(),
                title: "Install MariaDB for MySQL compatibility".to_string(),
                reason: "MariaDB is the drop-in replacement for MySQL! Fully compatible, often faster, more features, truly open-source. Great for web apps, WordPress, Drupal, or any MySQL application. 'systemctl start mariadb' and you're MySQL-compatible!".to_string(),
                action: "Install MariaDB".to_string(),
                command: Some("pacman -S --noconfirm mariadb".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Development Tools".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/MariaDB".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check for Redis
        let has_redis = Command::new("pacman")
            .args(&["-Q", "redis"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_redis {
            result.push(Advice {
                id: "database-redis".to_string(),
                title: "Install Redis for in-memory data storage".to_string(),
                reason: "Redis is blazingly fast in-memory database! Perfect for caching, session storage, queues, real-time analytics. Used by Twitter, GitHub, Snapchat. Simple key-value store with rich data types. If your app needs speed, Redis is the answer!".to_string(),
                action: "Install Redis".to_string(),
                command: Some("pacman -S --noconfirm redis".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Development Tools".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Redis".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_system_monitoring_advanced() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for s-tui (stress terminal UI)
    let has_stui = Command::new("pacman")
        .args(&["-Q", "s-tui"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_stui {
        result.push(Advice {
            id: "monitor-stui".to_string(),
            title: "Install s-tui for CPU stress testing and monitoring".to_string(),
            reason: "s-tui is a terminal UI for CPU stress testing! Monitor CPU frequency, temperature, power, utilization in real-time. Built-in stress test. Great for testing cooling, overclocking, or just seeing your CPU at full load. Beautiful TUI interface!".to_string(),
            action: "Install s-tui".to_string(),
            command: Some("pacman -S --noconfirm s-tui stress".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "System Configuration".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Stress_testing".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    result
}

fn is_package_installed(package: &str) -> bool {
    std::process::Command::new("pacman")
        .args(&["-Qq", package])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

