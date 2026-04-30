# PLAN_34_IMPACT_V2_DEPENDENCY_AWARE.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_33_REFERENCE_GRAPH_V2.md is complete and green.

Progress:
- [x] Phase 1: inspect current impact command, reference graph, dependency graph, and Plan 33 state
- [x] Phase 2: implement dependency-aware grouped impact evidence and confidence labels
- [x] Phase 3: add test/build/config mapping, deterministic ordering, limits, and deduplication
- [x] Phase 4: update fixture tests for dependency edges, tests, confidence, determinism, duplicates, and stale dependency changes
- [x] Phase 5: document impact v2 behavior and run required verification
- [x] Phase 6: commit completed Plan 34 work

Goal:
Upgrade `wi impact` from symbol-evidence hints to dependency-aware impact analysis.

Product rule:
Impact output must distinguish evidence strength. Do not pretend best-effort analysis is complete semantic truth.

Required:
- Use parser records, refs, dependency graph, and optional semantic facts.
- Add confidence labels:
  - direct
  - dependency
  - test-related
  - semantic if adapter supplied it
  - heuristic
- Add reason strings for every impacted file.
- Add test-file mapping.
- Add build/config-file mapping where practical.
- Keep output deterministic.
- Do not add network access or external tool requirements.

Test mapping:
Support:
- same-name test conventions
- test directory conventions
- language/package conventions where practical
- existing manifest expected test mappings if present

CLI behavior:
- `wi impact <symbol>` should show grouped impact:
  - direct definitions
  - references
  - dependent files
  - likely tests
  - unresolved/unknown areas
- Keep concise default output.
- Add verbose mode if already supported.

Tests:
- fixture impact includes dependency edges
- fixture impact includes tests
- confidence labels are correct
- impact remains deterministic
- no duplicate impacted files
- stale dependency changes update impact

Docs:
Document:
- impact is evidence-backed, not exhaustive
- confidence levels
- how to improve impact quality with manifests/semantic adapters

Acceptance:
- impact v2 uses dependency graph
- impacted files include reasons/confidence
- tests are mapped where practical
- output remains stable and compact

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
- impact evidence model
- test mapping behavior
- sample impact output
- verification results
- commit hash
