//! Session context for REPL memory.

/// Session context for REPL memory
#[derive(Debug, Clone, Default)]
pub struct SessionContext {
    /// Questions asked in this session
    pub questions: Vec<String>,
    /// Last domain context
    pub last_domain: Option<String>,
    /// Last ticket ID
    pub last_ticket_id: Option<String>,
    /// Session start time
    pub started_at: u64,
}

impl SessionContext {
    pub fn new() -> Self {
        Self {
            started_at: now_secs(),
            ..Default::default()
        }
    }

    /// Record a question
    pub fn add_question(&mut self, question: &str, domain: Option<&str>, ticket_id: Option<&str>) {
        self.questions.push(question.to_string());
        if self.questions.len() > 10 {
            self.questions.remove(0); // Keep last 10
        }
        if let Some(d) = domain {
            self.last_domain = Some(d.to_string());
        }
        if let Some(t) = ticket_id {
            self.last_ticket_id = Some(t.to_string());
        }
    }

    /// Get session duration
    pub fn duration_secs(&self) -> u64 {
        now_secs().saturating_sub(self.started_at)
    }

    /// Context hint for internal comms
    pub fn context_hint(&self) -> Option<String> {
        if let Some(ref domain) = self.last_domain {
            Some(format!("Continuing from {} context", domain))
        } else {
            None
        }
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
