# Parser Support Levels

thinindex uses explicit parser support levels so documentation does not overclaim language coverage. The source of truth is the support matrix in `src/support.rs`; this document mirrors that policy for users.

## Levels

- `supported`: grammar/query/fixture/license/docs exist; conformance passes; real-repo checks pass where configured.
- `experimental`: grammar/query exists, but conformance or real-repo coverage is incomplete.
- `blocked`: missing permissive grammar, broken integration, unclear license, or unacceptable parser quality.
- `extras-backed`: project-owned extras intentionally handle deterministic format landmarks instead of Tree-sitter.

## Matrix

| Language/format | Extensions | Level | Backend | Grammar/package | License status | Record kinds | Known gaps |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Rust | `.rs` | supported | tree_sitter | `tree-sitter-rust` | MIT grammar notice | function, struct, enum, trait, type, module, constant, variable | use records are deferred to deterministic refs; no macro expansion or type resolution. |
| Python | `.py` | supported | tree_sitter | `tree-sitter-python` | MIT grammar notice | function, method, class, variable, import | Syntactic extraction only; decorators and complex assignment targets are not semantic analysis. |
| JavaScript | `.js` | supported | tree_sitter | `tree-sitter-javascript` | MIT grammar notice | function, method, class, variable, import, export | No runtime module, prototype, or bundler resolution. |
| JSX | `.jsx` | supported | tree_sitter | `tree-sitter-javascript` | MIT grammar notice | function, method, class, variable, import, export | Definition extraction is Tree-sitter-backed; element usage remains deterministic reference evidence. |
| TypeScript | `.ts` | supported | tree_sitter | `tree-sitter-typescript` | MIT grammar notice | function, method, class, interface, type, variable, import, export | No type alias, generic constraint, or project graph resolution. |
| TSX | `.tsx` | supported | tree_sitter | `tree-sitter-typescript` | MIT grammar notice | function, method, class, interface, type, variable, import, export | Definition extraction is Tree-sitter-backed; element usage remains deterministic reference evidence. |
| Java | `.java` | supported | tree_sitter | `tree-sitter-java` | MIT grammar notice | method, class, enum, interface, type, variable, import | No package visibility, inherited member, or build-system classpath resolution. |
| Go | `.go` | supported | tree_sitter | `tree-sitter-go` | MIT grammar notice | function, method, struct, interface, type, module, variable, constant, import | No semantic exported API set or module graph resolution. |
| C | `.c`, `.h` | supported | tree_sitter | `tree-sitter-c` | MIT grammar notice | function, struct, enum, type, variable, import | No macro expansion, preprocessor configuration, or compile database semantics. |
| C# | `.cs` | supported | tree_sitter | `tree-sitter-c-sharp` | MIT grammar notice | method, class, struct, enum, interface, type, module, variable, import | No partial-type, assembly, or Roslyn-level resolution. |
| C++ | `.cc`, `.cpp`, `.cxx`, `.hh`, `.hpp`, `.hxx` | supported | tree_sitter | `tree-sitter-cpp` | MIT grammar notice | function, method, class, struct, enum, type, module, variable, import | No template instantiation, macro expansion, or compile database semantics. |
| Shell | `.sh`, `.bash` | supported | tree_sitter | `tree-sitter-bash` | MIT grammar notice | function, variable | Sourced files and shell runtime expansion are not resolved. |
| Ruby | `.rb` | supported | tree_sitter | `tree-sitter-ruby` | MIT grammar notice | method, class, module, constant | No require/load target or metaprogramming resolution. |
| PHP | `.php` | supported | tree_sitter | `tree-sitter-php` | MIT grammar notice | function, method, class, interface, trait, enum, module, variable, constant, import | No dynamic include, autoload, or runtime namespace resolution. |
| Scala | `.scala` | experimental | tree_sitter | `tree-sitter-scala` | MIT grammar notice | function, class, enum, trait, type, module, variable, constant, import | Conformance exists, but real-repo coverage and givens/implicits/extension handling remain incomplete. |
| Kotlin | `.kt`, `.kts` | experimental | tree_sitter | `tree-sitter-kotlin-ng` | MIT grammar notice | function, class, enum, type, module, variable, import | Conformance exists, but real-repo coverage and interface/enum-class/extension distinctions remain incomplete. |
| Swift | `.swift` | experimental | tree_sitter | `tree-sitter-swift` | MIT grammar notice | function, method, class, struct, enum, interface, type, variable, import | Conformance exists, but real-repo coverage and extension/overload/module handling remain incomplete. |
| Dart | `.dart` | experimental | tree_sitter | `tree-sitter-dart` | MIT grammar notice | function, method, class, enum, type, variable, constant, import, export | Conformance exists, but real-repo coverage and package/extension/type-alias handling remain incomplete. |
| Nix | `.nix` | experimental | tree_sitter | `tree-sitter-nix` | MIT grammar notice | function, module, import | Conformance exists, but real-repo coverage and exhaustive attr/scalar extraction remain incomplete by design. |
| CSS | `.css` | extras-backed | extras | project-owned extras | project-owned extras; no third-party parser dependency | css_class, css_id, css_variable, keyframes | Selectors and keyframes only; no cascade or browser semantics. |
| HTML | `.html` | extras-backed | extras | project-owned extras | project-owned extras; no third-party parser dependency | html_tag, html_id, html_class, data_attribute | Tags and attributes only; no DOM or browser semantics. |
| Markdown | `.md`, `.markdown` | extras-backed | extras | project-owned extras | project-owned extras; no third-party parser dependency | section, checklist, link, todo, fixme | Useful landmarks only; not a full Markdown AST. |
| JSON | `.json` | extras-backed | extras | project-owned extras | project-owned extras; no third-party parser dependency | key | Object keys only; scalar values are intentionally skipped. |
| TOML | `.toml` | extras-backed | extras | project-owned extras | project-owned extras; no third-party parser dependency | key, table | Keys and tables only; scalar values are intentionally skipped. |
| YAML | `.yaml`, `.yml` | extras-backed | extras | project-owned extras | project-owned extras; no third-party parser dependency | key, section | Mapping keys and sections only; scalar values are intentionally skipped. |
| Vue/Svelte single-file components | `.vue`, `.svelte` | blocked | none | none | blocked: no approved parser/extras support path | none | No selected permissive grammar/query/fixture/notice path and no component section adapter. |
| Objective-C/Objective-C++ | `.m`, `.mm` | blocked | none | none | blocked: no approved parser/extras support path | none | No selected permissive grammar/query/fixture/notice path. |
| SQL | `.sql` | blocked | none | none | blocked: no approved parser/extras support path | none | No product-approved grammar/query policy for dialect differences. |
| XML | `.xml` | blocked | none | none | blocked: no approved parser/extras support path | none | No product-approved extras policy for non-noisy XML landmarks. |
| Lua | `.lua` | blocked | none | none | blocked: no approved parser/extras support path | none | No selected permissive grammar/query/fixture/notice path. |
| Haskell | `.hs` | blocked | none | none | blocked: no approved parser/extras support path | none | No selected permissive grammar/query/fixture/notice path. |
| Elixir | `.ex`, `.exs` | blocked | none | none | blocked: no approved parser/extras support path | none | No selected permissive grammar/query/fixture/notice path. |

## Claim Rules

- Do not describe `experimental` or `blocked` entries as fully supported.
- Do not describe `extras-backed` formats as Tree-sitter-backed.
- Do not claim a new `supported` language without grammar registration, extension mapping, query spec, conformance fixture, license notice, docs entry, and passing gates.
- Do not claim a new `extras-backed` format without fixture coverage, non-noisy record policy, docs entry, and notice coverage.
- Languages and formats not listed are unsupported.
