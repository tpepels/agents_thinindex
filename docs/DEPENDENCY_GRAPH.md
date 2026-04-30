# Dependency Graph

thinindex builds a local, best-effort module/dependency graph alongside `records` and `refs` in `.dev_index/index.sqlite`.

The graph is internal foundation data for future refs, pack, and impact improvements. It is not a semantic compiler graph and does not change current `wi refs`, `wi pack`, or `wi impact` output by itself.

## Data Model

Each dependency edge records:

- source file path
- source line and column
- import/module string as written or normalized from the import record
- target file path when it can be resolved locally
- dependency kind
- language
- confidence
- unresolved reason when no target file is found
- compact evidence line
- extraction source

Resolved edges use `confidence = "resolved"`. Unresolved edges use `confidence = "unresolved"` and keep an explicit reason such as `target_not_found`, `external_package`, `system_include`, or `absolute_path`.

## Supported Ecosystems In This Foundation

The first dependency graph pass records local relationships for:

- Rust `mod` and local `use crate::` / `use super::` statements
- Python `import` and `from ... import ...` modules
- JavaScript/TypeScript relative imports
- Go imports, including local package suffix matches when files exist
- Java imports with package-to-path resolution when source files exist
- C and C++ includes
- Ruby `require_relative` and local `require`
- PHP includes/requires from Tree-sitter import records
- Shell `source` and dot includes

Package-manager, compiler, LSP, network, and external environment resolution are intentionally not part of this phase.

## Unresolved Imports

Unresolved imports are expected and preserved. Examples include standard libraries, package-manager dependencies, system headers, absolute paths, missing files, and syntax that needs a future resolver.

Do not silently drop unresolved imports just because they cannot be mapped to a local file. The unresolved edge is evidence for future dependency-aware context and impact behavior.

## Boundaries

- The dependency graph is local and deterministic.
- It does not install packages or download metadata.
- It does not invoke compilers, LSP servers, or package managers.
- It does not claim semantic type resolution.
- It is separate from parser records and from deterministic references.
- Current CLI output remains stable until a later plan explicitly uses dependency edges in user-facing commands.
