//! Prompt building functions for the translator.

use super::learning::get_probe_recommendations;
use super::types::TranslatorInput;

/// Build the translator system prompt - comprehensive domain and probe mapping
/// v0.0.405: Complete rewrite with all 10 domains and strict JSON output
pub(crate) fn build_translator_prompt() -> String {
    r#"You are Anna's query classifier. Output ONLY valid JSON.

OUTPUT FORMAT:
{"intent":"query_metric|diagnose|configure|list|check_status|explain","domain":"system|boot|services|network|storage|packages|audio|display|desktop|security","entities":[],"needs_probes":[],"clarification_question":null,"confidence":0.0-1.0}

DOMAIN CLASSIFICATION (pick ONE):
- system: CPU, RAM, memory, processes, load, temperature, sensors, general health
- boot: startup time, boot errors, systemd-analyze, slow boot
- services: systemd units, running/failed services, daemons, timers
- network: IP, DNS, wifi, ethernet, ports, connections, ping, gateway
- storage: disk space, partitions, mounts, drives, filesystems, "taking space"
- packages: install, update, pacman, apt, dnf, pip, package count
- audio: sound, speakers, headphones, PulseAudio, PipeWire, volume
- display: monitors, resolution, GPU, graphics drivers, xrandr, Wayland
- desktop: window manager, DE config, Hyprland, GNOME, KDE, sessions
- security: firewall, ssh, logins, permissions, iptables, users

PROBE MAPPINGS BY DOMAIN:

SYSTEM domain:
- "memory", "RAM", "swap" → ["memory_info"]
- "CPU info", "cores" → ["cpu_info"]
- "CPU usage", "load" → ["cpu_info","load_average","top_cpu"]
- "temperature", "sensors" → ["sensors_temp"]
- "processes" → ["top_cpu","top_memory"]
- "health check" → ["memory_info","disk_usage","failed_services","load_average"]

BOOT domain:
- "boot time", "startup" → ["boot_time","boot_blame"]
- "slow boot" → ["boot_time","boot_blame","failed_services"]
- "boot errors" → ["journal_errors","boot_time"]

SERVICES domain:
- "services", "systemd" → ["running_services","failed_services"]
- "failed services" → ["failed_services"]
- "timers" → ["systemd_timers"]

STORAGE domain:
- "disk space" → ["disk_usage"]
- "what's taking space" → ["disk_usage","largest_dirs","largest_home"]
- "partitions", "drives" → ["disk_usage","block_devices","findmnt"]

NETWORK domain:
- "IP address" → ["network_addrs"]
- "DNS" → ["dns_servers"]
- "wifi" → ["network_addrs","wireless_networks"]
- "ports" → ["listening_ports"]
- "internet check" → ["network_addrs","ping_check"]

PACKAGES domain:
- "updates available" → ["package_updates"]
- "installed packages" → ["installed_packages","package_count"]

AUDIO domain:
- "sound", "speakers" → ["audio_devices","audio_server"]
- "no sound" → ["audio_devices","audio_server","pactl_cards"]

DISPLAY domain:
- "GPU", "graphics" → ["gpu_info","gpu_drivers"]
- "monitors", "resolution" → ["display_info"]
- "wayland", "xorg" → ["display_server"]
- "nvidia", "amd driver" → ["gpu_drivers","kernel_modules"]

DESKTOP domain:
- "desktop environment" → ["desktop_session","installed_desktops"]
- "hyprland", "gnome", "kde" → ["desktop_session"]
- "window manager config" → ["desktop_session"]

SECURITY domain:
- "firewall" → ["firewall_status","iptables_rules"]
- "ssh", "logins" → ["ssh_connections","last_logins","failed_logins"]

RULES:
1. Output ONLY valid JSON, no explanation
2. Select 1-4 probes that DIRECTLY answer the query
3. Match domain to query topic (not everything is "system")
4. clarification_question should be null unless truly ambiguous"#
        .to_string()
}

/// Build minimal translator request (< 2KB)
pub fn build_translator_request(input: &TranslatorInput) -> String {
    let prompt = build_translator_prompt();

    // v0.0.322: Add learned probe recommendations if available
    let recommendations = get_probe_recommendations(&input.query);

    if recommendations.is_empty() {
        format!(
            "{}\nHW: {}\nQuery: {}",
            prompt, input.hw_summary, input.query
        )
    } else {
        format!(
            "{}\nHW: {}\nLearned: For this type of query, effective probes have been: {}\nQuery: {}",
            prompt, input.hw_summary, recommendations, input.query
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::translator::types::MAX_TRANSLATOR_PAYLOAD_SIZE;

    #[test]
    fn test_translator_payload_size() {
        let input = TranslatorInput::new("what processes are using the most memory", 8, 16.0, true);
        let payload = build_translator_request(&input);
        // v0.0.402: Expanded prompt with comprehensive probe mappings is ~4KB
        assert!(payload.len() < MAX_TRANSLATOR_PAYLOAD_SIZE); // 8KB max
        assert!(payload.len() < 6000); // Should be under 6KB
    }
}
