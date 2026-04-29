# PLAN_12F_PARSER_COVERAGE_CLOSURE.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_12A through PLAN_12E are complete and green.

Goal:
Take parser symbol coverage from “good enough” to coverage-closed for declared supported languages.

Coverage-closed means:
- every declared supported language has broad construct coverage
- every known missed-symbol case from fixtures or `test_repos/` is fixed or explicitly marked unsupported
- every supported language has a conformance fixture that covers common real-world syntax
- every supported language has manifest-backed real-repo expected-symbol checks where practical
- no known supported-language parser gap remains undocumented

Do not use the word “perfect” in docs or reports. Perfect parser coverage is not testable across all language syntax, generated code, macros, framework magic, and semantic resolution.

Product rule:
A supported language must capture important symbols reliably enough for agent navigation. Unsupported syntax must be explicit.

Scope:
This pass is parser-quality closure only.

Do not add:
- a second parser architecture
- hand parsers
- line scanners
- ctags
- release packaging
- license enforcement
- payment behavior
- telemetry
- cloud behavior
- new product commands

Hard requirements:
- Use the existing Tree-sitter extraction framework.
- Do not create a second parser architecture.
- Do not use line-oriented or regex-based code-symbol parsing.
- Do not call or reintroduce ctags.
- Do not add GPL or AGPL dependencies.
- No newly built index may emit `source = "ctags"`.
- Do not weaken duplicate-location invariants.
- Do not weaken real-repo hardening checks from PLAN_12E.

Coverage closure checklist:
For every declared supported code language, verify fixture coverage for common constructs where applicable:

- top-level functions
- methods
- constructors
- destructors/finalizers where applicable
- classes
- structs
- enums
- interfaces
- traits/protocols
- type aliases
- modules/packages/namespaces
- constants
- variables assigned to named functions/lambdas where useful
- imports/includes/requires/using statements
- exports/public declarations
- nested declarations
- multiline declarations
- generic/template declarations
- attributes/decorators/annotations
- comments/strings that look like code but must not emit symbols

For config/document/web formats, verify useful non-noisy coverage:

- CSS classes/ids/variables/keyframes
- HTML tags/ids/classes/data attributes
- Markdown sections/checklists/links
- JSON/TOML/YAML keys/sections/tables only where useful
- no noisy scalar-value symbol flood

Real-repo expected-symbol manifests:
Extend `test_repos/MANIFEST.toml` support if needed.

Support explicit expected symbols:

[[repo.expected_symbol]]
language = "rust"
path = "src/indexer.rs"
kind = "function"
name = "build_index"

Support expected symbol patterns:

[[repo.expected_symbol_pattern]]
language = "typescript"
path_glob = "src/**/*.ts"
kind = "function"
name_regex = "^[A-Za-z_].*"
min_count = 20

Use expected symbols/patterns to catch missed parser output in real repos.

Do not rely on fragile exact total record counts.

Failure policy:
Fail coverage closure if:
- an expected symbol is missing
- an expected symbol pattern falls below its minimum
- an obvious top-level declaration is missed in a supported language fixture
- a supported language emits zero useful records on real files without a documented reason
- comments/strings emit fake code symbols
- duplicate path+line+col records occur
- records contain `source = "ctags"`
- parser panics on supported-language files
- malformed records or refs are emitted

Allowed non-failures:
- unsupported language/extension, if documented
- generated/vendor/minified files, if ignored or documented
- framework/semantic relationships that require type resolution, if documented
- macro-generated symbols, if documented
- language syntax explicitly marked unsupported with a blocker

Conformance backfill:
Every fix for a missed real-repo symbol must add a fixture/conformance case where practical.

Do not only fix the specific repo file. Add a generalized fixture for the syntax pattern.

Parser diagnostics:
Improve failure output so missed-symbol and parser failures show:
- repo
- language
- path
- expected symbol or pattern
- record kind
- relevant query/capture if available
- suspected grammar/query gap if known

Docs:
Update parser support docs with:
- supported language matrix
- coverage confidence per language if useful
- known unsupported syntax
- unsupported extensions
- real-repo hardening status
- symbol coverage policy
- how to add expected symbols/patterns to `test_repos/MANIFEST.toml`
- statement that Tree-sitter is syntax-tree based, not semantic/LSP-level analysis

Tests:
- add or update conformance tests for every supported language
- add expected-symbol manifest tests
- add expected-symbol-pattern tests
- add missed-symbol fixture cases from real repos
- keep shared assertion logic centralized
- no duplicate per-language assertion copy/paste
- no ctags skip logic
- no line-scanner fallback

Acceptance:
- supported languages have broad construct coverage fixtures
- `test_repos/` expected symbols/patterns are checked where present
- no known supported-language symbol miss remains unaddressed or undocumented
- every fixed real-repo parser miss has fixture coverage where practical
- parser support matrix is honest
- no second parser architecture is introduced
- no ctags or line-scanner code parser backend is reintroduced
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
- `cargo test --test real_repos -- --ignored`

Report:
- changed files
- supported-language coverage closure summary
- expected symbols checked/missing
- expected symbol patterns checked/missing
- missed-symbol failures fixed
- fixture/conformance cases added
- remaining unsupported syntax and why
- parser/grammar license changes, if any
- verification commands and results
- ignored local/real repo test status
- commit hash
