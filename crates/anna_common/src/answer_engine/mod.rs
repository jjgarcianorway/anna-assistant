//! Answer Engine
//!
//! v0.14.0: Aligned to Reality (6 real probes)
//! v0.15.0: Junior/Senior architecture with dynamic checks
//! v0.18.0: Step-by-step orchestration (one action per iteration)
//! v0.19.0: Subproblem decomposition, fact-aware planning, Senior as mentor
//! v0.21.0: Hybrid answer pipeline (fast-first, selective probing, no loops)
//! v0.22.0: Fact Brain & Question Decomposition (TTLs, validated facts)
//! v0.23.0: System Brain, User Brain & Idle Learning
//! v0.24.0: App Awareness, Stats & Faster Answers
//! v0.25.0: Relevance First, Idle Learning, No Hard-Coding
//! v0.26.0: Auto-update Reliability, Self-Healing, Logging
//!
//! Features:
//! - Deterministic probe usage
//! - Explicit evidence citations
//! - Supervised LLM-A/LLM-B loop
//! - Transparent reliability scoring
//! - Partial answer fallback
//! - v0.15.0: Risk classification, user questions, learning
//! - v0.18.0: One probe per iteration, clear Junior/Senior roles
//! - v0.19.0: Subproblem decomposition, mentor-style Senior
//! - v0.21.0: Fast-first from facts, selective probing, no iteration loops
//! - v0.22.0: TTL-based facts, question decomposition, semantic linking
//! - v0.23.0: System/User knowledge layers, idle learning, safe file scanning
//! - v0.24.0: WM/app awareness, MIME defaults, stats engine, answer caching
//! - v0.25.0: Relevance-first, usage tracking, session awareness, disambiguation
//! - v0.26.0: Auto-update manager, daemon watchdog, self-healing, tracing

pub mod ambiguity_resolver;
pub mod app_awareness;
pub mod auto_update;
pub mod daemon_watchdog;
pub mod default_apps;
pub mod evidence;
pub mod faster_answers;
pub mod file_scanner;
pub mod idle_learning;
pub mod protocol;
pub mod protocol_v15;
pub mod protocol_v18;
pub mod protocol_v19;
pub mod protocol_v21;
pub mod protocol_v22;
pub mod protocol_v23;
pub mod protocol_v25;
pub mod protocol_v26;
pub mod protocol_v43;
pub mod relevance_engine;
pub mod scoring;
pub mod session_awareness;
pub mod stats_engine;
pub mod usage_tracking;

pub use evidence::*;
pub use protocol::*;
pub use scoring::*;

// v0.15.0 protocol types (legacy)
pub use protocol_v15::{
    AnswerContinueRequest, AnswerSessionResponse, AnswerStartRequest, CheckApproval, CheckRequest,
    CheckResult, CheckRisk, CheckRiskEval, ConfirmCommandRequest, CoreProbeId, CoreProbeInfo,
    DynamicCheck, FactSource, Intent, LearningUpdate, LlmARequestV15, LlmAResponseV15,
    LlmBRequestV15, LlmBResponseV15, LlmBVerdict, QuestionOption, QuestionStyle, ReasoningTrace,
    TraceStep, TrackedFact, UserAnswer, UserAnswerValue, UserQuestion,
    PROTOCOL_VERSION as PROTOCOL_VERSION_V15,
};

// v0.18.0 protocol types (legacy)
pub use protocol_v18::{
    Clarification, EscalationRequest, EscalationSummary, FinalAnswerV18, HistoryEntry,
    JuniorDraft, JuniorRequest, JuniorScores, JuniorStep, LoopStatus, ProbeResultV18,
    QuestionLoopState, SeniorRequest, SeniorResponse, SeniorScores, MAX_ITERATIONS,
    MIN_SCORE_WITHOUT_SENIOR, PROTOCOL_VERSION as PROTOCOL_VERSION_V18, SCORE_GREEN, SCORE_YELLOW,
};

// v0.19.0 protocol types (legacy)
pub use protocol_v19::{
    FinalAnswerV19, JuniorDecomposition, JuniorScoresV19, JuniorStepV19, KnownFact, MentorContext,
    ProbeResultV19, QuestionStateV19, SeniorMentor, SeniorScoresV19, Subproblem, SubproblemMerge,
    SubproblemStatus, SubproblemSummary, SuggestedSubproblem, MAX_ITERATIONS_V19, MAX_SUBPROBLEMS,
    MIN_CONFIDENCE_FOR_SYNTHESIS,
};

// v0.21.0 protocol types (legacy)
pub use protocol_v21::{
    AnswerSource, FactFreshness, FastPathConfidence, FastPathResult, HybridAnswer, HybridConfig,
    HybridPipeline, JuniorActionV21, KnowledgeGap, PipelineStage, ProbeOutcome, ProbingResult,
    RelevantFact, SeniorReviewV21, TargetedProbe, TargetedProbing, FAST_PATH_MIN_CONFIDENCE,
    MAX_PROBES_V21, PROBE_TIMEOUT_MS,
};

// v0.22.0 protocol types (legacy)
pub use protocol_v22::{
    AggregateOp, AnalyzeQuestionRequest, AnalyzeQuestionResponse, FactBrain, FactSourceV22,
    FactTtlCategory, QuestionDecomposition, QuestionType, RequiredFact, SemanticLink,
    SemanticRelation, Subquestion, SynthesisStrategy, TtlFact, ValidationResult, ValidationStatus,
    DEFAULT_FACT_CONFIDENCE, MAX_SUBQUESTIONS, MIN_LINK_STRENGTH,
};

// v0.23.0 protocol types (current)
pub use protocol_v23::{
    DualBrain, FactSourceV23, FactValue, KnowledgeScope, SystemFact, SystemIdentity, UserFact,
    UserIdentity, PROTOCOL_VERSION_V23,
};

// v0.23.0 idle learning
pub use idle_learning::{
    IdleConditions, IdleLearningConfig, IdleState, LearningMission, MissionQueue, MissionStatus,
    DEFAULT_CPU_THRESHOLD, DEFAULT_IDLE_INTERVAL_SECS, DEFAULT_MAX_BYTES_PER_CYCLE,
    DEFAULT_MAX_FILES_PER_CYCLE, DEFAULT_MAX_PROBES_PER_CYCLE, DEFAULT_MAX_SCAN_TIME_MS,
};

// v0.23.0 file scanner
pub use file_scanner::{
    KnownPattern, PathValidation, ScanConfig, ScanResult, ScannedFile, get_allowed_base_dirs,
    get_known_patterns, validate_path, MAX_BYTES_PER_SCAN, MAX_FILES_PER_SCAN, MAX_FILE_SIZE,
    SYSTEM_ALLOWED_PATHS, USER_ALLOWED_RELATIVE_PATHS,
};

// v0.24.0 app awareness
pub use app_awareness::{
    AppCategory, AppDetectionProbe, AppDetectionTarget, DesktopEnvironment, DesktopSession,
    DisplayProtocol, OutputParser, RunningApp, SessionType, SuggestionSource, UserSession,
    WindowManagerInfo, WindowManagerType, WmAwareSuggestion, PROTOCOL_VERSION_V24,
};

// v0.24.0 default apps
pub use default_apps::{
    AppRole, DefaultApp, DefaultAppProbe, DefaultAppProbeTarget, DefaultAppSource,
    DefaultAppsRegistry, DesktopEntry, DesktopParseResult, MimeCategory, MimeType, XdgPaths,
};

// v0.24.0 stats engine
pub use stats_engine::{
    AnnaStats, CacheStats as StatsCacheStats, CpuStats, DaemonStats, DiskStats, InterfaceStats,
    KnowledgeStats, LlmStats, MemoryStats, MountStats, NetworkStats, PipelineStats, ProbeStats,
    StatsCategory, StatsQuery, StatsSnapshot, SystemStats, UserStats,
};

// v0.24.0 faster answers
pub use faster_answers::{
    AnswerCache, AnswerStrategy, CacheConfig as FastCacheConfig, CacheStatistics,
    CachedAnswer, ClassifiedQuestion, DetectedEntity, EntityType as FastEntityType,
    FastAnswerConfig, FastPath, FastPathResult as FastPathAnswerResult, QuestionPattern,
    QuestionType as FastQuestionType,
};

// v0.25.0 protocol types (current)
pub use protocol_v25::{
    AmbiguityCandidate, AmbiguityResolution, AmbiguityType, AnswerPath, DetectedAmbiguity,
    DiscoverySource, RankedEntity, RankedEntityType, RelevanceFactors, RelevanceScore,
    RelevantAnswerStrategy, RelevantMission, UsageEvent, UsageEventType, PROTOCOL_VERSION_V25,
};

// v0.25.0 relevance engine
pub use relevance_engine::{
    EntityMetadata, RankingEngine, RelevanceCalculator, RelevanceConfig, UsageStats,
    DEFAULT_MIN_USAGE_FOR_FREQUENCY, DEFAULT_RECENCY_HALFLIFE_SECS, DEFAULT_SCORE_REFRESH_INTERVAL,
};

// v0.25.0 usage tracking
pub use usage_tracking::{
    EntityUsageStats, PatternDetector, SequencePattern, UsageTracker, UsageTrackingConfig,
    DEFAULT_BATCH_SIZE, DEFAULT_MAX_EVENTS, DEFAULT_PERIOD_WINDOW_SECS,
};

// v0.25.0 session awareness
pub use session_awareness::{
    AppSessionUsage, FocusedApp, SessionConfig, SessionContext, SessionManager, SessionState,
    SessionSummary, WorkspaceInfo, DEFAULT_MAX_SESSIONS, DEFAULT_SESSION_TIMEOUT_SECS,
};

// v0.25.0 ambiguity resolver
pub use ambiguity_resolver::{
    AmbiguityConfig, AmbiguityDetector, DisambiguationPrompt, PromptBuilder, PromptOption,
    RememberedResolution, ResolutionMemory, DEFAULT_MAX_CANDIDATES, DEFAULT_MAX_REMEMBERED,
    DEFAULT_UNAMBIGUOUS_THRESHOLD,
};

// v0.26.0 protocol types (current)
pub use protocol_v26::{
    BinaryChecksum, DownloadState, HealingEvent, HealingEventType, HealingTrace, InstallStrategy,
    LogComponentV26, UpdateEvent, UpdateEventType, UpdateProgress, UpdateResultV26, UpdateTrace,
    WatchdogConfig, WatchdogState, PROTOCOL_VERSION_V26,
};

// v0.26.0 auto-update manager
pub use auto_update::{
    AutoUpdateConfig, AutoUpdateManager, DownloadedBinary, verify_checksum,
    DEFAULT_CHUNK_SIZE, DEFAULT_DOWNLOAD_DIR, DEFAULT_MAX_RETRIES, DEFAULT_STAGING_DIR,
};

// v0.26.0 daemon watchdog
pub use daemon_watchdog::{
    CheckTarget, DaemonWatchdog, HealthCheck, HealthCheckResult, RestartStats,
    check_daemon_api, check_ipc_socket, check_llm_backend,
    DEFAULT_CHECK_INTERVAL, DEFAULT_MAX_FAILURES, MAX_HISTORY_EVENTS,
};

// v0.43.0 protocol types (live debug streaming)
pub use protocol_v43::{
    DebugEvent, DebugEventData, DebugEventEmitter, DebugEventType, DebugStreamConfig,
    ProbeResultSnippet, PROTOCOL_VERSION_V43,
};
