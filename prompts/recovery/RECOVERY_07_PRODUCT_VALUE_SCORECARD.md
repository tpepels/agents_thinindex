# RECOVERY_07_PRODUCT_VALUE_SCORECARD.md

Use superpowers:subagent-driven-development.

Do not implement this until RECOVERY_06_RELEASE_DECISION_AUDIT.md is complete and green.

Goal:
Create a product value scorecard that measures whether thinindex is actually useful for humans and agents.

Scope:
This is measurement and small alignment fixes only. Do not add major architecture, packaging, licensing, payment, cloud, telemetry, or MCP work.

Current failure it addresses:
The project has many technical plans, but not enough evidence that the core product loop creates value.

Phases:
- [ ] Define scorecard dimensions.
- [ ] Add fixture or scripted checks for each dimension where practical.
- [ ] Add a local report command/script/test if useful.
- [ ] Document how to interpret the scorecard.
- [ ] Fix obvious wording/reporting issues found while implementing the scorecard.
- [ ] Run verification.
- [ ] Commit.

Scorecard dimensions:
- wi <term> gives useful file:line results.
- stale/missing index auto-recovers.
- warm query latency is acceptable.
- wi refs <term> gives useful references.
- wi pack <term> gives a bounded useful read set.
- wi impact <term> gives plausible affected files with reasons.
- wi doctor gives actionable state.
- wi-init creates useful agent instructions.
- generated instructions match actual behavior.
- unsupported/experimental parser support is not overclaimed.

Tests:
- scorecard generation is deterministic.
- scorecard includes all required dimensions.
- failures are actionable.
- normal tests do not require test_repos.
- ignored real-repo scorecard can run if test_repos exists.

Acceptance:
- scorecard exists.
- scorecard distinguishes pass/warn/fail.
- scorecard can be run locally.
- scorecard output is compact enough for agents.
- no product claims are made without evidence.

Verification:
- cargo fmt --check
- cargo test
- cargo clippy --all-targets --all-features -- -D warnings
- scorecard command/script/test if added
- cargo run --bin build_index
- cargo run --bin wi -- build_index
- cargo run --bin wi -- pack build_index
- cargo run --bin wi -- impact build_index

Commit:
Add product value scorecard
