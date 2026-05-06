# PLAN_59_AGENT_INTEGRATION_HELPERS_AND_MCP_DECISION.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_58_GO_PHP_REAL_REPO_SUPPORT_EVIDENCE.md is complete and green.

Goal:
Decide and implement the next minimum useful agent integration layer: config helpers, MCP, or explicit deferral.

Scope:
Agent integration only. Keep it local and bounded.

Do not add:
- hosted services
- telemetry
- payment/licensing enforcement
- package-manager execution
- parser architecture changes
- ctags production use
- JSONL canonical storage
- `WI.md`

Context:
The feature-gap audit says MCP is documented-only, OpenCode uses shared AGENTS.md guidance, and agent usage remains advisory. If agents still ignore `wi`, tool-level integration may be needed.

Product rule:
Agent integration should reduce reliance on vague instructions, but must not compromise local-only safety or CLI performance.

Phases:
- [x] Audit current agent surfaces: AGENTS, CLAUDE, Cursor, Copilot, OpenCode, Codex docs.
- [x] Identify the minimum next useful integration.
- [x] Decide among:
  - config helpers only
  - MCP server/helper
  - both
  - explicit deferral
- [x] If implementing helpers, keep them dry-run/safe by default.
- [x] If implementing MCP, keep it local-only, bounded, and non-invasive. (MCP not implemented; explicitly deferred.)
- [x] Add tests.
- [x] Update docs.
- [x] Run verification.
- [x] Commit.

Decision:
Implement config-helper dry-runs for `wi-init` and explicitly defer MCP. The
existing useful integration layer is repo-local instruction generation for
AGENTS, CLAUDE normalization, Cursor, and Copilot. MCP remains unimplemented
because no MCP handler/server exists yet, and adding one would be broader than
the minimum local helper hardening needed here.

Agent targets:
- Codex
- Claude
- Cursor
- GitHub Copilot
- OpenCode
- generic agents

Config helper requirements if implemented:
- no global config writes by default
- print/dry-run mode first
- repo-local files are idempotent
- Linux paths are correct
- XDG-aware where practical
- no network/telemetry

MCP requirements if implemented:
- optional only
- local-only
- no arbitrary shell execution
- repo path validation
- bounded outputs
- stale index behavior matches CLI
- no quality/comparator/real-repo workflows in normal search calls
- no external MCP client required for normal tests

If deferring MCP:
- docs must clearly say it is not implemented.
- include exact blocker/reason.
- do not leave docs implying it exists.

Tests:
- generated instruction surfaces remain idempotent.
- helper dry-runs are deterministic.
- global config is not modified by default.
- MCP handler tests if MCP is implemented.
- docs do not overclaim MCP/OpenCode-specific support.

Docs:
Update:
- `docs/AGENT_INTEGRATION.md`
- integration README files
- `wi-init --help` if commands change

Acceptance:
- next agent integration step is implemented or explicitly deferred.
- docs match reality.
- no unsafe global mutation by default.
- no network/telemetry behavior.
- core CLI performance remains unaffected.

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run --bin wi-init -- --help`
- `cargo run --bin wi-stats`
- representative helper dry-runs if implemented
- MCP handler tests if implemented
- `cargo run --bin build_index`
- `cargo run --bin wi -- pack build_index`

Commit:
Decide agent integration helper path
