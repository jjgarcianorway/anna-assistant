//! Department Ownership Rules (Part F) - v0.0.439.
//!
//! Stop cross-department nonsense. Each department owns specific domains:
//! - Storage: filesystems, mounts, SMART, btrfs, du/df/lsblk
//! - Hardware: CPU/GPU sensors, kernel modules, PCI devices, drivers
//! - Services: systemd services, journald, pacman locks, timers
//! - Performance: CPU/mem top consumers, iowait, boot performance
//! - Network: WiFi disconnects, DNS, routing, DHCP
//! - Desktop: editors, shells, dotfiles, DE/WM configs
//! - Security: firewall, permissions, vuln checks
//!
//! If translator outputs a conflicting department, we override and log.

pub mod conflict;
pub mod ownership;
pub mod router;
pub mod rules;

// Re-exports for convenience
pub use conflict::DepartmentConflict;
pub use ownership::DepartmentOwnership;
pub use router::{DeterministicRouter, RouteResult};
pub use rules::DepartmentRules;
