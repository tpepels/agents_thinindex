# PLAN_61_REAL_REPO_EVIDENCE_STABILIZATION.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_60_OPTIONAL_QUALITY_CLI_DECISION.md is complete and green.

Goal:
Make real-repo/language-support evidence more reproducible without depending only on uncommitted local `test_repos/`.

Context:
The feature wiring audit found that real-repo checks pass locally, but remain ignored/manual and depend on local third-party repos. That is useful for dogfood, but weak as release evidence.

Scope:
Evidence stabilization only. Do not add broad parser architecture, new language packs, package-manager execution, network access, MCP, hosted features, telemetry, payment/licensing enforcement, ctags production use, JSONL canonical storage, or `WI.md`.

Product rule:
Support claims need reproducible evidence. Local third-party `test_repos/` are useful, but committed synthetic/minimal corpora are needed for stable CI.

Phases:
- [ ] Inspect current real-repo checks and manifest support.
- [ ] Identify support areas that rely only on local `test_repos/`.
- [ ] Add committed synthetic mini-repos or fixture corpora for high-risk evidence gaps.
- [ ] Prioritize Go, PHP, file references, import/export references, refs, pack, and impact.
- [ ] Add expected symbols, expected patterns, expected absent symbols, and thresholds where useful.
- [ ] Keep third-party repos uncommitted.
- [ ] Update docs explaining local `test_repos/` vs committed synthetic evidence.
- [ ] Run verification.
- [ ] Commit.

Required behavior:
- Normal tests get stable committed evidence for important parser/ref/file-reference behavior.
- Ignored real-repo tests remain available for local third-party corpora.
- Docs do not claim local `test_repos/` evidence is always-on CI evidence.
- Support claims distinguish fixture/synthetic evidence from local real-repo evidence.

Tests:
- synthetic mini-repo tests run in normal `cargo test` or deterministic test suites.
- expected-symbol manifest parsing is covered.
- Go/PHP evidence gaps get targeted fixture coverage where practical.
- file-reference/import-export evidence gets targeted fixture coverage.
- real-repo ignored tests still skip/fail clearly based on local manifest state.

Docs:
Update:
- `docs/REAL_REPO_MANIFEST.md`
- `docs/LANGUAGE_SUPPORT.md`
- `docs/LANGUAGE_SUPPORT_AUDIT.md`
- `docs/QUALITY.md` if needed

Acceptance:
- support evidence is less dependent on local-only corpora.
- committed synthetic evidence covers the highest-risk gaps.
- third-party repos remain uncommitted.
- docs accurately describe evidence tiers.
- no support claim is strengthened without evidence.

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run --bin build_index -- --stats`
- `cargo run --bin wi -- refs build_index`
- `cargo run --bin wi -- pack build_index`
- `cargo run --bin wi -- impact build_index`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` if `test_repos/` exists

Commit:
Stabilize real repo evidence
