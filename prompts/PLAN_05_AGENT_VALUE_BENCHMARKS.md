# PLAN_05_AGENT_VALUE_BENCHMARKS.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_00, PLAN_01, PLAN_02, PLAN_03, and PLAN_04 are complete and green.

Goal:
Add a deterministic benchmark/evaluation layer that measures whether thinindex actually improves agent discovery.

This pass does not add new search semantics, ML prediction, or new reference extraction rules. It measures existing behavior and creates evidence for product value.

Product rule:
Do not claim performance or agent-value improvements without measured evidence.

Core question:
Does thinindex help agents find the right files faster, with fewer broad discovery calls and smaller read sets?

Scope:
Add a manual benchmark/evaluation command or test harness. Prefer a simple CLI command if clean:

- `wi bench`

Alternative acceptable shape:

- `cargo test --test bench_repos -- --ignored`

Use whichever fits the current CLI architecture better. Do not overbuild.

Inputs:
Use:

- `.dev_index/index.sqlite`
- `records`
- `refs`
- `usage_events`
- ignored real repos under `test_repos/`
- optional per-repo query files if useful

Suggested optional per-repo query file:
For each repo, allow:

- `test_repos/<repo>/.thinindex-bench.txt`

Each non-empty, non-comment line is a query.

Example query lines:

- `build_index`
- `SearchOptions`
- `HeaderNavigation`
- `PromptService`
- `css_variable`
- `test`

If no per-repo query file exists, use a small built-in fallback query set derived from indexed records.

Metrics:
For each tested repo, report:

Index/build metrics:
- build duration
- SQLite DB size
- record count
- ref count
- indexed file count

Search metrics:
- query count
- hit count
- miss count
- hit rate
- average `wi <term>` latency
- average result count
- max result count

Context command metrics:
- average `wi refs <term>` latency
- average `wi pack <term>` latency
- average `wi impact <term>` latency
- average suggested file count for pack
- average impacted file count for impact

Quality guard metrics:
- duplicate location count must be zero
- malformed record count must be zero
- malformed ref count must be zero
- `.dev_index` indexed path count must be zero

Output:
Print compact text output by default.

Example shape:

Repo: thinindex
- build: 182ms
- db: 412KB
- files: 38
- records: 921
- refs: 244
- queries: 20
- hit rate: 85%
- avg wi latency: 3ms
- avg pack files: 6
- avg impact files: 8
- integrity: ok

Output must be deterministic enough for tests where fixed fixtures are used.

Optional JSON output:
Add `--json` only if simple. If added, tests should cover it.

Normal tests:
Add a small fixture benchmark test in normal `cargo test`.

It should:
- create a temp repo
- build the SQLite index
- run the benchmark/evaluation logic against a fixed query list
- assert metrics are sane:
  - records > 0
  - duplicate locations = 0
  - query count matches input
  - at least one hit
  - no panic

Do not assert fragile exact timings.

Ignored real repo benchmark:
Add or extend an ignored test or command that loops over `test_repos/`.

If `test_repos/` is missing:
- print `skipped: test_repos/ missing`
- return successfully

If no repo dirs are found:
- print `skipped: test_repos/ has no repo directories`
- return successfully

For each accepted repo:
- delete `.dev_index`
- rebuild index
- run benchmark queries
- print metrics

Repo detection:
Accept immediate child directories under `test_repos/` if they contain `.git` OR at least one project marker:

- `Cargo.toml`
- `package.json`
- `pyproject.toml`
- `go.mod`
- `.gitignore`
- `src/`

Do not download repos automatically.

Data storage:
Do not add persistent benchmark storage in this pass unless trivial.
Printing results is enough.

If writing a report file, use:

- `.dev_index/bench.txt`
- or `.dev_index/bench.json`

Do not write outside `.dev_index`.

Usage events:
Benchmark queries should not pollute normal `usage_events` unless explicitly desired.
Prefer a non-logging internal path for benchmark query execution if practical.
If not practical, document that benchmark runs create usage events and update tests accordingly.

Help text:
If adding `wi bench`, update `wi --help`.

Do not reintroduce `WI.md`.

AGENTS/CLAUDE:
Do not update the canonical Repository search block unless a new recommended agent workflow is proven.

Help/instruction update:
- If `wi bench` is added, update `wi --help`.
- Do not add benchmark workflow to AGENTS/CLAUDE unless it becomes part of normal agent operation.
- Do not mention `WI.md`.

Acceptance:
- benchmark/evaluation logic exists
- fixture benchmark runs in normal tests
- real-repo benchmark can run manually against `test_repos/`
- benchmark reports build/index/search/context metrics
- benchmark does not assert fragile timing values
- existing `wi`, `wi refs`, `wi pack`, `wi impact`, and `wi-stats` behavior remains stable
- no JSONL storage is reintroduced
- no normal test depends on local repos or `test_repos/`

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- if CLI command exists: `cargo run --bin wi -- bench`
- if ignored real-repo test exists: `cargo test --test bench_repos -- --ignored`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` if repos exist

Report:
- changed files
- benchmark command/test shape
- sample benchmark output
- validation results
- whether ignored local test passed
- whether real-repo benchmark ran, skipped, or failed
- repos benchmarked under `test_repos/`, if any

<!-- thinindex-plan-instruction-surfaces-start -->

Instruction surfaces:
- Do not reintroduce `WI.md`.
- Keep `wi --help` as the source of truth for command syntax, filters, examples, and subcommands.
- Keep AGENTS.md and existing CLAUDE.md generation in sync with the canonical `## Repository search` block.
- AGENTS.md should be created if absent.
- CLAUDE.md should be normalized only if present; do not create CLAUDE.md.
- Repeated `wi-init` runs must not duplicate `## Repository search`.
- Remove/normalize legacy markers: `@WI.md`, `See WI.md for repository search/index usage.`, `See `WI.md` for repository search/index usage.`, and old paragraph-style Repository search blocks.
- Update tests whenever help text or canonical Repository search text changes.

<!-- thinindex-plan-instruction-surfaces-end -->

<!-- thinindex-plan-help-update-start -->

Help/instruction update:
- If `wi bench` is added, update `wi --help`.
- Do not add benchmark workflow to AGENTS/CLAUDE unless it becomes part of normal agent operation.
- Do not mention `WI.md`.

<!-- thinindex-plan-help-update-end -->
