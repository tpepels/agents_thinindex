# RECOVERY_09_MINIMUM_AGENT_ACCEPTANCE_TEST.md

Use superpowers:subagent-driven-development.

Do not implement this until RECOVERY_08_ERROR_MESSAGE_AND_HELP_AUDIT.md is complete and green.

Goal:
Add a minimum agent acceptance test that proves an agent can use thinindex for a realistic small code-change workflow.

Scope:
Local acceptance workflow only. Do not add MCP, packaging, licensing, payment, cloud, telemetry, or hosted features.

Current failure it addresses:
Agents may still ignore thinindex or find it unhelpful. We need an executable workflow that proves the tool is useful before deeper integrations.

Phases:
- [ ] Define a small fixture repo/task.
- [ ] Define expected agent workflow steps.
- [ ] Add an acceptance test or scripted check.
- [ ] Ensure workflow uses wi, wi pack, and wi impact.
- [ ] Ensure output is bounded and useful.
- [ ] Update docs with the workflow.
- [ ] Run verification.
- [ ] Commit.

Workflow:
- initialize or use fixture repo
- search for a symbol/concept with wi
- inspect refs with wi refs
- build a context pack with wi pack
- inspect impact with wi impact
- identify likely files to edit/test
- avoid broad blind read/grep as the first step

Acceptance criteria:
- the workflow completes without manual build_index pre-step
- stale/missing index self-heals
- outputs are compact
- pack output includes the relevant files
- impact output includes plausible affected/test/config files
- docs explain this as the minimum expected agent behavior

Tests:
- scripted acceptance path passes
- stale index path passes
- no unnecessary rebuild on second run
- no quality/comparator/real-repo work runs in normal path

Verification:
- cargo fmt --check
- cargo test
- cargo clippy --all-targets --all-features -- -D warnings
- minimum agent acceptance test/script
- cargo run --bin build_index
- cargo run --bin wi -- pack build_index
- cargo run --bin wi -- impact build_index

Commit:
Add minimum agent acceptance workflow
