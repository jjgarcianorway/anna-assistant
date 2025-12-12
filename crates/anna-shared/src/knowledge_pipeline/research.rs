//! Research pattern for specialists (v0.0.432).
//!
//! When a specialist is unsure, they should research first before answering.
//! This module provides the research workflow.

use super::fetcher::{FetchConfig, FetchResult, KnowledgeFetcher};
use super::sources::{Citation, KnowledgeSource, SourceResult};
use serde::{Deserialize, Serialize};

/// Research request from a specialist.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchRequest {
    /// The question to research.
    pub question: String,
    /// Topic area (helps narrow sources).
    pub topic: Option<String>,
    /// Specific sources to try.
    pub sources: Vec<KnowledgeSource>,
    /// Maximum time to spend researching (ms).
    pub timeout_ms: Option<u64>,
    /// Minimum confidence required.
    pub min_confidence: f32,
}

impl ResearchRequest {
    /// Create a new research request.
    pub fn new(question: &str) -> Self {
        Self {
            question: question.to_string(),
            topic: None,
            sources: Vec::new(),
            timeout_ms: None,
            min_confidence: 0.7,
        }
    }

    /// Set the topic area.
    pub fn with_topic(mut self, topic: &str) -> Self {
        self.topic = Some(topic.to_string());
        self
    }

    /// Add a source to try.
    pub fn with_source(mut self, source: KnowledgeSource) -> Self {
        self.sources.push(source);
        self
    }

    /// Add multiple sources.
    pub fn with_sources(mut self, sources: Vec<KnowledgeSource>) -> Self {
        self.sources.extend(sources);
        self
    }

    /// Set timeout.
    pub fn with_timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = Some(ms);
        self
    }

    /// Set minimum confidence.
    pub fn with_min_confidence(mut self, confidence: f32) -> Self {
        self.min_confidence = confidence;
        self
    }
}

/// Outcome of research.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResearchOutcome {
    /// Found a confident answer with citations.
    Found {
        /// The synthesized answer.
        answer: String,
        /// Supporting citations.
        citations: Vec<Citation>,
        /// Overall confidence.
        confidence: f32,
    },
    /// Found partial information, needs more research.
    Partial {
        /// What was found.
        findings: String,
        /// What's still missing.
        gaps: Vec<String>,
        /// Sources consulted.
        sources_tried: Vec<String>,
    },
    /// Could not find relevant information.
    NotFound {
        /// Why the search failed.
        reason: String,
        /// Sources that were tried.
        sources_tried: Vec<String>,
    },
    /// Need clarification from user.
    NeedsClarification {
        /// What needs to be clarified.
        question: String,
        /// Options if applicable.
        options: Vec<String>,
    },
}

impl ResearchOutcome {
    /// Check if research was successful.
    pub fn is_found(&self) -> bool {
        matches!(self, Self::Found { .. })
    }

    /// Get confidence if found.
    pub fn confidence(&self) -> Option<f32> {
        match self {
            Self::Found { confidence, .. } => Some(*confidence),
            _ => None,
        }
    }

    /// Get citations if available.
    pub fn citations(&self) -> Vec<&Citation> {
        match self {
            Self::Found { citations, .. } => citations.iter().collect(),
            _ => Vec::new(),
        }
    }
}

/// Research pattern implementation.
pub struct ResearchPattern {
    fetcher: KnowledgeFetcher,
}

impl ResearchPattern {
    /// Create a new research pattern.
    pub fn new() -> Self {
        Self {
            fetcher: KnowledgeFetcher::new(),
        }
    }

    /// Create with custom fetcher config.
    pub fn with_config(config: FetchConfig) -> Self {
        Self {
            fetcher: KnowledgeFetcher::with_config(config),
        }
    }

    /// Execute research.
    pub fn research(&self, request: &ResearchRequest) -> ResearchOutcome {
        // Build source list
        let sources = if request.sources.is_empty() {
            // Auto-suggest sources based on topic/question
            let topic = request.topic.as_deref().unwrap_or(&request.question);
            self.fetcher.suggest_sources(topic)
        } else {
            request.sources.clone()
        };

        if sources.is_empty() {
            return ResearchOutcome::NotFound {
                reason: "No suitable sources identified".to_string(),
                sources_tried: Vec::new(),
            };
        }

        // Fetch from sources
        let result = self.fetcher.fetch(&request.question, &sources);

        // Analyze results
        self.analyze_results(request, result)
    }

    /// Analyze fetch results and determine outcome.
    fn analyze_results(&self, request: &ResearchRequest, result: FetchResult) -> ResearchOutcome {
        if result.results.is_empty() {
            return ResearchOutcome::NotFound {
                reason: "No results from any source".to_string(),
                sources_tried: result.failed_sources,
            };
        }

        // Check if we have confident results
        let best = result.results.first().unwrap();
        let confidence = best.trust_score();

        if confidence >= request.min_confidence {
            // Synthesize answer from results
            let answer = self.synthesize_answer(&result.results);
            ResearchOutcome::Found {
                answer,
                citations: result.citations,
                confidence,
            }
        } else if !result.results.is_empty() {
            // Partial results
            let findings = self.summarize_findings(&result.results);
            let gaps = self.identify_gaps(request, &result.results);
            let sources_tried: Vec<String> = result
                .results
                .iter()
                .map(|r| r.source.description())
                .collect();

            ResearchOutcome::Partial {
                findings,
                gaps,
                sources_tried,
            }
        } else {
            ResearchOutcome::NotFound {
                reason: "Results below confidence threshold".to_string(),
                sources_tried: result.failed_sources,
            }
        }
    }

    /// Synthesize an answer from multiple results.
    fn synthesize_answer(&self, results: &[SourceResult]) -> String {
        // For now, use the best result's content
        // In production, this would use an LLM to synthesize
        if let Some(best) = results.first() {
            // Truncate if too long
            let content = &best.content;
            if content.len() > 2000 {
                format!("{}...\n\n[Truncated - full content available from {}]",
                    &content[..2000],
                    best.source.description())
            } else {
                content.clone()
            }
        } else {
            "No information found.".to_string()
        }
    }

    /// Summarize partial findings.
    fn summarize_findings(&self, results: &[SourceResult]) -> String {
        let summaries: Vec<String> = results
            .iter()
            .take(3)
            .map(|r| format!("- {}: relevance {:.0}%", r.source.description(), r.relevance * 100.0))
            .collect();

        format!("Found information from:\n{}", summaries.join("\n"))
    }

    /// Identify gaps in the research.
    fn identify_gaps(&self, request: &ResearchRequest, results: &[SourceResult]) -> Vec<String> {
        let mut gaps = Vec::new();

        // Check confidence
        if let Some(best) = results.first() {
            if best.trust_score() < request.min_confidence {
                gaps.push(format!(
                    "Confidence {:.0}% is below required {:.0}%",
                    best.trust_score() * 100.0,
                    request.min_confidence * 100.0
                ));
            }
        }

        // Check relevance
        let low_relevance: Vec<_> = results
            .iter()
            .filter(|r| r.relevance < 0.5)
            .collect();
        if !low_relevance.is_empty() {
            gaps.push("Some sources had low relevance to the question".to_string());
        }

        gaps
    }

    /// Quick check if a topic can be answered from local sources.
    pub fn can_answer_locally(&self, topic: &str) -> bool {
        let sources = self.fetcher.suggest_sources(topic);
        sources.iter().any(|s| {
            matches!(
                s.priority(),
                super::sources::SourcePriority::Probe | super::sources::SourcePriority::LocalDoc
            )
        })
    }
}

impl Default for ResearchPattern {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_research_request_builder() {
        let req = ResearchRequest::new("how much ram?")
            .with_topic("memory")
            .with_min_confidence(0.8)
            .with_source(KnowledgeSource::probe("meminfo"));

        assert_eq!(req.question, "how much ram?");
        assert_eq!(req.topic, Some("memory".to_string()));
        assert_eq!(req.min_confidence, 0.8);
        assert_eq!(req.sources.len(), 1);
    }

    #[test]
    fn test_outcome_checks() {
        let found = ResearchOutcome::Found {
            answer: "test".to_string(),
            citations: vec![],
            confidence: 0.9,
        };
        assert!(found.is_found());
        assert_eq!(found.confidence(), Some(0.9));

        let not_found = ResearchOutcome::NotFound {
            reason: "test".to_string(),
            sources_tried: vec![],
        };
        assert!(!not_found.is_found());
        assert_eq!(not_found.confidence(), None);
    }

    #[test]
    fn test_can_answer_locally() {
        let pattern = ResearchPattern::new();

        // Memory questions should be answerable locally
        assert!(pattern.can_answer_locally("memory usage"));
        assert!(pattern.can_answer_locally("cpu info"));
    }
}
