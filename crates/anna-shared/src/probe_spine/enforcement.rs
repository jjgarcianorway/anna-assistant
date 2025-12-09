//! Probe enforcement logic (v0.0.193).

use super::commands::{probe_to_command, probes_for_evidence};
use super::types::{EvidenceKind, ProbeId, RouteCapability};

/// Decision from probe spine enforcement.
#[derive(Debug, Clone)]
pub struct ProbeSpineDecision {
    pub enforced: bool,
    pub reason: String,
    pub probes: Vec<ProbeId>,
    pub evidence_kinds: Vec<EvidenceKind>,
}

/// Enforce spine probes: if translator proposed empty probes but query requires evidence,
/// return the minimum required probes.
pub fn enforce_spine_probes(
    translator_probes: &[String],
    capability: &RouteCapability,
) -> (Vec<String>, Option<String>) {
    if !translator_probes.is_empty() {
        return (translator_probes.to_vec(), None);
    }

    if !capability.evidence_required {
        return (vec![], None);
    }

    if capability.spine_probes.is_empty() && capability.required_evidence.is_empty() {
        return (vec![], None);
    }

    // Build probe list from spine_probes and required_evidence
    let mut probes: Vec<String> = capability
        .spine_probes
        .iter()
        .map(probe_to_command)
        .collect();

    for kind in &capability.required_evidence {
        for probe in probes_for_evidence(*kind) {
            let cmd = probe_to_command(&probe);
            if !probes.contains(&cmd) {
                probes.push(cmd);
            }
        }
    }

    let reason = if probes.is_empty() {
        None
    } else {
        Some(format!(
            "query requires {:?} evidence, enforcing {} probe(s)",
            capability.required_evidence,
            probes.len()
        ))
    };

    (probes, reason)
}

/// Extract package name from "do I have X" style queries.
fn extract_package_name(text: &str) -> Option<String> {
    let lower = text.to_lowercase();
    // Patterns: "do I have nano", "is nano installed", "have I got vim"
    let patterns = [
        ("do i have ", true),
        ("do you have ", true),
        ("is ", false), // "is nano installed"
        ("have i got ", true),
        ("got ", true),
    ];

    for (pattern, after) in patterns {
        if let Some(idx) = lower.find(pattern) {
            let start = if after {
                idx + pattern.len()
            } else {
                idx + pattern.len()
            };
            let rest = &text[start..];
            // Extract first word as package name
            let pkg: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                .collect();
            if !pkg.is_empty() && pkg.len() > 1 {
                // Skip if followed by "installed" (for "is X installed" pattern)
                let pkg_lower = pkg.to_lowercase();
                if pkg_lower != "it" && pkg_lower != "there" && pkg_lower != "this" {
                    return Some(pkg.to_lowercase());
                }
            }
        }
    }
    None
}

/// Enforce minimum probes based on USER TEXT keywords (last line of defense).
pub fn enforce_minimum_probes(user_text: &str, translator_probes: &[String]) -> ProbeSpineDecision {
    let lower = user_text.to_lowercase();
    let mut probes: Vec<ProbeId> = Vec::new();
    let mut evidence_kinds: Vec<EvidenceKind> = Vec::new();
    let mut reasons: Vec<&str> = Vec::new();

    // Rule 1: Package/tool check
    if lower.contains("do i have")
        || lower.contains("is installed")
        || lower.contains("have i got")
        || lower.contains("installed?")
    {
        if let Some(pkg) = extract_package_name(user_text) {
            probes.push(ProbeId::PacmanQ(pkg.clone()));
            probes.push(ProbeId::CommandV(pkg));
            evidence_kinds.push(EvidenceKind::Packages);
            evidence_kinds.push(EvidenceKind::ToolExists);
            reasons.push("package/tool check");
        }
    }

    // Rule 2: Sound/audio
    if (lower.contains("sound card")
        || lower.contains("audio device")
        || lower.contains("audio card")
        || lower.contains("sound device")
        || (lower.contains("sound") && lower.contains("hardware"))
        || (lower.contains("audio") && lower.contains("hardware")))
        && !probes.iter().any(|p| matches!(p, ProbeId::LspciAudio))
    {
        probes.push(ProbeId::LspciAudio);
        probes.push(ProbeId::PactlCards);
        evidence_kinds.push(EvidenceKind::Audio);
        reasons.push("audio hardware query");
    }

    // Rule 3: Temperature
    if (lower.contains("temperature")
        || lower.contains(" temp ")
        || lower.contains("thermal")
        || lower.contains("temps?")
        || lower.contains("how hot"))
        && !probes.iter().any(|p| matches!(p, ProbeId::Sensors))
    {
        probes.push(ProbeId::Sensors);
        evidence_kinds.push(EvidenceKind::CpuTemperature);
        reasons.push("temperature query");
    }

    // Rule 4: CPU cores/model/architecture
    if (lower.contains("cores")
        || lower.contains("cpu model")
        || lower.contains("architecture")
        || lower.contains("processor")
        || lower.contains("how many cpu"))
        && !probes.iter().any(|p| matches!(p, ProbeId::Lscpu))
    {
        probes.push(ProbeId::Lscpu);
        evidence_kinds.push(EvidenceKind::Cpu);
        reasons.push("CPU info query");
    }

    // Rule 5: System health / errors / problems
    if (lower.contains("how is my computer")
        || lower.contains("errors")
        || lower.contains("problems")
        || lower.contains("system health")
        || lower.contains("what's wrong")
        || lower.contains("issues"))
        && !probes.iter().any(|p| matches!(p, ProbeId::JournalErrors))
    {
        probes.push(ProbeId::JournalErrors);
        probes.push(ProbeId::FailedUnits);
        probes.push(ProbeId::SystemdAnalyze);
        evidence_kinds.push(EvidenceKind::Journal);
        evidence_kinds.push(EvidenceKind::Services);
        evidence_kinds.push(EvidenceKind::BootTime);
        reasons.push("system health query");
    }

    // Rule 6: Editor configuration queries
    let editor_config_verbs = lower.contains("enable")
        || lower.contains("turn on")
        || lower.contains("activate")
        || lower.contains("set up")
        || lower.contains("configure")
        || lower.contains("show")
        || lower.contains("set ");

    let editor_config_features = lower.contains("syntax")
        || lower.contains("highlight")
        || lower.contains("line number")
        || lower.contains("word wrap")
        || lower.contains("auto indent")
        || lower.contains("theme")
        || lower.contains("colorscheme")
        || lower.contains("color scheme");

    let named_editor = lower.contains(" vim")
        || lower.contains(" nvim")
        || lower.contains(" nano")
        || lower.contains(" emacs")
        || lower.contains(" micro")
        || lower.contains(" helix")
        || lower.contains(" code")
        || lower.contains("vscode")
        || lower.contains(" kate")
        || lower.contains(" gedit");

    let is_editor_config =
        (editor_config_verbs && editor_config_features) || (named_editor && editor_config_features);

    if is_editor_config {
        let editors = [
            "code", "vim", "nvim", "nano", "emacs", "micro", "helix", "hx", "kate", "gedit",
        ];
        for editor in editors {
            probes.push(ProbeId::CommandV(editor.to_string()));
        }
        evidence_kinds.push(EvidenceKind::ToolExists);
        reasons.push("editor configuration needs installed editor detection");
    }

    // Merge with translator probes (translator probes come first)
    let mut final_probes = probes.clone();
    for tp in translator_probes {
        let probe_id = ProbeId::Custom(tp.clone());
        if !final_probes.iter().any(|p| probe_to_command(p) == *tp) {
            final_probes.insert(0, probe_id);
        }
    }

    let enforced = !probes.is_empty();
    let reason = if reasons.is_empty() {
        "no keyword matches".to_string()
    } else {
        format!("enforced for: {}", reasons.join(", "))
    };

    ProbeSpineDecision {
        enforced,
        reason,
        probes: final_probes,
        evidence_kinds,
    }
}
