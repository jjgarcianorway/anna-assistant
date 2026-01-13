//! Daemon state management.

use anna_shared::session::{Session, SessionStore};
use anna_shared::status::{DaemonState, UpdateCheckState};
use anna_shared::{DEFAULT_UPDATE_CHECK_INTERVAL, VERSION};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Cache expiration time in seconds (5 minutes)
const ANSWER_CACHE_TTL_SECS: u64 = 300;

/// Shared daemon state handle
#[derive(Clone)]
pub struct SharedState {
    inner: Arc<RwLock<StateInner>>,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(StateInner::new())),
        }
    }

    pub async fn read(&self) -> tokio::sync::RwLockReadGuard<'_, StateInner> {
        self.inner.read().await
    }

    pub async fn write(&self) -> tokio::sync::RwLockWriteGuard<'_, StateInner> {
        self.inner.write().await
    }

    /// Wait for active connections to drain before restart
    /// Returns true if drained, false if timeout
    pub async fn wait_for_connections_to_drain(&self, timeout_secs: u64) -> bool {
        use tokio::time::{sleep, Duration};

        // Signal that restart is pending
        {
            let mut state = self.write().await;
            state.restart_pending = true;
        }

        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(timeout_secs);

        loop {
            let active = {
                let state = self.read().await;
                state.active_connections
            };

            if active == 0 {
                tracing::info!("All connections drained, safe to restart");
                return true;
            }

            if start.elapsed() > timeout {
                tracing::warn!("Timeout waiting for {} connections to drain, restarting anyway", active);
                return false;
            }

            tracing::info!("Waiting for {} active connection(s) to finish before restart...", active);
            sleep(Duration::from_secs(2)).await;
        }
    }
}

impl Default for SharedState {
    fn default() -> Self {
        Self::new()
    }
}

/// Inner state
pub struct StateInner {
    pub state: DaemonState,
    pub started_at: Instant,
    pub ollama_running: bool,
    pub model: Option<String>,
    pub last_error: Option<String>,
    pub update: UpdateState,
    pub gpu: Option<String>,
    pub vram_mb: Option<u64>,
    /// Persistent session storage
    pub sessions: SessionStore,
    /// Number of active connections (for graceful shutdown)
    pub active_connections: u32,
    /// Flag indicating restart is pending (clients should finish quickly)
    pub restart_pending: bool,
    /// Counter for periodic session saves
    session_save_counter: u32,
    /// Answer cache for identical questions (normalized question -> (answer, timestamp))
    answer_cache: HashMap<String, CachedAnswer>,
    // v0.0.891: Removed command_cache - consolidated into core_loop.rs COMMAND_CACHE
}

/// A cached answer with timestamp
#[derive(Clone)]
pub struct CachedAnswer {
    pub answer: String,
    pub cached_at: Instant,
}

// v0.0.891: Removed duplicate CachedCommandOutput struct - using core_loop.rs cache instead

/// Commands that rarely change and can be cached longer (5 minutes)
/// Used by core_loop.rs for cache TTL decisions
/// v0.0.928: Expanded list for better cache hit rate
pub const STATIC_COMMANDS: &[&str] = &[
    // System info
    "uname -r",
    "uname -a",
    "uname -m",
    "cat /etc/os-release",
    "hostnamectl",
    "hostname",
    // Hardware
    "lscpu",
    "lsblk",
    "lspci",
    "lsusb",
    "lsmod",
    "cat /proc/cpuinfo",
    "cat /proc/meminfo",
    // GPU info
    "lspci | grep -i vga",
    "lspci | grep -i nvidia",
    "lspci | grep -i amd",
    // Package info
    "pacman -Q",
    "pacman -Qe",
    "pacman -Qm",
    // Resource usage (semi-static - changes slowly)
    "free -h",
    "df -h",
    "findmnt",
    // Config files (rarely change)
    "cat /etc/fstab",
    "cat /etc/hostname",
    "cat /etc/locale.conf",
    "cat /etc/vconsole.conf",
    "cat /etc/mkinitcpio.conf",
    // Kernel
    "cat /proc/cmdline",
    "cat /proc/version",
    // Network config (changes infrequently)
    "cat /etc/resolv.conf",
    "ip link",
    // Desktop environment
    "echo $XDG_SESSION_TYPE",
    "echo $XDG_CURRENT_DESKTOP",
];

impl StateInner {
    pub fn new() -> Self {
        // Load persisted sessions from disk
        let sessions = match SessionStore::load() {
            Ok(store) => {
                let count = store.active_count();
                if count > 0 {
                    info!("Loaded {} persisted sessions from disk", count);
                }
                store
            }
            Err(e) => {
                warn!("Failed to load persisted sessions: {}", e);
                SessionStore::new()
            }
        };

        Self {
            state: DaemonState::Starting,
            started_at: Instant::now(),
            ollama_running: false,
            model: None,
            last_error: None,
            update: UpdateState::default(),
            gpu: None,
            vram_mb: None,
            sessions,
            active_connections: 0,
            restart_pending: false,
            session_save_counter: 0,
            answer_cache: HashMap::new(),
        }
    }

    /// Normalize a question for cache lookup (lowercase, trim, remove punctuation)
    fn normalize_question(question: &str) -> String {
        question
            .to_lowercase()
            .trim()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Get a cached answer if available and not expired
    pub fn get_cached_answer(&self, question: &str) -> Option<String> {
        let key = Self::normalize_question(question);
        if let Some(cached) = self.answer_cache.get(&key) {
            if cached.cached_at.elapsed().as_secs() < ANSWER_CACHE_TTL_SECS {
                debug!("Cache hit for question: {}", question);
                return Some(cached.answer.clone());
            }
        }
        None
    }

    /// Cache an answer for a question
    pub fn cache_answer(&mut self, question: &str, answer: &str) {
        let key = Self::normalize_question(question);
        self.answer_cache.insert(key, CachedAnswer {
            answer: answer.to_string(),
            cached_at: Instant::now(),
        });

        // Cleanup old entries periodically (keep max 100)
        if self.answer_cache.len() > 100 {
            self.cleanup_answer_cache();
        }
    }

    /// Remove expired cache entries
    fn cleanup_answer_cache(&mut self) {
        self.answer_cache.retain(|_, cached| {
            cached.cached_at.elapsed().as_secs() < ANSWER_CACHE_TTL_SECS
        });
    }

    // v0.0.891: Removed duplicate command cache methods - using core_loop.rs cache

    /// Get or create a session for a client
    pub fn get_or_create_session(&mut self, client_id: &str) -> &mut Session {
        self.sessions.get_or_create(client_id)
    }

    /// Cleanup sessions and optionally persist to disk
    /// Saves every 5 interactions to avoid excessive disk writes
    /// v0.0.896: Also mines patterns for recurring issue detection
    pub fn cleanup_sessions(&mut self) {
        self.sessions.cleanup_old_sessions();

        // Increment counter and save periodically
        self.session_save_counter += 1;
        if self.session_save_counter >= 5 {
            self.session_save_counter = 0;

            // v0.0.896: Mine patterns from session history for better suggestions
            self.sessions.mine_patterns();

            if let Err(e) = self.sessions.save() {
                warn!("Failed to persist sessions: {}", e);
            }
        }
    }

    /// v0.0.896: Check if a question matches a recurring issue pattern
    pub fn check_recurring_issue(&self, question: &str) -> Option<String> {
        self.sessions.is_recurring_issue(question)
            .map(|issue| format!(
                "This looks like a recurring issue: {} ({} occurrences, last seen: {})",
                issue.description, issue.occurrences, issue.last_seen
            ))
    }

    /// Force save sessions to disk (called on shutdown)
    pub fn save_sessions(&mut self) {
        if let Err(e) = self.sessions.save() {
            warn!("Failed to save sessions on shutdown: {}", e);
        } else {
            info!("Sessions persisted to disk");
        }
    }

    /// Increment active connection count
    pub fn connection_started(&mut self) {
        self.active_connections = self.active_connections.saturating_add(1);
    }

    /// Decrement active connection count
    pub fn connection_ended(&mut self) {
        self.active_connections = self.active_connections.saturating_sub(1);
    }

    pub fn uptime_secs(&self) -> u64 {
        self.started_at.elapsed().as_secs()
    }

    pub fn to_status(&self) -> anna_shared::status::DaemonStatus {
        // v0.0.924: Get memory health info
        let (memory_experiences, memory_health_issues) = Self::get_memory_health();

        // v0.1.0: Get pattern and recipe counts
        let pattern_count = crate::patterns::total_pattern_count();
        let recipe_count = Self::get_recipe_count();

        // v0.2.7: Get RPG stats
        let rpg_stats = anna_shared::stats::PersistentStats::load()
            .map(|s| s.get_rpg_stats())
            .unwrap_or_default();

        // v0.3.3: Get team roster and ticket stats
        let (team_roster, ticket_tracker) = Self::get_team_and_tickets();

        anna_shared::status::DaemonStatus {
            state: self.state,
            version: VERSION.to_string(),
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
            pattern_count,
            recipe_count,
            rpg_stats,
            ticket_tracker,
            team_roster,
        }
    }

    /// v0.1.0: Get recipe count
    fn get_recipe_count() -> usize {
        use anna_shared::recipe::RecipeBook;
        RecipeBook::load().map(|s| s.recipes.len()).unwrap_or(0)
    }

    /// v0.0.924: Get memory health statistics
    fn get_memory_health() -> (usize, Vec<String>) {
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
    fn get_team_and_tickets() -> (anna_shared::status::TeamRoster, anna_shared::status::TicketTracker) {
        use anna_shared::status::{TeamRoster, TicketTracker, SpecialistStats, SpecialistStatus, DepartmentTicketStats, ActiveTicket};
        use crate::department::{get_department, get_ticket_store, TicketStatus as DeptTicketStatus};
        use std::collections::HashMap;

        let store = get_ticket_store();

        // v0.3.5: First, collect per-specialist stats from ticket history
        let mut spec_tickets: HashMap<String, (u64, u64, u64, Vec<i64>)> = HashMap::new(); // (handled, resolved, escalated, resolution_times)

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

impl Default for StateInner {
    fn default() -> Self {
        Self::new()
    }
}

/// Update state
pub struct UpdateState {
    pub enabled: bool,
    pub check_interval_secs: u64,
    pub last_check_at: Option<DateTime<Utc>>,
    pub next_check_at: Option<DateTime<Utc>>,
    pub latest_version: Option<String>,
    pub latest_checked_at: Option<DateTime<Utc>>,
    pub update_available: bool,
    pub check_state: UpdateCheckState,
}

impl Default for UpdateState {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval_secs: DEFAULT_UPDATE_CHECK_INTERVAL,
            last_check_at: None,
            next_check_at: None,
            latest_version: None,
            latest_checked_at: None,
            update_available: false,
            check_state: UpdateCheckState::NeverChecked,
        }
    }
}
