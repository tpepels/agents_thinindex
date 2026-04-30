# PLAN_24_SINGLE_CYCLE_QUALITY_IMPROVEMENT_RUNNER.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_23_COMPARATOR_TRIAGE_WORKFLOW.md is complete and green.

Progress:
- [x] Inspect existing quality loop implementation and Plan 23 prerequisites.
- [x] Implement one-cycle runner/workflow enforcement.
- [x] Add max-gap, stop-condition, deterministic-output, and final-report tests.
- [x] Update quality loop docs and final report format.
- [x] Run required verification.
- [x] Commit with completed plan checkboxes.

Goal:
Make the Check → Plan → Act quality loop safe by enforcing one bounded quality-improvement cycle per execution.

Product rule:
Quality improvement must be bounded. One run performs one cycle, verifies it, commits it, and stops.

Hard constraints:
- Run exactly one quality cycle per execution.
- Do not automatically start a second cycle.
- Do not expand language support during a quality cycle.
- Do not change parser architecture during a quality cycle.
- Do not weaken existing gates.
- Do not make ctags required.
- Do not add packaging, license enforcement, payment behavior, telemetry, or cloud behavior.

Cycle phases:
1. Check:
   - run normal quality gate
   - run real-repo gate if `test_repos/` exists
   - run optional comparator report if available
   - collect gaps

2. Plan:
   - write one cycle plan under `.dev_index/quality/`
   - select at most 10 gaps
   - prioritize supported-language missed expected symbols
   - deprioritize comparator noise
   - list files/tests/docs to change

3. Act:
   - implement selected fixes
   - add conformance fixtures or expected-symbol checks where practical
   - update docs only if support status changes
   - rerun verification
   - commit

Stop conditions:
Stop after one cycle if:
- all selected gaps are fixed
- remaining gaps are unsupported syntax
- remaining gaps are comparator false positives
- remaining gaps require architecture/language-expansion work
- verification fails and needs human review

Required implementation:
1. Add a quality-cycle runner or documented script/test workflow.
2. Enforce one-cycle limit.
3. Add max-gap selection config/default of 10.
4. Add tests for cycle selection, stop conditions, and deterministic output.
5. Add docs warning agents not to start another cycle automatically.
6. Add final report format.

Acceptance:
- one-cycle quality runner/workflow exists
- max gap limit exists
- stop conditions are enforced
- cycle output is deterministic and actionable
- parser fixes from cycle require regression fixtures where practical
- no runaway agent loop is possible by design
- existing quality/plugin/parser behavior remains stable

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- ctags allowlist gate
- license audit command if configured
- one-cycle quality runner/script if added
- `cargo run --bin build_index`
- `cargo run --bin wi -- build_index`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored`

Report:
- changed files
- one-cycle workflow
- max gap selection behavior
- stop conditions
- sample cycle plan/report
- verification commands and results
- commit hash
