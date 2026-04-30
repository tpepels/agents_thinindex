# PLAN_12D_EXTENDED_LANGUAGE_PACK_CONFORMANCE_AND_DOCS.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_12A, PLAN_12B, and PLAN_12C are complete and green.

Goal:
Finalize the extended language pack with shared conformance, docs, license entries, and real-repo hardening.

This pass is cleanup and hardening for the extended pack. Do not add a second parser architecture. Do not add release packaging, license enforcement, payment behavior, telemetry, cloud behavior, or new product commands.

Product rule:
Extended language support is only real if it is documented, license-audited, conformance-tested, and stable on real repos.

Scope:
Cover all extended languages/formats added or attempted in PLAN_12A through PLAN_12C:
- C#
- Scala
- Kotlin
- Swift
- Dart
- Nix
- CSS
- HTML
- Markdown
- JSON
- TOML
- YAML
- any Ruby/PHP/JSX/TSX carryover if they were not already completed in PLAN_11B

Required checks:
- every supported extended language has grammar/extension/query/fixture/license/docs entries
- every unsupported/deferred language has a clear blocker
- no supported language uses hand parsing or line scanning for code-symbol extraction
- accepted extras-backed formats are explicitly documented
- no active ctags dependency
- no `source = "ctags"` in newly built indexes
- no GPL/AGPL dependency

Shared conformance:
Use the same conformance harness as PLAN_11C.

Do not copy/paste assertion logic per language.

Real-repo hardening:
Run ignored real-repo tests if `test_repos/` exists.

Report parser coverage by language where supported:
- files seen
- records emitted
- parse errors
- unsupported extensions
- top gaps

Docs:
Update:
- README.md
- docs/ROADMAP.md
- docs/PRODUCT_BOUNDARY.md
- parser support matrix docs
- THIRD_PARTY_NOTICES or equivalent
- install/release docs if they mention parser support

Docs must state:
- supported languages/formats
- unsupported/deferred languages/formats
- Tree-sitter-backed vs extras-backed support
- known parser gaps
- Tree-sitter is not semantic/LSP-level analysis
- Universal Ctags is not bundled and not used

Tests/checks:
- support matrix exists and includes extended languages
- THIRD_PARTY_NOTICES or equivalent lists parser dependencies
- docs do not claim unsupported languages are supported
- docs do not claim ctags can be bundled
- no `source = "ctags"` in fixture/local indexes
- existing refs/pack/impact/stats tests pass
- local ignored test passes
- real-repo ignored test passes or reports clear unsupported gaps

Instruction surfaces:
- Do not reintroduce `WI.md`.
- Keep `wi --help` as the source of truth.
- Keep AGENTS.md and existing CLAUDE.md generation aligned with the canonical Repository search block.

Acceptance:
- extended language pack is finalized
- support matrix is honest
- licenses/notices are updated
- unsupported languages are explicitly blocked/deferred
- no second parser architecture is introduced
- no ctags or line-scanner code parser backend is reintroduced
- no GPL/AGPL dependency is introduced
- SQLite, refs, pack, impact, bench, stats, and wi-init remain stable
- packaging plans may proceed only if license status allows

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- license audit command if configured
- `grep -R "ctags\\|Ctags\\|CTAGS" src tests docs README.md Cargo.toml install.sh uninstall.sh THIRD_PARTY_NOTICES || true`
- `cargo run --bin build_index`
- `cargo run --bin wi -- build_index`
- `cargo run --bin wi -- refs build_index`
- `cargo run --bin wi -- pack build_index`
- `cargo run --bin wi -- impact build_index`
- `cargo run --bin wi-stats`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` if `test_repos/` exists

Report:
- changed files
- extended support matrix
- Tree-sitter-backed vs extras-backed support
- parser/grammar license summary
- unsupported/deferred blockers
- known gaps
- packaging blocker status
- verification commands and results
- ignored local/real repo test status
- commit hash

## Phase tracking

- [x] Add shared extras-backed format conformance for CSS, HTML, Markdown, JSON, TOML, and YAML.
- [x] Verify extended language and format support matrix/docs/notices are explicit and honest.
- [x] Document unlisted/deferred language and format blockers.
- [x] Keep real-repo parser coverage reporting on the ignored real-repo path.
- [x] Run required 12D verification.
- [x] Commit with `Finalize extended language pack conformance`.
