//! Network Diagnostics Engine (Beta.265)
//!
//! Proactive network diagnostics for detecting:
//! - Multi-interface collisions (WiFi vs Ethernet priority conflicts)
//! - Connectivity degradation (packet loss, DNS failures, latency spikes)
//! - Misconfigured routing tables (duplicate default routes, missing fallbacks)
//! - Interface ranking mismatches (slow Ethernet taking priority over fast WiFi)
//!
//! **Design Principle**: 100% deterministic, zero LLM involvement.
//! All detection logic uses hard thresholds and pattern matching.

use anna_common::network_monitoring::{
    InterfaceType, NetworkInterface, NetworkMonitoring, Route,
};
use serde::{Deserialize, Serialize};

/// Network diagnostic results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDiagnostics {
    /// Multi-interface collision findings
    pub interface_collision: Option<InterfaceCollision>,
    /// Connectivity degradation findings
    pub connectivity_degradation: Option<ConnectivityDegradation>,
    /// Routing table misconfigurations
    pub routing_issues: Vec<RoutingIssue>,
    /// Interface ranking vs actual priority mismatch
    pub priority_mismatch: Option<PriorityMismatch>,
    /// Overall network health status
    pub health_status: NetworkHealthStatus,
}

/// Multi-interface collision detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceCollision {
    /// Interfaces involved in collision
    pub interfaces: Vec<String>,
    /// Type of collision
    pub collision_type: CollisionType,
    /// Severity (warning, critical)
    pub severity: DiagnosticSeverity,
    /// Human-readable description
    pub description: String,
    /// Metrics supporting the finding
    pub metrics: CollisionMetrics,
}

/// Types of interface collisions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CollisionType {
    /// Ethernet slower than WiFi but taking priority
    EthernetSlowerThanWiFi,
    /// Multiple interfaces with similar priority
    MultipleActiveInterfaces,
    /// Multiple default routes
    DuplicateDefaultRoutes,
}

/// Metrics supporting collision finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollisionMetrics {
    /// Ethernet link speed (Mbps)
    pub ethernet_speed_mbps: Option<u32>,
    /// WiFi link speed (Mbps)
    pub wifi_speed_mbps: Option<u32>,
    /// Ethernet RX/TX error rate
    pub ethernet_error_rate: f64,
    /// WiFi RX/TX error rate
    pub wifi_error_rate: f64,
    /// Which interface has default route
    pub default_route_interface: Option<String>,
}

/// Connectivity degradation detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectivityDegradation {
    /// Type of degradation
    pub degradation_type: DegradationType,
    /// Severity
    pub severity: DiagnosticSeverity,
    /// Description
    pub description: String,
    /// Supporting metrics
    pub metrics: DegradationMetrics,
}

/// Types of connectivity degradation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DegradationType {
    /// High packet loss
    HighPacketLoss,
    /// DNS lookup failures
    DnsFailures,
    /// High latency spikes
    HighLatency,
    /// Interface state flapping
    InterfaceFlapping,
}

/// Metrics for connectivity degradation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegradationMetrics {
    /// Packet loss percentage
    pub packet_loss_percent: Option<f64>,
    /// Gateway latency in ms
    pub gateway_latency_ms: Option<f64>,
    /// Internet latency in ms
    pub internet_latency_ms: Option<f64>,
    /// Interface error count
    pub interface_errors: u64,
}

/// Routing table issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingIssue {
    /// Type of routing issue
    pub issue_type: RoutingIssueType,
    /// Severity
    pub severity: DiagnosticSeverity,
    /// Description
    pub description: String,
    /// Affected routes
    pub affected_routes: Vec<String>,
}

/// Types of routing issues
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RoutingIssueType {
    /// Multiple default routes with same metric
    DuplicateDefaultRoute,
    /// Missing IPv6 default route when IPv4 present
    MissingIpv6Route,
    /// Missing IPv4 default route when IPv6 present
    MissingIpv4Route,
    /// High metric on default route
    HighMetricDefaultRoute,
}

/// Interface priority mismatch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityMismatch {
    /// Expected interface (based on ranking)
    pub expected_interface: String,
    /// Actual interface (from routing table)
    pub actual_interface: String,
    /// Ranking score for expected
    pub expected_rank: u32,
    /// Ranking score for actual
    pub actual_rank: u32,
    /// Description
    pub description: String,
}

/// Overall network health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NetworkHealthStatus {
    /// No issues detected
    Healthy,
    /// Minor issues, degraded performance
    Degraded,
    /// Major issues, connectivity problems
    Critical,
}

/// Diagnostic severity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiagnosticSeverity {
    Warning,
    Critical,
}

impl NetworkDiagnostics {
    /// Run full network diagnostics
    pub fn analyze(net_mon: &NetworkMonitoring) -> Self {
        let interface_collision = detect_interface_collision(&net_mon.interfaces, &net_mon.routes);
        let connectivity_degradation = detect_connectivity_degradation(net_mon);
        let routing_issues = detect_routing_issues(&net_mon.routes, net_mon);
        let priority_mismatch = detect_priority_mismatch(&net_mon.interfaces, &net_mon.routes);

        // Determine overall health status
        let health_status = if interface_collision.as_ref().map(|c| &c.severity) == Some(&DiagnosticSeverity::Critical)
            || connectivity_degradation.as_ref().map(|d| &d.severity) == Some(&DiagnosticSeverity::Critical)
            || routing_issues.iter().any(|r| r.severity == DiagnosticSeverity::Critical)
        {
            NetworkHealthStatus::Critical
        } else if interface_collision.is_some() || connectivity_degradation.is_some() || !routing_issues.is_empty() || priority_mismatch.is_some() {
            NetworkHealthStatus::Degraded
        } else {
            NetworkHealthStatus::Healthy
        };

        NetworkDiagnostics {
            interface_collision,
            connectivity_degradation,
            routing_issues,
            priority_mismatch,
            health_status,
        }
    }
}

/// Detect multi-interface collisions
fn detect_interface_collision(
    interfaces: &[NetworkInterface],
    routes: &[Route],
) -> Option<InterfaceCollision> {
    // Find active Ethernet and WiFi interfaces
    let ethernet: Vec<_> = interfaces
        .iter()
        .filter(|i| i.interface_type == InterfaceType::Ethernet && i.is_up)
        .collect();

    let wifi: Vec<_> = interfaces
        .iter()
        .filter(|i| i.interface_type == InterfaceType::WiFi && i.is_up)
        .collect();

    // No collision if only one type is active
    if ethernet.is_empty() || wifi.is_empty() {
        return None;
    }

    // Get default route interface
    let default_route_interface = routes
        .iter()
        .find(|r| r.destination == "default" || r.destination == "0.0.0.0/0")
        .map(|r| r.interface.clone());

    // Check for multiple default routes
    let default_routes_count = routes
        .iter()
        .filter(|r| r.destination == "default" || r.destination == "0.0.0.0/0")
        .count();

    if default_routes_count > 1 {
        return Some(InterfaceCollision {
            interfaces: vec![
                ethernet.get(0).map(|e| e.name.clone()).unwrap_or_default(),
                wifi.get(0).map(|w| w.name.clone()).unwrap_or_default(),
            ],
            collision_type: CollisionType::DuplicateDefaultRoutes,
            severity: DiagnosticSeverity::Critical,
            description: format!("Multiple default routes detected ({} routes) - this can cause unpredictable routing behavior", default_routes_count),
            metrics: CollisionMetrics {
                ethernet_speed_mbps: ethernet.get(0).and_then(|e| e.speed_mbps),
                wifi_speed_mbps: wifi.get(0).and_then(|w| w.speed_mbps),
                ethernet_error_rate: calculate_error_rate(ethernet.get(0)),
                wifi_error_rate: calculate_error_rate(wifi.get(0)),
                default_route_interface: default_route_interface.clone(),
            },
        });
    }

    // Get speeds
    let eth_speed = ethernet.get(0).and_then(|e| e.speed_mbps).unwrap_or(0);
    let wifi_speed = wifi.get(0).and_then(|w| w.speed_mbps).unwrap_or(0);

    // Check if Ethernet is slower than WiFi AND has the default route
    if eth_speed > 0 && wifi_speed > 0 && eth_speed < wifi_speed {
        if let Some(ref default_iface) = default_route_interface {
            if ethernet.iter().any(|e| &e.name == default_iface) {
                // Ethernet slower but has priority
                return Some(InterfaceCollision {
                    interfaces: vec![
                        ethernet.get(0).map(|e| e.name.clone()).unwrap_or_default(),
                        wifi.get(0).map(|w| w.name.clone()).unwrap_or_default(),
                    ],
                    collision_type: CollisionType::EthernetSlowerThanWiFi,
                    severity: DiagnosticSeverity::Critical,
                    description: format!(
                        "Ethernet ({} Mbps) is slower than WiFi ({} Mbps) but is taking default route priority",
                        eth_speed, wifi_speed
                    ),
                    metrics: CollisionMetrics {
                        ethernet_speed_mbps: Some(eth_speed),
                        wifi_speed_mbps: Some(wifi_speed),
                        ethernet_error_rate: calculate_error_rate(ethernet.get(0)),
                        wifi_error_rate: calculate_error_rate(wifi.get(0)),
                        default_route_interface,
                    },
                });
            }
        }
    }

    // Both active but no clear priority issue
    if ethernet.len() >= 1 && wifi.len() >= 1 {
        return Some(InterfaceCollision {
            interfaces: vec![
                ethernet.get(0).map(|e| e.name.clone()).unwrap_or_default(),
                wifi.get(0).map(|w| w.name.clone()).unwrap_or_default(),
            ],
            collision_type: CollisionType::MultipleActiveInterfaces,
            severity: DiagnosticSeverity::Warning,
            description: "Both Ethernet and WiFi are active - consider disabling one for predictable routing".to_string(),
            metrics: CollisionMetrics {
                ethernet_speed_mbps: Some(eth_speed),
                wifi_speed_mbps: Some(wifi_speed),
                ethernet_error_rate: calculate_error_rate(ethernet.get(0)),
                wifi_error_rate: calculate_error_rate(wifi.get(0)),
                default_route_interface,
            },
        });
    }

    None
}

/// Calculate error rate for an interface
fn calculate_error_rate(interface: Option<&&NetworkInterface>) -> f64 {
    if let Some(iface) = interface {
        let total_packets = iface.stats.rx_packets + iface.stats.tx_packets;
        if total_packets > 0 {
            let total_errors = iface.stats.rx_errors + iface.stats.tx_errors;
            (total_errors as f64 / total_packets as f64) * 100.0
        } else {
            0.0
        }
    } else {
        0.0
    }
}

/// Detect connectivity degradation
fn detect_connectivity_degradation(net_mon: &NetworkMonitoring) -> Option<ConnectivityDegradation> {
    // Check packet loss
    if let Some(loss) = net_mon.packet_loss.internet_loss_percent {
        if loss > 10.0 {
            return Some(ConnectivityDegradation {
                degradation_type: DegradationType::HighPacketLoss,
                severity: if loss > 30.0 {
                    DiagnosticSeverity::Critical
                } else {
                    DiagnosticSeverity::Warning
                },
                description: format!("High packet loss detected: {:.1}% to internet", loss),
                metrics: DegradationMetrics {
                    packet_loss_percent: Some(loss),
                    gateway_latency_ms: net_mon.latency.gateway_latency_ms,
                    internet_latency_ms: net_mon.latency.internet_latency_ms,
                    interface_errors: 0,
                },
            });
        }
    }

    // Check latency
    if let Some(latency) = net_mon.latency.internet_latency_ms {
        if latency > 200.0 {
            return Some(ConnectivityDegradation {
                degradation_type: DegradationType::HighLatency,
                severity: if latency > 500.0 {
                    DiagnosticSeverity::Critical
                } else {
                    DiagnosticSeverity::Warning
                },
                description: format!("High network latency detected: {:.1}ms to internet", latency),
                metrics: DegradationMetrics {
                    packet_loss_percent: net_mon.packet_loss.internet_loss_percent,
                    gateway_latency_ms: net_mon.latency.gateway_latency_ms,
                    internet_latency_ms: Some(latency),
                    interface_errors: 0,
                },
            });
        }
    }

    // Check for high error rates on active interfaces
    for interface in &net_mon.interfaces {
        if interface.is_up {
            let error_rate = calculate_error_rate(Some(&interface));
            if error_rate > 1.0 {
                let total_errors = interface.stats.rx_errors + interface.stats.tx_errors;
                return Some(ConnectivityDegradation {
                    degradation_type: DegradationType::InterfaceFlapping,
                    severity: if error_rate > 5.0 {
                        DiagnosticSeverity::Critical
                    } else {
                        DiagnosticSeverity::Warning
                    },
                    description: format!(
                        "Interface {} has high error rate: {:.2}% ({} errors)",
                        interface.name, error_rate, total_errors
                    ),
                    metrics: DegradationMetrics {
                        packet_loss_percent: None,
                        gateway_latency_ms: None,
                        internet_latency_ms: None,
                        interface_errors: total_errors,
                    },
                });
            }
        }
    }

    None
}

/// Detect routing table issues
fn detect_routing_issues(routes: &[Route], net_mon: &NetworkMonitoring) -> Vec<RoutingIssue> {
    let mut issues = Vec::new();

    // Check for duplicate default routes with same metric
    let default_routes: Vec<_> = routes
        .iter()
        .filter(|r| r.destination == "default" || r.destination == "0.0.0.0/0")
        .collect();

    if default_routes.len() > 1 {
        let same_metric = default_routes
            .windows(2)
            .any(|w| w[0].metric == w[1].metric && w[0].metric.is_some());

        if same_metric {
            issues.push(RoutingIssue {
                issue_type: RoutingIssueType::DuplicateDefaultRoute,
                severity: DiagnosticSeverity::Critical,
                description: "Multiple default routes with identical metrics detected".to_string(),
                affected_routes: default_routes.iter().map(|r| {
                    format!("{} via {} metric {}", r.destination, r.gateway.as_ref().map(|g| g.as_str()).unwrap_or("direct"), r.metric.unwrap_or(0))
                }).collect(),
            });
        }
    }

    // Check for IPv4/IPv6 fallback issues
    let has_ipv4_default = routes
        .iter()
        .any(|r| r.destination == "default" || r.destination == "0.0.0.0/0");
    let has_ipv6_default = routes
        .iter()
        .any(|r| r.destination == "default" || r.destination == "::/0");

    if has_ipv4_default && !has_ipv6_default && net_mon.ipv6_status.enabled {
        issues.push(RoutingIssue {
            issue_type: RoutingIssueType::MissingIpv6Route,
            severity: DiagnosticSeverity::Warning,
            description: "IPv6 is enabled but no IPv6 default route configured".to_string(),
            affected_routes: vec![],
        });
    }

    if has_ipv6_default && !has_ipv4_default && net_mon.ipv4_status.enabled {
        issues.push(RoutingIssue {
            issue_type: RoutingIssueType::MissingIpv4Route,
            severity: DiagnosticSeverity::Warning,
            description: "IPv4 is enabled but no IPv4 default route configured".to_string(),
            affected_routes: vec![],
        });
    }

    // Check for unusually high metrics on default routes
    for route in default_routes {
        if let Some(metric) = route.metric {
            if metric > 1000 {
                issues.push(RoutingIssue {
                    issue_type: RoutingIssueType::HighMetricDefaultRoute,
                    severity: DiagnosticSeverity::Warning,
                    description: format!("Default route has unusually high metric: {}", metric),
                    affected_routes: vec![format!("{} via {} metric {}", route.destination, route.gateway.as_ref().map(|g| g.as_str()).unwrap_or("direct"), metric)],
                });
            }
        }
    }

    issues
}

/// Detect interface priority mismatches
fn detect_priority_mismatch(
    interfaces: &[NetworkInterface],
    routes: &[Route],
) -> Option<PriorityMismatch> {
    // Get default route interface
    let actual_interface = routes
        .iter()
        .find(|r| r.destination == "default" || r.destination == "0.0.0.0/0")
        .map(|r| r.interface.clone())?;

    // Rank all active interfaces
    let mut ranked: Vec<_> = interfaces
        .iter()
        .filter(|i| i.is_up && !i.ipv4_addresses.is_empty())
        .map(|i| (i, rank_interface(i)))
        .collect();

    ranked.sort_by(|a, b| b.1.cmp(&a.1)); // Highest rank first

    if ranked.is_empty() {
        return None;
    }

    let expected_interface = ranked[0].0.name.clone();
    let expected_rank = ranked[0].1;

    // Find actual interface rank
    let actual_rank = ranked
        .iter()
        .find(|(i, _)| i.name == actual_interface)
        .map(|(_, rank)| *rank)
        .unwrap_or(0);

    // Mismatch if expected != actual and expected has significantly higher rank
    if expected_interface != actual_interface && expected_rank > actual_rank + 10 {
        return Some(PriorityMismatch {
            expected_interface: expected_interface.clone(),
            actual_interface: actual_interface.clone(),
            expected_rank,
            actual_rank,
            description: format!(
                "Interface {} (rank {}) should have priority over {} (rank {})",
                expected_interface, expected_rank, actual_interface, actual_rank
            ),
        });
    }

    None
}

/// Deterministic interface ranking heuristic
///
/// Higher score = higher preference
/// - Ethernet with high speed (1000+ Mbps): 100+ points
/// - WiFi with strong signal: 80+ points
/// - Ethernet with low speed (100 Mbps or less): 50 points
/// - WiFi with weak signal: 30 points
/// - Other (tethering, VPN, virtual): 10 points
fn rank_interface(interface: &NetworkInterface) -> u32 {
    let mut score = 0u32;

    // Base score by type
    match interface.interface_type {
        InterfaceType::Ethernet => score += 50,
        InterfaceType::WiFi => score += 40,
        _ => score += 10,
    }

    // Speed bonus (speed is more important than interface type)
    if let Some(speed) = interface.speed_mbps {
        if speed >= 1000 {
            score += 60; // Gigabit or better
        } else if speed >= 500 {
            score += 50; // Fast WiFi (AC/AX)
        } else if speed >= 100 {
            score += 20; // Fast Ethernet
        } else if speed >= 10 {
            score += 5; // 10 Mbps
        }
    }

    // Penalty for errors
    let error_rate = calculate_error_rate(Some(&interface));
    if error_rate > 5.0 {
        score = score.saturating_sub(30);
    } else if error_rate > 1.0 {
        score = score.saturating_sub(10);
    }

    // Bonus for having gateway (likely internet-connected)
    if !interface.ipv4_addresses.is_empty() && !interface.ipv4_addresses.iter().all(|a| a.starts_with("169.254")) {
        score += 10;
    }

    score
}

#[cfg(test)]
mod tests {
    use super::*;
    use anna_common::network_monitoring::InterfaceStats;

    fn mock_interface(name: &str, iface_type: InterfaceType, speed_mbps: Option<u32>, is_up: bool) -> NetworkInterface {
        NetworkInterface {
            name: name.to_string(),
            interface_type: iface_type,
            is_up,
            mac_address: Some("00:11:22:33:44:55".to_string()),
            ipv4_addresses: if is_up { vec!["192.168.1.100/24".to_string()] } else { vec![] },
            ipv6_addresses: vec![],
            mtu: Some(1500),
            speed_mbps,
            config_method: anna_common::network_monitoring::AddressConfig::DHCP,
            stats: InterfaceStats {
                rx_bytes: 1000000,
                rx_packets: 1000,
                rx_errors: 0,
                rx_dropped: 0,
                tx_bytes: 500000,
                tx_packets: 500,
                tx_errors: 0,
                tx_dropped: 0,
            },
        }
    }

    #[test]
    fn test_ranking_gigabit_ethernet_highest() {
        let eth = mock_interface("eth0", InterfaceType::Ethernet, Some(1000), true);
        let wifi = mock_interface("wlan0", InterfaceType::WiFi, Some(300), true);

        assert!(rank_interface(&eth) > rank_interface(&wifi));
    }

    #[test]
    fn test_ranking_fast_wifi_over_slow_ethernet() {
        let eth_slow = mock_interface("eth0", InterfaceType::Ethernet, Some(100), true);
        let wifi_fast = mock_interface("wlan0", InterfaceType::WiFi, Some(866), true);

        // WiFi 866 Mbps should rank higher than 100 Mbps Ethernet
        assert!(rank_interface(&wifi_fast) > rank_interface(&eth_slow));
    }

    #[test]
    fn test_detect_collision_slow_ethernet_priority() {
        let interfaces = vec![
            mock_interface("eth0", InterfaceType::Ethernet, Some(100), true),
            mock_interface("wlan0", InterfaceType::WiFi, Some(866), true),
        ];

        let routes = vec![Route {
            destination: "default".to_string(),
            gateway: Some("192.168.1.1".to_string()),
            interface: "eth0".to_string(),
            metric: Some(100),
            protocol: Some("dhcp".to_string()),
        }];

        let collision = detect_interface_collision(&interfaces, &routes);
        assert!(collision.is_some());
        let coll = collision.unwrap();
        assert_eq!(coll.collision_type, CollisionType::EthernetSlowerThanWiFi);
        assert_eq!(coll.severity, DiagnosticSeverity::Critical);
    }

    #[test]
    fn test_detect_duplicate_default_routes() {
        let interfaces = vec![
            mock_interface("eth0", InterfaceType::Ethernet, Some(1000), true),
            mock_interface("wlan0", InterfaceType::WiFi, Some(300), true),
        ];

        let routes = vec![
            Route {
                destination: "default".to_string(),
                gateway: Some("192.168.1.1".to_string()),
                interface: "eth0".to_string(),
                metric: Some(100),
                protocol: Some("dhcp".to_string()),
            },
            Route {
                destination: "default".to_string(),
                gateway: Some("192.168.1.1".to_string()),
                interface: "wlan0".to_string(),
                metric: Some(200),
                protocol: Some("dhcp".to_string()),
            },
        ];

        let collision = detect_interface_collision(&interfaces, &routes);
        assert!(collision.is_some());
        assert_eq!(
            collision.unwrap().collision_type,
            CollisionType::DuplicateDefaultRoutes
        );
    }
}
