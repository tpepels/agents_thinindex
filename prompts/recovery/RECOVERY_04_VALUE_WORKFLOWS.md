# RECOVERY_04_VALUE_WORKFLOWS.md

Use superpowers:subagent-driven-development.

Do not implement this until RECOVERY_03_PERFORMANCE_PROFILING_AND_BUDGETS.md is complete and green.

Goal:
Define and test real product workflows that prove thinindex creates value for humans and agents.

Scope:
Workflow tests, docs, and focused fixes only. Do not add broad architecture, packaging, licensing, payment, cloud, telemetry, or MCP work.

Current failure it addresses:
The project has architecture, but not enough proof that the actual workflows are useful.

Phases:
- [x] Define core value workflows.
- [x] Add fixture or scripted checks for each workflow.
- [x] Run workflows on a real repo if available.
- [x] Fix obvious workflow failures.
- [x] Document workflows.
- [x] Run verification.
- [x] Commit.

Core workflows:
1. find symbol:
   - wi <symbol> returns useful file:line results.

2. find broad concept:
   - wi <term> returns useful candidates without huge noise.

3. inspect refs:
   - wi refs <term> returns useful reference evidence.

4. build context:
   - wi pack <term> returns bounded useful read set.

5. inspect impact:
   - wi impact <term> returns plausible affected files and reasons.

6. diagnose state:
   - wi doctor explains missing/stale/current state.

7. initialize agent instructions:
   - wi-init creates useful AGENTS.md and normalizes existing CLAUDE.md if present.

Required output:
Create or update:
- prompts/recovery/VALUE_WORKFLOWS.md
- docs workflow section if appropriate

Tests:
- workflow fixture tests pass.
- pack output includes expected files.
- impact output includes expected reason/confidence where available.
- outputs are bounded.
- stale/missing index self-heals during workflows.
- normal workflow does not run quality/comparator/real-repo checks.

Acceptance:
- value workflows are documented.
- value workflows are tested.
- obvious workflow failures are fixed.
- output is useful and bounded.

Verification:
- cargo fmt --check
- cargo test
- cargo clippy --all-targets --all-features -- -D warnings
- cargo run --bin build_index
- cargo run --bin wi -- build_index
- cargo run --bin wi -- refs build_index
- cargo run --bin wi -- pack build_index
- cargo run --bin wi -- impact build_index
- cargo test --test local_index -- --ignored
- cargo test --test real_repos -- --ignored if test_repos exists

Commit:
Define and test thinindex value workflows
