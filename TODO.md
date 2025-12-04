# Anna Assistant - Implementation Roadmap

**Current Version: 0.0.73**

This roadmap migrates from the v7.42.5 snapshot-based architecture to the full natural language assistant while preserving performance.

---

## Phase 1: CLI Surface Lockdown (0.0.x)

### 0.0.74 - Multi-Doctor Handoff + Senior Escalation (NEXT)
- [ ] Multi-doctor case handoff for complex issues
- [ ] Department collaboration for cross-domain problems
- [ ] Senior escalation when Junior < 50%
- [ ] Multi-round improvement loops
- [ ] Evidence bundle aggregation across departments

### 0.0.73 - Human Transcript Realism + Auto-Update Rewrite (COMPLETED)
- [x] Role-based phrasing per department in humanizer/phrases.rs
- [x] Service Desk: "I'm triaging the request and deciding who should handle it."
- [x] Network: "Looking at link state and active connections."
- [x] Storage: "Checking disk and filesystem status."
- [x] Performance: "Checking the latest hardware and load snapshot."
- [x] Doctor selection shows ownership: "Network team is taking this case."
- [x] Doctor selection shows first check (1 line, no tool names)
- [x] Evidence labels with source context: "CPU model (from hardware snapshot)"
- [x] HumanEvidenceLabel with topic + source formatting
- [x] humanize_doctor_selection() returns ownership + first check messages
- [x] Auto-update state machine in updater module (11 steps)
- [x] UpdateStep enum: AcquireLock, CheckRemote, CompareVersions, DownloadAssets, VerifyAssets, InstallCli, InstallDaemon, RestartDaemon, Healthcheck, ReleaseLock, Rollback
- [x] Hard filesystem locking with stale lock recovery
- [x] LockInfo with pid, timestamp, hostname, step
- [x] Atomic installs: download to temp, verify, rename in place
- [x] Binary backups in /var/lib/anna/internal/backups/
- [x] Automatic rollback on restart failure or healthcheck failure
- [x] Version mismatch detection (CLI vs daemon)
- [x] annactl status shows version mismatch warning
- [x] UpdateStateV73 with complete update lifecycle tracking
- [x] Ops log integration for all update steps
- [x] 49 new tests (11 updater + 38 humanizer)

### 0.0.72 - Dual Transcript Mode (COMPLETED)
- [x] Human mode (default): Clean IT department dialogue, no tool names/evidence IDs/raw commands
- [x] Debug mode: Full internals with canonical translator output, evidence IDs, timing, parse warnings
- [x] Both modes from SAME event stream (cannot diverge)
- [x] Enable debug via `/etc/anna/config.toml` with `transcript_mode = "debug"`
- [x] Enable debug via `ANNA_UI_TRANSCRIPT_MODE=debug` env var
- [x] Enable debug via `ANNA_DEBUG_TRANSCRIPT=1` shorthand (for tests)
- [x] No new public CLI commands or flags
- [x] Unified event types: TranscriptEventV72, EventDataV72, RoleV72, ToneV72
- [x] Dual renderers: render_human_v72(), render_debug_v72()
- [x] Forbidden pattern validation: FORBIDDEN_HUMAN_PATTERNS, FORBIDDEN_HUMAN_LITERALS
- [x] validate_human_output() asserts no leakage of internals
- [x] validate_debug_has_internals() asserts debug mode has expected details
- [x] Humanized equivalents: "Translator struggled; we used house rules" instead of "deterministic fallback"
- [x] Confirmation prompts humanized but exact confirm_phrase unchanged
- [x] Reliability score in both modes: "85% (direct evidence)"
- [x] Per-participant spinner/working indicators
- [x] Deep test harness: run_dual_mode_tests() with transcripts-human/ and transcripts-debug/
- [x] 26 transcript_v072 tests

### 0.0.71 - Real IT Department Humanizer (COMPLETED)
- [x] Role/tone metadata: StaffRole enum (ServiceDesk, Department, Anna, Translator, Junior, Senior)
- [x] MessageTone enum with confidence-based selection (Brisk, Neutral, Helpful, Skeptical)
- [x] ConfidenceHint enum derived from scores (Low, Medium, High)
- [x] DepartmentTag enum for standard transcript tags
- [x] Humanizer layer transforms events to natural language (humanize_*)
- [x] HumanizedMessage struct with tag, text, tone, is_side_thread
- [x] HumanizerContext tracks confidence, evidence_missing, complexity, parse warnings
- [x] Micro-threads: ThreadBuilder with indented side threads for evidence gathering
- [x] ThreadedTranscript with main thread + side thread rendering
- [x] Shorter human labels (no "snapshot" suffix): "hardware inventory", "network link and routing signals"
- [x] HumanLabel enum with from_legacy() for backwards compatibility
- [x] EvidenceSummary generators for CPU, memory, disk, network, service, boot, audio
- [x] validate_answer_relevance() prevents wrong-topic answers (memory→CPU, disk→CPU)
- [x] AnswerValidation enum: Ok, WrongTopic, TooGeneric, MissingEvidence
- [x] Junior critique phrasing: "didn't ground in X" instead of evidence IDs
- [x] Allowed realism (uncertainty expressions) vs NOT allowed (fabrication)
- [x] 29 humanizer tests + misclassification guardrail tests in case_coordinator

### 0.0.70 - Dual Transcript Renderer (COMPLETED)
- [x] Human Mode: IT department dialogue without tool names, evidence IDs, or raw commands
- [x] Human Mode: Topic abstractions ("hardware inventory snapshot", "network status snapshot")
- [x] Debug Mode: Full transparency with canonical translator output (6-line format)
- [x] Debug Mode: Tool names, evidence IDs, timing, parse warnings, retries, fallbacks
- [x] Modular transcript_v070 system: events.rs, topics.rs, render.rs, colored.rs, validation.rs
- [x] EvidenceTopicV70 enum with 16 topic categories + Custom variant
- [x] tool_to_evidence_topic() maps tool names to human-readable topics
- [x] ActorV70: You, ServiceDesk, Networking, Storage, Boot, Audio, Graphics, Security, Performance, InfoDesk + internal (Translator, Junior, Senior, Annad)
- [x] EventV70: UserToAnna, StaffMessage, Evidence, ToolCall, ToolResult, ParseWarning, TranslatorCanonical, Reliability, Perf, Retry, FinalAnswer, Phase, Working
- [x] TranscriptStreamV70 with TranscriptStatsV70 tracking (parse warnings, retries, fallbacks, tool calls)
- [x] render_human() / render_debug() for text output; print_human_colored() / print_debug_colored() for terminal
- [x] FORBIDDEN_HUMAN validation prevents tool names, evidence IDs, raw commands in human output
- [x] annactl status shows transcript mode in [CASES] section
- [x] Toggle via ANNA_UI_TRANSCRIPT_MODE env var or --debug flag
- [x] 11 unit tests: topic mapping, actor visibility, event stats, human/debug separation, validation
- [x] All files under 400 lines per CLAUDE.md requirement

### 0.0.69 - Service Desk Case Coordinator (COMPLETED)
- [x] CaseCoordinator: open_case, triage, dispatch, merge_reports, compose_user_answer, log
- [x] DepartmentReport: summary_human, findings, evidence_topics, confidence, recommended_next_steps, action_plan, policy_notes
- [x] RequestIntent enum: SystemQuery, ProblemReport, ActionRequest
- [x] classify_intent() with hard rules (queries start with "how much", "what is" = SystemQuery)
- [x] Fix wrong targeting: memory queries no longer become ActionRequest, disk queries get DiskFree not CPU info
- [x] get_evidence_topics_for_target() mapping table: memory→MemoryInfo, disk→DiskFree, network→NetworkStatus, etc.
- [x] TriageDecision: primary department + up to 2 supporting departments
- [x] Multi-dept detection for compound queries ("wifi disconnecting and sound crackling")
- [x] ConsolidatedAssessment: merge multiple department reports with weighted confidence
- [x] EvidenceTopicSummary: topic title, summary_human, raw_ref for department reports
- [x] ActionPlan: steps_human, risk, confirmation_phrase, rollback_plan
- [x] Human Mode transcript: "[service-desk] Opening case", "[networking] Network is stable"
- [x] Debug Mode transcript: timestamps, case_id, triage rationale, report details
- [x] compose_user_answer() with reliability footer ("84% good evidence coverage")
- [x] 9 unit tests: intent classification, evidence mapping, triage, multi-dept, transcript validation

### 0.0.68 - Two-Layer Transcript Renderer (COMPLETED)
- [x] TranscriptMode: Human (default) vs Debug with full separation
- [x] TranscriptEventV2: Structured events with dual summaries (summary_human, summary_debug)
- [x] TranscriptRole: Visible roles (departments) vs internal (translator/junior/annad)
- [x] Human Mode: No evidence IDs ([E1]), no tool names, no raw commands
- [x] Human Mode: Topic-based evidence display ("Link State: WiFi is connected")
- [x] Debug Mode: Full fidelity (exact prompts, tool names, evidence IDs, timings)
- [x] Humanized reliability line: "Reliability: 84% (good evidence coverage)"
- [x] TranscriptStreamV2: Event collector with coverage and reliability tracking
- [x] render_human() / render_debug() / render() for mode-based rendering
- [x] print_human_colored() for terminal output with role colors
- [x] write_transcripts() writes both human.log and debug.log to case directory
- [x] validate_human_output() checks for forbidden terms (tool names, evidence IDs)
- [x] 5 unit tests: human mode no evidence IDs, debug mode has IDs, internal roles hidden, reliability line, validation

### 0.0.67 - Department Evidence Playbooks v1 (COMPLETED)
- [x] PlaybookTopic struct: id, title, why_it_matters, tool_steps, required
- [x] PlaybookEvidence struct: topic_id, summary_human, summary_debug, raw_refs, success, duration_ms
- [x] PlaybookBundle struct: items, coverage_score, missing_topics, department
- [x] NetworkCauseCategory enum: Link, Dhcp, Dns, ManagerConflict, DriverFirmware, Unknown
- [x] StorageRiskLevel enum (playbook): None, Low, Medium, High
- [x] NetworkingDiagnosis and StorageDiagnosis result types
- [x] ActionProposal struct: id, title, risk, confirmation_phrase, rollback_plan_human, steps_human, steps_debug, affected_*
- [x] ActionProposal builders: read_only(), low_risk(), medium_risk(), high_risk() with appropriate confirmation gates
- [x] networking_actions module: restart_networkmanager, restart_iwd, reconnect_wifi, flush_dns_cache, switch_to_iwd, install_helper
- [x] storage_actions module: check_filesystem, btrfs_scrub_start, btrfs_balance_start, clean_package_cache, create_btrfs_snapshot, dangerous_btrfs_check_repair
- [x] Networking Playbook: collect_link_evidence, collect_addr_route_evidence, collect_dns_evidence, collect_manager_evidence, collect_errors_evidence
- [x] Storage Playbook: collect_mount_evidence, collect_btrfs_evidence, collect_smart_evidence, collect_io_errors_evidence, collect_fstab_evidence
- [x] run_networking_playbook() and run_storage_playbook() for full diagnosis flows
- [x] classify_network_cause() for root cause classification
- [x] Human Mode: summary_human shows topics and findings without tool names
- [x] Debug Mode: summary_debug shows commands and technical details
- [x] 19 unit tests for playbook framework, action proposals, and evidence collection

### 0.0.66 - Service Desk Dispatcher + Department Protocol (COMPLETED)
- [x] CaseFile schema extended: ticket_type, outcome, human_transcript, evidence_summaries, debug_trace_path
- [x] TicketType enum: Question, Incident, ChangeRequest
- [x] CaseOutcome enum: Pending, Answered, NeedsConfirmation, BlockedByPolicy, InsufficientEvidence, Abandoned
- [x] Case folder structure: /var/lib/anna/cases/<case_id>/ with case.json, human.log, debug.log
- [x] Department Protocol: DepartmentTrait, DepartmentName, RoutingDecision, WorkOrder
- [x] DepartmentResult: findings, evidence_bundle, recommended_actions, reliability_hint, needs_escalation
- [x] Service Desk: detect_ticket_type(), create_work_order(), create_routing_decision()
- [x] Department tone tags: ServiceDesk (terse), Networking (investigative), Storage (cautious), Audio (practical), Graphics (driver-aware), Boot (timeline-focused)
- [x] ActorVoice enum extended with department voices
- [x] get_investigation_message(), get_summary_prefix(), format_department_message()
- [x] Config: ui.show_spinner flag
- [x] Multi-department escalation support (max 2 departments)
- [x] WorkOrder.can_escalate() and record_escalation()
- [x] EvidenceBundle.empty() method for department protocol
- [x] Integration tests: network/disk routing, human mode no evidence IDs, debug mode has details, department tone visible

### 0.0.65 - Evidence Topics v1 + Answer Shaping (COMPLETED)
- [x] Typed evidence system: EvidenceRecord, EvidenceBundle, EvidenceSchema
- [x] ProbeKind enum (Passive vs Active) for read-only vs traffic-generating probes
- [x] Evidence Router: route_evidence() maps queries to correct topics
- [x] tool_satisfies_topic() prevents hw/sw_snapshot_summary from satisfying specific queries
- [x] get_tool_for_topic() returns correct tool for each topic
- [x] AnswerShaper: shape_answer() generates topic-appropriate responses
- [x] ShapedAnswer with confidence_notes for audio/network (no overclaiming "working")
- [x] Junior verification: verify_answer_with_topic() + check_tool_relevance()
- [x] IRRELEVANT_TOOL_PENALTY (-40%) caps at 45% for wrong tools
- [x] format_human_answer() / format_debug_answer() for output modes
- [x] Integration tests: disk/kernel/network/audio queries must return relevant info
- [x] Regression test: disk query must NOT contain CPU model

### 0.0.64 - Service Desk Dispatcher + Doctor Lifecycle (COMPLETED)
- [x] Service Desk dispatcher with Ticket, RoutingPlan, HumanNarrationPlan
- [x] Every request gets a Ticket (A-YYYYMMDD-XXXX) with category and severity
- [x] TicketSeverity: Low, Medium, High, Critical (based on problem keywords)
- [x] TicketCategory: Networking, Storage, Audio, Boot, Graphics, Security, Performance, Services, Packages, General
- [x] dispatch_request() runs FIRST, creates Ticket and determines routing
- [x] Problem reports route to Doctors, informational queries use Evidence Topic Router
- [x] is_problem_report() detects problem vs informational queries
- [x] Doctor trait enhanced with explicit lifecycle stages (Intake, EvidenceGathering, Diagnosis, Planning, Verification, HandOff, Complete)
- [x] DoctorLifecycleStage enum, IntakeResult struct, DoctorLifecycleState tracker
- [x] intake(), evidence_topics(), human_dialogue(), domain_label(), checking_what() methods
- [x] Narrator events: TicketOpened, RoutingDecision, DoctorStage
- [x] Human Mode narration describes tickets, routing, and doctor actions
- [x] Debug Mode shows ticket_id, category, severity, stage details
- [x] CaseFileV2 extended with ticket and routing fields (ticket_id, ticket_category, ticket_severity, primary_doctor, secondary_doctors, evidence_topics, doctor_stages_completed, used_doctor_flow)
- [x] Pipeline integration: dispatch before translator, narrate ticket/routing
- [x] Integration tests for dispatch, problem detection, severity heuristics

### 0.0.63 - Deterministic Evidence Router + Strict Validation (COMPLETED)
- [x] Deterministic topic router: detect_topic() runs BEFORE LLM
- [x] Topic-based tool routing in classify_intent_deterministic()
- [x] generate_tool_plan_from_topic() for deterministic tool selection
- [x] Strict answer validation with 40% reliability cap for mismatched answers
- [x] cap_reliability() function enforces max score for wrong answers
- [x] Evidence freshness tracking (calculate_freshness_penalty)
- [x] with_evidence_freshness() updates validation with age penalty
- [x] Human Mode narration for evidence collection (topic_evidence_narration)
- [x] audio_status tool label added to human_labels registry
- [x] Network/audio pattern detection improved (wifi working, pipewire running)
- [x] Integration tests for 5 common questions: memory, disk, kernel, network, audio
- [x] TopicValidation extended with evidence_age_secs, freshness_penalty

### 0.0.62 - Human Mode vs Debug Mode (COMPLETED)
- [x] Narrator component (narrator.rs) for human-readable IT department dialogue
- [x] Human Mode (default): No tool names, evidence IDs, or raw prompts
- [x] Debug Mode (--debug or ANNA_DEBUG=1): Full internal details
- [x] IT department tone: Anna calm/competent, Translator brisk, Junior skeptical
- [x] ActorVoice enum for consistent role presentation
- [x] NarratorEvent enum for all transcript events
- [x] Evidence displayed by topic description, not by ID
- [x] Honest narration of fallbacks and retries
- [x] --debug CLI flag support
- [x] ANNA_DEBUG env var support
- [x] Status page shows mode and toggle instructions
- [x] Integration tests for Human/Debug mode contracts

### 0.0.61 - Evidence Topics (Targeted Answers) (COMPLETED)
- [x] EvidenceTopic enum with 13 topics (CpuInfo, MemoryInfo, KernelVersion, DiskFree, NetworkStatus, AudioStatus, ServiceState, RecentErrors, BootTime, PackagesChanged, GraphicsStatus, Alerts, Unknown)
- [x] TopicConfig with required_tools, required_fields, evidence_description
- [x] detect_topic() deterministic pre-LLM pattern matching
- [x] TopicDetection with confidence, secondary, service_name, is_diagnostic
- [x] generate_answer() templates for structured responses
- [x] validate_evidence() for Junior verification
- [x] human_label() and working_message() for transcript
- [x] Pipeline integration: detect_topic runs before translator
- [x] Answer validation: penalizes mismatched topic responses

### 0.0.60 - 3-Tier Transcript Rendering (COMPLETED)
- [x] TranscriptMode enum: Human (default), Debug, Test
- [x] TranscriptEvent event bus for all transcript steps
- [x] Human labels registry (tool names -> human descriptions)
- [x] Transcript renderer with mode-aware output
- [x] ANNA_UI_TRANSCRIPT_MODE env var precedence
- [x] Config: [ui] transcript_mode = "human|debug|test"
- [x] Human mode hides: tool names, evidence IDs, raw prompts
- [x] Debug/test modes show: full internal details
- [x] Debug logs always saved to case directories
- [x] Status display shows current transcript mode
- [x] Integration tests for human vs debug rendering

### 0.0.59 - Auto-Case Opening + Departmental IT Org (COMPLETED)
- [x] Case lifecycle v2 (case_lifecycle.rs)
  - [x] CaseStatus enum: new, triaged, investigating, plan_ready, awaiting_confirmation, executing, verifying, resolved, abandoned
  - [x] Department enum: service_desk, networking, storage, boot, audio, graphics, security, performance
  - [x] CaseFileV2 schema with timeline, participants, findings, hypotheses, actions
  - [x] Alert linking via linked_alert_ids
  - [x] Active cases count in annactl status
- [x] Service Desk routing (service_desk.rs)
  - [x] triage_request() with keyword-based department scoring
  - [x] Auto-link alerts to related queries
  - [x] dispatch_to_specialist() maps department to DoctorRegistry
  - [x] open_case_for_alert() for alert-triggered cases
- [x] Transcript v2 (transcript_v2.rs)
  - [x] TranscriptBuilder for structured output
  - [x] DepartmentOutput with findings, evidence, hypotheses, action plan
  - [x] render_case_transcript() with phase separators
  - [x] render_handoff() for departmental assignments
  - [x] render_junior_disagreement() when evidence is insufficient
- [x] Integration tests (case_lifecycle_tests.rs)
  - [x] Doctor query opens case with lifecycle progression
  - [x] Transcript includes departmental handoff and case_id
  - [x] Junior forces re-check when coverage below threshold
  - [x] Alerts can be linked to cases deterministically

### 0.0.58 - Proactive Monitoring Loop v1 (COMPLETED)
- [x] Alerts subsystem (proactive_alerts.rs)
  - [x] ProactiveAlert struct with stable ID (hash of type + dedupe_key)
  - [x] AlertType enum: BOOT_REGRESSION, DISK_PRESSURE, JOURNAL_ERROR_BURST, SERVICE_FAILED, THERMAL_THROTTLING
  - [x] AlertSeverity: Critical, Warning, Info
  - [x] ProactiveAlertsState with upsert/resolve/get_active
  - [x] Daemon-owned state in /var/lib/anna/internal/alerts.json
- [x] 5 high-signal alert detectors (alert_detectors.rs)
  - [x] detect_boot_regression() - boot time > baseline + 2 stddev
  - [x] detect_disk_pressure() - / free < 10% or < 15 GiB (warning), < 5% or < 5 GiB (critical)
  - [x] detect_journal_error_burst() - >= 20 errors in 10 min per unit
  - [x] detect_service_failed() - systemd units in failed state
  - [x] detect_thermal_throttling() - CPU temp > 85C (warning), > 95C (critical)
- [x] Daemon probes for detection (alert_probes.rs)
  - [x] probe_alerts_summary() for user queries
  - [x] probe_boot_time_summary(), probe_disk_pressure_summary()
  - [x] probe_journal_error_burst_summary(), probe_failed_units_summary()
  - [x] probe_thermal_summary()
- [x] Status output integration
  - [x] [ALERTS] section shows proactive alert counts
  - [x] Top 3 active alerts with evidence IDs
  - [x] Snapshot age indicator
- [x] Tool routing for alert queries
  - [x] QueryTarget::Alerts with proactive_alerts_summary routing
  - [x] "show alerts", "why are you warning me?" patterns detected
  - [x] validate_answer_for_target() for alerts

### 0.0.57 - Evidence Coverage + Correct Tool Routing (COMPLETED)
- [x] EvidenceCoverage scoring (evidence_coverage.rs)
  - [x] TargetFacets struct defining required/optional fields per QueryTarget
  - [x] analyze_coverage() comparing evidence types against targets
  - [x] check_evidence_mismatch() detecting wrong evidence for target
  - [x] get_gap_filling_tools() suggesting tools to fill coverage gaps
  - [x] COVERAGE_SUFFICIENT_THRESHOLD = 90, COVERAGE_PENALTY_THRESHOLD = 50
- [x] Junior Rubric v2 (junior_rubric.rs)
  - [x] verify_answer() with coverage-aware scoring
  - [x] WRONG_EVIDENCE_PENALTY caps at 20%
  - [x] MISSING_EVIDENCE_PENALTY caps at 50%
  - [x] UNCITED_CLAIM_PENALTY for answers without [E#] citations
  - [x] get_max_score_for_coverage() enforcing caps
- [x] Correct tool routing (already in system_query_router.rs)
  - [x] disk_free -> mount_usage
  - [x] memory -> memory_info
  - [x] kernel_version -> kernel_version
  - [x] network_status -> network_status
- [x] Coverage display in transcripts
  - [x] render_junior_coverage_check() showing coverage % and missing fields
  - [x] render_anna_coverage_retry() when fetching additional evidence
- [x] Case file coverage fields
  - [x] evidence_coverage_percent in CaseFileV1
  - [x] missing_evidence_fields list
  - [x] evidence_retry_triggered flag
- [x] Integration tests (evidence_coverage_tests.rs)
  - [x] Disk query with CPU evidence fails (main bug fix)
  - [x] Memory/kernel/network queries with wrong evidence fail
  - [x] Correct evidence passes with high reliability
  - [x] Coverage gap detection and tool suggestions
  - [x] Junior scoring caps enforced

### 0.0.56 - Fly-on-the-Wall Dialogue Layer v1 (COMPLETED)
- [x] DialogueRenderer with actor voices (dialogue_renderer.rs)
  - [x] Actor voices: anna=calm, translator=brisk, junior=skeptical QA, annad=robotic
  - [x] Tone applied at render-time only (logic layer stays deterministic)
  - [x] DialogueContext for tracking confidence, reliability, doctor domain
- [x] Explicit agree/disagree moments
  - [x] render_anna_translator_response() with "Acknowledged, I agree" vs "proceed carefully"
  - [x] render_junior_verification() with "Ship it" vs "Not good enough"
  - [x] render_anna_junior_response() acknowledging junior's verdict
- [x] Doctors as departments with handoffs
  - [x] doctor_actor_name() mapping domains to [networking-doctor], [audio-doctor], etc.
  - [x] render_doctor_handoff() with natural handoff dialogue
  - [x] Doctor initiates probe requests, not anna (when doctor present)
- [x] Transcript ergonomics
  - [x] phase_separator() with `----- triage -----` format
  - [x] render_evidence_summary() with "Evidence collected: [E1, E2, E3]"
  - [x] render_evidence_item() for individual evidence lines
  - [x] render_reliability_footer() with QA signoff
- [x] Uncertainty expressions
  - [x] CONFIDENCE_CERTAIN_THRESHOLD = 80
  - [x] Low confidence: "Not fully certain", "proceed carefully"
  - [x] Fallback: "conservative route"
- [x] Reliability as QA signoff
  - [x] RELIABILITY_SHIP_THRESHOLD = 75
  - [x] >= 90%: "Solid evidence. Ship it."
  - [x] >= 75%: "Acceptable. Ship it."
  - [x] < 75%: "Not good enough" with critique
- [x] Golden tests for transcript stability (dialogue_golden_tests.rs)
  - [x] SYSTEM_QUERY tests: actors, evidence IDs, agree/disagree, no command spam
  - [x] DIAGNOSE tests: doctor handoff, multiple evidence, correct actors
  - [x] ACTION_REQUEST tests: evidence, confirmation mention
  - [x] Low confidence tests: uncertainty expressions
  - [x] Low reliability tests: junior disagreement
- [x] Full render_dialogue_transcript() function

### 0.0.55 - Deterministic Case Engine + Doctor-first Routing (COMPLETED)
- [x] Case Engine state machine (case_engine.rs)
  - [x] CasePhase enum with 10 phases (Intake→LearnRecipe)
  - [x] CaseState with phase tracking, events, timings
  - [x] IntentType enum (SYSTEM_QUERY, DIAGNOSE, ACTION_REQUEST, HOWTO, META)
  - [x] CaseEvent for structured transcript events
  - [x] Phase transitions with timing records
- [x] Intent Taxonomy (intent_taxonomy.rs)
  - [x] classify_intent() with pattern matching
  - [x] Query target detection for SYSTEM_QUERY
  - [x] Problem domain detection for DIAGNOSE
  - [x] Action type detection for ACTION_REQUEST
  - [x] domain_to_doctor() mapping
- [x] Evidence Tools (evidence_tools.rs)
  - [x] EvidencePlan with PlannedTool list
  - [x] plan_evidence() based on intent classification
  - [x] Correct tool routing (CPU→hw_snapshot_cpu, disk→mount_usage, etc.)
  - [x] validate_evidence_for_query() ensures evidence matches target
- [x] Case File Schema v1 (case_file_v1.rs)
  - [x] CaseFileV1 with complete audit fields
  - [x] EvidenceRecordV1 for collected evidence
  - [x] Atomic save with case.json + summary.txt + transcript.log
  - [x] load_case() and list_recent_case_ids()
- [x] Recipe Extractor (recipe_extractor.rs)
  - [x] Gate rules: reliability >= 80%, >= 2 evidence, success
  - [x] check_recipe_gate() for case eligibility
  - [x] extract_recipe() for recipe creation
  - [x] calculate_case_xp() for XP calculation
- [x] Transcript Renderer (transcript_render.rs)
  - [x] render_transcript_from_state() and render_transcript_from_file()
  - [x] [actor] to [actor]: message format
  - [x] Phase separators (--- Phase ---)
  - [x] render_compact_summary() for status display
- [x] Unit tests for all new modules

### 0.0.54 - Action Engine v1 (Safe Mutations) (COMPLETED)
- [x] Action Engine contract (action_engine.rs)
  - [x] ActionPlan with risk, summary, steps, confirmation_phrase
  - [x] ActionStep with precheck_probes, verify_probes, rollback_hint
  - [x] ActionType enum: EditFile, WriteFile, DeleteFile, Systemd, Pacman
  - [x] MutationRiskLevel: Low, Medium, High, Destructive, Denied
- [x] Risk scoring rules (action_risk.rs)
  - [x] score_path_risk() - /etc → medium, /etc/fstab → high, /proc → denied
  - [x] score_systemd_risk() - network services → medium, sshd → high, journald → denied
  - [x] score_package_risk() - install → low, remove → medium, kernel remove → denied
  - [x] score_delete_risk() - always high or destructive
- [x] Confirmation phrases
  - [x] Low: "yes"
  - [x] Medium: "I CONFIRM (medium risk)"
  - [x] High: "I CONFIRM (high risk)"
  - [x] Destructive: "I ACCEPT DATA LOSS RISK"
- [x] Diff preview pipeline (action_executor.rs)
  - [x] generate_action_diff_preview() with unified diff
  - [x] ActionDiffPreview with additions, deletions, truncation indicator
  - [x] Backup path shown before execution
- [x] Rollback scaffolding
  - [x] RollbackRecord with steps, backups, restore_instructions
  - [x] Saved to /var/lib/anna/cases/<case_id>/rollback.json
  - [x] BackupRecord with original_path, backup_path, hash
- [x] Step execution
  - [x] execute_action_step() with atomic writes (temp + fsync + rename)
  - [x] execute_action_plan() with confirmation validation
  - [x] Systemd operations via systemctl
  - [x] Pacman operations via pacman
- [x] Integration tests
  - [x] Action request shows risk assessment
  - [x] File edit request mentions diff/config
  - [x] Package install request shows package info

### 0.0.53 - Doctor Flow v1 (Interactive Diagnostics) (COMPLETED)
- [x] Doctor Flow orchestration (doctor_flow.rs)
  - [x] DoctorFlowExecutor for running diagnostic flows
  - [x] DoctorFlowStep with status tracking (Pending, Running, Success, Failed, Skipped)
  - [x] DoctorFlowResult with evidence, diagnosis, and report
  - [x] DoctorCaseFile for audit persistence
- [x] Problem phrase detection for auto-triggering
  - [x] detect_problem_phrase() with confidence scoring (0-100)
  - [x] PROBLEM_PHRASES list ("not working", "keeps disconnecting", etc.)
  - [x] PROBLEM_WORDS list ("broken", "crashed", "slow", etc.)
  - [x] Integration into classify_intent_deterministic()
- [x] Pipeline integration
  - [x] FixIt intent routes to doctor flow
  - [x] run_doctor_flow() executes diagnostic plan
  - [x] Human-readable transcript with evidence IDs
  - [x] Case file creation after diagnosis
- [x] Case file persistence
  - [x] /var/lib/anna/cases/<case_id>/doctor.json
  - [x] Steps executed with tool names and durations
  - [x] Findings, most likely cause, suggested actions
  - [x] Reliability score
- [x] Integration tests
  - [x] "wifi keeps disconnecting" triggers doctor flow
  - [x] "no sound" triggers doctor flow
  - [x] "slow boot" triggers doctor flow
  - [x] Normal queries don't trigger doctor flow

### 0.0.52 - System Query Router v1 (Quality Sprint) (COMPLETED)
- [x] System Query Router with canonical targets
  - [x] QueryTarget enum (Cpu, Memory, DiskFree, KernelVersion, NetworkStatus, AudioStatus, ServicesStatus)
  - [x] ToolRouting struct with required_tools, optional_tools, output_description
  - [x] detect_target() with confidence scoring (0-100)
  - [x] get_tool_routing() maps targets to correct tools
  - [x] validate_answer_for_target() ensures answer matches query
- [x] Fixed system query routing bugs
  - [x] Disk space queries now use mount_usage (not hw_snapshot_summary)
  - [x] Kernel version queries now use kernel_version tool
  - [x] Memory queries now use memory_info tool
  - [x] Router runs FIRST before action keyword check
- [x] Updated Translator contract
  - [x] Domain-specific tools in system prompt (memory_info, mount_usage, kernel_version, etc.)
  - [x] Examples for each query type in prompt
  - [x] hw_snapshot_summary reserved for general CPU/GPU queries only
- [x] Junior verification upgrade
  - [x] enforce_answer_target_correctness() validates answer matches target
  - [x] 50-point penalty for wrong-target answers (e.g., CPU info for disk query)
  - [x] Critique includes "WRONG TARGET" message with details
- [x] Unit tests for router
  - [x] Detection tests for all canonical targets
  - [x] Validation tests (correct and wrong answers)
  - [x] Tool routing tests
  - [x] Edge cases (ambiguous queries, memory vs disk)

### 0.0.51 - Action Engine v1 (Systemd Service Operations) (COMPLETED)
- [x] Systemd service action engine
  - [x] ServiceOperation enum (Start, Stop, Restart, Enable, Disable)
  - [x] RiskLevel enum (Low, Medium, High, Denied)
  - [x] ServiceAction struct with service, operation, reason
  - [x] assess_risk() for automatic risk classification
- [x] Service lists for risk assessment
  - [x] NETWORK_SERVICES - medium risk (NetworkManager, iwd, etc.)
  - [x] CRITICAL_SERVICES - high risk (sshd, display-manager, gdm, etc.)
  - [x] CORE_SYSTEMD_SERVICES - denied (systemd-journald, dbus, etc.)
- [x] Confirmation phrases by risk level
  - [x] LOW_RISK_CONFIRMATION = "I CONFIRM (low risk)"
  - [x] MEDIUM_RISK_CONFIRMATION = "I CONFIRM (medium risk)"
  - [x] HIGH_RISK_CONFIRMATION = "I ASSUME THE RISK"
- [x] Systemd service tools in tool catalog
  - [x] systemd_service_probe_v1 - probe current state
  - [x] systemd_service_preview_v1 - preview with risk assessment
  - [x] systemd_service_apply_v1 - apply with rollback metadata
  - [x] systemd_service_rollback_v1 - rollback by case_id
- [x] Modular file structure (< 400 lines each)
  - [x] systemd_action.rs - core types and risk assessment
  - [x] systemd_probe.rs - probe functionality
  - [x] systemd_apply.rs - preview and apply
  - [x] systemd_rollback.rs - rollback functionality
  - [x] systemd_tools.rs - tool executors
- [x] Rollback infrastructure
  - [x] Rollback metadata at /var/lib/anna/rollback/<case_id>/
  - [x] service_rollback.json with pre/post state snapshots
  - [x] Verified state after apply with verify_message
- [x] Unit tests for risk assessment and operations

### 0.0.50 - User File Mutations (COMPLETED)
- [x] User file edit primitive with append_line and set_key_value modes
  - [x] UserFileEditAction struct in user_file_mutation.rs
  - [x] EditMode enum (AppendLine, SetKeyValue)
  - [x] VerifyStrategy enum (FileContains, HashChanged, None)
  - [x] Idempotent operations (skip if already exists)
- [x] Path policy enforcement (HOME-only rule)
  - [x] check_path_policy() function
  - [x] Blocked prefixes: /etc, /usr, /var, /boot, /root, /proc, /sys, /dev
  - [x] Symlink escape detection
  - [x] PathPolicyResult with allowed, reason, evidence_id
- [x] File edit tools in tool catalog (tools.rs)
  - [x] file_edit_preview_v1 - read-only preview with diff
  - [x] file_edit_apply_v1 - apply with backup and verification
  - [x] file_edit_rollback_v1 - restore from case_id
- [x] File edit tool executors (file_edit_tools.rs)
  - [x] execute_file_edit_preview_v1()
  - [x] execute_file_edit_apply_v1() with confirmation phrase check
  - [x] execute_file_edit_rollback_v1()
- [x] Backup and rollback infrastructure
  - [x] Backup to /var/lib/anna/rollback/<case_id>/backup/
  - [x] apply_result.json for rollback metadata
  - [x] Operations logging to /var/lib/anna/internal/ops.log
- [x] Confirmation flow (medium risk)
  - [x] USER_FILE_CONFIRMATION = "I CONFIRM (medium risk)"
  - [x] preview_id requirement before apply
- [x] Integration tests for mutations
  - [x] Path policy tests (blocked, home, symlink escape)
  - [x] Preview tests (append, set_key_value, idempotent)
- [x] Updated version to 0.0.50

### 0.0.49 - Doctor Lifecycle System (COMPLETED)
- [x] Unified Doctor interface and lifecycle contract
  - [x] Doctor trait with id(), domains(), matches(), plan(), diagnose()
  - [x] DiagnosticCheck for ordered evidence collection
  - [x] CollectedEvidence with evidence_id, check_id, tool_name, data
  - [x] DiagnosisFinding with severity, evidence_ids, confidence, tags
  - [x] DiagnosisResult with summary, most_likely_cause, findings, next_steps
  - [x] DoctorRunner for orchestrating diagnosis flows
  - [x] DoctorReport with render() for human-readable output
- [x] NetworkingDoctorV2 implementing Doctor trait
  - [x] Ordered diagnostic plan (link -> IP -> route -> DNS -> connectivity)
  - [x] Evidence-based findings with confidence scores
  - [x] ProposedAction with risk levels and rollback commands
  - [x] SafeNextStep for read-only suggestions
- [x] Network evidence tools
  - [x] net_interfaces_summary - detailed interface info
  - [x] net_routes_summary - routing table info
  - [x] dns_summary - DNS configuration
  - [x] iw_summary - wireless status
  - [x] recent_network_errors - network journal errors
  - [x] ping_check - connectivity test
- [x] Fix case file permissions for non-root users
  - [x] Try user directory first (~/.local/share/anna/cases/)
  - [x] Fall back to system directory for daemon use
- [x] Knowledge learning integration
  - [x] qualifies_for_learning() method on DoctorReport
  - [x] tools_used() and targets() for recipe creation
- [x] Integration tests for doctor lifecycle

### 0.0.48 - Learning System (COMPLETED)
- [x] Knowledge Pack v1 format with strict limits
  - [x] Local JSON at /var/lib/anna/knowledge_packs/installed/*.json
  - [x] Max 50 packs, 500 total recipes, 24KB per recipe
  - [x] Schema with pack_id, name, version, source, tags, entries
  - [x] LearnedRecipe structure with intent, targets, triggers, actions, rollback
- [x] Knowledge search tool with local retrieval
  - [x] learned_recipe_search(query, limit) tool
  - [x] Token-based scoring (no embeddings for v1)
  - [x] Returns SearchHit with recipe_id, title, score, evidence_id
  - [x] Evidence IDs: K1, K2 prefix for knowledge
- [x] Learning pipeline (case -> recipe conversion)
  - [x] LearningManager with storage/retrieval
  - [x] Auto-create monthly packs (learned-pack-YYYYMM)
  - [x] Recipe deduplication (increment wins instead of duplicate)
  - [x] Minimum reliability 90%, minimum evidence count 1
- [x] XP system with deterministic progression
  - [x] Non-linear XP curve (100→500→1200→2000→...)
  - [x] Level 0-100 with title progression
  - [x] Titles: Intern, Apprentice, Junior, Competent, Senior, Expert, Wizard, Grandmaster
  - [x] XP gains: +2 (85% reliability), +5 (90%), +10 (recipe created)
  - [x] learning_stats tool for XP/level display
- [x] Transcript and case file updates
  - [x] LearningRecord structure in transcript.rs
  - [x] knowledge_searched, knowledge_query, recipes_matched fields
  - [x] recipe_written, recipe_id, xp_gained, level_after fields
- [x] Integration tests (run_learning_tests())
  - [x] Learning stats retrieval
  - [x] Query timing comparison
  - [x] Knowledge search capability
  - [x] XP directory structure check
- [x] Updated version to 0.0.48

### 0.0.47 - First Mutation Flow (COMPLETED)
- [x] Append line mutation with full lifecycle
  - [x] append_line_mutation.rs module
  - [x] SandboxCheck for dev-safe paths (cwd, /tmp, $HOME)
  - [x] AppendMutationEvidence collection (stat, preview, hash, policy)
  - [x] AppendDiffPreview for showing changes before execution
  - [x] Risk levels: Sandbox (low/yes), Home (medium/I CONFIRM), System (blocked)
  - [x] execute_append_line with backup, verification, and rollback info
  - [x] execute_rollback by case_id
- [x] File evidence tools (4 new tools)
  - [x] file_stat - uid/gid, mode, size, mtime, exists
  - [x] file_preview - first N bytes with secrets redacted
  - [x] file_hash - SHA256 for before/after verification
  - [x] path_policy_check - allowed/blocked decision with evidence ID
- [x] Case files for every request
  - [x] User-readable copies in $HOME/.local/share/anna/cases/
  - [x] save_user_copy() method on CaseFile
- [x] Deep test mutation tests (run_mutation_tests())
  - [x] Test diff preview and confirmation requirement
  - [x] Test file unchanged without confirmation
  - [x] Test sandbox path policy
  - [x] Test blocked path policy
- [x] Updated version to 0.0.47

### 0.0.46 - Evidence Quality Release (COMPLETED)
- [x] Domain-specific evidence tools (10 new tools)
  - [x] uname_summary - kernel version and architecture
  - [x] mem_summary - memory total/available from /proc/meminfo
  - [x] mount_usage - disk space with root free/used
  - [x] nm_summary - NetworkManager status and connections
  - [x] ip_route_summary - routing table and default gateway
  - [x] link_state_summary - interface link states
  - [x] audio_services_summary - pipewire/wireplumber/pulseaudio status
  - [x] pactl_summary - PulseAudio/PipeWire sinks and sources
  - [x] boot_time_summary - uptime and boot timestamp
  - [x] recent_errors_summary - journal errors filtered by keyword
- [x] Domain routing in translator (route_to_domain_evidence())
- [x] Tool sanity gate (apply_tool_sanity_gate())
  - [x] Prevents generic hw_snapshot for domain-specific queries
  - [x] Auto-replaces with correct domain tools
- [x] Deep test evidence validation (run_evidence_tool_validation())
- [x] Updated deep test to v0.0.46

### 0.0.45 - Deep Test Harness + Correctness Fixes (COMPLETED)
- [x] Deep test harness (scripts/anna_deep_test.sh)
  - [x] Environment capture
  - [x] Translator stability tests (50 queries)
  - [x] Read-only correctness tests
  - [x] Doctor auto-trigger tests
  - [x] Policy gating tests
  - [x] Case file verification
  - [x] REPORT.md and report.json outputs
- [x] New evidence tools for correctness
  - [x] kernel_version - direct uname
  - [x] memory_info - direct /proc/meminfo
  - [x] network_status - interfaces, routes, NM status
  - [x] audio_status - pipewire/wireplumber
- [x] Enhanced disk_usage with explicit free space values
- [x] Version mismatch display in status (CLI vs daemon)
- [x] Table-driven doctor selection tests (25 phrases)
- [x] docs/TESTING.md documentation

### 0.0.2 - Strict CLI Surface (COMPLETED)
- [x] Remove `sw` command from public surface
- [x] Remove `hw` command from public surface
- [x] Remove all JSON flags from public surface
- [x] Keep only: `annactl`, `annactl <request>`, `annactl status`, `annactl --version`
- [x] Legacy commands route as natural language requests (no custom error)
- [x] REPL mode basic implementation (exit, quit, help, status)
- [x] CLI tests for new surface

### 0.0.3 - Request Pipeline Skeleton (COMPLETED)
- [x] Create DialogueActor enum (You, Anna, Translator, Junior, Annad)
- [x] Full multi-party dialogue transcript
- [x] Deterministic Translator mock (intent classification: question, system_query, action_request, unknown)
- [x] Target detection (cpu, memory, disk, docker, etc.)
- [x] Risk classification (read-only, low-risk, medium-risk, high-risk)
- [x] Evidence retrieval mock from snapshots
- [x] Junior scoring rubric (+40 evidence, +30 confident, +20 observational+cited, +10 read-only)
- [x] CLI tests for pipeline behavior

### 0.0.4 - Real Junior Verifier (COMPLETED)
- [x] Ollama HTTP client in anna_common
- [x] Junior config keys (junior.enabled, junior.model, junior.timeout_ms, junior.ollama_url)
- [x] Real Junior LLM verification via Ollama
- [x] Junior system prompt with scoring rubric
- [x] Junior output parsing (SCORE, CRITIQUE, SUGGESTIONS, MUTATION_WARNING)
- [x] Fallback to deterministic scoring when Ollama unavailable
- [x] Spinner while Junior thinks
- [x] Graceful handling when Ollama not available
- [x] Model auto-selection (prefers qwen2.5:1.5b, llama3.2:1b, etc.)
- [x] CLI tests for Junior LLM behavior

### 0.0.5 - Role-Based Model Selection + Benchmarking (COMPLETED)
- [x] Hardware detection module (CPU/RAM/GPU profiling)
- [x] Hardware tier classification (Low: <8GB, Medium: 8-16GB, High: >16GB)
- [x] LlmRole enum (Translator, Junior)
- [x] Role-based model candidate pools with priority
- [x] Model selection based on hardware tier and availability
- [x] Translator benchmark suite (30 prompts for intent classification)
- [x] Junior benchmark suite (15 cases for verification quality)
- [x] Ollama pull with progress streaming and ETA
- [x] BootstrapPhase state machine (detecting_ollama, installing_ollama, pulling_models, benchmarking, ready, error)
- [x] Bootstrap state in status snapshot
- [x] annactl progress display when models not ready
- [x] Graceful degradation with reduced reliability score when LLM unavailable
- [x] Unit tests for hardware bucketing and model selection

### 0.0.6 - Real Translator LLM (COMPLETED)
- [x] Real Translator LLM integration replacing deterministic translator
- [x] Translator structured output parsing (intent, targets, risk, evidence_needs, clarification)
- [x] Clarify-or-proceed loop with multiple-choice prompts
- [x] Evidence-first pipeline with real snapshot integration
- [x] 8KB evidence excerpt cap with truncation indication
- [x] Action plan generation for action requests (steps, affected resources, rollback outline)
- [x] Confirmation-gated action plan display (no execution)
- [x] Deterministic fallback when Translator LLM unavailable
- [x] 15 unit tests for Translator parsing, clarification, evidence, action plans
- [x] CLI tests updated for v0.0.6

### 0.0.7 - Read-Only Tooling & Evidence Citations (COMPLETED)
- [x] Read-only tool catalog (tools.rs) with allowlist enforcement
- [x] 10 read-only tools: status_snapshot, sw_snapshot_summary, hw_snapshot_summary, recent_installs, journal_warnings, boot_time_trend, top_resource_processes, package_info, service_status, disk_usage
- [x] Tool executor with structured outputs and human summaries
- [x] Evidence IDs (E1, E2, ...) assigned by EvidenceCollector
- [x] Translator outputs tool plans in TOOLS field
- [x] Tool plan parsing from Translator LLM output
- [x] Deterministic fallback generates tool plans from evidence_needs
- [x] Junior no-guessing enforcement (require citations, UNCITED_CLAIMS output)
- [x] Natural language dialogue transcripts for tool execution
- [x] Evidence ID citations in final responses
- [x] 7 new unit tests for tool catalog, evidence collector, uncited claims
- [x] Updated main.rs and pipeline.rs to v0.0.7

### 0.0.8 - First Safe Mutations (COMPLETED)
- [x] Mutation tool catalog with allowlist enforcement (6 tools)
- [x] Config file edits: /etc/**, $HOME/** (< 1 MiB, text only)
- [x] Systemd operations: restart, reload, enable --now, disable --now, daemon-reload
- [x] Automatic rollback: timestamped backups in /var/lib/anna/rollback/
- [x] Structured mutation logs (JSON per-request + JSONL append log)
- [x] Confirmation gate: exact phrase "I CONFIRM (medium risk)"
- [x] Junior verification threshold: >= 70% reliability required
- [x] MutationPlan, MutationRequest, MutationResult types
- [x] RollbackManager with file hashing and diff summaries
- [x] ActionPlan extended with mutation_plan and is_medium_risk_executable
- [x] handle_mutation_execution() in pipeline
- [x] Unit tests for path validation, confirmation, backup, rollback

### 0.0.9 - Package Management + Helper Tracking (COMPLETED)
- [x] Helper tracking system with provenance (helpers.rs)
- [x] HelperDefinition, HelperState, HelpersManifest types
- [x] InstalledBy enum: anna | user | unknown
- [x] Two dimensions tracked: present/missing + installed_by
- [x] get_helper_status_list(), refresh_helper_states()
- [x] Package management mutation tools: package_install, package_remove (8 tools total)
- [x] Only anna-installed packages removable via package_remove
- [x] Package transaction logging (MutationType::PackageInstall/Remove)
- [x] [HELPERS] section in annactl status
- [x] StatusSnapshot extended with helpers_total, helpers_present, helpers_missing, helpers_anna_installed
- [x] Unit tests for helper provenance tracking

### 0.0.10 - Reset + Uninstall + Installer Review (COMPLETED)
- [x] Add `annactl reset` command with factory reset
- [x] Add `annactl uninstall` command with provenance-aware helper removal
- [x] Install state tracking (install_state.json)
- [x] Installer review with auto-repair capabilities
- [x] Confirmation phrases: "I CONFIRM (reset)" and "I CONFIRM (uninstall)"
- [x] [INSTALL REVIEW] section in annactl status
- [x] Reset runs installer review at end and reports health

### 0.0.11 - Safe Auto-Update System (COMPLETED)
- [x] Update channels: stable (default) and canary
- [x] UpdateConfig with channel and min_disk_space
- [x] UpdateState enhanced with phase, progress, ETA tracking
- [x] UpdateManager for complete update lifecycle
- [x] Safe download with integrity verification (SHA256)
- [x] Staging directory for atomic updates
- [x] Backup of current binaries for rollback
- [x] Atomic installation via rename
- [x] Zero-downtime restart via systemd
- [x] Automatic rollback on failure
- [x] Guardrails: disk space, mutation lock, installer review
- [x] Update progress display in annactl status
- [x] Unit tests for version comparison and channel matching

### 0.0.12 - Proactive Anomaly Detection (COMPLETED)
- [x] AnomalyEngine with periodic detection (anomaly_engine.rs)
- [x] Anomaly signals: boot time regression, CPU load, memory pressure, disk space, crashes, services
- [x] Alert queue with deduplication and severity (Info, Warning, Critical)
- [x] Evidence IDs for all anomalies (ANO##### format)
- [x] what_changed(days) correlation tool
- [x] slowness_hypotheses(days) analysis tool
- [x] Alert surfacing in REPL welcome
- [x] Alert footer in one-shot mode
- [x] [ALERTS] section enhanced in status display
- [x] 16 unit tests for anomaly detection

### 0.0.13 - Conversation Memory + Recipe Evolution (COMPLETED)
- [x] Session memory with local storage (memory.rs)
- [x] Privacy default: summaries only, not raw transcripts
- [x] Recipe system with named intent patterns (recipes.rs)
- [x] Recipe creation threshold: Junior reliability >= 80%
- [x] Recipe matching (keyword-based/BM25 style)
- [x] User introspection via natural language (introspection.rs)
- [x] Forget/delete requires "I CONFIRM (forget)" confirmation
- [x] Status display with [LEARNING] section
- [x] Junior enforcement for learning claims (MEM/RCP citations required)
- [x] 28 unit tests for memory, recipes, introspection

### 0.0.14 - Policy Engine + Security Posture (COMPLETED)
- [x] Policy engine with TOML configuration files (policy.rs)
- [x] Four policy files: capabilities.toml, risk.toml, blocked.toml, helpers.toml
- [x] Policy-driven allowlists (no hardcoded deny rules)
- [x] Policy evidence IDs (POL##### format) in transcript
- [x] Audit logging with secret redaction (audit_log.rs)
- [x] Installer review policy sanity checks
- [x] Junior policy enforcement rules
- [x] 24 unit tests for policy and audit systems

### 0.0.15 - Governance UX Polish (COMPLETED)
- [x] Debug levels configuration in config.toml (ui.debug_level = 0|1|2)
- [x] Level 0 (minimal): only [you]->[anna] and final [anna]->[you], plus confirmations
- [x] Level 1 (normal/default): dialogues condensed, tool calls summarized, evidence IDs
- [x] Level 2 (full): full dialogues, tool execution summaries, Junior critique
- [x] Unified formatting module (display_format.rs) with colors, SectionFormatter, DialogueFormatter
- [x] Enhanced annactl status with sections: VERSION, INSTALLER REVIEW, UPDATES, MODELS, POLICY, HELPERS, ALERTS, LEARNING, RECENT ACTIONS, STORAGE
- [x] Format helpers: format_bytes, format_timestamp, wrap_text, indent
- [x] UiConfig struct with colors_enabled and max_width
- [x] 19 unit tests for display formatting
- [x] 5 unit tests for UI config

### 0.0.16 - Better Mutation Safety (COMPLETED)
- [x] Mutation state machine: planned -> preflight_ok -> confirmed -> applied -> verified_ok | rolled_back
- [x] Preflight checks for file edits (path, permissions, size, hash, backup)
- [x] Preflight checks for systemd ops (unit exists, state captured, policy)
- [x] Preflight checks for package ops (distro, packages, disk space)
- [x] Dry-run diff preview for file edits (line-based diff, truncated output)
- [x] DiffPreview struct with additions/deletions/modifications counts
- [x] Post-check verification for all mutation types
- [x] Automatic rollback on post-check failure
- [x] SafeMutationExecutor with full lifecycle management
- [x] Junior enforcement rules for mutation safety (penalties for missing preflight/diff/post-check)
- [x] 21 unit tests for mutation safety system

### 0.0.17 - Multi-User Correctness (COMPLETED)
- [x] Target user selection with strict precedence: REPL session > SUDO_USER > invoking user > primary interactive
- [x] Safe home directory detection via /etc/passwd (never guess /home/<name>)
- [x] User-scoped file operations: write_file_as_user, backup_file_as_user, fix_file_ownership
- [x] UserHomePolicy in capabilities.toml for allowed/blocked subpaths
- [x] Default allowed: .config/**, .bashrc, .zshrc, .vimrc, .gitconfig, etc.
- [x] Default blocked: .ssh/**, .gnupg/**, .password-store/**, browser credentials
- [x] Clarification prompt for ambiguous user selection (multiple candidates)
- [x] Evidence ID citations for user selection (E-user-##### format)
- [x] Target user transcript message in pipeline
- [x] 15 unit tests for target user system
- [x] 10 unit tests for user home policy

### 0.0.18 - Secrets Hygiene (COMPLETED)
- [x] Centralized redaction module with 22 secret types (Password, ApiKey, BearerToken, PrivateKey, etc.)
- [x] Compiled regex patterns via LazyLock for performance
- [x] Pattern matching for: passwords, tokens, API keys, bearer tokens, private keys, PEM blocks, SSH keys, cookies, AWS/Azure/GCP credentials, git credentials, database URLs, connection strings
- [x] Evidence restriction policy for sensitive paths (~/.ssh/**, ~/.gnupg/**, /etc/shadow, /proc/*/environ, etc.)
- [x] Redaction format: [REDACTED:TYPE] with type-specific placeholders
- [x] Junior leak detection enforcement (rules 20-24, penalties for secret leaks)
- [x] Redaction integration in dialogue output, evidence summaries, and LLM prompts
- [x] check_for_leaks() function with penalty calculation
- [x] 22 unit tests for redaction patterns and path restrictions

### 0.0.19 - Offline Documentation Engine (COMPLETED)
- [x] Knowledge packs stored under /var/lib/anna/knowledge_packs/
- [x] Pack schema: id, name, source type, trust level, retention policy, timestamps
- [x] SQLite FTS5 index for fast full-text search
- [x] Evidence IDs for citations (K1, K2, K3...)
- [x] Default pack ingestion: man pages, /usr/share/doc, Anna project docs
- [x] knowledge_search(query, top_k) tool with excerpts and citations
- [x] knowledge_stats() tool for index information
- [x] [KNOWLEDGE] section in annactl status
- [x] Secrets hygiene applied to all excerpts
- [x] 10+ unit tests for knowledge pack system

### 0.0.20 - Ask Me Anything Mode (COMPLETED)
- [x] Source labeling for answers: [E#] for system evidence, [K#] for knowledge, (Reasoning) for inference
- [x] New tools: answer_context(), source_plan(), qa_stats()
- [x] Translator plans source mix by question type (how-to vs system status vs mixed)
- [x] Junior enforcement: penalize unlabeled factual claims
- [x] "I don't know" behavior: report missing evidence, suggest read-only tools
- [x] [Q&A TODAY] section in annactl status: answers count, avg reliability, top sources
- [x] QuestionType classification: HowTo, SystemStatus, Mixed, General
- [x] SourcePlan with primary_sources, knowledge_query, system_tools
- [x] 8+ unit tests for source labeling

### 0.0.21 - Performance and Latency Sprint (COMPLETED)
- [x] TTFO (Time to First Output) < 150ms with header and working indicator
- [x] Token budgets per role: translator.max_tokens=256, translator.max_ms=1500
- [x] Token budgets per role: junior.max_tokens=384, junior.max_ms=2500
- [x] Read-only tool result caching with TTL policy (5 min default)
- [x] LLM response caching keyed by request, evidence, policy, model versions
- [x] Cache storage in /var/lib/anna/internal/cache/
- [x] Performance statistics tracking (samples, latencies, hit rates)
- [x] [PERFORMANCE] section in annactl status (avg latency, cache hit rate, top tools)
- [x] BudgetSettings, PerformanceConfig in config.toml
- [x] 10+ unit tests for cache key determinism and budget validation

### 0.0.22 - Reliability Engineering (COMPLETED)
- [x] Metrics collection module with local JSON storage (metrics.json)
- [x] Track: request success/failure, tool success/failure, mutation rollbacks, LLM timeouts
- [x] Latency tracking with p50/p95 percentile calculations
- [x] Error budgets with configurable thresholds (1% request, 2% tool, 0.5% rollback, 3% timeout)
- [x] Budget burn rate calculation with warning (50%) and critical (80%) thresholds
- [x] BudgetState enum: Ok, Warning, Critical, Exhausted
- [x] self_diagnostics() read-only tool for comprehensive system health report
- [x] metrics_summary() tool for reliability metrics display
- [x] error_budgets() tool for budget status
- [x] DiagnosticsReport with evidence IDs per section
- [x] Sections: Version, Install Review, Update State, Model Readiness, Policy, Storage, Error Budgets, Recent Errors, Active Alerts
- [x] Redaction integration for error logs in diagnostics
- [x] [RELIABILITY] section in annactl status with budget alerts
- [x] ReliabilityConfig in config.toml
- [x] 14 unit tests for metrics, budgets, and diagnostics

### 0.0.23 - REPL Enhancement (Planned)
- [ ] Improve REPL welcome message with level/XP display
- [ ] Add history support for REPL

---

## Phase 2: Full LLM Integration (0.1.x)

### 0.1.0 - Anna Response Generation
- [ ] Connect Anna response generation to Ollama
- [ ] Stream output per participant
- [ ] Real evidence parsing and answer formulation

### 0.1.1 - Senior Escalation
- [ ] Create escalation criteria (confidence < threshold, needs_senior flag)
- [ ] Create Senior LLM prompt template
- [ ] Implement query_senior() function
- [ ] Multi-round improvement loop

---

## Phase 3: Evidence System (0.2.x)

### 0.2.0 - Snapshot Integration
- [ ] Route hardware queries to hw.json snapshot
- [ ] Route software queries to sw.json snapshot
- [ ] Route status queries to status_snapshot.json
- [ ] Add snapshot source citations to answers

### 0.2.1 - Command Execution
- [ ] Define safe command whitelist (read-only)
- [ ] Implement safe command runner
- [ ] Capture command output as evidence
- [ ] Add command output citations

### 0.2.2 - Log Evidence
- [ ] Query journalctl for relevant logs
- [ ] Extract error/warning patterns
- [ ] Add log excerpt citations

---

## Phase 4: Safety Gates (0.3.x)

### 0.3.0 - Action Classification
- [ ] Define ActionRisk enum (ReadOnly, LowRisk, MediumRisk, HighRisk)
- [ ] Create action classifier
- [ ] Implement confirmation prompts per risk level
- [ ] "I assume the risk" for high-risk actions

### 0.3.1 - Rollback Foundation
- [ ] Create backup before file modifications
- [ ] Store timestamped backups
- [ ] Store patch diffs
- [ ] Create rollback instruction set

### 0.3.2 - btrfs Integration
- [ ] Detect btrfs filesystem
- [ ] Create pre-action snapshots when available
- [ ] Expose snapshot in rollback plan

---

## Phase 5: Learning System (0.4.x) - COMPLETED in 0.0.13

### 0.4.0 - Recipe Storage (COMPLETED)
- [x] Define Recipe struct (trigger, steps, verification, rollback, risk)
- [x] Create recipe store (JSON files)
- [x] Implement recipe save/load
- [x] Recipe versioning

### 0.4.1 - Recipe Matching (COMPLETED)
- [x] Match user intent to existing recipes
- [x] Execute recipe steps
- [ ] Skip Junior/Senior when recipe exists (future optimization)
- [x] Track recipe usage stats

### 0.4.2 - Recipe Learning (COMPLETED)
- [x] Detect recipe-worthy interactions (reliability >= 80%)
- [x] Extract recipe from successful answer
- [x] Create new recipe
- [x] Update existing recipes

---

## Phase 6: XP and Gamification (0.5.x)

### 0.5.0 - XP System
- [ ] Define XP curve (non-linear)
- [ ] Track XP for Anna, Junior, Senior
- [ ] Award XP for correct answers
- [ ] Award XP for new recipes

### 0.5.1 - Level and Titles
- [ ] Define level 0-100 progression
- [ ] Create title list (nerdy, ASCII-friendly)
- [ ] Display level/title in status
- [ ] Display level/title in REPL welcome

---

## Phase 7: Self-Sufficiency (0.6.x)

### 0.6.0 - Ollama Auto-Setup
- [ ] Detect Ollama installation
- [ ] Install Ollama if missing
- [ ] Detect hardware capabilities
- [ ] Select appropriate models
- [ ] Download models with progress

### 0.6.1 - Auto-Update
- [ ] Check GitHub releases every 10 minutes
- [ ] Download new version
- [ ] Verify checksum
- [ ] Restart annad safely
- [ ] Record update state

### 0.6.2 - Helper Tracking
- [ ] Track Anna-installed packages
- [ ] Display helpers in status
- [ ] Remove helpers on uninstall

### 0.6.3 - Clean Uninstall
- [ ] Implement `annactl uninstall`
- [ ] List helpers for removal choice
- [ ] Remove services, data, models
- [ ] Clean permissions

### 0.6.4 - Factory Reset
- [ ] Implement `annactl reset`
- [ ] Delete recipes
- [ ] Remove helpers
- [ ] Reset DBs
- [ ] Keep binaries

---

## Phase 8: Proactive Monitoring (0.7.x)

### 0.7.0 - Trend Detection
- [ ] Track boot time trends
- [ ] Track performance metrics over time
- [ ] Detect regressions

### 0.7.1 - Correlation Engine
- [ ] Correlate degradation with recent changes
- [ ] Package install timeline
- [ ] Service state changes

### 0.7.2 - Anomaly Alerts
- [ ] Thermal anomalies
- [ ] Network instability
- [ ] Disk I/O regressions
- [ ] Service failures

---

## Phase 9: Production Polish (0.8.x - 0.9.x)

### 0.8.0 - UI Polish
- [ ] ASCII borders and formatting
- [ ] True color support
- [ ] Spinner indicators
- [ ] Streaming output

### 0.8.1 - Performance Optimization
- [ ] Minimize LLM prompt sizes
- [ ] Cache frequent queries
- [ ] Optimize snapshot reads

### 0.9.0 - Testing and Hardening
- [ ] Integration tests for all flows
- [ ] Error handling coverage
- [ ] Edge case testing

### 0.9.1 - Documentation
- [ ] Complete README
- [ ] Architecture docs
- [ ] User guide

---

## Milestone: v1.0.0 - Production Ready

All phases complete, tested, documented. Ready for production use.

---

## Notes

- Each task should be small and verifiable
- Preserve snapshot performance throughout
- Debug mode always on for now
- No emojis or icons in output
- Every completed task moves to RELEASE_NOTES.md
