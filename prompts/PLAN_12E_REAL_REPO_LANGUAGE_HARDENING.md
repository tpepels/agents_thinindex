# PLAN_12E_REAL_REPO_LANGUAGE_HARDENING.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_12A, PLAN_12B, PLAN_12C, and PLAN_12D are complete and green.

Goal:
Run the Tree-sitter parser framework against real repos under `test_repos/` and harden language support until every declared supported language works on those repos and captures the important symbols those repos contain.

This pass is iterative parser hardening. Do not add a second parser architecture. Do not add hand parsers, line scanners, release packaging, license enforcement, payment behavior, telemetry, cloud behavior, or new product commands unless required for test/reporting support.

Product rule:
A language is only supported if it survives real repositories, not just fixtures.

Definitions:
- “Supported language” means a language listed as supported in the parser support matrix.
- “Works on real repos” means:
  - parser does not panic
  - parser errors are bounded and reported
  - index records are valid
  - no duplicate `path + line + col`
  - no `source = "ctags"`
  - representative symbols are emitted for files of that language
  - important symbol kinds for that language are captured with useful names and locations
  - obvious top-level declarations are not missed
  - refs/pack/impact do not break on those repos
  - known unsupported syntax is documented instead of silently failing

Real repo source:
Use `test_repos/`.

Behavior:
- If `test_repos/` is missing, skip with a clear message.
- If `test_repos/` has no repos, skip with a clear message.
- If `test_repos/MANIFEST.toml` exists, use it.
- Otherwise discover immediate child repos as already implemented.
- Do not download repos automatically.
- Do not commit third-party repos.

Required hardening loop:
For each repo under `test_repos/`:
1. delete that repo’s `.dev_index`
2. run `build_index`
3. run shared index integrity checks
4. run shared ref integrity checks
5. collect parser coverage by language
6. collect parse failures/errors by language
7. collect unsupported extensions
8. collect empty-output supported-language cases
9. collect suspected missed-symbol cases for supported files
10. run representative `wi` queries where manifest queries exist
11. run symbol-coverage checks where manifests define expected symbols or expected symbol patterns
12. run `wi refs`, `wi pack`, and `wi impact` smoke checks where useful

Then:
- fix parser/query/conformance issues found
- add or update fixtures for each real-repo failure class
- rerun until all declared supported languages pass on available real repos, or a blocker is explicitly documented

Do not leave a real-repo failure as “known caveat” if it is fixable in the framework/query pack.

Symbol coverage policy:
For real repos, hardening should check both validity and coverage.

Coverage checks may use:
- manifest-defined expected symbols
- expected symbol regex/patterns per repo
- expected minimum records per supported-language file class
- expected presence of common top-level declarations
- fixture backfills for missed real-world syntax

Do not use fragile exact total record counts as the main quality gate. Prefer targeted expected symbols/patterns and language-specific minimum sanity checks.

Failure policy:
Fail the ignored real-repo hardening test if:
- any supported-language file causes a parser panic
- malformed records are emitted
- duplicate record locations exist
- records contain `source = "ctags"`
- `.dev_index/` is indexed
- refs are malformed
- a supported language has files in real repos but emits zero useful records without a documented reason
- expected manifest symbols/patterns are missing
- obvious top-level declarations are missed by a supported language
- a parser/query failure is recurring and not documented as a blocker

Allowed non-failures:
- unsupported languages/extensions, if reported
- generated/vendor/minified files, if ignored or documented
- syntax variants not yet supported, if documented and not claimed supported
- repos absent from `test_repos/`

Reporting:
Add or improve real-repo parser report output.

Report per repo:
- repo name/path
- files indexed
- records emitted
- refs emitted
- languages detected
- supported-language file counts
- record counts per language
- parser errors per language
- unsupported extensions
- expected symbols checked
- expected symbols missing
- suspected missed-symbol areas
- top missing/weak language areas

Report aggregate:
- total repos checked
- total supported languages seen
- supported languages passing
- supported languages with failures
- unsupported extensions
- parser gap summary

Conformance backfill:
Every bug found in real repos must get a fixture/conformance case where practical.

Examples:
- multiline declarations
- nested classes/modules
- decorators/annotations/attributes
- generics/templates
- namespace/package declarations
- imports/includes/requires
- overloaded methods/functions
- constructors/destructors
- anonymous functions assigned to named variables/constants
- exported declarations
- public API declarations
- comments/strings that looked like code
- unusual but valid syntax

Tests:
- ignored real-repo hardening test runs the full report/check path
- normal tests do not require `test_repos/`
- fixture/conformance tests are added for real-repo parser bugs
- no duplicate assertion logic; use shared integrity/conformance helpers
- no ctags skip logic
- no line-scanner fallback for code symbols

Docs:
Update parser support docs with:
- real-repo hardening status
- symbol coverage expectations
- known unsupported syntax
- unsupported extensions
- language confidence levels if useful
- how to add a real repo to `test_repos/`
- how to add manifest queries, expected paths, expected symbols, and expected symbol patterns if supported

Instruction surfaces:
- Do not reintroduce `WI.md`.
- Keep `wi --help` as the source of truth.
- Keep AGENTS.md and existing CLAUDE.md generation aligned with the canonical Repository search block.

Acceptance:
- real-repo hardening report exists
- ignored real-repo test validates parser health and symbol coverage across `test_repos/`
- every declared supported language seen in `test_repos/` passes the hardening checks
- manifest-defined expected symbols/patterns are checked where present
- missed important symbols become fixture/conformance tests where practical
- failures are fixed with framework/query changes, not hand parsers
- each fixed failure gets fixture/conformance coverage where practical
- unsupported/deferred cases are documented
- no ctags or line-scanner code parser backend is reintroduced
- no GPL/AGPL dependency is introduced
- SQLite, refs, pack, impact, bench, stats, and wi-init remain stable

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run --bin build_index`
- `cargo run --bin wi -- build_index`
- `cargo run --bin wi -- refs build_index`
- `cargo run --bin wi -- pack build_index`
- `cargo run --bin wi -- impact build_index`
- `cargo run --bin wi-stats`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored`

Report:
- changed files
- repos checked under `test_repos/`
- language coverage summary
- symbol coverage summary
- expected symbols checked/missing
- parser failures fixed
- missed-symbol failures fixed
- fixture/conformance cases added
- unsupported/deferred blockers
- known gaps
- verification commands and results
- ignored local/real repo test status
- commit hash
