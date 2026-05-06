# PLAN_57_GRAPH_INCREMENTAL_RELATIONSHIP_REBUILDS.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_56_BOUNDED_IMPORT_EXPORT_FILE_REFERENCE_HARDENING.md is complete and green.

Goal:
Make relationship rebuilds more incremental so changed-file builds do not perform unnecessary global graph/reference work.

Scope:
Improve incremental rebuild behavior for refs, dependency edges, file references, and derived relationship data.

Do not add:
- new parser architecture
- language-server integration
- package-manager execution
- network access
- MCP
- packaging
- licensing enforcement
- telemetry
- cloud behavior
- ctags production use
- JSONL canonical storage
- `WI.md`

Context:
No-change builds are fast, but changed-file runs can still trigger global relationship recomputation. This may become a scale blocker.

Product rule:
Incremental indexing must preserve correctness while avoiding unnecessary work.

Phases:
- [x] Profile changed-file builds.
- [x] Identify which relationship phases run globally.
- [x] Define invalidation model for records, refs, dependency edges, file references, and reverse lookups.
- [x] Implement incremental invalidation where safe.
- [x] Keep a safe fallback for full rebuild when schema/config/parser support changes.
- [x] Add stats showing incremental relationship work.
- [x] Add regression tests.
- [x] Update docs.
- [x] Run verification.
- [x] Commit.

Required behavior:
- Changed source file rebuilds only recompute affected relationships where practical.
- Deleted file cleanup removes dependent records/refs/dependencies/file refs.
- Renamed/moved file behavior is deterministic.
- Full relationship rebuild still happens when required by schema/config/indexer version changes.
- No stale relationship edges remain after changes.
- No duplicate relationship edges are introduced.
- No quality/comparator/real-repo workflows run in normal build.

Metrics:
Track or report:
- changed files
- parsed files
- relationship source files recomputed
- refs recomputed
- dependency edges recomputed
- file references recomputed
- stale edges removed
- total relationship phase time

Tests:
- changed file only recomputes its direct refs/deps/file refs where practical.
- deleted file removes stale edges.
- changed target file updates reverse evidence.
- config/support/schema change can force full rebuild.
- no stale refs/dependency/file-reference rows remain.
- no duplicate edges.
- no-change build performs zero parser and zero relationship recomputation.
- performance guard still passes.

Docs:
Update performance/incremental docs:
- what is incremental
- what still triggers full rebuild
- how to diagnose relationship rebuild cost

Acceptance:
- changed-file relationship rebuilds are more incremental.
- correctness is preserved.
- performance reporting shows relationship work.
- existing refs/pack/impact remain useful.
- no stale graph data is left behind.

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- performance guard if present
- `cargo run --bin build_index`
- immediate second `cargo run --bin build_index -- --stats`
- touch/edit one fixture/current source file and run `cargo run --bin build_index -- --stats`
- `cargo run --bin wi -- refs build_index`
- `cargo run --bin wi -- pack build_index`
- `cargo run --bin wi -- impact build_index`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored` if `test_repos/` exists

Commit:
Make relationship rebuilds incremental
