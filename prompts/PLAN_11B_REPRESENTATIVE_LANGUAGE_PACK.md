# PLAN_11B_REPRESENTATIVE_LANGUAGE_PACK.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_11A is complete and green.

Goal:
Add a representative Tree-sitter language pack through the generic framework.

This pass proves generality across language families. Do not write hand parsers. Do not add release packaging, license enforcement, payment behavior, telemetry, cloud behavior, or new product commands.

Product rule:
Generality is the product. Supported languages must be added through the same registry + grammar + query + capture-mapper + conformance-fixture path.

Representative language pack:
Add support for:
- Rust
- Python
- JavaScript
- TypeScript
- JSX
- TSX
- Java
- Go
- C
- C++
- Shell
- Ruby
- PHP

If one language lacks a permissively licensed Tree-sitter grammar or usable Rust integration, do not silently replace it with line scanning. Mark it unsupported with a clear blocker and continue only if the plan allows partial completion with documented gaps.

Hard requirements:
- use Tree-sitter AST/query extraction for code-symbol languages
- no line-oriented or regex-based code-symbol parser
- no ctags
- no GPL/AGPL dependencies
- every grammar/dependency must have a license entry
- every supported language must have a conformance fixture
- no newly built index may emit `source = "ctags"`

Per-language artifacts:
For each supported language add:
- grammar registration
- extension mapping
- Tree-sitter query spec
- conformance fixture
- license/audit entry
- expected record-kind coverage note

Common record kinds:
Where language syntax supports them, extract:
- function
- method
- class
- struct
- enum
- interface
- trait
- type
- module
- variable
- constant
- import
- export

JSX/TSX:
- JSX/TSX component-like definitions should be extracted where practical.
- JSX/TSX component usage can be extracted through Tree-sitter queries or existing accepted extras, but code-symbol definitions must not be line-scanned.

Shell:
- extract functions and sourced/import-like constructs where practical.
- do not overfit to one shell dialect unless documented.

Ruby/PHP:
- extract classes, modules/namespaces where applicable, methods/functions, constants where practical, imports/includes where practical.

Tests:
- each supported language has at least one fixture indexed through the shared conformance suite
- multiline definitions are included for representative languages
- comments/strings do not create fake symbols
- line/col are 1-based and accurate
- no duplicate path+line+col records
- no `source = "ctags"`
- `wi` can find representative symbols from several language groups
- existing refs/pack/impact/stats tests pass

Docs:
- update parser support matrix
- document unsupported/deferred languages honestly
- update THIRD_PARTY_NOTICES or equivalent for all parser/grammar dependencies
- do not claim semantic/LSP-level analysis

Acceptance:
- representative language pack is implemented through the generic framework
- unsupported languages are documented with explicit blockers, not silently faked
- no ctags or line-scanner parser backend remains for supported code languages
- no GPL/AGPL dependency is introduced
- existing product behavior remains stable

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run --bin build_index`
- representative `wi` commands for Rust, Python, Java, Go, JS/TS, C/C++, Ruby, PHP, and Shell
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
