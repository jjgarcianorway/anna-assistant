//! SSH connection and tunnel patterns.
//! v0.0.970: Initial implementation.

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Helper to create an SSH-related DeepUnderstanding
fn make_understanding(interpreted: &str, topic: &str, commands: &[&str]) -> DeepUnderstanding {
    DeepUnderstanding {
        interpreted_as: interpreted.to_string(),
        category: IntentCategory::Factual,
        confidence: 0.9,
        topic: Some(topic.to_string()),
        needs_confirmation: false,
        suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
        ..Default::default()
    }
}

type SshPattern<'a> = (&'a [&'a str], &'a str, &'a str, &'a [&'a str]);

/// Match SSH-related patterns
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    match_ssh_service(q)
        .or_else(|| match_ssh_connections(q))
        .or_else(|| match_ssh_config(q))
        .or_else(|| match_ssh_keys(q))
        .or_else(|| match_ssh_troubleshoot(q))
}

/// SSH service patterns
fn match_ssh_service(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[SshPattern] = &[
        // SSH service status
        (&["sshd", "status"], "show sshd status", "ssh",
         &["systemctl status sshd"]),
        (&["ssh", "daemon", "status"], "show SSH daemon status", "ssh",
         &["systemctl status sshd"]),
        (&["ssh", "server", "status"], "show SSH server status", "ssh",
         &["systemctl status sshd"]),
        // SSH running
        (&["ssh", "running"], "check if SSH is running", "ssh",
         &["systemctl is-active sshd", "pgrep sshd"]),
        // SSH version
        (&["ssh", "version"], "show SSH version", "ssh",
         &["ssh -V"]),
        (&["openssh", "version"], "show OpenSSH version", "ssh",
         &["ssh -V"]),
        // SSH port
        (&["ssh", "port"], "show SSH port", "ssh",
         &["grep -E '^Port' /etc/ssh/sshd_config 2>/dev/null || echo 'Default: 22'", "ss -tlnp | grep ssh"]),
        (&["sshd", "port"], "show sshd listening port", "ssh",
         &["ss -tlnp | grep ssh"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// SSH connection patterns
fn match_ssh_connections(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[SshPattern] = &[
        // Active SSH connections
        (&["ssh", "connections"], "show SSH connections", "ssh",
         &["ss -tnp | grep ssh", "who | grep pts"]),
        (&["active", "ssh"], "show active SSH sessions", "ssh",
         &["who", "ss -tnp | grep ssh"]),
        (&["ssh", "sessions"], "show SSH sessions", "ssh",
         &["who", "w"]),
        // Who is connected
        (&["who", "connected", "ssh"], "show who is connected via SSH", "ssh",
         &["who | grep pts", "ss -tnp | grep ssh | grep ESTAB"]),
        // Outgoing SSH
        (&["outgoing", "ssh"], "show outgoing SSH connections", "ssh",
         &["ss -tnp | grep ssh | grep -v LISTEN"]),
        // SSH agent
        (&["ssh", "agent"], "show SSH agent status", "ssh",
         &["ssh-add -l 2>/dev/null || echo 'No agent or no keys'", "echo $SSH_AUTH_SOCK"]),
        (&["ssh", "agent", "keys"], "list SSH agent keys", "ssh",
         &["ssh-add -l"]),
        // Known hosts
        (&["known", "hosts"], "show known hosts", "ssh",
         &["cat ~/.ssh/known_hosts 2>/dev/null | wc -l", "head -10 ~/.ssh/known_hosts 2>/dev/null"]),
        (&["ssh", "known", "hosts"], "show SSH known hosts", "ssh",
         &["cat ~/.ssh/known_hosts 2>/dev/null | head -20"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// SSH config patterns
fn match_ssh_config(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[SshPattern] = &[
        // SSH config
        (&["ssh", "config"], "show SSH config", "ssh",
         &["cat ~/.ssh/config 2>/dev/null || echo 'No user SSH config'"]),
        (&["my", "ssh", "config"], "show my SSH config", "ssh",
         &["cat ~/.ssh/config 2>/dev/null"]),
        // SSHD config
        (&["sshd", "config"], "show sshd config", "ssh",
         &["cat /etc/ssh/sshd_config | grep -v '^#' | grep -v '^$' | head -40"]),
        (&["ssh", "server", "config"], "show SSH server config", "ssh",
         &["cat /etc/ssh/sshd_config | grep -v '^#' | grep -v '^$' | head -40"]),
        // SSH config hosts
        (&["ssh", "hosts"], "list SSH config hosts", "ssh",
         &["grep '^Host ' ~/.ssh/config 2>/dev/null"]),
        (&["configured", "ssh", "hosts"], "show configured SSH hosts", "ssh",
         &["grep -E '^Host |HostName ' ~/.ssh/config 2>/dev/null"]),
        // Password auth
        (&["ssh", "password", "auth"], "check SSH password authentication", "ssh",
         &["grep -i 'PasswordAuthentication' /etc/ssh/sshd_config"]),
        // Root login
        (&["ssh", "root", "login"], "check SSH root login setting", "ssh",
         &["grep -i 'PermitRootLogin' /etc/ssh/sshd_config"]),
        // SSH authorized keys
        (&["authorized", "keys"], "show authorized keys", "ssh",
         &["cat ~/.ssh/authorized_keys 2>/dev/null | wc -l", "head -5 ~/.ssh/authorized_keys 2>/dev/null"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// SSH key patterns
fn match_ssh_keys(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[SshPattern] = &[
        // List SSH keys
        (&["my", "ssh", "keys"], "list my SSH keys", "ssh",
         &["ls -la ~/.ssh/*.pub 2>/dev/null", "ls -la ~/.ssh/id_* 2>/dev/null"]),
        (&["ssh", "key", "files"], "show SSH key files", "ssh",
         &["ls -la ~/.ssh/"]),
        // SSH key fingerprint
        (&["ssh", "key", "fingerprint"], "show SSH key fingerprint", "ssh",
         &["ssh-keygen -lf ~/.ssh/id_ed25519.pub 2>/dev/null || ssh-keygen -lf ~/.ssh/id_rsa.pub 2>/dev/null"]),
        (&["ssh", "fingerprint"], "show SSH fingerprint", "ssh",
         &["for f in ~/.ssh/id_*.pub; do ssh-keygen -lf $f 2>/dev/null; done"]),
        // Host keys
        (&["host", "keys"], "show SSH host keys", "ssh",
         &["ls -la /etc/ssh/ssh_host_*_key.pub"]),
        (&["ssh", "host", "keys"], "list SSH host keys", "ssh",
         &["for f in /etc/ssh/ssh_host_*_key.pub; do ssh-keygen -lf $f 2>/dev/null; done"]),
        // Key types
        (&["ssh", "key", "type"], "show SSH key types", "ssh",
         &["for f in ~/.ssh/id_*.pub; do echo $f: $(ssh-keygen -lf $f 2>/dev/null | awk '{print $4}'); done"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// SSH troubleshooting patterns
fn match_ssh_troubleshoot(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[SshPattern] = &[
        // SSH logs
        (&["ssh", "logs"], "show SSH logs", "ssh",
         &["journalctl -u sshd -n 30", "grep -i ssh /var/log/auth.log 2>/dev/null | tail -30"]),
        (&["sshd", "logs"], "show sshd logs", "ssh",
         &["journalctl -u sshd -n 30"]),
        // SSH failed logins
        (&["ssh", "failed", "logins"], "show SSH failed logins", "ssh",
         &["journalctl -u sshd | grep -i 'failed\\|invalid' | tail -30"]),
        (&["ssh", "login", "attempts"], "show SSH login attempts", "ssh",
         &["journalctl -u sshd | grep -i 'accepted\\|failed' | tail -30"]),
        // SSH errors
        (&["ssh", "errors"], "show SSH errors", "ssh",
         &["journalctl -u sshd -p err -n 20"]),
        // SSH permissions
        (&["ssh", "permissions"], "check SSH directory permissions", "ssh",
         &["ls -la ~/.ssh/", "stat ~/.ssh ~/.ssh/id_* ~/.ssh/authorized_keys 2>/dev/null | grep -E 'File:|Access:'"]),
        // Debug connection
        (&["ssh", "debug"], "SSH debug info", "ssh",
         &["echo 'Use: ssh -vvv user@host for debugging'"]),
        // Connection refused
        (&["ssh", "connection", "refused"], "troubleshoot SSH connection refused", "ssh",
         &["systemctl is-active sshd", "ss -tlnp | grep :22", "ip a | grep inet"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssh_service() {
        assert!(match_patterns("sshd status").is_some());
        assert!(match_patterns("ssh version").is_some());
        assert!(match_patterns("ssh port").is_some());
    }

    #[test]
    fn test_ssh_connections() {
        assert!(match_patterns("ssh connections").is_some());
        assert!(match_patterns("ssh sessions").is_some());
        assert!(match_patterns("ssh agent").is_some());
        assert!(match_patterns("known hosts").is_some());
    }

    #[test]
    fn test_ssh_config() {
        assert!(match_patterns("ssh config").is_some());
        assert!(match_patterns("sshd config").is_some());
        assert!(match_patterns("authorized keys").is_some());
    }

    #[test]
    fn test_ssh_keys() {
        assert!(match_patterns("my ssh keys").is_some());
        assert!(match_patterns("ssh fingerprint").is_some());
        assert!(match_patterns("host keys").is_some());
    }

    #[test]
    fn test_ssh_troubleshoot() {
        assert!(match_patterns("ssh logs").is_some());
        assert!(match_patterns("ssh failed logins").is_some());
        assert!(match_patterns("ssh permissions").is_some());
    }
}
