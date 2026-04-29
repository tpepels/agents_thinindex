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

Indexing uses native Rust parser code and project-owned extra extractors. No external parser command is required for the shipped local workflow.

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

The current ignored checks cover local index behavior, real-repo integrity, and real-repo benchmarks when the local data exists.

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

The native parser boundary is current infrastructure. Language-specific parser quality is still incomplete and should improve incrementally without changing storage or command semantics.

Before proprietary Windows/macOS/Linux packages are viable, thinindex needs:

- permissively licensed bundled parser dependencies
- license audit coverage for those dependencies
- cross-platform installer and archive hardening

Native parser improvements should be introduced as product infrastructure, not as search-semantics changes by themselves.

## Product boundary

The current tool is local/free. No payment, account, license enforcement, telemetry, cloud sync, remote indexing, feature lockout, or release installer behavior is implemented.

The free/local core must continue to include local indexing, `build_index`, `wi <term>`, basic filters, `wi --help`, `wi-init`, repository instruction setup, local cache rebuilds, `.dev_index/index.sqlite`, `wi-stats`, and no-network local operation.

Future paid work is documented in [PRODUCT_BOUNDARY.md](PRODUCT_BOUNDARY.md). Candidate Pro value should come from proof, hardening, integrations, packaging convenience, and advanced workflows, not from paywalling basic local navigation.

## Future product work

Future product work may include:

- richer native parsing
- broader language support
- better diagnostics for stale or missing indexes
- cross-platform archives and installers after dependency audits and release hardening
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
