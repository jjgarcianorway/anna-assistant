//! Network recommendations

use super::{check_command_usage, is_package_installed};

use anna_common::{Advice, Alternative, Priority, RiskLevel, SystemFacts};
use std::path::Path;
use std::process::Command;

pub(crate) fn check_network_manager(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if system has wifi
    let has_wifi = facts
        .network_interfaces
        .iter()
        .any(|iface| iface.starts_with("wl"));

    if has_wifi {
        // Check if NetworkManager is installed
        let has_nm = Command::new("pacman")
            .args(&["-Q", "networkmanager"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_nm {
            result.push(Advice {
                id: "networkmanager".to_string(),
                title: "Install NetworkManager for easier WiFi".to_string(),
                reason: "You have a wireless card, but NetworkManager isn't installed. NetworkManager makes it super easy to connect to WiFi networks, switch between them, and manage VPNs. It's especially helpful if you use a laptop or move between different networks.".to_string(),
                action: "Install NetworkManager to simplify network management".to_string(),
                command: Some("pacman -S --noconfirm networkmanager && systemctl enable --now NetworkManager".to_string()),
                risk: RiskLevel::Medium,
                priority: Priority::Recommended,
                category: "System Maintenance".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/NetworkManager".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        } else {
            // Check if it's enabled
            let is_enabled = Command::new("systemctl")
                .args(&["is-enabled", "NetworkManager"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            if !is_enabled {
                result.push(Advice {
                    id: "networkmanager-enable".to_string(),
                    title: "Enable NetworkManager".to_string(),
                    reason: "You have NetworkManager installed, but it's not running yet. Without it running, you can't use its nice WiFi management features.".to_string(),
                    action: "Start NetworkManager so you can manage your WiFi connections".to_string(),
                    command: Some("systemctl enable --now NetworkManager".to_string()),
                    risk: RiskLevel::Low,
                    priority: Priority::Recommended,
                    category: "System Maintenance".to_string(),
                    alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/NetworkManager".to_string()],
                                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
            }
        }
    }

    result
}

pub(crate) fn check_firewall() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if ufw is installed
    let has_ufw = Command::new("pacman")
        .args(&["-Q", "ufw"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if has_ufw {
        // Check if ufw is enabled
        let ufw_status = Command::new("ufw").arg("status").output();

        if let Ok(output) = ufw_status {
            let status = String::from_utf8_lossy(&output.stdout);
            if status.contains("Status: inactive") {
                result.push(Advice {
                    id: "ufw-enable".to_string(),
                    title: "Turn on your firewall".to_string(),
                    reason: "You have UFW (Uncomplicated Firewall) installed, but it's not turned on. A firewall acts like a security guard for your computer, blocking unwanted network connections while allowing the ones you trust.".to_string(),
                    action: "Enable UFW to protect your system from network threats".to_string(),
                    command: Some("ufw enable".to_string()),
                    risk: RiskLevel::Low,
                    priority: Priority::Mandatory,
                    category: "Security & Privacy".to_string(),
                    alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Uncomplicated_Firewall".to_string()],
                                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
            }
        }
    } else {
        // Check if iptables is being actively used
        let iptables_rules = Command::new("iptables").args(&["-L", "-n"]).output();

        if let Ok(output) = iptables_rules {
            let rules = String::from_utf8_lossy(&output.stdout);
            // If only default policies exist, no firewall is configured
            let lines: Vec<&str> = rules.lines().collect();
            if lines.len() < 10 {
                // Very few rules = probably no firewall
                result.push(Advice {
                    id: "firewall-missing".to_string(),
                    title: "Set up a firewall for security".to_string(),
                    reason: "Your system doesn't have a firewall configured yet. A firewall protects you by controlling which network connections are allowed in and out of your computer. It's especially important if you connect to public WiFi or run any services.".to_string(),
                    action: "Install and configure UFW (Uncomplicated Firewall)".to_string(),
                    command: Some("pacman -S --noconfirm ufw && ufw default deny && ufw enable".to_string()),
                    risk: RiskLevel::Medium,
                    priority: Priority::Mandatory,
                    category: "Security & Privacy".to_string(),
                    alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Uncomplicated_Firewall".to_string()],
                                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
            }
        }
    }

    result
}

pub(crate) fn check_ssh_config() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if SSH server is installed
    let has_sshd = Command::new("pacman")
        .args(&["-Q", "openssh"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_sshd {
        // No SSH server, nothing to check
        return result;
    }

    // Check if sshd_config exists
    if let Ok(config) = std::fs::read_to_string("/etc/ssh/sshd_config") {
        // Check for root login
        let permits_root = config.lines().any(|l| {
            l.trim().starts_with("PermitRootLogin")
                && !l.contains("no")
                && !l.trim().starts_with("#")
        });

        if permits_root {
            result.push(Advice {
                id: "ssh-no-root-login".to_string(),
                title: "Disable direct root login via SSH".to_string(),
                reason: "Your SSH server allows direct root login, which is a security risk. If someone guesses or cracks your root password, they have complete control. It's much safer to log in as a regular user and then use 'sudo' when you need admin rights.".to_string(),
                action: "Set 'PermitRootLogin no' in /etc/ssh/sshd_config".to_string(),
                command: Some("sed -i 's/^#\\?PermitRootLogin.*/PermitRootLogin no/' /etc/ssh/sshd_config && systemctl restart sshd".to_string()),
                risk: RiskLevel::Medium,
                priority: Priority::Mandatory,
                category: "Security & Privacy".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/OpenSSH#Deny".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check for password authentication
        let password_auth = config.lines().any(|l| {
            l.trim().starts_with("PasswordAuthentication")
                && l.contains("yes")
                && !l.trim().starts_with("#")
        });

        if password_auth {
            // Only suggest if user has SSH keys set up
            if std::path::Path::new("/root/.ssh/authorized_keys").exists()
                || std::path::Path::new(&format!(
                    "/home/{}/.ssh/authorized_keys",
                    std::env::var("SUDO_USER").unwrap_or_default()
                ))
                .exists()
            {
                result.push(Advice {
                    id: "ssh-key-only".to_string(),
                    title: "Use SSH keys instead of passwords".to_string(),
                    reason: "Password authentication over SSH can be brute-forced by attackers. SSH keys are much more secure - they're like having a 4096-character password that's impossible to guess. Since you already have SSH keys set up, you can safely disable password login.".to_string(),
                    action: "Disable password authentication in SSH".to_string(),
                    command: Some("sed -i 's/^#\\?PasswordAuthentication.*/PasswordAuthentication no/' /etc/ssh/sshd_config && systemctl restart sshd".to_string()),
                    risk: RiskLevel::Medium,
                    priority: Priority::Recommended,
                    category: "Security & Privacy".to_string(),
                    alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/OpenSSH#Force_public_key_authentication".to_string()],
                                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
            }
        }

        // Check for empty passwords
        let permits_empty = config.lines().any(|l| {
            l.trim().starts_with("PermitEmptyPasswords")
                && l.contains("yes")
                && !l.trim().starts_with("#")
        });

        if permits_empty {
            result.push(Advice {
                id: "ssh-no-empty-passwords".to_string(),
                title: "Disable empty passwords for SSH".to_string(),
                reason: "Your SSH configuration allows accounts with empty passwords to log in. This is extremely dangerous - anyone could access your system without any authentication at all!".to_string(),
                action: "Set 'PermitEmptyPasswords no' immediately".to_string(),
                command: Some("sed -i 's/^#\\?PermitEmptyPasswords.*/PermitEmptyPasswords no/' /etc/ssh/sshd_config && systemctl restart sshd".to_string()),
                risk: RiskLevel::High,
                priority: Priority::Mandatory,
                category: "Security & Privacy".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/OpenSSH#Deny".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check for Protocol version (SSH-1 is insecure)
        let allows_protocol_1 = config.lines().any(|l| {
            l.trim().starts_with("Protocol") && l.contains("1") && !l.trim().starts_with("#")
        });

        if allows_protocol_1 {
            result.push(Advice {
                id: "ssh-protocol-2-only".to_string(),
                title: "Use SSH Protocol 2 only".to_string(),
                reason: "SSH Protocol 1 has known security vulnerabilities and should never be used. Protocol 2 has been the standard since 2006 and is much more secure with better encryption. Modern OpenSSH versions default to Protocol 2 only, but your config explicitly allows Protocol 1.".to_string(),
                action: "Remove Protocol 1 support from SSH config".to_string(),
                command: Some("sed -i 's/^#\\?Protocol.*/Protocol 2/' /etc/ssh/sshd_config && systemctl restart sshd".to_string()),
                risk: RiskLevel::High,
                priority: Priority::Mandatory,
                category: "Security & Privacy".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/OpenSSH#Configuration".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check for X11 forwarding (security risk if not needed)
        let x11_forwarding = config.lines().any(|l| {
            l.trim().starts_with("X11Forwarding") && l.contains("yes") && !l.trim().starts_with("#")
        });

        if x11_forwarding {
            result.push(Advice {
                id: "ssh-disable-x11-forwarding".to_string(),
                title: "Consider disabling X11 forwarding in SSH".to_string(),
                reason: "X11 forwarding allows remote systems to interact with your X server, which can be a security risk if compromised. Unless you specifically need to run graphical applications over SSH, it's safer to disable this feature.".to_string(),
                action: "Set 'X11Forwarding no' in SSH config".to_string(),
                command: Some("sed -i 's/^#\\?X11Forwarding.*/X11Forwarding no/' /etc/ssh/sshd_config && systemctl restart sshd".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Security & Privacy".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/OpenSSH#X11_forwarding".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check for MaxAuthTries (limit brute force attempts)
        let has_max_auth_tries = config
            .lines()
            .any(|l| l.trim().starts_with("MaxAuthTries") && !l.trim().starts_with("#"));

        if !has_max_auth_tries {
            result.push(Advice {
                id: "ssh-max-auth-tries".to_string(),
                title: "Limit SSH authentication attempts".to_string(),
                reason: "Setting MaxAuthTries limits how many password attempts someone can make before being disconnected. This slows down brute-force attacks. The default is 6, but setting it to 3 is more secure - legitimate users rarely need more than 3 tries!".to_string(),
                action: "Add 'MaxAuthTries 3' to SSH config".to_string(),
                command: Some("echo 'MaxAuthTries 3' >> /etc/ssh/sshd_config && systemctl restart sshd".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Security & Privacy".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/OpenSSH#Protection".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check for ClientAliveInterval (detect dead connections)
        let has_client_alive = config
            .lines()
            .any(|l| l.trim().starts_with("ClientAliveInterval") && !l.trim().starts_with("#"));

        if !has_client_alive {
            result.push(Advice {
                id: "ssh-client-alive-interval".to_string(),
                title: "Configure SSH connection timeouts".to_string(),
                reason: "ClientAliveInterval makes the server send keepalive messages to detect dead connections. This prevents abandoned SSH sessions from staying open forever, which is both a security and resource management issue. Setting it to 300 seconds (5 minutes) is a good balance.".to_string(),
                action: "Add connection timeout settings to SSH".to_string(),
                command: Some("echo -e 'ClientAliveInterval 300\\nClientAliveCountMax 2' >> /etc/ssh/sshd_config && systemctl restart sshd".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Security & Privacy".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/OpenSSH#Keep_alive".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check for AllowUsers/AllowGroups (whitelist approach)
        let has_allow_users = config.lines().any(|l| {
            (l.trim().starts_with("AllowUsers") || l.trim().starts_with("AllowGroups"))
                && !l.trim().starts_with("#")
        });

        if !has_allow_users {
            result.push(Advice {
                id: "ssh-allowusers-consideration".to_string(),
                title: "Consider using AllowUsers for SSH access control".to_string(),
                reason: "Instead of letting any user SSH in, you can whitelist specific users with 'AllowUsers username'. This is the most secure approach - even if someone creates a new user account on your system, they won't be able to SSH in unless you explicitly allow them. Great for single-user or small team systems!".to_string(),
                action: "Review if you should add 'AllowUsers' directive".to_string(),
                command: Some("cat /etc/ssh/sshd_config | grep -E '^(AllowUsers|AllowGroups|DenyUsers|DenyGroups)'".to_string()), // Show current access control
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Security & Privacy".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/OpenSSH#Deny".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check for default SSH port (22)
        let uses_default_port = !config.lines().any(|l| {
            l.trim().starts_with("Port") && !l.contains("22") && !l.trim().starts_with("#")
        });

        if uses_default_port {
            result.push(Advice {
                id: "ssh-non-default-port".to_string(),
                title: "Consider changing SSH to non-default port".to_string(),
                reason: "Running SSH on port 22 (the default) means your server gets hammered by automated bot attacks 24/7. Changing to a non-standard port (like 2222 or 22222) drastically reduces these attacks. It's 'security through obscurity' but it works surprisingly well for reducing noise and log spam! Just make sure you remember the new port.".to_string(),
                action: "Consider changing SSH port from 22 to something else".to_string(),
                command: Some("ss -tlnp | grep sshd".to_string()), // Show current SSH port
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Security & Privacy".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/OpenSSH#Protection".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check for post-quantum resistant SSH keys
        // Ed25519 is currently quantum-resistant (256-bit security)
        // RSA-4096 provides ~140-bit post-quantum security
        if let Ok(key_output) = Command::new("sh")
            .arg("-c")
            .arg("ssh-keygen -lf /etc/ssh/ssh_host_*_key.pub 2>/dev/null")
            .output()
        {
            let keys_info = String::from_utf8_lossy(&key_output.stdout);
            let has_ed25519 = keys_info.contains("ED25519");
            let has_weak_keys = keys_info.contains("RSA")
                && keys_info.lines().any(|l| {
                    l.contains("RSA")
                        && l.split_whitespace()
                            .next()
                            .and_then(|s| s.parse::<u32>().ok())
                            .map(|bits| bits < 3072)
                            .unwrap_or(false)
                });

            if !has_ed25519 {
                result.push(Advice {
                    id: "ssh-ed25519-keys".to_string(),
                    title: "Generate Ed25519 SSH host keys for better security".to_string(),
                    reason: "Ed25519 keys are faster, more secure, and resistant to many quantum computing attacks. They provide 256-bit security with much smaller key sizes than RSA. Modern SSH servers should use Ed25519 as the primary key type. Your server is missing Ed25519 keys!".to_string(),
                    action: "Generate Ed25519 host keys for SSH server".to_string(),
                    command: Some("ssh-keygen -t ed25519 -f /etc/ssh/ssh_host_ed25519_key -N '' && systemctl restart sshd".to_string()),
                    risk: RiskLevel::Low,
                    priority: Priority::Recommended,
                    category: "Security & Privacy".to_string(),
                    alternatives: Vec::new(),
                    wiki_refs: vec!["https://wiki.archlinux.org/title/OpenSSH#Host_keys".to_string()],
                    depends_on: Vec::new(),
                    related_to: vec!["ssh-no-root-login".to_string()],
                    bundle: Some("ssh-hardening".to_string()),
            satisfies: Vec::new(),
                    popularity: 70,
            requires: Vec::new(),
                });
            }

            if has_weak_keys {
                result.push(Advice {
                    id: "ssh-strong-rsa-keys".to_string(),
                    title: "Replace weak RSA SSH keys with 4096-bit keys".to_string(),
                    reason: "Your SSH server has RSA keys smaller than 3072 bits. For post-quantum security, RSA keys should be at least 3072 bits, preferably 4096 bits. Weak keys are vulnerable to faster cracking with modern computers and quantum advances.".to_string(),
                    action: "Regenerate SSH RSA host keys with 4096 bits".to_string(),
                    command: Some("ssh-keygen -t rsa -b 4096 -f /etc/ssh/ssh_host_rsa_key -N '' && systemctl restart sshd".to_string()),
                    risk: RiskLevel::Medium,
                    priority: Priority::Recommended,
                    category: "Security & Privacy".to_string(),
                    alternatives: vec![
                        Alternative {
                            name: "Switch to Ed25519 only".to_string(),
                            description: "Use only Ed25519 keys (recommended)".to_string(),
                            install_command: "ssh-keygen -t ed25519 -f /etc/ssh/ssh_host_ed25519_key -N ''".to_string(),
                        },
                    ],
                    wiki_refs: vec!["https://wiki.archlinux.org/title/OpenSSH#Host_keys".to_string()],
                    depends_on: Vec::new(),
                    related_to: vec!["ssh-ed25519-keys".to_string()],
                    bundle: Some("ssh-hardening".to_string()),
            satisfies: Vec::new(),
                    popularity: 65,
            requires: Vec::new(),
                });
            }
        }

        // Check for modern cipher and KEX algorithms
        if !config.contains("Ciphers") || config.contains("arcfour") || config.contains("3des") {
            result.push(Advice {
                id: "ssh-modern-ciphers".to_string(),
                title: "Configure modern SSH ciphers and algorithms".to_string(),
                reason: "Your SSH config uses default or outdated ciphers. Modern ciphers like chacha20-poly1305 and AES-GCM are faster and more secure. Old ciphers like 3DES and arcfour have known vulnerabilities and should never be used!".to_string(),
                action: "Add modern cipher configuration to sshd_config".to_string(),
                command: Some("echo -e '\\n# Modern ciphers and KEX algorithms\\nCiphers chacha20-poly1305@openssh.com,aes256-gcm@openssh.com,aes128-gcm@openssh.com,aes256-ctr,aes192-ctr,aes128-ctr\\nMACs hmac-sha2-512-etm@openssh.com,hmac-sha2-256-etm@openssh.com,hmac-sha2-512,hmac-sha2-256\\nKexAlgorithms curve25519-sha256,curve25519-sha256@libssh.org,diffie-hellman-group-exchange-sha256' >> /etc/ssh/sshd_config && systemctl restart sshd".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Security & Privacy".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/OpenSSH#Encryption".to_string()],
                depends_on: Vec::new(),
                related_to: vec!["ssh-no-root-login".to_string()],
                bundle: Some("ssh-hardening".to_string()),
            satisfies: Vec::new(),
                popularity: 60,
            requires: Vec::new(),
            });
        }
    }

    // Recommend ssh-audit tool
    if !is_package_installed("ssh-audit") {
        // Only recommend if SSH server is running
        if Command::new("systemctl")
            .args(&["is-active", "sshd"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            result.push(Advice {
                id: "install-ssh-audit".to_string(),
                title: "Install ssh-audit to analyze SSH security".to_string(),
                reason: "ssh-audit is an excellent tool that analyzes your SSH server configuration and finds security issues. It checks for weak algorithms, outdated protocols, and gives you a security grade. Since you're running an SSH server, this tool helps you verify it's properly hardened. Run 'ssh-audit localhost' after installing!".to_string(),
                action: "Install ssh-audit security auditing tool".to_string(),
                command: Some("pacman -S --noconfirm ssh-audit".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Security & Privacy".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/OpenSSH#Protection".to_string()],
                depends_on: Vec::new(),
                related_to: vec!["ssh-no-root-login".to_string(), "ssh-modern-ciphers".to_string()],
                bundle: Some("ssh-hardening".to_string()),
            satisfies: Vec::new(),
                popularity: 55,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_bluetooth() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for Bluetooth hardware
    let has_bluetooth_hw = Command::new("rfkill")
        .args(&["list", "bluetooth"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| !s.is_empty() && s.contains("bluetooth"))
        .unwrap_or(false);

    if has_bluetooth_hw {
        // Check if bluez is installed
        let has_bluez = Command::new("pacman")
            .args(&["-Q", "bluez"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_bluez {
            result.push(Advice {
                id: "bluetooth-bluez".to_string(),
                title: "Install BlueZ for Bluetooth support".to_string(),
                reason: "Your system has Bluetooth hardware, but BlueZ (the Linux Bluetooth stack) isn't installed! Without it, you can't connect any Bluetooth devices - headphones, mice, keyboards, game controllers, etc. BlueZ is essential for Bluetooth to work at all.".to_string(),
                action: "Install BlueZ and enable bluetooth service".to_string(),
                command: Some("pacman -S --noconfirm bluez bluez-utils && systemctl enable --now bluetooth.service".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Mandatory,
                category: "Hardware Support".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Bluetooth".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        } else {
            // Check for GUI tools
            let has_blueman = Command::new("pacman")
                .args(&["-Q", "blueman"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            if !has_blueman {
                result.push(Advice {
                    id: "bluetooth-blueman".to_string(),
                    title: "Install Blueman for easy Bluetooth management".to_string(),
                    reason: "Blueman gives you a nice GUI to manage Bluetooth devices. Pair headphones, connect mice, transfer files - all with a simple interface. Much friendlier than command-line bluetoothctl!".to_string(),
                    action: "Install Blueman".to_string(),
                    command: Some("pacman -S --noconfirm blueman".to_string()),
                    risk: RiskLevel::Low,
                    priority: Priority::Recommended,
                    category: "Hardware Support".to_string(),
                    alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Bluetooth#Blueman".to_string()],
                                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
            }
        }
    }

    result
}

pub(crate) fn check_wifi_setup() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for WiFi hardware
    let has_wifi = Command::new("iw")
        .arg("dev")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.contains("Interface"))
        .unwrap_or(false);

    if has_wifi {
        // Check for common firmware packages
        let has_linux_firmware = Command::new("pacman")
            .args(&["-Q", "linux-firmware"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_linux_firmware {
            result.push(Advice {
                id: "wifi-firmware".to_string(),
                title: "Install linux-firmware for WiFi support".to_string(),
                reason: "Your system has WiFi hardware, but the firmware package isn't installed! WiFi cards need firmware to work, and linux-firmware contains drivers for most WiFi chips (Intel, Realtek, Atheros, Broadcom). Without it, your WiFi probably doesn't work at all!".to_string(),
                action: "Install linux-firmware".to_string(),
                command: Some("pacman -S --noconfirm linux-firmware".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Mandatory,
                category: "Hardware Support".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Wireless#WiFi".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check for Intel WiFi specific firmware
        let cpu_info = std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
        let is_intel_cpu = cpu_info.to_lowercase().contains("intel");

        if is_intel_cpu {
            let _has_intel_ucode = Command::new("pacman")
                .args(&["-Q", "linux-firmware-iwlwifi"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            // Note: linux-firmware-iwlwifi might not exist as separate package in all repos
            // This is informational - could be used for future Intel-specific WiFi advice
        }

        // Check for network management GUI
        let has_nm = Command::new("pacman")
            .args(&["-Q", "networkmanager"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if has_nm {
            let has_nm_applet = Command::new("pacman")
                .args(&["-Q", "network-manager-applet"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            if !has_nm_applet {
                result.push(Advice {
                    id: "wifi-nm-applet".to_string(),
                    title: "Install NetworkManager applet for WiFi management".to_string(),
                    reason: "You have NetworkManager but no system tray applet! The applet gives you a nice GUI to connect to WiFi networks, see signal strength, and manage connections. Much easier than using nmcli commands!".to_string(),
                    action: "Install network-manager-applet".to_string(),
                    command: Some("pacman -S --noconfirm network-manager-applet".to_string()),
                    risk: RiskLevel::Low,
                    priority: Priority::Recommended,
                    category: "Network Configuration".to_string(),
                    alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/NetworkManager#nm-applet".to_string()],
                                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
            }
        }
    }

    result
}

pub(crate) fn check_network_quality(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // Check latency
    if let Some(latency) = facts.performance_metrics.average_latency_ms {
        if latency > 150.0 {
            result.push(Advice {
                id: "high-network-latency".to_string(),
                title: "High network latency detected".to_string(),
                reason: format!(
                    "Your network latency is {:.0}ms, which is quite high! This can make web browsing feel sluggish, video calls choppy, and online gaming frustrating. Good latency is usually under 50ms. High latency can be caused by: WiFi interference, being far from your router, ISP congestion, or using VPN/proxy servers.",
                    latency
                ),
                action: "Investigate and improve network latency".to_string(),
                command: Some("ping -c 10 1.1.1.1 | tail -1".to_string()), // Test current latency
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Network Configuration".to_string(),
                alternatives: vec![
                    Alternative {
                        name: "Move closer to router".to_string(),
                        description: "Physical distance significantly impacts WiFi latency".to_string(),
                        install_command: String::new(),
                    },
                    Alternative {
                        name: "Use Ethernet".to_string(),
                        description: "Wired connection typically has lower latency than WiFi".to_string(),
                        install_command: String::new(),
                    },
                    Alternative {
                        name: "Change WiFi channel".to_string(),
                        description: "Reduce interference by using less congested channel".to_string(),
                        install_command: String::new(),
                    },
                ],
                wiki_refs: vec!["https://wiki.archlinux.org/title/Network_configuration".to_string()],
                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
                popularity: 10,
            requires: Vec::new(),
            });
        } else if latency > 80.0 {
            result.push(Advice {
                id: "moderate-network-latency".to_string(),
                title: "Moderate network latency".to_string(),
                reason: format!(
                    "Your network latency is {:.0}ms. This is acceptable for most tasks, but could be better. For video calls and gaming, aim for under 50ms. Consider switching to Ethernet if you're on WiFi, or check if your ISP is experiencing issues.",
                    latency
                ),
                action: "Consider optimizing network connection".to_string(),
                command: Some("ip link show | grep -E '^[0-9]+:' | awk '{print $2}' | sed 's/:$//'".to_string()), // Show network interfaces
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Network Configuration".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Network_configuration".to_string()],
                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
                popularity: 10,
            requires: Vec::new(),
            });
        }
    }

    // Check packet loss
    if let Some(packet_loss) = facts.performance_metrics.packet_loss_percent {
        if packet_loss > 5.0 {
            result.push(Advice {
                id: "high-packet-loss".to_string(),
                title: "Significant packet loss detected".to_string(),
                reason: format!(
                    "Your network is experiencing {:.1}% packet loss! Any packet loss above 2-3% is problematic. This causes: stuttering video, disconnects in video calls, lag spikes in games, and slow downloads. Common causes: WiFi interference, weak signal, router problems, or ISP issues.",
                    packet_loss
                ),
                action: "Diagnose and fix packet loss issues".to_string(),
                command: Some("ping -c 100 8.8.8.8 | grep -E 'packet loss|transmitted'".to_string()), // Test packet loss
                risk: RiskLevel::Medium,
                priority: Priority::Recommended,
                category: "Network Configuration".to_string(),
                alternatives: vec![
                    Alternative {
                        name: "Use Ethernet".to_string(),
                        description: "Wired connections have virtually zero packet loss".to_string(),
                        install_command: String::new(),
                    },
                    Alternative {
                        name: "Check router/cables".to_string(),
                        description: "Faulty hardware can cause packet loss".to_string(),
                        install_command: String::new(),
                    },
                    Alternative {
                        name: "Contact ISP".to_string(),
                        description: "Persistent packet loss may indicate ISP problems".to_string(),
                        install_command: String::new(),
                    },
                ],
                wiki_refs: vec!["https://wiki.archlinux.org/title/Network_Debugging".to_string()],
                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
                popularity: 15,
            requires: Vec::new(),
            });
        } else if packet_loss > 1.0 {
            result.push(Advice {
                id: "minor-packet-loss".to_string(),
                title: "Minor packet loss detected".to_string(),
                reason: format!(
                    "Your network has {:.1}% packet loss. While not critical, ideally you want 0% packet loss. This minor loss might occasionally cause brief hiccups in video calls or online games. Worth investigating if it persists.",
                    packet_loss
                ),
                action: "Monitor network stability".to_string(),
                command: Some("ip -s link show".to_string()), // Show network interface statistics
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Network Configuration".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Network_Debugging".to_string()],
                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
                popularity: 10,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_dns_configuration() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if systemd-resolved is available but not used
    let has_resolved = std::path::Path::new("/usr/lib/systemd/systemd-resolved").exists();
    let resolved_active = Command::new("systemctl")
        .args(&["is-active", "systemd-resolved"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // Check current DNS setup
    let resolv_conf = std::fs::read_to_string("/etc/resolv.conf").unwrap_or_default();
    let using_stub_resolver = resolv_conf.contains("127.0.0.53");

    if has_resolved && !resolved_active && !using_stub_resolver {
        result.push(Advice {
            id: "dns-systemd-resolved".to_string(),
            title: "Consider systemd-resolved for modern DNS".to_string(),
            reason: "systemd-resolved provides modern DNS with caching, DNSSEC validation, and per-interface DNS settings. It's faster than traditional DNS (caches queries) and more secure (validates DNSSEC). Especially useful with NetworkManager or multiple network interfaces!".to_string(),
            action: "Enable systemd-resolved".to_string(),
            command: Some("systemctl enable --now systemd-resolved && ln -sf /run/systemd/resolve/stub-resolv.conf /etc/resolv.conf".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Network Configuration".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Systemd-resolved".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    // Check for public DNS servers
    if !resolv_conf.contains("127.0.0.") {
        let using_isp_dns = !resolv_conf.contains("1.1.1.1")
            && !resolv_conf.contains("8.8.8.8")
            && !resolv_conf.contains("9.9.9.9");

        if using_isp_dns {
            result.push(Advice {
                id: "dns-public-servers".to_string(),
                title: "Consider using public DNS servers".to_string(),
                reason: "You're using your ISP's DNS servers, which may be slow, unreliable, or log your queries! Public DNS like Cloudflare (1.1.1.1), Google (8.8.8.8), or Quad9 (9.9.9.9) are usually faster, more reliable, and respect privacy better. Cloudflare is the fastest with strong privacy!".to_string(),
                action: "Consider switching to public DNS".to_string(),
                command: Some("cat /etc/resolv.conf | grep nameserver".to_string()), // Show current DNS servers
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Network Configuration".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Domain_name_resolution".to_string()],
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

pub(crate) fn check_vpn_tools() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for WireGuard
    let has_wireguard = Command::new("pacman")
        .args(&["-Q", "wireguard-tools"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // Check for OpenVPN
    let has_openvpn = Command::new("pacman")
        .args(&["-Q", "openvpn"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_wireguard && !has_openvpn {
        result.push(Advice {
            id: "vpn-wireguard".to_string(),
            title: "Consider installing WireGuard for modern VPN".to_string(),
            reason: "WireGuard is the modern, fast, and secure VPN protocol! It's built into the Linux kernel, incredibly fast (faster than OpenVPN), and super easy to configure. Perfect for secure remote access, privacy, or connecting to VPN services. Way simpler than OpenVPN!".to_string(),
            action: "Install WireGuard tools".to_string(),
            command: Some("pacman -S --noconfirm wireguard-tools".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Network Configuration".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/WireGuard".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    // Check for NetworkManager VPN plugins
    let has_nm_vpn = Command::new("pacman")
        .args(&["-Q", "networkmanager-openvpn"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if has_openvpn && !has_nm_vpn {
        result.push(Advice {
            id: "vpn-nm-plugin".to_string(),
            title: "Install NetworkManager VPN plugin".to_string(),
            reason: "You have OpenVPN but no NetworkManager plugin! The plugin adds VPN support to NetworkManager's GUI - you can import .ovpn files and connect to VPNs with a single click instead of command line. Much more convenient!".to_string(),
            action: "Install NetworkManager OpenVPN plugin".to_string(),
            command: Some("pacman -S --noconfirm networkmanager-openvpn".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "Network Configuration".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/NetworkManager#VPN_support".to_string()],
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

pub(crate) fn check_network_tools() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for advanced network usage
    let net_usage = check_command_usage(&["ping", "traceroute", "netstat", "ss"]);

    if net_usage > 10 {
        // Check for Wireshark
        let has_wireshark = Command::new("pacman")
            .args(&["-Q", "wireshark-qt"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_wireshark {
            result.push(Advice {
                id: "network-wireshark".to_string(),
                title: "Install Wireshark for network analysis".to_string(),
                reason: "Wireshark is THE network protocol analyzer! Capture and inspect packets, debug network issues, analyze traffic, learn protocols. Essential for network admins, security researchers, or anyone debugging network problems. GUI and CLI (tshark) included!".to_string(),
                action: "Install Wireshark".to_string(),
                command: Some("pacman -S --noconfirm wireshark-qt".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Network Configuration".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Wireshark".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check for nmap
        let has_nmap = Command::new("pacman")
            .args(&["-Q", "nmap"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_nmap {
            result.push(Advice {
                id: "network-nmap".to_string(),
                title: "Install nmap for network scanning".to_string(),
                reason: "nmap is the network exploration tool! Scan networks, discover hosts, identify services, detect OS. Used by security professionals worldwide. 'nmap 192.168.1.0/24' scans your local network. Essential for network administration and security auditing!".to_string(),
                action: "Install nmap".to_string(),
                command: Some("pacman -S --noconfirm nmap".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Network Configuration".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Nmap".to_string()],
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

pub(crate) fn check_bluetooth_setup(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    if facts.bluetooth_status.available && !facts.bluetooth_status.enabled {
        result.push(Advice::new(
            "bluetooth-enable".to_string(),
            "Bluetooth Hardware Detected But Not Enabled".to_string(),
            "Your system has Bluetooth hardware but the service is not running. Enable it to use wireless mice, keyboards, headphones, and speakers.".to_string(),
            "Install bluez packages and enable bluetooth service".to_string(),
            Some("sudo pacman -S --noconfirm bluez bluez-utils && sudo systemctl enable --now bluetooth".to_string()),
            RiskLevel::Low,
            Priority::Optional,
            vec!["https://wiki.archlinux.org/title/Bluetooth".to_string()],
            "hardware".to_string(),
        ).with_popularity(65));
    }

    if facts.bluetooth_status.enabled && !facts.bluetooth_status.connected_devices.is_empty() {
        if !Command::new("which")
            .arg("blueman-manager")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            result.push(Advice::new(
                "blueman-gui".to_string(),
                "Install Blueman for Bluetooth GUI Management".to_string(),
                format!(
                    "You have {} Bluetooth device(s) connected. Blueman provides a user-friendly system tray applet and GUI for managing connections.",
                    facts.bluetooth_status.connected_devices.len()
                ),
                "Install Blueman for convenient Bluetooth management instead of command-line tools".to_string(),
                Some("sudo pacman -S --noconfirm blueman".to_string()),
                RiskLevel::Low,
                Priority::Cosmetic,
                vec!["https://wiki.archlinux.org/title/Bluetooth#Graphical".to_string()],
                "Usability".to_string(),
            ).with_popularity(55));
        }
    }

    result
}

