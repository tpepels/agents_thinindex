# File References

thinindex stores best-effort file-to-file relationships in SQLite `file_references`. These edges are separate from symbol `refs`: they record that one file names another file, asset, fixture, template, config path, or entrypoint.

Each row stores the source file, line, column, raw target string, resolved local target when one exists, reference kind, language or format, confidence, unresolved reason, evidence, and extractor source.

## Reference Kinds

- `import`: local module imports from the dependency graph plus bounded local JS/TS dynamic imports and Python relative imports.
- `export`: local JS/TS re-export edges such as `export * from "./module"`.
- `include`: C/C++ and similar include relationships.
- `require`: local require/include-style relationships.
- `source`: shell source/dot references.
- `link`: local document links.
- `asset`: images, fonts, HTML `srcset`, CSS `url(...)`, and other static assets.
- `script`: HTML script references.
- `stylesheet`: HTML stylesheet references and CSS/SCSS `@import` paths.
- `config_path`: explicit path-like config values.
- `package_entry`: package/build entrypoints and file lists.
- `fixture`: test fixture paths.
- `unknown`: reserved for future best-effort extractors that cannot classify a path.

## Resolution

Resolution is local and deterministic:

- relative paths resolve from the source file directory;
- extensionless imports use practical local candidates and directory indexes;
- bare HTML/CSS/Markdown filenames resolve relative to the source file;
- package/config paths with directories resolve from the repository root;
- project-file entries such as `.csproj` `Include` paths resolve relative to the project file;
- Markdown and similar targets with query strings or fragments preserve the raw target but resolve the file portion;
- unresolved local-looking paths remain in the table with `target_not_found`, `ambiguous_match`, or `absolute_path`;
- external URLs, anchors, `mailto:`, `data:`, package names, system includes, template expressions, variables, route-like URIs, and version strings are not stored as local file references.

No network access, package-manager execution, compiler, LSP, or broad semantic analysis is used.

## Bounds And Noise Controls

File-reference extraction stores one file-level edge per import/export/include
statement rather than one row per imported symbol. Rows are deduplicated by
source path, raw target, resolved target, and kind. A single source file can
emit at most 96 file-reference rows, and a single source file/kind pair can emit
at most 64 rows. `build_index --stats` reports file-reference totals, counts by
kind, unresolved reasons, top source files by file-reference count, and cap
warnings with the source file, kind, kept count, dropped count, and violated
cap.

Normal builds exclude `.dev_index/` and `test_repos/` and do not run package
managers or fetch package metadata. External packages and system includes stay
in the dependency graph when useful for diagnostics, but they are not copied
into `file_references` as local file edges.

## CLI Impact

`wi refs <term>` can show `file_<kind>` rows when a file references a primary result file. `wi pack <term>` uses forward and reverse file references to include related assets, docs, configs, fixtures, and dependent files. `wi impact <term>` uses reverse file references as evidence for affected files.

File-reference confidence labels are:

- `resolved`: exactly one local target file matched;
- `ambiguous`: multiple local target files matched;
- `unresolved`: the target looked local but could not be resolved.

Context commands translate these to user-facing labels such as `dependency`, `test-related`, or `heuristic` based on kind and file role.

## Known Limits

Config and package scanning is intentionally allowlisted and may miss project-specific keys. File references do not prove runtime behavior. Dynamic route construction, generated paths, package-manager aliases, webpack/Vite aliases, compiler path mapping, framework-specific asset pipelines, and root-relative web paths are not resolved unless they appear as explicit local paths. JS/TS dynamic imports are recorded only when the argument is a literal local path.
