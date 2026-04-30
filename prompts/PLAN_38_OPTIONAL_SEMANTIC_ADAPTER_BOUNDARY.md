# PLAN_38_OPTIONAL_SEMANTIC_ADAPTER_BOUNDARY.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_37_MONOREPO_SCALE_AND_INCREMENTAL_INDEXING.md is complete and green.

Progress:
- [x] Phase 1: inspect reference, impact, indexer, and SQLite storage boundaries
- [x] Phase 2: add semantic fact model and optional adapter registry
- [x] Phase 3: store semantic facts in isolated SQLite table with schema bump
- [x] Phase 4: add fake adapter tests for success, missing/failing adapters, and record isolation
- [x] Phase 5: document optional semantic adapter boundary and future adapter placeholders
- [x] Phase 6: run required verification and commit completed Plan 38 work

Goal:
Add an optional semantic-adapter plugin boundary for future LSP/compiler integrations without making them required product dependencies.

Product rule:
Compiler/LSP integrations must be optional plugins. The local Tree-sitter index remains the baseline.

Scope:
Create the boundary only. Do not integrate every LSP yet.

Required:
- Define `SemanticAdapter` or equivalent trait/interface.
- Define semantic facts model:
  - resolved definition
  - resolved reference
  - type reference
  - call target
  - implementation relationship
  - diagnostic if available
- Keep semantic facts separate from parser records.
- Store semantic facts only if adapter runs successfully.
- Make adapters optional and disabled by default.
- No network access.
- No telemetry.
- No required external tools in normal tests.

Initial adapters:
Add fake/test adapter only, plus optional placeholder docs for:
- rust-analyzer
- pyright/jedi
- tsserver
- gopls
- clangd
- jdtls
- omnisharp/csharp-ls

Do not implement real adapters in this plan unless trivial.

Tests:
- fake adapter can add semantic facts
- missing adapter is skipped cleanly
- semantic facts do not pollute parser records
- normal tests require no external tools
- pack/impact can ignore semantic facts if absent

Docs:
Document:
- Tree-sitter is syntax layer
- semantic adapters are optional
- future adapter requirements
- adapters are not bundled by default

Acceptance:
- semantic plugin boundary exists
- no required LSP/compiler dependencies
- semantic data path is isolated
- existing parser/index behavior remains stable

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run --bin build_index`
- `cargo run --bin wi -- pack build_index`
- `cargo run --bin wi -- impact build_index`

Report:
- changed files
- semantic adapter boundary
- semantic fact model
- tests added
- verification results
- commit hash
