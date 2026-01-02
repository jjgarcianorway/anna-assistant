//! Misclassification detection logic.

use super::types::MisclassificationSignal;

/// Detector for misclassification signals.
pub struct MisclassificationDetector;

impl MisclassificationDetector {
    /// Phrases that indicate misclassification.
    const MISCLASS_PHRASES: &'static [&'static str] = &[
        "that's not what i asked",
        "not what i meant",
        "wrong question",
        "i didn't ask about",
        "i asked about",
        "i wanted to know",
        "that doesn't answer",
        "you didn't answer",
        "different question",
        "misunderstood",
    ];

    /// Phrases that indicate rephrase (potential misclassification).
    const REPHRASE_PHRASES: &'static [&'static str] = &[
        "let me rephrase",
        "what i mean is",
        "to clarify",
        "more specifically",
        "i meant",
        "what i'm asking is",
    ];

    /// Check if user response indicates misclassification.
    pub fn detect(user_response: &str) -> MisclassificationSignal {
        let lower = user_response.to_lowercase();

        // Check for explicit misclassification phrases
        for phrase in Self::MISCLASS_PHRASES {
            if lower.contains(phrase) {
                return MisclassificationSignal::Explicit {
                    phrase: phrase.to_string(),
                };
            }
        }

        // Check for rephrase phrases
        for phrase in Self::REPHRASE_PHRASES {
            if lower.contains(phrase) {
                return MisclassificationSignal::Rephrase {
                    phrase: phrase.to_string(),
                };
            }
        }

        MisclassificationSignal::None
    }

    /// Check if user is asking about a different subject than answered.
    pub fn subject_mismatch(answered_subject: crate::question_contract::intent::Subject, user_response: &str) -> bool {
        use crate::question_contract::intent::Subject;

        let lower = user_response.to_lowercase();

        let subject_keywords: &[(&[&str], Subject)] = &[
            (&["memory", "ram", "swap"], Subject::Memory),
            (&["cpu", "processor"], Subject::Cpu),
            (&["disk", "storage", "partition"], Subject::Disk),
            (&["service", "systemd", "unit"], Subject::Service),
            (&["network", "wifi", "ethernet", "ip"], Subject::Network),
            (&["gpu", "graphics", "nvidia", "driver"], Subject::Gpu),
            (&["boot", "startup"], Subject::Boot),
            (&["audio", "sound", "volume"], Subject::Audio),
        ];

        for (keywords, subject) in subject_keywords {
            if keywords.iter().any(|k| lower.contains(k)) {
                if *subject != answered_subject && answered_subject != Subject::Unknown {
                    return true;
                }
            }
        }

        false
    }
}
