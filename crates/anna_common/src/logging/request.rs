//! Request Context v0.8.0
//!
//! Request correlation and tracing across Anna components.
//! Every annactl invocation gets a unique request_id that flows
//! through daemon, probes, and LLM orchestration.

use super::{
    LlmPhase, LlmTrace, LogComponent, LogEntry, LogLevel, ReliabilityBreakdown,
    RequestStatus, RequestTrace,
};
use chrono::{DateTime, Utc};
use std::cell::RefCell;

thread_local! {
    /// Current request context for this thread
    static CURRENT_REQUEST: RefCell<Option<RequestContext>> = const { RefCell::new(None) };
}

/// Request context that flows through the system
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// Unique request ID
    pub request_id: String,
    /// Start timestamp
    pub start_time: DateTime<Utc>,
    /// User query (sanitized)
    pub query: String,
    /// Probes executed during this request
    pub probes: Vec<String>,
    /// Self-health actions taken
    pub health_actions: Vec<String>,
    /// LLM phases logged
    pub llm_phases: Vec<LlmPhase>,
    /// Final reliability score
    pub reliability_score: Option<f64>,
    /// Result status
    pub status: Option<RequestStatus>,
}

impl RequestContext {
    /// Create a new request context
    pub fn new(request_id: String, query: String) -> Self {
        Self {
            request_id,
            start_time: Utc::now(),
            query,
            probes: Vec::new(),
            health_actions: Vec::new(),
            llm_phases: Vec::new(),
            reliability_score: None,
            status: None,
        }
    }

    /// Record a probe execution
    pub fn add_probe(&mut self, probe_name: &str) {
        self.probes.push(probe_name.to_string());
    }

    /// Record a self-health action
    pub fn add_health_action(&mut self, action: &str) {
        self.health_actions.push(action.to_string());
    }

    /// Record an LLM phase
    pub fn add_llm_phase(&mut self, phase: LlmPhase) {
        self.llm_phases.push(phase);
    }

    /// Set the final result
    pub fn set_result(&mut self, reliability_score: f64, status: RequestStatus) {
        self.reliability_score = Some(reliability_score);
        self.status = Some(status);
    }

    /// Calculate duration in milliseconds
    pub fn duration_ms(&self) -> u64 {
        let now = Utc::now();
        (now - self.start_time).num_milliseconds().max(0) as u64
    }

    /// Build a request trace for logging
    pub fn to_trace(&self) -> RequestTrace {
        RequestTrace {
            request_id: self.request_id.clone(),
            timestamp_start: self.start_time,
            timestamp_end: Utc::now(),
            duration_ms: self.duration_ms(),
            user_query: self.query.clone(),
            probe_summary: self.probes.clone(),
            self_health_actions: self.health_actions.clone(),
            reliability_score: self.reliability_score.unwrap_or(0.0),
            result_status: self.status.clone().unwrap_or(RequestStatus::Failed),
        }
    }

    /// Create a log entry with this request's ID
    pub fn log_entry(
        &self,
        level: LogLevel,
        component: LogComponent,
        message: impl Into<String>,
    ) -> LogEntry {
        LogEntry::new(level, component, message).with_request_id(&self.request_id)
    }
}

/// Set the current request context for this thread
pub fn set_current_request(ctx: RequestContext) {
    CURRENT_REQUEST.with(|current| {
        *current.borrow_mut() = Some(ctx);
    });
}

/// Clear the current request context
pub fn clear_current_request() {
    CURRENT_REQUEST.with(|current| {
        *current.borrow_mut() = None;
    });
}

/// Get the current request ID (if any)
pub fn current_request_id() -> Option<String> {
    CURRENT_REQUEST.with(|current| current.borrow().as_ref().map(|c| c.request_id.clone()))
}

/// Execute a closure with access to the current request context
pub fn with_current_request<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut RequestContext) -> R,
{
    CURRENT_REQUEST.with(|current| current.borrow_mut().as_mut().map(f))
}

/// Record a probe in the current request context
pub fn record_probe(probe_name: &str) {
    with_current_request(|ctx| ctx.add_probe(probe_name));
}

/// Record a health action in the current request context
pub fn record_health_action(action: &str) {
    with_current_request(|ctx| ctx.add_health_action(action));
}

/// Record an LLM phase in the current request context
pub fn record_llm_phase(phase: LlmPhase) {
    with_current_request(|ctx| ctx.add_llm_phase(phase));
}

/// LLM trace builder for structured logging
pub struct LlmTraceBuilder {
    request_id: String,
    phase: LlmPhase,
    user_query_summary: String,
    plan_summary: Option<String>,
    probes_executed: Option<Vec<String>>,
    evidence_summary: Option<Vec<String>>,
    reliability_breakdown: Option<ReliabilityBreakdown>,
    conflicts: Option<Vec<String>>,
    final_answer_status: Option<String>,
}

impl LlmTraceBuilder {
    pub fn new(request_id: impl Into<String>, phase: LlmPhase, query: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
            phase,
            user_query_summary: query.into(),
            plan_summary: None,
            probes_executed: None,
            evidence_summary: None,
            reliability_breakdown: None,
            conflicts: None,
            final_answer_status: None,
        }
    }

    pub fn plan(mut self, plan: impl Into<String>) -> Self {
        self.plan_summary = Some(plan.into());
        self
    }

    pub fn probes(mut self, probes: Vec<String>) -> Self {
        self.probes_executed = Some(probes);
        self
    }

    pub fn evidence(mut self, evidence: Vec<String>) -> Self {
        self.evidence_summary = Some(evidence);
        self
    }

    pub fn reliability(mut self, breakdown: ReliabilityBreakdown) -> Self {
        self.reliability_breakdown = Some(breakdown);
        self
    }

    pub fn conflicts(mut self, conflicts: Vec<String>) -> Self {
        self.conflicts = Some(conflicts);
        self
    }

    pub fn final_status(mut self, status: impl Into<String>) -> Self {
        self.final_answer_status = Some(status.into());
        self
    }

    pub fn build(self) -> LlmTrace {
        LlmTrace {
            request_id: self.request_id,
            timestamp: Utc::now(),
            phase: self.phase,
            user_query_summary: self.user_query_summary,
            plan_summary: self.plan_summary,
            probes_executed: self.probes_executed,
            evidence_summary: self.evidence_summary,
            reliability_breakdown: self.reliability_breakdown,
            conflicts: self.conflicts,
            final_answer_status: self.final_answer_status,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_context_new() {
        let ctx = RequestContext::new("req-test-0001".to_string(), "test query".to_string());
        assert_eq!(ctx.request_id, "req-test-0001");
        assert_eq!(ctx.query, "test query");
        assert!(ctx.probes.is_empty());
    }

    #[test]
    fn test_request_context_add_probe() {
        let mut ctx = RequestContext::new("req-test-0001".to_string(), "test".to_string());
        ctx.add_probe("cpu.info");
        ctx.add_probe("mem.info");
        assert_eq!(ctx.probes.len(), 2);
        assert!(ctx.probes.contains(&"cpu.info".to_string()));
    }

    #[test]
    fn test_request_context_to_trace() {
        let mut ctx = RequestContext::new("req-test-0001".to_string(), "test".to_string());
        ctx.add_probe("cpu.info");
        ctx.set_result(0.95, RequestStatus::Ok);

        let trace = ctx.to_trace();
        assert_eq!(trace.request_id, "req-test-0001");
        assert_eq!(trace.probe_summary.len(), 1);
        assert_eq!(trace.reliability_score, 0.95);
        assert_eq!(trace.result_status, RequestStatus::Ok);
    }

    #[test]
    fn test_thread_local_context() {
        let ctx = RequestContext::new("req-local-0001".to_string(), "test".to_string());
        set_current_request(ctx);

        let id = current_request_id();
        assert_eq!(id, Some("req-local-0001".to_string()));

        record_probe("test.probe");
        with_current_request(|ctx| {
            assert!(ctx.probes.contains(&"test.probe".to_string()));
        });

        clear_current_request();
        assert_eq!(current_request_id(), None);
    }

    #[test]
    fn test_llm_trace_builder() {
        let trace = LlmTraceBuilder::new("req-123", LlmPhase::Planning, "How many cores?")
            .plan("Request cpu.info probe")
            .probes(vec!["cpu.info".to_string()])
            .build();

        assert_eq!(trace.request_id, "req-123");
        assert_eq!(trace.phase, LlmPhase::Planning);
        assert_eq!(trace.plan_summary, Some("Request cpu.info probe".to_string()));
    }
}
