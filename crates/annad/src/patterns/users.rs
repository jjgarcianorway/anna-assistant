//! User and group management patterns.
//! v0.0.965: Initial implementation.

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Helper to create a users-related DeepUnderstanding
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

type UsersPattern<'a> = (&'a [&'a str], &'a str, &'a str, &'a [&'a str]);

/// Match user/group-related patterns
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    match_list_users(q)
        .or_else(|| match_user_info(q))
        .or_else(|| match_groups(q))
        .or_else(|| match_password(q))
        .or_else(|| match_login_history(q))
        .or_else(|| match_shells(q))
}

/// List users patterns
fn match_list_users(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[UsersPattern] = &[
        // All users
        (&["all", "users"], "list all users", "users",
         &["cat /etc/passwd | cut -d: -f1"]),
        (&["list", "users"], "list users", "users",
         &["cat /etc/passwd | cut -d: -f1,3,6"]),
        (&["system", "users"], "list system users", "users",
         &["awk -F: '$3 < 1000 {print $1}' /etc/passwd"]),
        // Human users
        (&["human", "users"], "list human users", "users",
         &["awk -F: '$3 >= 1000 && $3 < 65534 {print $1}' /etc/passwd"]),
        (&["real", "users"], "list real users", "users",
         &["awk -F: '$3 >= 1000 && $3 < 65534 {print $1}' /etc/passwd"]),
        (&["normal", "users"], "list normal users", "users",
         &["awk -F: '$3 >= 1000 && $3 < 65534 {print $1}' /etc/passwd"]),
        // User count
        (&["how", "many", "users"], "count users", "users",
         &["wc -l < /etc/passwd"]),
        (&["count", "users"], "count users", "users",
         &["wc -l < /etc/passwd"]),
        // Logged in users
        (&["logged", "in", "users"], "show logged in users", "users",
         &["who", "w"]),
        (&["who", "logged"], "show who is logged in", "users",
         &["who", "w"]),
        (&["active", "users"], "show active users", "users",
         &["who", "w -h"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// User info patterns
fn match_user_info(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[UsersPattern] = &[
        // Current user
        (&["current", "user"], "show current user", "users",
         &["whoami", "id"]),
        (&["my", "username"], "show my username", "users",
         &["whoami"]),
        (&["who", "am", "i"], "show who am i", "users",
         &["whoami", "id"]),
        // User ID - Note: avoid short keywords like "id" that match substrings (bandwidth, idle)
        (&["my", "uid"], "show my UID", "users",
         &["id -u"]),
        (&["my", "gid"], "show my GID", "users",
         &["id -g"]),
        (&["my", "userid"], "show my user ID", "users",
         &["id"]),
        (&["my", "user", "id"], "show my user ID", "users",
         &["id"]),
        (&["show", "id"], "show user ID", "users",
         &["id"]),
        // User details
        (&["user", "details"], "show user details", "users",
         &["echo 'Use: id <username> or getent passwd <username>'"]),
        (&["user", "info"], "show user info", "users",
         &["id", "finger $USER 2>/dev/null || getent passwd $USER"]),
        // Home directory
        (&["my", "home"], "show my home directory", "users",
         &["echo $HOME", "pwd"]),
        (&["home", "directory"], "show home directory", "users",
         &["echo $HOME"]),
        (&["home", "directories"], "list home directories", "users",
         &["ls -la /home/"]),
        // User environment
        (&["my", "environment"], "show my environment", "users",
         &["env | sort"]),
        (&["my", "path"], "show my PATH", "users",
         &["echo $PATH | tr ':' '\\n'"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Group patterns
fn match_groups(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[UsersPattern] = &[
        // My groups
        (&["my", "groups"], "show my groups", "users",
         &["groups", "id -Gn"]),
        // All groups
        (&["all", "groups"], "list all groups", "users",
         &["cat /etc/group | cut -d: -f1"]),
        (&["list", "groups"], "list groups", "users",
         &["cat /etc/group | cut -d: -f1"]),
        // System groups
        (&["system", "groups"], "list system groups", "users",
         &["awk -F: '$3 < 1000 {print $1}' /etc/group"]),
        // Group members
        (&["group", "members"], "show group members", "users",
         &["echo 'Use: getent group <groupname>'", "cat /etc/group"]),
        // Wheel/sudo group
        (&["wheel", "group"], "show wheel group members", "users",
         &["getent group wheel"]),
        (&["sudo", "group"], "show sudo group members", "users",
         &["getent group sudo 2>/dev/null || getent group wheel"]),
        // Who can sudo
        (&["who", "sudo"], "show who can sudo", "users",
         &["getent group wheel 2>/dev/null || getent group sudo", "cat /etc/sudoers.d/* 2>/dev/null | grep -v '^#'"]),
        (&["sudoers"], "show sudoers", "users",
         &["cat /etc/sudoers | grep -v '^#' | grep -v '^$'", "ls /etc/sudoers.d/"]),
        // Group info
        (&["group", "info"], "show group info", "users",
         &["echo 'Use: getent group <groupname>'"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Password patterns
fn match_password(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[UsersPattern] = &[
        // Password status
        (&["password", "status"], "check password status", "users",
         &["passwd -S $USER"]),
        (&["password", "expiry"], "check password expiry", "users",
         &["chage -l $USER"]),
        (&["password", "aging"], "check password aging", "users",
         &["chage -l $USER"]),
        // Account status
        (&["account", "expiry"], "check account expiry", "users",
         &["chage -l $USER"]),
        (&["account", "status"], "check account status", "users",
         &["passwd -S $USER", "chage -l $USER"]),
        // Locked accounts
        (&["locked", "accounts"], "show locked accounts", "users",
         &["passwd -S -a 2>/dev/null | grep ' L ' || awk -F: '$2 ~ /^!/ {print $1}' /etc/shadow 2>/dev/null"]),
        (&["locked", "users"], "show locked users", "users",
         &["passwd -S -a 2>/dev/null | grep ' L '"]),
        // Password policy
        (&["password", "policy"], "show password policy", "users",
         &["cat /etc/login.defs | grep -E '^PASS'"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Login history patterns
fn match_login_history(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[UsersPattern] = &[
        // Last logins
        (&["last", "logins"], "show last logins", "users",
         &["last -n 20"]),
        (&["login", "history"], "show login history", "users",
         &["last -n 30"]),
        (&["recent", "logins"], "show recent logins", "users",
         &["last -n 20"]),
        // Failed logins
        (&["failed", "logins"], "show failed logins", "users",
         &["lastb -n 20 2>/dev/null || journalctl -u sshd | grep -i 'failed\\|invalid' | tail -20"]),
        (&["login", "failures"], "show login failures", "users",
         &["lastb -n 20 2>/dev/null"]),
        // Last login per user
        (&["lastlog"], "show last login per user", "users",
         &["lastlog | grep -v 'Never'"]),
        (&["user", "last", "login"], "show when users last logged in", "users",
         &["lastlog | grep -v 'Never'"]),
        // Current sessions
        (&["current", "sessions"], "show current sessions", "users",
         &["who", "loginctl list-sessions"]),
        (&["login", "sessions"], "show login sessions", "users",
         &["loginctl list-sessions"]),
        // Boot history
        (&["reboot", "history"], "show reboot history", "users",
         &["last reboot | head -20"]),
        (&["shutdown", "history"], "show shutdown history", "users",
         &["last -x shutdown | head -20"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Shell patterns
fn match_shells(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[UsersPattern] = &[
        // My shell
        (&["my", "shell"], "show my shell", "users",
         &["echo $SHELL", "getent passwd $USER | cut -d: -f7"]),
        (&["current", "shell"], "show current shell", "users",
         &["echo $SHELL", "echo $0"]),
        // Available shells
        (&["available", "shells"], "list available shells", "users",
         &["cat /etc/shells"]),
        (&["installed", "shells"], "list installed shells", "users",
         &["cat /etc/shells"]),
        // Default shell
        (&["default", "shell"], "show default shell", "users",
         &["getent passwd $USER | cut -d: -f7"]),
        // Shell version
        (&["shell", "version"], "show shell version", "users",
         &["$SHELL --version 2>/dev/null || echo $SHELL"]),
        (&["bash", "version"], "show bash version", "users",
         &["bash --version | head -1"]),
        (&["zsh", "version"], "show zsh version", "users",
         &["zsh --version 2>/dev/null || echo 'zsh not installed'"]),
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
    fn test_list_users() {
        assert!(match_patterns("all users").is_some());
        assert!(match_patterns("list users").is_some());
        assert!(match_patterns("logged in users").is_some());
        assert!(match_patterns("human users").is_some());
    }

    #[test]
    fn test_user_info() {
        assert!(match_patterns("current user").is_some());
        assert!(match_patterns("my username").is_some());
        assert!(match_patterns("my home").is_some());
        assert!(match_patterns("home directories").is_some());
    }

    #[test]
    fn test_groups() {
        assert!(match_patterns("my groups").is_some());
        assert!(match_patterns("all groups").is_some());
        assert!(match_patterns("wheel group").is_some());
        assert!(match_patterns("sudoers").is_some());
    }

    #[test]
    fn test_password() {
        assert!(match_patterns("password status").is_some());
        assert!(match_patterns("password expiry").is_some());
        assert!(match_patterns("locked accounts").is_some());
    }

    #[test]
    fn test_login_history() {
        assert!(match_patterns("last logins").is_some());
        assert!(match_patterns("failed logins").is_some());
        assert!(match_patterns("login history").is_some());
        assert!(match_patterns("reboot history").is_some());
    }

    #[test]
    fn test_shells() {
        assert!(match_patterns("my shell").is_some());
        assert!(match_patterns("available shells").is_some());
        assert!(match_patterns("bash version").is_some());
    }
}
