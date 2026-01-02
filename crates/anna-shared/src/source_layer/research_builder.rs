//! Research Citation Builder - v0.0.443.
//!
//! Build citations from fetched sources and evidence.

use super::providers::SourceContent;
use super::research::truncate;
use super::research_types::Citation;

/// Build citations from fetched sources.
pub struct CitationBuilder {
    /// Source citations.
    sources: Vec<Citation>,
    /// Evidence citations.
    evidence: Vec<Citation>,
    /// Source counter.
    source_counter: usize,
    /// Evidence counter.
    evidence_counter: usize,
}

impl CitationBuilder {
    /// Create new builder.
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
            evidence: Vec::new(),
            source_counter: 0,
            evidence_counter: 0,
        }
    }

    /// Add source from fetched content.
    pub fn add_source(&mut self, content: &SourceContent) -> Option<String> {
        if !content.success {
            return None;
        }

        self.source_counter += 1;
        let id = format!("S{}", self.source_counter);

        let citation = Citation::source(
            &id,
            content.request.source_type,
            &content.request.id,
            &content.request.query,
        );

        // Add excerpt if available
        let citation = if let Some(ref excerpt) = content.excerpt {
            citation.with_excerpt(excerpt)
        } else if let Some(ref full_content) = content.content {
            citation.with_excerpt(&truncate(full_content, 200))
        } else {
            citation
        };

        self.sources.push(citation);
        Some(id)
    }

    /// Add evidence from probe.
    pub fn add_evidence(&mut self, probe: &str, output: &str) -> String {
        self.evidence_counter += 1;
        let id = format!("E{}", self.evidence_counter);

        self.evidence.push(Citation::evidence(&id, probe, output));
        id
    }

    /// Build citations.
    pub fn build(self) -> (Vec<Citation>, Vec<Citation>) {
        (self.sources, self.evidence)
    }
}

impl Default for CitationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_layer::providers::SourceRequest;

    #[test]
    fn test_citation_builder() {
        let mut builder = CitationBuilder::new();

        let content = SourceContent::success(
            SourceRequest::man("pacman(8)", "DESC"),
            "Full content",
            Some("Excerpt"),
        );
        let id = builder.add_source(&content);
        assert_eq!(id, Some("S1".to_string()));

        let eid = builder.add_evidence("pacman -Qu", "10 packages");
        assert_eq!(eid, "E1");

        let (sources, evidence) = builder.build();
        assert_eq!(sources.len(), 1);
        assert_eq!(evidence.len(), 1);
    }
}
