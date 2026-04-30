# Language Support Dashboard

Generated from the source-controlled support matrix in `src/support.rs`. Do not edit support rows by hand; update the matrix and regenerate this document through the stale-check test.

- last generated: deterministic
- source of truth: `src/support.rs::support_matrix()`
- quality reports: `.dev_index/quality/QUALITY_REPORT.md`, `.dev_index/quality/QUALITY_REPORT.json`, and `.dev_index/quality/QUALITY_REPORT_DETAILS.jsonl` when local quality workflows run
- verification note: conformance uses checked fixtures; real-repo and comparator status are local quality signals and are not semantic/LSP-level analysis

## Summary

| Level | Count |
| --- | ---: |
| supported | 14 |
| experimental | 5 |
| blocked | 7 |
| extras-backed | 6 |

## Support Levels

- `supported`: grammar/query/fixture/license/docs exist; conformance passes; real-repo checks pass where configured.
- `experimental`: grammar/query exists, but conformance or real-repo coverage is incomplete.
- `blocked`: missing permissive grammar, broken integration, unclear license, or unacceptable parser quality.
- `extras-backed`: project-owned extras intentionally handle deterministic format landmarks instead of Tree-sitter.

## Dashboard

| Language/format | Extensions | Level | Backend | Record kinds | Known gaps | License status | Conformance status | Real-repo status | Expected-symbol coverage | Comparator status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| C | .c, .h | supported | tree_sitter | function, struct, enum, type, variable, import | No macro expansion, preprocessor configuration, or compile database semantics. | MIT grammar notice | fixture declared; normal conformance required | checked by ignored real-repo gate when configured | manifest expected symbols/patterns checked when configured | optional local comparator report when available |
| C# | .cs | supported | tree_sitter | method, class, struct, enum, interface, type, module, variable, import | No partial-type, assembly, or Roslyn-level resolution. | MIT grammar notice | fixture declared; normal conformance required | checked by ignored real-repo gate when configured | manifest expected symbols/patterns checked when configured | optional local comparator report when available |
| C++ | .cc, .cpp, .cxx, .hh, .hpp, .hxx | supported | tree_sitter | function, method, class, struct, enum, type, module, variable, import | No template instantiation, macro expansion, or compile database semantics. | MIT grammar notice | fixture declared; normal conformance required | checked by ignored real-repo gate when configured | manifest expected symbols/patterns checked when configured | optional local comparator report when available |
| Go | .go | supported | tree_sitter | function, method, struct, interface, type, module, variable, constant, import | No semantic exported API set or module graph resolution. | MIT grammar notice | fixture declared; normal conformance required | checked by ignored real-repo gate when configured | manifest expected symbols/patterns checked when configured | optional local comparator report when available |
| Java | .java | supported | tree_sitter | method, class, enum, interface, type, variable, import | No package visibility, inherited member, or build-system classpath resolution. | MIT grammar notice | fixture declared; normal conformance required | checked by ignored real-repo gate when configured | manifest expected symbols/patterns checked when configured | optional local comparator report when available |
| JavaScript | .js | supported | tree_sitter | function, method, class, variable, import, export | No runtime module, prototype, or bundler resolution. | MIT grammar notice | fixture declared; normal conformance required | checked by ignored real-repo gate when configured | manifest expected symbols/patterns checked when configured | optional local comparator report when available |
| JSX | .jsx | supported | tree_sitter | function, method, class, variable, import, export | Definition extraction is Tree-sitter-backed; element usage remains deterministic reference evidence. | MIT grammar notice | fixture declared; normal conformance required | checked by ignored real-repo gate when configured | manifest expected symbols/patterns checked when configured | optional local comparator report when available |
| PHP | .php | supported | tree_sitter | function, method, class, interface, trait, enum, module, variable, constant, import | No dynamic include, autoload, or runtime namespace resolution. | MIT grammar notice | fixture declared; normal conformance required | checked by ignored real-repo gate when configured | manifest expected symbols/patterns checked when configured | optional local comparator report when available |
| Python | .py | supported | tree_sitter | function, method, class, variable, import | Syntactic extraction only; decorators and complex assignment targets are not semantic analysis. | MIT grammar notice | fixture declared; normal conformance required | checked by ignored real-repo gate when configured | manifest expected symbols/patterns checked when configured | optional local comparator report when available |
| Ruby | .rb | supported | tree_sitter | method, class, module, constant | No require/load target or metaprogramming resolution. | MIT grammar notice | fixture declared; normal conformance required | checked by ignored real-repo gate when configured | manifest expected symbols/patterns checked when configured | optional local comparator report when available |
| Rust | .rs | supported | tree_sitter | function, struct, enum, trait, type, module, constant, variable | use records are deferred to deterministic refs; no macro expansion or type resolution. | MIT grammar notice | fixture declared; normal conformance required | checked by ignored real-repo gate when configured | manifest expected symbols/patterns checked when configured | optional local comparator report when available |
| Shell | .sh, .bash | supported | tree_sitter | function, variable | Sourced files and shell runtime expansion are not resolved. | MIT grammar notice | fixture declared; normal conformance required | checked by ignored real-repo gate when configured | manifest expected symbols/patterns checked when configured | optional local comparator report when available |
| TSX | .tsx | supported | tree_sitter | function, method, class, interface, type, variable, import, export | Definition extraction is Tree-sitter-backed; element usage remains deterministic reference evidence. | MIT grammar notice | fixture declared; normal conformance required | checked by ignored real-repo gate when configured | manifest expected symbols/patterns checked when configured | optional local comparator report when available |
| TypeScript | .ts | supported | tree_sitter | function, method, class, interface, type, variable, import, export | No type alias, generic constraint, or project graph resolution. | MIT grammar notice | fixture declared; normal conformance required | checked by ignored real-repo gate when configured | manifest expected symbols/patterns checked when configured | optional local comparator report when available |
| Dart | .dart | experimental | tree_sitter | function, method, class, enum, type, variable, constant, import, export | Conformance exists, but real-repo coverage and package/extension/type-alias handling remain incomplete. | MIT grammar notice | fixture declared; coverage incomplete | local real-repo hardening incomplete | manifest expected symbols/patterns checked when configured | optional local comparator report when available |
| Kotlin | .kt, .kts | experimental | tree_sitter | function, class, enum, type, module, variable, import | Conformance exists, but real-repo coverage and interface/enum-class/extension distinctions remain incomplete. | MIT grammar notice | fixture declared; coverage incomplete | local real-repo hardening incomplete | manifest expected symbols/patterns checked when configured | optional local comparator report when available |
| Nix | .nix | experimental | tree_sitter | function, module, import | Conformance exists, but real-repo coverage and exhaustive attr/scalar extraction remain incomplete by design. | MIT grammar notice | fixture declared; coverage incomplete | local real-repo hardening incomplete | manifest expected symbols/patterns checked when configured | optional local comparator report when available |
| Scala | .scala | experimental | tree_sitter | function, class, enum, trait, type, module, variable, constant, import | Conformance exists, but real-repo coverage and givens/implicits/extension handling remain incomplete. | MIT grammar notice | fixture declared; coverage incomplete | local real-repo hardening incomplete | manifest expected symbols/patterns checked when configured | optional local comparator report when available |
| Swift | .swift | experimental | tree_sitter | function, method, class, struct, enum, interface, type, variable, import | Conformance exists, but real-repo coverage and extension/overload/module handling remain incomplete. | MIT grammar notice | fixture declared; coverage incomplete | local real-repo hardening incomplete | manifest expected symbols/patterns checked when configured | optional local comparator report when available |
| CSS | .css | extras-backed | extras | css_class, css_id, css_variable, keyframes | Selectors and keyframes only; no cascade or browser semantics. | project-owned extras; no third-party parser dependency | fixture declared for extras landmarks | checked by fixture/real-repo extras coverage when present | extras landmarks checked by fixtures and quality reports | not a Tree-sitter comparator claim |
| HTML | .html | extras-backed | extras | html_tag, html_id, html_class, data_attribute | Tags and attributes only; no DOM or browser semantics. | project-owned extras; no third-party parser dependency | fixture declared for extras landmarks | checked by fixture/real-repo extras coverage when present | extras landmarks checked by fixtures and quality reports | not a Tree-sitter comparator claim |
| JSON | .json | extras-backed | extras | key | Object keys only; scalar values are intentionally skipped. | project-owned extras; no third-party parser dependency | fixture declared for extras landmarks | checked by fixture/real-repo extras coverage when present | extras landmarks checked by fixtures and quality reports | not a Tree-sitter comparator claim |
| Markdown | .md, .markdown | extras-backed | extras | section, checklist, link, todo, fixme | Useful landmarks only; not a full Markdown AST. | project-owned extras; no third-party parser dependency | fixture declared for extras landmarks | checked by fixture/real-repo extras coverage when present | extras landmarks checked by fixtures and quality reports | not a Tree-sitter comparator claim |
| TOML | .toml | extras-backed | extras | key, table | Keys and tables only; scalar values are intentionally skipped. | project-owned extras; no third-party parser dependency | fixture declared for extras landmarks | checked by fixture/real-repo extras coverage when present | extras landmarks checked by fixtures and quality reports | not a Tree-sitter comparator claim |
| YAML | .yaml, .yml | extras-backed | extras | key, section | Mapping keys and sections only; scalar values are intentionally skipped. | project-owned extras; no third-party parser dependency | fixture declared for extras landmarks | checked by fixture/real-repo extras coverage when present | extras landmarks checked by fixtures and quality reports | not a Tree-sitter comparator claim |
| Elixir | .ex, .exs | blocked | none | none | No selected permissive grammar/query/fixture/notice path. | blocked: no approved parser/extras support path | blocked; no conformance fixture claimed | blocked; no real-repo support claim | blocked; no expected-symbol coverage claim | blocked; comparator findings must not promote support |
| Haskell | .hs | blocked | none | none | No selected permissive grammar/query/fixture/notice path. | blocked: no approved parser/extras support path | blocked; no conformance fixture claimed | blocked; no real-repo support claim | blocked; no expected-symbol coverage claim | blocked; comparator findings must not promote support |
| Lua | .lua | blocked | none | none | No selected permissive grammar/query/fixture/notice path. | blocked: no approved parser/extras support path | blocked; no conformance fixture claimed | blocked; no real-repo support claim | blocked; no expected-symbol coverage claim | blocked; comparator findings must not promote support |
| Objective-C/Objective-C++ | .m, .mm | blocked | none | none | No selected permissive grammar/query/fixture/notice path. | blocked: no approved parser/extras support path | blocked; no conformance fixture claimed | blocked; no real-repo support claim | blocked; no expected-symbol coverage claim | blocked; comparator findings must not promote support |
| SQL | .sql | blocked | none | none | No product-approved grammar/query policy for dialect differences. | blocked: no approved parser/extras support path | blocked; no conformance fixture claimed | blocked; no real-repo support claim | blocked; no expected-symbol coverage claim | blocked; comparator findings must not promote support |
| Vue/Svelte single-file components | .vue, .svelte | blocked | none | none | No selected permissive grammar/query/fixture/notice path and no component section adapter. | blocked: no approved parser/extras support path | blocked; no conformance fixture claimed | blocked; no real-repo support claim | blocked; no expected-symbol coverage claim | blocked; comparator findings must not promote support |
| XML | .xml | blocked | none | none | No product-approved extras policy for non-noisy XML landmarks. | blocked: no approved parser/extras support path | blocked; no conformance fixture claimed | blocked; no real-repo support claim | blocked; no expected-symbol coverage claim | blocked; comparator findings must not promote support |

## Backend Notes

- `tree_sitter` means deterministic syntax extraction through registered Tree-sitter grammars and query specs.
- `extras` means project-owned deterministic format landmarks, not Tree-sitter parser support.
- `none` means blocked: no parser or extras-backed support is claimed.
- thinindex does not claim semantic/LSP-level analysis, type resolution, macro expansion, runtime module resolution, or inherited member resolution.

## Quality Report Linkage

- Conformance status comes from matrix fixture declarations guarded by parser/support tests.
- Real-repo status is checked by `cargo test --test real_repos -- --ignored` when `test_repos/` exists.
- Expected-symbol coverage is checked by quality manifests and summarized by quality report exports when configured.
- Comparator status is optional local QA data; external comparator tools remain optional, not bundled, and not production index sources.

## Claim Rules

- Do not claim `experimental` or `blocked` entries as supported.
- Do not describe `extras-backed` formats as Tree-sitter-backed.
- Do not hide blocked languages or formats.
- Do not add support claims without updating the support matrix, conformance fixtures, docs, and notices required by the support level.
- Languages and formats not listed here are unsupported.
