//! Enhanced Redaction (v0.0.446).
//!
//! Mandatory redaction of secrets before printing or logging.
//! Never print sensitive data, even at debug level 3.

use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

/// Safe environment variables (allowlist).
static SAFE_ENV_VARS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    let vars = [
        "PATH", "HOME", "USER", "SHELL", "LANG", "LC_ALL", "TERM",
        "XDG_CURRENT_DESKTOP", "XDG_SESSION_TYPE", "DISPLAY", "WAYLAND_DISPLAY",
        "DESKTOP_SESSION", "PWD", "OLDPWD", "HOSTNAME", "EDITOR", "VISUAL",
        "TZ", "XDG_CONFIG_HOME", "XDG_DATA_HOME", "XDG_CACHE_HOME",
    ];
    vars.into_iter().collect()
});

/// Regex patterns for sensitive data.
static TOKEN_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(token|bearer|oauth|api[_-]?key|secret|credential)\s*[=:]\s*[^\s\n]+")
        .unwrap()
});

static PASSWORD_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(password|passwd|pwd|pass)\s*[=:]\s*[^\s\n]+").unwrap()
});

static PSK_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(psk|pre[-_]?shared[-_]?key|wpa[-_]?passphrase)\s*[=:]\s*[^\s\n]+").unwrap()
});

static SSH_KEY_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"-----BEGIN [A-Z ]+ PRIVATE KEY-----[\s\S]*?-----END [A-Z ]+ PRIVATE KEY-----")
        .unwrap()
});

static AUTH_HEADER_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)Authorization:\s*[^\r\n]+").unwrap()
});

static PRIVATE_KEY_CONTENT: LazyLock<Regex> = LazyLock::new(|| {
    // Matches base64-like content that looks like key material
    Regex::new(r"[A-Za-z0-9+/]{40,}={0,2}").unwrap()
});

static ENV_VAR_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    // Match VAR=value where value stops at space, newline, or end
    Regex::new(r"([A-Z_][A-Z0-9_]*)=([^\s]+)").unwrap()
});

static HASH_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    // Shadow file hashes ($6$..., $y$..., etc.)
    Regex::new(r"\$[0-9a-z]+\$[^\s:]+").unwrap()
});

static AWS_KEY_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(AKIA|ABIA|ACCA|ASIA)[A-Z0-9]{16}").unwrap()
});

static JWT_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"eyJ[A-Za-z0-9_-]+\.eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+").unwrap()
});

/// Redaction placeholder.
const REDACTED: &str = "***REDACTED***";

/// Redactor for sensitive data.
#[derive(Debug, Clone)]
pub struct Redactor {
    /// Redact private IPs
    pub redact_ips: bool,
    /// Redact emails
    pub redact_emails: bool,
    /// Maximum lines to keep
    pub max_lines: usize,
}

impl Default for Redactor {
    fn default() -> Self {
        Self {
            redact_ips: true,
            redact_emails: true,
            max_lines: 100,
        }
    }
}

impl Redactor {
    /// Create with custom settings.
    pub fn new(redact_ips: bool, redact_emails: bool, max_lines: usize) -> Self {
        Self {
            redact_ips,
            redact_emails,
            max_lines,
        }
    }

    /// Redact all sensitive data from text.
    pub fn redact(&self, input: &str) -> String {
        let mut text = input.to_string();

        // Always redact: tokens, passwords, PSK, SSH keys, auth headers
        text = TOKEN_PATTERN.replace_all(&text, REDACTED).to_string();
        text = PASSWORD_PATTERN.replace_all(&text, REDACTED).to_string();
        text = PSK_PATTERN.replace_all(&text, REDACTED).to_string();
        text = SSH_KEY_PATTERN.replace_all(&text, REDACTED).to_string();
        text = AUTH_HEADER_PATTERN.replace_all(&text, REDACTED).to_string();
        text = HASH_PATTERN.replace_all(&text, REDACTED).to_string();
        text = AWS_KEY_PATTERN.replace_all(&text, REDACTED).to_string();
        text = JWT_PATTERN.replace_all(&text, REDACTED).to_string();

        // Redact unsafe environment variables
        text = self.redact_env_vars(&text);

        // Optionally redact IPs
        if self.redact_ips {
            text = self.redact_private_ips(&text);
        }

        // Optionally redact emails
        if self.redact_emails {
            text = self.redact_email_addresses(&text);
        }

        // Truncate if too long
        self.truncate_lines(&text)
    }

    /// Redact environment variables except safe ones.
    fn redact_env_vars(&self, input: &str) -> String {
        ENV_VAR_PATTERN
            .replace_all(input, |caps: &regex::Captures| {
                let var_name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                if SAFE_ENV_VARS.contains(var_name) {
                    caps[0].to_string()
                } else {
                    format!("{}={}", var_name, REDACTED)
                }
            })
            .to_string()
    }

    /// Redact private IP addresses.
    fn redact_private_ips(&self, input: &str) -> String {
        // 10.x.x.x, 172.16-31.x.x, 192.168.x.x
        let re = Regex::new(
            r"(?:10\.\d{1,3}\.\d{1,3}\.\d{1,3}|172\.(?:1[6-9]|2\d|3[0-1])\.\d{1,3}\.\d{1,3}|192\.168\.\d{1,3}\.\d{1,3})"
        ).unwrap();
        re.replace_all(input, REDACTED).to_string()
    }

    /// Redact email addresses.
    fn redact_email_addresses(&self, input: &str) -> String {
        let re = Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap();
        re.replace_all(input, REDACTED).to_string()
    }

    /// Truncate to max lines.
    fn truncate_lines(&self, input: &str) -> String {
        let lines: Vec<&str> = input.lines().collect();
        if lines.len() <= self.max_lines {
            return input.to_string();
        }

        let truncated: Vec<&str> = lines[..self.max_lines].to_vec();
        format!(
            "{}\n...TRUNCATED ({} more lines)",
            truncated.join("\n"),
            lines.len() - self.max_lines
        )
    }

    /// Redact command line (hide sensitive args).
    pub fn redact_command(&self, cmd: &str) -> String {
        let mut result = cmd.to_string();

        // Redact common sensitive command patterns
        let patterns = [
            (r"--password[=\s]+\S+", "--password=***REDACTED***"),
            (r"--token[=\s]+\S+", "--token=***REDACTED***"),
            (r"--key[=\s]+\S+", "--key=***REDACTED***"),
            (r"-p\s+\S+", "-p ***REDACTED***"),
        ];

        for (pattern, replacement) in patterns {
            if let Ok(re) = Regex::new(pattern) {
                result = re.replace_all(&result, replacement).to_string();
            }
        }

        result
    }

    /// Check if text contains sensitive patterns.
    pub fn contains_sensitive(&self, input: &str) -> bool {
        TOKEN_PATTERN.is_match(input)
            || PASSWORD_PATTERN.is_match(input)
            || PSK_PATTERN.is_match(input)
            || SSH_KEY_PATTERN.is_match(input)
            || AUTH_HEADER_PATTERN.is_match(input)
            || AWS_KEY_PATTERN.is_match(input)
            || JWT_PATTERN.is_match(input)
    }
}

/// Check if a path should never be shown in full.
pub fn is_sensitive_path(path: &str) -> bool {
    let sensitive_paths = [
        "/etc/shadow",
        "/etc/gshadow",
        "/etc/passwd",
        "/.ssh/",
        "/id_rsa",
        "/id_ed25519",
        "/id_ecdsa",
        "/.gnupg/",
        "/.config/gcloud/",
        "/.aws/credentials",
        "/.netrc",
    ];

    let path_lower = path.to_lowercase();
    sensitive_paths.iter().any(|p| path_lower.contains(p))
}

/// Redact /proc/cmdline content.
pub fn redact_proc_cmdline(cmdline: &str) -> String {
    // Only show non-sensitive kernel params
    let safe_params = ["root=", "ro", "quiet", "splash", "init=", "rw", "single"];

    let parts: Vec<&str> = cmdline.split_whitespace().collect();
    let redacted: Vec<String> = parts
        .iter()
        .map(|p| {
            if safe_params.iter().any(|safe| p.starts_with(safe)) {
                p.to_string()
            } else if p.contains('=') {
                let key = p.split('=').next().unwrap_or("");
                format!("{}={}", key, REDACTED)
            } else {
                p.to_string()
            }
        })
        .collect();

    redacted.join(" ")
}

/// Redact journal log lines.
pub fn redact_journal_line(line: &str) -> String {
    let redactor = Redactor::default();
    redactor.redact(line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_tokens() {
        let redactor = Redactor::default();

        let input = "API_KEY=sk-secret123 TOKEN=abc123";
        let result = redactor.redact(input);
        assert!(result.contains(REDACTED));
        assert!(!result.contains("sk-secret123"));
        assert!(!result.contains("abc123"));
    }

    #[test]
    fn test_redact_passwords() {
        let redactor = Redactor::default();

        let input = "password=hunter2 PASSWD=secret";
        let result = redactor.redact(input);
        assert!(result.contains(REDACTED));
        assert!(!result.contains("hunter2"));
        assert!(!result.contains("secret"));
    }

    #[test]
    fn test_redact_ssh_key() {
        let redactor = Redactor::default();

        let input = r#"-----BEGIN RSA PRIVATE KEY-----
MIIEpAIBAAKCAQEA0Z...
-----END RSA PRIVATE KEY-----"#;
        let result = redactor.redact(input);
        assert!(result.contains(REDACTED));
        assert!(!result.contains("MIIEpAIBAAKCAQEA0Z"));
    }

    #[test]
    fn test_redact_jwt() {
        let redactor = Redactor::default();

        let input = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U";
        let result = redactor.redact(input);
        assert!(result.contains(REDACTED));
        assert!(!result.contains("eyJhbGciOiJIUzI1NiI"));
    }

    #[test]
    fn test_redact_env_vars() {
        let redactor = Redactor::default();

        let input = "PATH=/usr/bin SECRET_KEY=mysecret HOME=/home/user";
        let result = redactor.redact(input);

        // Safe vars should be kept
        assert!(result.contains("PATH=/usr/bin"));
        assert!(result.contains("HOME=/home/user"));

        // Unsafe vars should be redacted
        assert!(!result.contains("mysecret"));
    }

    #[test]
    fn test_safe_env_vars() {
        assert!(SAFE_ENV_VARS.contains("PATH"));
        assert!(SAFE_ENV_VARS.contains("HOME"));
        assert!(SAFE_ENV_VARS.contains("XDG_CURRENT_DESKTOP"));
        assert!(!SAFE_ENV_VARS.contains("AWS_SECRET_KEY"));
    }

    #[test]
    fn test_redact_command() {
        let redactor = Redactor::default();

        let cmd = "curl --token=secret123 https://api.example.com";
        let result = redactor.redact_command(cmd);
        assert!(!result.contains("secret123"));
    }

    #[test]
    fn test_sensitive_path_detection() {
        assert!(is_sensitive_path("/etc/shadow"));
        assert!(is_sensitive_path("/home/user/.ssh/id_rsa"));
        assert!(is_sensitive_path("/root/.aws/credentials"));
        assert!(!is_sensitive_path("/etc/hostname"));
        assert!(!is_sensitive_path("/var/log/syslog"));
    }

    #[test]
    fn test_redact_proc_cmdline() {
        let cmdline = "root=/dev/sda1 ro quiet cryptkey=/dev/sdb1:ext4:/key";
        let result = redact_proc_cmdline(cmdline);

        assert!(result.contains("root=/dev/sda1"));
        assert!(result.contains("ro"));
        assert!(result.contains("quiet"));
        assert!(result.contains("cryptkey=***REDACTED***"));
    }

    #[test]
    fn test_truncate_lines() {
        let redactor = Redactor::new(false, false, 5);
        let input = "line1\nline2\nline3\nline4\nline5\nline6\nline7";
        let result = redactor.redact(input);

        assert!(result.contains("line1"));
        assert!(result.contains("line5"));
        assert!(result.contains("TRUNCATED"));
        assert!(!result.contains("line7"));
    }

    #[test]
    fn test_no_false_positives() {
        let redactor = Redactor::new(false, false, 100);

        // Normal text should not be redacted
        let input = "The disk is 80% full. Memory usage is high.";
        let result = redactor.redact(input);
        assert_eq!(input, result);
    }
}
