# RECOVERY_02_CORE_TOUCHPOINT_REPAIR.md

Use superpowers:subagent-driven-development.

Do not implement this until RECOVERY_01_CURRENT_STATE_AUDIT.md is complete and green.

Goal:
Repair the core user and agent touchpoints before any roadmap work continues.

Scope:
Fix wi, build_index, wi doctor, wi --help, wi-init, and generated instructions only where they directly affect the core loop.

Current failure it addresses:
wi currently tells users to manually run build_index when the index is stale. Intended behavior is automatic one-shot rebuild followed by the original query.

Phases:
- [ ] Reproduce missing/stale/schema-stale index behavior.
- [ ] Implement one-shot auto-build for missing index in wi <query>.
- [ ] Implement one-shot auto-rebuild for stale index in wi <query>.
- [ ] Implement one-shot auto-rebuild for schema-stale index in wi <query>.
- [ ] Ensure original query continues after rebuild.
- [ ] Ensure rebuild failure reports clearly.
- [ ] Ensure no rebuild loop is possible.
- [ ] Repair wi doctor state reporting.
- [ ] Align wi --help and generated instructions with actual behavior.
- [ ] Add tests.
- [ ] Run verification.
- [ ] Commit.

Required behavior:
- wi <query> with missing index builds once and continues.
- wi <query> with stale index rebuilds once and continues.
- wi <query> with schema-stale index rebuilds once and continues.
- immediate second wi <query> does not rebuild.
- failed rebuild gives clear error and next action.
- build_index remains available but is not required before every query.
- normal wi/build_index paths do not run quality/comparator/real-repo checks.

Tests:
- missing index auto-builds during wi <query>.
- stale index auto-rebuilds during wi <query>.
- schema-stale index auto-rebuilds during wi <query>.
- rebuild failure reports clearly.
- no rebuild loop.
- immediate second query does not rebuild.
- wi doctor accurately reports missing/stale/current states.
- generated AGENTS.md guidance matches auto-rebuild behavior.
- existing CLAUDE.md normalized if present and not created if absent.
- WI.md is not reintroduced.

Acceptance:
- basic wi query self-heals stale/missing index.
- doctor/help/init agree with actual behavior.
- tests prevent regression.
- no unrelated roadmap features are added.

Verification:
- cargo fmt --check
- cargo test
- cargo clippy --all-targets --all-features -- -D warnings
- cargo run --bin build_index
- stale-state wi test showing auto-rebuild then results
- immediate second wi test showing no rebuild
- cargo run --bin wi -- --help
- cargo run --bin wi-init -- --help
- cargo test --test local_index -- --ignored
- cargo test --test real_repos -- --ignored if test_repos exists

Commit:
Repair core thinindex touchpoints
