# Anna QA Test Suite - Arch Linux Questions

## Purpose

This test suite measures Anna's ability to answer typical Arch Linux system administration questions. The goal is to ensure Anna provides answers that are **as good as or better than** a careful Arch expert.

## Structure

```
tests/qa/
├── questions_archlinux.jsonl     # All test questions (currently 20, target 700)
├── golden/                        # Reference answers from Arch experts
│   ├── arch-001_golden.json
│   ├── arch-002_golden.json
│   └── ...
├── results/                       # Test run outputs
│   ├── arch-001_anna.txt         # Anna's raw output
│   ├── arch-002_anna.txt
│   ├── summary.json              # Machine-readable results
│   └── summary.md                # Human-readable summary
├── run_qa_suite.sh               # Main test harness
├── EVALUATION_RULES.md           # Criteria for PASS/PARTIAL/FAIL
├── HUMAN_REVIEW_SAMPLE.md        # Manual verification of automated verdicts
└── README.md                     # This file
```

## Question Format

Each line in `questions_archlinux.jsonl` is a JSON object:

```json
{
  "id": "arch-001",
  "category": "networking",
  "question": "How do I configure a static IP address on Arch Linux using systemd-networkd?"
}
```

### Categories

- `networking` - Network configuration, firewall, WiFi
- `package_management` - pacman, AUR, package operations
- `system_services` - systemd services, daemon management
- `boot` - GRUB, kernel, initramfs
- `desktop` - X11, Wayland, GPU drivers
- `troubleshooting` - Debugging system issues
- `security` - Users, permissions, encryption
- `storage` - Filesystems, mounting, LVM
- `performance` - Optimization, monitoring

## Golden Answer Format

Each golden answer is a JSON file in `golden/` directory:

```json
{
  "id": "arch-001",
  "question": "How do I configure a static IP address on Arch Linux using systemd-networkd?",
  "golden_answer": {
    "summary": "Configure static IP in /etc/systemd/network/*.network file and restart systemd-networkd",
    "steps": [
      "Create or edit a .network file in /etc/systemd/network/ (e.g., 20-wired.network)",
      "Add [Match] section with Name=<interface>",
      "Add [Network] section with Address=<ip>/<prefix> and Gateway=<gateway>",
      "Add DNS=<dns> in [Network] section if needed",
      "Restart systemd-networkd: sudo systemctl restart systemd-networkd",
      "Verify with: ip addr show <interface>"
    ],
    "required_commands": [
      "systemctl restart systemd-networkd",
      "systemctl enable systemd-networkd"
    ],
    "required_files": [
      "/etc/systemd/network/*.network"
    ],
    "validation": [
      "ip addr show <interface>",
      "ping <gateway>"
    ],
    "warnings": [
      "Back up existing network configuration before changes",
      "Ensure systemd-networkd is enabled and not conflicting with NetworkManager"
    ],
    "references": [
      "Arch Wiki: systemd-networkd"
    ]
  }
}
```

## Running the Test Suite

### Full suite (all questions)
```bash
cd tests/qa
./run_qa_suite.sh --all
```

### Single question
```bash
./run_qa_suite.sh --id arch-001
```

### Specific category
```bash
./run_qa_suite.sh --category networking
```

### Regenerate results
```bash
./run_qa_suite.sh --all --clean
```

## Results Format

### summary.json
Machine-readable JSON with per-question results:

```json
{
  "run_timestamp": "2025-11-20T12:34:56Z",
  "anna_version": "5.7.0-beta.149",
  "total_questions": 20,
  "pass": 15,
  "partial": 3,
  "fail": 2,
  "results": [
    {
      "id": "arch-001",
      "question": "...",
      "verdict": "PASS",
      "score": 0.95,
      "issues": [],
      "anna_output_file": "results/arch-001_anna.txt"
    }
  ]
}
```

### summary.md
Human-readable report with statistics and examples.

## Adding New Questions

1. Add to `questions_archlinux.jsonl`:
```bash
echo '{"id":"arch-NNN","category":"X","question":"..."}' >> questions_archlinux.jsonl
```

2. Create golden answer:
```bash
$EDITOR golden/arch-NNN_golden.json
```

3. Run the test:
```bash
./run_qa_suite.sh --id arch-NNN
```

## Evaluation Criteria

See `EVALUATION_RULES.md` for detailed PASS/PARTIAL/FAIL criteria.

**Summary:**
- **PASS**: All critical steps included, correct commands, proper warnings
- **PARTIAL**: Most steps correct but missing minor details or too generic
- **FAIL**: Wrong commands, dangerous operations without warnings, or no useful guidance

## Current Status

- **Questions**: 20 / 700 (initial batch)
- **Golden answers**: 0 / 20 (in progress)
- **Test harness**: In development
- **Evaluation rules**: In progress

## Design Principles

1. **Honesty**: Never claim "all tests pass" without running them
2. **Determinism**: Same question → same verdict (given same Anna version)
3. **Traceability**: Every verdict backed by explicit rules and evidence
4. **Reproducibility**: One command to rerun entire suite
5. **No optimistic language**: Report failures clearly, not as "opportunities"

## Next Steps

1. Write 20 golden answers for initial batch ✅ (Step 2)
2. Implement test harness (run_qa_suite.sh) ✅ (Step 3)
3. Define evaluation rules (EVALUATION_RULES.md) ✅ (Step 4)
4. Run initial batch and generate results ✅ (Step 5)
5. Create human review sample (HUMAN_REVIEW_SAMPLE.md) ✅ (Step 5)
6. Extend to full 700 questions (iterative)
