# PLAN_37_MONOREPO_SCALE_AND_INCREMENTAL_INDEXING.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_36_TEST_BUILD_CONFIG_MAPPING.md is complete and green.

Goal:
Make thinindex robust on large repositories and monorepos.

Product rule:
Parser quality is not useful if indexing large repos is too slow, memory-heavy, or noisy.

Required:
- Profile build_index on large real repos if available.
- Add incremental indexing safeguards.
- Add parser scheduling/parallelism only if measured and safe.
- Add file-size limits/warnings.
- Add generated/vendor/minified ignore guidance.
- Add SQLite tuning if needed.
- Add index compaction/vacuum behavior if useful.
- Add performance reports that do not require huge snapshots.

Potential features:
- parallel parsing
- bounded worker pool
- per-language timing
- large-file skip/report policy
- generated/vendor detection
- `build_index --stats` if not already present
- watch mode only if small and safe; otherwise defer

Tests:
- large-ish fixture does not panic
- generated/vendor ignored if configured
- file-size skip/report behavior
- incremental changed-file rebuild remains correct
- no record/ref explosion

Docs:
Update:
- docs/PERFORMANCE.md
- ignore guidance
- monorepo guidance

Acceptance:
- large repo behavior is bounded
- performance metrics exist
- incremental indexing remains correct
- no silent important-file drops

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run --bin build_index`
- `cargo test --test real_repos -- --ignored`

Report:
- changed files
- performance improvements
- scale limits
- large repo guidance
- verification results
- commit hash
