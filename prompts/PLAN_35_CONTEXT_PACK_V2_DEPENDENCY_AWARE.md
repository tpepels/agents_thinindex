# PLAN_35_CONTEXT_PACK_V2_DEPENDENCY_AWARE.md

Use superpowers:subagent-driven-development.

Do not implement this until PLAN_34_IMPACT_V2_DEPENDENCY_AWARE.md is complete and green.

Progress:
- [x] Phase 1: inspect Plan 34 impact evidence and existing pack behavior
- [x] Phase 2: implement dependency-aware pack groups, confidence labels, and deterministic caps
- [x] Phase 3: add dependency/dependent/test/config/docs/unresolved evidence selection and deduplication
- [x] Phase 4: update pack tests for grouping, confidence, limits, determinism, duplicates, compactness, and ctags independence
- [x] Phase 5: document context pack model and run required verification
- [x] Phase 6: commit completed Plan 35 work

Goal:
Upgrade `wi pack` to produce better bounded read sets using dependency graph and impact evidence.

Product rule:
A context pack should give agents the smallest useful set of files, with reasons and confidence.

Required:
- Use records, refs, dependency graph, and impact evidence.
- Group pack entries by reason.
- Include confidence labels.
- Cap output deterministically.
- Prefer high-signal files over noisy broad matches.
- Include tests/config files where relevant.
- Include unresolved/unknown hints where useful.
- Do not dump huge file contents.

Pack groups:
- primary definitions
- direct references
- dependencies
- dependents
- tests
- configs/build files
- docs/examples if relevant
- unresolved hints

Ranking:
Improve ranking with:
- exact symbol matches
- local dependency proximity
- reference count
- file role
- test/source pairing
- support confidence
- quality manifest hints if available

Tests:
- pack includes dependency-related files
- pack includes likely tests
- pack explains why entries are included
- cap behavior deterministic
- no duplicate files
- output remains compact
- quality does not depend on ctags

Docs:
Document:
- pack groups
- ranking model
- caps
- how agents should use pack output

Acceptance:
- `wi pack` is dependency-aware
- pack output is reasoned and bounded
- existing search/impact behavior remains stable
- output remains agent-readable

Verification:
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run --bin build_index`
- `cargo run --bin wi -- pack build_index`
- `cargo run --bin wi -- impact build_index`
- `cargo test --test local_index -- --ignored`
- `cargo test --test real_repos -- --ignored`

Report:
- changed files
- pack v2 ranking/evidence model
- sample output
- verification commands and results
- commit hash
