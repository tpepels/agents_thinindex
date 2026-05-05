# RECOVERY_11_REAL_REPO_READINESS_AND_SUPPORT_CLAIMS.md

Use superpowers:subagent-driven-development.

Do not implement this until RECOVERY_10_RECOVERY_FINAL_AUDIT_AND_NEXT_DECISION.md is complete and green.

Goal:
Harden real-repo readiness and support claims before resuming the old roadmap.

Scope:
Audit and fix real-repo checks, parser support claims, support levels, expected-symbol coverage, and mismatches between docs and implementation. Do not add broad parser architecture, packaging, licensing, payment, cloud, telemetry, hosted features, or MCP work.

Current failure it addresses:
RECOVERY_STATUS.md says the core product loop is usable, but old roadmap work should remain paused until real-repo readiness, skip reasons, and support-claim confidence are hardened. The known gaps are evidence quality gaps, not permission to resume broad roadmap execution.

Phases:
- [ ] Read RECOVERY_STATUS.md and record the real-repo/support recommendation.
- [ ] Read the current parser support matrix and generated support docs.
- [ ] Run or inspect `cargo test --test real_repos -- --ignored`.
- [ ] Audit `test_repos/MANIFEST.toml` coverage, skip behavior, and local-only ergonomics.
- [ ] Verify declared supported languages are not overclaimed.
- [ ] Verify experimental, blocked, and extras-backed statuses are honest.
- [ ] Classify Go/PHP real-repo coverage gaps without weakening fixture-backed support claims.
- [ ] Add expected-symbol or expected-pattern checks for real repos where useful.
- [ ] Classify exploratory side corpora that are not ready for expected-symbol checks.
- [ ] Fix stale docs/support claims.
- [ ] Run verification.
- [ ] Commit.

Initial classified gaps from RECOVERY_STATUS and current inspection:
- Go and PHP are fixture-backed supported languages, but `docs/LANGUAGE_SUPPORT_AUDIT.md` records no Go-heavy or PHP-heavy local real-repo target in this checkout. Classify this as a real-repo hardening gap unless a practical local manifest target exists.
- `test_repos/MANIFEST.toml` exists and has expected-symbol/expected-pattern checks for many repos, but several side corpora are still exploratory. Add checks where practical; otherwise document why the corpus remains exploratory.
- Extra local repos under ignored `test_repos/` may exist outside the manifest. Decide whether each practical repo should be added, explicitly skipped, or documented as out of scope without committing third-party contents.
- The ignored real-repo test can run for a long time with limited progress visibility. Improve or document expected runtime, progress, and triage behavior if this blocks practical use.
- Support docs mostly align with `src/support.rs`; the risk is insufficient real-repo evidence for some claims, not an obvious static support-level mismatch.

Required checks:
- Read `prompts/recovery/RECOVERY_STATUS.md`.
- Read `src/support.rs`, `docs/LANGUAGE_SUPPORT.md`, and `docs/LANGUAGE_SUPPORT_AUDIT.md`.
- Run or inspect `cargo test --test real_repos -- --ignored`; if it cannot complete locally, record concrete runtime/progress evidence and classify the blocker.
- Verify declared supported languages are not overclaimed.
- Verify experimental languages are not promoted without evidence.
- Verify blocked languages remain unclaimed.
- Verify extras-backed formats are not described as Tree-sitter-backed code-symbol parsers.
- Verify `test_repos/MANIFEST.toml` expectations exist where practical.
- Add expected-symbol, expected-pattern, or expected-absent-symbol checks for practical manifest repos.
- Fix stale docs/support claims found during the audit.
- Keep normal `wi` and `build_index` paths free of quality/comparator/real-repo work.

Hard requirements:
- Do not resume old roadmap plans.
- Do not add broad new parser architecture.
- Do not reintroduce Universal Ctags as a production parser.
- Do not reintroduce `WI.md`.
- Do not reintroduce JSONL as canonical storage.
- Do not commit third-party repository contents from `test_repos/`.
- Do not treat missing local third-party repos as normal validation blockers.
- Do not downgrade supported languages reflexively when fixture/conformance coverage is valid; classify missing real-repo coverage separately.

Acceptance:
- RECOVERY_11 exists and is listed in `prompts/recovery/PLAN_ORDER.md`.
- Real-repo readiness gaps from RECOVERY_STATUS.md are addressed or explicitly classified.
- Support claims match actual tested behavior.
- Remaining gaps are documented as blocked, experimental, unsupported, exploratory, or future work.
- Manifest repos have expected-symbol or expected-pattern checks where practical.
- Ignored real-repo test behavior is usable or its blocker is evidence-backed and documented.
- Verification is green or explicitly inapplicable with evidence.

Verification:
- cargo fmt --check
- cargo test
- cargo clippy --all-targets --all-features -- -D warnings
- cargo test --test real_repos -- --ignored
- cargo run --bin build_index
- cargo run --bin wi -- build_index
- cargo run --bin wi -- pack build_index
- cargo run --bin wi -- impact build_index

Commit:
Harden real repo readiness and support claims
