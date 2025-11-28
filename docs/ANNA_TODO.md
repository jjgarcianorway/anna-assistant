# Anna TODO Tracker

All pending tasks with ID, description, and status.

Status: TODO -> IN_PROGRESS -> DONE

---

## v0.73.0 Engineering Reset

### Critical Fixes (Core Pipeline)

| ID | Description | Status | Priority |
|----|-------------|--------|----------|
| T001 | Fix rubber-stamping: Senior parse failures must NOT approve with 70/70/70 | DONE | CRITICAL |
| T002 | Fix rubber-stamping: Reject answers with overall score 0 | DONE | CRITICAL |
| T003 | Fix iteration loop: Must run at least 1 real probe before answering | DONE | CRITICAL |
| T004 | Test: CPU question - "What CPU do I have?" | BLOCKED | CRITICAL |
| T005 | Test: RAM question - "How much RAM do I have?" | BLOCKED | CRITICAL |
| T006 | Test: Disk question - "What disks do I have?" | BLOCKED | CRITICAL |
| T007 | Test: GPU question - "What GPU do I have?" | BLOCKED | CRITICAL |
| T008 | Test: OS question - "What operating system?" | BLOCKED | CRITICAL |
| T009 | Update ANNA_SPEC.md to reflect fixed v0.73.0 | TODO | HIGH |

**T004-T008 BLOCKED**: Daemon restart required. Run:
```bash
sudo cp ~/anna-assistant/target/release/annad /usr/local/bin/
sudo cp ~/anna-assistant/target/release/annactl /usr/local/bin/
sudo systemctl restart annad
```

### Deferred (Keep but UNTRUSTED)

| Feature | Status | Notes |
|---------|--------|-------|
| Progression/XP | KEEP | Numbers may change after fixes |
| Auto-update | KEEP | Untouched |
| Stats display | KEEP | Numbers may change |
| Debug streaming | KEEP | Untouched |
| Fast path | KEEP | May need review after core fix |

---

## Completed

(See ANNA_HISTORY.md)
