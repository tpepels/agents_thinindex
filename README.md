# thinindex

thinindex is a local agent-navigation layer for coding repositories. It builds a repo-local SQLite index so coding agents and the developers supervising them can start with file:line landmarks instead of broad file reads.

No daemon. No embeddings. No vector database. No source upload. The SQLite engine is bundled into the Rust binaries, so users do not need a system SQLite package.

Privacy note: `.dev_index/` is local and disposable, but it can contain project paths, symbols, references, and compact evidence strings. Add secrets and sensitive paths to `.thinindexignore` before indexing. See [docs/SECURITY_PRIVACY.md](docs/SECURITY_PRIVACY.md).

## What thinindex is

thinindex indexes named repository landmarks: functions, classes, methods, CSS selectors, HTML ids/classes, Markdown headings, TODO/FIXME markers, and related deterministic references. The `wi` command returns compact file:line results from that index.

The context commands use the same local data:

- `wi refs <term>` shows deterministic references around a landmark.
- `wi pack <term>` returns a compact read set for implementation work.
- `wi impact <term>` returns evidence-backed files to inspect before edits.

The index is local-first and repo-local. It lives under `.dev_index/` and is intended to be disposable.

## Why agents need it

Agents waste context when they explore a codebase by reading files blindly. thinindex gives them a cheap first pass:

- use file:line landmarks before broad reads
- read fewer irrelevant files
- gather compact read sets from refs, pack, and impact
- measure index and command behavior with local benchmarks

thinindex is for navigation. It is not a hosted search service, an IDE replacement, an LSP replacement, or full semantic code understanding.

## Quickstart

Install the binaries, then run these commands inside a repository:

```bash
wi-init
build_index
wi --help
wi <term>
wi pack <term>
wi impact <term>
```

Example:

```bash
wi build_index
wi pack build_index
wi impact build_index
```

Run `wi --help` for the current command syntax, filters, examples, and subcommands. Keep that help output as the source of truth for CLI details.

## Agent workflow

The canonical agent workflow is:

1. Before broad repository discovery, run `build_index`.
2. Run `wi --help` if search filters, examples, or subcommands are needed.
3. Use `wi <term>` before grep/find/ls/Read to locate code.
4. For implementation work, prefer `wi pack <term>` to get a compact read set.
5. Before editing a symbol or feature area, run `wi impact <term>` to find related tests/docs/callers.
6. Read only files returned by `wi` unless the result is insufficient.
7. If `wi` returns no useful result, rerun `build_index` once and retry.
8. Fall back to grep/find/Read only after that retry fails.

`wi-init` creates or normalizes this workflow in `AGENTS.md` and normalizes an existing `CLAUDE.md` when present. It does not generate a `WI.md` instruction file.

Optional integration packs for Codex, Claude, generic agents, and future local tool wrappers live under `integrations/agents/`. See [docs/AGENT_INTEGRATION.md](docs/AGENT_INTEGRATION.md) for read-budget guidance and the local-only `wi-stats` workflow audit.

## Commands

Installed commands:

- `build_index`: builds or updates `.dev_index/index.sqlite`; `build_index --stats` adds compact scale diagnostics.
- `wi <term>`: searches named landmarks and returns compact file:line results.
- `wi refs <term>`: shows deterministic references for matching landmarks.
- `wi pack <term>`: returns a dependency-aware, deduplicated read set for implementation work.
- `wi impact <term>`: returns dependency-aware related files with concrete reasons and confidence labels.
- `wi bench`: measures build, search, context-command, size, count, latency, and integrity behavior.
- `wi-stats`: shows local usage stats, hit/miss graphs, and advisory agent workflow audit counts.
- `wi-init`: prepares a repository for agent use.
- `wi-init --remove`: removes the repo-local index.

Search filters and examples are documented by `wi --help`, not duplicated here.

All installed binaries support `--version`.

## Storage model

The canonical index path is:

```text
.dev_index/index.sqlite
```

`.dev_index` is a disposable local cache. Do not commit it. If the schema changes, `build_index` may rebuild it automatically.

Pre-alpha JSONL `.dev_index` caches are also disposable. `build_index` detects the old cache shape and rebuilds `.dev_index/index.sqlite`.

`wi` does not silently rebuild a missing or stale index. It tells users to run `build_index` so rebuilds are explicit.

Usage stats are stored in the same SQLite database. `make uninstall` removes installed binaries only; it does not remove repo-local caches.

The SQLite index also stores internal local dependency and reference graphs for resolved/unresolved imports, syntax references, structured docs/style references, capped text fallback, file-role mapping, and an optional semantic-facts table. The graphs are best-effort foundation data for dependency-aware context and impact. Semantic adapters are disabled by default and are not required for normal indexing. See [docs/DEPENDENCY_GRAPH.md](docs/DEPENDENCY_GRAPH.md), [docs/REFERENCE_GRAPH.md](docs/REFERENCE_GRAPH.md), [docs/FILE_ROLES.md](docs/FILE_ROLES.md), [docs/CONTEXT_PACKS.md](docs/CONTEXT_PACKS.md), [docs/IMPACT_ANALYSIS.md](docs/IMPACT_ANALYSIS.md), [docs/SEMANTIC_ADAPTERS.md](docs/SEMANTIC_ADAPTERS.md), and [docs/TECHNICAL_FINAL_AUDIT.md](docs/TECHNICAL_FINAL_AUDIT.md).

## Real-repo hardening

Normal tests use fixtures and do not depend on local clones. Real-repo validation is opt-in:

- `test_repos/` is ignored and local-only.
- Clone third-party repositories there manually when needed.
- No third-party repository contents are committed.
- `test_repos/MANIFEST.toml` records local benchmark and integrity targets when present.
- Ignored tests validate indexing, references, context commands, parser coverage, and benchmark behavior against those repos.
- Manifest entries must include `name`, `path`, `kind`, `languages`, and `queries`. See [docs/REAL_REPO_MANIFEST.md](docs/REAL_REPO_MANIFEST.md) for curation rules and the full local-only schema.
- Manifest entries can define `expected_paths`, `expected_symbols`, and `expected_symbol_patterns`. Expected symbols are checked by the ignored real-repo parser hardening test; patterns are Rust regular expressions matched against indexed symbol names.
- For more precise coverage checks, use `[[repo.expected_symbol]]` with `language`, `path`, `kind`, and `name`, or `[[repo.expected_symbol_pattern]]` with `language`, `path_glob`, `kind`, `name_regex`, and `min_count`.
- The real-repo parser report lists supported languages with zero emitted records as weak areas. These usually mean the files contain no query-matched declarations, and they should become fixture cases if important symbols are missed.
- Parser performance reports include parse time by language, record/ref counts by language, slow files, noisy files, large files, parse errors, unsupported extensions, and expected-symbol coverage.
- Built-in resource guards cap records per file and refs per file/build. Warnings identify unusually slow, large, or noisy files so generated/vendor/minified paths can be ignored deliberately instead of silently slowing indexing.

Example manifest fields:

```toml
[[repo]]
name = "local-project"
path = "local-project"
queries = ["build_index"]
expected_paths = ["src/"]

[[repo.expected_symbol]]
language = "rs"
path = "src/indexer.rs"
kind = "function"
name = "build_index"

[[repo.expected_symbol_pattern]]
language = "ts"
path_glob = "src/**/*.ts"
kind = "function"
name_regex = "^[A-Za-z_].*"
min_count = 20
```

Run real-repo checks with:

```bash
cargo test --test local_index -- --ignored
cargo test --test real_repos -- --ignored
cargo test --test bench_repos -- --ignored
```

## Benchmarks/value measurement

`wi bench` measures local behavior without asserting fragile timing promises. The benchmark reports:

- build time
- database size
- record and reference counts
- hit/miss behavior for deterministic queries
- search latency
- pack and impact output sizes
- integrity status

Use those numbers to evaluate whether thinindex helps a particular repository and workflow. Do not infer broad agent-performance gains from a single benchmark run.

Parser performance expectations are intentionally practical rather than exact: fixture builds should stay fast in normal tests, real-repo timing is reported by ignored tests, and timing regressions should be investigated with local repo context instead of brittle global thresholds. See [docs/PERFORMANCE.md](docs/PERFORMANCE.md) for file-size limits, SQLite tuning, and monorepo ignore guidance.

## Limitations

thinindex is intentionally conservative:

- It is not full semantic code understanding.
- It is not an IDE or LSP replacement.
- References are deterministic and incomplete.
- Impact output is dependency-aware and evidence-backed but not exhaustive.
- Agents can still ignore repository instructions.
- Generated, build, vendor, dependency, and large fixture paths should be ignored.
- Tree-sitter parser support is deterministic symbol extraction, not semantic or LSP-level analysis.
- Semantic adapters are optional and disabled by default; external compiler/LSP tools are not bundled or required.
- Bundled parser dependencies must stay permissively licensed and audited with `cargo deny check licenses` before commercial release artifacts.
- Generated, vendor, dependency, lockfile, and minified paths should be ignored when they dominate parser timing or record/ref counts without adding navigation value.

## Parser Support

Parser and format support claims use explicit levels from the source-controlled support matrix:

- `supported`: grammar/query/fixture/license/docs exist; conformance passes; real-repo checks pass where configured.
- `experimental`: grammar/query exists, but conformance or real-repo coverage is incomplete.
- `blocked`: missing permissive grammar, broken integration, unclear license, or unacceptable parser quality.
- `extras-backed`: project-owned extras intentionally handle deterministic format landmarks instead of Tree-sitter.

Detailed gaps and blocked reasons are maintained in [docs/PARSER_SUPPORT.md](docs/PARSER_SUPPORT.md). The generated support dashboard in [docs/LANGUAGE_SUPPORT.md](docs/LANGUAGE_SUPPORT.md) summarizes support levels, backend claims, conformance status, real-repo status, expected-symbol coverage, comparator status, and blocked entries from the source-controlled matrix.

| Language/format | Extensions | Level | Backend | Grammar/package | Expected record kinds |
| --- | --- | --- | --- | --- | --- |
| Rust | `.rs` | supported | tree_sitter | `tree-sitter-rust` | function, struct, enum, trait, type, module, constant, variable |
| Python | `.py` | supported | tree_sitter | `tree-sitter-python` | function, method, class, variable, import |
| JavaScript | `.js` | supported | tree_sitter | `tree-sitter-javascript` | function, method, class, variable, import, export |
| JSX | `.jsx` | supported | tree_sitter | `tree-sitter-javascript` | function, method, class, variable, import, export |
| TypeScript | `.ts` | supported | tree_sitter | `tree-sitter-typescript` | function, method, class, interface, type, variable, import, export |
| TSX | `.tsx` | supported | tree_sitter | `tree-sitter-typescript` | function, method, class, interface, type, variable, import, export |
| Java | `.java` | supported | tree_sitter | `tree-sitter-java` | method, class, enum, interface, type, variable, import |
| Go | `.go` | supported | tree_sitter | `tree-sitter-go` | function, method, struct, interface, type, module, variable, constant, import |
| C | `.c`, `.h` | supported | tree_sitter | `tree-sitter-c` | function, struct, enum, type, variable, import |
| C# | `.cs` | supported | tree_sitter | `tree-sitter-c-sharp` | method, class, struct, enum, interface, type, module, variable, import |
| C++ | `.cc`, `.cpp`, `.cxx`, `.hh`, `.hpp`, `.hxx` | supported | tree_sitter | `tree-sitter-cpp` | function, method, class, struct, enum, type, module, variable, import |
| Shell | `.sh`, `.bash` | supported | tree_sitter | `tree-sitter-bash` | function, variable |
| Ruby | `.rb` | supported | tree_sitter | `tree-sitter-ruby` | method, class, module, constant |
| PHP | `.php` | supported | tree_sitter | `tree-sitter-php` | function, method, class, interface, trait, enum, module, variable, constant, import |
| Scala | `.scala` | experimental | tree_sitter | `tree-sitter-scala` | function, class, enum, trait, type, module, variable, constant, import |
| Kotlin | `.kt`, `.kts` | experimental | tree_sitter | `tree-sitter-kotlin-ng` | function, class, enum, type, module, variable, import |
| Swift | `.swift` | experimental | tree_sitter | `tree-sitter-swift` | function, method, class, struct, enum, interface, type, variable, import |
| Dart | `.dart` | experimental | tree_sitter | `tree-sitter-dart` | function, method, class, enum, type, variable, constant, import, export |
| Nix | `.nix` | experimental | tree_sitter | `tree-sitter-nix` | function, module, import |
| CSS | `.css` | extras-backed | extras | project-owned extras | css_class, css_id, css_variable, keyframes |
| HTML | `.html` | extras-backed | extras | project-owned extras | html_tag, html_id, html_class, data_attribute |
| Markdown | `.md`, `.markdown` | extras-backed | extras | project-owned extras | section, checklist, link, todo, fixme |
| JSON | `.json` | extras-backed | extras | project-owned extras | key |
| TOML | `.toml` | extras-backed | extras | project-owned extras | key, table |
| YAML | `.yaml`, `.yml` | extras-backed | extras | project-owned extras | key, section |
| Vue/Svelte single-file components | `.vue`, `.svelte` | blocked | none | none | none |
| Objective-C/Objective-C++ | `.m`, `.mm` | blocked | none | none | none |
| SQL | `.sql` | blocked | none | none | none |
| XML | `.xml` | blocked | none | none | none |
| Lua | `.lua` | blocked | none | none | none |
| Haskell | `.hs` | blocked | none | none | none |
| Elixir | `.ex`, `.exs` | blocked | none | none | none |

Languages and formats not listed are unsupported. They are not silently parsed through line scanning. New code-language support needs a permissively licensed Tree-sitter grammar, an extension mapping, a query spec, a conformance fixture, a notice entry, and support-matrix documentation before it is claimed as supported. New extras-backed format support needs explicit non-noisy record policy, fixture coverage, and support-matrix documentation.

Known extraction gaps: Rust `use` records, Ruby `require` targets, Shell sourced files, dynamic PHP includes, macro-expanded C/C++, template instantiation, C# partial-type and assembly resolution, Scala givens/implicits/extension resolution, Kotlin interface/enum-class distinctions, Swift extensions/overloads/module resolution, Dart package and extension resolution, exhaustive Nix attribute/scalar extraction, exhaustive JSON/TOML/YAML scalar extraction, inherited members, and LSP-level type resolution are not claimed as parser-backed symbol extraction.

## Free/local and future Pro

thinindex is currently a local/free tool. There is no license enforcement, payment flow, account login, cloud sync, telemetry, remote indexing, or feature lockout.

The free/local core includes local indexing, `build_index`, `wi <term>`, basic filters, `wi --help`, `wi-init`, repo-local SQLite storage, `wi-stats`, AGENTS.md setup, existing CLAUDE.md normalization, local cache rebuilds, and no-network operation.

Future Pro candidates are documented in [docs/PRODUCT_BOUNDARY.md](docs/PRODUCT_BOUNDARY.md). They are candidates, not current restrictions.

## Install/uninstall

Requires Rust/Cargo. Indexing is self-contained and does not require an external parser command.

Arch Linux:

```bash
sudo pacman -S rust
```

Debian / Ubuntu:

```bash
sudo apt update
sudo apt install cargo
```

Fedora:

```bash
sudo dnf install rust cargo
```

macOS with Homebrew:

```bash
brew install rust
```

Install:

```bash
make install
```

Default install location:

```text
$HOME/.local/bin
```

Check:

```bash
build_index --version
wi --version
wi-init --version
wi-stats --version
```

Uninstall installed binaries:

```bash
make uninstall
```

Uninstall does not delete `.dev_index`, `.thinindexignore`, `AGENTS.md`, or `CLAUDE.md`.

Remove a repo-local index:

```bash
wi-init --remove
```

## Packaging/licensing caveat

SQLite and Tree-sitter parser dependencies are bundled through the Rust dependency configuration. Universal Ctags is optional, external, not bundled, not required, and not used by production indexing.

Dependency license policy is configured in `deny.toml` and checked with:

```bash
cargo deny check licenses
```

Only permissively licensed parser dependencies are allowed. Proprietary Windows/macOS/Linux packages remain blocked if the audit finds GPL, AGPL, LGPL-only, MPL-only, EPL, CDDL, unknown, custom, or non-commercial dependency terms.

`THIRD_PARTY_NOTICES` records direct runtime dependencies, bundled SQLite status, Tree-sitter grammar notices, and generated parser source status. It is part of release artifacts. See [docs/LICENSE_AUDIT.md](docs/LICENSE_AUDIT.md) for the policy and audit process.

Release archives are built with:

```bash
scripts/package-release
```

Archives include all four binaries, `README.md`, `INSTALL.md`, `SBOM.md`, `docs/RELEASING.md`, `docs/INSTALLERS.md`, helper install/uninstall scripts, and `THIRD_PARTY_NOTICES`. They do not include `.dev_index/index.sqlite`, `.dev_index/quality/`, `test_repos/`, build output junk, local reports, signing secret material, or source checkout contents. Native installers/package formats, completed signing, and notarization remain scaffolded release-hardening work through `scripts/sign-release-artifact`. See [docs/RELEASING.md](docs/RELEASING.md), [docs/INSTALLERS.md](docs/INSTALLERS.md), and [docs/SECURITY_PRIVACY.md](docs/SECURITY_PRIVACY.md).

Local release gates can be run with:

```bash
scripts/check-release
```

## Development

Build and test:

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

Local CI parity:

```bash
scripts/check-ci
```

Normal CI uses checked-in parser and quality fixtures only. It does not require local real repositories, ignored tests, network-fetched side repos, or optional external comparator commands. Manual quality checks that use `test_repos/` stay ignored and local.

Parser-quality audit status is summarized in [docs/QUALITY_SYSTEM_AUDIT.md](docs/QUALITY_SYSTEM_AUDIT.md). Relationship/navigation audit status is summarized in [docs/TECHNICAL_FINAL_AUDIT.md](docs/TECHNICAL_FINAL_AUDIT.md).

Manual smoke:

```bash
rm -rf .dev_index
cargo run --bin build_index
cargo run --bin wi -- build_index
cargo run --bin wi -- refs build_index
cargo run --bin wi -- pack build_index
cargo run --bin wi -- impact build_index
cargo run --bin wi-stats
```

Generated and local files:

- `.dev_index/` is ignored and disposable.
- `.dev_index/quality/` and `test_repos/` are local-only and must not be committed.
- `.thinindexignore` is repo-local configuration for thinindex-only ignores.
- `AGENTS.md` is the canonical repository instruction surface created by `wi-init`.
- Existing `CLAUDE.md` files are normalized when present; `wi-init` does not create one.
