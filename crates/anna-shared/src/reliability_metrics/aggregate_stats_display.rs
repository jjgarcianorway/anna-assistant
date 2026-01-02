//! Display and formatting methods for reliability statistics (v0.0.444).

use super::aggregate_stats_types::ReliabilityStats;

impl ReliabilityStats {
    /// Format for display.
    pub fn display(&self) -> String {
        let mut out = String::new();
        out.push_str("[requests]\n");
        out.push_str(&format!(
            "  total_requests         {}\n",
            self.total_requests
        ));
        out.push_str(&format!(
            "  answered_verified      {}\n",
            self.answered_verified
        ));
        out.push_str(&format!(
            "  answered_partial       {}\n",
            self.answered_partial
        ));
        out.push_str(&format!(
            "  clarification_needed   {}\n",
            self.clarification_needed
        ));
        out.push_str(&format!(
            "  failed_timeout         {}\n",
            self.failed_timeout
        ));
        out.push_str(&format!("  failed_parse           {}\n", self.failed_parse));
        out.push_str(&format!(
            "  failed_probes          {}\n",
            self.failed_probes
        ));
        out.push_str(&format!(
            "  aborted_by_user        {}\n",
            self.aborted_by_user
        ));
        out.push_str(&format!(
            "  error_internal         {}\n",
            self.error_internal
        ));
        out.push('\n');

        out.push_str("[latency]\n");
        out.push_str(&format!(
            "  avg_total_ms           {}\n",
            self.avg_total_ms()
        ));
        out.push_str(&format!(
            "  avg_probe_ms           {}\n",
            self.avg_probe_ms()
        ));
        out.push_str(&format!("  avg_llm_ms             {}\n", self.avg_llm_ms()));
        out.push_str(&format!(
            "  p50_total_ms           {}\n",
            self.p50_total_ms()
        ));
        out.push_str(&format!(
            "  p90_total_ms           {}\n",
            self.p90_total_ms()
        ));
        out.push('\n');

        out.push_str("[reliability]\n");
        out.push_str(&format!(
            "  verified_rate          {:.1}%\n",
            self.verified_rate() * 100.0
        ));
        out.push_str(&format!(
            "  useful_rate            {:.1}%\n",
            self.useful_rate() * 100.0
        ));
        out.push_str(&format!(
            "  failure_rate           {:.1}%\n",
            self.failure_rate() * 100.0
        ));
        out.push('\n');

        if !self.by_topic.is_empty() {
            out.push_str("[coverage]\n");
            out.push_str("  top_topics_by_count:\n");
            for (topic, stats) in self.top_topics(5) {
                let rate = if stats.total > 0 {
                    stats.verified as f64 / stats.total as f64 * 100.0
                } else {
                    0.0
                };
                out.push_str(&format!(
                    "    {} ({} total, {:.0}% verified)\n",
                    topic, stats.total, rate
                ));
            }

            let low = self.low_verified_topics(0.5);
            if !low.is_empty() {
                out.push_str("  topics_with_low_verified_rate:\n");
                for (topic, rate) in low {
                    out.push_str(&format!("    {} ({:.0}%)\n", topic, rate * 100.0));
                }
            }
        }

        out
    }

    /// Compact summary for status line.
    pub fn summary_line(&self) -> String {
        format!(
            "{}req | {:.0}% verified | {:.0}% useful | {:.0}% failed | {}ms avg",
            self.total_requests,
            self.verified_rate() * 100.0,
            self.useful_rate() * 100.0,
            self.failure_rate() * 100.0,
            self.avg_total_ms(),
        )
    }
}
