# Thinindex Roadmap

Thinindex is a local agent-navigation layer for coding repositories. The product direction is to make coding agents start with local file:line landmarks and compact context commands before broad file reads.

This roadmap separates shipped behavior from future work. It should not be read as a promise that future product, licensing, parser, or packaging features exist today.

## Current shipped behavior

The current toolchain provides:

- `build_index` for explicit repo-local index builds.
- `wi <term>` for named landmark search.
- `wi refs <term>` for deterministic reference output.
- `wi pack <term>` for compact implementation read sets.
- `wi impact <term>` for conservative, evidence-backed related files.
- `wi bench` for local benchmark and integrity measurements.
- `wi-stats` for local usage stats.
- `wi-init` for repository setup and instruction normalization.
- `wi-init --remove` for repo-local index removal.

Current storage is `.dev_index/index.sqlite`. The entire `.dev_index/` directory is a disposable local cache. Old pre-alpha cache files are not current storage; `build_index` rebuilds them into SQLite.

Current instruction surfaces are `AGENTS.md` and, when already present, `CLAUDE.md`. `wi-init` creates or normalizes the canonical `## Repository search` block and does not create a separate instruction file.

Indexing uses the Tree-sitter extraction framework and project-owned extras. No external parser command is required for the shipped local workflow. Parser claims use the support levels in `src/support.rs`, `README.md`, and `docs/PARSER_SUPPORT.md`: supported, experimental, blocked, and extras-backed. CSS, HTML, Markdown, JSON, TOML, and YAML are extras-backed deterministic format extraction, not Tree-sitter-backed code-symbol parsing.

Formats and languages not listed in the support matrix are unsupported until they have the required grammar or extras policy, extension mapping, fixtures, docs, and notice coverage.

## Product direction

Thinindex should remain:

- local-first
- explicit rather than daemon-driven
- deterministic where possible
- useful for both coding agents and supervising developers
- compact in output
- measurable with local benchmarks
- independent of source upload or hosted search

The core product question is not whether thinindex is faster than grep. The question is whether it reduces blind discovery by giving agents better first reads.

## Current value measurement

`wi bench` and ignored real-repo benchmark tests measure:

- build time
- database size
- record count
- reference count
- search hit/miss behavior
- search latency
- pack output size
- impact output size
- integrity status

Benchmarks should stay deterministic and should avoid fragile exact timing assertions. Claims about agent value should be tied to benchmark output or concrete workflow observations.

## Real-repo hardening

Real-repo work uses local, ignored repositories:

- `test_repos/` is never committed.
- Developers clone third-party repositories there manually.
- `test_repos/MANIFEST.toml` records local real-repo benchmark targets when present.
- Ignored tests validate real repositories without making normal `cargo test` depend on local clones.
- Manifest entries may include `expected_symbols` and `expected_symbol_patterns` so real-repo hardening can check targeted symbol coverage without relying on fragile total record counts.
- Structured `[[repo.expected_symbol]]` and `[[repo.expected_symbol_pattern]]` entries can also constrain expected records by language, path/path glob, kind, name, regex, and minimum count.
- Real-repo parser reports include zero-record supported languages as weak areas rather than using fragile total record assertions.
- Parser performance reports include parse time by language, record/ref counts by language, slow/noisy/large files, parse errors, unsupported extension gaps, and expected-symbol coverage. Generated, vendor, dependency, and minified files should be ignored when they add noise rather than navigation value.

The current ignored checks cover local index behavior, real-repo integrity, and real-repo benchmarks when the local data exists.

Release archive packaging is available through `scripts/package-release` for the current platform. The script stages all thinindex binaries plus release notices, documentation, and archive install/uninstall helpers, then writes a `.tar.gz` or `.zip` archive and SHA256 checksum under `dist/`. Native package formats, signing, notarization, and CI publishing remain future release-hardening work.

## Near-term work

Near-term work should focus on hardening the current agent-navigation surface:

- improve documentation and product positioning
- expand real-repo benchmark coverage
- keep command help as the CLI source of truth
- improve release checks and installer smoke tests
- reduce noisy or incomplete reference extraction only when precision stays high
- continue proving that normal tests are independent of local `.dev_index/` and `test_repos/`

Do not add new command families or broader search semantics without a plan that explains the agent workflow they improve.

## Parser and packaging

Universal Ctags is removed from the active parser path. Dependency license audit policy is configured through `cargo deny check licenses`, and proprietary cross-platform packages still require a passing audit, notice review, and release hardening.

Before proprietary Windows/macOS/Linux packages are viable, thinindex needs:

- permissively licensed bundled parser dependencies
- passing `cargo deny check licenses` output for the committed `Cargo.lock`
- `THIRD_PARTY_NOTICES` included as a release artifact
- install docs that match the bundled parser stack
- release artifacts that pass archive and installer smoke tests

Tree-sitter parser improvements should be introduced as product infrastructure, not as search-semantics changes by themselves.

Current parser support is limited to the documented support matrix and its support levels. Additional languages must bring a permissively licensed grammar, registry entry, extension mapping, query spec, conformance fixture, and notice entry before they are claimed as supported.

## Product boundary

The current tool is local/free. No payment, account, license enforcement, telemetry, cloud sync, remote indexing, feature lockout, or release installer behavior is implemented.

The free/local core must continue to include local indexing, `build_index`, `wi <term>`, basic filters, `wi --help`, `wi-init`, repository instruction setup, local cache rebuilds, `.dev_index/index.sqlite`, `wi-stats`, and no-network local operation.

Future paid work is documented in [PRODUCT_BOUNDARY.md](PRODUCT_BOUNDARY.md). Candidate Pro value should come from proof, hardening, integrations, packaging convenience, and advanced workflows, not from paywalling basic local navigation.

## Future product work

Future product work may include:

- richer Tree-sitter parser coverage
- broader language support
- better diagnostics for stale or missing indexes
- cross-platform archives and installers after the parser blocker is removed
- optional paid editions after licensing, packaging, and parser licensing are solved
- team policy templates after a stable individual workflow exists

These items are not shipped behavior today.

## Documentation rules

Documentation should:

- describe task workflows before implementation internals
- keep `wi --help` as the source of truth for CLI syntax, filters, examples, and subcommands
- describe `.dev_index/index.sqlite` as the canonical cache
- describe old pre-alpha caches only as disposable migration input
- avoid claiming semantic code intelligence, LSP replacement, hosted search, or predictive ranking
- avoid commercial packaging claims before dependency audits and release hardening are complete
