//! Tier builder functions for various intents.

use super::super::evidence_gate::EvidenceStatus;
use super::super::intent_schema::CanonicalIntent;
use super::extractors::*;
use super::types::TieredAnswer;

/// Build tiered answer for boot_perf intent.
pub fn build_boot_perf_tiers(evidence: &EvidenceStatus) -> TieredAnswer {
    let mut answer = TieredAnswer::new(CanonicalIntent::BootPerf);

    // Tier 1: Boot time facts from systemd-analyze
    if let Some(analyze) = evidence.get_output("systemd_analyze") {
        let boot_time = extract_boot_time_fact(analyze);
        answer = answer.with_facts(&boot_time);
    }

    // Tier 2: Top offenders from systemd-analyze blame
    if let Some(blame) = evidence.get_output("systemd_blame") {
        let top_offenders = extract_top_offenders(blame, 5);
        if !top_offenders.is_empty() {
            answer = answer.with_key_items(top_offenders);
        }
    }

    // Tier 3 would be specialist synthesis (not built here)

    answer
}

/// Build tiered answer for mem_status intent.
pub fn build_mem_status_tiers(evidence: &EvidenceStatus) -> TieredAnswer {
    let mut answer = TieredAnswer::new(CanonicalIntent::MemStatus);

    // Tier 1: Memory facts from free -h
    if let Some(free_h) = evidence.get_output("free_h") {
        let mem_fact = extract_memory_fact(free_h);
        answer = answer.with_facts(&mem_fact);
    }

    // Tier 2: Top memory consumers (if we have ps data)
    if let Some(ps_mem) = evidence.get_output("ps_mem_top") {
        let top_procs = extract_top_mem_processes(ps_mem, 5);
        if !top_procs.is_empty() {
            answer = answer.with_key_items(top_procs);
        }
    }

    answer
}

/// Build tiered answer for disk_usage intent.
pub fn build_disk_usage_tiers(evidence: &EvidenceStatus) -> TieredAnswer {
    let mut answer = TieredAnswer::new(CanonicalIntent::DiskUsage);

    // Tier 1: Disk facts from df -h
    if let Some(df_h) = evidence.get_output("df_h") {
        let disk_fact = extract_disk_fact(df_h);
        answer = answer.with_facts(&disk_fact);
    }

    // Tier 2: Top directories (if we have du data)
    if let Some(du) = evidence.get_output("du_top_dirs") {
        let top_dirs = extract_top_directories(du, 5);
        if !top_dirs.is_empty() {
            answer = answer.with_key_items(top_dirs);
        }
    }

    answer
}

/// Build tiered answer for cpu_load intent.
pub fn build_cpu_load_tiers(evidence: &EvidenceStatus) -> TieredAnswer {
    let mut answer = TieredAnswer::new(CanonicalIntent::CpuLoad);

    // Tier 1: Load average from uptime
    if let Some(uptime) = evidence.get_output("uptime") {
        let load_fact = extract_load_fact(uptime);
        answer = answer.with_facts(&load_fact);
    }

    // Tier 2: Top CPU consumers
    if let Some(top_cpu) = evidence.get_output("top_cpu") {
        let top_procs = extract_top_cpu_processes(top_cpu, 5);
        if !top_procs.is_empty() {
            answer = answer.with_key_items(top_procs);
        }
    }

    answer
}

/// Build tiered answer for gpu_driver intent.
pub fn build_gpu_driver_tiers(evidence: &EvidenceStatus) -> TieredAnswer {
    let mut answer = TieredAnswer::new(CanonicalIntent::GpuDriver);

    // Tier 1: GPU hardware from lspci
    if let Some(lspci) = evidence.get_output("lspci_gpu") {
        let gpu_fact = extract_gpu_fact(lspci);
        answer = answer.with_facts(&gpu_fact);
    }

    // Tier 2: Driver info
    let mut driver_items = Vec::new();
    if let Some(lspci_k) = evidence.get_output("lspci_k_gpu") {
        if let Some(driver) = extract_kernel_driver(lspci_k) {
            driver_items.push(format!("Kernel driver: {}", driver));
        }
    }
    if let Some(lsmod) = evidence.get_output("lsmod_gpu") {
        let modules = extract_gpu_modules(lsmod);
        if !modules.is_empty() {
            driver_items.push(format!("Loaded modules: {}", modules.join(", ")));
        }
    }
    if !driver_items.is_empty() {
        answer = answer.with_key_items(driver_items);
    }

    answer
}
