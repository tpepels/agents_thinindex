# RECOVERY_03_PERFORMANCE_PROFILING_AND_BUDGETS.md

Use superpowers:subagent-driven-development.

Do not implement this until RECOVERY_02_CORE_TOUCHPOINT_REPAIR.md is complete and green.

Goal:
Profile build_index and wi latency, fix measured bottlenecks, and add practical performance budgets.

Scope:
Performance profiling and targeted fixes only. Do not add parser architecture, packaging, licensing, payment, cloud, telemetry, or MCP work.

Current failure it addresses:
build_index became slow on a small repo and agents will avoid thinindex if commands feel slow.

Phases:
- [x] Add or use low-noise timing instrumentation.
- [x] Measure cold/stale build_index.
- [x] Measure immediate no-change build_index.
- [x] Measure stale wi <query> that auto-rebuilds.
- [x] Measure warm wi <query>.
- [x] Measure wi refs, wi pack, and wi impact.
- [x] Identify measured bottleneck.
- [x] Fix measured bottleneck.
- [x] Add performance regression tests or checks where practical.
- [x] Document budgets and troubleshooting.
- [x] Run verification.
- [x] Commit.

Profile phases:
- CLI startup
- repo root detection
- file discovery
- ignore matching
- metadata/stat collection
- freshness/change detection
- parser setup
- Tree-sitter query compilation
- per-file parse/extract
- SQLite open/schema/migrations
- SQLite deletes/inserts
- refs/dependency generation
- output formatting

Likely bottlenecks to check:
- unchanged files reparsed
- Tree-sitter queries compiled per file
- SQLite writes not batched
- dependency/ref graph globally recomputed on every small change
- quality/comparator/real-repo logic leaking into normal path
- .dev_index, test_repos, vendor/generated/minified paths included
- unbounded output formatting before truncation

Performance targets:
Define practical measured budgets for:
- warm wi <query>
- no-change build_index
- stale rebuild with changed files
- wi pack
- wi impact

Tests:
- unchanged files are not reparsed on second build.
- quality/comparator hooks do not run during normal query/build.
- .dev_index is not indexed.
- test_repos is not indexed by normal repo build unless explicitly intended.
- warm query output remains bounded.
- timing/report output is deterministic enough where tested.

Acceptance:
- bottleneck is measured before fixing.
- normal query path is fast.
- no-change build_index is fast.
- performance budgets are documented.
- regression tests/checks exist where practical.

Verification:
- cargo fmt --check
- cargo test
- cargo clippy --all-targets --all-features -- -D warnings
- timing run for build_index
- immediate second build_index timing
- stale-state wi test timing
- immediate warm wi test timing
- cargo run --bin wi -- refs test
- cargo run --bin wi -- pack test
- cargo run --bin wi -- impact test
- cargo test --test local_index -- --ignored
- cargo test --test real_repos -- --ignored if test_repos exists

Commit:
Profile and fix thinindex performance
