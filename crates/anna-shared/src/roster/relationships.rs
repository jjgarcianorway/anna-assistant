//! Staff relationships and dynamics (v0.0.262).
//!
//! Defines mentor-mentee, friendship, and cross-team collaboration
//! relationships between staff members for richer dialogue.

use serde::{Deserialize, Serialize};

/// Type of relationship between two staff members
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationType {
    /// Senior mentors junior (within same team)
    Mentor,
    /// Cross-team friendship (similar interests)
    Friend,
    /// Friendly rivalry (competitive but respectful)
    Rival,
    /// Cross-team collaboration (complementary skills)
    Collaborator,
    /// Coffee buddies (same shift, hang out together)
    ShiftBuddy,
}

/// A relationship between two staff members
#[derive(Debug, Clone, Copy)]
pub struct Relationship {
    /// Source person ID
    pub from_id: &'static str,
    /// Target person ID
    pub to_id: &'static str,
    /// Type of relationship
    pub relation_type: RelationType,
}

/// All relationships in the department
/// Designed based on personality quirks and logical interactions
const RELATIONSHIPS: &[Relationship] = &[
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

/// Get the mentor for a junior (returns senior in same team)
pub fn get_mentor(junior_id: &str) -> Option<&'static str> {
    RELATIONSHIPS
        .iter()
        .find(|r| r.to_id == junior_id && r.relation_type == RelationType::Mentor)
        .map(|r| r.from_id)
}

/// Get all relationships for a person
pub fn relationships_for(person_id: &str) -> Vec<Relationship> {
    RELATIONSHIPS
        .iter()
        .filter(|r| r.from_id == person_id || r.to_id == person_id)
        .copied()
        .collect()
}

/// Get relationships of a specific type for a person
pub fn relationships_of_type(person_id: &str, rel_type: RelationType) -> Vec<&'static str> {
    RELATIONSHIPS
        .iter()
        .filter_map(|r| {
            if r.relation_type != rel_type {
                return None;
            }
            if r.from_id == person_id {
                Some(r.to_id)
            } else if r.to_id == person_id {
                Some(r.from_id)
            } else {
                None
            }
        })
        .collect()
}

/// Get friends for a person
pub fn get_friends(person_id: &str) -> Vec<&'static str> {
    relationships_of_type(person_id, RelationType::Friend)
}

/// Get collaborators for a person
pub fn get_collaborators(person_id: &str) -> Vec<&'static str> {
    relationships_of_type(person_id, RelationType::Collaborator)
}

/// Get shift buddies for a person
pub fn get_shift_buddies(person_id: &str) -> Vec<&'static str> {
    relationships_of_type(person_id, RelationType::ShiftBuddy)
}

/// Get rival (if any) for a person
pub fn get_rival(person_id: &str) -> Option<&'static str> {
    relationships_of_type(person_id, RelationType::Rival).first().copied()
}

/// Check if two people have a relationship
pub fn have_relationship(person_a: &str, person_b: &str) -> Option<RelationType> {
    RELATIONSHIPS.iter().find_map(|r| {
        if (r.from_id == person_a && r.to_id == person_b)
            || (r.from_id == person_b && r.to_id == person_a)
        {
            Some(r.relation_type)
        } else {
            None
        }
    })
}

/// Get a relationship-aware phrase for escalation
pub fn escalation_phrase(junior_id: &str, senior_id: &str, seed: u64) -> &'static str {
    let rel = have_relationship(junior_id, senior_id);
    let phrases: &[&str] = match rel {
        Some(RelationType::Mentor) => &[
            "Hey {senior}, got a tricky one for you.",
            "Mind taking a look at this, {senior}?",
            "{senior}, I could use your expertise here.",
            "This one's above my pay grade, {senior}.",
        ],
        Some(RelationType::Rival) => &[
            "Alright {senior}, let's see if you can crack this.",
            "Bet you can't figure this one out, {senior}.",
            "Here's a challenge for you, {senior}.",
        ],
        Some(RelationType::Friend) => &[
            "Hey {senior}! Got something interesting for you.",
            "{senior}! Check this out, I think you'll like it.",
            "Friend to friend, {senior} - need your help here.",
        ],
        _ => &[
            "Escalating to {senior}.",
            "{senior}, could you review this?",
            "Passing this to {senior} for review.",
        ],
    };
    let idx = (seed as usize) % phrases.len();
    phrases[idx]
}

/// Get a relationship-aware response from senior to junior
pub fn senior_response_phrase(senior_id: &str, junior_id: &str, helpful: bool, seed: u64) -> &'static str {
    let rel = have_relationship(senior_id, junior_id);
    let phrases: &[&str] = match (rel, helpful) {
        (Some(RelationType::Mentor), true) => &[
            "Good question, {junior}. Let me show you...",
            "Ah, I remember this one. Here's the trick...",
            "Nice catch bringing this to me, {junior}.",
            "{junior}, watch and learn...",
        ],
        (Some(RelationType::Mentor), false) => &[
            "Hmm, that's a tough one. Let me think...",
            "Good instinct to escalate, {junior}.",
            "You were right to ask, {junior}. This is tricky.",
        ],
        (Some(RelationType::Rival), true) => &[
            "Ha! Easy. Watch this, {junior}.",
            "Thought you had me stumped? Nope.",
            "Child's play. Here's how it's done...",
        ],
        (Some(RelationType::Rival), false) => &[
            "Okay, {junior}, you found a good one.",
            "I'll admit, this is interesting...",
            "Don't get used to stumping me, but...",
        ],
        (_, true) => &[
            "Let me see... Ah, I know this one.",
            "I've seen this before. Here's what we do...",
            "Good catch. Here's the answer...",
        ],
        (_, false) => &[
            "Hmm, tricky one. Let me think...",
            "That's unusual. Give me a moment.",
            "Interesting edge case here...",
        ],
    };
    let idx = (seed as usize) % phrases.len();
    phrases[idx]
}

/// Get a relationship-aware greeting when mentioning someone
pub fn mention_phrase(from_id: &str, about_id: &str, seed: u64) -> &'static str {
    let rel = have_relationship(from_id, about_id);
    let phrases: &[&str] = match rel {
        Some(RelationType::Mentor) => &[
            "My mentor {name} always says...",
            "{name} taught me that...",
            "As {name} would put it...",
        ],
        Some(RelationType::Friend) => &[
            "My buddy {name}...",
            "{name} and I were just discussing...",
            "Funny, {name} mentioned something similar...",
        ],
        Some(RelationType::Rival) => &[
            "{name} would disagree, but...",
            "Don't tell {name} I said this...",
            "{name} has a different take, but...",
        ],
        Some(RelationType::ShiftBuddy) => &[
            "{name} from the shift was saying...",
            "Ran into {name} at the coffee machine...",
        ],
        Some(RelationType::Collaborator) => &[
            "I work with {name} on these...",
            "{name} and I often see this...",
        ],
        None => &["{name} might know more...", "I'd ask {name} about this..."],
    };
    let idx = (seed as usize) % phrases.len();
    phrases[idx]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_mentor() {
        assert_eq!(get_mentor("network_jr"), Some("network_sr"));
        assert_eq!(get_mentor("desktop_jr"), Some("desktop_sr"));
        assert_eq!(get_mentor("network_sr"), None); // Seniors don't have mentors
    }

    #[test]
    fn test_get_friends() {
        let friends = get_friends("desktop_jr");
        assert!(friends.contains(&"storage_jr")); // Sofia & Lars
    }

    #[test]
    fn test_get_collaborators() {
        let collabs = get_collaborators("network_jr");
        assert!(collabs.contains(&"security_jr")); // Network + Security
    }

    #[test]
    fn test_have_relationship() {
        assert_eq!(
            have_relationship("network_sr", "network_jr"),
            Some(RelationType::Mentor)
        );
        assert_eq!(
            have_relationship("desktop_sr", "services_sr"),
            Some(RelationType::Rival)
        );
        assert_eq!(have_relationship("network_jr", "logs_sr"), None);
    }

    #[test]
    fn test_escalation_phrase() {
        let phrase = escalation_phrase("network_jr", "network_sr", 42);
        assert!(phrase.contains("{senior}"));
    }

    #[test]
    fn test_mention_phrase() {
        let phrase = mention_phrase("network_jr", "network_sr", 42);
        assert!(phrase.contains("{name}"));
    }

    #[test]
    fn test_shift_buddies() {
        // Morning shift buddies
        let buddies = get_shift_buddies("network_jr");
        assert!(buddies.contains(&"hardware_jr")); // Michael & Nora
    }
}
