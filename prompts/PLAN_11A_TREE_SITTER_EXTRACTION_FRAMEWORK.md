# PLAN_11A_TREE_SITTER_EXTRACTION_FRAMEWORK.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_00 through PLAN_10 are complete and green.

Goal:
Replace the current ctags-shaped/native-extractor path with a general Tree-sitter extraction framework.

This pass builds the parser backbone. It should remove active ctags dependency surfaces and establish the architecture for adding languages through grammar registration, query specs, fixtures, and license entries.

Product rule:
thinindex must be self-contained, proprietary-package-compatible, cross-platform, and general across common languages. The parser backbone must be Tree-sitter-based, not line-oriented.

Architecture pattern:
Use Registry + Strategy + Adapter.

Required architecture:
- TreeSitterExtractionEngine
- LanguageRegistry
- GrammarAdapter
- QueryPack
- CaptureMapper
- ConformanceSuite
- LicenseRegistry or equivalent license metadata source

Core design:
Adding a language should mostly mean:
1. register grammar
2. map file extensions
3. add Tree-sitter query specs
4. add conformance fixture
5. add license entry

Adding a language must not mean writing a new parser.

Forbidden:
- ctags calls
- bundled ctags
- optional ctags fallback
- ctags PATH detection
- ctags test skipping
- line-oriented or regex-based code-symbol extraction as the parser backend
- copy/pasted per-language extraction loops
- GPL or AGPL parser dependencies
- license/payment/Pro-gating behavior
- release packaging behavior

Allowed language-specific code:
- grammar registration
- file extension mapping
- query files/specs
- tiny grammar adapters where unavoidable
- accepted extras for CSS/HTML/Markdown/TODO/FIXME if already project-owned and explicitly documented

Normalized capture names:
The query/capture layer should normalize language-specific syntax into shared capture names such as:
- `@definition.function`
- `@definition.method`
- `@definition.class`
- `@definition.struct`
- `@definition.enum`
- `@definition.interface`
- `@definition.trait`
- `@definition.type`
- `@definition.module`
- `@definition.variable`
- `@definition.constant`
- `@definition.import`
- `@definition.export`
- `@reference.call`
- `@reference.type`
- `@reference.import`
- `@name`

The shared CaptureMapper must convert normalized captures into `IndexRecord` and later `ReferenceRecord`.

Record source:
- Tree-sitter records use `source = "tree_sitter"`.
- Existing accepted extras use `source = "extras"`.
- No newly built index may contain `source = "ctags"`.

Required implementation:
1. Remove active Universal Ctags integration.
2. Remove ctags calls, ctags checks, ctags PATH detection, and ctags test skips.
3. Remove `ctags_universal` from BuildStats and tests/output.
4. Add the Tree-sitter extraction framework modules.
5. Add a language registry and parser dispatch by file extension.
6. Add query spec loading or embedded query specs.
7. Add the shared capture mapper.
8. Add license metadata plumbing for parser/grammar dependencies.
9. Keep deterministic sorting and duplicate-location canonicalization.
10. Increment `INDEX_SCHEMA_VERSION`.
11. Keep SQLite storage, refs, pack, impact, bench, stats, and wi-init behavior stable.
12. Do not claim full parser parity yet.

Minimal proof:
This plan may wire one minimal Tree-sitter language only if needed to prove the framework. Broad language support belongs in PLAN_11B.

Tests:
- build works without ctags installed.
- generated indexes contain no `source = "ctags"`.
- no tests skip because ctags is missing.
- parser framework tests prove extraction flows through registry/query/capture mapper.
- duplicate path+line+col invariant still holds.
- existing SQLite/ref/search/stats tests pass.
- docs/install tests do not mention ctags as required.

Instruction surfaces:
- Do not reintroduce `WI.md`.
- Keep `wi --help` as the source of truth.
- Keep AGENTS.md and existing CLAUDE.md generation aligned with the canonical Repository search block.

Acceptance:
- ctags is no longer an architectural dependency.
- Tree-sitter framework exists.
- language support is added through registry + query pack, not hand parsers.
- no line-oriented code-symbol extraction is presented as parser support.
- no GPL/AGPL dependency is introduced.
- current non-parser product behavior remains stable.

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `grep -R "ctags\\|Ctags\\|CTAGS" src tests docs README.md Cargo.toml install.sh uninstall.sh || true`
- `cargo run --bin build_index`
- `cargo run --bin wi -- build_index`
- `cargo run --bin wi-stats`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` if `test_repos/` exists

Report:
- changed files
- framework modules added
- removed ctags surfaces
- parser dependencies and licenses
- unsupported languages after this framework pass
- verification commands and results
- ignored local/real repo test status
- commit hash

Phase tracking:
- [x] Remove active Universal Ctags integration surfaces.
- [x] Add Tree-sitter registry, strategy, adapter, query pack, capture mapper, and license metadata plumbing.
- [x] Wire the extraction engine into `build_index` with deterministic records and schema bump.
- [x] Run required verification.
- [x] Commit scoped PLAN_11A changes.
