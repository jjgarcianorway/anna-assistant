[0;34m
╔═══════════════════════════════════════╗
║                                       ║
║    ANNA QA TEST HARNESS               ║
║    Sprint 3 Validation Suite          ║
║                                       ║
╚═══════════════════════════════════════╝
[0m

[0;34m━━━ Project Structure ━━━[0m
TEST: Found Cargo.toml [0;32m… PASS[0m
TEST: Found src/annad/Cargo.toml [0;32m… PASS[0m
TEST: Found src/annactl/Cargo.toml [0;32m… PASS[0m
TEST: Found scripts/install.sh [0;32m… PASS[0m
TEST: Found scripts/uninstall.sh [0;32m… PASS[0m
TEST: Found etc/systemd/annad.service [0;32m… PASS[0m
TEST: Found config/default.toml [0;32m… PASS[0m
TEST: Found polkit/com.anna.policy [0;32m… PASS[0m
TEST: Found completion/annactl.bash [0;32m… PASS[0m
TEST: Found DESIGN-NOTE-privilege-model.md [0;32m… PASS[0m

[0;34m━━━ Compilation ━━━[0m
[1;33mℹ[0m Running cargo check...
TEST: cargo check succeeded [0;32m… PASS[0m
[1;33mℹ[0m Building release binaries...
TEST: Release build succeeded [0;32m… PASS[0m
TEST: annad binary exists [0;32m… PASS[0m
TEST: annactl binary exists [0;32m… PASS[0m

[0;34m━━━ Binary Smoke Tests ━━━[0m
TEST: annactl --help works [0;32m… PASS[0m
TEST: annactl --version works [0;32m… PASS[0m

[0;34m━━━ Configuration ━━━[0m
TEST: Default config exists [0;32m… PASS[0m
TEST: Config structure valid (all sections present) [0;32m… PASS[0m
TEST: autonomy.level key present [0;32m… PASS[0m
TEST: telemetry.local_store key present [0;32m… PASS[0m
TEST: shell.integrations.autocomplete key present [0;32m… PASS[0m

[0;34m━━━ Installation Scripts ━━━[0m
TEST: install.sh is executable [0;32m… PASS[0m
TEST: uninstall.sh is executable [0;32m… PASS[0m
TEST: install.sh syntax valid [0;32m… PASS[0m
TEST: uninstall.sh syntax valid [0;32m… PASS[0m
TEST: install.sh includes Sprint 1 features [0;32m… PASS[0m
TEST: uninstall.sh creates backup README [0;32m… PASS[0m

[0;34m━━━ Systemd Service ━━━[0m
TEST: Service file exists [0;32m… PASS[0m
TEST: Service ExecStart correct [0;32m… PASS[0m
TEST: Service type is simple [0;32m… PASS[0m

[0;34m━━━ Polkit Policy ━━━[0m
TEST: Polkit policy file exists [0;32m… PASS[0m
TEST: Polkit actions defined correctly [0;32m… PASS[0m
TEST: Polkit policy XML valid [0;32m… PASS[0m

[0;34m━━━ Bash Completion ━━━[0m
TEST: Bash completion file exists [0;32m… PASS[0m
TEST: Bash completion syntax valid [0;32m… PASS[0m
TEST: Bash completion function defined [0;32m… PASS[0m

[0;34m━━━ Privilege Separation ━━━[0m
TEST: annactl runs without root privileges [0;32m… PASS[0m
TEST: annad enforces root requirement in code [0;32m… PASS[0m
TEST: Polkit module exists [0;32m… PASS[0m

[0;34m━━━ Config Operations ━━━[0m
TEST: Config module has get/set/list functions [0;32m… PASS[0m
TEST: RPC handlers for config operations exist [0;32m… PASS[0m
TEST: annactl has config subcommands [0;32m… PASS[0m

[0;34m━━━ Doctor Checks ━━━[0m
TEST: Doctor check: check_daemon_active implemented [0;32m… PASS[0m
TEST: Doctor check: check_socket_ready implemented [0;32m… PASS[0m
TEST: Doctor check: check_polkit_policies implemented [0;32m… PASS[0m
TEST: Doctor check: check_paths_writable implemented [0;32m… PASS[0m
TEST: Doctor check: check_autocomplete_installed implemented [0;32m… PASS[0m
TEST: Doctor checks include fix hints [0;32m… PASS[0m
TEST: annactl doctor exits non-zero on failure [0;32m… PASS[0m

[0;34m━━━ Telemetry ━━━[0m
TEST: Telemetry module exists [0;32m… PASS[0m
TEST: Telemetry has required event types [0;32m… PASS[0m
TEST: Telemetry implements rotation [0;32m… PASS[0m
TEST: Telemetry is local-only (no network code) [0;32m… PASS[0m

[0;34m━━━ Documentation ━━━[0m
TEST: Privilege model design note exists [0;32m… PASS[0m
TEST: README.md exists [0;32m… PASS[0m
TEST: README has quickstart section [0;32m… PASS[0m
TEST: GENESIS.md exists [0;32m… PASS[0m

[0;34m━━━ Sprint 2: Autonomy Framework ━━━[0m
TEST: Autonomy module exists [0;32m… PASS[0m
TEST: Autonomy levels defined (Off, Low, Safe) [0;32m… PASS[0m
TEST: Autonomy tasks defined (Doctor, TelemetryCleanup, ConfigSync) [0;32m… PASS[0m
TEST: Autonomy API functions present [0;32m… PASS[0m
TEST: Autonomy RPC handlers present [0;32m… PASS[0m
TEST: annactl autonomy commands present [0;32m… PASS[0m

[0;34m━━━ Sprint 2: Persistence Layer ━━━[0m
TEST: Persistence module exists [0;32m… PASS[0m
TEST: Persistence API functions present [0;32m… PASS[0m
TEST: Persistence includes state rotation logic [0;32m… PASS[0m
TEST: Persistence RPC handlers present [0;32m… PASS[0m
TEST: annactl state commands present [0;32m… PASS[0m
TEST: Persistence initialized in daemon main [0;32m… PASS[0m

[0;34m━━━ Sprint 2: Auto-Fix Mechanism ━━━[0m
TEST: Auto-fix function exists [0;32m… PASS[0m
TEST: AutoFixResult type defined [0;32m… PASS[0m
TEST: Auto-fix function: autofix_socket_directory implemented [0;32m… PASS[0m
TEST: Auto-fix function: autofix_paths implemented [0;32m… PASS[0m
TEST: Auto-fix function: autofix_config_directory implemented [0;32m… PASS[0m
TEST: Auto-fix RPC handler present [0;32m… PASS[0m
TEST: annactl doctor --autofix flag present [0;32m… PASS[0m
TEST: Auto-fix attempts logged to telemetry [0;32m… PASS[0m

[0;34m━━━ Sprint 2: Telemetry Commands ━━━[0m
TEST: annactl telemetry subcommand exists [0;32m… PASS[0m
TEST: Telemetry actions (list, stats) defined [0;32m… PASS[0m
TEST: Telemetry print functions present [0;32m… PASS[0m
TEST: Telemetry list has limit parameter [0;32m… PASS[0m
TEST: Telemetry reads from system and user paths [0;32m… PASS[0m

[0;34m━━━ Sprint 2: State Directory Structure ━━━[0m
TEST: STATE_DIR constant defined [0;32m… PASS[0m
TEST: State directory path correct [0;32m… PASS[0m
TEST: Persistence init creates directories [0;32m… PASS[0m

[0;34m━━━ Sprint 2: Integration Validation ━━━[0m
TEST: Sprint 2 modules declared in daemon main [0;32m… PASS[0m
TEST: RPC imports Sprint 2 modules [0;32m… PASS[0m
TEST: Telemetry rotation accessible from autonomy [0;32m… PASS[0m
TEST: dirs dependency added to annactl [0;32m… PASS[0m

[0;34m━━━ Sprint 3: Policy Engine ━━━[0m
TEST: Policy module exists [0;32m… PASS[0m
TEST: PolicyEngine struct defined [0;32m… PASS[0m
TEST: Policy types (Rule, Action) defined [0;32m… PASS[0m
TEST: Policy evaluation functions present [0;32m… PASS[0m
TEST: Condition parsing implemented [0;32m… PASS[0m
TEST: PolicyContext struct defined [0;32m… PASS[0m
TEST: Policy actions defined (DisableAutonomy, RunDoctor, SendAlert) [0;32m… PASS[0m
TEST: serde_yaml dependency added [0;32m… PASS[0m

[0;34m━━━ Sprint 3: Event Reaction System ━━━[0m
TEST: Events module exists [0;32m… PASS[0m
TEST: Event struct defined [0;32m… PASS[0m
TEST: EventDispatcher struct defined [0;32m… PASS[0m
TEST: Event types defined (TelemetryAlert, ConfigChange, DoctorResult) [0;32m… PASS[0m
TEST: Event severity levels defined [0;32m… PASS[0m
TEST: Event dispatch function present [0;32m… PASS[0m
TEST: Event filtering functions present [0;32m… PASS[0m
TEST: EventReactor struct defined [0;32m… PASS[0m
TEST: uuid dependency added [0;32m… PASS[0m

[0;34m━━━ Sprint 3: Learning Cache ━━━[0m
TEST: Learning module exists [0;32m… PASS[0m
TEST: LearningCache struct defined [0;32m… PASS[0m
TEST: ActionStats struct defined [0;32m… PASS[0m
TEST: Outcome enum defined (Success, Failure) [0;32m… PASS[0m
TEST: record_action function present [0;32m… PASS[0m
TEST: Recommendation functions present [0;32m… PASS[0m
TEST: Learning persistence functions present [0;32m… PASS[0m
TEST: LearningAnalytics struct defined [0;32m… PASS[0m

[0;34m━━━ Sprint 3: CLI Commands ━━━[0m
TEST: annactl policy subcommand exists [0;32m… PASS[0m
TEST: Policy actions (list, reload, eval) defined [0;32m… PASS[0m
TEST: annactl events subcommand exists [0;32m… PASS[0m
TEST: Event actions (show, list, clear) defined [0;32m… PASS[0m
TEST: annactl learning subcommand exists [0;32m… PASS[0m
TEST: Learning actions (stats, recommendations, reset) defined [0;32m… PASS[0m
TEST: Sprint 3 print functions present [0;32m… PASS[0m

[0;34m━━━ Sprint 3: RPC Handlers ━━━[0m
TEST: Policy RPC handlers present [0;32m… PASS[0m
TEST: Events RPC handlers present [0;32m… PASS[0m
TEST: Learning RPC handlers present [0;32m… PASS[0m
TEST: RPC imports Sprint 3 modules [0;32m… PASS[0m

[0;34m━━━ Sprint 3: Policy Files ━━━[0m
TEST: Example telemetry policy exists [0;32m… PASS[0m
TEST: Example system policy exists [0;32m… PASS[0m
TEST: Policy documentation exists [0;32m… PASS[0m
TEST: Telemetry policy YAML valid [0;32m… PASS[0m
TEST: System policy YAML valid [0;32m… PASS[0m

[0;34m━━━ Sprint 3: Integration Validation ━━━[0m
TEST: Sprint 3 modules declared in daemon main [0;32m… PASS[0m
TEST: tempfile dependency added (for tests) [0;32m… PASS[0m
TEST: Events module links to PolicyEngine [0;32m… PASS[0m
TEST: Learning cache path configured correctly [0;32m… PASS[0m

[0;34m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━[0m

  Tests Run:  134
  [0;32mPassed:     134[0m
  [0;31mFailed:     0[0m
  [1;33mSkipped:    0[0m
  Duration:   1s

[0;32m✅ All tests passed (134 total)[0m

