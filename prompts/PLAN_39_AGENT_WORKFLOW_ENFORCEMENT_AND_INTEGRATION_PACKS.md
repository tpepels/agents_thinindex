# PLAN_39_AGENT_WORKFLOW_ENFORCEMENT_AND_INTEGRATION_PACKS.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_38_OPTIONAL_SEMANTIC_ADAPTER_BOUNDARY.md is complete and green.

Progress:
- [x] Phase 1: inspect existing wi-init, wi-stats, usage logging, and instruction generation
- [x] Phase 2: add local usage command categories and agent workflow audit summary
- [x] Phase 3: add isolated Codex, Claude, generic, and MCP integration packs
- [x] Phase 4: add tests for audit output, local-only claims, and instruction invariants
- [x] Phase 5: run required verification and commit completed Plan 39 work

Goal:
Improve agent compliance with thinindex by adding integration packs, wrappers, and workflow checks.

Product rule:
Text instructions are weak. Add practical enforcement and audit surfaces where possible.

Required:
- Keep AGENTS.md and existing CLAUDE.md generation correct.
- Add optional agent integration packs under isolated paths.
- Add read-budget/search-behavior guidance.
- Add command wrappers or scripts only if they do not break normal CLI use.
- Add audit report for whether an agent used `wi` before broad search, if possible from available logs/usage events.
- Do not require proprietary agent APIs.
- Do not add telemetry/network/cloud behavior.

Integration targets:
- Codex CLI guidance/config examples
- Claude guidance/config examples
- generic agent instructions
- optional MCP/tool integration plan if feasible, but do not require it

Possible features:
- `wi-stats` report: agent workflow compliance summary
- usage event categories for query/ref/pack/impact
- local-only audit report
- prompt snippets for agents

Tests:
- generated AGENTS.md includes direct Repository search block
- existing CLAUDE.md normalized if present
- no WI.md reintroduced
- usage stats remain local
- integration docs do not claim hard enforcement if only advisory

Docs:
Add:
- docs/AGENT_INTEGRATION.md
- examples for Codex/Claude/manual agents
- guidance for using `wi pack` and `wi impact`

Acceptance:
- agent integration docs/packs exist
- generated instructions are direct and current
- compliance/audit reporting exists where practical
- no network/telemetry behavior introduced

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run --bin wi-init -- --help`
- `cargo run --bin wi-stats`
- `cargo run --bin wi -- pack build_index`

Report:
- changed files
- integration packs added
- enforcement/audit behavior
- verification results
- commit hash
