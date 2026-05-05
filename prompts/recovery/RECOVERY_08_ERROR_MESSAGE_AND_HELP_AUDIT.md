# RECOVERY_08_ERROR_MESSAGE_AND_HELP_AUDIT.md

Use superpowers:subagent-driven-development.

Do not implement this until RECOVERY_07_PRODUCT_VALUE_SCORECARD.md is complete and green.

Goal:
Audit and repair thinindex error messages, help text, and next-step guidance.

Scope:
CLI UX only. Do not add parser architecture, packaging, licensing, payment, cloud, telemetry, or MCP work.

Current failure it addresses:
The tool told the user to manually run build_index when it should have auto-rebuilt. Help/error text must match actual behavior.

Phases:
- [ ] Inventory user-facing messages for wi, build_index, wi doctor, wi-init, and wi-stats.
- [ ] Remove stale guidance that contradicts auto-rebuild behavior.
- [ ] Ensure every failure has a next action.
- [ ] Ensure help text mentions current commands and avoids removed concepts.
- [ ] Add tests for important messages.
- [ ] Run verification.
- [ ] Commit.

Required checks:
- no WI.md guidance
- no JSONL canonical-storage guidance
- no ctags install requirement
- no stale "run build_index manually" guidance for ordinary stale index recovery
- clear message when auto-rebuild fails
- clear message when repo root cannot be found
- clear message when index schema is invalid
- clear message when config/instruction surfaces are stale

Tests:
- missing index message
- stale index auto-rebuild message
- failed auto-rebuild message
- doctor current/stale/missing states
- help includes current commands
- help excludes removed concepts

Acceptance:
- core CLI messages are accurate.
- users/agents get direct next actions.
- help text and behavior agree.
- tests cover the highest-risk messages.

Verification:
- cargo fmt --check
- cargo test
- cargo clippy --all-targets --all-features -- -D warnings
- cargo run --bin wi -- --help
- cargo run --bin wi-init -- --help
- cargo run --bin wi-stats -- --help
- cargo run --bin build_index

Commit:
Align CLI help and error messages
