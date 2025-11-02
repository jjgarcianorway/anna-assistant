//! Distro detection for Anna v0.12.9
//!
//! Auto-detect Linux distribution from /etc/os-release

use anyhow::Result;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone, PartialEq)]
pub enum DistroProvider {
    Arch,
    Debian,
    Fedora,
    Rhel,
    OpenSuse,
    Generic,
}

impl DistroProvider {
    pub fn as_str(&self) -> &str {
        match self {
            DistroProvider::Arch => "arch",
            DistroProvider::Debian => "debian",
            DistroProvider::Fedora => "fedora",
            DistroProvider::Rhel => "rhel",
            DistroProvider::OpenSuse => "opensuse",
            DistroProvider::Generic => "generic",
        }
    }
}

/// Detect Linux distribution
pub fn detect_distro() -> Result<DistroProvider> {
    // Parse /etc/os-release
    let os_release = fs::read_to_string("/etc/os-release")
        .or_else(|_| fs::read_to_string("/usr/lib/os-release"))?;

    let vars = parse_os_release(&os_release);

    // Check ID first
    if let Some(id) = vars.get("ID") {
        match id.as_str() {
            "arch" | "archlinux" | "manjaro" | "endeavouros" => {
                return Ok(DistroProvider::Arch);
            }
            "debian" | "ubuntu" | "mint" | "pop" => {
                return Ok(DistroProvider::Debian);
            }
            "fedora" => {
                return Ok(DistroProvider::Fedora);
            }
            "rhel" | "centos" | "rocky" | "alma" => {
                return Ok(DistroProvider::Rhel);
            }
            "opensuse" | "opensuse-leap" | "opensuse-tumbleweed" | "sles" => {
                return Ok(DistroProvider::OpenSuse);
            }
            _ => {}
        }
    }

    // Check ID_LIKE as fallback
    if let Some(id_like) = vars.get("ID_LIKE") {
        if id_like.contains("arch") {
            return Ok(DistroProvider::Arch);
        }
        if id_like.contains("debian") || id_like.contains("ubuntu") {
            return Ok(DistroProvider::Debian);
        }
        if id_like.contains("fedora") {
            return Ok(DistroProvider::Fedora);
        }
        if id_like.contains("rhel") {
            return Ok(DistroProvider::Rhel);
        }
        if id_like.contains("suse") {
            return Ok(DistroProvider::OpenSuse);
        }
    }

    // Generic fallback
    eprintln!("Warning: Unknown distribution, using generic provider");
    Ok(DistroProvider::Generic)
}

/// Parse /etc/os-release into key-value map
fn parse_os_release(content: &str) -> HashMap<String, String> {
    let mut vars = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim().to_string();
            let value = value
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
            vars.insert(key, value);
        }
    }

    vars
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_os_release() {
        let content = r#"
ID=arch
ID_LIKE=""
NAME="Arch Linux"
PRETTY_NAME="Arch Linux"
"#;

        let vars = parse_os_release(content);
        assert_eq!(vars.get("ID"), Some(&"arch".to_string()));
        assert_eq!(vars.get("NAME"), Some(&"Arch Linux".to_string()));
    }

    #[test]
    fn test_detect_arch() {
        let content = "ID=arch\n";
        let vars = parse_os_release(content);
        assert_eq!(vars.get("ID").map(|s| s.as_str()), Some("arch"));
    }

    #[test]
    fn test_detect_ubuntu() {
        let content = "ID=ubuntu\nID_LIKE=debian\n";
        let vars = parse_os_release(content);
        assert_eq!(vars.get("ID").map(|s| s.as_str()), Some("ubuntu"));
        assert_eq!(vars.get("ID_LIKE").map(|s| s.as_str()), Some("debian"));
    }

    #[test]
    fn test_detect_fedora() {
        let content = "ID=fedora\n";
        let vars = parse_os_release(content);
        assert_eq!(vars.get("ID").map(|s| s.as_str()), Some("fedora"));
    }
}
