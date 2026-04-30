# Dependency Graph

thinindex builds a local, best-effort module/dependency graph alongside `records` and `refs` in `.dev_index/index.sqlite`.

The graph is internal foundation data for refs, pack, and impact improvements. It is not a semantic compiler graph and does not make `wi refs`, `wi pack`, or `wi impact` exhaustive semantic output by itself.

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

Resolved edges use `confidence = "resolved"`. Unresolved edges use `confidence = "unresolved"` and keep an explicit reason such as `target_not_found`, `external_package`, `system_include`, or `absolute_path`. Ambiguous local matches use `confidence = "ambiguous"`, leave `target_path` empty, and store `unresolved_reason = "ambiguous_match"` instead of choosing a target silently.

## Resolver Support Matrix

The resolver packs are best-effort and local-only:

- Rust: `mod`, `pub mod`, and `use crate::`, `use self::`, `use super::`, plus unresolved external `use` roots.
- Python: `import` and `from ... import ...` modules, including same-directory, repository-root, and common `src`/`lib`/`app` layouts.
- JavaScript/TypeScript/JSX/TSX: relative imports and package self-imports from root `package.json` `name`.
- Go: relative imports, module imports using root `go.mod`, and deterministic local package suffix matches.
- Java/Kotlin/Scala: dotted imports mapped to common JVM source roots and package path suffixes.
- C#: `using` directives mapped by namespace path when a unique local `.cs` file exists.
- C and C++: quoted includes resolved relative to the source file and then by unique local include suffix; known system headers remain unresolved as system includes.
- Ruby: `require_relative`, relative `require`, and unique repository-root local `require` paths.
- PHP: include/require imports from Tree-sitter records, resolved relative to the source file or repository root.
- Shell: `source` and dot includes resolved relative to the source file or repository root; absolute paths remain unresolved.
- Nix: local `import ./...` and `import ../...` paths are recorded as config-format dependency edges.

Package-manager, compiler, LSP, network, and external environment resolution are intentionally not part of this phase.

## Confidence Levels

- `resolved`: exactly one local file matched the resolver rules.
- `ambiguous`: more than one local file matched at the same resolver priority, so no target is chosen.
- `unresolved`: no local target was found, the import is external, the import is a known system include, the path is absolute, or the language is unsupported.

Resolvers use priority tiers where the ecosystem has a practical local precedence. For example, same-directory Python modules are checked before repository-level source roots. Ambiguity is reported only within the first matching priority tier.

## Unresolved Imports

Unresolved imports are expected and preserved. Examples include standard libraries, package-manager dependencies, system headers, absolute paths, missing files, and syntax that needs a future resolver. External imports are not hidden just because no local target exists.

Do not silently drop unresolved imports just because they cannot be mapped to a local file. The unresolved edge is evidence for future dependency-aware context and impact behavior.

Ambiguous imports are also preserved. They mean thinindex found multiple plausible local files and is intentionally refusing to guess.

## Known Resolver Gaps

- Resolver packs do not invoke package managers or read installed dependency metadata.
- TypeScript `paths`, Babel aliases, Python virtual environments, Ruby `$LOAD_PATH`, PHP autoloaders, JVM build tool metadata, C/C++ compiler include paths, and Swift package/module metadata are not compiler-complete.
- Scala wildcard and grouped imports are conservative; grouped imports with braces are not expanded.
- Nix dependency edges cover local path imports only.
- C# namespace resolution maps to local paths when practical but does not model assemblies or project references.

## Boundaries

- The dependency graph is local and deterministic.
- It does not install packages or download metadata.
- It does not invoke compilers, LSP servers, or package managers.
- It does not claim semantic type resolution.
- It is separate from parser records and from deterministic references.
- Current CLI output remains stable until a later plan explicitly uses dependency edges in user-facing commands.
