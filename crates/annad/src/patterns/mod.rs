//! Common Linux patterns that should get instant answers without clarification.
//!
//! These are well-known issues with standard solutions.
//! The pattern system provides:
//! - Direct pattern matching for common queries
//! - Synonym expansion for query variations
//! - Typo correction for misspellings
//! - Fuzzy matching as last resort
//! - Pattern pre-execution for grounded answers

// Pattern category modules
mod appimage;
mod audio;
mod aur;
mod backup;
mod bluetooth;
mod boot;
mod container;
mod cron;
mod desktop;
mod development;
mod display;
mod encryption;
mod errors;
mod factual;
mod filesystem;
mod gaming;
mod hardware;
mod howto;
mod kernel;
mod locale;
mod logs;
mod memory;
mod network;
mod nvidia;
mod pacman;
mod performance;
pub mod power;
mod printing;
mod process;
mod recovery;
mod security;
mod selinux;
mod smart;
mod ssh;
mod sysinfo;
mod systemd;
mod time;
mod users;
mod virtualization;
mod wm;
mod xorg;
mod zfs;

// Helper modules
mod normalize;
mod preexec;
mod stats;
mod synonyms;
mod typos;

// Re-exports
pub use preexec::{match_and_preexec, PatternPreExecResult};
pub use stats::{get_pattern_stats, get_total_pattern_hits, total_pattern_count};

use anna_shared::rpc::DeepUnderstanding;
use tracing::debug;

use normalize::normalize_query;
use stats::record_pattern_hit;
use synonyms::expand_with_synonyms;
use typos::{fix_typos, fuzzy_correct_query};

/// Check if query contains keyword as a whole word (not substring).
/// Prevents "bandwidth" matching "id", "what" matching "at", "update" matching "date".
pub fn contains_word(query: &str, word: &str) -> bool {
    for (i, _) in query.match_indices(word) {
        let before_ok = i == 0 || !query.as_bytes()[i - 1].is_ascii_alphanumeric();
        let after_ok = i + word.len() >= query.len()
            || !query.as_bytes()[i + word.len()].is_ascii_alphanumeric();
        if before_ok && after_ok {
            return true;
        }
    }
    false
}

/// Check if a question matches a common pattern that has a known solution.
/// Returns Some(DeepUnderstanding) with high confidence if matched.
pub fn match_common_pattern(question: &str) -> Option<DeepUnderstanding> {
    let q = question.to_lowercase();

    // Try direct match first
    if let Some(result) = match_patterns_internal(&q) {
        return Some(result);
    }

    // Try with synonym expansion
    let expanded = expand_with_synonyms(&q);
    if expanded != q {
        debug!("Pattern: trying synonym expansion: {} -> {}", q, expanded);
        if let Some(result) = match_patterns_internal(&expanded) {
            return Some(result);
        }
    }

    // Try with normalized query
    let normalized = normalize_query(&q);
    if normalized != q {
        debug!("Pattern: trying normalized query: {} -> {}", q, normalized);
        if let Some(result) = match_patterns_internal(&normalized) {
            return Some(result);
        }
        // Try normalized + synonyms
        let norm_expanded = expand_with_synonyms(&normalized);
        if norm_expanded != normalized {
            if let Some(result) = match_patterns_internal(&norm_expanded) {
                return Some(result);
            }
        }
    }

    // Try with known typo corrections
    let typo_fixed = fix_typos(&q);
    if typo_fixed != q {
        debug!("Pattern: trying typo correction: {} -> {}", q, typo_fixed);
        if let Some(result) = match_patterns_internal(&typo_fixed) {
            return Some(result);
        }
        // Try typo-fixed + synonyms
        let typo_expanded = expand_with_synonyms(&typo_fixed);
        if let Some(result) = match_patterns_internal(&typo_expanded) {
            return Some(result);
        }
    }

    // Try fuzzy matching (edit distance) as last resort
    if let Some(fuzzy_corrected) = fuzzy_correct_query(&q) {
        debug!("Pattern: trying fuzzy correction: {} -> {}", q, fuzzy_corrected);
        if let Some(result) = match_patterns_internal(&fuzzy_corrected) {
            return Some(result);
        }
        // Try fuzzy + synonyms
        let fuzzy_expanded = expand_with_synonyms(&fuzzy_corrected);
        if let Some(result) = match_patterns_internal(&fuzzy_expanded) {
            return Some(result);
        }
    }

    None
}

/// Internal pattern matching (called with original and expanded queries).
fn match_patterns_internal(q: &str) -> Option<DeepUnderstanding> {
    // Check each pattern category (order matters - more specific first)
    if let Some(r) = factual::match_patterns(q) {
        record_pattern_hit("factual");
        return Some(r);
    }
    if let Some(r) = hardware::match_patterns(q) {
        record_pattern_hit("hardware");
        return Some(r);
    }
    if let Some(r) = network::match_patterns(q) {
        record_pattern_hit("network");
        return Some(r);
    }
    if let Some(r) = gaming::match_patterns(q) {
        record_pattern_hit("gaming");
        return Some(r);
    }
    if let Some(r) = boot::match_patterns(q) {
        record_pattern_hit("boot");
        return Some(r);
    }
    if let Some(r) = container::match_patterns(q) {
        record_pattern_hit("container");
        return Some(r);
    }
    if let Some(r) = logs::match_patterns(q) {
        record_pattern_hit("logs");
        return Some(r);
    }
    if let Some(r) = audio::match_patterns(q) {
        record_pattern_hit("audio");
        return Some(r);
    }
    if let Some(r) = power::match_patterns(q) {
        record_pattern_hit("power");
        return Some(r);
    }
    if let Some(r) = systemd::match_patterns(q) {
        record_pattern_hit("systemd");
        return Some(r);
    }
    if let Some(r) = filesystem::match_patterns(q) {
        record_pattern_hit("filesystem");
        return Some(r);
    }
    if let Some(r) = process::match_patterns(q) {
        record_pattern_hit("process");
        return Some(r);
    }
    if let Some(r) = cron::match_patterns(q) {
        record_pattern_hit("cron");
        return Some(r);
    }
    if let Some(r) = users::match_patterns(q) {
        record_pattern_hit("users");
        return Some(r);
    }
    if let Some(r) = time::match_patterns(q) {
        record_pattern_hit("time");
        return Some(r);
    }
    if let Some(r) = printing::match_patterns(q) {
        record_pattern_hit("printing");
        return Some(r);
    }
    if let Some(r) = backup::match_patterns(q) {
        record_pattern_hit("backup");
        return Some(r);
    }
    if let Some(r) = locale::match_patterns(q) {
        record_pattern_hit("locale");
        return Some(r);
    }
    if let Some(r) = ssh::match_patterns(q) {
        record_pattern_hit("ssh");
        return Some(r);
    }
    if let Some(r) = memory::match_patterns(q) {
        record_pattern_hit("memory");
        return Some(r);
    }
    if let Some(r) = bluetooth::match_patterns(q) {
        record_pattern_hit("bluetooth");
        return Some(r);
    }
    if let Some(r) = virtualization::match_patterns(q) {
        record_pattern_hit("virtualization");
        return Some(r);
    }
    if let Some(r) = display::match_patterns(q) {
        record_pattern_hit("display");
        return Some(r);
    }
    if let Some(r) = encryption::match_patterns(q) {
        record_pattern_hit("encryption");
        return Some(r);
    }
    if let Some(r) = nvidia::match_patterns(q) {
        record_pattern_hit("nvidia");
        return Some(r);
    }
    if let Some(r) = aur::match_patterns(q) {
        record_pattern_hit("aur");
        return Some(r);
    }
    if let Some(r) = appimage::match_patterns(q) {
        record_pattern_hit("appimage");
        return Some(r);
    }
    if let Some(r) = sysinfo::match_patterns(q) {
        record_pattern_hit("sysinfo");
        return Some(r);
    }
    if let Some(r) = wm::match_patterns(q) {
        record_pattern_hit("wm");
        return Some(r);
    }
    if let Some(r) = kernel::match_patterns(q) {
        record_pattern_hit("kernel");
        return Some(r);
    }
    if let Some(r) = zfs::match_patterns(q) {
        record_pattern_hit("zfs");
        return Some(r);
    }
    if let Some(r) = smart::match_patterns(q) {
        record_pattern_hit("smart");
        return Some(r);
    }
    if let Some(r) = selinux::match_patterns(q) {
        record_pattern_hit("selinux");
        return Some(r);
    }
    if let Some(r) = xorg::match_patterns(q) {
        record_pattern_hit("xorg");
        return Some(r);
    }
    if let Some(r) = development::match_patterns(q) {
        record_pattern_hit("development");
        return Some(r);
    }
    if let Some(r) = security::match_patterns(q) {
        record_pattern_hit("security");
        return Some(r);
    }
    if let Some(r) = desktop::match_patterns(q) {
        record_pattern_hit("desktop");
        return Some(r);
    }
    if let Some(r) = pacman::match_patterns(q) {
        record_pattern_hit("pacman");
        return Some(r);
    }
    if let Some(r) = recovery::match_patterns(q) {
        record_pattern_hit("recovery");
        return Some(r);
    }
    if let Some(r) = errors::match_patterns(q) {
        record_pattern_hit("errors");
        return Some(r);
    }
    if let Some(r) = howto::match_patterns(q) {
        record_pattern_hit("howto");
        return Some(r);
    }
    if let Some(r) = performance::match_patterns(q) {
        record_pattern_hit("performance");
        return Some(r);
    }

    None
}

#[cfg(test)]
mod test_500;

#[cfg(test)]
mod tests;
