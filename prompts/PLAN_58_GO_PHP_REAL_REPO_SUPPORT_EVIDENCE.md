# PLAN_58_GO_PHP_REAL_REPO_SUPPORT_EVIDENCE.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_57_GRAPH_INCREMENTAL_RELATIONSHIP_REBUILDS.md is complete and green.

Goal:
Strengthen Go and PHP support evidence with real-repo manifests, expected symbols, expected patterns, and support-claim alignment.

Scope:
Real-repo evidence and support-claim hardening for Go and PHP only.

Do not add:
- broad new language support
- parser architecture changes unrelated to Go/PHP evidence
- package-manager execution
- network access unless explicitly limited to user-provided local repos; prefer no network
- MCP
- packaging
- licensing enforcement
- telemetry
- cloud behavior
- ctags production use
- JSONL canonical storage
- `WI.md`

Context:
The feature-gap audit says Go/PHP support still needs stronger real-repo evidence before confidence is high.

Product rule:
Do not overclaim support. A language needs fixture and real-repo evidence before being described as fully supported.

Phases:
- [x] Inspect current Go/PHP support level and docs.
- [x] Inspect existing Go/PHP fixtures.
- [x] Inspect local `test_repos/` for Go/PHP coverage.
- [x] Add or improve local manifest entries for Go/PHP where repos exist.
- [x] Add expected symbols, expected patterns, and expected absent symbols where practical.
- [x] Add fixture regressions for any parser/ref/file-reference gaps found.
- [x] Update support matrix if evidence changes support level.
- [x] Document remaining gaps honestly.
- [x] Run verification.
- [x] Commit.

Rules:
- Do not commit third-party real repos.
- Normal tests must not require `test_repos/`.
- Ignored/manual real-repo tests may use local `test_repos/`.
- If local Go/PHP repos are absent, document that evidence remains blocked and do not fabricate support.
- Do not promote Go/PHP support without evidence.

Implementation note:
Local `test_repos/` was present, but no `.go`, `.php`, `go.mod`, or
`composer.json` files were found. No local manifest entries were added because
no Go/PHP local repos exist in this checkout.

Evidence to collect:
- files scanned
- records emitted
- refs emitted
- file references emitted
- expected symbols checked
- expected patterns checked
- parser errors
- unsupported syntax
- pack/impact examples

Tests:
- Go fixture expected symbols.
- PHP fixture expected symbols.
- Go/PHP expected-symbol manifest parsing.
- no fake symbols from comments/strings.
- file references/imports for Go/PHP where practical.
- real-repo ignored test uses manifest entries if local repos exist.

Docs:
Update:
- `docs/LANGUAGE_SUPPORT.md`
- `docs/LANGUAGE_SUPPORT_AUDIT.md`
- `docs/REAL_REPO_MANIFEST.md` if manifest guidance changes
- support matrix source of truth if present

Acceptance:
- Go/PHP support claims match evidence.
- Real-repo manifest coverage exists where local repos are available.
- Missing evidence is documented honestly.
- No broad support overclaim remains.
- No third-party repos are committed.

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run --bin build_index`
- `cargo run --bin wi -- refs build_index`
- `cargo run --bin wi -- pack build_index`
- `cargo run --bin wi -- impact build_index`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` if `test_repos/` exists

Commit:
Harden Go and PHP support evidence
