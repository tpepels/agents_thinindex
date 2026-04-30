# Parser Maintenance Guide

This guide is for maintainers changing Tree-sitter query specs or adding parser-backed language support. Query changes should be small, fixture-backed, license-audited, and easy to review.

Universal Ctags is optional, external, not bundled, not required, and not used by production indexing. It must not become a parser fallback or production index source.

## Parser Architecture Overview

Production code-symbol extraction runs through `src/tree_sitter_extraction.rs`:

- `LanguageRegistry` owns grammar adapters and extension routing.
- Each `GrammarAdapter` declares a language id, display name, file extensions, Tree-sitter grammar function, query spec, and license metadata.
- `TreeSitterExtractionEngine` selects an adapter by file extension, parses the file with Tree-sitter, compiles the adapter query, validates capture names, and sends matches through `CaptureMapper`.
- `CaptureMapper` turns normalized captures into `IndexRecord` values with `source = "tree_sitter"`.
- Project-owned extras handle CSS, HTML, Markdown, JSON, TOML, and YAML landmarks. Extras are not Tree-sitter code-symbol support.

Do not create a second parser architecture for code symbols.

## How LanguageRegistry Works

`LanguageRegistry::default()` is the production registry. A language is active only when its adapter is registered there.

An adapter must include:

- stable `id` used as `IndexRecord.lang`
- human `display_name`
- extension list without leading dots
- grammar loader
- query spec
- `LicenseEntry`

The support matrix in `src/support.rs` must have a matching Tree-sitter entry for each registered adapter. The matrix uses extensions with leading dots for documentation, but it must describe the same file types.

## How Query Specs Work

Query specs are inline constants in `src/tree_sitter_extraction.rs`. Each record-producing query pattern should capture:

- one symbol name with `@name`
- one record kind with `@definition.<kind>`

Auxiliary captures used only by predicates must be named `@internal.<purpose>`. They do not produce records.

Keep query specs conservative. Prefer missing a difficult construct over indexing noisy comments, strings, broad expression fragments, or synthetic names.

## Normalized Capture Names

Allowed definition captures are:

- `@definition.class`
- `@definition.constant`
- `@definition.constructor`
- `@definition.enum`
- `@definition.export`
- `@definition.field`
- `@definition.function`
- `@definition.import`
- `@definition.interface`
- `@definition.macro`
- `@definition.method`
- `@definition.module`
- `@definition.namespace`
- `@definition.object`
- `@definition.property`
- `@definition.struct`
- `@definition.trait`
- `@definition.type`
- `@definition.variable`

Normalized record kinds are:

- `class`
- `constant`
- `enum`
- `export`
- `function`
- `import`
- `interface`
- `method`
- `module`
- `struct`
- `trait`
- `type`
- `variable`

The explicit aliases are:

- `field` -> `variable`
- `macro` -> `function`
- `namespace` -> `module`
- `constructor` -> `method`
- `object` -> `module`
- `property` -> `variable`

Unsupported captures must fail validation instead of silently creating a new record kind.

## Capture-To-Record Mapping Rules

For each query match, `CaptureMapper` uses the first `@definition.<kind>` capture and the `@name` capture from the same match.

Record fields are mapped as follows:

- `path`: relative path being parsed
- `line` and `col`: one-based start position of `@name`
- `lang`: adapter id
- `kind`: normalized definition capture kind
- `name`: trimmed `@name` text
- `text`: source line containing the symbol name, truncated by `IndexRecord`
- `source`: `tree_sitter`

Records are sorted and deduplicated by path, line, column, kind, and name. Query edits must preserve deterministic output.

## How To Add A Language

Adding a language means adding all of these in one reviewed change:

- permissively licensed Tree-sitter grammar dependency in `Cargo.toml`
- license entry in `THIRD_PARTY_NOTICES`
- `LicenseEntry` in the new `GrammarAdapter`
- extension mapping in `LanguageRegistry::default()`
- query spec using only allowed captures
- conformance fixture under `tests/fixtures/language_pack`
- expected symbols in `tests/parser_conformance.rs`
- support matrix entry in `src/support.rs`
- docs row in `README.md` and `docs/PARSER_SUPPORT.md`
- generated support dashboard refresh when the matrix changes

If any item is blocked, document the language as `blocked` or `experimental`; do not claim full support.

## How To Update A Language Query

Before changing a query:

- run `cargo test parser_conformance`
- inspect fixture records for the language being changed
- update or add fixture syntax that proves the behavior
- keep captures within the allowed set
- avoid adding broad patterns that match comments, strings, or arbitrary identifiers
- check output ordering and duplicate behavior

After changing a query, update the expected symbols and known gaps if behavior changed.

## How To Add Conformance Fixtures

Fixtures should be small, source-controlled examples under `tests/fixtures/language_pack`. A fixture should include:

- expected symbols for supported declarations
- at least one comment or string fake that must stay absent when practical
- representative import/export/module syntax when supported by the query
- unsupported syntax notes in the conformance case when behavior is intentionally absent

Normal tests must not depend on `test_repos/`.

## How To Add Real-Repo Expected Symbols

Real-repo expectations live in local `test_repos/MANIFEST.toml` and are checked by ignored tests. Keep third-party repos local and uncommitted.

For each curated repo entry, prefer:

- name, path, kind, languages, and queries
- expected paths for files that should be indexed
- expected symbols or expected symbol patterns
- expected absent symbols for known false-positive risks
- notes for generated, vendor, or unsupported-heavy areas

Real-repo failures should improve fixtures or queries only when the behavior is deterministic and precise.

## How To Run Quality Gates

Run these before committing parser or query changes:

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo deny check licenses
cargo run --bin build_index
cargo test --test local_index -- --ignored
cargo test --test real_repos -- --ignored
```

If `test_repos/` is absent or intentionally empty, the ignored real-repo gate may skip. If `cargo-deny` is unavailable locally, install it before release-quality verification or report the missing tool clearly.

## How To Handle Unsupported Syntax

Unsupported syntax should be visible and honest:

- document it in `known_gaps`
- add an absent-symbol assertion when the risk is false positives
- keep blocked languages visible in support docs
- prefer no record over a noisy low-confidence record
- do not promote `experimental` entries to `supported` until fixture, license, docs, and real-repo gates are ready

Do not hide unsupported syntax behind broad fallback extraction.

## How To Audit Grammar Licenses

Every grammar dependency must have:

- package metadata in `Cargo.lock`
- an allowed license expression in `deny.toml`
- notice text in `THIRD_PARTY_NOTICES`
- matching `LicenseEntry` metadata in the adapter
- support matrix license status

Run `cargo deny check licenses` after adding or updating grammar dependencies.

## What Not To Do

Forbidden patterns:

- no line scanners for code symbols
- no hand parsers
- no ctags parser fallback
- no broad regex parser
- no unsupported language support claims
- no grammar dependency without license entry

Also avoid:

- adding parser support without conformance fixtures
- adding support rows without registered adapters
- indexing every identifier as a symbol
- changing `IndexRecord.kind` values without updating capture validation, docs, fixtures, and compatibility tests
- treating extras-backed format landmarks as Tree-sitter-backed code-symbol extraction
