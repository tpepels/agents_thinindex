# PLAN_64_MCP_INTEGRATION_DECISION.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_63_SEMANTIC_FACT_USER_VALUE_DECISION.md is complete and green.

Goal:
Decide whether to implement, defer, or explicitly reject MCP integration for thinindex.

Context:
The feature wiring audit found MCP is documented-only: there is no MCP server/helper bundled. Agent integration currently relies on repo-local instruction surfaces and advisory stats.

Scope:
Decision and minimal implementation only. Do not add hosted services, telemetry, payment/licensing enforcement, package-manager execution, parser architecture changes, ctags production use, JSONL canonical storage, or `WI.md`.

Product rule:
MCP should exist only if it improves agent use without compromising local-only safety, bounded output, or CLI performance.

Decision options:
- defer MCP and clean docs
- implement config snippets only
- implement a minimal local-only MCP server/helper
- explicitly reject MCP for now

Phases:
- [x] Inspect current MCP docs and agent integration docs.
- [x] Inspect current agent helper surfaces.
- [x] Decide whether MCP is needed now.
- [x] If deferring, make docs explicit and remove any wording implying MCP is implemented.
- [x] If implementing, keep it minimal, local-only, bounded, and safe. (MCP not implemented; explicitly deferred.)
- [x] Add tests for whichever path is chosen.
- [x] Run verification.
- [x] Commit.

Decision:
Defer MCP for this release. Current agent integration remains repo-local
instruction surfaces plus `wi-init --dry-run` and advisory `wi-stats`.
thinindex does not bundle an MCP server, MCP helper command, or MCP client
configuration helper. A future MCP plan must be local-only, bounded, path
validated, stale-index compatible with `wi`, and must not add arbitrary shell
execution, network access, telemetry, hosted behavior, or normal quality/
comparator/real-repo workflows.

If MCP is implemented:
- local-only
- optional
- no arbitrary shell execution
- repo path validation
- bounded outputs
- stale index behavior matches CLI
- no quality/comparator/real-repo workflows in normal search calls
- no external MCP client required for normal tests

If MCP is deferred:
- docs must say no MCP server/helper is bundled.
- docs may include future design notes only if clearly marked future.
- no config helper should produce a non-existent command.

Tests:
- docs do not overclaim MCP.
- helper dry-runs are deterministic if helpers exist.
- MCP handler tests if MCP is implemented.
- normal CLI behavior remains unchanged.

Acceptance:
- MCP status is unambiguous.
- docs match implementation.
- no unsafe global or network behavior is introduced.
- agent integration claims remain honest.

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run --bin wi-init -- --help`
- `cargo run --bin wi -- pack build_index`
- MCP tests or docs checks if added

Commit:
Decide MCP integration path
