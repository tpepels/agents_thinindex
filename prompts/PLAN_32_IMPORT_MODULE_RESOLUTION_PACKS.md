# PLAN_32_IMPORT_MODULE_RESOLUTION_PACKS.md

Use superpowers:subagent-driven-development.

Progress:
- [x] Phase 1: inspect dependency graph foundation and existing resolver behavior
- [x] Phase 2: implement local resolver packs and confidence semantics
- [x] Phase 3: add resolver-group fixtures and integrity coverage
- [x] Phase 4: document resolver matrix, confidence, ambiguity, and gaps
- [x] Phase 5: run required verification and commit

Do not implement this until PLAN_31_DEPENDENCY_GRAPH_FOUNDATION.md is complete and green.

Goal:
Add practical local import/module resolution packs on top of the generic dependency graph.

Product rule:
Resolution should be best-effort, explicit about confidence, and never pretend to be compiler-complete.

Scope:
Implement resolution packs for major ecosystems already supported by the parser.

Resolution groups:
- Rust
- Python
- JavaScript/TypeScript/JSX/TSX
- Go
- Java/Kotlin/Scala
- C/C++
- Ruby/PHP
- Shell
- config/document formats only where meaningful

Hard requirements:
- Use existing dependency graph model.
- Keep unresolved external dependencies explicit.
- Do not invoke package managers.
- Do not require language servers.
- Do not require network access.
- Do not add telemetry, payment, licensing, or cloud behavior.
- Do not create a second graph architecture.

Resolver behavior:
- relative file imports resolve to files
- local module/package imports resolve where practical
- external dependencies remain unresolved with reason
- ambiguous matches remain explicit
- confidence labels are stored
- resolution is deterministic

Tests:
- one fixture per resolver group
- resolved local import edges
- unresolved external package edges
- ambiguous resolution behavior
- stale cleanup
- deterministic graph output
- real-repo ignored test still passes

Docs:
Document:
- resolver support matrix
- resolver confidence levels
- unresolved/ambiguous behavior
- ecosystem-specific gaps

Acceptance:
- local dependency resolution improves for major ecosystems
- unresolved edges are useful, not hidden
- refs/pack/impact remain stable
- no external runtime dependencies introduced

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run --bin build_index`
- `cargo run --bin wi -- impact build_index`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored`

Report:
- changed files
- resolver groups implemented
- confidence model
- known resolver gaps
- verification commands and results
- commit hash
