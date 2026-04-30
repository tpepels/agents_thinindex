# PLAN_20_SUPPORT_LEVELS_AND_LANGUAGE_CLAIMS.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_19_CONTINUOUS_QUALITY_IMPROVEMENT_PLUGIN_LOOP.md is complete and green.

Goal:
Add explicit parser support levels so thinindex cannot overclaim language support.

This is a policy/docs/tests pass. Do not add new parser architecture, new languages, release packaging, licensing enforcement, payment behavior, telemetry, cloud behavior, or new product commands unless needed for support-level reporting.

Progress:
- [x] Add a source-of-truth language and format support-level model.
- [x] Update README and parser support docs to use support levels consistently.
- [x] Update quality/product/license docs where support-level claims depend on them.
- [x] Add claim-protection tests for support levels, conformance fixtures, and license metadata.
- [x] Run required verification.
- [x] Commit completed Plan 20 changes.

Product rule:
A language is not “supported” unless that claim is backed by conformance fixtures, real-repo checks, docs, and license metadata.

Support levels:
Define exactly these support levels:
- supported
- experimental
- blocked
- extras-backed

Definitions:
- supported: grammar/query/fixture/license/docs exist; conformance passes; real-repo checks pass where relevant; expected-symbol checks pass where configured.
- experimental: grammar/query exists, but conformance or real-repo coverage is incomplete.
- blocked: missing permissive grammar, broken integration, unclear license, or unacceptable parser quality.
- extras-backed: intentionally handled by project-owned extras instead of Tree-sitter, usually for CSS/HTML/Markdown/TODO/config-style records.

Required implementation:
1. Add a single source of truth for language/format support levels.
2. Include:
   - language/format name
   - extensions
   - support level
   - backend: tree_sitter or extras
   - supported record kinds
   - known gaps
   - license status
3. Use this source in docs/tests/reports where practical.
4. Remove or rewrite docs that imply all listed languages are equally supported.
5. Add tests that fail if docs claim support for a language marked experimental or blocked.
6. Add tests that fail if a supported language lacks conformance coverage.
7. Add tests that fail if a supported language lacks license metadata.

Do not:
- promote experimental languages to supported without evidence
- hide blocked languages
- call extras-backed support Tree-sitter-backed
- weaken existing conformance/quality gates

Docs:
Update:
- README.md
- docs/QUALITY.md
- docs/PRODUCT_BOUNDARY.md
- parser support matrix docs
- THIRD_PARTY_NOTICES or equivalent if support-level references depend on license entries

Acceptance:
- every language/format has an explicit support level
- docs use support levels consistently
- no unsupported or experimental language is described as fully supported
- extras-backed formats are clearly marked
- tests protect support-level claims
- no ctags or line-scanner code parser backend is reintroduced
- no GPL/AGPL dependency is introduced

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- license audit command if configured
- targeted ctags gate from quality plans
- `cargo run --bin build_index`
- `cargo run --bin wi -- build_index`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored`

Report:
- changed files
- support level model
- supported/experimental/blocked/extras-backed matrix
- claim-protection tests added
- verification commands and results
- commit hash
