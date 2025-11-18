# Reddit QA Validation - Beta.78

## Overview

Real-world validation system using **actual questions from r/archlinux**.

Instead of synthetic tests, we validate Anna against the community's collective wisdom by:
1. Fetching 500-1000 real user questions
2. Running them through Anna's LLM
3. Comparing responses to top-voted community answers
4. Measuring helpfulness and accuracy

## Why This Matters

**Synthetic tests** validate technical correctness.
**Reddit validation** validates **real-world helpfulness**.

If Anna can match or exceed community answers, she's truly useful.

## Quick Start

### 1. Fetch Questions from Reddit

```bash
./scripts/fetch_reddit_qa.sh reddit_questions.json 1000
```

This fetches 1000 questions from r/archlinux using the public JSON API.

### 2. Run Validation

```rust
use anna_common::reddit_qa_validator::*;

let client = RedditClient::new();
let questions = client.load_from_file("reddit_questions.json")?;

// Run Anna on each question
for question in &questions[0..100] {
    let response = run_anna_query(&question.title, &question.body).await?;
    // Compare with top community answer...
}
```

### 3. Generate Report

The validator generates a report showing:
- **Pass rate:** % of questions Anna answered helpfully
- **Community match:** % where Anna matched consensus
- **Best matches:** Examples where Anna ≈ Community
- **Areas for improvement:** Where Anna diverged

## Metrics

### Helpfulness (1-5)
- Does Anna's answer actually help the user?
- Is it actionable?
- Does it provide clear next steps?

### Accuracy (1-5)
- Is the information correct?
- Does it follow Arch best practices?
- Are commands safe?

### Completeness (1-5)
- Does it address all aspects of the question?
- Are edge cases handled?
- Is context provided?

### Community Match (0.0-1.0)
- Similarity to most-voted answer
- Uses semantic comparison (not just string matching)

## Data Format

### Input (reddit_questions.json)

```json
[
  {
    "id": "abc123",
    "title": "Bluetooth headphones won't connect",
    "body": "I'm using bluez but my Sony headphones won't pair...",
    "score": 45,
    "num_comments": 12,
    "url": "https://reddit.com/r/archlinux/comments/abc123"
  }
]
```

### Output (validation_report.json)

```json
{
  "total_questions": 1000,
  "helpful_count": 850,
  "matched_community": 720,
  "avg_similarity": 0.75,
  "pass_rate": 0.85,
  "results": [...]
}
```

## Usage Examples

### Fetch Recent Questions

```bash
# Get questions from this month
./scripts/fetch_reddit_qa.sh questions_month.json 500

# Get questions from this week
./scripts/fetch_reddit_qa.sh questions_week.json 100
```

### Filter by Topic

```bash
# Only GPU/graphics questions
jq '[.[] | select(.title | test("gpu|graphics|nvidia|amd"; "i"))]' questions.json > gpu_questions.json
```

### Validate Specific Areas

```bash
# Test Anna on Bluetooth issues
jq '[.[] | select(.title | test("bluetooth"; "i"))]' questions.json > bluetooth.json
cargo test --test reddit_qa -- bluetooth
```

## Benefits

1. **Real Problems:** Tests actual user pain points
2. **Community Wisdom:** Compares against collective knowledge
3. **Continuous Validation:** Re-run monthly to track improvement
4. **Identify Gaps:** Find topics where Anna needs work
5. **Benchmark Progress:** Measure Anna's evolution over time

## Next Steps

- **Beta.79:** Implement semantic similarity scoring
- **Beta.80:** Add manual validation UI
- **Beta.81:** Automated monthly validation runs
- **Beta.82:** Public validation dashboard

## Architecture

```
┌─────────────┐
│ r/archlinux │
└──────┬──────┘
       │ fetch_reddit_qa.sh
       ▼
┌──────────────────┐
│ reddit_questions │
│     .json        │
└──────┬───────────┘
       │ RedditClient::load_from_file()
       ▼
┌──────────────────┐
│ ValidationRunner │
└──────┬───────────┘
       │ For each question:
       │   1. Query Anna
       │   2. Compare to top answer
       │   3. Calculate similarity
       ▼
┌──────────────────┐
│ ValidationSuite  │
│   - pass_rate    │
│   - avg_sim      │
│   - results[]    │
└──────┬───────────┘
       │ generate_report()
       ▼
┌──────────────────┐
│  Markdown Report │
│  + JSON Export   │
└──────────────────┘
```

## Example Report

```
# Reddit QA Validation Report

**Total Questions:** 1000
**Helpful Answers:** 850 (85.0%)
**Community Match:** 720 (72.0%)
**Avg Similarity:** 0.75
**Avg Response Time:** 1500ms
**Pass Rate:** 85.0%

## ✅ Best Matches (Anna ≈ Community)

**Q:** How do I enable tap-to-click on my touchpad?
**Similarity:** 92%
**Anna:** Edit `/etc/X11/xorg.conf.d/40-libinput.conf` and add
`Option "Tapping" "on"` to the touchpad section. Restart X11.

---

**Q:** What's the difference between pacman -S and yay -S?
**Similarity:** 88%
**Anna:** `pacman -S` installs from official repos only. `yay -S`
also checks AUR. For system packages, prefer pacman. Use yay
for AUR-only packages like google-chrome or spotify.
```

## Validation Workflow

```bash
# 1. Fetch questions
./scripts/fetch_reddit_qa.sh data/reddit_questions.json 1000

# 2. Review sample
jq '.[0:5]' data/reddit_questions.json

# 3. Run validation
cargo test --test reddit_qa_integration -- --nocapture

# 4. Generate report
cat validation_report.md

# 5. Identify improvements
jq '.results | sort_by(.similarity_score) | .[0:10]' validation_results.json
```

## Rate Limiting

Reddit API rate limits:
- **Without auth:** 60 requests/minute
- **With OAuth:** 600 requests/10 minutes

For 1000 questions:
- No auth: ~17 minutes
- With OAuth: ~2 minutes

The fetch script includes automatic rate limiting (2s delay between requests).
