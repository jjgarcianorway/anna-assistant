//! LLM, Ollama, and models section handlers for status display.

use anna_shared::ledger::Ledger;
use anna_shared::status::{DaemonStatus, LlmState};
use anna_shared::status_snapshot::StatusSnapshot;
use anna_shared::ui::{colors, kv, print_section_header};

/// Print the [ollama] section (v0.0.449: Separate Ollama status per VISION.md)
pub fn print_ollama_section(snapshot: &StatusSnapshot) {
    print_section_header("ollama");
    let present_str = if snapshot.models.ollama_present {
        format!("{}YES{}", colors::OK, colors::RESET)
    } else {
        format!("{}NO{}", colors::ERR, colors::RESET)
    };
    kv("installed", &present_str);
    let running_str = if snapshot.models.ollama_running {
        format!("{}RUNNING{}", colors::OK, colors::RESET)
    } else {
        format!("{}STOPPED{}", colors::ERR, colors::RESET)
    };
    kv("status", &running_str);
    if let Some(ver) = &snapshot.models.ollama_version {
        kv("version", ver);
    }
}

/// Print the [llm] section
pub fn print_llm_section(status: &DaemonStatus) {
    print_section_header("llm");
    kv("provider", &status.llm.provider);
    let llm_state_str = match status.llm.state {
        LlmState::Ready => format!("{}READY{}", colors::OK, colors::RESET),
        LlmState::Bootstrapping => {
            if let Some(phase) = &status.llm.phase {
                format!("{}{}...{}", colors::WARN, phase, colors::RESET)
            } else {
                format!("{}STARTING...{}", colors::WARN, colors::RESET)
            }
        }
        LlmState::PullingModels => {
            // v0.0.310: Ready for queries, models loading in background
            format!("{}READY{} (models loading)", colors::OK, colors::RESET)
        }
        LlmState::Error => format!("{}ERROR{}", colors::ERR, colors::RESET),
    };
    kv("state", &llm_state_str);

    // v0.0.278: Show full model hierarchy (translator < junior < senior)
    kv("model_hierarchy", "");
    if let Some(model) = &status.llm.translator_model {
        println!(
            "    {}translator{}  {}  (query classification, fastest)",
            colors::DIM,
            colors::RESET,
            model
        );
    }
    if let Some(model) = &status.llm.junior_model {
        println!(
            "    {}junior{}      {}  (regular queries)",
            colors::DIM,
            colors::RESET,
            model
        );
    } else if let Some(model) = &status.llm.specialist_model {
        // Fallback to legacy specialist_model
        println!(
            "    {}junior{}      {}  (regular queries)",
            colors::DIM,
            colors::RESET,
            model
        );
    }
    if let Some(model) = &status.llm.senior_model {
        println!(
            "    {}senior{}      {}  (complex/escalated)",
            colors::DIM,
            colors::RESET,
            model
        );
    }

    kv("routing_policy", "hardware-aware  (local)");
    kv(
        "last_model_check",
        &format!("{}OK{}", colors::OK, colors::RESET),
    );

    // v0.0.267: Show models downloaded by Anna from ledger
    if let Ok(ledger) = Ledger::load() {
        let models = ledger.models_pulled();
        if !models.is_empty() {
            kv("models_by_anna", &format!("{}", models.len()));
            for model in models.iter().take(5) {
                println!("    {}{}{}", colors::DIM, model, colors::RESET);
            }
            if models.len() > 5 {
                println!(
                    "    {}... and {} more{}",
                    colors::DIM,
                    models.len() - 5,
                    colors::RESET
                );
            }
        }
    }
}
