//! Relationship query functions (v0.0.262).

use super::relationships_types::{Relationship, RelationType, RELATIONSHIPS};

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
    relationships_of_type(person_id, RelationType::Rival)
        .first()
        .copied()
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
