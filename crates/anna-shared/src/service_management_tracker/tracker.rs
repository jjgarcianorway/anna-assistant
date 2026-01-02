//! Service tracker implementation - Phase 81

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::{OperationResult, ServiceOperation, ServiceRecord};

/// Service management tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceTracker {
    /// All operation records
    pub records: Vec<ServiceRecord>,
    /// Count by operation type
    pub by_operation: HashMap<String, u64>,
    /// Count by service
    pub by_service: HashMap<String, u64>,
    /// Success count
    pub success_count: u64,
    /// Failure count
    pub failure_count: u64,
}

impl ServiceTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a service operation
    pub fn record(&mut self, record: ServiceRecord) {
        let op_key = format!("{:?}", record.operation);
        *self.by_operation.entry(op_key).or_insert(0) += 1;
        *self.by_service.entry(record.service_name.clone()).or_insert(0) += 1;

        match record.result {
            OperationResult::Success => self.success_count += 1,
            OperationResult::Failed => self.failure_count += 1,
            _ => {}
        }

        self.records.push(record);
    }

    /// Get recent operations
    pub fn recent(&self, limit: usize) -> Vec<&ServiceRecord> {
        self.records.iter().rev().take(limit).collect()
    }

    /// Get operations by type
    pub fn by_operation_type(&self, op: ServiceOperation) -> Vec<&ServiceRecord> {
        self.records.iter().filter(|r| r.operation == op).collect()
    }

    /// Get operations for a service
    pub fn for_service(&self, name: &str) -> Vec<&ServiceRecord> {
        self.records.iter().filter(|r| r.service_name == name).collect()
    }

    /// Get failed operations
    pub fn failed(&self) -> Vec<&ServiceRecord> {
        self.records
            .iter()
            .filter(|r| r.result == OperationResult::Failed)
            .collect()
    }

    /// Get successful operations
    pub fn successful(&self) -> Vec<&ServiceRecord> {
        self.records
            .iter()
            .filter(|r| r.result == OperationResult::Success)
            .collect()
    }

    /// Total operation count
    pub fn total_count(&self) -> usize {
        self.records.len()
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        let total = self.success_count + self.failure_count;
        if total == 0 {
            return 100.0;
        }
        (self.success_count as f64 / total as f64) * 100.0
    }

    /// Most managed service
    pub fn most_managed(&self) -> Option<(&str, u64)> {
        self.by_service
            .iter()
            .max_by_key(|(_, v)| *v)
            .map(|(k, v)| (k.as_str(), *v))
    }

    /// Most common operation
    pub fn most_common_op(&self) -> Option<(&str, u64)> {
        self.by_operation
            .iter()
            .max_by_key(|(_, v)| *v)
            .map(|(k, v)| (k.as_str(), *v))
    }

    /// Unique services managed
    pub fn unique_services(&self) -> usize {
        self.by_service.len()
    }
}
