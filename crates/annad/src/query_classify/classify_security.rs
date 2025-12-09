//! Security query classification patterns (v0.0.174).
//!
//! Firewall, iptables, SELinux, AppArmor, SSH, logins, sudoers.

use crate::router::QueryClass;

/// Classify security queries.
/// Returns Some if matched, None otherwise.
pub fn classify_security(q: &str) -> Option<QueryClass> {
    // v0.0.128: Boot loader
    if q.contains("bootloader")
        || q.contains("boot loader")
        || q.contains("grub")
        || q.contains("systemd-boot")
        || q.contains("bootctl")
        || (q.contains("what") && q.contains("boot") && !q.contains("last boot"))
    {
        return Some(QueryClass::BootLoader);
    }

    // v0.0.128: Firewall status
    if q.contains("firewall")
        || q.contains("iptables")
        || q.contains("nftables")
        || q.contains("ufw")
        || (q.contains("port") && q.contains("block"))
    {
        return Some(QueryClass::FirewallStatus);
    }

    // v0.0.128: SSH connections
    if q.contains("ssh connection")
        || q.contains("ssh session")
        || (q.contains("who") && q.contains("ssh"))
        || (q.contains("connected") && q.contains("ssh"))
        || q.contains("remote connection")
    {
        return Some(QueryClass::SshConnections);
    }

    // v0.0.129: Last logins
    if q.contains("last login")
        || q.contains("login history")
        || q.contains("recent login")
        || q.contains("who logged in")
        || q.trim() == "last"
    {
        return Some(QueryClass::LastLogins);
    }

    // v0.0.129: Failed logins
    if q.contains("failed login")
        || q.contains("login failure")
        || q.contains("unsuccessful login")
        || q.contains("bad login")
        || q.trim() == "lastb"
    {
        return Some(QueryClass::FailedLogins);
    }

    // v0.0.130: Sudoers info
    if q.contains("sudo access")
        || q.contains("sudoers")
        || q.contains("sudo privilege")
        || q.contains("sudo permission")
        || (q.contains("can i") && q.contains("sudo"))
        || q.trim() == "sudo -l"
    {
        return Some(QueryClass::SudoersInfo);
    }

    // v0.0.131: SELinux status
    if q.contains("selinux") || q.trim() == "sestatus" || q.contains("security enhanced linux") {
        return Some(QueryClass::SelinuxStatus);
    }

    // v0.0.131: AppArmor status
    if q.contains("apparmor") || q.trim() == "aa-status" || q.contains("app armor") {
        return Some(QueryClass::AppArmorStatus);
    }

    // v0.0.132: Iptables rules
    if q.contains("iptables rule")
        || q.contains("netfilter")
        || q.trim() == "iptables"
        || q.contains("iptables -l")
        || (q.contains("firewall") && q.contains("rule"))
    {
        return Some(QueryClass::IptablesRules);
    }

    // v0.0.104: SSH key management
    let is_ssh = q.contains("ssh")
        && (q.contains("key")
            || q.contains("keygen")
            || q.contains("generate")
            || q.contains("create")
            || q.contains("copy")
            || q.contains("ssh-copy")
            || q.contains("config")
            || q.contains("agent")
            || q.contains("github")
            || q.contains("gitlab")
            || q.contains("authorized")
            || q.contains("passphrase"));
    if is_ssh {
        return Some(QueryClass::SshKeyManagement);
    }

    // v0.0.136: Hosts file
    if q.contains("/etc/hosts")
        || q.contains("hosts file")
        || q.contains("host entry")
        || q.contains("host entries")
        || (q.contains("show") && q.contains("hosts"))
    {
        return Some(QueryClass::HostsFile);
    }

    None
}
