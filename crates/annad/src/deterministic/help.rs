//! Help response (v0.0.176).

use super::DeterministicResult;

/// Help response describing available commands (v0.0.116: removed email, added inbox)
pub fn answer_help(route_class: &str) -> DeterministicResult {
    let answer = r#"**Anna - Linux System Assistant**

I can answer questions about your system:

**Hardware:** "What CPU?", "How much RAM?", "What GPU?"
**Processes:** "Top memory processes", "What's using CPU?"
**Storage:** "Disk space", "How full is my disk?"
**Network:** "Network interfaces", "What's my IP?"
**Health:** "System health", "Any errors?", "Status report"
**Diagnostics:** "It's slow" - Full diagnostic

**Service Desk:**
- "Who is on shift?" - Meet the IT team
- "Show my tickets" - Check ticket history

**Ways to reach me:**
- `annactl "question"` - One-shot query (immediate)
- `annactl` - Interactive REPL (immediate)
- `~/.anna/inbox` - Async queries (creates tickets)

**Commands:**
- `annactl status` - Daemon status
- `annactl stats` - Team statistics

To set your email: just tell me \"my email is you@example.com\"

Ask a question to get started!"#;

    DeterministicResult {
        answer: answer.to_string(),
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    }
}
