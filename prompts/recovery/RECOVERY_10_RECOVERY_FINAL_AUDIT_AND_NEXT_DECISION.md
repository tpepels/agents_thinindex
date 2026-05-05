# RECOVERY_10_RECOVERY_FINAL_AUDIT_AND_NEXT_DECISION.md

Use superpowers:subagent-driven-development.

Do not implement this until RECOVERY_09_MINIMUM_AGENT_ACCEPTANCE_TEST.md is complete and green.

Goal:
Audit the full recovery cycle and decide whether to resume the old roadmap, continue recovery, or cut scope.

Scope:
Audit and decision only. Do not add major new features.

Current failure it addresses:
The roadmap became too large while basic product touchpoints were broken. This plan prevents returning to roadmap execution without evidence.

Phases:
- [x] Audit RECOVERY_01 through RECOVERY_09.
- [x] Verify core touchpoints.
- [x] Review performance budgets.
- [x] Review value scorecard.
- [x] Review agent acceptance workflow.
- [x] List unresolved bugs.
- [x] List unresolved value gaps.
- [x] Decide next path.
- [x] Commit.

Decision options:
- resume old roadmap at the next incomplete plan
- continue recovery with new focused plans
- cut scope to stabilize release
- block release because core loop is still not good enough

Required checks:
- wi <query> auto-builds/rebuilds and continues
- no-change build_index is fast enough
- warm wi <query> is fast enough
- wi doctor is accurate
- wi pack is useful and bounded
- wi impact is useful and honest
- agent instructions match behavior
- minimum agent acceptance test passes
- support claims are not overbroad

Output:
Create or update:
- prompts/recovery/RECOVERY_STATUS.md

Include:
- completed recovery plans
- remaining bugs
- remaining value gaps
- measured performance
- decision
- next recommended plan

Verification:
- cargo fmt --check
- cargo test
- cargo clippy --all-targets --all-features -- -D warnings
- value scorecard
- minimum agent acceptance test
- cargo run --bin build_index
- immediate second cargo run --bin build_index
- cargo run --bin wi -- build_index
- cargo run --bin wi -- pack build_index
- cargo run --bin wi -- impact build_index

Commit:
Complete recovery cycle audit
