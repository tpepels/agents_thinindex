# PLAN_11C_PARSER_CONFORMANCE_AND_DOCS.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_11A and PLAN_11B are complete and green.

Goal:
Harden the Tree-sitter parser framework with shared conformance tests, real-repo validation, license documentation, and honest parser-support docs.

This pass does not add the extended language pack. That belongs in PLAN_12_EXTENDED_LANGUAGE_PACK. Do not add release packaging, license enforcement, payment behavior, telemetry, cloud behavior, or new product commands.

Product rule:
A supported language is not supported until it passes shared conformance and has a license entry, docs entry, and real-repo behavior that does not break integrity checks.

Shared conformance suite:
Create or finish a reusable conformance harness for all supported languages.

Each supported language fixture should cover where applicable:
- multiline definitions
- nested declarations
- methods
- classes/types/interfaces
- imports/exports
- comments/strings that should not produce symbols
- line/col correctness
- duplicate-location absence
- stable record source
- representative `wi` searchability

The suite should be data-driven:
- language name
- fixture path
- expected symbols
- expected record kinds
- expected absent symbols
- expected extensions
- optional unsupported feature notes

Do not copy/paste assertion logic per language.

Real-repo hardening:
Extend ignored real-repo tests or benchmark reporting to show parser coverage by language:
- files seen per language
- records emitted per language
- parse errors per language
- unsupported extensions
- top gaps

Do not fail on arbitrary low record count unless a manifest threshold is added. Do fail on malformed records, duplicates, ctags source, parser panics, or unsupported parser states that violate declared support.

Diagnostics:
Parser errors should be non-fatal for unsupported/bad files but visible in doctor/bench/report output if such surfaces exist.

License/docs audit:
For every grammar dependency in the representative pack, document:
- package/crate name
- upstream repository if known
- license
- why accepted
- notice requirement

Docs to update:
- README.md
- docs/ROADMAP.md
- docs/PRODUCT_BOUNDARY.md
- THIRD_PARTY_NOTICES or equivalent
- install/release docs if present

Docs must state:
- Universal Ctags is removed.
- thinindex uses bundled permissively licensed Tree-sitter parser dependencies.
- parser support is limited to documented languages.
- parser gaps are documented.
- Tree-sitter is not semantic/LSP-level analysis.
- proprietary packaging is no longer blocked by Universal Ctags only if license audit passes.

Tests/checks:
- shared conformance harness has at least one fixture per supported language
- fixture failures print language, file, query/capture, and missing symbol
- no duplicate record locations
- no `source = "ctags"`
- no code-symbol records from comments/strings in representative fixtures
- support matrix exists
- THIRD_PARTY_NOTICES or equivalent lists parser dependencies
- docs do not claim full semantic parsing
- docs do not claim unsupported languages are supported
- docs do not claim ctags can be bundled
- local ignored test passes
- real-repo ignored test passes or reports clear unsupported gaps

Instruction surfaces:
- Do not reintroduce `WI.md`.
- Keep `wi --help` as the source of truth.
- Keep AGENTS.md and existing CLAUDE.md generation aligned with the canonical Repository search block.

Acceptance:
- language support is tested through a shared conformance suite
- parser framework is validated on fixtures and real repos
- parser support matrix is honest
- licenses/notices are documented
- active ctags dependency is gone
- no language-specific assertion duplication
- no ctags or line-scanner parser backend is reintroduced
- no GPL/AGPL dependency is introduced
- SQLite, refs, pack, impact, bench, stats, and wi-init remain stable

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
- conformance harness shape
- language support matrix
- parser/grammar license summary
- real-repo parser coverage summary
- known gaps
- packaging blocker status
- verification commands and results
- ignored local/real repo test status
- commit hash

## Phase tracking

- [x] Replace ad hoc language-pack assertions with a shared data-driven parser conformance harness.
- [x] Cover expected symbols, record kinds, extensions, absent comment/string symbols, line/col, stable source, and `wi` searchability.
- [x] Add parser diagnostics plumbing for parse-error visibility without changing CLI output.
- [x] Extend ignored real-repo validation with parser coverage reporting.
- [x] Update support matrix, parser gap docs, release docs, and notices audit tests.
- [x] Run required 11C verification.
- [x] Commit with `Harden Tree-sitter parser conformance and docs`.
