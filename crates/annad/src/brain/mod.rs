//! Anna's Brain v0.11.0
//!
//! The learning and knowledge management system.
//! - Knowledge store integration
//! - Learning job processing
//! - System mapping
//! - Knowledge hygiene
//! - Event detection

pub mod event_watcher;
pub mod hygiene;
pub mod job_processor;
pub mod system_mapper;

pub use job_processor::*;

use anna_common::{KnowledgeStore, MappingState, UserTelemetry};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Anna's brain - manages knowledge and learning
pub struct AnnaBrain {
    /// Knowledge store
    pub store: Arc<RwLock<KnowledgeStore>>,
    /// User telemetry
    pub telemetry: Arc<RwLock<UserTelemetry>>,
    /// System mapping state
    pub mapping_state: Arc<RwLock<MappingState>>,
    /// Job processor
    pub job_processor: Arc<JobProcessor>,
}

impl AnnaBrain {
    /// Initialize Anna's brain
    pub fn new(store: KnowledgeStore) -> Self {
        let store = Arc::new(RwLock::new(store));
        let telemetry = Arc::new(RwLock::new(UserTelemetry::new()));
        let mapping_state = Arc::new(RwLock::new(MappingState::new()));
        let job_processor = Arc::new(JobProcessor::new(store.clone()));

        Self {
            store,
            telemetry,
            mapping_state,
            job_processor,
        }
    }

    /// Start background learning tasks
    pub async fn start_background_tasks(&self) {
        // Check if initial mapping is needed
        let needs_mapping = {
            let state = self.mapping_state.read().await;
            state.needs_initial_mapping()
        };

        if needs_mapping {
            tracing::info!("First run detected - scheduling initial system mapping");
            self.schedule_initial_mapping().await;
        }

        // Schedule periodic hygiene check
        self.schedule_hygiene_check().await;
    }

    /// Schedule initial system mapping
    async fn schedule_initial_mapping(&self) {
        use anna_common::{LearningEvent, LearningJob, MappingPhase};

        for phase in MappingPhase::all() {
            let event = LearningEvent::InitialMapping { phase };
            let job = LearningJob::new(event);
            self.job_processor.enqueue(job).await;
        }
    }

    /// Schedule knowledge hygiene check
    async fn schedule_hygiene_check(&self) {
        use anna_common::{LearningEvent, LearningJob};

        let event = LearningEvent::ScheduledRefresh {
            target: "hygiene".to_string(),
        };
        let job = LearningJob::new(event);
        self.job_processor.enqueue(job).await;
    }

    /// Record a user query for telemetry
    pub async fn record_query(&self, query: &str) {
        let mut telemetry = self.telemetry.write().await;
        telemetry.record_query(query);
    }

    /// Get proactive notices
    pub async fn get_notices(&self) -> Vec<anna_common::ProactiveNotice> {
        // TODO: Implement notice generation based on recent findings
        Vec::new()
    }
}
