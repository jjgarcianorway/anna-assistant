//! Relationship data for all staff members (v0.0.262).
//!
//! Defines mentor-mentee, friendship, and cross-team collaboration
//! relationships between staff members for richer dialogue.

use super::types::{Relationship, RelationType};

/// All relationships in the department
/// Designed based on personality quirks and logical interactions
pub(super) const RELATIONSHIPS: &[Relationship] = &[
    // === MENTOR RELATIONSHIPS (Senior -> Junior within teams) ===
    // These are implicit but we define them for dialogue variation
    Relationship {
        from_id: "network_sr",
        to_id: "network_jr",
        relation_type: RelationType::Mentor,
    },
    Relationship {
        from_id: "desktop_sr",
        to_id: "desktop_jr",
        relation_type: RelationType::Mentor,
    },
    Relationship {
        from_id: "hardware_sr",
        to_id: "hardware_jr",
        relation_type: RelationType::Mentor,
    },
    Relationship {
        from_id: "storage_sr",
        to_id: "storage_jr",
        relation_type: RelationType::Mentor,
    },
    Relationship {
        from_id: "perf_sr",
        to_id: "perf_jr",
        relation_type: RelationType::Mentor,
    },
    Relationship {
        from_id: "security_sr",
        to_id: "security_jr",
        relation_type: RelationType::Mentor,
    },
    Relationship {
        from_id: "services_sr",
        to_id: "services_jr",
        relation_type: RelationType::Mentor,
    },
    Relationship {
        from_id: "logs_sr",
        to_id: "logs_jr",
        relation_type: RelationType::Mentor,
    },
    Relationship {
        from_id: "general_sr",
        to_id: "general_jr",
        relation_type: RelationType::Mentor,
    },
    // === FRIENDSHIPS (Cross-team, based on interests) ===
    // Sofia (vim enthusiast) & Lars (documentation geek) - both love configs
    Relationship {
        from_id: "desktop_jr",
        to_id: "storage_jr",
        relation_type: RelationType::Friend,
    },
    // Michael (TCP/IP) & Hugo (systemd) - both morning shift, infrastructure talks
    Relationship {
        from_id: "network_jr",
        to_id: "services_jr",
        relation_type: RelationType::Friend,
    },
    // Nora (hardware, beep boop) & Kari (htop) - both excited about metrics
    Relationship {
        from_id: "hardware_jr",
        to_id: "perf_jr",
        relation_type: RelationType::Friend,
    },
    // Ana (calm) & Ines (calm) - both senior architects, similar temperament
    Relationship {
        from_id: "network_sr",
        to_id: "storage_sr",
        relation_type: RelationType::Friend,
    },
    // Daniel (coffee, night) & Oskar (night owl) - night shift buddies
    Relationship {
        from_id: "logs_jr",
        to_id: "security_sr",
        relation_type: RelationType::Friend,
    },
    // === RIVALRIES (Friendly competition) ===
    // Erik (X11) vs Mina (containers) - old school vs new school debate
    Relationship {
        from_id: "desktop_sr",
        to_id: "services_sr",
        relation_type: RelationType::Rival,
    },
    // Lars (ext4/btrfs) vs Ines (ZFS) - filesystem debates
    Relationship {
        from_id: "storage_jr",
        to_id: "storage_sr",
        relation_type: RelationType::Rival,
    },
    // === COLLABORATORS (Complementary skills) ===
    // Network + Security often work together
    Relationship {
        from_id: "network_jr",
        to_id: "security_jr",
        relation_type: RelationType::Collaborator,
    },
    Relationship {
        from_id: "network_sr",
        to_id: "security_sr",
        relation_type: RelationType::Collaborator,
    },
    // Services + Logs work together on troubleshooting
    Relationship {
        from_id: "services_jr",
        to_id: "logs_jr",
        relation_type: RelationType::Collaborator,
    },
    // Performance + Hardware collaborate on bottlenecks
    Relationship {
        from_id: "perf_jr",
        to_id: "hardware_jr",
        relation_type: RelationType::Collaborator,
    },
    Relationship {
        from_id: "perf_sr",
        to_id: "hardware_sr",
        relation_type: RelationType::Collaborator,
    },
    // Storage + Performance for disk I/O issues
    Relationship {
        from_id: "storage_sr",
        to_id: "perf_sr",
        relation_type: RelationType::Collaborator,
    },
    // === SHIFT BUDDIES (Same shift, coffee breaks) ===
    // Morning shift: Michael, Nora, Hugo
    Relationship {
        from_id: "network_jr",
        to_id: "hardware_jr",
        relation_type: RelationType::ShiftBuddy,
    },
    Relationship {
        from_id: "hardware_jr",
        to_id: "services_jr",
        relation_type: RelationType::ShiftBuddy,
    },
    // Day shift: Sofia, Lars, Priya, Lea
    Relationship {
        from_id: "desktop_jr",
        to_id: "security_jr",
        relation_type: RelationType::ShiftBuddy,
    },
    Relationship {
        from_id: "storage_jr",
        to_id: "logs_sr",
        relation_type: RelationType::ShiftBuddy,
    },
    // Evening shift: Erik, Ines, Kari
    Relationship {
        from_id: "desktop_sr",
        to_id: "storage_sr",
        relation_type: RelationType::ShiftBuddy,
    },
    Relationship {
        from_id: "storage_sr",
        to_id: "perf_jr",
        relation_type: RelationType::ShiftBuddy,
    },
    // Night shift: Oskar, Daniel
    Relationship {
        from_id: "security_sr",
        to_id: "logs_jr",
        relation_type: RelationType::ShiftBuddy,
    },
];
