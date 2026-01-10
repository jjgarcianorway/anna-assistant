//! Security and permissions patterns
//! v0.0.917: Firewall, permissions, users, and security queries

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Match security-related queries
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    if let Some(u) = match_firewall(q) {
        return Some(u);
    }
    if let Some(u) = match_permissions(q) {
        return Some(u);
    }
    if let Some(u) = match_users(q) {
        return Some(u);
    }
    if let Some(u) = match_ssh(q) {
        return Some(u);
    }
    if let Some(u) = match_audit(q) {
        return Some(u);
    }
    None
}

/// Pattern with keywords, description, topic, and commands
type SecPattern = (&'static [&'static str], &'static str, &'static str, &'static [&'static str]);

fn match_firewall(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[SecPattern] = &[
        // UFW (Ubuntu/Debian style)
        (&["ufw", "status"], "UFW firewall status", "firewall", &["sudo ufw status verbose"]),
        (&["ufw", "allow"], "UFW allow rule", "firewall",
            &["echo 'Allow port: sudo ufw allow <port>'",
              "echo 'Allow service: sudo ufw allow ssh'"]),
        (&["ufw", "deny"], "UFW deny rule", "firewall",
            &["echo 'Deny port: sudo ufw deny <port>'"]),
        (&["ufw", "enable"], "enable UFW", "firewall",
            &["echo 'Enable: sudo ufw enable'", "sudo ufw status"]),
        // Firewalld (RHEL/Fedora style)
        (&["firewalld", "status"], "firewalld status", "firewall",
            &["sudo firewall-cmd --state", "sudo firewall-cmd --list-all"]),
        (&["firewall-cmd"], "firewalld query", "firewall",
            &["sudo firewall-cmd --list-all"]),
        // iptables
        (&["iptables", "list"], "iptables rules", "firewall",
            &["sudo iptables -L -n -v | head -30"]),
        (&["iptables", "rule"], "iptables rules", "firewall",
            &["sudo iptables -L -n -v | head -30"]),
        // nftables
        (&["nftables"], "nftables rules", "firewall",
            &["sudo nft list ruleset | head -30"]),
        (&["nft", "list"], "nftables rules", "firewall",
            &["sudo nft list ruleset | head -30"]),
        // General firewall
        (&["firewall", "status"], "firewall status", "firewall",
            &["sudo ufw status 2>/dev/null || sudo firewall-cmd --state 2>/dev/null || sudo iptables -L -n | head -10"]),
        (&["open", "firewall"], "open firewall port", "firewall",
            &["echo 'UFW: sudo ufw allow <port>'",
              "echo 'firewalld: sudo firewall-cmd --add-port=<port>/tcp --permanent'"]),
        (&["block", "ip"], "block IP address", "firewall",
            &["echo 'UFW: sudo ufw deny from <ip>'",
              "echo 'iptables: sudo iptables -A INPUT -s <ip> -j DROP'"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Factual,
                confidence: 0.9,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

fn match_permissions(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[SecPattern] = &[
        // File permissions
        (&["file", "permission"], "file permissions query", "permissions",
            &["echo 'Check: ls -la <file>'", "echo 'Change: chmod <mode> <file>'"]),
        (&["chmod"], "chmod usage", "permissions",
            &["echo 'Usage: chmod <mode> <file>'",
              "echo 'Examples: chmod 755 script.sh | chmod u+x file'"]),
        (&["chown"], "chown usage", "permissions",
            &["echo 'Usage: chown <user>:<group> <file>'",
              "echo 'Recursive: chown -R user:group directory/'"]),
        // Ownership
        (&["file", "owner"], "file ownership", "permissions",
            &["echo 'Check: ls -la <file>'", "echo 'Change: chown <user>:<group> <file>'"]),
        (&["change", "owner"], "change file ownership", "permissions",
            &["echo 'chown <user>:<group> <file>'",
              "echo 'Recursive: chown -R user:group directory/'"]),
        // SUID/SGID
        (&["suid", "file"], "SUID files", "permissions",
            &["find / -perm -4000 -type f 2>/dev/null | head -20"]),
        (&["setuid"], "SUID files", "permissions",
            &["find / -perm -4000 -type f 2>/dev/null | head -20"]),
        (&["sgid", "file"], "SGID files", "permissions",
            &["find / -perm -2000 -type f 2>/dev/null | head -20"]),
        // World-writable
        (&["world", "writable"], "world-writable files", "permissions",
            &["find /tmp /var/tmp -perm -002 -type f 2>/dev/null | head -20"]),
        // ACL
        (&["getfacl"], "file ACL", "permissions",
            &["echo 'Usage: getfacl <file>'"]),
        (&["setfacl"], "set file ACL", "permissions",
            &["echo 'Usage: setfacl -m u:<user>:rwx <file>'"]),
        (&["acl", "permission"], "ACL permissions", "permissions",
            &["echo 'View: getfacl <file>'", "echo 'Set: setfacl -m u:<user>:rwx <file>'"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Factual,
                confidence: 0.9,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

fn match_users(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[SecPattern] = &[
        // User listing
        (&["list", "user"], "list users", "users",
            &["cat /etc/passwd | grep -E '/bin/(ba)?sh' | cut -d: -f1"]),
        (&["all", "user"], "all users", "users",
            &["cat /etc/passwd | cut -d: -f1"]),
        (&["system", "user"], "system users", "users",
            &["cat /etc/passwd | awk -F: '$3 < 1000 {print $1}'"]),
        // Groups
        (&["list", "group"], "list groups", "users",
            &["cat /etc/group | cut -d: -f1 | head -30"]),
        (&["user", "group"], "user groups", "users",
            &["groups", "id"]),
        (&["member", "group"], "group members", "users",
            &["echo 'Usage: getent group <groupname>'"]),
        // Sudo
        (&["sudo", "user"], "sudo users", "users",
            &["getent group sudo wheel 2>/dev/null | cut -d: -f4"]),
        (&["sudoer"], "sudoers", "users",
            &["cat /etc/sudoers 2>/dev/null | grep -v '^#' | grep -v '^$' | head -20"]),
        (&["who", "sudo"], "sudo access", "users",
            &["getent group sudo wheel 2>/dev/null"]),
        // Add/remove user
        (&["add", "user"], "add user", "users",
            &["echo 'Add user: sudo useradd -m <username>'",
              "echo 'Set password: sudo passwd <username>'"]),
        (&["create", "user"], "create user", "users",
            &["echo 'Create: sudo useradd -m -s /bin/bash <username>'",
              "echo 'With home: sudo useradd -m <username>'"]),
        (&["delete", "user"], "delete user", "users",
            &["echo 'Delete user: sudo userdel <username>'",
              "echo 'With home: sudo userdel -r <username>'"]),
        (&["remove", "user"], "remove user", "users",
            &["echo 'Remove: sudo userdel <username>'",
              "echo 'With home dir: sudo userdel -r <username>'"]),
        // Login history
        (&["login", "history"], "login history", "users",
            &["last -n 20"]),
        (&["last", "login"], "last logins", "users",
            &["lastlog | grep -v 'Never logged in' | head -20"]),
        (&["failed", "login"], "failed logins", "users",
            &["sudo lastb | head -20 2>/dev/null || journalctl -u sshd | grep -i failed | tail -20"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Factual,
                confidence: 0.9,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

fn match_ssh(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[SecPattern] = &[
        // SSH status
        (&["ssh", "status"], "SSH service status", "ssh",
            &["systemctl status sshd"]),
        (&["sshd", "status"], "SSH daemon status", "ssh",
            &["systemctl status sshd"]),
        // SSH keys
        (&["ssh", "key"], "SSH keys", "ssh",
            &["ls -la ~/.ssh/", "cat ~/.ssh/authorized_keys 2>/dev/null | head -5"]),
        (&["authorized_key"], "authorized keys", "ssh",
            &["cat ~/.ssh/authorized_keys 2>/dev/null"]),
        (&["generate", "ssh"], "generate SSH key", "ssh",
            &["echo 'Generate: ssh-keygen -t ed25519 -C \"comment\"'",
              "echo 'Or RSA: ssh-keygen -t rsa -b 4096'"]),
        // SSH config
        (&["ssh", "config"], "SSH configuration", "ssh",
            &["cat /etc/ssh/sshd_config 2>/dev/null | grep -v '^#' | grep -v '^$' | head -30"]),
        (&["sshd_config"], "SSHD configuration", "ssh",
            &["cat /etc/ssh/sshd_config | grep -v '^#' | grep -v '^$' | head -30"]),
        // SSH troubleshooting
        (&["ssh", "connection", "refused"], "SSH connection refused", "ssh",
            &["systemctl status sshd", "ss -tlnp | grep 22",
              "echo 'Check: sudo systemctl start sshd'"]),
        (&["ssh", "timeout"], "SSH timeout", "ssh",
            &["echo 'Try: ssh -v <host> to debug'",
              "cat /etc/ssh/sshd_config | grep -i timeout"]),
        (&["ssh", "permission", "denied"], "SSH permission denied", "ssh",
            &["ls -la ~/.ssh/", "cat /etc/ssh/sshd_config | grep -E 'PasswordAuth|PubkeyAuth'"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Factual,
                confidence: 0.9,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

fn match_audit(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[SecPattern] = &[
        // Security audit
        (&["security", "audit"], "security audit", "security",
            &["sudo lynis audit system 2>/dev/null | head -50 || echo 'Install: pacman -S lynis'"]),
        (&["lynis"], "lynis audit", "security",
            &["sudo lynis audit system 2>/dev/null | head -50"]),
        // Failed services
        (&["failed", "service"], "failed services", "security",
            &["systemctl --failed"]),
        // Open ports
        (&["open", "port"], "open ports", "security",
            &["ss -tlnp 2>/dev/null | head -20"]),
        (&["listening", "port"], "listening ports", "security",
            &["ss -tlnp 2>/dev/null | head -20"]),
        // Installed packages security
        (&["package", "vulnerab"], "package vulnerabilities", "security",
            &["pacman -Qu 2>/dev/null | head -20 || apt list --upgradable 2>/dev/null | head -20"]),
        // Process running as root
        (&["process", "root"], "processes running as root", "security",
            &["ps aux | awk '$1 == \"root\" {print}' | head -20"]),
        // Cron jobs
        (&["cron", "job"], "cron jobs", "security",
            &["crontab -l 2>/dev/null", "ls -la /etc/cron.d/ 2>/dev/null"]),
        (&["crontab"], "crontab", "security",
            &["crontab -l 2>/dev/null || echo 'No crontab for current user'"]),
        // SELinux/AppArmor
        (&["selinux", "status"], "SELinux status", "security",
            &["getenforce 2>/dev/null || echo 'SELinux not installed'"]),
        (&["apparmor", "status"], "AppArmor status", "security",
            &["sudo aa-status 2>/dev/null || echo 'AppArmor not installed'"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Factual,
                confidence: 0.9,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_firewall_status() {
        let result = match_patterns("firewall status");
        assert!(result.is_some());
    }

    #[test]
    fn test_ssh_keys() {
        let result = match_patterns("list ssh keys");
        assert!(result.is_some());
    }

    #[test]
    fn test_list_users() {
        let result = match_patterns("list all users");
        assert!(result.is_some());
    }

    #[test]
    fn test_sudo_users() {
        let result = match_patterns("who has sudo access");
        assert!(result.is_some());
    }
}
