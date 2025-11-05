# Changelog

All notable changes to Anna Assistant will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0-beta.34] - 2025-11-05

### 📊 History Tracking & Enhanced Wiki Cache!

**ANALYTICS:** Track your system improvements over time! See success rates, top categories, and health improvements.

### ✨ Major Features

**📈 Application History Tracking**
- Persistent JSONL-based history at `/var/log/anna/application_history.jsonl`
- Tracks every recommendation you apply with full details
- Records success/failure status and health score changes
- Command-level audit trail with timestamps

**📊 Analytics & Insights**
- Success rate calculations with visual progress bars
- Top category analysis - see what you optimize most
- Average health improvement tracking
- Period-based statistics (last N days)
- Detailed entry view for troubleshooting

**🖥️ New `annactl history` Command**
- `--days N` - Show history for last N days (default: 30)
- `--detailed` - Show full command output and details
- Beautiful visual bars for success rates
- Category popularity ranking with charts
- Health score improvement trends

**📚 Massively Expanded Wiki Cache**
- Increased from 15 to 40+ essential Arch Wiki pages
- Categories: Installation, Security, Package Management, Hardware, Desktop Environments
- Development tools (Python, Rust, Node.js, Docker, Git)
- Gaming pages (Gaming, Steam, Wine)
- Network configuration (SSH, Firewall, Wireless)
- Power management for laptops (TLP, powertop)
- Troubleshooting resources (FAQ, Debugging)

### 🔧 Technical Details

**History Module:**
```rust
pub struct HistoryEntry {
    pub advice_id: String,
    pub advice_title: String,
    pub category: String,
    pub applied_at: DateTime<Utc>,
    pub applied_by: String,
    pub command_run: Option<String>,
    pub success: bool,
    pub output: String,
    pub health_score_before: Option<u8>,
    pub health_score_after: Option<u8>,
}

pub struct ApplicationHistory {
    pub entries: Vec<HistoryEntry>,
}

impl ApplicationHistory {
    pub fn success_rate(&self) -> f64
    pub fn top_categories(&self, count: usize) -> Vec<(String, usize)>
    pub fn average_health_improvement(&self) -> Option<f64>
    pub fn period_stats(&self, days: i64) -> PeriodStats
}
```

**Wiki Cache Expansion:**
- Essential guides (Installation, General recommendations, System maintenance)
- Security hardening resources
- Complete hardware driver documentation (NVIDIA, Intel, AMD)
- All major desktop environments (GNOME, KDE, Xfce)
- Development language resources
- Gaming optimization guides
- Network and SSH configuration
- Laptop power management

### 💡 What This Means

**Track Your Progress:**
- See how many recommendations you've applied
- Monitor your success rate over time
- Identify which categories you optimize most
- Measure actual health score improvements

**Data-Driven Decisions:**
- Understand which optimizations work best
- See trends in your system maintenance
- Identify patterns in failures for better troubleshooting

**Enhanced Offline Access:**
- 40+ essential Arch Wiki pages cached locally
- Faster access to documentation
- Work offline with full wiki resources
- Curated selection of most useful pages

### 📊 Example Usage

**View Recent History:**
```bash
annactl history --days 7
```

**Detailed Output:**
```bash
annactl history --days 30 --detailed
```

**Example Output:**
```
📊 Last 30 Days

  Total Applications:  42
  Successful:          39
  Failed:              3
  Success Rate:        92.9%

  ████████████████████████████████████░░░░

  📈 Top Categories:
     1. security           15  ████████████████████
     2. performance        12  ████████████████
     3. hardware           8   ███████████
     4. packages           5   ███████
     5. development        2   ███

  Average Health Improvement: +5.3 points
```

## [1.0.0-beta.33] - 2025-01-05

### 📚 Smart Recommendations & Wiki Integration!

**WORKFLOW-AWARE:** Anna now suggests packages based on YOUR workflow and displays wiki links for learning!

### ✨ Major Features

**🎯 Smart Package Recommendation Engine**
- Analyzes your development profile and suggests missing LSP servers
- Recommends gaming enhancements based on detected games/platforms
- Suggests desktop environment-specific tools
- Proposes networking tools based on your setup
- Recommends laptop power management tools
- Content creation tool suggestions

**📖 Wiki Link Display**
- Every recommendation now shows relevant Arch Wiki links
- Beautiful "📚 Learn More" section with clickable URLs
- Direct links to official documentation
- Category-specific wiki pages

**🧠 Workflow Detection**
- Python developers → pyright LSP server
- Rust developers → rust-analyzer
- Go developers → gopls
- TypeScript/JavaScript → typescript-language-server
- Steam users → ProtonGE, MangoHud
- Laptop users → TLP, powertop
- And many more!

### 🔧 Technical Details

**Smart Recommender Module:**
- `smart_recommender.rs` - New module with workflow-based logic
- Analyzes `DevelopmentProfile`, `GamingProfile`, `NetworkProfile`
- Detects missing LSP servers by language
- Context-aware package suggestions
- Integration with existing recommendation pipeline

**Recommendation Categories:**
- Development tools (LSP servers, debuggers, container tools)
- Gaming enhancements (Proton-GE, MangoHud, gamepad support)
- Desktop environment tools (GNOME Tweaks, KDE themes)
- Network tools (WireGuard, OpenSSH)
- Content creation (OBS plugins)
- Laptop utilities (TLP, powertop)

**Functions:**
```rust
pub fn generate_smart_recommendations(facts: &SystemFacts) -> Vec<Advice>
fn recommend_for_development(profile: &DevelopmentProfile) -> Vec<Advice>
fn recommend_for_gaming(profile: &GamingProfile) -> Vec<Advice>
fn recommend_for_desktop(de: &str) -> Vec<Advice>
fn recommend_for_networking(profile: &NetworkProfile) -> Vec<Advice>
fn recommend_for_content_creation() -> Vec<Advice>
fn recommend_for_laptop() -> Vec<Advice>
```

### 💡 What This Means

**For Developers:**
- Automatic detection of missing language servers
- Never miss essential development tools
- LSP suggestions for Python, Rust, Go, TypeScript
- Container tool recommendations (docker-compose)
- Debugger suggestions (GDB for C/C++)

**For Gamers:**
- ProtonGE recommendations for better game compatibility
- MangoHud for performance monitoring
- Gamepad driver suggestions
- Steam-specific enhancements

**For Everyone:**
- Learn more with integrated wiki links
- Discover tools you didn't know existed
- Category-specific recommendations
- Laptop-specific power management
- Desktop environment enhancements

### 📊 Example Recommendations

**Development:**
```
[1]  Install Rust Language Server (rust-analyzer)

  RECOMMENDED  LOW RISK

  You have 45 Rust files but no LSP server installed. rust-analyzer
  provides excellent IDE features for Rust development.

  Action:
  ❯ sudo pacman -S rust-analyzer

  📚 Learn More:
  https://wiki.archlinux.org/title/Rust

  ID: rust-analyzer
```

**Gaming:**
```
[5]  Install MangoHud for in-game performance overlay

  OPTIONAL  LOW RISK

  MangoHud shows FPS, GPU/CPU usage, and temperatures in games.
  Great for monitoring performance.

  Action:
  ❯ sudo pacman -S mangohud

  📚 Learn More:
  https://wiki.archlinux.org/title/Gaming#Performance_overlays

  ID: mangohud
```

**Laptop:**
```
[7]  Install TLP for better battery life

  RECOMMENDED  LOW RISK

  TLP is an advanced power management tool that can significantly
  extend your laptop's battery life.

  Action:
  ❯ sudo pacman -S tlp && sudo systemctl enable tlp

  📚 Learn More:
  https://wiki.archlinux.org/title/TLP

  ID: tlp-power
```

### 🎨 UI Enhancements

**Wiki Link Section:**
- Beautiful "📚 Learn More" header
- Blue italic links for easy scanning
- Multiple wiki references when relevant
- Category wiki pages included

**Recommendation Quality:**
- Context-aware descriptions
- File counts in explanations ("You have 45 Rust files...")
- Platform-specific suggestions
- Clear installation commands

### 🏗️ Infrastructure

**New Module:**
- `crates/annad/src/smart_recommender.rs` - 280+ lines
- Integrated into advice generation pipeline
- Works alongside existing recommenders
- Updates on system refresh

**Integration Points:**
- Called during initial advice generation
- Included in refresh_advice() updates
- Uses existing SystemFacts data
- Seamless with learning system (can be dismissed)

### 📝 Notes

- Smart recommendations respect feedback system
- Can be dismissed like any other advice
- Learning system tracks preferences
- All recommendations have wiki links
- Low-risk, high-value suggestions

### 🎯 Detection Examples

**Detects:**
- 50+ Python files → suggests pyright
- Steam installed → suggests ProtonGE
- Laptop detected → suggests TLP
- C/C++ projects → suggests GDB
- Docker usage → suggests docker-compose
- GNOME desktop → suggests gnome-tweaks
- No VPN → suggests WireGuard

### 🚀 Future Enhancements

Planned improvements:
- ML-based package suggestions
- Community package recommendations
- AUR package smart detection
- Workflow bundle creation from suggestions
- Installation success tracking

## [1.0.0-beta.32] - 2025-01-05

### 🧠 Learning System & Health Scoring!

**ADAPTIVE INTELLIGENCE:** Anna now learns from your behavior and tracks system health with detailed scoring!

### ✨ Major Features

**📊 System Health Scoring**
- Comprehensive health score (0-100) with letter grades (A+ to F)
- Breakdown by category: Security, Performance, Maintenance
- Visual score bars and trend indicators (Improving/Stable/Declining)
- Intelligent health interpretation with actionable next steps
- New `annactl health` command for quick health check

**🎓 Learning & Feedback System**
- Tracks user interactions: applied, dismissed, viewed
- Learns category preferences from your behavior
- Auto-hides dismissed recommendations
- Persistent feedback log at `/var/log/anna/feedback.jsonl`
- New `annactl dismiss` command to hide unwanted advice
- Automatic feedback recording when applying recommendations

**🎯 New CLI Commands**
- `annactl health` - Show system health score with visual breakdown
- `annactl dismiss --id <id>` or `--num <n>` - Dismiss recommendations

### 🔧 Technical Details

**Learning System:**
- `FeedbackEvent` - Track user interactions with timestamps
- `UserFeedbackLog` - Persistent JSONL storage
- `LearnedPreferences` - Analyze patterns from feedback
- `FeedbackType` enum: Applied, Dismissed, Viewed

**Health Scoring:**
- `SystemHealthScore` - Overall + category scores
- `HealthTrend` enum: Improving, Stable, Declining
- Weighted calculation: Security (40%), Performance (30%), Maintenance (30%)
- Dynamic scoring based on system facts and pending advice

**Data Structures:**
```rust
pub struct SystemHealthScore {
    pub overall_score: u8,       // 0-100
    pub security_score: u8,
    pub performance_score: u8,
    pub maintenance_score: u8,
    pub issues_count: usize,
    pub critical_issues: usize,
    pub health_trend: HealthTrend,
}

pub struct FeedbackEvent {
    pub advice_id: String,
    pub advice_category: String,
    pub event_type: FeedbackType,
    pub timestamp: DateTime<Utc>,
    pub username: String,
}

pub struct LearnedPreferences {
    pub prefers_categories: Vec<String>,
    pub dismisses_categories: Vec<String>,
    pub power_user_level: u8,
}
```

### 💡 What This Means

**For Users:**
- Get instant feedback on system health (like a report card!)
- Anna learns what you care about and what you don't
- Dismissed advice stays hidden - no more seeing the same unwanted suggestions
- Clear, actionable guidance based on your health score

**For System Monitoring:**
- Track health trends over time
- See exactly which areas need attention
- Understand the impact of applied recommendations
- Get grade-based assessments (A+ to F)

**For Personalization:**
- Anna adapts to YOUR preferences
- Categories you dismiss appear less frequently
- Categories you apply get prioritized
- Power user detection based on behavior

### 📊 Usage Examples

**Check System Health:**
```bash
# Show full health score
annactl health

# Output example:
#   📊 Overall Health
#
#      85/100  B+
#      ██████████████████████████████████████████░░░░░░░░░░
#      Trend: → Stable
#
#   📈 Score Breakdown
#   Security              95  ███████████████████
#   Performance           80  ████████████████
#   Maintenance           75  ███████████████
```

**Dismiss Unwanted Advice:**
```bash
# Dismiss by ID
annactl dismiss --id orphan-packages

# Dismiss by number from advise list
annactl dismiss --num 5
```

**See Learning in Action:**
```bash
# Dismissed items are automatically hidden
annactl advise
# Output: "Hiding 3 previously dismissed recommendation(s)"
```

### 🎨 UI Enhancements

**Health Score Display:**
- Large, colorful score display with grade letter
- Visual progress bars (█ for filled, ░ for empty)
- Color-coded scores: Green (90+), Yellow (70-89), Orange (50-69), Red (<50)
- Trend arrows: ↗ Improving, → Stable, ↘ Declining
- Contextual interpretation based on score range
- Specific next steps based on issues

**Feedback Integration:**
- Automatic notification when advice is dismissed
- Confirmation when feedback is recorded
- Learning message: "Anna will learn from your preferences"

### 🏗️ Infrastructure

**New Features:**
- Feedback logging with JSONL format
- Dismissal tracking per advice ID
- Category-level preference analysis
- Health score caching (planned)
- Trend calculation from historical data (planned)

**Integration Points:**
- `apply` command now records successful applications
- `dismiss` command records user rejections
- `advise` command filters out dismissed items
- `health` command calculates real-time scores

### 📝 Notes

- Feedback log persists across daemon restarts
- Dismissed advice can be re-enabled by deleting feedback log
- Health scores are calculated in real-time (no caching yet)
- Learning improves with more user interactions
- All feedback is user-specific (username tracked)

### 🎯 What's Next

Planned improvements:
- Health score history tracking
- Trend calculation from historical scores
- ML-based recommendation prioritization
- Category weight adjustment based on preferences
- Export feedback data for analysis

## [1.0.0-beta.31] - 2025-01-05

### 🤖 Autonomous Maintenance & Offline Wiki Cache!

**MAJOR UPDATE:** Anna can now maintain your system autonomously and provides offline access to Arch Wiki pages!

### ✨ Major Features

**🔧 Low-Level Autonomy System**
- 4-tier autonomy system for safe automatic maintenance
- Tier 0 (Advise Only): Monitor and report only
- Tier 1 (Safe Auto-Apply): Clean orphan packages, package cache, and journal
- Tier 2 (Semi-Autonomous): + Remove old kernels, clean tmp directories
- Tier 3 (Fully Autonomous): + Update mirrorlist automatically
- Comprehensive action logging with undo capabilities
- Scheduled autonomous runs every 6 hours
- Smart thresholds (10+ orphans, 5GB+ cache, 1GB+ logs)

**📚 Arch Wiki Offline Cache**
- Download and cache 15 common Arch Wiki pages
- HTML parsing and content extraction
- Checksum-based change detection
- 7-day automatic refresh cycle
- Fallback to online fetch if cache is stale
- Pages cached: Security, Performance, System Maintenance, Power Management, Pacman, Systemd, Kernel Parameters, Docker, Python, Rust, Gaming, Firewall, SSH, Hardware, Desktop Environment

**🎯 New CLI Commands**
- `annactl autonomy [--limit=20]` - View autonomous actions log
- `annactl wiki-cache [--force]` - Update Arch Wiki cache

### 🔧 Technical Details

**Autonomy System:**
- `autonomy.rs` - Core autonomy logic with tier-based execution
- `AutonomyAction` - Action tracking with timestamps, success/failure, output
- `AutonomyLog` - Persistent logging to `/var/log/anna/autonomy.jsonl`
- Safe execution with detailed output capture
- Undo command tracking for reversible operations

**Autonomy Tasks:**
- Tier 1: `clean_orphan_packages()`, `clean_package_cache()`, `clean_journal()`
- Tier 2: `remove_old_kernels()`, `clean_tmp_dirs()`
- Tier 3: `update_mirrorlist()`
- Each task respects safety thresholds and logs all operations

**Wiki Cache System:**
- `wiki_cache.rs` - Wiki fetching and caching infrastructure
- `WikiCacheEntry` - Page metadata, content, timestamp, checksum
- `WikiCache` - Cache management with refresh logic
- HTTP fetching with curl
- Smart HTML content extraction
- Automatic cache refresh when stale (>7 days)

**Data Structures:**
```rust
pub struct AutonomyAction {
    pub action_type: String,
    pub executed_at: DateTime<Utc>,
    pub description: String,
    pub command_run: String,
    pub success: bool,
    pub output: String,
    pub can_undo: bool,
    pub undo_command: Option<String>,
}

pub struct WikiCacheEntry {
    pub page_title: String,
    pub url: String,
    pub content: String,
    pub cached_at: DateTime<Utc>,
    pub checksum: String,
}
```

### 💡 What This Means

**For Users:**
- Your system can now maintain itself automatically (if you enable it)
- Safe, conservative defaults - only truly safe operations in Tier 1
- Full transparency - every autonomous action is logged
- Offline access to critical Arch Wiki pages
- No more hunting for wiki pages when offline

**For System Health:**
- Automatic cleanup of orphaned packages
- Automatic cache management
- Log rotation to save space
- Old kernel removal (keeps 2 latest)
- Updated mirrorlist for faster downloads (Tier 3)

**For Power Users:**
- Fine-grained control via 4 autonomy tiers
- Comprehensive action logging with timestamps
- Undo capability for reversible operations
- Configure via: `annactl config --set autonomy_tier=<0-3>`

### 📊 Usage Examples

**View Autonomous Actions:**
```bash
# View last 20 actions
annactl autonomy

# View more/fewer
annactl autonomy --limit=50
annactl autonomy --limit=10
```

**Configure Autonomy:**
```bash
# Enable safe auto-apply (Tier 1)
annactl config --set autonomy_tier=1

# Semi-autonomous (Tier 2)
annactl config --set autonomy_tier=2

# Fully autonomous (Tier 3)
annactl config --set autonomy_tier=3

# Back to advise-only (Tier 0)
annactl config --set autonomy_tier=0
```

**Wiki Cache:**
```bash
# Update cache (only if stale)
annactl wiki-cache

# Force refresh
annactl wiki-cache --force
```

### 🎨 UI Enhancements

**Autonomy Log Display:**
- Color-coded success/failure indicators
- Action type badges (CLEANUP, MAINT, UPDATE)
- Timestamps for all actions
- Command execution details
- Output preview (first 3 lines)
- Undo command display when available
- Clean, readable formatting with separators

### 🏗️ Infrastructure

**New Modules:**
- `crates/annad/src/autonomy.rs` - Autonomous maintenance system
- `crates/annad/src/wiki_cache.rs` - Wiki caching infrastructure

**Daemon Integration:**
- Periodic autonomy runs scheduled every 6 hours
- Integrated into main event loop
- Error handling and logging
- Respects user configuration

### ⚙️ Configuration

Default autonomy configuration:
```toml
[autonomy]
tier = "AdviseOnly"  # Safe default
confirm_high_risk = true
snapshot_before_apply = false
```

### 📝 Notes

- Autonomy is opt-in (defaults to Tier 0 - Advise Only)
- All autonomous actions are logged for transparency
- Wiki cache update via RPC will be implemented in next version
- Autonomy scheduling is configurable via refresh_interval setting

## [1.0.0-beta.30] - 2025-01-04

### 🧠 Deep System Intelligence & Dynamic Categories!

**GAME CHANGER:** Anna now deeply understands your workflow, preferences, and system state! Categories are dynamic and linked to Arch Wiki.

### ✨ Major Features

**📊 Comprehensive Telemetry System**
- 10 new data structures for deep system understanding
- 30+ new collection functions
- Real-time system state analysis
- Intelligent preference detection

**🎯 Dynamic Category System**
- Categories now show plain English names (e.g., "Security & Privacy" not "security")
- Only displays categories relevant to YOUR system
- Each category linked to official Arch Wiki documentation
- Rich descriptions for every category
- 12 categories: Security & Privacy, Performance & Optimization, Hardware Support, Network Configuration, Desktop Environment, Development Tools, Gaming & Entertainment, Multimedia & Graphics, System Maintenance, Terminal & CLI Tools, Power Management, System Configuration

**🔍 Advanced System Understanding**

*Development Profile:*
- Detects programming languages used (Python, Rust, Go, JavaScript)
- Counts projects and files per language
- Tracks LSP server installation status
- Detects IDEs (VSCode, Vim, Neovim, Emacs, IntelliJ, PyCharm, CLion)
- Counts Git repositories
- Detects container usage (Docker/Podman)
- Detects virtualization (QEMU/VirtualBox/VMware)

*Gaming Profile:*
- Steam/Lutris/Wine detection
- ProtonGE and MangoHud status
- Gamepad driver detection
- Game count tracking

*Network Profile:*
- VPN configuration detection (WireGuard/OpenVPN)
- Firewall status (UFW/iptables)
- SSH server monitoring
- DNS configuration (systemd-resolved/dnsmasq)
- Network share detection (NFS/Samba)

*User Preferences (AI-inferred):*
- CLI vs GUI preference
- Power user detection
- Aesthetics appreciation
- Gamer/Developer/Content Creator profiles
- Laptop user detection
- Minimalism preference

*System Health:*
- Recent package installations (last 30 days)
- Active and enabled services
- Disk usage trends with largest directories
- Cache and log sizes
- Session information (login patterns, multiple users)
- System age tracking

### 🔧 Technical Improvements

**New Data Structures:**
- `CategoryInfo` - Arch Wiki-aligned categories with metadata
- `PackageInstallation` - Installation tracking with timestamps
- `DiskUsageTrend` - Space analysis and trends
- `DirectorySize` - Storage consumption tracking
- `SessionInfo` - User activity patterns
- `DevelopmentProfile` - Programming environment analysis
- `LanguageUsage` - Per-language statistics and LSP status
- `ProjectInfo` - Active project tracking
- `GamingProfile` - Gaming setup detection
- `NetworkProfile` - Network configuration analysis
- `UserPreferences` - AI-inferred user behavior

**New Telemetry Functions:**
- `get_recently_installed_packages()` - Track what was installed when
- `get_active_services()` / `get_enabled_services()` - Service monitoring
- `analyze_disk_usage()` - Comprehensive storage analysis
- `collect_session_info()` - User activity patterns
- `analyze_development_environment()` - Deep dev tool detection
- `detect_programming_languages()` - Language usage analysis
- `count_files_by_extension()` - Project scope analysis
- `detect_ides()` - IDE installation detection
- `count_git_repos()` - Development activity
- `analyze_gaming_profile()` - Gaming setup detection
- `analyze_network_profile()` - Network configuration
- `get_system_age_days()` - Installation age tracking
- `infer_user_preferences()` - Behavioral analysis
- 20+ helper functions for deep system inspection

### 💡 What This Means

Anna now knows:
- **What you build**: "You're working on 3 Python projects with 150 .py files"
- **How you work**: "CLI power user with Neovim and tmux"
- **What you do**: "Gamer with Steam + ProtonGE, Developer with Docker"
- **Your style**: "Values aesthetics (starship + eza installed), prefers minimalism"
- **System health**: "5.2GB cache, logs growing, 42 active services"

This enables **context-aware recommendations** that understand YOUR specific setup and workflow!

### 📦 User Experience Improvements

- Category names are now human-friendly everywhere
- `annactl advise` shows categories with descriptions
- `annactl report` displays categories relevant to your system
- Each category shows item count and purpose
- Wiki links provided for deeper learning

### 📈 Performance & Reliability

- Intelligent caching of telemetry data
- Limited search depths to prevent slowdowns
- Graceful fallbacks for unavailable data
- Async operations for non-blocking collection

## [1.0.0-beta.29] - 2025-01-04

### 🔄 Bundle Rollback System!

**NEW:** Safely rollback workflow bundles with full tracking and reverse dependency order removal!

### ✨ Added

**🔄 Bundle Rollback Feature**
- New `annactl rollback --bundle "Bundle Name"` command
- Full installation history tracking stored in `/var/lib/anna/bundle_history.json`
- Tracks what was installed, when, and by whom
- Automatic reverse dependency order removal
- `--dry-run` support to preview what will be removed
- Interactive confirmation before removal
- Safe rollback only for completed installations

**📊 Bundle History System**
- New `BundleHistory` type for tracking installations
- `BundleHistoryEntry` records each installation with:
  - Bundle name and installed items
  - Installation timestamp and user
  - Status (Completed/Partial/Failed)
  - Rollback availability flag
- Persistent storage with JSON format
- Automatic directory creation

**🛡️ Safety Features**
- Only completed bundles can be rolled back
- Partial/failed installations are tracked but not rolled back
- Interactive prompt before removing packages
- Graceful handling of already-removed packages
- Detailed status reporting during rollback

### 🔧 Technical Improvements
- Added `BundleStatus` enum (Completed/Partial/Failed)
- Added `BundleHistoryEntry` and `BundleHistory` types
- Implemented bundle history load/save with JSON serialization
- Updated `apply_bundle()` to track installations
- Added `rollback()` function with reverse-order removal
- CLI command structure extended with Rollback subcommand

### 📦 Example Usage

```bash
# Install a bundle (now tracked for rollback)
annactl apply --bundle "Python Development Stack"

# See what would be removed
annactl rollback --bundle "Python Development Stack" --dry-run

# Rollback a bundle
annactl rollback --bundle "Python Development Stack"

# View installation history
cat /var/lib/anna/bundle_history.json
```

### 💡 How It Works

1. **Installation Tracking**: When you install a bundle, Anna records:
   - Which items were installed
   - Timestamp and username
   - Success/failure status

2. **Reverse Order Removal**: Rollback removes items in reverse dependency order:
   - If you installed: Docker → docker-compose → lazydocker
   - Rollback removes: lazydocker → docker-compose → Docker

3. **Safety First**: Only fully completed bundles can be rolled back, preventing partial rollbacks that could break dependencies.

## [1.0.0-beta.28] - 2025-01-04

### 🎁 Workflow Bundles & Enhanced Reporting!

**NEW:** One-command workflow bundle installation with smart dependency resolution! Plus enhanced report command with category filtering.

### ✨ Added

**📦 Workflow Bundle System**
- New `annactl bundles` command to list available workflow bundles
- Install complete development stacks with `annactl apply --bundle "Bundle Name"`
- Smart dependency resolution using Kahn's algorithm (topological sort)
- Bundles install tools in the correct order automatically
- Three predefined bundles:
  - "Container Development Stack" (Docker → docker-compose → lazydocker)
  - "Python Development Stack" (python-lsp-server, python-black, ipython)
  - "Rust Development Stack" (rust-analyzer)
- `--dry-run` support to preview what will be installed
- Progress tracking showing X/Y items during installation

**📊 Enhanced Report Command**
- New `--category` flag to filter reports by category
- `annactl report --category security` shows only security recommendations
- `annactl report --category development` shows only dev tools
- Helpful error message listing available categories if category not found
- Report output speaks plain English with sysadmin-level insights

### 🔧 Technical Improvements
- Added `bundles()` function with bundle grouping and display
- Added `apply_bundle()` function with dependency resolution
- Added `topological_sort()` implementing Kahn's algorithm for dependency ordering
- Bundle metadata integration across Docker, Python, and Rust recommendations
- Category parameter support in report generation

### 📦 Example Usage

```bash
# List available bundles
annactl bundles

# Install a complete workflow bundle
annactl apply --bundle "Python Development Stack"

# Preview what will be installed
annactl apply --bundle "Container Development Stack" --dry-run

# Get a focused report on security issues
annactl report --category security
```

## [1.0.0-beta.27] - 2025-01-04

### 🚀 Advanced Telemetry & Intelligent Recommendations!

**GAME CHANGER:** Anna now analyzes boot performance, AUR usage, package cache, kernel parameters, and understands workflow dependencies!

### ✨ Added

**⚡ Boot Performance Analysis**
- Tracks total boot time using `systemd-analyze time`
- Detects slow-starting services (>5 seconds)
- Identifies failed systemd services
- Recommends disabling `NetworkManager-wait-online` and other slow services
- Links to Arch Wiki boot optimization guides

**🎯 AUR Helper Intelligence**
- Counts AUR packages vs official repos using `pacman -Qm`
- Detects which AUR helper is installed (yay, paru, aurutils, pikaur, aura, trizen)
- Suggests installing AUR helper if you have AUR packages but no helper
- Recommends paru over yay for users with 20+ AUR packages (faster, Rust-based)
- Offers 3 alternatives with trade-offs explained

**💾 Package Cache Intelligence**
- Monitors `/var/cache/pacman/pkg/` size with `du`
- Warns when cache exceeds 5GB
- Suggests `paccache` for safe cleanup
- Offers 3 cleanup strategies:
  - Keep last 3 versions (safe default)
  - Keep last 1 version (aggressive, saves more space)
  - Remove all uninstalled packages
- Auto-suggests installing `pacman-contrib` if needed

**🔧 Kernel Parameter Optimization**
- Parses `/proc/cmdline` for current boot parameters
- Suggests `noatime` for SSD systems (reduces wear)
- Recommends `quiet` parameter for cleaner boot screen
- Links to Arch Wiki kernel parameter documentation

**🔗 Dependency Chains & Workflow Bundles**
- Added 3 new fields to Advice struct:
  - `depends_on: Vec<String>` - IDs that must be applied first
  - `related_to: Vec<String>` - Suggestions for related advice
  - `bundle: Option<String>` - Workflow bundle name
- Foundation for smart ordering and grouped recommendations
- Example: "Container Development Stack" (Docker → docker-compose → lazydocker)

### 📊 Enhanced Telemetry (10 New Fields)

**Boot Performance**
- `boot_time_seconds: Option<f64>`
- `slow_services: Vec<SystemdService>`
- `failed_services: Vec<String>`

**Package Management**
- `aur_packages: usize`
- `aur_helper: Option<String>`
- `package_cache_size_gb: f64`
- `last_system_upgrade: Option<DateTime<Utc>>`

**Kernel & Boot**
- `kernel_parameters: Vec<String>`

**Advice Metadata**
- `depends_on: Vec<String>`
- `related_to: Vec<String>`
- `bundle: Option<String>`

### 🛠️ New Detection Functions

- `get_boot_time()` - Parse systemd-analyze output
- `get_slow_services()` - Find services taking >5s to start
- `get_failed_services()` - List failed systemd units
- `count_aur_packages()` - Count foreign packages
- `detect_aur_helper()` - Find installed AUR helper
- `get_package_cache_size()` - Calculate cache size in GB
- `get_last_upgrade_time()` - Parse pacman.log timestamps
- `get_kernel_parameters()` - Read /proc/cmdline
- `check_boot_performance()` - Generate boot recommendations
- `check_package_cache()` - Generate cache recommendations
- `check_aur_helper_usage()` - Generate AUR helper recommendations
- `check_kernel_params_optimization()` - Generate kernel parameter recommendations

### 🎯 Real-World Impact

**Boot Optimization Example:**
```
[15] Disable slow service: NetworkManager-wait-online.service (12.3s)
     RECOMMENDED   LOW RISK

     NetworkManager-wait-online delays boot waiting for network.
     Most systems don't need this.

     ❯ systemctl disable NetworkManager-wait-online.service
```

**Package Cache Cleanup Example:**
```
[23] Package cache is large (8.4 GB)
     RECOMMENDED   LOW RISK

     Alternatives:
     ★ Keep last 3 versions - Safe default
     ○ Keep last 1 version - More aggressive
     ○ Remove uninstalled packages
```

### 🔧 Technical

- Added `SystemdService` type for boot analysis
- All new telemetry functions are async-compatible
- Dependency tracking foundation for future auto-ordering
- Workflow bundles enable "install complete stack" features

## [1.0.0-beta.26] - 2025-01-04

### 🎨 Software Alternatives - Choose What You Love!

**THE FEATURE YOU ASKED FOR:** Instead of "install X", Anna now offers 2-3 alternatives for most tools!

### ✨ Added

**🔄 Software Alternatives System**
- New `Alternative` type with name, description, and install command
- Visual display with ★ for recommended option, ○ for alternatives
- Wrapped descriptions for readability
- Install commands shown for each option

**🛠️ Tools with Alternatives (5 major categories)**
- **Status bars**: Waybar, eww, yambar
- **Application launchers**: Wofi, Rofi (Wayland), Fuzzel
- **Notification daemons**: Mako, Dunst, SwayNC
- **Terminal emulators**: Alacritty, Kitty, WezTerm
- **Web browsers**: Firefox, Chromium, LibreWolf

### 🎯 Why This Matters
- User choice > forced recommendations
- See trade-offs at a glance (performance vs features)
- Learn about alternatives you might not know
- Better UX: "choose what fits you" vs "install this one thing"

### 🔧 Technical
- Added `alternatives: Vec<Alternative>` field to `Advice` struct
- Backward compatible with `#[serde(default)]`
- Enhanced `display_advice_item_enhanced()` to show alternatives
- All existing advice gets empty alternatives by default

## [1.0.0-beta.25] - 2025-01-04

### 🧠 MAJOR UX OVERHAUL - Smart Filtering & Intelligence!

**THE BIG PROBLEM SOLVED:** 80+ recommendations was overwhelming. Now you see ~25 most relevant by default!

### ✨ Added

**🎯 Smart Filtering System**
- **Smart Mode (default)**: Shows ~25 most relevant recommendations
- **Critical Mode** (`--mode=critical`): Security & mandatory items only
- **Recommended Mode** (`--mode=recommended`): Critical + recommended items
- **All Mode** (`--mode=all`): Everything for power users
- **Category Filter** (`--category=security`): Focus on specific categories
- **Limit Control** (`--limit=10`): Control number of results

**🧠 Intelligent Behavior-Based Detection (3 new rules)**
- Docker power users → docker-compose recommendations (50+ docker commands)
- Python developers → pyenv suggestions (30+ python commands)
- Git power users → lazygit recommendations (50+ git commands)

**📊 Enhanced Report Command**
- Sysadmin-level system health analysis
- Hardware specs (CPU, RAM, GPU)
- Storage analysis with visual indicators
- Software environment details
- Development tools detection
- Network capabilities overview
- Color-coded status indicators

**🎨 Better Discoverability**
- Helpful footer with command examples
- Category list with item counts
- Clear filtering indicators
- Quick action guide

### 🐛 Fixed
- Desktop environment detection now works when daemon runs as root
- No more irrelevant suggestions (KDE tips on GNOME systems)
- Installer box rendering with proper width calculation
- Removed unused functions causing build warnings

### 🔧 Changed
- Default `annactl advise` now shows smart-filtered view (was: show all)
- Recommendations sorted by relevance and priority
- Better visual hierarchy in output

## [1.0.0-beta.24] - 2025-01-04

### ✨ Added

**🎨 Beautiful Category-Based Output**
- 80-character boxes with centered, color-coded category titles
- 14 organized categories with emojis
- Priority badges (CRITICAL, RECOMMENDED, OPTIONAL, COSMETIC)
- Risk level indicators (HIGH RISK, MED RISK, LOW RISK)
- Smart sorting by priority and risk within categories

**⚙️ Configuration System**
- TOML-based configuration at `~/.config/anna/config.toml`
- 6 sections: General, Autonomy, Notifications, Snapshots, Learning, Categories
- Auto-creation with sensible defaults

**💾 Snapshot & Rollback System**
- Multi-backend support: Btrfs, Timeshift, rsync
- Automatic snapshots before risky operations
- Retention policies with automatic cleanup

**📊 Deep Telemetry Foundation**
- Process CPU time tracking
- Bash/zsh history parsing
- Workflow pattern detection
- System configuration analysis

## [1.0.0-beta.20] - 2025-01-XX

### 🌟 Professional Coverage - 220+ Rules, 95%+ Wiki Coverage! 🌟

**PHENOMENAL expansion!** Added 30+ professional-grade tools covering Python, Rust, multimedia, science, engineering, and productivity!

### ✨ Added

**🐍 Python Development Tools (3 new rules)**
- Poetry for modern dependency management
- virtualenv for isolated environments
- IPython enhanced REPL

**🦀 Rust Development Tools (2 new rules)**
- cargo-watch for automatic rebuilds
- cargo-audit for security vulnerability scanning

**📺 Terminal Tools (1 new rule)**
- tmux terminal multiplexer

**🖼️ Image Viewers (2 new rules)**
- feh for X11 (lightweight, wallpaper setter)
- imv for Wayland (fast, keyboard-driven)

**📚 Documentation (1 new rule)**
- tldr for quick command examples

**💾 Disk Management (2 new rules)**
- smartmontools for disk health monitoring
- GParted for partition management

**💬 Communication (1 new rule)**
- Discord for gaming and communities

**🔬 Scientific Computing (1 new rule)**
- Jupyter Notebook for interactive Python

**🎨 3D Graphics (1 new rule)**
- Blender for 3D modeling and animation

**🎵 Audio Production (1 new rule)**
- Audacity for audio editing

**📊 System Monitoring (1 new rule)**
- s-tui for CPU stress testing

**🏗️ CAD Software (1 new rule)**
- FreeCAD for parametric 3D modeling

**📝 Markdown Tools (1 new rule)**
- glow for beautiful markdown rendering

**📓 Note-Taking (1 new rule)**
- Obsidian for knowledge management

### 🔄 Changed
- Detection function count increased from 84 to 98 (+16%)
- Total recommendations increased from 190+ to 220+ (+15%)
- Added professional tool detection (Python/Rust dev tools)
- Scientific computing support (Jupyter)
- Engineering tools (CAD, 3D graphics)
- Enhanced disk health monitoring
- Arch Wiki coverage increased from ~90% to ~95%+

### 📊 Coverage Status
- **Total detection functions**: 98
- **Total recommendations**: 220+
- **Wiki coverage**: 95%+ for typical users
- **New professional categories**: Python Tools, Rust Tools, Scientific Computing, 3D Graphics, CAD, Engineering, Audio Production

## [1.0.0-beta.19] - 2025-01-XX

### 🎯 Complete Coverage - 190+ Rules, 90%+ Wiki Coverage! 🎯

**INCREDIBLE expansion!** Added 30+ more rules covering tools, utilities, development workflows, and system administration!

### ✨ Added

**🎵 Music Players (1 new rule)**
- MPD (Music Player Daemon) with ncmpcpp

**📄 PDF Readers (1 new rule)**
- Zathura vim-like PDF viewer

**🖥️ Monitor Management (1 new rule)**
- arandr for X11 multi-monitor setup

**⏰ System Scheduling (1 new rule)**
- Systemd timers vs cron comparison

**🐚 Shell Alternatives (1 new rule)**
- Fish shell with autosuggestions

**🗜️ Advanced Compression (1 new rule)**
- Zstandard (zstd) modern compression

**🔄 Dual Boot Support (1 new rule)**
- os-prober for GRUB multi-OS detection

**🎯 Git Advanced Tools (2 new rules)**
- git-delta for beautiful diffs
- lazygit terminal UI

**📦 Container Alternatives (1 new rule)**
- Podman rootless container runtime

**💻 Modern Code Editors (1 new rule)**
- Visual Studio Code

**🗄️ Additional Databases (2 new rules)**
- MariaDB (MySQL replacement)
- Redis in-memory database

**🌐 Network Analysis (2 new rules)**
- Wireshark packet analyzer
- nmap network scanner

**⚙️ Dotfile Management (1 new rule)**
- GNU Stow for dotfile symlinks

**📦 Package Development (2 new rules)**
- namcap PKGBUILD linter
- devtools clean chroot builds

### 🔄 Changed
- Detection function count increased from 70 to 84 (+20%)
- Total recommendations increased from 160+ to 190+ (+18%)
- Added behavior-based detection for power users
- Systemd timer suggestions for cron users
- Multi-monitor setup detection
- PKGBUILD developer tools
- Arch Wiki coverage increased from ~85% to ~90%+

### 📊 Coverage Status
- **Total detection functions**: 84
- **Total recommendations**: 190+
- **Wiki coverage**: 90%+ for typical users
- **New categories**: Music, PDF, Monitors, Scheduling, Compression, Dotfiles, Network Tools, Package Development

## [1.0.0-beta.18] - 2025-01-XX

### 🚀 Comprehensive Coverage - 160+ Rules, 85%+ Wiki Coverage!

**MASSIVE expansion!** Added 30+ new rules covering development, productivity, multimedia, networking, and creative software!

### ✨ Added

**✏️ Text Editors (1 new rule)**
- Neovim upgrade for Vim users

**📧 Mail Clients (1 new rule)**
- Thunderbird for email management

**📂 File Sharing (2 new rules)**
- Samba for Windows file sharing
- NFS for Linux/Unix file sharing

**☁️ Cloud Storage (1 new rule)**
- rclone for universal cloud sync (40+ providers)

**💻 Programming Languages - Go (2 new rules)**
- Go compiler installation
- gopls LSP server for Go development

**☕ Programming Languages - Java (2 new rules)**
- OpenJDK installation
- Maven build tool

**🟢 Programming Languages - Node.js (2 new rules)**
- Node.js and npm installation
- TypeScript for type-safe JavaScript

**🗄️ Databases (1 new rule)**
- PostgreSQL database

**🌐 Web Servers (1 new rule)**
- nginx web server

**🖥️ Remote Desktop (1 new rule)**
- TigerVNC for remote desktop access

**🌊 Torrent Clients (1 new rule)**
- qBittorrent for torrent downloads

**📝 Office Suites (1 new rule)**
- LibreOffice for document editing

**🎨 Graphics Software (2 new rules)**
- GIMP for photo editing
- Inkscape for vector graphics

**🎬 Video Editing (1 new rule)**
- Kdenlive for video editing

### 🔄 Changed
- Detection rule count increased from 130+ to 160+ (+23%)
- Now supporting 3 additional programming languages (Go, Java, Node.js/TypeScript)
- Command history analysis for intelligent editor/tool suggestions
- Arch Wiki coverage increased from ~80% to ~85%+

### 📊 Coverage Status
- **Total detection functions**: 70
- **Total recommendations**: 160+
- **Wiki coverage**: 85%+ for typical users
- **Categories covered**: Security, Desktop (8 DEs), Development (6 languages), Multimedia, Productivity, Gaming, Networking, Creative

## [1.0.0-beta.17] - 2025-01-XX

### 🌐 Privacy, Security & Gaming - Reaching 80% Wiki Coverage!

**High-impact features!** VPN, browsers, security tools, backups, screen recording, password managers, gaming enhancements, and mobile integration!

### ✨ Added

**🔒 VPN & Networking (2 new rules)**
- WireGuard modern VPN support
- NetworkManager VPN plugin recommendations

**🌐 Browser Recommendations (2 new rules)**
- Firefox/Chromium installation detection
- uBlock Origin privacy extension reminder

**🛡️ Security Tools (3 new rules)**
- rkhunter for rootkit detection
- ClamAV antivirus for file scanning
- LUKS encryption passphrase backup reminder

**💾 Backup Solutions (2 new rules)**
- rsync for file synchronization
- BorgBackup for encrypted deduplicated backups

**🎥 Screen Recording (2 new rules)**
- OBS Studio for professional recording/streaming
- SimpleScreenRecorder for easy captures

**🔐 Password Managers (1 new rule)**
- KeePassXC for secure password storage

**🎮 Gaming Enhancements (3 new rules)**
- Proton-GE for better Windows game compatibility
- MangoHud for in-game performance overlay
- Wine for Windows application support

**📱 Android Integration (2 new rules)**
- KDE Connect for phone notifications and file sharing
- scrcpy for Android screen mirroring

### 🔄 Changed
- Detection rule count increased from 110+ to 130+ (+18%)
- Arch Wiki coverage improved from 70% to ~80%
- Enhanced privacy and security recommendations

### 📚 Documentation
- README.md updated to v1.0.0-beta.17
- Wiki coverage analysis added
- CHANGELOG.md updated with beta.17 features

---

## [1.0.0-beta.16] - 2025-01-XX

### 💻 Laptop, Audio, Shell & Bootloader Enhancements!

**Complete laptop support!** Battery optimization, touchpad, backlight, webcam, audio enhancements, shell productivity tools, filesystem maintenance, and bootloader optimization!

### ✨ Added

**💻 Laptop Optimizations (4 new rules)**
- powertop for battery optimization and power tuning
- libinput for modern touchpad support with gestures
- brightnessctl for screen brightness control
- laptop-mode-tools for advanced power management

**📷 Webcam Support (2 new rules)**
- v4l-utils for webcam control and configuration
- Cheese webcam viewer for testing

**🎵 Audio Enhancements (2 new rules)**
- EasyEffects for PipeWire audio processing (EQ, bass, effects)
- pavucontrol for advanced per-app volume control

**⚡ Shell Productivity (3 new rules)**
- bash-completion for intelligent tab completion
- fzf for fuzzy finding (history, files, directories)
- tmux for terminal multiplexing and session management

**💾 Filesystem Maintenance (2 new rules)**
- ext4 fsck periodic check reminders
- Btrfs scrub for data integrity verification

**🔧 Kernel & Boot (4 new rules)**
- 'quiet' kernel parameter for cleaner boot
- 'splash' parameter for graphical boot screen
- GRUB timeout reduction for faster boot
- Custom GRUB background configuration

### 🔄 Changed
- Detection rule count increased from 90+ to 110+ (+22%)
- Enhanced laptop and mobile device support
- Improved boot experience recommendations

### 📚 Documentation
- README.md updated to v1.0.0-beta.16
- Version bumped across all crates
- CHANGELOG.md updated with beta.16 features

---

## [1.0.0-beta.15] - 2025-01-XX

### ⚡ System Optimization & Configuration!

**Essential system optimizations!** Firmware updates, SSD optimizations, swap compression, DNS configuration, journal management, AUR safety, and locale/timezone setup!

### ✨ Added

**🔧 Firmware & Hardware Optimization (2 new rules)**
- fwupd installation for automatic firmware updates
- Firmware update check recommendations

**💾 SSD Optimizations (2 new rules)**
- noatime mount option detection for reduced writes
- discard/continuous TRIM recommendations
- Automatic SSD detection via /sys/block

**🗜️ Swap Compression (1 new rule)**
- zram detection and installation for compressed swap in RAM

**🌐 DNS Configuration (2 new rules)**
- systemd-resolved recommendation for modern DNS with caching
- Public DNS server suggestions (Cloudflare, Google, Quad9)

**📜 Journal Management (2 new rules)**
- Large journal size detection and cleanup
- SystemMaxUse configuration for automatic size limiting

**🛡️ AUR Helper Safety (2 new rules)**
- PKGBUILD review reminder for security
- Development package (-git/-svn) update notifications

**🌍 System Configuration (3 new rules)**
- Locale configuration detection
- Timezone setup verification
- NTP time synchronization enablement

### 🔄 Changed
- Detection rule count increased from 75+ to 90+ (+20%)
- Enhanced system optimization category
- Improved SSD detection logic

### 📚 Documentation
- README.md updated to v1.0.0-beta.15
- Version bumped across all crates
- CHANGELOG.md updated with beta.15 features

---

## [1.0.0-beta.14] - 2025-01-XX

### 🐳 Containers, Virtualization, Printers & More!

**Development and system tools!** Docker containerization, QEMU/KVM virtualization, printer support, archive tools, and system monitoring!

### ✨ Added

**🐳 Docker & Container Support (4 new rules)**
- Docker installation detection for container users
- Docker service enablement check
- Docker group membership for sudo-free usage
- Docker Compose for multi-container applications

**💻 Virtualization Support (QEMU/KVM) (4 new rules)**
- CPU virtualization capability detection
- BIOS virtualization enablement check (/dev/kvm)
- QEMU installation for KVM virtual machines
- virt-manager GUI for easy VM management
- libvirt service configuration

**🖨️ Printer Support (CUPS) (3 new rules)**
- USB printer detection
- CUPS printing system installation
- CUPS service enablement
- Gutenprint universal printer drivers

**📦 Archive Management Tools (3 new rules)**
- unzip for ZIP archive support
- unrar for RAR archive extraction
- p7zip for 7z archives and better compression

**📊 System Monitoring Tools (3 new rules)**
- htop for interactive process monitoring
- btop for advanced system monitoring with graphs
- iotop for disk I/O monitoring

### 🔄 Changed
- Detection rule count increased from 60+ to 75+ (+25%)
- Added development category recommendations
- Enhanced hardware support detection

### 📚 Documentation
- README.md updated to v1.0.0-beta.14
- Version bumped across all crates
- CHANGELOG.md updated with beta.14 features

---

## [1.0.0-beta.13] - 2025-01-XX

### 🌟 More Desktop Environments + SSH Hardening + Snapshots!

**New desktop environments!** Cinnamon, XFCE, and MATE now fully supported. Plus comprehensive SSH hardening and snapshot system recommendations!

### ✨ Added

**🖥️ Desktop Environment Support (3 new DEs!)**
- **Cinnamon desktop environment**
  - Nemo file manager with dual-pane view
  - GNOME Terminal integration
  - Cinnamon screensaver for security
- **XFCE desktop environment**
  - Thunar file manager with plugin support
  - xfce4-terminal with dropdown mode
  - xfce4-goodies collection (panel plugins, system monitoring)
- **MATE desktop environment**
  - Caja file manager (GNOME 2 fork)
  - MATE Terminal with tab support
  - MATE utilities (screenshot, search, disk analyzer)

**🔒 SSH Hardening Detection (7 new rules)**
- SSH Protocol 1 detection (critical vulnerability)
- X11 forwarding security check
- MaxAuthTries recommendation (brute-force protection)
- ClientAliveInterval configuration (connection timeouts)
- AllowUsers whitelist suggestion
- Non-default SSH port recommendation
- Improved root login and password authentication checks

**💾 Snapshot System Recommendations (Timeshift/Snapper)**
- Snapper detection for Btrfs users
- Timeshift detection for ext4 users
- snap-pac integration for automatic pacman snapshots
- grub-btrfs for bootable snapshot recovery
- Snapper configuration validation
- Context-aware recommendations based on filesystem type

### 🔄 Changed
- Detection rule count increased from 50+ to 60+
- README.md updated with new feature count
- "Coming Soon" section updated (implemented features removed)

### 📚 Documentation
- README.md updated to v1.0.0-beta.13
- Version bumped across all crates
- CHANGELOG.md updated with beta.13 features

---

## [1.0.0-beta.12] - 2025-01-XX

### 🎨 The Beautiful Box Update!

**Box rendering completely fixed!** Plus 50+ new detection rules, batch apply, auto-refresh, and per-user advice!

### 🔧 Fixed
- **Box rendering completely rewritten** - Fixed box drawing character alignment by using `console::measure_text_width()` to measure visible text width BEFORE adding ANSI color codes
- Terminal broadcast notifications now use proper box drawing (╭╮╰╯│─)
- All header formatting uses beautiful Unicode boxes with perfect alignment
- Tests updated to validate box structure correctly

### ✨ Added - 50+ New Detection Rules!

**🎮 Hardware Support**
- Gamepad drivers (Xbox, PlayStation, Nintendo controllers) via USB detection
- Bluetooth stack (bluez, bluez-utils) with hardware detection
- WiFi firmware for Intel, Qualcomm, Atheros, Broadcom chipsets
- USB automount with udisks2
- NetworkManager for easy WiFi management
- TLP power management for laptops (with battery detection)

**🖥️ Desktop Environments & Display**
- XWayland compatibility layer for running X11 apps on Wayland
- Picom compositor for X11 (transparency, shadows, tearing fixes)
- Modern GPU-accelerated terminals (Alacritty, Kitty, WezTerm)
- Status bars for tiling WMs (Waybar for Wayland, i3blocks for i3)
- Application launchers (Rofi for X11, Wofi for Wayland)
- Notification daemons (Dunst for X11, Mako for Wayland)
- Screenshot tools (grim/slurp for Wayland, maim/scrot for X11)

**🔤 Fonts & Rendering**
- Nerd Fonts for terminal icons and glyphs
- Emoji font support (Noto Emoji)
- CJK fonts for Chinese, Japanese, Korean text
- FreeType rendering library

**🎬 Multimedia**
- yt-dlp for downloading videos from YouTube and 1000+ sites
- FFmpeg for video/audio processing and conversion
- VLC media player for any format
- ImageMagick for command-line image editing
- GStreamer plugins for codec support in GTK apps

### 🚀 Major Features

**Batch Apply Functionality**
- Apply single recommendation: `annactl apply --nums 1`
- Apply range: `annactl apply --nums 1-5`
- Apply multiple ranges: `annactl apply --nums 1,3,5-7`
- Smart range parsing with duplicate removal and sorting
- Shows progress and summary for each item

**Per-User Context Detection**
- Added `GetAdviceWithContext` IPC method
- Personalizes advice based on:
  - Desktop environment (i3, Hyprland, Sway, GNOME, KDE, etc.)
  - Shell (bash, zsh, fish)
  - Display server (Wayland vs X11)
  - Username for multi-user systems
- CLI automatically detects and sends user environment
- Daemon filters advice appropriately

**Automatic System Monitoring**
- Daemon now automatically refreshes advice when:
  - Packages installed/removed (monitors `/var/lib/pacman/local`)
  - Config files change (pacman.conf, sshd_config, fstab)
  - System reboots (detected via `/proc/uptime`)
- Uses `notify` crate with inotify for filesystem watching
- Background task with tokio::select for event handling

**Smart Notifications**
- Critical issues trigger notifications via:
  - GUI notifications (notify-send) for desktop users
  - Terminal broadcasts (wall) for SSH/TTY users
  - Both channels for critical issues
- Uses loginctl to detect active user sessions
- Only notifies for High risk level advice

**Plain English System Reports**
- `annactl report` generates conversational health summaries
- Analyzes system state and provides friendly assessment
- Shows disk usage, package count, recommendations by category
- Provides actionable next steps

### 🔄 Changed
- **Refresh command removed from public CLI** - Now internal-only, triggered automatically by daemon
- **Advice numbering** - All items numbered for easy reference in batch apply
- **Improved text wrapping** - Multiline text wraps at 76 chars with proper indentation
- **Enhanced installer** - Auto-installs missing dependencies (curl, jq, tar)
- **Beautiful installer intro** - Shows what Anna does before installation

### 🏗️ Technical
- Added `notify` crate for filesystem watching (v6.1)
- Added `console` crate for proper text width measurement (v0.15)
- New modules: `watcher.rs` (system monitoring), `notifier.rs` (notifications)
- Enhanced `beautiful.rs` with proper box rendering using `measure_text_width()`
- `parse_number_ranges()` function for batch apply range parsing
- Better error handling across all modules
- Improved separation of concerns in recommender systems

### 📊 Statistics
- Detection rules: 27 → 50+ (85% increase)
- Advice categories: 10 → 12
- IPC methods: 8 → 9 (added GetAdviceWithContext)
- Functions for range parsing, text wrapping, user context detection
- Total code: ~3,500 → ~4,500 lines

---

## [1.0.0-beta.11] - 2025-11-04

### 🎉 The MASSIVE Feature Drop!

Anna just got SO much smarter! This is the biggest update yet with **27 intelligent detection rules** covering your entire system!

### What's New

**📦 Perfect Terminal Formatting!**
- Replaced custom box formatting with battle-tested libraries (owo-colors + console)
- Proper unicode-aware width calculation - no more broken boxes!
- All output is now gorgeous and professional

**🎮 Gaming Setup Detection!**
- **Steam gaming stack** - Multilib repo, GameMode, MangoHud, Gamescope, Lutris
- **Xbox controller drivers** - xpadneo/xone for full controller support
- **AntiMicroX** - Map gamepad buttons to keyboard/mouse
- Only triggers if you actually have Steam installed!

**🖥️ Desktop Environment Intelligence!**
- **GNOME** - Extensions, Tweaks for customization
- **KDE Plasma** - Dolphin file manager, Konsole terminal
- **i3** - i3status/polybar, Rofi launcher
- **Hyprland** - Waybar, Wofi, Mako notifications
- **Sway** - Wayland-native tools
- **XWayland** - X11 app compatibility on Wayland
- Detects your actual DE from environment variables!

**🎬 Multimedia Stack!**
- **mpv** - Powerful video player
- **yt-dlp** - Download from YouTube and 500+ sites
- **FFmpeg** - Media processing Swiss Army knife
- **PipeWire** - Modern audio system (suggests upgrade from PulseAudio)
- **pavucontrol** - GUI audio management

**💻 Terminal & Fonts!**
- **Modern terminals** - Alacritty, Kitty, WezTerm (GPU-accelerated)
- **Nerd Fonts** - Essential icons for terminal apps

**🔧 System Tools!**
- **fwupd** - Firmware updates for BIOS, SSD, USB devices
- **TLP** - Automatic laptop battery optimization (laptop detection!)
- **powertop** - Battery drain analysis

**📡 Hardware Detection!**
- **Bluetooth** - BlueZ stack + Blueman GUI (only if hardware detected)
- **WiFi** - linux-firmware + NetworkManager applet (hardware-aware)
- **USB automount** - udisks2 + udiskie for plug-and-play drives

### Why This Release is INCREDIBLE

**27 detection rules** that understand YOUR system:
- Hardware-aware (Bluetooth/WiFi only if you have the hardware)
- Context-aware (gaming tools only if you have Steam)
- Priority-based (critical firmware first, beautification optional)
- All in plain English with clear explanations!

### Technical Details
- Added `check_gaming_setup()` with Steam detection
- Added `check_desktop_environment()` with DE/WM detection
- Added `check_terminal_and_fonts()` for modern terminal stack
- Added `check_firmware_tools()` for fwupd
- Added `check_media_tools()` for multimedia apps
- Added `check_audio_system()` with PipeWire/Pulse detection
- Added `check_power_management()` with laptop detection
- Added `check_gamepad_support()` for controller drivers
- Added `check_usb_automount()` for udisks2/udiskie
- Added `check_bluetooth()` with hardware detection
- Added `check_wifi_setup()` with hardware detection
- Integrated owo-colors and console for proper formatting
- Fixed git identity message clarity

## [1.0.0-beta.10] - 2025-11-04

### ✨ The Ultimate Terminal Experience!

Anna now helps you build the most beautiful, powerful terminal setup possible!

### What's New

**🎨 Shell Enhancements Galore!**
- **Starship prompt** - Beautiful, fast prompts for zsh and bash with git status, language versions, and gorgeous colors
- **zsh-autosuggestions** - Autocomplete commands from your history as you type!
- **zsh-syntax-highlighting** - Commands turn green when valid, red when invalid - catch typos instantly
- **Smart bash → zsh upgrade** - Suggests trying zsh with clear explanations of benefits
- All context-aware based on your current shell

**🚀 Modern CLI Tools Revolution!**
- **eza replaces ls** - Colors, icons, git integration, tree views built-in
- **bat replaces cat** - Syntax highlighting, line numbers, git integration for viewing files
- **ripgrep replaces grep** - 10x-100x faster code searching with smart defaults
- **fd replaces find** - Intuitive syntax, respects .gitignore, blazing fast
- **fzf fuzzy finder** - Game-changing fuzzy search for files, history, everything!
- Smart detection - only suggests tools you actually use based on command history

**🎉 Beautiful Release Notes!**
- Install script now shows proper formatted release notes
- Colored output with emoji and hierarchy
- Parses markdown beautifully in the terminal
- Falls back to summary if API fails

**🔧 Release Automation Fixes!**
- Removed `--prerelease` flag - all releases now marked as "latest"
- Fixed installer getting stuck on beta.6
- Better jq-based JSON parsing

### Why This Release is HUGE

**16 intelligent detection rules** across security, performance, development, and beautification!

Anna can now transform your terminal from basic to breathtaking. She checks what tools you actually use and suggests modern, faster, prettier replacements - all explained in plain English.

### Technical Details
- Added `check_shell_enhancements()` with shell detection
- Added `check_cli_tools()` with command history analysis
- Enhanced install.sh with proper markdown parsing
- Fixed release.sh to mark releases as latest
- Over 240 lines of new detection code

---

## [1.0.0-beta.9] - 2025-11-04

### 🔐 Security Hardening & System Intelligence!

Anna gets even smarter with SSH security checks and memory management!

### What's New

**🛡️ SSH Hardening Detection!**
- **Checks for root login** - Warns if SSH allows direct root access (huge security risk!)
- **Password vs Key authentication** - Suggests switching to SSH keys if you have them set up
- **Empty password detection** - Critical alert if empty passwords are allowed
- Explains security implications in plain English
- All checks are Mandatory priority for your safety

**💾 Smart Swap Management!**
- **Detects missing swap** - Suggests adding swap if you have <16GB RAM
- **Zram recommendations** - Suggests compressed RAM swap for better performance
- Explains what swap is and why it matters (no more mysterious crashes!)
- Context-aware suggestions based on your RAM and current setup

**📝 Amazing Documentation!**
- **Complete README overhaul** - Now visitors will actually want to try Anna!
- Shows all features organized by category
- Includes real example messages
- Explains the philosophy and approach
- Beautiful formatting with emoji throughout

**🚀 Automated Release Notes!**
- Release script now auto-extracts notes from CHANGELOG
- GitHub releases get full, enthusiastic descriptions
- Shows preview during release process
- All past releases updated with proper notes

### Why This Release Matters
- **Security-first** - SSH hardening can prevent system compromises
- **Better stability** - Swap detection helps prevent crashes
- **Professional presentation** - README makes Anna accessible to everyone
- **14 detection rules total** - Growing smarter every release!

### Technical Details
- Added `check_ssh_config()` with sshd_config parsing
- Added `check_swap()` with RAM detection and zram suggestions
- Enhanced release.sh to extract and display CHANGELOG entries
- Updated all release notes retroactively with gh CLI
- Improved README with clear examples and philosophy

---

## [1.0.0-beta.8] - 2025-11-04

### 🚀 Major Quality of Life Improvements!

Anna just got a whole lot smarter and prettier!

### What's New

**🎨 Fixed box formatting forever!**
- Those annoying misaligned boxes on the right side? Gone! ANSI color codes are now properly handled everywhere.
- Headers, boxes, and all terminal output now look pixel-perfect.

**🔐 Security First!**
- **Firewall detection** - Anna checks if you have a firewall (UFW) and helps you set one up if you don't. Essential for security, especially on laptops!
- Anna now warns you if your firewall is installed but not turned on.

**📡 Better Networking!**
- **NetworkManager detection** - If you have WiFi but no NetworkManager, Anna will suggest installing it. Makes connecting to networks so much easier!
- Checks if NetworkManager is enabled and ready to use.

**📦 Unlock the Full Power of Arch!**
- **AUR helper recommendations** - Anna now suggests installing 'yay' or 'paru' if you don't have one. This gives you access to over 85,000 community packages!
- Explains what the AUR is in plain English - no jargon!

**⚡ Lightning-Fast Downloads!**
- **Reflector for mirror optimization** - Anna suggests installing reflector to find the fastest mirrors near you.
- Checks if your mirror list is old (30+ days) and offers to update it.
- Can make your downloads 10x faster if you're on slow mirrors!

### Why This Release Rocks
- **5 new detection rules** covering security, networking, and performance
- **Box formatting finally perfect** - no more visual glitches
- **Every message in plain English** - accessible to everyone
- **Smarter recommendations** - Anna understands your system better

### Technical Details
- Fixed ANSI escape code handling in boxed() function
- Added `check_firewall()` with UFW and iptables detection
- Added `check_network_manager()` with WiFi card detection
- Added `check_aur_helper()` suggesting yay/paru
- Added `check_reflector()` with mirror age checking
- All new features include Arch Wiki citations

---

## [1.0.0-beta.7] - 2025-11-04

### 🎉 Anna Speaks Human Now!

We've completely rewritten every message Anna shows you. No more technical jargon!

### What Changed
- **All advice is now in plain English** - Instead of "AMD CPU detected without microcode updates," Anna now says "Your AMD processor needs microcode updates to protect against security vulnerabilities like Spectre and Meltdown. Think of it like a security patch for your CPU itself."
- **Friendly messages everywhere** - "Taking a look at your system..." instead of "Analyzing system..."
- **Your system looks great!** - When everything is fine, Anna celebrates with you
- **Better counting** - "Found 1 thing that could make your system better!" reads naturally
- **Enthusiastic release notes** - This changelog is now exciting to read!

### Why This Matters
Anna is for everyone, not just Linux experts. Whether you're brand new to Arch or you've been using it for years, Anna talks to you like a helpful friend, not a robot. Every message explains *why* something matters and what it actually does.

### Technical Details (for the curious)
- Rewrote all `Advice` messages in `recommender.rs` with conversational explanations
- Updated CLI output to be more welcoming
- Made sure singular/plural grammar is always correct
- Added analogies to help explain technical concepts

---

## [1.0.0-beta.6] - 2025-11-04

### 🎉 New: Beautiful Installation Experience!
The installer now shows you exactly what Anna can do and what's new in this release. No more guessing!

### What's New
- **Your SSD will thank you** - Anna now checks if your solid-state drive has TRIM enabled. This keeps it fast and healthy for years to come.
- **Save hundreds of gigabytes** - If you're using Btrfs, Anna will suggest turning on compression. You'll get 20-30% of your disk space back without slowing things down.
- **Faster package downloads** - Anna can set up parallel downloads in pacman, making updates 5x faster. Why wait around?
- **Prettier terminal output** - Enable colorful pacman output so you can actually see what's happening during updates.
- **Health monitoring** - Anna keeps an eye on your system services and lets you know if anything failed. No more silent problems.
- **Better performance tips** - Learn about noatime and other mount options that make your system snappier.

### Why You'll Love It
- You don't need to be a Linux expert - Anna explains everything in plain English
- Every suggestion comes with a link to the Arch Wiki if you want to learn more
- Your system stays healthy and fast without you having to remember all the tweaks

---

## [1.0.0-beta.5] - 2025-11-04

### Added
- **Missing config detection** - detects installed packages without configuration:
  - bat without ~/.config/bat/config
  - starship without ~/.config/starship.toml
  - git without user.name/user.email
  - zoxide without shell integration
- Better microcode explanations (Spectre/Meltdown patches)

### Changed
- **Microcode now Mandatory priority** (was Recommended) - critical for CPU security
- Microcode category changed to "security" (was "maintenance")

### Fixed
- Box formatting now handles ANSI color codes correctly
- Header boxes dynamically size to content

---

## [1.0.0-beta.4] - 2025-11-04

### Added
- Category-based colors for advice titles (💻 blue, 🎨 pink, ⚡ yellow, 🎵 purple)
- Comprehensive FACTS_CATALOG.md documenting all telemetry to collect
- Implementation roadmap with 3 phases for v1.0.0-rc.1, v1.0.0, v1.1.0+

### Changed
- **Smarter Python detection** - requires BOTH .py files AND python/pip command usage
- **Smarter Rust detection** - requires BOTH .rs files AND cargo command usage
- Grayed out reasons and commands for better visual hierarchy
- Improved advice explanations with context

### Fixed
- False positive development tool recommendations
- Better color contrast and readability in advice output

---

## [1.0.0-beta.3] - 2025-11-04

### Added
- Emojis throughout CLI output for better visual appeal
  - 💻 Development tools, 🎨 Beautification, ⚡ Performance
  - 💡 Reasons, 📋 Commands, 🔧 Maintenance, ✨ Suggestions
- Better spacing between advice items for improved readability

### Changed
- Report command now fetches real-time data from daemon
- Improved Go language detection - only triggers on actual .go files
- Better explanations with context-aware emoji prefixes

### Fixed
- Double "v" in version string (was "vv1.0.0-beta.2", now "v1.0.0-beta.3")
- Inconsistent advice counts between report and advise commands

---

## [1.0.0-beta.2] - 2025-11-04

### Fixed
- Missing `hostname` command causing daemon crash on minimal installations
  - Added fallback to read `/etc/hostname` directly
  - Prevents "No such file or directory" error on systems without hostname utility

---

## [1.0.0-beta.1] - 2025-11-04

### 🎉 Major Release - Beta Status Achieved!

Anna is now **intelligent, personalized, and production-ready** for testing!

### Added

#### Intelligent Behavior-Based Recommendations (20+ new rules)
- **Development Tools Detection**
  - Python development → python-lsp-server, black, ipython
  - Rust development → rust-analyzer, sccache
  - JavaScript/Node.js → typescript-language-server
  - Go development → gopls language server
  - Git usage → git-delta (beautiful diffs), lazygit (TUI)
  - Docker usage → docker-compose, lazydocker
  - Vim usage → neovim upgrade suggestion

- **CLI Tool Improvements** (based on command history analysis)
  - `ls` usage → eza (colors, icons, git integration)
  - `cat` usage → bat (syntax highlighting)
  - `grep` usage → ripgrep (10x faster)
  - `find` usage → fd (modern, intuitive)
  - `du` usage → dust (visual disk usage)
  - `top/htop` usage → btop (beautiful system monitor)

- **Shell Enhancements**
  - fzf (fuzzy finder)
  - zoxide (smart directory jumping)
  - starship (beautiful cross-shell prompt)
  - zsh-autosuggestions (if using zsh)
  - zsh-syntax-highlighting (if using zsh)

- **Media Player Recommendations**
  - Video files → mpv player
  - Audio files → cmus player
  - Image files → feh viewer

#### Enhanced Telemetry System
- Command history analysis (top 1000 commands from bash/zsh history)
- Development tools detection (git, docker, vim, cargo, python, node, etc.)
- Media usage profiling (video/audio/image files and players)
- Desktop environment detection (GNOME, KDE, i3, XFCE)
- Shell detection (bash, zsh, fish)
- Display server detection (X11, Wayland)
- Package group detection (base-devel, desktop environments)
- Network interface analysis (wifi, ethernet)
- Common file type detection (.py, .rs, .js, .go, etc.)

#### New SystemFacts Fields
- `frequently_used_commands` - Top 20 commands from history
- `dev_tools_detected` - Installed development tools
- `media_usage` - Video/audio/image file presence and player status
- `common_file_types` - Programming languages detected
- `desktop_environment` - Detected DE
- `display_server` - X11 or Wayland
- `shell` - User's shell
- `has_wifi`, `has_ethernet` - Network capabilities
- `package_groups` - Detected package groups

#### Priority System
- **Mandatory**: Critical security and driver issues
- **Recommended**: Significant quality-of-life improvements
- **Optional**: Performance optimizations
- **Cosmetic**: Beautification enhancements

#### Action Executor
- Execute commands with dry-run support
- Full audit logging to `/var/log/anna/audit.jsonl`
- Rollback token generation (for future rollback capability)
- Safe command execution via tokio subprocess

#### Systemd Integration
- `annad.service` systemd unit file
- Automatic startup on boot
- Automatic restart on failure
- Install script enables/starts service automatically

#### Documentation
- `ROADMAP.md` - Project vision and implementation plan
- `TESTING.md` - Testing guide for IPC system
- `CHANGELOG.md` - This file

### Changed
- **Advice struct** now includes:
  - `priority` field (Mandatory/Recommended/Optional/Cosmetic)
  - `category` field (security/drivers/development/media/beautification/etc.)
- Install script now installs and enables systemd service
- Daemon logs more detailed startup information
- Recommendations now sorted by priority

### Fixed
- Install script "Text file busy" error when daemon is running
- Version embedding in GitHub Actions workflow
- Socket permission issues for non-root users

---

## [1.0.0-alpha.3] - 2024-11-03

### Added
- Unix socket IPC between daemon and client
- RPC protocol with Request/Response message types
- Real-time communication for status and recommendations
- Version verification in install script

### Fixed
- GitHub Actions release workflow permissions
- Install script process stopping logic

---

## [1.0.0-alpha.2] - 2024-11-02

### Added
- Release automation scripts (`scripts/release.sh`)
- Install script (`scripts/install.sh`) for GitHub releases
- GitHub Actions workflow for releases
- Version embedding via build.rs

---

## [1.0.0-alpha.1] - 2024-11-01

### Added
- Initial project structure
- Core data models (SystemFacts, Advice, Action, etc.)
- Basic telemetry collection (hardware, packages)
- 5 initial recommendation rules:
  - Microcode installation (AMD/Intel)
  - GPU driver detection (NVIDIA/AMD)
  - Orphaned packages cleanup
  - Btrfs maintenance
  - System updates
- Beautiful CLI with pastel colors
- Basic daemon and client binaries

---

## Future Plans

### v1.0.0-rc.1 (Release Candidate)
- Arch Wiki caching system
- Wiki-grounded recommendations with citations
- More recommendation rules (30+ total)
- Configuration persistence
- Periodic telemetry refresh

### v1.0.0 (Stable Release)
- Autonomous execution tiers (0-3)
- Auto-apply safe recommendations
- Rollback capability
- Performance optimizations
- Comprehensive documentation

### v1.1.0+
- AUR package
- Web dashboard
- Multi-user support
- Plugin system
- Machine learning for better predictions

## [1.0.0-beta.21] - 2025-01-XX

### 🎛️ Configuration System - TOML-based Settings! 🎛️

**MAJOR NEW FEATURE!** Implemented comprehensive configuration system with TOML support for user preferences and automation!

### ✨ Added

**Configuration Module**
- Created `config.rs` in anna_common with full TOML serialization/deserialization
- Configuration file automatically created at `~/.config/anna/config.toml`
- Structured configuration with multiple sections:
  - General settings (refresh interval, verbosity, emoji, colors)
  - Autonomy configuration (tier levels, auto-apply rules, risk filtering)
  - Notification preferences (desktop, terminal, priority filtering)
  - Snapshot settings (method, retention, auto-snapshot triggers)
  - Learning preferences (behavior tracking, history analysis)
  - Category filters (enable/disable recommendation categories)
  - User profiles (multi-user system support)

**Enhanced annactl config Command**
- Display all current configuration settings beautifully organized
- Set individual config values: `annactl config --set key=value`
- Supported configuration keys:
  - `autonomy_tier` (0-3): Control auto-apply behavior
  - `snapshots_enabled` (true/false): Enable/disable snapshots
  - `snapshot_method` (btrfs/timeshift/rsync/none): Choose snapshot backend
  - `learning_enabled` (true/false): Enable/disable behavior learning
  - `desktop_notifications` (true/false): Control notifications
  - `refresh_interval` (seconds): Set telemetry refresh frequency
- Validation on all settings with helpful error messages
- Beautiful output showing all configuration sections

**Configuration Features**
- Autonomy tiers: Advise Only, Safe Auto-Apply, Semi-Autonomous, Fully Autonomous
- Risk-based filtering for auto-apply
- Category-based allow/blocklists
- Snapshot integration planning (method selection, retention policies)
- Learning system configuration (command history days, usage thresholds)
- Notification customization (urgency levels, event filtering)
- Multi-user profiles for personalized recommendations

### 🔧 Changed
- Added `toml` dependency to workspace
- Updated anna_common to export config module
- Enhanced config command from stub to fully functional

### 📚 Technical Details
- Config validation ensures safe values (min 60s refresh, min 1 snapshot, etc.)
- Default configuration provides sensible security-first defaults
- TOML format allows easy manual editing
- Auto-creates config directory structure on first use

This lays the foundation for the TUI dashboard and autonomous operation!


## [1.0.0-beta.22] - 2025-01-XX

### 📸 Snapshot & Rollback System - Safe Execution! 📸

**MAJOR NEW FEATURE!** Implemented comprehensive snapshot management for safe action execution with rollback capability!

### ✨ Added

**Snapshot Manager Module**
- Created `snapshotter.rs` with multi-backend support
- Three snapshot methods supported:
  - **Btrfs**: Native subvolume snapshots (read-only, instant)
  - **Timeshift**: Integration with popular backup tool
  - **Rsync**: Incremental backups of critical directories
- Automatic snapshot creation before risky operations
- Configurable risk-level triggers (Medium/High by default)
- Snapshot retention policies with automatic cleanup
- Snapshot metadata tracking (ID, timestamp, description, size)

**Enhanced Executor**
- `execute_action_with_snapshot()`: New function with snapshot support
- Automatic snapshot creation based on risk level
- Rollback token generation with snapshot IDs
- Graceful degradation if snapshot fails (warns but proceeds)
- Backward compatibility maintained for existing code

**Snapshot Features**
- List all snapshots with metadata
- Automatic cleanup of old snapshots (configurable max count)
- Size tracking for disk space management
- Timestamp-based naming scheme
- Support for custom descriptions

**Safety Features**
- Snapshots created BEFORE executing risky commands
- Risk-based triggers (Low/Medium/High)
- Category-based blocking (bootloader, kernel blocked by default)
- Read-only Btrfs snapshots prevent accidental modification
- Metadata preservation for audit trails

### 🔧 Configuration Integration
- Snapshot settings in config.toml:
  - `snapshots.enabled` - Enable/disable snapshots
  - `snapshots.method` - Choose backend (btrfs/timeshift/rsync)
  - `snapshots.max_snapshots` - Retention count
  - `snapshots.snapshot_risk_levels` - Which risks trigger snapshots
  - `snapshots.auto_snapshot_on_risk` - Auto-snapshot toggle

### 📚 Technical Details
- Async snapshot creation with tokio
- Proper error handling and logging
- Filesystem type detection for Btrfs
- Directory size calculation with `du`
- Graceful handling of missing tools (timeshift, etc.)

This provides the foundation for safe autonomous operation and rollback capability!


## [1.0.0-beta.23] - 2025-01-XX

### 🔍 Enhanced Telemetry - Deep System Intelligence! 🔍

**MAJOR ENHANCEMENT!** Added comprehensive system analysis from a sysadmin perspective with CPU time tracking, deep bash history analysis, and system configuration insights!

### ✨ Added

**Process CPU Time Analysis**
- Track actual CPU time per process for user behavior understanding
- Filter user processes vs system processes
- CPU and memory percentage tracking
- Identify what users actually spend time doing

**Deep Bash History Analysis**
- Multi-user bash/zsh history parsing
- Command frequency analysis across all users
- Tool categorization (editor, vcs, container, development, etc.)
- Workflow pattern detection with confidence scores
- Detect: Version Control Heavy, Container Development, Software Development patterns
- Evidence-based pattern matching

**System Configuration Analysis** (sysadmin perspective)
- Bootloader detection (GRUB, systemd-boot, rEFInd)
- Init system verification
- Failed systemd services detection
- Firewall status (ufw/firewalld)
- MAC system detection (SELinux/AppArmor)
- Swap analysis (size, usage, swappiness, zswap)
- Boot time analysis (systemd-analyze)
- I/O scheduler per device
- Important kernel parameters tracking

**Swap Deep Dive**
- Total/used swap in MB
- Swappiness value
- Zswap detection and status
- Recommendations based on swap configuration

**I/O Scheduler Analysis**
- Per-device scheduler detection
- Identify if using optimal schedulers for SSD/HDD
- Foundation for SSD optimization recommendations

**Kernel Parameter Tracking**
- Command line parameters
- Important sysctl values (swappiness, ip_forward, etc.)
- Security and performance parameter analysis

### 🔧 Technical Details
- All analysis functions are async for performance
- Processes are filtered by CPU time (>0.1%)
- Bash history supports both bash and zsh formats
- Workflow patterns calculated with confidence scores (0.0-1.0)
- System config analysis covers bootloader, init, security, performance
- Graceful handling of missing files/permissions

This provides the foundation for truly intelligent, sysadmin-level system analysis!


## [1.0.0-beta.24] - 2025-01-XX

### 🎨 Beautiful Category-Based Advise Output! 🎨

**MAJOR UX IMPROVEMENT!** Completely redesigned `annactl advise` output with category boxes, priority badges, risk badges, and visual hierarchy!

### ✨ Added

**Category-Based Organization**
- Recommendations grouped by category with beautiful boxes
- 14 predefined categories sorted by importance:
  - Security, Drivers, Updates, Maintenance, Cleanup
  - Performance, Power, Development, Desktop, Gaming
  - Multimedia, Hardware, Networking, Beautification
- Each category gets unique emoji and color
- Automatic fallback for unlisted categories

**Beautiful Category Headers**
- 80-character wide boxes with centered titles
- Category-specific emojis (🔒 Security, ⚡ Performance, 💻 Development, etc.)
- Color-coded titles (red for security, yellow for performance, etc.)
- Proper spacing between categories for easy scanning

**Enhanced Item Display**
- Priority badges: CRITICAL, RECOMMENDED, OPTIONAL, COSMETIC
- Risk badges: HIGH RISK, MED RISK, LOW RISK
- Color-coded backgrounds (red, yellow, green, blue)
- Bold titles for quick scanning
- Wrapped text with proper indentation (72 chars)
- Actions highlighted with ❯ symbol
- ID shown subtly in italics

**Smart Sorting**
- Categories sorted by importance (security first)
- Within each category: sort by priority, then risk
- Highest priority and risk items shown first

**Better Summary**
- Shows total recommendations and category count
- Usage instructions at bottom
- Visual separator with double-line (═)

**Fixed Issues**
- RiskLevel now implements Ord for proper sorting
- Box titles properly padded and centered
- All ANSI codes use proper escapes
- Consistent spacing throughout

This makes long advice lists MUCH easier to scan and understand!

