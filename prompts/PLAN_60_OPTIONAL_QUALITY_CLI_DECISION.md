# PLAN_60_OPTIONAL_QUALITY_CLI_DECISION.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_59_AGENT_INTEGRATION_HELPERS_AND_MCP_DECISION.md is complete and green.

Goal:
Decide whether optional quality/comparator workflows need a normal CLI surface, and implement the minimum useful path or explicitly defer.

Scope:
Quality workflow UX only. Keep it optional and isolated.

Do not add:
- production parser changes
- package-manager execution
- network access
- hosted services
- telemetry
- payment/licensing enforcement
- ctags production use
- JSONL canonical storage
- `WI.md`

Context:
The feature-gap audit says quality/comparator workflows exist as library/test/report surfaces, but are not normal CLI workflows. That may be fine, or it may make quality maintenance too hidden.

Product rule:
Quality tooling should be accessible enough for maintainers, but must not become part of normal user indexing/search paths.

Phases:
- [x] Audit current quality/comparator/triage surfaces.
- [x] Decide whether a CLI is needed now.
- [x] If no CLI, document why and keep workflows test/script based.
- [x] If CLI is useful, add a narrow optional surface. (Not implemented; explicitly deferred.)
- [x] Ensure quality CLI never runs during normal `wi` or `build_index`.
- [x] Add tests.
- [x] Update docs.
- [x] Run verification.
- [x] Commit.

Decision:
Do not add a `wi quality` CLI in this pass. Quality/comparator/triage workflows
remain maintainer/test/script workflows because they depend on ignored tests,
optional local `test_repos/`, optional external comparator commands, and local
`.dev_index/quality/` artifacts. Normal `wi` and `build_index` stay isolated
from quality/comparator phases.

Possible CLI shapes:
- `wi quality report`
- `wi quality gate`
- `wi quality comparator`
- `wi quality triage`

If implemented:
- commands must be optional
- outputs must be bounded
- no ctags required
- ctags comparator remains optional/external
- no comparator output enters production records/refs
- missing `test_repos/` skips clearly
- no network
- no telemetry
- normal tests do not require external comparators

If deferred:
- docs must clearly say quality workflows are maintainer/test/script workflows.
- no docs should imply `wi quality` exists.
- list exact reasons for deferral.

Tests:
- CLI help reflects actual decision.
- quality commands are bounded if implemented.
- missing comparator skips cleanly if comparator path exists.
- normal `wi` and `build_index` do not invoke quality workflows.
- no production index pollution.
- ctags allowlist remains valid.

Docs:
Update:
- `docs/QUALITY.md`
- `docs/QUALITY_GATES.md` if present
- `docs/QUALITY_LOOP.md`
- README if it mentions quality workflows

Acceptance:
- quality CLI is either implemented minimally or explicitly deferred.
- docs match implementation.
- normal product workflows remain fast and isolated.
- optional comparator boundaries remain intact.

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- quality command/help if implemented
- ctags allowlist gate if present
- `cargo run --bin build_index`
- `cargo run --bin wi -- build_index`
- `cargo run --bin wi -- pack build_index`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` if `test_repos/` exists

Commit:
Decide optional quality CLI surface
