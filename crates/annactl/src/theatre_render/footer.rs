//! Theatre footer rendering (v0.0.202).

use anna_shared::narrator::{it_confidence, it_domain_context};
use anna_shared::roster::person_by_id;
use anna_shared::rpc::ServiceDeskResult;
use anna_shared::ui::colors;

use super::helpers::reliability_color;

/// Print the footer
/// v0.0.106: Shows case number and assigned staff when available
/// v0.0.109: Shows staff specializations
/// v0.0.170: Enhanced staff display with name and position prominently
pub fn print_footer(result: &ServiceDeskResult) {
    let rel_color = reliability_color(result.reliability_score);
    let confidence_note = it_confidence(result.reliability_score);
    let domain_str = result.domain.to_string();
    let domain_context = it_domain_context(&domain_str);

    // v0.0.170: Show staff member who handled the request with name and role prominently
    if let Some(ref staff_id) = result.staff_id {
        if let Some(person) = person_by_id(staff_id) {
            println!(
                "{}Handled by:{} {}{} ({}){}",
                colors::DIM,
                colors::RESET,
                colors::WARN,
                person.display_name,
                person.role_title,
                colors::RESET
            );
            if !person.specializations.is_empty() {
                let specs = person.specialization_str();
                println!(
                    "{}  Specializes in: {}{}",
                    colors::DIM,
                    specs,
                    colors::RESET
                );
            }
        }
    } else if let Some(ref assigned) = result.assigned_staff {
        // Fallback to assigned_staff string if no staff_id
        println!(
            "{}Handled by:{} {}{}{}",
            colors::DIM,
            colors::RESET,
            colors::WARN,
            assigned,
            colors::RESET
        );
    }

    // v0.0.106: Case number on separate line
    if let Some(ref case_num) = result.case_number {
        println!("{}Case: {}{}", colors::DIM, case_num, colors::RESET);
    }

    // Evidence source
    let evidence_source = if result.reliability_signals.answer_grounded {
        format_evidence_source(result)
    } else {
        String::new()
    };

    if evidence_source.is_empty() {
        println!(
            "{}{} | {} | {}{}%{}",
            colors::DIM,
            domain_context,
            confidence_note,
            rel_color,
            result.reliability_score,
            colors::RESET
        );
    } else {
        println!(
            "{}{} | {} | {}{}%{} | {}{}",
            colors::DIM,
            domain_context,
            confidence_note,
            rel_color,
            result.reliability_score,
            colors::RESET,
            colors::DIM,
            evidence_source
        );
    }
}

/// Format evidence source for footer
pub fn format_evidence_source(result: &ServiceDeskResult) -> String {
    if let Some(trace) = &result.execution_trace {
        if !trace.evidence_kinds.is_empty() {
            let kinds: Vec<&str> = trace
                .evidence_kinds
                .iter()
                .map(|k| match k {
                    anna_shared::trace::EvidenceKind::Audio => "audio",
                    anna_shared::trace::EvidenceKind::ToolExists => "tools",
                    anna_shared::trace::EvidenceKind::Memory => "memory",
                    anna_shared::trace::EvidenceKind::Disk => "disk",
                    anna_shared::trace::EvidenceKind::Cpu => "cpu",
                    anna_shared::trace::EvidenceKind::Processes => "ps",
                    anna_shared::trace::EvidenceKind::Network => "network",
                    anna_shared::trace::EvidenceKind::Services => "services",
                    anna_shared::trace::EvidenceKind::Journal => "logs",
                    _ => "probe",
                })
                .collect();
            return format!("Verified from {}", kinds.join("+"));
        }
    }

    let success = result
        .evidence
        .probes_executed
        .iter()
        .filter(|p| p.exit_code == 0)
        .count();
    if success > 0 {
        format!(
            "Verified from {} probe{}",
            success,
            if success == 1 { "" } else { "s" }
        )
    } else {
        String::new()
    }
}
