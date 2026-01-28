//! Status reporting methods for StateInner.

use anna_shared::recipe::{RecipeBook, RecipeSource};
use anna_shared::status::{
    ActiveTicket, DepartmentTicketStats, LearningStatus, SpecialistStats, SpecialistStatus,
    TeamRoster, TicketTracker,
};
use std::collections::HashMap;

use super::types::StateInner;
use crate::department::{get_department, get_ticket_store, TicketStatus as DeptTicketStatus};

impl StateInner {
    pub fn to_status(&self) -> anna_shared::status::DaemonStatus {
        // v0.0.924: Get memory health info
        let (memory_experiences, memory_health_issues) = Self::get_memory_health();

        // LLM-first: no pattern matching, no recipes
        let pattern_count = 0;
        let recipe_count = 0;

        // v0.2.7: Get RPG stats
        let rpg_stats = anna_shared::stats::PersistentStats::load()
            .map(|s| s.get_rpg_stats())
            .unwrap_or_default();

        // v0.3.3: Get team roster and ticket stats
        let (team_roster, ticket_tracker) = Self::get_team_and_tickets();

        // v0.3.20: Get additional stats for spec compliance
        let ollama_version = Self::get_ollama_version();
        let permissions = anna_shared::status::PermissionsAudit::check();
        let escalated_tickets_count = ticket_tracker.dept_stats.values()
            .map(|s| s.escalations_out)
            .sum();
        let solved_alone_count = rpg_stats.instant_answers + rpg_stats.memory_answers;

        // v0.3.21: New full status contract fields
        let build_info = anna_shared::status::BuildMetadata::from_build_info();
        let socket_path = anna_shared::socket_path();
        let mut socket_health = anna_shared::status::SocketHealth::check(&socket_path);
        // Mark as healthy since daemon is running
        socket_health.mark_healthy();
        let config_snapshot = anna_shared::status::ConfigSnapshot::current();
        let model_mappings = anna_shared::status::ModelMapping::defaults(
            &self.model.clone().unwrap_or_else(|| "qwen2.5:7b".to_string())
        );
        let helpers = anna_shared::status::HelperInfo::check_all();

        // v0.3.24: Backup status
        let backup_info = Self::get_backup_status();

        anna_shared::status::DaemonStatus {
            state: self.state,
            version: anna_shared::VERSION.to_string(),
            ollama_running: self.ollama_running,
            model: self.model.clone(),
            uptime_secs: self.uptime_secs(),
            gpu: self.gpu.clone(),
            vram_mb: self.vram_mb,
            memory_experiences,
            memory_health_issues,
            // v0.1.0: Update timing info
            update_check_interval_secs: self.update.check_interval_secs,
            last_update_check: self.update.last_check_at.map(|t| t.to_rfc3339()),
            next_update_check: self.update.next_check_at.map(|t| t.to_rfc3339()),
            latest_version: self.update.latest_version.clone(),
            update_state: self.update.check_state,
            // v0.3.25: Auto-update enabled status
            auto_update_enabled: self.update.enabled,
            pattern_count,
            recipe_count,
            rpg_stats,
            ticket_tracker,
            team_roster,
            // v0.3.20: New fields for spec compliance
            ollama_version,
            permissions,
            escalated_tickets_count,
            solved_alone_count,
            // v0.3.21: Full status contract fields
            build_info,
            socket_health,
            error_summary: anna_shared::status::ErrorSummary::default(),
            config_snapshot,
            model_mappings,
            helpers,
            // v0.3.24: Backup status
            backup_info,
            // v0.3.29: Skill learning status with actual recipe counts
            learning_status: Self::get_learning_status(),
            // v0.3.36: Self-healing recovery metrics
            recovery_status: self.recovery_status.clone(),
        }
    }

    /// v0.3.24: Get backup status for status display
    pub(crate) fn get_backup_status() -> anna_shared::status::BackupStatus {
        use anna_shared::config::anna_data_dir;
        use std::fs;

        let backup_dir = anna_data_dir().join("backups");
        let directory = backup_dir.display().to_string();

        let mut backup_count = 0;
        let mut total_size_bytes = 0u64;
        let mut last_backup: Option<String> = None;
        let mut last_backup_time: Option<std::time::SystemTime> = None;

        if backup_dir.exists() {
            if let Ok(entries) = fs::read_dir(&backup_dir) {
                for entry in entries.flatten() {
                    if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                        backup_count += 1;

                        // Get directory size
                        if let Ok(metadata) = entry.metadata() {
                            if let Ok(modified) = metadata.modified() {
                                // Track most recent backup
                                if last_backup_time.is_none() || Some(modified) > last_backup_time {
                                    last_backup_time = Some(modified);
                                    last_backup = Some(entry.file_name().to_string_lossy().to_string());
                                }
                            }
                        }

                        // Sum file sizes in backup directory
                        if let Ok(files) = fs::read_dir(entry.path()) {
                            for file in files.flatten() {
                                if let Ok(meta) = file.metadata() {
                                    total_size_bytes += meta.len();
                                }
                            }
                        }
                    }
                }
            }
        }

        anna_shared::status::BackupStatus {
            directory,
            backup_count,
            last_backup,
            total_size_bytes,
            retention_policy: "Manual cleanup only (annactl reset does not delete old backups)".to_string(),
        }
    }

    /// v0.3.20: Get ollama version
    pub(crate) fn get_ollama_version() -> Option<String> {
        std::process::Command::new("ollama")
            .arg("--version")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| {
                // Parse "ollama version is X.X.X" or similar
                s.trim()
                    .strip_prefix("ollama version is ")
                    .or_else(|| s.trim().strip_prefix("ollama version "))
                    .unwrap_or(s.trim())
                    .to_string()
            })
    }

    /// v0.1.0: Get recipe count
    pub(crate) fn get_recipe_count() -> usize {
        RecipeBook::load().map(|s| s.recipes.len()).unwrap_or(0)
    }

    /// v0.3.29: Get learning status with skill tier counts
    pub(crate) fn get_learning_status() -> LearningStatus {
        let mut status = LearningStatus {
            enabled: true, // Learning mode enabled by default
            ..Default::default()
        };

        if let Ok(book) = RecipeBook::load() {
            for recipe in &book.recipes {
                // v0.3.29: Categorize recipes by tier based on source and success_count
                match &recipe.source {
                    RecipeSource::BuiltIn => {
                        // Built-in recipes are trusted
                        status.trusted_skills += 1;
                    }
                    RecipeSource::Learned | RecipeSource::Llm { .. } => {
                        // Learned recipes start as candidates, graduate to probation/trusted based on success
                        if recipe.success_count >= 10 {
                            status.trusted_skills += 1;
                        } else if recipe.success_count >= 3 {
                            status.probation_skills += 1;
                        } else {
                            status.candidate_skills += 1;
                        }
                    }
                    RecipeSource::Wiki { .. } => {
                        // Wiki recipes are probationary
                        status.probation_skills += 1;
                    }
                    RecipeSource::User => {
                        // User-provided recipes start as candidates
                        status.candidate_skills += 1;
                    }
                }
            }
        }

        status
    }

    /// v0.0.924: Get memory health statistics
    pub(crate) fn get_memory_health() -> (usize, Vec<String>) {
        use anna_shared::memory::Memory;

        match Memory::load() {
            Ok(memory) => {
                let experiences = memory.experiences.len();
                let health_issues = memory.health_check();
                (experiences, health_issues)
            }
            Err(_) => (0, vec!["Memory not loaded".to_string()])
        }
    }

    /// v0.3.3: Get team roster and ticket statistics for status display
    /// v0.3.5: Added per-specialist stats from ticket history
    pub(crate) fn get_team_and_tickets() -> (TeamRoster, TicketTracker) {
        let store = get_ticket_store();

        // v0.3.5: First, collect per-specialist stats from ticket history
        let mut spec_tickets: HashMap<String, (u64, u64, u64, Vec<i64>)> = HashMap::new();

        for ticket in &store.tickets {
            if let Some(ref assigned) = ticket.assigned_to {
                let entry = spec_tickets.entry(assigned.clone()).or_insert((0, 0, 0, Vec::new()));
                entry.0 += 1; // handled
                if ticket.status == DeptTicketStatus::Resolved {
                    entry.1 += 1; // resolved
                    if let Some(time) = ticket.resolution_time_secs() {
                        entry.3.push(time * 1000); // store as ms
                    }
                }
                if ticket.was_escalated {
                    entry.2 += 1; // escalated
                }
            }
        }

        // Build team roster from department with real stats
        let dept = get_department();
        let mut specialists_map: HashMap<String, Vec<SpecialistStats>> = HashMap::new();
        let mut total = 0;
        let mut available = 0;

        for specialist in &dept.specialists {
            // Look up this specialist's actual stats
            let (handled, resolved, escalated, avg_ms) = if let Some((h, r, e, times)) = spec_tickets.get(specialist.name) {
                let avg = if times.is_empty() { 0 } else {
                    times.iter().sum::<i64>() as u64 / times.len() as u64
                };
                (*h, *r, *e, avg)
            } else {
                (0, 0, 0, 0)
            };

            let stats = SpecialistStats {
                name: specialist.name.to_string(),
                department: specialist.department.to_string(),
                is_senior: specialist.role == crate::department::SpecialistRole::Senior
                    || specialist.role == crate::department::SpecialistRole::Manager,
                tickets_handled: handled,
                tickets_resolved: resolved,
                tickets_escalated: escalated,
                avg_resolution_ms: avg_ms,
                top_topics: specialist.expertise.iter().take(3).map(|s| s.to_string()).collect(),
                current_status: SpecialistStatus::Available,
            };

            specialists_map
                .entry(specialist.department.to_string())
                .or_default()
                .push(stats);

            total += 1;
            available += 1;
        }

        let team_roster = TeamRoster {
            specialists: specialists_map,
            total_specialists: total,
            available_count: available,
        };

        // Build ticket tracker from ticket store (reuse store from above)
        let mut dept_stats: HashMap<String, DepartmentTicketStats> = HashMap::new();

        // Count tickets per department
        for ticket in &store.tickets {
            let entry = dept_stats.entry(ticket.department.clone()).or_default();
            entry.total_received += 1;
            if ticket.status == DeptTicketStatus::Resolved {
                entry.resolved += 1;
            }
            if ticket.was_escalated {
                entry.escalations_out += 1;
            }
        }

        // Get active tickets
        let active_tickets: Vec<ActiveTicket> = store.get_active_tickets()
            .into_iter()
            .take(5) // Only show last 5 active
            .map(|t| ActiveTicket {
                id: t.case_number.clone(),
                summary: if t.question.len() > 40 {
                    format!("{}...", &t.question[..37])
                } else {
                    t.question.clone()
                },
                assigned_to: t.assigned_to.clone(),
                department: t.department.clone(),
                created_at: t.created_at.to_rfc3339(),
                status: match t.status {
                    DeptTicketStatus::New | DeptTicketStatus::Assigned => anna_shared::status::TicketStatus::Open,
                    DeptTicketStatus::Investigating => anna_shared::status::TicketStatus::Investigating,
                    DeptTicketStatus::Experimenting => anna_shared::status::TicketStatus::Experimenting,
                    DeptTicketStatus::InProgress | DeptTicketStatus::WaitingUser => anna_shared::status::TicketStatus::InProgress,
                    DeptTicketStatus::Escalated | DeptTicketStatus::Researching => anna_shared::status::TicketStatus::Escalated,
                    DeptTicketStatus::Resolved => anna_shared::status::TicketStatus::Resolved,
                    DeptTicketStatus::Failed => anna_shared::status::TicketStatus::Failed,
                },
            })
            .collect();

        // Get today's count from tickets
        let today = chrono::Local::now().format("%d%m%Y").to_string();
        let today_count = store.tickets.iter()
            .filter(|t| t.case_number.ends_with(&today))
            .count() as u64;

        let ticket_tracker = TicketTracker {
            next_number: store.tickets.len() as u64 + 1,
            today_count,
            current_date: today,
            active_tickets,
            dept_stats,
        };

        (team_roster, ticket_tracker)
    }
}
