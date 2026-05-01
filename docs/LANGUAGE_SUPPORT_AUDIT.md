# Language Support Audit

Audit date: 2026-05-01.

This audit compares language support claims against the current implementation,
fixtures, docs, and local real-repo manifest state. It is a truth-in-claims
document, not a support expansion plan.

## Summary

| Level | Count | Meaning |
| --- | ---: | --- |
| supported | 14 | Tree-sitter grammar/query/fixture/license/docs exist and normal validation covers the core parser path. |
| experimental | 5 | Tree-sitter grammar/query/fixture/license/docs exist, but syntax coverage or real-repo hardening is incomplete. |
| extras-backed | 6 | Project-owned deterministic format extraction exists outside Tree-sitter code-symbol parsing. |
| blocked | 7 | No parser or extras-backed support is currently claimed. |

No support levels were changed by this audit. No parser behavior, query spec,
fixture, grammar dependency, or language registry entry was added.

## Architecture Findings

Production code-symbol extraction still uses the Tree-sitter framework:
registry entries, extension mappings, query specs, normalized captures,
conformance fixtures, support-level docs, and license/notice entries.

Project-owned extras handle CSS, HTML, Markdown, JSON, TOML, and YAML landmarks.
Those formats are not claimed as Tree-sitter-backed code-symbol parsers.

No active architecture violation was found. The audit did not find a production
code-symbol path based on a hand-written parser, line scanner, regex parser, or
external tagger fallback.

Universal Ctags is optional, external, not bundled, not required, and not used by production indexing.

## Support Table

| Language/format | Support level | Backend | Grammar | Mapping | Queries/captures | Fixtures | Real-repo coverage | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Rust | supported | tree_sitter | `tree-sitter-rust` | `.rs` | query spec maps functions, structs, enums, traits, types, modules, constants, and variables | `tests/fixtures/language_pack/src/rust/widget.rs` | local manifest includes Rust repos, including `fd` | `use` records remain deterministic refs; no macro expansion or type resolution. |
| Python | supported | tree_sitter | `tree-sitter-python` | `.py` | query spec maps functions, methods, classes, variables, and imports | `tests/fixtures/language_pack/src/python/widget.py` | local manifest includes `httpx` | Syntactic extraction only; decorators and complex assignment targets are not semantic analysis. |
| JavaScript | supported | tree_sitter | `tree-sitter-javascript` | `.js` | query spec maps functions, methods, classes, variables, imports, and exports | `tests/fixtures/language_pack/src/javascript/widget.js` | local manifest includes JavaScript repos such as `zustand`, `web-50projects`, and `gray-matter` | No runtime module, prototype, or bundler resolution. |
| JSX | supported | tree_sitter | `tree-sitter-javascript` | `.jsx` | query spec maps JavaScript declarations; element usage remains reference evidence | `tests/fixtures/language_pack/src/javascript/widget.jsx` | local manifest includes JSX coverage through `zustand` | JSX component/element references are deterministic references, not semantic React analysis. |
| TypeScript | supported | tree_sitter | `tree-sitter-typescript` | `.ts` | query spec maps functions, methods, classes, interfaces, types, variables, imports, and exports | `tests/fixtures/language_pack/src/typescript/widget.ts` | local manifest includes TypeScript repos such as `zustand` and `gray-matter` | No type alias, generic constraint, project graph, or compiler semantics. |
| TSX | supported | tree_sitter | `tree-sitter-typescript` | `.tsx` | query spec maps TypeScript declarations; element usage remains reference evidence | `tests/fixtures/language_pack/src/typescript/widget.tsx` | local manifest includes TSX coverage through `zustand` | TSX component/element references are deterministic references, not semantic React analysis. |
| Java | supported | tree_sitter | `tree-sitter-java` | `.java` | query spec maps methods, classes, enums, interfaces, types, variables, and imports | `tests/fixtures/language_pack/src/java/JavaWidget.java` | local manifest includes Java files in JVM/mobile sample repos | No package visibility, inherited member, or build-system classpath resolution. |
| Go | supported | tree_sitter | `tree-sitter-go` | `.go` | query spec maps functions, methods, structs, interfaces, types, modules, variables, constants, and imports | `tests/fixtures/language_pack/src/go/widget.go` | deferred: no current local manifest entry for a Go-heavy repo was found in this checkout | Fixture and normal parser validation cover the core path; real-repo expansion remains future hardening. |
| C | supported | tree_sitter | `tree-sitter-c` | `.c`, `.h` | query spec maps functions, structs, enums, types, variables, and imports | `tests/fixtures/language_pack/src/c/widget.c` | local manifest includes C sample coverage | No macro expansion, preprocessor configuration, or compile database semantics. |
| C# | supported | tree_sitter | `tree-sitter-c-sharp` | `.cs` | query spec maps methods, classes, structs, enums, interfaces, types, modules, variables, and imports | `tests/fixtures/language_pack/src/csharp/Widget.cs` | local manifest includes `csharp-paint` and C# sample repos | No partial-type, assembly, or Roslyn-level resolution. |
| C++ | supported | tree_sitter | `tree-sitter-cpp` | `.cc`, `.cpp`, `.cxx`, `.hh`, `.hpp`, `.hxx` | query spec maps functions, methods, classes, structs, enums, types, modules, variables, and imports | `tests/fixtures/language_pack/src/cpp/widget.cpp` | local manifest includes C++ coverage in mixed native/mobile samples | No template instantiation, macro expansion, or compile database semantics. |
| Shell | supported | tree_sitter | `tree-sitter-bash` | `.sh`, `.bash` | query spec maps functions and variables | `tests/fixtures/language_pack/src/shell/widget.sh` | local manifest includes shell files in mixed native/mobile samples | Sourced files and shell runtime expansion are not resolved. |
| Ruby | supported | tree_sitter | `tree-sitter-ruby` | `.rb` | query spec maps methods, classes, modules, and constants | `tests/fixtures/language_pack/src/ruby/widget.rb` | local manifest includes `ruby-gem-template` | No require/load target or metaprogramming resolution. |
| PHP | supported | tree_sitter | `tree-sitter-php` | `.php` | query spec maps functions, methods, classes, interfaces, traits, enums, modules, variables, constants, and imports | `tests/fixtures/language_pack/src/php/widget.php` | deferred: no current local manifest entry for a PHP-heavy repo was found in this checkout | Fixture and normal parser validation cover the core path; real-repo expansion remains future hardening. |
| Scala | experimental | tree_sitter | `tree-sitter-scala` | `.scala` | query spec maps functions, classes, enums, traits, types, modules, variables, constants, and imports | `tests/fixtures/language_pack/src/scala/Widget.scala` | local manifest includes Scala sample repos | Givens, implicits, extension handling, and real-repo hardening remain incomplete. |
| Kotlin | experimental | tree_sitter | `tree-sitter-kotlin-ng` | `.kt`, `.kts` | query spec maps functions, classes, enums, types, modules, variables, and imports | `tests/fixtures/language_pack/src/kotlin/Widget.kt` | local manifest includes Kotlin/mobile sample repos | Interface, enum-class, extension distinctions, and real-repo hardening remain incomplete. |
| Swift | experimental | tree_sitter | `tree-sitter-swift` | `.swift` | query spec maps functions, methods, classes, structs, enums, interfaces, types, variables, and imports | `tests/fixtures/language_pack/src/swift/Widget.swift` | local manifest includes Swift sample repos | Extension, overload, module handling, and real-repo hardening remain incomplete. |
| Dart | experimental | tree_sitter | `tree-sitter-dart` | `.dart` | query spec maps functions, methods, classes, enums, types, variables, constants, imports, and exports | `tests/fixtures/language_pack/src/dart/widget.dart` | local manifest includes Dart/Flutter sample repos | Package, extension, type-alias handling, and real-repo hardening remain incomplete. |
| Nix | experimental | tree_sitter | `tree-sitter-nix` | `.nix` | query spec maps functions, modules, and imports | `tests/fixtures/language_pack/src/nix/default.nix` | local manifest includes `nix-dotfiles` | Exhaustive attr/scalar extraction is incomplete by design. |
| CSS | extras-backed | extras | none; project-owned extras | `.css` | extras map classes, ids, variables, and keyframes | `tests/fixtures/sample_repo/frontend/styles/header.css` | local manifest includes CSS in web/doc sample repos | No cascade or browser semantics. |
| HTML | extras-backed | extras | none; project-owned extras | `.html` | extras map tags, ids, classes, and data attributes | `tests/fixtures/html_repo/templates/base.html` | local manifest includes HTML in web/doc sample repos | No DOM, browser, or component semantics. |
| Markdown | extras-backed | extras | none; project-owned extras | `.md`, `.markdown` | extras map sections, checklists, links, TODOs, and FIXMEs | `tests/fixtures/sample_repo/docs/guide.md` | local manifest includes Markdown in docs/config repos | Useful landmarks only; not a full Markdown AST. |
| JSON | extras-backed | extras | none; project-owned extras | `.json` | extras map object keys | `tests/fixtures/sample_repo/config/app.json` | local manifest includes JSON in config-heavy repos | Scalar values are intentionally skipped. |
| TOML | extras-backed | extras | none; project-owned extras | `.toml` | extras map keys and tables | `tests/fixtures/sample_repo/config/thinindex.toml` | local manifest includes TOML in Rust/doc sample repos | Scalar values are intentionally skipped. |
| YAML | extras-backed | extras | none; project-owned extras | `.yaml`, `.yml` | extras map keys and sections | `tests/fixtures/sample_repo/config/pipeline.yaml` | local manifest includes YAML in config-heavy repos | Scalar values are intentionally skipped. |
| Vue/Svelte single-file components | blocked | none | none | `.vue`, `.svelte` | none | none | none; no support claim | No selected permissive grammar/query/fixture/notice path and no component section adapter. |
| Objective-C/Objective-C++ | blocked | none | none | `.m`, `.mm` | none | none | none; no support claim | No selected permissive grammar/query/fixture/notice path. |
| SQL | blocked | none | none | `.sql` | none | none | none; no support claim | No product-approved grammar/query policy for dialect differences. |
| XML | blocked | none | none | `.xml` | none | none | none; no support claim | No product-approved extras policy for non-noisy XML landmarks. |
| Lua | blocked | none | none | `.lua` | none | none | none; no support claim | No selected permissive grammar/query/fixture/notice path. |
| Haskell | blocked | none | none | `.hs` | none | none | none; no support claim | No selected permissive grammar/query/fixture/notice path. |
| Elixir | blocked | none | none | `.ex`, `.exs` | none | none | none; no support claim | No selected permissive grammar/query/fixture/notice path. |

## Claim Corrections

No public support-level downgrades were required.

The audit adds this canonical claim-vs-implementation view and records two
real-repo coverage gaps: Go and PHP have supported Tree-sitter fixture coverage,
but this checkout's local ignored manifest does not currently include a
Go-heavy or PHP-heavy real-repo target. That is a real-repo hardening gap, not a
reason to remove existing supported fixture-backed parser claims.

## Real-repo Coverage Status

Normal validation does not depend on local third-party repositories. The
ignored real-repo suite should run when `test_repos/` exists locally and should
skip or report explicit blockers when it does not.

Local `test_repos/` contents must stay ignored and uncommitted. Future real-repo
test readiness work should make coverage gaps explicit without requiring local
corpora for normal tests.

## Known Caveats

- Tree-sitter extraction is syntax-tree extraction, not semantic/compiler/LSP
  analysis.
- Supported code languages still have language-specific caveats listed in the
  table above and in `src/support.rs`.
- Experimental languages must not be promoted until a scoped plan adds the
  missing fixtures, docs, real-repo evidence, or query hardening.
- Blocked languages must remain visible and unclaimed.
- Extras-backed formats must not be described as Tree-sitter-backed
  code-symbol parsers.

## Recommended Next Action

Run a real-repo test readiness audit. It should inspect
`test_repos/MANIFEST.toml` behavior, skip reasons, local corpus expectations,
coverage gaps such as Go/PHP real-repo targets, and ignored-test ergonomics
without committing third-party repository contents.
