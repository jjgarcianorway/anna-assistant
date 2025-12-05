//! Output parsers for system command outputs.
//!
//! Parses stdout from probes like ps, df, ip addr, lscpu, free.

use anna_shared::rpc::ProbeResult;

/// Find a probe by command prefix
pub fn find_probe<'a>(probes: &'a [ProbeResult], cmd_prefix: &str) -> Option<&'a ProbeResult> {
    probes
        .iter()
        .find(|p| p.exit_code == 0 && p.command.starts_with(cmd_prefix))
}

/// Parse lscpu output for model and cores
pub fn parse_lscpu(output: &str) -> Option<(String, u32)> {
    let mut model = None;
    let mut cores = None;

    for line in output.lines() {
        if line.starts_with("Model name:") {
            model = Some(line.split(':').nth(1)?.trim().to_string());
        } else if line.starts_with("CPU(s):") {
            cores = line.split(':').nth(1)?.trim().parse().ok();
        }
    }

    Some((model?, cores?))
}

/// Parse free -h output for total memory
pub fn parse_free_total(output: &str) -> Option<String> {
    for line in output.lines() {
        if line.starts_with("Mem:") {
            return line.split_whitespace().nth(1).map(String::from);
        }
    }
    None
}

/// Process info from ps aux
pub struct ProcessInfo {
    pub pid: String,
    pub user: String,
    pub cpu_percent: String,
    pub mem_percent: String,
    pub rss: Option<String>, // Resident set size in KB
    pub command: String,
}

/// Parse ps aux output
/// Format: USER PID %CPU %MEM VSZ RSS TTY STAT START TIME COMMAND
pub fn parse_ps_aux(output: &str, limit: usize) -> Vec<ProcessInfo> {
    output
        .lines()
        .skip(1) // Skip header
        .take(limit)
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 11 {
                // RSS is in KB, format it human-readable
                let rss_kb: Option<u64> = parts[5].parse().ok();
                let rss = rss_kb.map(format_rss);
                Some(ProcessInfo {
                    pid: parts[1].to_string(),
                    user: parts[0].to_string(),
                    cpu_percent: parts[2].to_string(),
                    mem_percent: parts[3].to_string(),
                    rss,
                    command: parts[10..].join(" "),
                })
            } else {
                None
            }
        })
        .collect()
}

/// Format RSS (in KB) to human-readable
fn format_rss(kb: u64) -> String {
    if kb >= 1_048_576 {
        format!("{:.1}G", kb as f64 / 1_048_576.0)
    } else if kb >= 1024 {
        format!("{:.0}M", kb as f64 / 1024.0)
    } else {
        format!("{}K", kb)
    }
}

/// Filesystem info from df -h
pub struct FilesystemInfo {
    pub filesystem: String,
    pub size: String,
    pub used: String,
    pub avail: String,
    pub use_percent: u8,
    pub mount: String,
}

/// Parse df -h output
pub fn parse_df_h(output: &str) -> Vec<FilesystemInfo> {
    output
        .lines()
        .skip(1) // Skip header
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 6 {
                let use_str = parts[4].trim_end_matches('%');
                let use_percent = use_str.parse().unwrap_or(0);
                // Skip tmpfs and small filesystems
                if parts[0].starts_with("tmpfs") || parts[0].starts_with("devtmpfs") {
                    return None;
                }
                Some(FilesystemInfo {
                    filesystem: parts[0].to_string(),
                    size: parts[1].to_string(),
                    used: parts[2].to_string(),
                    avail: parts[3].to_string(),
                    use_percent,
                    mount: parts[5].to_string(),
                })
            } else {
                None
            }
        })
        .collect()
}

/// Network interface info from ip addr show
pub struct InterfaceInfo {
    pub name: String,
    pub ipv4: Option<String>,
    pub ipv6: Option<String>,
    pub state: String,
}

/// Parse ip addr show output
pub fn parse_ip_addr(output: &str) -> Vec<InterfaceInfo> {
    let mut interfaces = Vec::new();
    let mut current: Option<InterfaceInfo> = None;

    for line in output.lines() {
        // New interface line: "2: eth0: <...> state UP"
        if line.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            if let Some(iface) = current.take() {
                interfaces.push(iface);
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let name = parts[1].trim_end_matches(':').to_string();
                let state = if line.contains("state UP") {
                    "UP"
                } else if line.contains("state DOWN") {
                    "DOWN"
                } else {
                    "UNKNOWN"
                };
                current = Some(InterfaceInfo {
                    name,
                    ipv4: None,
                    ipv6: None,
                    state: state.to_string(),
                });
            }
        } else if let Some(ref mut iface) = current {
            let trimmed = line.trim();
            if trimmed.starts_with("inet ") {
                if let Some(addr) = trimmed.split_whitespace().nth(1) {
                    iface.ipv4 = Some(addr.split('/').next().unwrap_or(addr).to_string());
                }
            } else if trimmed.starts_with("inet6 ") && !trimmed.contains("fe80::") {
                if let Some(addr) = trimmed.split_whitespace().nth(1) {
                    iface.ipv6 = Some(addr.split('/').next().unwrap_or(addr).to_string());
                }
            }
        }
    }

    if let Some(iface) = current {
        interfaces.push(iface);
    }

    interfaces
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ps_aux() {
        let output = "USER       PID %CPU %MEM    VSZ   RSS TTY      STAT START   TIME COMMAND
root         1  0.0  0.1 169936 12456 ?        Ss   Dec01   0:03 /sbin/init
user      1234  5.0 10.2 500000 51200 ?        Sl   10:00   1:23 firefox";
        let procs = parse_ps_aux(output, 5);
        assert_eq!(procs.len(), 2);
        assert_eq!(procs[1].pid, "1234");
        assert_eq!(procs[1].command, "firefox");
        assert_eq!(procs[1].mem_percent, "10.2");
        assert_eq!(procs[1].rss, Some("50M".to_string()));
    }

    #[test]
    fn test_parse_df_h() {
        let output = "Filesystem      Size  Used Avail Use% Mounted on
/dev/sda1       100G   80G   20G  80% /
/dev/sdb1        50G   48G    2G  96% /home";
        let fs = parse_df_h(output);
        assert_eq!(fs.len(), 2);
        assert_eq!(fs[1].use_percent, 96);
        assert_eq!(fs[1].mount, "/home");
    }

    #[test]
    fn test_parse_ip_addr() {
        let output = "1: lo: <LOOPBACK,UP,LOWER_UP> mtu 65536 state UNKNOWN
    link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00
    inet 127.0.0.1/8 scope host lo
2: eth0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 state UP
    link/ether aa:bb:cc:dd:ee:ff brd ff:ff:ff:ff:ff:ff
    inet 192.168.1.100/24 brd 192.168.1.255 scope global eth0";
        let ifaces = parse_ip_addr(output);
        assert_eq!(ifaces.len(), 2);
        assert_eq!(ifaces[1].name, "eth0");
        assert_eq!(ifaces[1].ipv4, Some("192.168.1.100".to_string()));
        assert_eq!(ifaces[1].state, "UP");
    }
}
