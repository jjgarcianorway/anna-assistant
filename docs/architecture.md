# Architecture

- assistantd: daemon that ingests signals, proposes plans, and applies approved actions.
- assistantctl: CLI to review history and reports.
- Skills: signed YAML with precheck, actions, postcheck, rollback.
- Policy: controls autonomy per category.
- Audit: all changes are committed to a local git repo with diffs and revert steps.
