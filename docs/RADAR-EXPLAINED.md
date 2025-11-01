# Anna Radars Explained

## Current Implementation (v0.12.3-v0.12.4)

The current radars are **system health metrics** - they tell you about technical performance, not about YOU as a user.

### Health Radar

**What it measures**: Hardware resource health

| Category | What It Means | Good Score | Bad Score |
|----------|--------------|------------|-----------|
| **CPU Load** | How busy your CPU cores are | 10/10 = idle/light use | 0/10 = maxed out |
| **Memory Pressure** | How much RAM is available | 10/10 = plenty free | 0/10 = almost full |
| **Disk Headroom** | Free space on your root partition (/) | 10/10 = 30%+ free | 0/10 = 5% or less |
| **Thermal** | CPU temperature | 10/10 = cool (≤70°C) | 0/10 = hot (≥90°C) |

**Example Output**:
```
│  ✓ CPU Load: 10/10           # Your CPU is mostly idle
│  ✓ Memory Pressure: 10/10     # Plenty of RAM available
│  ✓ Disk Headroom: 10/10       # Lots of disk space
│  ✓ Thermal: 10/10             # System is running cool
```

### Network Radar

**What it measures**: Network connectivity quality

| Category | What It Means | How It's Measured |
|----------|--------------|-------------------|
| **DNS Reliability** | Can you reach the internet? | Pings 8.8.8.8 (Google DNS) |
| **Latency** | How fast is your connection? | Ping response time in ms |
| **Packet Loss** | Are packets being dropped? | Percentage of failed pings |

**Better Name**: "Network Connectivity" instead of "DNS Reliability"

---

## The Problem

These radars are too **technical** and don't help users understand:
- Who they are as a user
- What kind of computer they have
- How they actually use their system

---

## Future Vision (v0.13.0+)

### 1. Hardware Radar

**Purpose**: Detect what physical hardware you have

**Categories**:
- GPU: "NVIDIA RTX 3080 8GB"
- CPU: "AMD Ryzen 9 5900X (16 cores)"
- RAM: "32GB DDR4"
- Storage: "1TB NVMe SSD + 2TB HDD"
- Display: "2560x1440 @ 144Hz"
- Audio: "Realtek ALC1220"
- Input: "Logitech G Pro Wireless"

**Output Example**:
```
╭─ Hardware Profile ────────────────────────
│
│  GPU:      NVIDIA RTX 3080 (8GB)
│  CPU:      AMD Ryzen 9 5900X (16 cores)
│  RAM:      32GB DDR4 @ 3600MHz
│  Storage:  1TB NVMe + 2TB HDD
│  Display:  2560x1440 @ 144Hz
│  Audio:    Realtek ALC1220
│
│  Profile:  High-End Gaming/Workstation
│
╰───────────────────────────────────────────
```

### 2. System Radar

**Purpose**: Understand your software ecosystem

**Categories**:
- **Distro**: "Arch Linux (rolling)"
- **Packages**: "847 packages installed"
- **Software Categories**:
  - Text Editors: 3 (nvim, code, vim)
  - Browsers: 2 (firefox, chromium)
  - Video Players: 6 (vlc, mpv, ...)
  - Dev Tools: 45 (gcc, python, rust, ...)
  - Games: 12 (steam games)
  - Creative: 8 (blender, gimp, ...)

**Output Example**:
```
╭─ System Profile ──────────────────────────
│
│  Distro:   Arch Linux (rolling)
│  Kernel:   6.17.6-arch1-1
│  Packages: 847 installed
│
│  Software Categories:
│    Development Tools:  45 packages
│    Gaming:             12 packages
│    Creative Suite:     8 packages
│    Text Editors:       3 packages
│    Browsers:           2 packages
│    Video Players:      6 packages
│
│  Profile:  Developer + Gamer
│
╰───────────────────────────────────────────
```

### 3. Behavior Radar

**Purpose**: Show how YOU actually use your system

**Categories** (tracked over 7 days):
- **Time Distribution**:
  - Coding: 40% (IDE, terminal, git)
  - Browsing: 30% (firefox, chrome)
  - Gaming: 20% (steam, wine)
  - Reading: 5% (PDF readers, documents)
  - Creative: 5% (blender, gimp)

- **Work Patterns**:
  - Peak Hours: 9 AM - 6 PM
  - Night Owl: Yes (active past midnight)
  - Weekend Usage: Heavy

**Output Example**:
```
╭─ Usage Behavior (Last 7 Days) ───────────
│
│  Time Distribution:
│    Coding     [████████████░░░] 40%
│    Browsing   [█████████░░░░░░] 30%
│    Gaming     [██████░░░░░░░░░] 20%
│    Creative   [███░░░░░░░░░░░░]  8%
│    Other      [░░░░░░░░░░░░░░░]  2%
│
│  Work Patterns:
│    Peak Hours:      9 AM - 6 PM
│    Night Owl:       Yes (active past 1 AM)
│    Weekend Warrior: Heavy usage Sat/Sun
│
│  Profile:  Software Developer
│            (with gaming & creative hobbies)
│
╰───────────────────────────────────────────
```

---

## Persona Classification

Combining all three radars gives you a **User Persona**:

**Example Persona**: "Software Developer"
- **Hardware**: High-end gaming/workstation rig
- **System**: Arch Linux with dev-heavy software
- **Behavior**: 40% coding, 20% gaming, active late nights

**Other Personas**:
- **Casual User**: Low-spec hardware, minimal packages, mostly browsing
- **Creative Professional**: High RAM/GPU, Adobe/creative tools, regular work hours
- **Gamer**: High-end GPU, Steam-heavy, evening/weekend usage
- **Data Scientist**: Python-heavy packages, Jupyter, long-running processes
- **System Admin**: Server tools, SSH sessions, automation scripts

---

## Implementation Roadmap

**v0.12.4** (Current):
- ✓ Technical health radars (CPU, memory, disk, thermal, network)
- ✓ `annactl doctor check` for system diagnostics
- ✓ Clean number formatting (10/10 not 10.0/10)

**v0.13.0** (Next):
- Hardware detection radar
- Package categorization system
- Basic behavior tracking (window focus time)

**v0.14.0** (Future):
- Full persona classification
- Behavior analysis with trends
- Personalized recommendations based on profile

---

## Why This Matters

**Current radars**: "Your CPU is at 10/10"
→ Tells you nothing useful

**Future radars**: "You're a Software Developer with a gaming PC who codes 40% of the time and games 20%"
→ Now Anna can give you **relevant** advice:
  - Suggest dev tools you don't have
  - Optimize for your workload
  - Recommend games based on your hardware
  - Adjust system settings for your usage pattern

This is the difference between a **monitoring tool** and an **intelligent assistant**.
