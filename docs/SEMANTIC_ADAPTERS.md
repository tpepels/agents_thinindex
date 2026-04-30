# Semantic Adapters

thinindex now has an optional semantic-adapter boundary for future compiler and language-server integrations. The default product path remains Tree-sitter plus deterministic local extraction.

No semantic adapter is bundled or required by default. Normal `build_index`, `wi`, `wi pack`, and `wi impact` work without external tools, network access, telemetry, daemons, or compiler/LSP startup.

## Layering

- Tree-sitter and extras produce parser records in SQLite `records`.
- Deterministic refs and dependency extraction produce `refs` and `dependencies`.
- Optional semantic adapters may produce `semantic_facts`.

Semantic facts are stored separately from parser records. They must not be written into `records`, and adapter failures must not break normal indexing.

## Fact Model

Semantic facts can describe:

- `resolved_definition`
- `resolved_reference`
- `type_reference`
- `call_target`
- `implementation`
- `diagnostic`

Each fact stores a source location, symbol, optional target location, optional detail, confidence, and adapter name.

## Adapter Boundary

Future adapters implement the `SemanticAdapter` trait. An adapter must:

- declare a stable adapter name;
- report whether its external tool is available;
- collect facts from local repository state only;
- return no facts when unavailable;
- fail closed without polluting parser records;
- avoid network access and telemetry.

The test-only static adapter exists to prove the data path. Real adapters remain placeholders until separate plans define their tool invocation, timeout, cache, licensing, and failure policy.

## Future Adapter Candidates

- Rust: `rust-analyzer`
- Python: `pyright` or `jedi`
- JavaScript/TypeScript: `tsserver`
- Go: `gopls`
- C/C++: `clangd`
- Java: `jdtls`
- C#: `omnisharp` or `csharp-ls`

These tools are not bundled by default and are not required for normal tests.
