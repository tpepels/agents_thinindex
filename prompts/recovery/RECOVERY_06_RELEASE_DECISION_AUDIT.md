# RECOVERY_06_RELEASE_DECISION_AUDIT.md

Use superpowers:subagent-driven-development.

Do not implement this until RECOVERY_05_AGENT_INTEGRATION_MINIMUM_USEFUL.md is complete and green.

Goal:
Decide whether thinindex is ready to resume the old roadmap, continue recovery, or cut scope.

Scope:
Audit and decision only. Do not add major features.

Current failure it addresses:
The project needs an evidence-based decision before returning to long-range roadmap work.

Phases:
- [ ] Audit RECOVERY_00 through RECOVERY_05.
- [ ] Verify core touchpoints.
- [ ] Verify performance budgets.
- [ ] Verify value workflows.
- [ ] Verify minimum agent integration.
- [ ] Identify remaining bugs.
- [ ] Identify remaining value gaps.
- [ ] Decide next path.
- [ ] Create or update prompts/recovery/RECOVERY_STATUS.md.
- [ ] Run verification.
- [ ] Commit.

Decision options:
- resume old roadmap at the next incomplete plan
- continue recovery with more focused plans
- cut scope to stabilize release
- block release because core loop is still not good enough

Required checks:
- wi <query> auto-builds/rebuilds and continues
- no-change build_index is fast enough
- warm wi <query> is fast enough
- wi doctor is accurate
- wi --help is accurate
- wi pack is useful and bounded
- wi impact is useful and honest
- wi-init instructions match behavior
- minimum agent surfaces exist and are idempotent
- support claims are not overbroad

Output:
Create or update prompts/recovery/RECOVERY_STATUS.md with:
- completed recovery plans
- remaining bugs
- remaining value gaps
- measured performance
- decision
- next recommended plan

Acceptance:
- release/resume decision exists.
- decision is evidence-backed.
- remaining bugs/gaps are explicit.
- next action is unambiguous.

Verification:
- cargo fmt --check
- cargo test
- cargo clippy --all-targets --all-features -- -D warnings
- cargo run --bin build_index
- immediate second cargo run --bin build_index
- cargo run --bin wi -- build_index
- cargo run --bin wi -- pack build_index
- cargo run --bin wi -- impact build_index
- cargo run --bin wi-stats
- cargo test --test local_index -- --ignored
- cargo test --test real_repos -- --ignored if test_repos exists

Commit:
Complete recovery release decision audit
