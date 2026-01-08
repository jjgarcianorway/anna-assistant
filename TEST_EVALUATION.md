# Anna Test Evaluation

## Test Results (20 questions)

| Metric | Count | Percentage |
|--------|-------|------------|
| Success | 14 | 70% |
| Timeout | 6 | 30% |
| Empty | 0 | 0% |
| Wiki Hit | 20 | 100% |

## Successful Questions
1. How do I install a package with pacman? - **Good answer**
2. How do I update my system? - **Good answer**
3. How do I install NVIDIA drivers? - **Good answer**
5. How do I connect to WiFi from terminal? - **Good answer**
8. How do I mount a USB drive? - **Good answer**
9. How do I change screen resolution? - **Good answer**
10. How do I add a new user? - **Good answer**
11. How do I enable SSH? - **Good answer**
12. How do I fix GRUB not showing? - **Good answer**
13. How do I install GNOME? - **Good answer**
14. How do I check my CPU temperature? - **Good answer**
15. How do I set up a VPN? - **Good answer**
16. How do I clear the pacman cache? - **Good answer**
18. How do I fix no sound issue? - **Good answer**

## Timeout Questions (Need Investigation)
4. How do I check my disk space? - **Wiki found wrong articles**
6. How do I start a systemd service? - Likely similar issue
7. How do I check system logs? - Likely similar issue
17. How do I check my GPU information? - Likely similar issue
19. Why is my system slow? - Likely similar issue
20. How do I change my screen brightness? - Likely similar issue

## Root Causes Identified

### 1. Wiki Search Returns Irrelevant Articles
For "How do I check my disk space?", wiki returned:
- Yandex Disk (irrelevant)
- Disk quota (irrelevant)
- Machine-check exception (irrelevant)

This happens because "disk" matches "Yandex Disk" and "check" matches "Machine-check".

### 2. Simple Questions Don't Need Wiki
Basic commands like `df -h`, `journalctl`, `systemctl` don't need wiki lookup.
The LLM should answer these from general knowledge.

### 3. Irrelevant Wiki Context Confuses LLM
When wiki returns garbage, the LLM gets confused by irrelevant context,
leading to slower responses or timeouts.

## Proposed Improvements

### Short-term Fixes
1. **Skip wiki for low-relevance results** - If top score < threshold, don't include wiki context
2. **Filter common-word articles** - "Yandex Disk" shouldn't match "disk space"
3. **Increase timeout** - Some legitimate queries take > 90s

### Medium-term Improvements
1. **Query classification** - Detect if question needs wiki or just commands
2. **Better article matching** - Consider article content, not just titles
3. **Fallback to basic commands** - For common tasks, suggest standard commands

### Long-term Vision
1. **Semantic search with embeddings** - Actually use the embedding index
2. **Query decomposition** - Extract topic + intent before searching
3. **Multi-step reasoning** - Gather info → analyze → answer

## Quality Metrics

| Category | Good | Issues |
|----------|------|--------|
| Answer correctness | 70% | Timeouts |
| Wiki relevance | 50% | Wrong articles for generic queries |
| Response time | Variable | Some > 90s |
| Command quality | Good | Extracts from wiki properly |
