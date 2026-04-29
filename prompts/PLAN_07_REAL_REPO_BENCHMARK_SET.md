# PLAN_07_REAL_REPO_BENCHMARK_SET.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_00 through PLAN_06 are complete and green.

Goal:
Create a curated real-repo benchmark set mechanism so thinindex can be tested against realistic projects with stable query sets.

This pass organizes `test_repos/` benchmarking. It does not add new search semantics, reference extraction rules, ML prediction, or new context commands.

Phase tracking:
- [x] Add `test_repos/MANIFEST.toml` parsing with skip, kind, queries, and expected paths.
- [x] Preserve fallback immediate-child repo discovery when no manifest exists.
- [x] Wire manifest repo selection into ignored real-repo integrity and benchmark tests.
- [x] Add normal tests for manifest parsing and missing manifest repo failures.
- [x] Run required formatting, tests, lint, ignored tests, and bench smoke.
- [x] Commit with `Add real-repo benchmark manifest support`.

Product rule:
Real-repo benchmarks must measure agent-navigation usefulness without making normal `cargo test` depend on downloaded third-party repos.

Scope:
Add a benchmark manifest for repos under `test_repos/`.

Preferred manifest:

- `test_repos/MANIFEST.toml`

The manifest is local-only and should not be required for normal tests.

Repo hygiene:
- `test_repos/` must remain ignored by git.
- Do not commit downloaded third-party repos.
- Do not require internet access in tests.
- Do not download repos automatically.

Manifest shape:
Use a simple TOML format.

Example:

[[repo]]
name = "thinindex"
path = "."
kind = "rust-cli"
queries = ["build_index", "SearchOptions", "usage_events", "Repository search"]

[[repo]]
name = "bookbatch"
path = "bookbatch"
kind = "python-cli"
queries = ["BookMetadata", "prompt builder", "original_price", "renderer"]

[[repo]]
name = "sample-user-project"
path = "sample-user-project"
kind = "mixed"
queries = ["config", "auth", "tests"]

Required manifest fields:
- `name`
- `path`
- `queries`

Optional fields:
- `kind`
- `description`
- `expected_paths`
- `skip`

Behavior:
- If `test_repos/MANIFEST.toml` exists, use it.
- If it is missing, fall back to discovering immediate child directories under `test_repos/` as before.
- If `test_repos/` is missing, print `skipped: test_repos/ missing` and return successfully for ignored/manual tests.
- If no repos are found, print `skipped: test_repos/ has no repo directories` and return successfully.
- Manifest paths are relative to `test_repos/`, except `path = "."` may mean the current thinindex repo only if explicitly supported and documented.
- Skip entries where `skip = true`.

Repo detection fallback:
Accept immediate child directories under `test_repos/` if they contain `.git` OR at least one project marker:

- `Cargo.toml`
- `package.json`
- `pyproject.toml`
- `go.mod`
- `.gitignore`
- `src/`

Queries:
- For manifest repos, use the manifest `queries`.
- For fallback-discovered repos, derive a small deterministic query set from indexed records if practical.
- Do not use random sampling.
- Query count should be capped.

Benchmark integration:
Use the existing benchmark/evaluation layer from PLAN_05.

For each repo:
1. Delete that repo’s `.dev_index`.
2. Run `build_index`.
3. Run shared index/ref integrity checks.
4. Run benchmark queries.
5. Report metrics.

Metrics:
For each repo, report at least:

- repo name
- repo path
- kind if available
- indexed file count
- record count
- ref count
- query count
- hit count
- miss count
- hit rate
- average `wi` latency if already supported
- average pack/impact output size if already supported
- integrity status

Output:
Print compact text output.

If JSON output already exists from PLAN_05, support manifest-based runs there too. Do not add JSON output unless it is already present or trivial.

Normal tests:
Add a small normal test that validates manifest parsing using a temp manifest.

Do not require real `test_repos/` in normal `cargo test`.

Ignored real-repo tests:
Update `tests/real_repos.rs` or benchmark ignored tests to:
- read manifest if present
- run only non-skipped manifest repos
- print tested repo names
- fail if a manifest-listed non-skipped repo path is missing
- continue to support fallback discovery when no manifest exists

Failure policy:
- If manifest exists and a listed repo is missing, fail clearly.
- If manifest does not exist, missing `test_repos/` is a skip, not a failure.
- If a repo fails integrity checks, fail the ignored test and include repo name/path.
- If a repo has benchmark misses, do not fail solely on miss rate unless the manifest later defines expected thresholds.

Optional expected paths:
If `expected_paths` is present, pass it into shared integrity checks.

Do not add thresholds yet:
Do not fail on hit rate, latency, or pack/impact output size in this plan. This plan is about stable measurement, not enforcement.

Instruction surfaces:
- Do not update AGENTS/CLAUDE unless benchmark workflow becomes part of normal agent operation.
- Do not mention `WI.md`.
- Keep `wi --help` current if benchmark command behavior changes.

Acceptance:
- `test_repos/MANIFEST.toml` is supported for ignored/manual real-repo benchmarks.
- Missing manifest falls back to existing discovery behavior.
- Missing `test_repos/` skips cleanly.
- Manifest-listed missing repo fails clearly.
- Manifest parsing has normal tests.
- Real-repo benchmark output lists repos tested and metrics.
- Existing fixture/local/real-repo integrity tests still pass.
- No normal test depends on downloaded repos.
- No JSONL storage is reintroduced.
- No new search/reference semantics are added.

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` if `test_repos/` exists
- If `wi bench` exists: `cargo run --bin wi -- bench`
- If manifest exists: run the real-repo benchmark path and report repos tested

Report:
- changed files
- manifest format implemented
- sample manifest if added
- verification commands and results
- whether ignored local test passed
- whether real-repo benchmark ran, skipped, or failed
- repos tested under `test_repos/`, if any
- commit hash
