# RECOVERY_01_CURRENT_STATE_AUDIT.md

Use superpowers:subagent-driven-development.

Do not implement this until RECOVERY_00_RECOVERY_SCOPE_AND_FREEZE.md is complete and green.

Goal:
Audit the current thinindex state and determine what actually works, what is broken, and what is only planned.

Scope:
Audit only. Do not implement product fixes unless they are trivial audit tooling.

Current failure it addresses:
There are many plan files, but basic behavior may still be wrong. The project needs evidence, not assumptions.

Phases:
- [ ] Inventory active plan files and superseded plan files.
- [ ] Inspect current code paths for wi, build_index, wi doctor, wi-init, wi-stats, refs, pack, and impact.
- [ ] Check whether ctags, WI.md, JSONL canonical storage, stale native parser code, or old assumptions remain.
- [ ] Run the core commands in the current repo.
- [ ] Run the core commands in at least one real repo if available.
- [ ] Record current bugs.
- [ ] Record current value gaps.
- [ ] Create prompts/recovery/CURRENT_STATE.md.
- [ ] Run verification.
- [ ] Commit.

Audit areas:
- stale/missing index behavior
- build_index performance
- warm wi query latency
- wi doctor accuracy
- wi --help accuracy
- wi-init generated instructions
- refs usefulness
- pack usefulness
- impact usefulness
- parser support claims
- quality/comparator isolation
- test_repos behavior

Required output:
Create prompts/recovery/CURRENT_STATE.md with:
- current behavior summary
- commands run
- failing commands
- performance observations
- confirmed bugs
- confirmed value gaps
- stale assumptions
- recommended next plan

Acceptance:
- CURRENT_STATE.md exists.
- bugs are evidence-backed.
- value gaps are prioritized.
- no product fixes are hidden inside the audit.

Verification:
- cargo fmt --check
- cargo test
- cargo clippy --all-targets --all-features -- -D warnings
- cargo run --bin build_index
- cargo run --bin wi -- build_index
- cargo run --bin wi -- pack build_index
- cargo run --bin wi -- impact build_index
- cargo run --bin wi-stats
- cargo test --test local_index -- --ignored
- cargo test --test real_repos -- --ignored if test_repos exists

Commit:
Audit current thinindex state
