# Reference Graph

thinindex stores references in SQLite `refs` as evidence-backed relationships from a source location to a target name or file.

Reference extraction is intentionally conservative. It combines Tree-sitter reference captures, dependency graph edges, structured document/style references, and a capped broad text fallback. It does not claim full semantic type, call, overload, inheritance, or runtime module resolution.

## Reference Kinds

- `import`: syntactic import/use/export-from references from existing import extraction.
- `export`: Tree-sitter export references where query specs capture them.
- `call`: Tree-sitter call-expression references where query specs capture them.
- `type_reference`: Tree-sitter type references when a query pack captures them.
- `module_dependency`: dependency graph evidence for resolved local files or unresolved external modules.
- `test_reference`: capped text references from test paths.
- `text_reference`: capped broad text fallback outside test paths.
- `markdown_link`: structured Markdown links.
- `css_usage`: CSS class and custom property usage.
- `html_usage`: HTML/JSX id, class, and data attribute usage.

## Confidence Labels

- `exact_local`: a syntax/import reference target matches a local Tree-sitter symbol name.
- `syntax`: the reference came from syntax captures or structured format parsing.
- `dependency`: the reference came from a resolved dependency graph edge.
- `heuristic`: the reference came from capped broad text fallback.
- `unresolved`: the reference came from an unresolved dependency graph edge.

Every ref row stores both `confidence` and `reason`. The reason is compact evidence for why the confidence label was assigned, such as `tree_sitter_reference_capture`, `local_symbol_match`, `dependency_graph_resolved_file`, or `broad_text_fallback`.

## Syntax vs Semantic References

Syntax references are AST-backed observations that a token appears in a call, export, import, or type-like position. They do not prove that the token resolves to a specific declaration.

Semantic references require compiler, language-server, package-manager, or build-system information. The baseline index does not perform that work. Optional semantic adapters may write isolated `semantic_facts`; they are not copied into `refs` or `records` by default. When a syntax reference target name matches a local symbol, thinindex labels it `exact_local`, but that remains a local name match rather than full compiler resolution.

## Dependency-backed References

The dependency graph contributes `module_dependency` rows:

- resolved local imports point at the target file path with `confidence = "dependency"`;
- unresolved external or missing imports keep the import string with `confidence = "unresolved"`;
- ambiguity remains explicit in the dependency graph and is not guessed into a target file.

These rows give future context and impact commands a relationship-backed way to reason about file/module connections without hiding unresolved imports.

## Fallback Behavior

The broad text fallback remains capped by per-target, per-file, and total-reference limits. Its rows use `confidence = "heuristic"` and a reason that makes the fallback explicit. This keeps compatibility for docs/tests/simple usage while preventing the graph from pretending text matches are semantic references.

## Known Limits

- No compiler, LSP, type checker, package manager, or network resolution is invoked.
- Method calls, dynamic calls, overloads, inheritance, traits/interfaces, generated code, aliases, and re-exports are not semantically resolved.
- Tree-sitter call/type coverage depends on each language query pack and is deliberately conservative.
- `module_dependency` rows identify module/file relationships, not the exact imported symbol inside the file.
