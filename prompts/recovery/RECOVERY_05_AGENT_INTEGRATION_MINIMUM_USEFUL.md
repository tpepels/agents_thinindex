# RECOVERY_05_AGENT_INTEGRATION_MINIMUM_USEFUL.md

Use superpowers:subagent-driven-development.

Do not implement this until RECOVERY_04_VALUE_WORKFLOWS.md is complete and green.

Goal:
Make the minimum agent integration useful across common agent surfaces without adding MCP or large integration architecture.

Scope:
Instruction surfaces and safe local helpers only. Do not add MCP, packaging, licensing, payment, cloud, telemetry, or hosted features.

Current failure it addresses:
Agents are not reliably using thinindex. AGENTS.md alone may be too weak or too vague.

Phases:
- [x] Audit existing AGENTS.md generation.
- [x] Audit existing CLAUDE.md normalization behavior.
- [x] Add or update Cursor rule generation if in scope.
- [x] Add or update GitHub Copilot instruction generation if in scope.
- [x] Add OpenCode guidance through AGENTS.md.
- [x] Add Codex guidance/config snippet docs.
- [x] Ensure all wording matches actual auto-rebuild behavior.
- [x] Add idempotency tests.
- [x] Run verification.
- [x] Commit.

Instruction targets:
- AGENTS.md
- existing CLAUDE.md only if present; do not create it
- .cursor/rules/thinindex.mdc if implemented
- .github/copilot-instructions.md if implemented
- docs/AGENT_INTEGRATION.md
- Codex config snippet docs, dry-run only if implemented

Required wording:
- wi replaces blind grep/find/ls/Read for repository discovery.
- agents should run wi <term> directly.
- stale/missing index self-heals.
- use wi refs before broad reference searches.
- use wi pack before implementation.
- use wi impact before edits.
- use wi --help for command details.

Hard requirements:
- Do not reintroduce WI.md.
- Do not create CLAUDE.md if absent.
- Repeated runs normalize, not duplicate.
- Global config is not modified by default.
- No network calls.
- No telemetry.

Tests:
- AGENTS.md generated if absent.
- AGENTS.md normalized if present.
- CLAUDE.md normalized if present.
- CLAUDE.md absent remains absent.
- repeated init does not duplicate blocks.
- stale WI.md references removed.
- generated wording includes auto-rebuild behavior.
- optional Cursor/Copilot files are idempotent if implemented.

Acceptance:
- minimum agent instruction surfaces are useful.
- instructions are direct and current.
- behavior and docs match.
- no MCP required.
- no duplicate/stale instruction blocks.

Verification:
- cargo fmt --check
- cargo test
- cargo clippy --all-targets --all-features -- -D warnings
- cargo run --bin wi-init -- --help
- representative init/helper dry-run if added
- cargo run --bin wi-stats

Commit:
Add minimum useful agent integration
