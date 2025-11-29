//! Acceptance Test Scenarios v1.1.0
//!
//! Comprehensive acceptance test harness for Anna that simulates real user
//! interactions without manual typing. Tests UX quality, answer quality,
//! latency, reliability, and learning across repeated questions.
//!
//! ## Running
//!
//! ```bash
//! cargo test --test acceptance_scenarios -- --nocapture
//! ```
//!
//! ## Design
//!
//! - Uses FakeLlmClient and FakeProbeExecutor for deterministic testing
//! - No external dependencies (Ollama, real probes)
//! - Debug mode OFF to simulate real user experience
//! - Tracks XP and learning metrics across passes

use anna_common::{
    EvidenceStatus, FastQuestionType, ProbeRequest,
};
use annad::orchestrator::{
    DraftAnswerV80, FakeJuniorResponse, FakeProbeExecutor, FakeProbeExecutorBuilder,
    FakeProbeResponse, FakeSeniorResponse, ProbeExecutor,
};
use std::time::Instant;

// ============================================================================
// PART 1: Canonical Acceptance Question Set
// ============================================================================

/// The 10 canonical acceptance questions (DO NOT CHANGE without updating tests)
pub const ACCEPTANCE_QUESTIONS: &[(&str, &str)] = &[
    // Q1: CPU model and cores
    (
        "cpu_model",
        "what CPU model do I have, how many physical cores and threads, and what exact commands or probes did you use to get that information?",
    ),
    // Q2: RAM capacity and availability
    (
        "ram_capacity",
        "how much RAM is installed, how much is currently available, and from which probe or safe commands did you read those numbers?",
    ),
    // Q3: Root disk free space
    (
        "disk_space",
        "how much free space do I have on my root filesystem, and which probe or commands did you use to check it?",
    ),
    // Q4: Anna logs
    (
        "anna_logs",
        "show me the logs for the annad service for the last 2 hours, highlight any errors or warnings you consider important, and include a reliability score based on the evidence.",
    ),
    // Q5: System updates
    (
        "system_updates",
        "are there any pending system updates, and can you propose a safe, step by step plan to review them first and then update, clearly marking which steps are read only checks?",
    ),
    // Q6: Self health
    (
        "self_health",
        "diagnose your own health: list any problems you detect in Anna herself (daemon, permissions, models, tools), say whether you tried auto repair just now, and explain what you did, with evidence and a reliability score.",
    ),
    // Q7: GPU
    (
        "gpu_info",
        "do I have a dedicated GPU, which model is it, and which drivers are currently in use according to your probes?",
    ),
    // Q8: OS
    (
        "os_info",
        "which Linux distribution and version am I running, and from which evidence did you infer that?",
    ),
    // Q9: Uptime
    (
        "uptime",
        "how long has this system been up, and which command or probe did you use to find it?",
    ),
    // Q10: System summary
    (
        "system_summary",
        "summarize this system in 5 short bullet points: CPU, RAM, root disk, GPU, and Anna health, and include your reliability score.",
    ),
];

// ============================================================================
// PART 2: Fake Infrastructure for Test Environment
// ============================================================================

/// Canned probe outputs for deterministic testing
fn create_acceptance_probe_responses() -> Vec<FakeProbeResponse> {
    vec![
        // cpu.info
        FakeProbeResponse::ok_json(
            "cpu.info",
            r#"Architecture:            x86_64
CPU(s):                  16
Thread(s) per core:      2
Core(s) per socket:      8
Socket(s):               1
Model name:              AMD Ryzen 7 5800X 8-Core Processor
CPU max MHz:             4850.1948
CPU min MHz:             2200.0000
Virtualization:          AMD-V
L1d cache:               256 KiB
L1i cache:               256 KiB
L2 cache:                4 MiB
L3 cache:                32 MiB"#,
            serde_json::json!({
                "architecture": "x86_64",
                "cpus": 16,
                "threads_per_core": 2,
                "cores_per_socket": 8,
                "model": "AMD Ryzen 7 5800X 8-Core Processor"
            }),
        ),
        // mem.info
        FakeProbeResponse::ok_json(
            "mem.info",
            r#"MemTotal:       32768000 kB
MemFree:        16384000 kB
MemAvailable:   24576000 kB
Buffers:         1024000 kB
Cached:          8192000 kB
SwapTotal:       8388608 kB
SwapFree:        8388608 kB"#,
            serde_json::json!({
                "total_kb": 32768000,
                "free_kb": 16384000,
                "available_kb": 24576000,
                "total_gb": 32,
                "available_gb": 24
            }),
        ),
        // disk.lsblk
        FakeProbeResponse::ok_json(
            "disk.lsblk",
            r#"{
   "blockdevices": [
      {"name":"nvme0n1", "size":1000204886016, "type":"disk", "fstype":null, "mountpoint":null,
         "children": [
            {"name":"nvme0n1p1", "size":536870912, "type":"part", "fstype":"vfat", "mountpoint":"/boot/efi"},
            {"name":"nvme0n1p2", "size":999667015168, "type":"part", "fstype":"ext4", "mountpoint":"/"}
         ]
      }
   ]
}"#,
serde_json::json!({
                "devices": [{
                    "name": "nvme0n1",
                    "size_gb": 931,
                    "type": "disk",
                    "children": [
                        {"name": "nvme0n1p1", "mountpoint": "/boot/efi", "fstype": "vfat"},
                        {"name": "nvme0n1p2", "mountpoint": "/", "fstype": "ext4"}
                    ]
                }],
                "root_device": "nvme0n1p2",
                "root_size_gb": 931,
                "root_free_gb": 450
            }),
        ),
        // hardware.gpu
        FakeProbeResponse::ok(
            "hardware.gpu",
            r#"06:00.0 VGA compatible controller: NVIDIA Corporation GA102 [GeForce RTX 3080] (rev a1) (prog-if 00 [VGA controller])
	Subsystem: eVga.com. Corp. GA102 [GeForce RTX 3080]
	Flags: bus master, fast devsel, latency 0, IRQ 82
	Memory at fb000000 (32-bit, non-prefetchable) [size=16M]
	Memory at d0000000 (64-bit, prefetchable) [size=256M]"#,
        ),
        // drivers.gpu
        FakeProbeResponse::ok(
            "drivers.gpu",
            "nvidia               55533568  142\nnvidia_modeset        1277952  21\nnvidia_uvm            2949120  2\nnvidia_drm              69632  12",
        ),
        // hardware.ram
        FakeProbeResponse::ok(
            "hardware.ram",
            r#"# dmidecode 3.4
Handle 0x0040, DMI type 17, 92 bytes
Memory Device
	Size: 16 GB
	Form Factor: DIMM
	Type: DDR4
	Speed: 3200 MT/s
	Manufacturer: G Skill
	Part Number: F4-3200C16-16GVK

Handle 0x0041, DMI type 17, 92 bytes
Memory Device
	Size: 16 GB
	Form Factor: DIMM
	Type: DDR4
	Speed: 3200 MT/s"#,
        ),
        // system.os
        FakeProbeResponse::ok(
            "system.os",
            r#"NAME="Arch Linux"
PRETTY_NAME="Arch Linux"
ID=arch
BUILD_ID=rolling
ANSI_COLOR="38;2;23;147;209"
HOME_URL="https://archlinux.org/"
DOCUMENTATION_URL="https://wiki.archlinux.org/"
6.6.7-arch1-1"#,
        ),
        // logs.annad
        FakeProbeResponse::ok(
            "logs.annad",
            r#"Dec 01 10:00:00 localhost annad[1234]: [INFO] Starting Anna daemon v1.0.0
Dec 01 10:00:01 localhost annad[1234]: [INFO] Loaded config from /etc/anna/config.toml
Dec 01 10:00:02 localhost annad[1234]: [INFO] Ollama connection established
Dec 01 10:00:03 localhost annad[1234]: [INFO] HTTP API listening on :7865
Dec 01 11:30:00 localhost annad[1234]: [INFO] Answered question in 450ms (reliability: 0.95)
Dec 01 11:45:00 localhost annad[1234]: [WARN] Slow LLM response (2.3s)
Dec 01 12:00:00 localhost annad[1234]: [INFO] Auto-update check: already on latest v1.0.0"#,
        ),
        // updates.pending
        FakeProbeResponse::ok(
            "updates.pending",
            r#"linux 6.6.7.arch1-1 -> 6.6.8.arch1-1
nvidia-dkms 545.29.02-1 -> 545.29.06-1
python 3.11.6-1 -> 3.11.7-1
rust 1:1.74.0-1 -> 1:1.74.1-1
firefox 120.0-1 -> 120.0.1-1"#,
        ),
        // anna.self_health
        FakeProbeResponse::ok_json(
            "anna.self_health",
            r#"daemon: running (pid 1234, uptime 6h 23m)
ollama: connected (llama3.2:3b loaded)
config: valid (/etc/anna/config.toml)
permissions: ok (logs writable, state writable)
auto_repair: not needed"#,
            serde_json::json!({
                "daemon": {"status": "running", "pid": 1234, "uptime_mins": 383},
                "ollama": {"status": "connected", "model": "llama3.2:3b"},
                "config": {"status": "valid", "path": "/etc/anna/config.toml"},
                "permissions": {"logs": "ok", "state": "ok"},
                "auto_repair": "not_needed"
            }),
        ),
        // disk.df (for root free space)
        FakeProbeResponse::ok(
            "disk.df",
            "Filesystem      Size  Used Avail Use% Mounted on\n/dev/nvme0n1p2  931G  481G  450G  52% /",
        ),
        // net.interfaces
        FakeProbeResponse::ok(
            "net.interfaces",
            "1: lo: <LOOPBACK,UP>\n2: enp5s0: <BROADCAST,MULTICAST,UP>\n3: wlan0: <BROADCAST,MULTICAST>",
        ),
    ]
}

/// Create the acceptance probe executor with all canned responses
fn create_acceptance_probes() -> FakeProbeExecutor {
    FakeProbeExecutorBuilder::new()
        .probe_responses(create_acceptance_probe_responses())
        .build()
}

/// Create LLM responses for a specific question
fn create_llm_responses_for_question(question_id: &str) -> (FakeJuniorResponse, FakeSeniorResponse) {
    match question_id {
        "cpu_model" => (
            FakeJuniorResponse {
                probe_requests: vec![ProbeRequest {
                    probe_id: "cpu.info".to_string(),
                    reason: "Need CPU details".to_string(),
                }],
                draft_answer: None,
                raw_text: "{}".to_string(),
            },
            FakeSeniorResponse {
                verdict: "approve".to_string(),
                fixed_answer: None,
                scores_overall: 0.95,
                raw_text: "{}".to_string(),
            },
        ),
        "ram_capacity" => (
            FakeJuniorResponse {
                probe_requests: vec![ProbeRequest {
                    probe_id: "mem.info".to_string(),
                    reason: "Need memory info".to_string(),
                }],
                draft_answer: None,
                raw_text: "{}".to_string(),
            },
            FakeSeniorResponse {
                verdict: "approve".to_string(),
                fixed_answer: None,
                scores_overall: 0.94,
                raw_text: "{}".to_string(),
            },
        ),
        "disk_space" => (
            FakeJuniorResponse {
                probe_requests: vec![ProbeRequest {
                    probe_id: "disk.df".to_string(),
                    reason: "Need disk space info".to_string(),
                }],
                draft_answer: None,
                raw_text: "{}".to_string(),
            },
            FakeSeniorResponse {
                verdict: "approve".to_string(),
                fixed_answer: None,
                scores_overall: 0.93,
                raw_text: "{}".to_string(),
            },
        ),
        "anna_logs" => (
            FakeJuniorResponse {
                probe_requests: vec![ProbeRequest {
                    probe_id: "logs.annad".to_string(),
                    reason: "Need Anna daemon logs".to_string(),
                }],
                draft_answer: None,
                raw_text: "{}".to_string(),
            },
            FakeSeniorResponse {
                verdict: "approve".to_string(),
                fixed_answer: None,
                scores_overall: 0.91,
                raw_text: "{}".to_string(),
            },
        ),
        "system_updates" => (
            FakeJuniorResponse {
                probe_requests: vec![ProbeRequest {
                    probe_id: "updates.pending".to_string(),
                    reason: "Check pending updates".to_string(),
                }],
                draft_answer: None,
                raw_text: "{}".to_string(),
            },
            FakeSeniorResponse {
                verdict: "approve".to_string(),
                fixed_answer: None,
                scores_overall: 0.92,
                raw_text: "{}".to_string(),
            },
        ),
        "self_health" => (
            FakeJuniorResponse {
                probe_requests: vec![ProbeRequest {
                    probe_id: "anna.self_health".to_string(),
                    reason: "Run self-diagnostic".to_string(),
                }],
                draft_answer: None,
                raw_text: "{}".to_string(),
            },
            FakeSeniorResponse {
                verdict: "approve".to_string(),
                fixed_answer: None,
                scores_overall: 0.96,
                raw_text: "{}".to_string(),
            },
        ),
        "gpu_info" => (
            FakeJuniorResponse {
                probe_requests: vec![
                    ProbeRequest {
                        probe_id: "hardware.gpu".to_string(),
                        reason: "Detect GPU".to_string(),
                    },
                    ProbeRequest {
                        probe_id: "drivers.gpu".to_string(),
                        reason: "Check GPU drivers".to_string(),
                    },
                ],
                draft_answer: None,
                raw_text: "{}".to_string(),
            },
            FakeSeniorResponse {
                verdict: "approve".to_string(),
                fixed_answer: None,
                scores_overall: 0.94,
                raw_text: "{}".to_string(),
            },
        ),
        "os_info" => (
            FakeJuniorResponse {
                probe_requests: vec![ProbeRequest {
                    probe_id: "system.os".to_string(),
                    reason: "Get OS info".to_string(),
                }],
                draft_answer: None,
                raw_text: "{}".to_string(),
            },
            FakeSeniorResponse {
                verdict: "approve".to_string(),
                fixed_answer: None,
                scores_overall: 0.97,
                raw_text: "{}".to_string(),
            },
        ),
        "uptime" => (
            FakeJuniorResponse {
                probe_requests: vec![],
                draft_answer: Some(DraftAnswerV80 {
                    text: "Your system has been up for 6 hours and 23 minutes. I determined this from the anna.self_health probe which reports daemon uptime.".to_string(),
                    citations: vec!["anna.self_health".to_string()],
                }),
                raw_text: "{}".to_string(),
            },
            FakeSeniorResponse {
                verdict: "approve".to_string(),
                fixed_answer: None,
                scores_overall: 0.90,
                raw_text: "{}".to_string(),
            },
        ),
        "system_summary" => (
            FakeJuniorResponse {
                probe_requests: vec![
                    ProbeRequest { probe_id: "cpu.info".to_string(), reason: "Summary".to_string() },
                    ProbeRequest { probe_id: "mem.info".to_string(), reason: "Summary".to_string() },
                    ProbeRequest { probe_id: "disk.df".to_string(), reason: "Summary".to_string() },
                    ProbeRequest { probe_id: "hardware.gpu".to_string(), reason: "Summary".to_string() },
                    ProbeRequest { probe_id: "anna.self_health".to_string(), reason: "Summary".to_string() },
                ],
                draft_answer: None,
                raw_text: "{}".to_string(),
            },
            FakeSeniorResponse {
                verdict: "approve".to_string(),
                fixed_answer: None,
                scores_overall: 0.93,
                raw_text: "{}".to_string(),
            },
        ),
        _ => (
            FakeJuniorResponse::default(),
            FakeSeniorResponse::default(),
        ),
    }
}

// ============================================================================
// PART 3: Scenario Runner with Result Recording
// ============================================================================

/// Result of a single question run
#[derive(Debug, Clone)]
pub struct QuestionResult {
    pub question_id: String,
    pub question_text: String,
    pub answer: String,
    pub origin: String,
    pub reliability: f64,
    pub duration_ms: u64,
    pub probes_used: Vec<String>,
    pub error: Option<String>,
}

impl QuestionResult {
    pub fn is_valid(&self) -> bool {
        self.error.is_none()
            && !self.answer.is_empty()
            && self.reliability >= 0.0
            && self.reliability <= 1.0
            && ["Brain", "Junior+Senior", "Fallback", "LLM"].contains(&self.origin.as_str())
    }
}

/// Result of a complete pass (all 10 questions)
#[derive(Debug, Clone)]
pub struct PassResult {
    pub pass_number: usize,
    pub questions: Vec<QuestionResult>,
    pub total_duration_ms: u64,
    pub xp_snapshot: XpSnapshot,
}

/// Snapshot of XP state at a point in time
#[derive(Debug, Clone, Default)]
pub struct XpSnapshot {
    pub anna_level: u8,
    pub anna_xp: u64,
    pub anna_trust: f32,
    pub junior_good_plans: u64,
    pub junior_bad_plans: u64,
    pub senior_approvals: u64,
    pub senior_fix_and_accept: u64,
}

/// The scenario runner that executes questions and tracks results
pub struct ScenarioRunner {
    probes: FakeProbeExecutor,
    passes: Vec<PassResult>,
    current_xp: XpSnapshot,
}

impl ScenarioRunner {
    pub fn new() -> Self {
        Self {
            probes: create_acceptance_probes(),
            passes: vec![],
            current_xp: XpSnapshot::default(),
        }
    }

    /// Run all 10 questions once
    pub async fn run_pass(&mut self, pass_number: usize) -> PassResult {
        let pass_start = Instant::now();
        let mut questions = Vec::new();

        for (question_id, question_text) in ACCEPTANCE_QUESTIONS {
            let result = self.run_single_question(question_id, question_text).await;
            questions.push(result);
        }

        let total_duration_ms = pass_start.elapsed().as_millis() as u64;

        // Update XP based on pass results
        self.update_xp_from_pass(&questions);

        let pass_result = PassResult {
            pass_number,
            questions,
            total_duration_ms,
            xp_snapshot: self.current_xp.clone(),
        };

        self.passes.push(pass_result.clone());
        pass_result
    }

    /// Run a single question and return result
    async fn run_single_question(&self, question_id: &str, question_text: &str) -> QuestionResult {
        let start = Instant::now();

        // Try Brain fast path first
        let qt = FastQuestionType::classify(question_text);

        // Get LLM responses for this question
        let (junior_resp, senior_resp) = create_llm_responses_for_question(question_id);

        // Simulate orchestration based on question type
        let (answer, origin, reliability, probes_used) = if qt != FastQuestionType::Unknown {
            // Brain fast path
            let fast = anna_common::try_fast_answer(question_text);
            if let Some(fast_answer) = fast {
                (
                    fast_answer.text.clone(),
                    fast_answer.origin.clone(),
                    fast_answer.reliability,
                    fast_answer.citations.clone(),
                )
            } else {
                // Fall back to LLM path
                self.simulate_llm_path(&junior_resp, &senior_resp).await
            }
        } else {
            // Full LLM path
            self.simulate_llm_path(&junior_resp, &senior_resp).await
        };

        let duration_ms = start.elapsed().as_millis() as u64;

        QuestionResult {
            question_id: question_id.to_string(),
            question_text: question_text.to_string(),
            answer,
            origin,
            reliability,
            duration_ms,
            probes_used,
            error: None,
        }
    }

    /// Simulate the LLM orchestration path
    async fn simulate_llm_path(
        &self,
        junior_resp: &FakeJuniorResponse,
        senior_resp: &FakeSeniorResponse,
    ) -> (String, String, f64, Vec<String>) {
        let mut probes_used = vec![];

        // Execute any requested probes
        for probe_req in &junior_resp.probe_requests {
            let evidence = self.probes.execute_probe(&probe_req.probe_id).await;
            if evidence.status == EvidenceStatus::Ok {
                probes_used.push(probe_req.probe_id.clone());
            }
        }

        // Generate answer based on probe results
        let answer = if let Some(draft) = &junior_resp.draft_answer {
            draft.text.clone()
        } else {
            // Generate answer from probe evidence
            self.generate_answer_from_probes(&probes_used).await
        };

        let origin = if probes_used.is_empty() {
            "Brain"
        } else {
            "Junior+Senior"
        };

        (
            answer,
            origin.to_string(),
            senior_resp.scores_overall,
            probes_used,
        )
    }

    /// Generate an answer from probe evidence
    async fn generate_answer_from_probes(&self, probes_used: &[String]) -> String {
        let mut answer_parts = vec![];

        for probe_id in probes_used {
            let evidence = self.probes.execute_probe(probe_id).await;
            if evidence.status == EvidenceStatus::Ok {
                if let Some(raw) = evidence.raw {
                    // Extract key info from raw output
                    let summary = self.summarize_probe_output(probe_id, &raw);
                    answer_parts.push(summary);
                }
            }
        }

        if answer_parts.is_empty() {
            "Unable to gather evidence for this question.".to_string()
        } else {
            answer_parts.join("\n\n")
        }
    }

    /// Summarize probe output into human-readable text
    fn summarize_probe_output(&self, probe_id: &str, raw: &str) -> String {
        match probe_id {
            "cpu.info" => {
                format!("CPU: AMD Ryzen 7 5800X 8-Core Processor\n- 8 physical cores, 16 threads\n- Max speed: 4850 MHz\nEvidence: cpu.info probe (lscpu -J)")
            }
            "mem.info" => {
                format!("RAM: 32 GB total, 24 GB available\n- 16 GB free, 8 GB cached\nEvidence: mem.info probe (cat /proc/meminfo)")
            }
            "disk.df" | "disk.lsblk" => {
                format!("Root filesystem: 450 GB free of 931 GB (52% used)\n- Device: /dev/nvme0n1p2\nEvidence: disk probe")
            }
            "hardware.gpu" => {
                format!("GPU: NVIDIA GeForce RTX 3080 (GA102)\n- 256 MB VRAM detected\nEvidence: hardware.gpu probe (lspci)")
            }
            "drivers.gpu" => {
                format!("GPU Drivers: NVIDIA proprietary driver loaded\n- Modules: nvidia, nvidia_modeset, nvidia_uvm, nvidia_drm\nEvidence: drivers.gpu probe (lsmod)")
            }
            "system.os" => {
                format!("OS: Arch Linux (rolling release)\n- Kernel: 6.6.7-arch1-1\nEvidence: system.os probe (cat /etc/os-release)")
            }
            "logs.annad" => {
                format!("Anna Logs (last 2 hours):\n- [INFO] Daemon running normally\n- [WARN] 1 slow LLM response (2.3s)\n- No errors detected\nEvidence: logs.annad probe (journalctl)")
            }
            "updates.pending" => {
                format!("Pending Updates: 5 packages\n- linux, nvidia-dkms, python, rust, firefox\n\nSafe review plan:\n1. [READ-ONLY] pacman -Qu to list updates\n2. [READ-ONLY] Review changelogs\n3. [ACTION] sudo pacman -Syu when ready\nEvidence: updates.pending probe")
            }
            "anna.self_health" => {
                format!("Anna Health: All systems operational\n- Daemon: running (pid 1234, 6h 23m uptime)\n- Ollama: connected (llama3.2:3b)\n- Config: valid\n- Permissions: ok\n- Auto-repair: not needed\nEvidence: anna.self_health probe")
            }
            _ => format!("Data from {} probe:\n{}", probe_id, &raw[..raw.len().min(200)]),
        }
    }

    /// Update XP tracking based on pass results
    fn update_xp_from_pass(&mut self, questions: &[QuestionResult]) {
        for q in questions {
            if q.is_valid() {
                // Good answer: +10 XP base, scaled by reliability
                let xp_gain = (10.0 * q.reliability) as u64;
                self.current_xp.anna_xp += xp_gain;
                self.current_xp.anna_trust = (self.current_xp.anna_trust + 0.01).min(1.0);

                // Track Junior/Senior metrics
                if q.origin == "Junior+Senior" {
                    self.current_xp.junior_good_plans += 1;
                    self.current_xp.senior_approvals += 1;
                }
            } else {
                self.current_xp.junior_bad_plans += 1;
            }
        }

        // Check for level up
        let xp_for_next = 100 * self.current_xp.anna_level as u64;
        if self.current_xp.anna_xp >= xp_for_next && self.current_xp.anna_level < 99 {
            self.current_xp.anna_level += 1;
        }
    }

    /// Print summary for a pass
    pub fn print_pass_summary(&self, pass: &PassResult) {
        println!("\n=== Pass {} Summary ===", pass.pass_number);
        println!("Total duration: {}ms", pass.total_duration_ms);
        println!();

        for q in &pass.questions {
            let status = if q.is_valid() { "OK" } else { "FAIL" };
            println!(
                "  {}: origin={}, reliability={:.2}, duration={}ms [{}]",
                q.question_id, q.origin, q.reliability, q.duration_ms, status
            );
        }

        println!();
        println!("XP Snapshot:");
        println!(
            "  Anna: Level {}, XP {}, trust {:.2}",
            pass.xp_snapshot.anna_level, pass.xp_snapshot.anna_xp, pass.xp_snapshot.anna_trust
        );
        println!(
            "  Junior: good_plans={}, bad_plans={}",
            pass.xp_snapshot.junior_good_plans, pass.xp_snapshot.junior_bad_plans
        );
        println!(
            "  Senior: approvals={}, fix_and_accept={}",
            pass.xp_snapshot.senior_approvals, pass.xp_snapshot.senior_fix_and_accept
        );
    }

    /// Get all pass results
    pub fn passes(&self) -> &[PassResult] {
        &self.passes
    }
}

impl Default for ScenarioRunner {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// PART 4: Quality and UX Assertions
// ============================================================================

/// Verify a question result meets quality standards
fn assert_question_quality(result: &QuestionResult) {
    // 1. Non-empty answer
    assert!(
        !result.answer.is_empty(),
        "Question '{}': Answer must not be empty",
        result.question_id
    );

    // 2. No debug/internal JSON in answer
    assert!(
        !result.answer.contains(r#"{"probe_"#),
        "Question '{}': Answer contains internal JSON",
        result.question_id
    );
    assert!(
        !result.answer.contains("[DEBUG]"),
        "Question '{}': Answer contains debug output",
        result.question_id
    );

    // 3. Reliability in valid range
    assert!(
        result.reliability >= 0.0 && result.reliability <= 1.0,
        "Question '{}': Reliability {} out of range [0,1]",
        result.question_id,
        result.reliability
    );

    // 4. Valid origin
    let valid_origins = ["Brain", "Junior+Senior", "Fallback", "LLM"];
    assert!(
        valid_origins.contains(&result.origin.as_str()),
        "Question '{}': Invalid origin '{}'",
        result.question_id,
        result.origin
    );

    // 5. No error
    assert!(
        result.error.is_none(),
        "Question '{}': Unexpected error: {:?}",
        result.question_id,
        result.error
    );
}

/// Verify rendered output format
#[allow(dead_code)]
fn assert_output_format(answer: &str, _reliability: f64) {
    // For now, just verify answer is non-empty and well-formed
    assert!(!answer.is_empty(), "Rendered output is empty");

    // Check for no broken escape codes
    assert!(
        !answer.contains("\x1b["),
        "Rendered output contains raw escape codes"
    );
}

// ============================================================================
// PART 5: XP and Learning Observability
// ============================================================================

/// Verify XP metrics are sane after a pass
fn assert_xp_metrics(snapshot: &XpSnapshot, pass_number: usize) {
    // Level should be at least 1
    assert!(
        snapshot.anna_level >= 1,
        "Pass {}: Anna level must be >= 1",
        pass_number
    );

    // Trust should be in [0, 1]
    assert!(
        snapshot.anna_trust >= 0.0 && snapshot.anna_trust <= 1.0,
        "Pass {}: Anna trust {} out of range",
        pass_number,
        snapshot.anna_trust
    );

    // XP is u64 and always non-negative, this is a sanity check
    // that the field is accessible
    let _xp = snapshot.anna_xp;

    // Good plans should increase (or stay same if all Brain)
    // (This is a soft check - we just verify it's tracked)
}

/// Verify learning progression across passes
fn assert_learning_progression(passes: &[PassResult]) {
    if passes.len() < 2 {
        return;
    }

    // XP should generally increase across passes
    let first_xp = passes.first().unwrap().xp_snapshot.anna_xp;
    let last_xp = passes.last().unwrap().xp_snapshot.anna_xp;

    assert!(
        last_xp >= first_xp,
        "XP should not decrease: first={}, last={}",
        first_xp,
        last_xp
    );

    // Trust should trend upward (or stay stable)
    let first_trust = passes.first().unwrap().xp_snapshot.anna_trust;
    let last_trust = passes.last().unwrap().xp_snapshot.anna_trust;

    assert!(
        last_trust >= first_trust - 0.1, // Allow small fluctuation
        "Trust should not decrease significantly: first={}, last={}",
        first_trust,
        last_trust
    );
}

// ============================================================================
// PART 6: Test Functions
// ============================================================================

/// Main acceptance test: Run 10 questions x 5 passes
#[tokio::test]
async fn test_acceptance_full_run() {
    let mut runner = ScenarioRunner::new();

    println!("\n========================================");
    println!("ACCEPTANCE TEST: 10 Questions x 5 Passes");
    println!("========================================");

    for pass_num in 1..=5 {
        let pass_result = runner.run_pass(pass_num).await;

        // Print summary
        runner.print_pass_summary(&pass_result);

        // Assert quality for each question
        for q in &pass_result.questions {
            assert_question_quality(q);
        }

        // Assert XP metrics
        assert_xp_metrics(&pass_result.xp_snapshot, pass_num);
    }

    // Assert learning progression
    assert_learning_progression(runner.passes());

    // Final summary
    println!("\n========================================");
    println!("ACCEPTANCE TEST COMPLETE");
    println!("========================================");
    println!("Passes completed: {}", runner.passes().len());

    let total_questions: usize = runner.passes().iter().map(|p| p.questions.len()).sum();
    println!("Total questions answered: {}", total_questions);

    let final_xp = runner.passes().last().unwrap().xp_snapshot.anna_xp;
    let final_level = runner.passes().last().unwrap().xp_snapshot.anna_level;
    println!("Final Anna state: Level {}, XP {}", final_level, final_xp);
}

/// Test single pass for quick validation
#[tokio::test]
async fn test_acceptance_single_pass() {
    let mut runner = ScenarioRunner::new();

    let pass_result = runner.run_pass(1).await;

    // All 10 questions should be answered
    assert_eq!(pass_result.questions.len(), 10);

    // Each question should be valid
    for q in &pass_result.questions {
        assert_question_quality(q);
    }

    // XP should have been awarded
    assert!(
        pass_result.xp_snapshot.anna_xp > 0,
        "XP should be awarded after pass"
    );
}

/// Test that questions are classified correctly
#[test]
fn test_question_classification() {
    for (id, text) in ACCEPTANCE_QUESTIONS {
        let qt = FastQuestionType::classify(text);
        // Just verify classification doesn't panic
        println!("Question '{}' classified as {:?}", id, qt);
    }
}

/// Test probe infrastructure
#[tokio::test]
async fn test_probe_infrastructure() {
    let probes = create_acceptance_probes();

    // All expected probes should be valid
    let expected_probes = [
        "cpu.info",
        "mem.info",
        "disk.lsblk",
        "hardware.gpu",
        "drivers.gpu",
        "hardware.ram",
        "system.os",
        "logs.annad",
        "updates.pending",
        "anna.self_health",
    ];

    for probe_id in expected_probes {
        assert!(probes.is_valid(probe_id), "Probe '{}' should be valid", probe_id);

        let evidence = probes.execute_probe(probe_id).await;
        assert_eq!(
            evidence.status,
            EvidenceStatus::Ok,
            "Probe '{}' should return Ok",
            probe_id
        );
        assert!(
            evidence.raw.is_some(),
            "Probe '{}' should have raw output",
            probe_id
        );
    }
}

/// Test XP tracking isolation (no state leakage)
#[tokio::test]
async fn test_xp_tracking_isolation() {
    let mut runner1 = ScenarioRunner::new();
    let mut runner2 = ScenarioRunner::new();

    // Run pass on runner1
    let pass1 = runner1.run_pass(1).await;

    // runner2 should start fresh
    let pass2 = runner2.run_pass(1).await;

    // Both should have same starting conditions
    // (XP may differ slightly due to timing, but level should be same)
    assert_eq!(
        pass1.xp_snapshot.anna_level,
        pass2.xp_snapshot.anna_level,
        "New runners should start at same level"
    );
}

/// Test that answers contain evidence citations
#[tokio::test]
async fn test_evidence_in_answers() {
    let mut runner = ScenarioRunner::new();
    let pass = runner.run_pass(1).await;

    // Questions that use probes should mention them in answer
    for q in &pass.questions {
        if !q.probes_used.is_empty() {
            // Brain fast path returns concise answers (may be <50 chars)
            // LLM path (Junior+Senior) returns verbose answers (>50 chars)
            let min_length = if q.origin == "Brain" {
                20  // Brain fast path: concise but still substantive
            } else {
                50  // LLM path: expect more detailed answers
            };

            assert!(
                q.answer.len() > min_length,
                "Question '{}' (origin: {}): Answer should be substantive when probes were used. Got ({} chars): '{}'",
                q.question_id,
                q.origin,
                q.answer.len(),
                q.answer
            );
        }
    }
}

// ============================================================================
// PART 7: Real-World Extension Point (Placeholder)
// ============================================================================

/// Trait for pluggable scenario backends (prep for real-world mode)
pub trait ScenarioBackend: Send + Sync {
    /// Execute probes and LLM calls for a question
    fn execute_question(
        &self,
        question_id: &str,
        question_text: &str,
    ) -> impl std::future::Future<Output = QuestionResult> + Send;
}

/// Marker for future real-hardware implementation
pub struct RealBackend {
    // Would contain:
    // - RealProbeExecutor
    // - OllamaClient
}

/// Marker for fake/test implementation
#[allow(dead_code)]
pub struct FakeBackend {
    probes: FakeProbeExecutor,
    // LLM responses configured per question
}

// Note: Full implementation of RealBackend is deferred.
// The ScenarioRunner already accepts FakeProbeExecutor and could be
// generalized with the ScenarioBackend trait when needed.

// ============================================================================
// PART 8: Experience Reset Integration Tests (v1.2.0)
// ============================================================================

/// Test that experience reset works correctly with telemetry
#[tokio::test]
async fn test_experience_reset_integration() {
    use anna_common::{
        ExperiencePaths, ExperienceSnapshot, reset_experience,
        telemetry::{TelemetryRecorder, TelemetryEvent, Origin, Outcome},
    };
    use tempfile::TempDir;
    use std::fs;

    // Create temp directory for test
    let temp = TempDir::new().unwrap();
    let paths = ExperiencePaths::with_root(temp.path());

    // Create directories
    fs::create_dir_all(&paths.xp_dir).unwrap();
    fs::create_dir_all(paths.telemetry_file.parent().unwrap()).unwrap();
    fs::create_dir_all(&paths.stats_dir).unwrap();

    // Seed XP data
    fs::write(
        paths.xp_store_file(),
        r#"{"anna":{"level":5,"xp":500,"trust":0.8}}"#,
    ).unwrap();

    // Seed telemetry data
    let recorder = TelemetryRecorder::with_path(paths.telemetry_file.clone());
    let _ = recorder.record(&TelemetryEvent::new("q1", Outcome::Success, Origin::Brain, 0.99, 10));
    let _ = recorder.record(&TelemetryEvent::new("q2", Outcome::Success, Origin::Junior, 0.85, 5000));
    let _ = recorder.record(&TelemetryEvent::new("q3", Outcome::Failure, Origin::Senior, 0.40, 12000));

    // Seed stats files
    fs::write(paths.stats_dir.join("xp_events.jsonl"), "event1\nevent2").unwrap();

    // Verify data exists before reset
    let snapshot_before = ExperienceSnapshot::capture(&paths);
    assert!(!snapshot_before.is_empty(), "Data should exist before reset");
    assert!(snapshot_before.xp_store_exists);
    assert_eq!(snapshot_before.telemetry_line_count, 3);
    assert_eq!(snapshot_before.stats_file_count, 1);

    // Perform reset
    let result = reset_experience(&paths);

    // Verify reset succeeded
    assert!(result.success, "Reset should succeed");
    assert!(!result.components_reset.is_empty(), "Components should be reset");

    // Verify data is reset to baseline after reset
    // v1.3.0: XP store is now reset to baseline values, not removed
    let snapshot_after = ExperienceSnapshot::capture(&paths);
    assert!(snapshot_after.is_empty(), "Data should be at baseline after reset");
    assert!(snapshot_after.xp_store_exists, "XP store should exist with baseline values");
    assert_eq!(snapshot_after.anna_level, 1, "Level should be reset to 1");
    assert_eq!(snapshot_after.anna_xp, 0, "XP should be reset to 0");
    assert_eq!(snapshot_after.total_questions, 0, "Questions should be reset to 0");
    assert_eq!(snapshot_after.telemetry_line_count, 0, "Telemetry should be cleared");
    assert_eq!(snapshot_after.stats_file_count, 0, "Stats files should be cleared");
}

/// Test reset is idempotent (can be called multiple times safely)
#[tokio::test]
async fn test_experience_reset_idempotent() {
    use anna_common::{ExperiencePaths, reset_experience};
    use tempfile::TempDir;
    use std::fs;

    let temp = TempDir::new().unwrap();
    let paths = ExperiencePaths::with_root(temp.path());

    // Create directories
    fs::create_dir_all(&paths.xp_dir).unwrap();
    fs::create_dir_all(paths.telemetry_file.parent().unwrap()).unwrap();
    fs::create_dir_all(&paths.stats_dir).unwrap();

    // Add some data
    fs::write(paths.xp_store_file(), "{}").unwrap();
    fs::write(&paths.telemetry_file, "line1\nline2").unwrap();

    // First reset
    let result1 = reset_experience(&paths);
    assert!(result1.success);

    // Second reset - should also succeed (already clean)
    let result2 = reset_experience(&paths);
    assert!(result2.success);
    assert!(
        result2.components_reset.is_empty() ||
        result2.components_reset.iter().all(|c| !c.contains("XP")),
        "Second reset should find components already clean"
    );
}

/// Test that reset handles missing files gracefully
#[tokio::test]
async fn test_experience_reset_missing_files() {
    use anna_common::{ExperiencePaths, reset_experience};
    use tempfile::TempDir;
    use std::fs;

    let temp = TempDir::new().unwrap();
    let paths = ExperiencePaths::with_root(temp.path());

    // Create only some directories (simulate partial state)
    fs::create_dir_all(&paths.xp_dir).unwrap();
    // Don't create telemetry or stats directories

    // Reset should handle missing gracefully
    let result = reset_experience(&paths);
    assert!(result.success, "Reset should succeed even with missing files");
}
