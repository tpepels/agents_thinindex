# PLAN_33_REFERENCE_GRAPH_V2.md

Use superpowers:subagent-driven-development.

Progress:
- [x] Phase 1: inspect Plan 32 dependency graph foundation and ref extraction
- [x] Phase 2: add stored ref confidence/reason schema
- [x] Phase 3: add Tree-sitter reference captures and exact-local labeling
- [x] Phase 4: add dependency-backed module refs and tests
- [x] Phase 5: document reference kinds, confidence, fallback, and limits
- [x] Phase 6: run required verification and commit

Do not implement this until PLAN_32_IMPORT_MODULE_RESOLUTION_PACKS.md is complete and green.

Goal:
Upgrade the reference graph so references use Tree-sitter captures plus dependency graph evidence.

Product rule:
References should be syntax-tree and relationship backed where possible, not broad text guesses.

Scope:
Improve refs only. Do not attempt full semantic type/call resolution.

Required:
- Use Tree-sitter definition/reference captures.
- Use dependency graph edges to connect imported/referenced symbols where practical.
- Keep broad text fallback capped and explicitly labeled if still used.
- Store reference confidence/reason.
- Keep stale ref cleanup correct.
- Keep duplicate ref constraints.
- Keep output deterministic.

Reference kinds:
- import reference
- export reference
- call reference where captured
- type reference where captured
- module/file dependency reference
- test/doc/style reference if already supported

Confidence labels:
- exact_local
- syntax
- dependency
- heuristic
- unresolved

Tests:
- references are created from Tree-sitter captures
- imports connect through dependency graph where practical
- stale refs cleaned on changed/deleted files
- duplicates prevented
- confidence labels correct
- pack/impact remain useful

Docs:
Document:
- reference kinds
- confidence labels
- known semantic limits
- difference between syntax references and semantic references

Acceptance:
- refs are materially more evidence-backed
- dependency graph contributes to references
- broad fallback is capped/labeled or removed
- existing commands remain stable

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run --bin build_index`
- `cargo run --bin wi -- refs build_index`
- `cargo run --bin wi -- pack build_index`
- `cargo run --bin wi -- impact build_index`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored`

Report:
- changed files
- reference graph changes
- confidence model
- fallback behavior
- verification commands and results
- commit hash
