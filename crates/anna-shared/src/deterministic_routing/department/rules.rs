//! Department Rules - v0.0.439.
//!
//! Enforces department ownership rules and handles conflicts.

use super::super::intent_map_table::IntentMapTable;
use super::super::intent_schema::{CanonicalIntent, Department, TicketIntentSchema};
use super::conflict::DepartmentConflict;
use super::ownership::{build_ownerships, DepartmentOwnership};

/// Department ownership rules.
pub struct DepartmentRules {
    /// Ownership definitions.
    ownerships: Vec<DepartmentOwnership>,
    /// Intent map for authoritative lookups.
    intent_map: IntentMapTable,
}

impl DepartmentRules {
    /// Create new rules.
    pub fn new() -> Self {
        Self {
            ownerships: build_ownerships(),
            intent_map: IntentMapTable::build(),
        }
    }

    /// Get the authoritative department for an intent.
    /// This is the CANONICAL source - overrides any translator suggestion.
    pub fn get_authoritative_department(&self, intent: CanonicalIntent) -> Department {
        self.intent_map.get_department(intent)
    }

    /// Check if translator department conflicts with canonical mapping.
    pub fn check_conflict(
        &self,
        intent: CanonicalIntent,
        translator_dept: Department,
    ) -> Option<DepartmentConflict> {
        let canonical = self.get_authoritative_department(intent);

        if canonical != translator_dept {
            Some(DepartmentConflict {
                intent,
                translator_suggested: translator_dept,
                canonical_department: canonical,
            })
        } else {
            None
        }
    }

    /// Override translator department if it conflicts with canonical mapping.
    /// Returns the corrected schema and optional conflict log.
    pub fn enforce_ownership(
        &self,
        mut schema: TicketIntentSchema,
    ) -> (TicketIntentSchema, Option<DepartmentConflict>) {
        let canonical = self.get_authoritative_department(schema.intent);

        if schema.department != canonical {
            let conflict = DepartmentConflict {
                intent: schema.intent,
                translator_suggested: schema.department,
                canonical_department: canonical,
            };
            schema.department = canonical;
            (schema, Some(conflict))
        } else {
            (schema, None)
        }
    }

    /// Get department that owns a keyword.
    pub fn department_for_keyword(&self, keyword: &str) -> Option<Department> {
        let keyword_lower = keyword.to_lowercase();
        for ownership in &self.ownerships {
            for kw in &ownership.keywords {
                if keyword_lower.contains(kw) || kw.contains(&keyword_lower) {
                    return Some(ownership.department);
                }
            }
        }
        None
    }

    /// Get ownership info for a department.
    pub fn get_ownership(&self, dept: Department) -> Option<&DepartmentOwnership> {
        self.ownerships.iter().find(|o| o.department == dept)
    }

    /// List all topics owned by a department.
    pub fn topics_for_department(&self, dept: Department) -> Vec<&str> {
        self.ownerships
            .iter()
            .find(|o| o.department == dept)
            .map(|o| o.owns_topics.clone())
            .unwrap_or_default()
    }
}

impl Default for DepartmentRules {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boot_routes_to_performance() {
        let rules = DepartmentRules::new();
        let dept = rules.get_authoritative_department(CanonicalIntent::BootPerf);
        assert_eq!(dept, Department::Performance);
    }

    #[test]
    fn test_gpu_routes_to_hardware() {
        let rules = DepartmentRules::new();
        assert_eq!(
            rules.get_authoritative_department(CanonicalIntent::GpuInfo),
            Department::Hardware
        );
        assert_eq!(
            rules.get_authoritative_department(CanonicalIntent::GpuDriver),
            Department::Hardware
        );
    }

    #[test]
    fn test_disk_routes_to_storage() {
        let rules = DepartmentRules::new();
        assert_eq!(
            rules.get_authoritative_department(CanonicalIntent::DiskUsage),
            Department::Storage
        );
    }

    #[test]
    fn test_ram_routes_to_performance() {
        let rules = DepartmentRules::new();
        assert_eq!(
            rules.get_authoritative_department(CanonicalIntent::MemStatus),
            Department::Performance
        );
    }

    #[test]
    fn test_conflict_detection() {
        let rules = DepartmentRules::new();

        // Boot should be Performance, not Desktop
        let conflict = rules.check_conflict(CanonicalIntent::BootPerf, Department::Desktop);
        assert!(conflict.is_some());
        let c = conflict.unwrap();
        assert_eq!(c.translator_suggested, Department::Desktop);
        assert_eq!(c.canonical_department, Department::Performance);
    }

    #[test]
    fn test_no_conflict_when_correct() {
        let rules = DepartmentRules::new();

        // GPU to Hardware is correct
        let conflict = rules.check_conflict(CanonicalIntent::GpuInfo, Department::Hardware);
        assert!(conflict.is_none());
    }

    #[test]
    fn test_enforce_ownership_override() {
        let rules = DepartmentRules::new();

        // Translator wrongly says Desktop for boot
        let schema = TicketIntentSchema::new(
            "why is boot slow?",
            CanonicalIntent::BootPerf,
            Department::Desktop, // WRONG
        );

        let (corrected, conflict) = rules.enforce_ownership(schema);
        assert_eq!(corrected.department, Department::Performance); // Fixed
        assert!(conflict.is_some());
    }

    #[test]
    fn test_department_for_keyword() {
        let rules = DepartmentRules::new();

        assert_eq!(
            rules.department_for_keyword("gpu"),
            Some(Department::Hardware)
        );
        assert_eq!(
            rules.department_for_keyword("boot"),
            Some(Department::Performance)
        );
        assert_eq!(
            rules.department_for_keyword("disk"),
            Some(Department::Storage)
        );
        assert_eq!(
            rules.department_for_keyword("firewall"),
            Some(Department::Security)
        );
    }
}
