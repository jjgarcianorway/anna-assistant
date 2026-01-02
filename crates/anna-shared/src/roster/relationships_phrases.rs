//! Relationship-aware dialogue phrases (v0.0.262).

use super::relationships_queries::have_relationship;
use super::relationships_types::RelationType;

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
pub fn senior_response_phrase(
    senior_id: &str,
    junior_id: &str,
    helpful: bool,
    seed: u64,
) -> &'static str {
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
