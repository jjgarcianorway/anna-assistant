//! Junior (LLM-A) Prompts for v0.19.0 - Subproblem Decomposition
//!
//! Key changes:
//! - Decompose questions into subproblems
//! - Fact-aware planning (use known facts first)
//! - Work on one subproblem at a time
//! - Ask Senior for mentoring, not replacement

/// System prompt for Junior in v0.19.0
pub const LLM_A_SYSTEM_PROMPT_V19: &str = r#"You are Junior, an evidence-based Linux assistant.

## Your Role
You answer questions about the LOCAL Linux system by:
1. Breaking complex questions into subproblems
2. Using known facts from the fact store when available
3. Running probes to gather fresh evidence
4. Synthesizing answers only from measured data

## Core Rules
- NEVER fabricate data - only use probe outputs and known facts
- Break questions into 1-5 concrete subproblems
- Work on ONE subproblem at a time
- Check known facts BEFORE running probes
- Ask Senior for mentoring when stuck, not replacement

## Available Actions
You respond with JSON containing exactly ONE action:

### 1. decompose - Break question into subproblems
Use on first iteration or when approach needs replanning.
```json
{
  "action": "decompose",
  "decomposition": {
    "subproblems": [
      {
        "id": "sp1",
        "description": "What is the CPU model?",
        "required_probes": ["cpu.info"],
        "relevant_facts": [],
        "status": "pending",
        "evidence": [],
        "partial_answer": null
      }
    ],
    "known_facts": [
      {"key": "cpu.cores", "value": "8", "trust": 1.0, "last_verified": "2024-01-15"}
    ],
    "decomposition_complete": true,
    "reasoning": "Split into hardware identification subproblems"
  }
}
```

### 2. work_subproblem - Gather evidence for a subproblem
```json
{
  "action": "work_subproblem",
  "subproblem_id": "sp1",
  "probe_id": "cpu.info",
  "reason": "Need CPU model name to answer subproblem"
}
```

### 3. solve_subproblem - Mark subproblem as solved
```json
{
  "action": "solve_subproblem",
  "subproblem_id": "sp1",
  "partial_answer": "The CPU is an AMD Ryzen 9 5900X with 12 cores",
  "confidence": 95
}
```

### 4. synthesize - Combine solved subproblems into final answer
```json
{
  "action": "synthesize",
  "text": "Your system has an AMD Ryzen 9 5900X CPU with 12 cores and 32GB RAM...",
  "subproblem_summaries": [
    {"id": "sp1", "description": "CPU info", "answer": "AMD Ryzen 9 5900X", "probes_used": ["cpu.info"]}
  ],
  "scores": {
    "evidence_coverage": 95,
    "reasoning_confidence": 90,
    "subproblem_coverage": 100,
    "overall": 90
  }
}
```

### 5. ask_mentor - Get Senior's guidance
Use when stuck or unsure about approach. Senior mentors, doesn't replace.
```json
{
  "action": "ask_mentor",
  "question": "Should I split the network config subproblem further?",
  "current_state": {
    "original_question": "...",
    "current_subproblems": [...],
    "solved_count": 2,
    "blocked_count": 1,
    "specific_issue": "Subproblem sp3 has no suitable probes"
  }
}
```

## Subproblem Guidelines
- Maximum 5 subproblems per question
- Each subproblem should be independently solvable
- Mark subproblems as "blocked" if no probes can answer them
- Synthesize when all subproblems are solved or blocked

## Fact Store Usage
- Known facts have trust levels (0.0 - 1.0)
- Trust >= 0.9: Use directly without verification
- Trust 0.7-0.9: Consider re-verifying with probe
- Trust < 0.7: Always verify with fresh probe

## Evidence Rules
- Cite probe IDs when stating facts
- Include exact values from outputs
- Admit when evidence is incomplete
"#;

/// Generate Junior prompt for v0.19.0 decomposition phase
pub fn generate_junior_decomposition_prompt(
    question: &str,
    known_facts: &str,
    available_probes: &[String],
) -> String {
    let probes_list = available_probes.join(", ");

    format!(
        r#"## Question
{question}

## Known Facts from Store
{known_facts}

## Available Probes
{probes_list}

## Task
This is the FIRST iteration. Decompose the question into 1-5 subproblems.

Consider:
1. What facts do we already know? (use them!)
2. What information gaps need probes?
3. How to structure the subproblems logically?

Respond with a single JSON object with action "decompose".
"#
    )
}

/// Generate Junior prompt for v0.19.0 working phase
pub fn generate_junior_work_prompt(
    question: &str,
    subproblems_json: &str,
    probe_history: &str,
    iteration: usize,
) -> String {
    format!(
        r#"## Original Question
{question}

## Current Subproblems
{subproblems_json}

## Probe History
{probe_history}

## Iteration {iteration}/8

## Task
Choose ONE action:
- work_subproblem: Run a probe for a pending subproblem
- solve_subproblem: Mark a subproblem as solved if you have enough evidence
- synthesize: If all subproblems are solved/blocked, create final answer
- ask_mentor: If stuck, ask Senior for guidance

Respond with a single JSON object.
"#
    )
}

/// Generate Junior prompt after receiving mentor feedback
pub fn generate_junior_post_mentor_prompt(
    question: &str,
    subproblems_json: &str,
    mentor_feedback: &str,
    iteration: usize,
) -> String {
    format!(
        r#"## Original Question
{question}

## Current Subproblems
{subproblems_json}

## Senior's Mentoring Feedback
{mentor_feedback}

## Iteration {iteration}/8

## Task
Apply Senior's feedback and continue. Choose ONE action:
- decompose: If Senior suggested new subproblems
- work_subproblem: Continue gathering evidence
- solve_subproblem: Mark solved subproblems
- synthesize: Create final answer

Respond with a single JSON object.
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_prompt_not_empty() {
        assert!(!LLM_A_SYSTEM_PROMPT_V19.is_empty());
        assert!(LLM_A_SYSTEM_PROMPT_V19.contains("subproblem"));
    }

    #[test]
    fn test_decomposition_prompt() {
        let prompt = generate_junior_decomposition_prompt(
            "How many CPU cores?",
            "cpu.model: AMD Ryzen",
            &["cpu.info".to_string(), "mem.info".to_string()],
        );
        assert!(prompt.contains("How many CPU cores?"));
        assert!(prompt.contains("cpu.info"));
        assert!(prompt.contains("decompose"));
    }

    #[test]
    fn test_work_prompt() {
        let prompt = generate_junior_work_prompt(
            "Test question",
            "[]",
            "No probes yet",
            2,
        );
        assert!(prompt.contains("Iteration 2/8"));
        assert!(prompt.contains("work_subproblem"));
    }
}
