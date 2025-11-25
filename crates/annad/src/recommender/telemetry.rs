//! Telemetry recommendations

use anna_common::{Advice, Alternative, Priority, RiskLevel, SystemFacts};
use std::path::Path;
use std::process::Command;

pub(crate) fn check_systemd_health() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for failed units
    let failed_output = Command::new("systemctl")
        .args(&["--failed", "--no-pager", "--no-legend"])
        .output();

    if let Ok(output) = failed_output {
        let failed = String::from_utf8_lossy(&output.stdout);
        let failed_count = failed.lines().filter(|l| !l.is_empty()).count();

        if failed_count > 0 {
            let msg = if failed_count == 1 {
                "1 system service has failed".to_string()
            } else {
                format!("{} system services have failed", failed_count)
            };
            let count_str = if failed_count == 1 {
                "a".to_string()
            } else {
                failed_count.to_string()
            };
            result.push(Advice {
                id: "systemd-failed".to_string(),
                title: msg,
                reason: format!("I found {} background {} that tried to start but failed. This could mean something isn't working properly on your system.",
                    count_str,
                    if failed_count == 1 { "service" } else { "services" }),
                action: "Take a look at what failed so you can fix any problems".to_string(),
                command: Some("systemctl --failed".to_string()),
                risk: RiskLevel::Medium,
                priority: Priority::Recommended,
                category: "System Maintenance".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Systemd#Analyzing_the_system_state".to_string()],
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

pub(crate) fn check_journal_errors(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    let errors = facts.system_health_metrics.journal_errors_last_24h;

    if errors > 100 {
        // Get actual error samples to show user
        let error_samples = Command::new("journalctl")
            .args(&[
                "-p",
                "err",
                "--since",
                "24 hours ago",
                "--no-pager",
                "-n",
                "5",
            ])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let sample_preview = if !error_samples.is_empty() {
            let lines: Vec<&str> = error_samples.lines().take(3).collect();
            format!("\n\n📋 Recent error samples:\n{}", lines.join("\n"))
        } else {
            String::new()
        };

        result.push(Advice {
            id: "journal-errors-excessive".to_string(),
            title: format!("EXCESSIVE system errors detected ({} in 24h)", errors),
            reason: format!("Your system logged {} errors in the last 24 hours! This is abnormal and indicates serious problems. Normal systems have very few errors. Check journalctl to identify failing services or hardware issues.{}", errors, sample_preview),
            action: "View full error log to diagnose issues".to_string(),
            command: Some("journalctl -p err --since '24 hours ago' --no-pager | less".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Mandatory,
            category: "System Maintenance".to_string(),
            alternatives: Vec::new(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Systemd/Journal".to_string()],
            depends_on: Vec::new(),
            related_to: Vec::new(),
            bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
        });
    } else if errors > 20 {
        // Get actual error samples to show user
        let error_samples = Command::new("journalctl")
            .args(&[
                "-p",
                "err",
                "--since",
                "24 hours ago",
                "--no-pager",
                "-n",
                "3",
            ])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .unwrap_or_default();

        let sample_preview = if !error_samples.is_empty() {
            let lines: Vec<&str> = error_samples.lines().take(3).collect();
            format!("\n\n📋 Recent error samples:\n{}", lines.join("\n"))
        } else {
            String::new()
        };

        result.push(Advice {
            id: "journal-errors-many".to_string(),
            title: format!("Multiple system errors detected ({} in 24h)", errors),
            reason: format!("Your system has {} errors in the last 24 hours. While not critical, this is worth investigating to prevent future problems.{}\n\n💡 Tip: After viewing, these errors will remain until you fix the underlying issue. Anna will stop warning once errors drop below 20/day.", errors, sample_preview),
            action: "View error log to identify issues".to_string(),
            command: Some("journalctl -p err --since '24 hours ago' --no-pager | less".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "System Maintenance".to_string(),
            alternatives: Vec::new(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Systemd/Journal".to_string()],
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

pub(crate) fn check_degraded_services(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    if !facts.system_health_metrics.degraded_services.is_empty() {
        // Format service list nicely
        let services_list = facts
            .system_health_metrics
            .degraded_services
            .iter()
            .map(|s| format!("  • {}", s))
            .collect::<Vec<_>>()
            .join("\n");

        let reason = format!(
            "{} service{} in degraded/failed state:\n\n{}\n\n{}",
            facts.system_health_metrics.degraded_services.len(),
            if facts.system_health_metrics.degraded_services.len() == 1 { "" } else { "s" },
            services_list,
            "Degraded services may not function properly. Check logs to identify configuration issues, missing dependencies, or permission problems."
        );

        result.push(Advice {
            id: "degraded-services".to_string(),
            title: format!(
                "{} Service{} in Degraded State",
                facts.system_health_metrics.degraded_services.len(),
                if facts.system_health_metrics.degraded_services.len() == 1 {
                    ""
                } else {
                    "s"
                }
            ),
            reason,
            action: "Check status with: systemctl status <service-name> for each failed service"
                .to_string(),
            command: Some("systemctl --failed --no-pager".to_string()), // Show all failed services
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "System Configuration".to_string(),
            alternatives: Vec::new(),
            wiki_refs: vec![
                "https://wiki.archlinux.org/title/Systemd#Basic_systemctl_usage".to_string(),
                "https://wiki.archlinux.org/title/Systemd#Investigating_failed_services"
                    .to_string(),
            ],
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

pub(crate) fn check_memory_pressure(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    match facts.predictive_insights.memory_pressure_risk {
        RiskLevel::High => {
            let available_gb = facts.hardware_monitoring.memory_available_gb;
            result.push(Advice {
                id: "memory-pressure-critical".to_string(),
                title: "CRITICAL: Very low memory available!".to_string(),
                reason: format!("Only {:.1}GB of RAM available! Your system is under severe memory pressure. This causes swap thrashing, slow performance, and potential OOM kills. Close unnecessary programs or add more RAM.", available_gb),
                action: "Close memory-heavy applications or add more RAM".to_string(),
                command: Some("ps aux --sort=-%mem | head -15".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Mandatory,
                category: "Performance & Optimization".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Improving_performance#Memory".to_string()],
                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
        RiskLevel::Medium => {
            let available_gb = facts.hardware_monitoring.memory_available_gb;
            result.push(Advice {
                id: "memory-pressure-moderate".to_string(),
                title: "Low memory available".to_string(),
                reason: format!("Only {:.1}GB of RAM available. Your system may start swapping soon, which degrades performance. Consider closing some applications.", available_gb),
                action: "Monitor memory usage and close unnecessary applications".to_string(),
                command: Some("ps aux --sort=-%mem | head -10".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Performance & Optimization".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Improving_performance#Memory".to_string()],
                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
        _ => {}
    }

    // Check OOM events
    if facts.system_health_metrics.oom_events_last_week > 0 {
        result.push(Advice {
            id: "oom-events-detected".to_string(),
            title: format!("{} Out-of-Memory kills in the last week!", facts.system_health_metrics.oom_events_last_week),
            reason: format!("The kernel killed {} processes due to memory exhaustion! This means you're running out of RAM regularly. Add more RAM, reduce workload, or enable zram/swap.", facts.system_health_metrics.oom_events_last_week),
            action: "Add more RAM or enable swap/zram compression".to_string(),
            command: Some("journalctl -k --since '7 days ago' | grep -i 'out of memory'".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Mandatory,
            category: "Performance & Optimization".to_string(),
            alternatives: Vec::new(),
            wiki_refs: vec!["https://wiki.archlinux.org/title/Improving_performance#Memory".to_string()],
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

pub(crate) fn check_service_crashes(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    let crashes = facts.system_health_metrics.recent_crashes.len();

    if crashes > 5 {
        // List actual services that crashed
        let crash_list = facts
            .system_health_metrics
            .recent_crashes
            .iter()
            .take(15) // Show up to 15 crashes
            .map(|crash| {
                let exit_info = if let Some(code) = crash.exit_code {
                    format!("(exit code: {})", code)
                } else if let Some(ref sig) = crash.signal {
                    format!("(signal: {})", sig)
                } else {
                    String::new()
                };
                format!(
                    "  • {} {} - {}",
                    crash.service_name,
                    exit_info,
                    crash.timestamp.format("%Y-%m-%d %H:%M")
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let reason = format!(
            "{} service crash{} detected in the last week:\n\n{}\n\n{}",
            crashes,
            if crashes == 1 { "" } else { "es" },
            crash_list,
            "Multiple service crashes indicate system instability. Common causes: misconfiguration, resource exhaustion, software bugs, or failing hardware."
        );

        result.push(Advice {
            id: "service-crashes-many".to_string(),
            title: format!("{} Service Crash{} in Last Week", crashes, if crashes == 1 { "" } else { "es" }),
            reason,
            action: "Review crash details with: systemctl status <service-name> and journalctl -u <service-name>".to_string(),
            command: Some("journalctl -p err -b --no-pager | grep -E 'Failed|crashed' | tail -10".to_string()), // Show recent service failures
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "System Configuration".to_string(),
            alternatives: Vec::new(),
            wiki_refs: vec![
                "https://wiki.archlinux.org/title/Systemd#Journal".to_string(),
                "https://wiki.archlinux.org/title/Systemd#Analyzing_the_system_state".to_string(),
            ],
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

pub(crate) fn check_kernel_errors(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    if !facts.system_health_metrics.kernel_errors.is_empty() {
        // Get actual kernel errors for display
        let error_list = facts
            .system_health_metrics
            .kernel_errors
            .iter()
            .take(10) // Show up to 10 most recent errors
            .map(|err| format!("  • {}", err))
            .collect::<Vec<_>>()
            .join("\n");

        let reason = format!(
            "{} kernel error{} found in the last 24 hours:\n\n{}\n\n{}",
            facts.system_health_metrics.kernel_errors.len(),
            if facts.system_health_metrics.kernel_errors.len() == 1 { "" } else { "s" },
            error_list,
            "Kernel errors can indicate hardware problems (RAM, disk, CPU), driver issues, or firmware bugs. Common causes: failing hardware, incompatible kernel modules, ACPI issues."
        );

        result.push(Advice {
            id: "kernel-errors-detected".to_string(),
            title: format!(
                "{} Kernel Error{} Detected",
                facts.system_health_metrics.kernel_errors.len(),
                if facts.system_health_metrics.kernel_errors.len() == 1 {
                    ""
                } else {
                    "s"
                }
            ),
            reason,
            action: "Review full kernel log with: journalctl -k -p err --since '24 hours ago'"
                .to_string(),
            command: Some("journalctl -k -p err -b --no-pager | tail -10".to_string()), // Show recent kernel errors
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "System Configuration".to_string(),
            alternatives: Vec::new(),
            wiki_refs: vec![
                "https://wiki.archlinux.org/title/Kernel_parameters".to_string(),
                "https://wiki.archlinux.org/title/Dmesg".to_string(),
            ],
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

pub(crate) fn check_network_health(facts: &SystemFacts) -> Vec<Advice> {
    use std::process::Command;
    let mut advice = Vec::new();

    // Check if we have any network interfaces up
    let has_network = Command::new("ip")
        .args(&["link", "show", "up"])
        .output()
        .map(|o| {
            let output = String::from_utf8_lossy(&o.stdout);
            output.lines().any(|l| l.contains("state UP"))
        })
        .unwrap_or(false);

    if !has_network {
        advice.push(
            Advice::new(
                "network-no-interfaces".to_string(),
                "No network interfaces are up".to_string(),
                "Anna detected that no network interfaces are currently active. This means you have no internet connectivity. Check your network cable, WiFi connection, or restart NetworkManager.".to_string(),
                "systemctl status NetworkManager".to_string(),
                None,
                RiskLevel::High,
                Priority::Mandatory,
                vec![
                    "https://wiki.archlinux.org/title/Network_configuration".to_string(),
                ],
                "Network Configuration".to_string(),
            )
        );
        return advice;
    }

    // Test internet connectivity with ping
    let can_ping_dns = Command::new("ping")
        .args(&["-c", "2", "-W", "3", "1.1.1.1"]) // Cloudflare DNS
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !can_ping_dns {
        advice.push(
            Advice::new(
                "network-no-connectivity".to_string(),
                "No internet connectivity detected".to_string(),
                "Network interfaces are up but Anna cannot reach the internet. This could be a DNS issue, router problem, or ISP outage. Try restarting your router or checking your network configuration.".to_string(),
                "ping -c 4 1.1.1.1 && ping -c 4 google.com".to_string(),
                None,
                RiskLevel::Medium,
                Priority::Recommended,
                vec![
                    "https://wiki.archlinux.org/title/Network_configuration".to_string(),
                ],
                "Network Configuration".to_string(),
            )
        );
    }

    // Check DNS resolution
    let can_resolve_dns = Command::new("nslookup")
        .args(&["archlinux.org"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if can_ping_dns && !can_resolve_dns {
        advice.push(
            Advice::new(
                "network-dns-broken".to_string(),
                "DNS resolution is not working".to_string(),
                "You can reach IP addresses but DNS name resolution is broken. This prevents browsing websites by name. Check your /etc/resolv.conf or NetworkManager DNS settings.".to_string(),
                "cat /etc/resolv.conf && systemctl status systemd-resolved".to_string(),
                None,
                RiskLevel::Medium,
                Priority::Recommended,
                vec![
                    "https://wiki.archlinux.org/title/Domain_name_resolution".to_string(),
                ],
                "Network Configuration".to_string(),
            )
        );
    }

    // Check for packet loss (connection quality)
    if can_ping_dns {
        if let Ok(output) = Command::new("ping")
            .args(&["-c", "10", "-W", "2", "1.1.1.1"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);

            // Parse packet loss percentage
            if let Some(loss_line) = output_str.lines().find(|l| l.contains("packet loss")) {
                if let Some(percent_str) = loss_line.split_whitespace().find(|s| s.contains('%')) {
                    if let Ok(loss) = percent_str.trim_end_matches('%').parse::<f32>() {
                        if loss > 20.0 {
                            advice.push(
                                Advice::new(
                                    "network-high-packet-loss".to_string(),
                                    format!("High packet loss detected ({:.0}%)", loss),
                                    format!(
                                        "Your network connection is unstable with {:.0}% packet loss. This causes slow or unreliable internet. Possible causes: weak WiFi signal, bad ethernet cable, router issues, or ISP problems. Try moving closer to WiFi router, checking cables, or restarting your router.",
                                        loss
                                    ),
                                    "ping -c 20 1.1.1.1".to_string(),
                                    None,
                                    RiskLevel::Medium,
                                    Priority::Recommended,
                                    vec![
                                        "https://wiki.archlinux.org/title/Network_configuration".to_string(),
                                    ],
                                    "Network Configuration".to_string(),
                                )
                            );
                        } else if loss > 5.0 {
                            advice.push(
                                Advice::new(
                                    "network-moderate-packet-loss".to_string(),
                                    format!("Moderate packet loss detected ({:.0}%)", loss),
                                    format!(
                                        "Your network has {:.0}% packet loss. While not critical, this can cause occasional slowdowns. Consider checking your WiFi signal strength or ethernet cable connection.",
                                        loss
                                    ),
                                    "ping -c 20 1.1.1.1".to_string(),
                                    None,
                                    RiskLevel::Low,
                                    Priority::Optional,
                                    vec![
                                        "https://wiki.archlinux.org/title/Network_configuration".to_string(),
                                    ],
                                    "Network Configuration".to_string(),
                                )
                            );
                        }
                    }
                }
            }

            // Check latency (ping time)
            if let Some(rtt_line) = output_str.lines().find(|l| l.contains("rtt min/avg/max")) {
                // Example: "rtt min/avg/max/mdev = 10.123/25.456/50.789/10.123 ms"
                if let Some(stats) = rtt_line.split('=').nth(1) {
                    let parts: Vec<&str> = stats.trim().split('/').collect();
                    if parts.len() >= 3 {
                        if let Ok(avg_ms) = parts[1].parse::<f32>() {
                            if avg_ms > 200.0 {
                                advice.push(
                                    Advice::new(
                                        "network-high-latency".to_string(),
                                        format!("High network latency ({:.0}ms)", avg_ms),
                                        format!(
                                            "Your average ping time is {:.0}ms, which is quite high. This causes noticeable delays in browsing and online activities. Possible causes: slow internet connection, WiFi interference, or distance from server.",
                                            avg_ms
                                        ),
                                        "ping -c 10 1.1.1.1".to_string(),
                                        None,
                                        RiskLevel::Low,
                                        Priority::Cosmetic,
                                        vec![
                                            "https://wiki.archlinux.org/title/Network_configuration".to_string(),
                                        ],
                                        "Network Configuration".to_string(),
                                    )
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    // Check if NetworkManager is running
    let nm_running = Command::new("systemctl")
        .args(&["is-active", "NetworkManager"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !nm_running {
        advice.push(
            Advice::new(
                "network-manager-not-running".to_string(),
                "NetworkManager is not running".to_string(),
                "NetworkManager handles network connections and WiFi. It's not currently running, which may cause connection issues. Start it with 'systemctl start NetworkManager' and enable it to start on boot.".to_string(),
                "systemctl start NetworkManager && systemctl enable NetworkManager".to_string(),
                None,
                RiskLevel::Medium,
                Priority::Recommended,
                vec![
                    "https://wiki.archlinux.org/title/NetworkManager".to_string(),
                ],
                "Network Configuration".to_string(),
            )
        );
    }

    advice
}

