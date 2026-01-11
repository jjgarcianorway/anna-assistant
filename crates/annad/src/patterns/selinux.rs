//! SELinux and AppArmor patterns for Linux security modules.
//! v0.0.987: Initial implementation.

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Helper to create a security module-related DeepUnderstanding
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

type SecPattern<'a> = (&'a [&'a str], &'a str, &'a str, &'a [&'a str]);

/// Match security module patterns
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    match_selinux(q)
        .or_else(|| match_apparmor(q))
        .or_else(|| match_mac_general(q))
}

/// SELinux patterns
fn match_selinux(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[SecPattern] = &[
        // SELinux status
        (&["selinux", "status"], "show SELinux status", "security",
         &["sestatus", "getenforce"]),
        (&["selinux", "mode"], "show SELinux mode", "security",
         &["getenforce", "sestatus"]),
        (&["selinux", "enforc"], "check SELinux enforcement", "security",
         &["getenforce"]),
        // SELinux enabled
        (&["selinux", "enabled"], "check if SELinux is enabled", "security",
         &["sestatus | grep 'SELinux status'"]),
        (&["selinux", "disabled"], "check if SELinux is disabled", "security",
         &["sestatus", "cat /etc/selinux/config | grep SELINUX="]),
        // SELinux config
        (&["selinux", "config"], "show SELinux configuration", "security",
         &["cat /etc/selinux/config"]),
        // SELinux policy
        (&["selinux", "policy"], "show SELinux policy", "security",
         &["sestatus | grep 'Loaded policy'", "seinfo 2>/dev/null || sestatus"]),
        // SELinux booleans
        (&["selinux", "boolean"], "list SELinux booleans", "security",
         &["getsebool -a | head -30"]),
        (&["sebool"], "show SELinux booleans", "security",
         &["getsebool -a | head -30"]),
        // SELinux context
        (&["selinux", "context"], "show SELinux contexts", "security",
         &["ls -Z", "id -Z"]),
        (&["security", "context"], "show security contexts", "security",
         &["ls -Z", "ps -eZ | head -20"]),
        // SELinux denials
        (&["selinux", "denial"], "show SELinux denials", "security",
         &["ausearch -m avc -ts recent 2>/dev/null | tail -20", "journalctl -t setroubleshoot"]),
        (&["selinux", "deny"], "show SELinux denials", "security",
         &["ausearch -m avc -ts recent 2>/dev/null | tail -20"]),
        (&["avc", "denial"], "show AVC denials", "security",
         &["ausearch -m avc | tail -30"]),
        // SELinux audit
        (&["selinux", "audit"], "show SELinux audit log", "security",
         &["ausearch -m avc | tail -30", "journalctl | grep 'avc:' | tail -20"]),
        // SELinux troubleshoot
        (&["selinux", "trouble"], "troubleshoot SELinux", "security",
         &["sealert -a /var/log/audit/audit.log 2>/dev/null | tail -50"]),
        // File contexts
        (&["file", "context"], "show file contexts", "security",
         &["ls -Z"]),
        (&["restorecon"], "info on restorecon", "security",
         &["echo 'Usage: restorecon -Rv /path to restore contexts'"]),
        // SELinux modules
        (&["selinux", "module"], "list SELinux modules", "security",
         &["semodule -l | head -30"]),
        // SELinux ports
        (&["selinux", "port"], "list SELinux port labels", "security",
         &["semanage port -l 2>/dev/null | head -30"]),
        // SELinux users
        (&["selinux", "user"], "list SELinux users", "security",
         &["semanage user -l 2>/dev/null"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// AppArmor patterns
fn match_apparmor(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[SecPattern] = &[
        // AppArmor status
        (&["apparmor", "status"], "show AppArmor status", "security",
         &["sudo aa-status", "systemctl status apparmor"]),
        (&["apparmor", "mode"], "show AppArmor mode", "security",
         &["sudo aa-status"]),
        // AppArmor enabled
        (&["apparmor", "enabled"], "check if AppArmor is enabled", "security",
         &["sudo aa-enabled", "cat /sys/module/apparmor/parameters/enabled"]),
        // AppArmor profiles
        (&["apparmor", "profile"], "list AppArmor profiles", "security",
         &["sudo aa-status", "ls /etc/apparmor.d/"]),
        (&["aa-status"], "show AppArmor status", "security",
         &["sudo aa-status"]),
        // AppArmor enforce
        (&["apparmor", "enforc"], "show enforced profiles", "security",
         &["sudo aa-status | grep -A50 'profiles are in enforce mode'"]),
        // AppArmor complain
        (&["apparmor", "complain"], "show complain mode profiles", "security",
         &["sudo aa-status | grep -A50 'profiles are in complain mode'"]),
        // AppArmor logs
        (&["apparmor", "log"], "show AppArmor logs", "security",
         &["journalctl | grep apparmor | tail -30", "dmesg | grep apparmor | tail -20"]),
        (&["apparmor", "denied"], "show AppArmor denials", "security",
         &["journalctl | grep 'apparmor=\"DENIED\"' | tail -20"]),
        // AppArmor config
        (&["apparmor", "config"], "show AppArmor configuration", "security",
         &["ls -la /etc/apparmor.d/", "cat /etc/apparmor/parser.conf 2>/dev/null"]),
        // AppArmor unconfined
        (&["apparmor", "unconfined"], "show unconfined processes", "security",
         &["sudo aa-unconfined"]),
        // AppArmor audit
        (&["apparmor", "audit"], "show AppArmor audit", "security",
         &["journalctl | grep 'apparmor' | tail -30"]),
        // AppArmor abstractions
        (&["apparmor", "abstraction"], "list AppArmor abstractions", "security",
         &["ls /etc/apparmor.d/abstractions/"]),
        // AppArmor tunables
        (&["apparmor", "tunable"], "show AppArmor tunables", "security",
         &["ls /etc/apparmor.d/tunables/", "cat /etc/apparmor.d/tunables/global 2>/dev/null"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// General MAC patterns
fn match_mac_general(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[SecPattern] = &[
        // Which security module
        (&["which", "security", "module"], "check security module", "security",
         &["cat /sys/kernel/security/lsm", "dmesg | grep -iE 'selinux|apparmor'"]),
        (&["lsm"], "show loaded security modules", "security",
         &["cat /sys/kernel/security/lsm"]),
        (&["security", "module"], "check loaded security modules", "security",
         &["cat /sys/kernel/security/lsm"]),
        // Mandatory access control
        (&["mac", "status"], "show MAC status", "security",
         &["cat /sys/kernel/security/lsm", "sestatus 2>/dev/null || sudo aa-status 2>/dev/null"]),
        // Security labels
        (&["security", "label"], "show security labels", "security",
         &["ls -Z 2>/dev/null", "getfattr -n security.selinux . 2>/dev/null"]),
        // Audit daemon
        (&["auditd", "status"], "show audit daemon status", "security",
         &["systemctl status auditd"]),
        (&["audit", "log"], "show audit log", "security",
         &["ausearch -i | tail -30 2>/dev/null || cat /var/log/audit/audit.log 2>/dev/null | tail -30"]),
        (&["ausearch"], "search audit logs", "security",
         &["ausearch -i | tail -30"]),
        // Seccomp
        (&["seccomp", "status"], "show seccomp status", "security",
         &["grep Seccomp /proc/self/status", "cat /proc/sys/kernel/seccomp/actions_avail 2>/dev/null"]),
        (&["seccomp"], "check seccomp", "security",
         &["grep Seccomp /proc/$$/status"]),
        // Capabilities
        (&["capabilit"], "show process capabilities", "security",
         &["capsh --print", "cat /proc/self/status | grep Cap"]),
        (&["getcap"], "show file capabilities", "security",
         &["getcap -r /usr/bin/ 2>/dev/null | head -20"]),
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
    fn test_selinux() {
        assert!(match_patterns("selinux status").is_some());
        assert!(match_patterns("selinux mode").is_some());
        assert!(match_patterns("selinux booleans").is_some());
    }

    #[test]
    fn test_apparmor() {
        assert!(match_patterns("apparmor status").is_some());
        assert!(match_patterns("apparmor profiles").is_some());
        assert!(match_patterns("apparmor logs").is_some());
    }

    #[test]
    fn test_mac_general() {
        assert!(match_patterns("which security module").is_some());
        assert!(match_patterns("lsm").is_some());
        assert!(match_patterns("audit log").is_some());
    }
}
