# PLAN_12B_EXTENDED_APP_SYSTEM_LANGUAGE_PACK.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_12A is complete and green.

Goal:
Add Swift, Dart, and Nix support through the existing generic Tree-sitter framework.

This pass expands language coverage for app/platform and system/config-heavy projects. Do not add a second parser architecture. Do not add hand parsers, line scanners, release packaging, license enforcement, payment behavior, telemetry, cloud behavior, or new product commands.

Product rule:
Adding a language must be routine: grammar registration, extension mapping, query specs, conformance fixture, license entry, docs entry.

Languages:
- Swift
- Dart
- Nix

Hard requirements:
- Use the existing Tree-sitter extraction framework.
- Do not create a second parser architecture.
- Do not use line-oriented or regex-based code-symbol parsing.
- Do not call or reintroduce ctags.
- Do not add GPL or AGPL dependencies.
- Every grammar/dependency must have a license entry.
- Every supported language must have a conformance fixture.
- No newly built index may emit `source = "ctags"`.

Allowed language-specific work:
- grammar registration
- file extension mapping
- Tree-sitter query specs
- conformance fixture
- license/audit entry
- tiny grammar adapter only if unavoidable and documented

Forbidden language-specific work:
- hand-written parser
- line scanner
- copy/pasted extraction loop
- broad regex parser
- ctags fallback

Per-language artifacts:
For each supported language add:
- grammar registration
- extension mapping
- Tree-sitter query spec
- conformance fixture
- license/audit entry
- support matrix entry
- expected record-kind coverage note

Record kinds:
Where syntax supports them, extract:
- function
- method
- class
- struct
- enum
- interface/protocol where applicable
- type
- module/package/import
- variable
- constant
- import/export-like constructs

Nix:
Keep extraction conservative and useful:
- attribute names where useful
- functions where practical
- imports where practical

Do not emit every scalar value as a symbol.

Tests:
- every newly supported language has at least one fixture in the shared conformance suite
- multiline declarations are tested where syntax supports them
- comments/strings do not create fake symbols
- line/col are 1-based and accurate
- no duplicate path+line+col records
- no `source = "ctags"`
- representative `wi` commands work for Swift, Dart, and Nix where supported
- existing representative pack tests still pass
- existing refs/pack/impact/stats tests still pass

Docs:
Update parser support matrix and THIRD_PARTY_NOTICES or equivalent for Swift, Dart, and Nix.

Docs must state:
- which languages are supported
- which languages are Tree-sitter-backed
- which languages are unsupported/deferred and why
- Tree-sitter is not semantic/LSP-level analysis

Instruction surfaces:
- Do not reintroduce `WI.md`.
- Keep `wi --help` as the source of truth.
- Keep AGENTS.md and existing CLAUDE.md generation aligned with the canonical Repository search block.

Acceptance:
- Swift, Dart, and Nix are implemented through the existing generic framework, or explicitly blocked with license/integration reasons
- no second parser architecture is introduced
- no ctags or line-scanner parser backend is reintroduced
- no GPL/AGPL dependency is introduced
- support matrix and license notices are updated
- existing product behavior remains stable

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run --bin build_index`
- representative `wi` commands for Swift, Dart, and Nix where supported
- `cargo run --bin wi -- refs build_index`
- `cargo run --bin wi -- pack build_index`
- `cargo run --bin wi -- impact build_index`
- `cargo run --bin wi-stats`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` if `test_repos/` exists

Report:
- changed files
- supported language matrix
- unsupported/deferred language blockers
- grammar dependencies and licenses
- representative smoke outputs
- known extraction gaps
- verification commands and results
- ignored local/real repo test status
- commit hash

## Phase tracking

- [x] Add permissively licensed Tree-sitter dependencies for Swift, Dart, and Nix.
- [x] Register Swift, Dart, and Nix through existing adapters, extension mapping, and query specs.
- [x] Add conformance fixtures and shared conformance cases for Swift, Dart, and Nix.
- [x] Update support matrix, known gaps, and third-party notices.
- [x] Run required 12B verification.
- [x] Commit with `Add extended app and system parser pack`.
