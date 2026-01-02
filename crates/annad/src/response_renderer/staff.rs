//! Staff member registry and lookup.
//!
//! This module defines the staff members and their domain assignments.
//! v0.0.406: Aligned with roster (anna_shared::roster) names for consistency.

use super::types::StaffMember;

/// The staff registry - one specialist per domain
/// Domain → Team mapping: system→Performance, boot/services/packages→Services,
/// network→Network, storage→Storage, audio/display→Hardware, desktop→Desktop, security→Security
pub const STAFF: &[(&str, StaffMember)] = &[
    (
        "system",
        StaffMember {
            name: "Kari",
            title: "Performance Analyst",
        },
    ),
    (
        "boot",
        StaffMember {
            name: "Hugo",
            title: "Services Administrator",
        },
    ),
    (
        "services",
        StaffMember {
            name: "Hugo",
            title: "Services Administrator",
        },
    ),
    (
        "network",
        StaffMember {
            name: "Michael",
            title: "Network Engineer",
        },
    ),
    (
        "storage",
        StaffMember {
            name: "Lars",
            title: "Storage Engineer",
        },
    ),
    (
        "packages",
        StaffMember {
            name: "Hugo",
            title: "Services Administrator",
        },
    ),
    (
        "audio",
        StaffMember {
            name: "Nora",
            title: "Hardware Technician",
        },
    ),
    (
        "display",
        StaffMember {
            name: "Nora",
            title: "Hardware Technician",
        },
    ),
    (
        "desktop",
        StaffMember {
            name: "Sofia",
            title: "Desktop Administrator",
        },
    ),
    (
        "security",
        StaffMember {
            name: "Priya",
            title: "Security Analyst",
        },
    ),
];

/// Get staff member by domain
pub fn get_staff(domain: &str) -> &'static StaffMember {
    STAFF
        .iter()
        .find(|(d, _)| *d == domain)
        .map(|(_, s)| s)
        .unwrap_or(&StaffMember {
            name: "Anna",
            title: "System Assistant",
        })
}
