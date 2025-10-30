//! Event Reaction System - Structured event handling and policy-driven reactions
//!
//! Sprint 3: Intelligence, Policies & Event Reactions
//!
//! Handles internal events from telemetry, config changes, and doctor results.
//! Links to Policy Engine to trigger appropriate reactions.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

use crate::policy::{PolicyAction, PolicyContext, PolicyEngine};

/// Event system errors
#[derive(Debug, Error)]
pub enum EventError {
    #[error("Event dispatch failed: {0}")]
    DispatchError(String),

    #[error("Handler registration failed: {0}")]
    HandlerError(String),

    #[error("Policy evaluation failed: {0}")]
    PolicyError(String),
}

/// Event severity level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum EventSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Event type categorization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    TelemetryAlert,
    ConfigChange,
    DoctorResult,
    AutonomyAction,
    PolicyTriggered,
    SystemStartup,
    SystemShutdown,
    Custom(String),
}

/// Structured event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub timestamp: u64,
    pub event_type: EventType,
    pub severity: EventSeverity,
    pub source: String,
    pub message: String,
    pub metadata: serde_json::Value,
}

impl Event {
    /// Create a new event
    pub fn new(
        event_type: EventType,
        severity: EventSeverity,
        source: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id: format!("{}-{}", timestamp, uuid::Uuid::new_v4()),
            timestamp,
            event_type,
            severity,
            source: source.into(),
            message: message.into(),
            metadata: serde_json::json!({}),
        }
    }

    /// Add metadata to the event
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Event reaction result
#[derive(Debug, Clone)]
pub struct ReactionResult {
    pub event_id: String,
    pub actions_taken: Vec<PolicyAction>,
    pub success: bool,
    pub error: Option<String>,
}

/// Event handler callback
type EventHandler = Arc<dyn Fn(&Event) -> Result<(), EventError> + Send + Sync>;

/// Event dispatcher - central event bus
pub struct EventDispatcher {
    handlers: Arc<Mutex<Vec<EventHandler>>>,
    event_history: Arc<Mutex<VecDeque<Event>>>,
    policy_engine: Arc<PolicyEngine>,
    max_history: usize,
}

impl EventDispatcher {
    /// Create a new event dispatcher
    pub fn new(policy_engine: Arc<PolicyEngine>) -> Self {
        Self {
            handlers: Arc::new(Mutex::new(Vec::new())),
            event_history: Arc::new(Mutex::new(VecDeque::new())),
            policy_engine,
            max_history: 1000, // Keep last 1000 events
        }
    }

    /// Register an event handler
    pub fn register_handler<F>(&self, handler: F) -> Result<(), EventError>
    where
        F: Fn(&Event) -> Result<(), EventError> + Send + Sync + 'static,
    {
        let mut handlers = self.handlers.lock().unwrap();
        handlers.push(Arc::new(handler));
        Ok(())
    }

    /// Dispatch an event to all handlers and evaluate policies
    pub fn dispatch(&self, event: Event) -> Result<ReactionResult, EventError> {
        // Add to history
        {
            let mut history = self.event_history.lock().unwrap();
            history.push_back(event.clone());
            if history.len() > self.max_history {
                history.pop_front();
            }
        }

        // Call all registered handlers
        let handlers = self.handlers.lock().unwrap().clone();
        for handler in handlers.iter() {
            if let Err(e) = handler(&event) {
                eprintln!("Event handler error: {}", e);
            }
        }

        // Evaluate policies based on the event
        let context = self.build_policy_context(&event);
        let policy_result = self.policy_engine.evaluate(&context)
            .map_err(|e| EventError::PolicyError(e.to_string()))?;

        let mut result = ReactionResult {
            event_id: event.id.clone(),
            actions_taken: Vec::new(),
            success: true,
            error: None,
        };

        // Execute policy actions
        if policy_result.matched {
            for action in policy_result.actions {
                match self.execute_action(&action, &event) {
                    Ok(_) => {
                        result.actions_taken.push(action);
                    }
                    Err(e) => {
                        result.success = false;
                        result.error = Some(format!("Action execution failed: {}", e));
                        eprintln!("Failed to execute action {:?}: {}", action, e);
                    }
                }
            }

            // Log policy trigger event
            if !result.actions_taken.is_empty() {
                let policy_event = Event::new(
                    EventType::PolicyTriggered,
                    EventSeverity::Info,
                    "policy_engine",
                    format!("Policy triggered {} actions", result.actions_taken.len()),
                ).with_metadata(serde_json::json!({
                    "original_event": event.id,
                    "actions": result.actions_taken,
                }));

                let mut history = self.event_history.lock().unwrap();
                history.push_back(policy_event);
            }
        }

        Ok(result)
    }

    /// Build policy context from event
    fn build_policy_context(&self, event: &Event) -> PolicyContext {
        let mut context = PolicyContext::new();

        // Add event-specific metrics
        context.set_string("event.type", format!("{:?}", event.event_type));
        context.set_string("event.severity", format!("{:?}", event.severity));
        context.set_string("event.source", event.source.clone());

        // Extract metadata into context
        if let Some(obj) = event.metadata.as_object() {
            for (key, value) in obj {
                match value {
                    serde_json::Value::Number(n) => {
                        if let Some(f) = n.as_f64() {
                            context.set_metric(&format!("event.{}", key), f);
                        }
                    }
                    serde_json::Value::Bool(b) => {
                        context.set_flag(&format!("event.{}", key), *b);
                    }
                    serde_json::Value::String(s) => {
                        context.set_string(&format!("event.{}", key), s.clone());
                    }
                    _ => {}
                }
            }
        }

        context
    }

    /// Execute a policy action
    fn execute_action(&self, action: &PolicyAction, event: &Event) -> Result<(), EventError> {
        match action {
            PolicyAction::DisableAutonomy => {
                println!("Policy Action: Disabling autonomy due to event: {}", event.message);
                // Integration hook: call autonomy module
                Ok(())
            }
            PolicyAction::EnableAutonomy => {
                println!("Policy Action: Enabling autonomy due to event: {}", event.message);
                // Integration hook: call autonomy module
                Ok(())
            }
            PolicyAction::RunDoctor => {
                println!("Policy Action: Running doctor diagnostics due to event: {}", event.message);
                // Integration hook: call diagnostics module
                Ok(())
            }
            PolicyAction::RestartService => {
                println!("Policy Action: Restarting service due to event: {}", event.message);
                // Integration hook: restart mechanism
                Ok(())
            }
            PolicyAction::SendAlert => {
                println!("Policy Action: Sending alert for event: {}", event.message);
                // Integration hook: alerting system
                Ok(())
            }
            PolicyAction::Custom(cmd) => {
                println!("Policy Action: Executing custom action '{}' due to event: {}", cmd, event.message);
                // Integration hook: custom action executor
                Ok(())
            }
        }
    }

    /// Get recent events
    pub fn get_recent_events(&self, count: usize) -> Vec<Event> {
        let history = self.event_history.lock().unwrap();
        history.iter()
            .rev()
            .take(count)
            .cloned()
            .collect()
    }

    /// Get events filtered by type
    pub fn get_events_by_type(&self, event_type: &EventType) -> Vec<Event> {
        let history = self.event_history.lock().unwrap();
        history.iter()
            .filter(|e| &e.event_type == event_type)
            .cloned()
            .collect()
    }

    /// Get events filtered by severity
    pub fn get_events_by_severity(&self, min_severity: EventSeverity) -> Vec<Event> {
        let history = self.event_history.lock().unwrap();
        history.iter()
            .filter(|e| e.severity >= min_severity)
            .cloned()
            .collect()
    }

    /// Clear event history
    pub fn clear_history(&self) {
        let mut history = self.event_history.lock().unwrap();
        history.clear();
    }

    /// Get total event count
    pub fn event_count(&self) -> usize {
        let history = self.event_history.lock().unwrap();
        history.len()
    }
}

/// Event reactor - high-level event reaction coordinator
pub struct EventReactor {
    dispatcher: Arc<EventDispatcher>,
}

impl EventReactor {
    /// Create a new event reactor
    pub fn new(policy_engine: Arc<PolicyEngine>) -> Self {
        let dispatcher = Arc::new(EventDispatcher::new(policy_engine));

        // Register default telemetry handler
        dispatcher.register_handler(|event: &Event| {
            if event.severity >= EventSeverity::Warning {
                println!("[EVENT] {} - {}: {}",
                    format!("{:?}", event.severity).to_uppercase(),
                    event.source,
                    event.message
                );
            }
            Ok(())
        }).unwrap();

        Self { dispatcher }
    }

    /// Get dispatcher reference
    pub fn dispatcher(&self) -> Arc<EventDispatcher> {
        self.dispatcher.clone()
    }

    /// Handle telemetry alert
    pub fn handle_telemetry_alert(
        &self,
        metric: &str,
        value: f64,
        threshold: f64,
    ) -> Result<ReactionResult, EventError> {
        let event = Event::new(
            EventType::TelemetryAlert,
            EventSeverity::Warning,
            "telemetry",
            format!("Metric '{}' exceeded threshold: {} > {}", metric, value, threshold),
        ).with_metadata(serde_json::json!({
            "metric": metric,
            "value": value,
            "threshold": threshold,
        }));

        self.dispatcher.dispatch(event)
    }

    /// Handle config change
    pub fn handle_config_change(
        &self,
        key: &str,
        old_value: &str,
        new_value: &str,
    ) -> Result<ReactionResult, EventError> {
        let event = Event::new(
            EventType::ConfigChange,
            EventSeverity::Info,
            "config",
            format!("Configuration '{}' changed", key),
        ).with_metadata(serde_json::json!({
            "key": key,
            "old_value": old_value,
            "new_value": new_value,
        }));

        self.dispatcher.dispatch(event)
    }

    /// Handle doctor result
    pub fn handle_doctor_result(
        &self,
        passed: bool,
        failures: Vec<String>,
    ) -> Result<ReactionResult, EventError> {
        let severity = if passed {
            EventSeverity::Info
        } else {
            EventSeverity::Warning
        };

        let event = Event::new(
            EventType::DoctorResult,
            severity,
            "diagnostics",
            if passed {
                "All diagnostic checks passed".to_string()
            } else {
                format!("Diagnostic checks failed: {} issues", failures.len())
            },
        ).with_metadata(serde_json::json!({
            "passed": passed,
            "failures": failures,
        }));

        self.dispatcher.dispatch(event)
    }

    /// Handle autonomy action
    pub fn handle_autonomy_action(
        &self,
        action: &str,
        success: bool,
    ) -> Result<ReactionResult, EventError> {
        let severity = if success {
            EventSeverity::Info
        } else {
            EventSeverity::Error
        };

        let event = Event::new(
            EventType::AutonomyAction,
            severity,
            "autonomy",
            format!("Autonomy action '{}' {}", action, if success { "succeeded" } else { "failed" }),
        ).with_metadata(serde_json::json!({
            "action": action,
            "success": success,
        }));

        self.dispatcher.dispatch(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::PolicyEngine;
    use std::sync::Arc;
    use tempfile::TempDir;

    #[test]
    fn test_event_creation() {
        let event = Event::new(
            EventType::TelemetryAlert,
            EventSeverity::Warning,
            "test",
            "Test message",
        );

        assert_eq!(event.event_type, EventType::TelemetryAlert);
        assert_eq!(event.severity, EventSeverity::Warning);
        assert_eq!(event.source, "test");
        assert_eq!(event.message, "Test message");
    }

    #[test]
    fn test_event_dispatch() {
        let temp_dir = TempDir::new().unwrap();
        let policy_engine = Arc::new(PolicyEngine::new(temp_dir.path()));
        let dispatcher = EventDispatcher::new(policy_engine);

        let event = Event::new(
            EventType::TelemetryAlert,
            EventSeverity::Warning,
            "test",
            "Test alert",
        );

        let result = dispatcher.dispatch(event.clone()).unwrap();
        assert_eq!(result.event_id, event.id);
        assert_eq!(dispatcher.event_count(), 1);
    }

    #[test]
    fn test_event_filtering() {
        let temp_dir = TempDir::new().unwrap();
        let policy_engine = Arc::new(PolicyEngine::new(temp_dir.path()));
        let dispatcher = EventDispatcher::new(policy_engine);

        let event1 = Event::new(EventType::TelemetryAlert, EventSeverity::Info, "test", "Info");
        let event2 = Event::new(EventType::ConfigChange, EventSeverity::Warning, "test", "Warning");
        let event3 = Event::new(EventType::TelemetryAlert, EventSeverity::Error, "test", "Error");

        dispatcher.dispatch(event1).unwrap();
        dispatcher.dispatch(event2).unwrap();
        dispatcher.dispatch(event3).unwrap();

        let alerts = dispatcher.get_events_by_type(&EventType::TelemetryAlert);
        assert_eq!(alerts.len(), 2);

        let warnings = dispatcher.get_events_by_severity(EventSeverity::Warning);
        assert_eq!(warnings.len(), 2); // Warning and Error
    }
}
