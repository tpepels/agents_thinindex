# Impact Analysis

`wi impact <term>` returns a compact, evidence-backed set of files to inspect before editing a symbol or feature area. It is deterministic and local, but it is not an exhaustive semantic impact engine.

Impact rows come from SQLite `records`, `refs`, `dependencies`, and `file_references`. A row is included only when thinindex can attach a concrete file:line reason from indexed evidence.

File-role classification controls how source, test, build, package manifest, config, docs, generated, and vendor paths are mapped into impact groups. See [FILE_ROLES.md](FILE_ROLES.md).

## Output Groups

- `Direct definitions`: primary search results for definitions or structured landmarks.
- `References`: syntax, import, type, export, call, text, or file-reference evidence to the primary names/files.
- `Dependent files`: files whose local dependency edge resolves to a primary definition file.
- `Likely tests`: test-path references, test dependency edges, and same-name test conventions.
- `Related docs`: Markdown and documentation-path references.
- `Build/config files`: config, package/build, route, schema, JSON, TOML, YAML, and similar path references.
- `Unresolved/unknown areas`: unresolved dependency or file-reference evidence and low-confidence fixture/example evidence.

Rows are deduplicated by file across non-primary groups. When a file has multiple possible reasons, the earlier, stronger group wins.

## Confidence Labels

- `direct`: exact primary definitions or local syntax/import references.
- `dependency`: local dependency graph or import-like file-reference evidence resolved to a repository file.
- `test-related`: test path, test reference, test dependency, or same-name test convention.
- `semantic`: internal/reserved only; current `wi impact` output does not consume `semantic_facts`.
- `heuristic`: capped text fallback, docs/config/file references, unresolved imports, fixtures, examples, or other best-effort evidence.

Every output row includes a reason string. The reason explains the indexed evidence, not a guarantee that editing the primary symbol will break the impacted file.

## Test Mapping

Test mapping combines:

- explicit references from files under common test paths or test-like filenames;
- dependency edges from test files to primary definition files;
- resolved import/export/include file references plus tests, docs, config, packages, and assets to primary files;
- same-name conventions between primary file/symbol names and test file paths.

This catches common repository layouts without invoking a test runner, package manager, language server, or network service.

## Improving Impact Quality

Impact quality improves when the index has better local evidence:

- keep imports/includes/requires resolvable through local files;
- prefer explicit local import/export/include paths over package aliases when possible;
- keep tests near conventional `tests/`, `test/`, `__tests__/`, `*_test`, `.test.`, or `.spec.` paths;
- add or improve deterministic Tree-sitter query captures for precise references;
- add future manifest data only when it can produce concrete file:line evidence and clear confidence labels.

Known limits: dynamic dispatch, generated code, inheritance, overloads, macro expansion, runtime routing, package-manager resolution, and LSP/compiler-level semantics are not claimed. Semantic facts are internal/deferred, optional adapters are disabled by default, and current impact output does not consume them; see [SEMANTIC_ADAPTERS.md](SEMANTIC_ADAPTERS.md).
