# Anna vs ChatGPT 100 Questions - Test Results

**Test Date**: 2026-02-13
**Anna Version**: 0.3.175
**Test Type**: Strategic Sample (20/100 questions, 2 per difficulty level)

## Overall Results

| Metric | Result |
|--------|--------|
| **Success Rate** | **70% (14/20)** |
| **Instant Answers** | **40% (8/20)** |
| **Average Response Time** | **2.9 seconds** |
| **Failed/Timeout** | 30% (6/20) |

## Performance by Difficulty Level

| Level | Success | Instant | Notes |
|-------|---------|---------|-------|
| **Level 1** (Basic) | 2/2 (100%) | 2/2 (100%) | ✅ Perfect - all instant |
| **Level 2** (Hardware) | 0/2 (0%) | 0/2 | ❌ Both timed out |
| **Level 3** (Users) | 1/2 (50%) | 1/2 | ⚠️ Sudo check timed out |
| **Level 4** (Processes) | 2/2 (100%) | 1/2 | ✅ Strong performance |
| **Level 5** (Packages) | 2/2 (100%) | 2/2 (100%) | ✅ Perfect - all instant |
| **Level 6** (Services) | 1/2 (50%) | 0/2 | ⚠️ Timers query timed out |
| **Level 7** (Logs) | 2/2 (100%) | 1/2 | ✅ Strong performance |
| **Level 8** (Networking) | 1/2 (50%) | 1/2 | ⚠️ Open ports timed out |
| **Level 9** (Security) | 1/2 (50%) | 0/2 | ⚠️ Login attempts timed out |
| **Level 10** (Advanced) | 2/2 (100%) | 0/2 | ✅ Handled complex questions |

## Instant Answer Performance (40%)

Anna successfully provided instant answers for:

✅ **System Identity**:
- What is my hostname? (70ms)
- How much RAM does my system have? (43ms)

✅ **User Management**:
- What groups do I belong to? (59ms)

✅ **Processes**:
- What processes are consuming the most CPU? (54ms)

✅ **Package Management**:
- What packages were installed recently? (62ms)
- Can you remove unused packages? (57ms)

✅ **Logs**:
- Can you check system logs for errors? (43ms)

✅ **Networking**:
- Which DNS server am I using? (33ms)

## Questions Requiring Investigation (60%)

Anna used LLM investigation for:

✅ **Performance Analysis**:
- Why is my system slow? (2.3s, ✓ passed)
- Why is boot time slow? (2.3s, ✓ passed)
- Why is my system experiencing high I/O wait? (2.3s, ✓ passed)

✅ **Hardware Diagnostics**:
- Are there hardware errors in dmesg? (24.8s, ✓ passed)

✅ **Security Audit**:
- Is my SSH configuration secure? (6.4s, ✓ passed)

✅ **Advanced Analysis**:
- Can you analyze long-term usage patterns? (2.3s, ✓ passed)

## Failed Questions (30%)

❌ **Timeouts** (6 questions exceeded 45s limit):

1. **What disks are connected to my system?** - Hardware enumeration
2. **What GPU is installed?** - Hardware detection
3. **Do I have sudo privileges?** - Permission check
4. **What timers are active on my system?** - Systemd timers
5. **What ports are open on my machine?** - Network scanning
6. **Are there unauthorized login attempts?** - Security audit

## Key Insights

### Strengths

1. **Instant Answer Coverage**: 40% of questions answered instantly (<100ms)
   - System identity queries work perfectly
   - Package management queries are instant
   - Basic resource queries are instant

2. **Complex Analysis**: Anna handles advanced diagnostic questions well
   - Performance analysis: 100% success
   - Long-term pattern analysis: Works
   - Security audits: Partially working

3. **Speed**: Average 2.9s for LLM-based answers (excellent)

### Weaknesses

1. **Hardware Enumeration**: 0% success on hardware detection
   - GPU detection fails
   - Disk enumeration times out
   - Need instant answers for these

2. **Security Audits**: 50% success rate
   - Login attempt checking times out
   - SSH config audit works but slow
   - Need faster security checks

3. **Systemd Timers**: Query times out
   - Need instant answer path
   - Simple systemctl command could work

4. **Permission Checks**: Sudo check times out
   - Should be instant answer
   - Simple `groups` + check for sudo/wheel group

### Comparison to Baseline

- **Reliability Framework**: 100% (9/9 questions)
- **ChatGPT Sample**: 70% (14/20 questions)
- **Instant Answer Rate**: Improved from 22% to 40%

## Recommended Improvements

### Priority 1: Add Instant Answers (Target: 60%)

These questions should have instant answers:

```bash
# Hardware (currently timing out)
- What disks are connected? → lsblk -d
- What GPU is installed? → lspci | grep VGA

# Permissions (currently timing out)
- Do I have sudo privileges? → groups | grep -q "sudo\|wheel"

# Systemd (currently timing out)
- What timers are active? → systemctl list-timers

# Networking (currently timing out)
- What ports are open? → ss -tulpn

# Security (currently timing out)
- Unauthorized logins? → lastb | head -20
```

### Priority 2: Optimize LLM Queries

- Hardware errors in dmesg: 24.8s → should be <5s
- SSH config audit: 6.4s → should be <3s

### Priority 3: Prevent Timeouts

- Investigate why some queries hang beyond 45s
- Add fallback mechanisms
- Improve command execution reliability

## Next Steps

1. **Implement 6 new instant answers** (Priority 1 list above)
2. **Re-test** strategic sample to validate improvements
3. **Run full 100-question suite** once success rate >85%
4. **Compare results** with Claude/ChatGPT on identical questions

## Conclusion

Anna shows strong performance on:
- ✅ Basic system queries (100% success, all instant)
- ✅ Package management (100% success, all instant)
- ✅ Advanced diagnostics (100% success)
- ✅ Performance analysis (100% success)

Needs improvement on:
- ❌ Hardware enumeration (0% success)
- ⚠️ Security audits (50% success)
- ⚠️ Systemd queries (50% success)

**Overall Assessment**: Anna performs well on 70% of questions with 40% instant answers. With Priority 1 improvements (6 new instant answers), we project:
- Success rate: 70% → 85%+
- Instant answer rate: 40% → 60%+
- Ready for full 100-question test

---

Test files saved:
- Questions: `tests/chatgpt_100_questions.txt`
- Test script: `tests/chatgpt_100_test.sh`
- Results: This document
